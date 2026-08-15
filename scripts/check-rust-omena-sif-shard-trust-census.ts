import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

type CensusRole = "producer" | "consumer";

type CensusRule = {
  id: string;
  role: CensusRole;
  call: string;
  lockKnowledge: string;
};

type CensusRow = {
  id: string;
  role: CensusRole;
  sourcePath: string;
  symbol: string;
  line: number;
  call: string;
  lockKnowledge: string;
};

const repoRoot = process.cwd();
const args = new Set(process.argv.slice(2).filter((arg) => arg !== "--"));
const injectNewStoreCall = args.has("--inject-new-store-call");
const injectCrossWorkspaceSharingPath = args.has("--inject-cross-workspace-sharing-path");
assert.deepEqual(
  [...args].filter(
    (arg) =>
      arg.startsWith("--") &&
      arg !== "--inject-new-store-call" &&
      arg !== "--inject-cross-workspace-sharing-path",
  ),
  [],
  "unknown SIF shard trust census option",
);

const rules: readonly CensusRule[] = [
  {
    id: "cli-sif-artifact-writer",
    role: "producer",
    call: "fs::write(&output_path, &sif_json)",
    lockKnowledge: "canonical-url+sif-hash-derivable; no-recorded-verdict",
  },
  {
    id: "cli-lock-update-writer",
    role: "producer",
    call: "fs::write(&lockfile, &lock_json)",
    lockKnowledge: "lock-entry+default-tier; no-attestation-verdict",
  },
  {
    id: "cli-recorded-verdict-writer",
    role: "producer",
    call: "write_recorded_shard_verdicts(",
    lockKnowledge: "verified-lock-entry+canonical-sif-hash+attestation-reference",
  },
  {
    id: "bridge-shard-store",
    role: "producer",
    call: "store_external_sif_cache_shard(",
    lockKnowledge: "recorded-verdict-sidecar-or-local-t1; no-lock-reader",
  },
  {
    id: "bridge-shard-load",
    role: "consumer",
    call: "load_external_sif_cache_shard(",
    lockKnowledge: "payload-digest+lock-binding+recorded-verdict-sidecar",
  },
  {
    id: "query-resolved-style-shard-consumer",
    role: "consumer",
    call: "generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(",
    lockKnowledge: "validated-tier-record+partitioned-cache; no-lock-reader",
  },
  {
    id: "lsp-in-process-style-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(",
    lockKnowledge: "recorded-verdict-tier+partitioned-cache; no-verifier",
  },
  {
    id: "lsp-refresh-style-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(",
    lockKnowledge: "recorded-verdict-tier+partitioned-cache; no-verifier",
  },
  {
    id: "lsp-seed-pair-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_cache_storage_and_trust(",
    lockKnowledge: "recorded-verdict-tier+workspace-owner; no-verifier",
  },
];

const expectedKeys = [
  "cli-sif-artifact-writer|rust/crates/omena-cli/src/sif.rs|generate_sif",
  "cli-lock-update-writer|rust/crates/omena-cli/src/lock.rs|lock_update",
  "cli-recorded-verdict-writer|rust/crates/omena-cli/src/lock.rs|lock_record_verification",
  "cli-recorded-verdict-writer|rust/crates/omena-cli/src/lock.rs|lock_verify_attestation",
  "bridge-shard-store|rust/crates/omena-bridge/src/style_resolution.rs|generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust",
  "bridge-shard-load|rust/crates/omena-bridge/src/style_resolution.rs|generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust",
  "query-resolved-style-shard-consumer|rust/crates/omena-query/src/source.rs|enqueue_alias",
  "lsp-in-process-style-shard-consumer|rust/crates/omena-lsp-server/src/external_sif_loader.rs|resolve_in_process_external_sifs_for_lsp",
  "lsp-refresh-style-shard-consumer|rust/crates/omena-lsp-server/src/external_sif_loader.rs|resolve_external_sifs_for_refresh_documents",
  "lsp-seed-pair-shard-consumer|rust/crates/omena-lsp-server/src/external_sif_loader.rs|resolve_bridge_external_sifs_for_sources",
].toSorted();

const trackedRustSources = execFileSync("git", ["ls-files", "-z", "--", "*.rs"], {
  cwd: repoRoot,
  encoding: "utf8",
})
  .split("\0")
  .filter(Boolean)
  .filter(isProductionRustSource);

const bridgeSourcePath = "rust/crates/omena-bridge/src/style_resolution.rs";
const rows = scanCensus(injectNewStoreCall);
const observedKeys = rows.map(rowKey).toSorted();
assert.deepEqual(
  observedKeys,
  expectedKeys,
  `SIF shard trust call-site census drifted:\n${formatRows(rows)}`,
);

const bridgeProduction = productionRustPrefix(
  fs.readFileSync(path.join(repoRoot, bridgeSourcePath), "utf8"),
);
const bridgeCargo = fs.readFileSync(
  path.join(repoRoot, "rust/crates/omena-bridge/Cargo.toml"),
  "utf8",
);
const lspBoundarySource = fs.readFileSync(
  path.join(repoRoot, "rust/crates/omena-lsp-server/src/boundary.rs"),
  "utf8",
);
const sassCompatibilitySource = fs.readFileSync(path.join(repoRoot, "docs/sass-compat.md"), "utf8");
const bridgeSource = fs.readFileSync(path.join(repoRoot, bridgeSourcePath), "utf8");
const bridgeCacheRootSource = fs.readFileSync(
  path.join(repoRoot, "rust/crates/omena-bridge/src/cache_root.rs"),
  "utf8",
);
const crossWorkspaceSharingSites = scanCrossWorkspaceSharingSites(injectCrossWorkspaceSharingPath);
assert.ok(
  !/(?:omena\.lock|lock_entry|lockfile|sigstore[_-]verify)/iu.test(bridgeProduction),
  "omena-bridge production code must consume recorded verdicts rather than lock or Sigstore authority",
);
assert.ok(
  !/sigstore[_-]verify/iu.test(bridgeCargo),
  "omena-bridge must not acquire a Sigstore verification dependency",
);
assert.ok(
  lspBoundarySource.includes("recordedShardVerdictsConsumedWithoutLockOrNetworkAuthority"),
  "the LSP boundary must name verdict-only shard trust consumption",
);
assert.ok(
  sassCompatibilitySource.includes(
    "Bridge and LSP\nconsumers read that local verdict sidecar; they do not treat a lockfile as\nshard-verification authority",
  ),
  "the Sass compatibility contract must document verdict-only shard trust consumption",
);
assert.deepEqual(
  crossWorkspaceSharingSites,
  [],
  `cross-workspace external-SIF sharing path exists:\n${crossWorkspaceSharingSites.join("\n")}`,
);
assert.equal(
  countOccurrences(bridgeCacheRootSource, "workspace: scoped_workspace_root("),
  3,
  "initialization, environment, and platform cache roots must all scope workspace storage",
);
assert.ok(
  !bridgeCacheRootSource.includes("workspace: Some(global"),
  "a global cache root must never become an unscoped external-SIF workspace root",
);
for (const standingInvariant of [
  "fn global_external_sif_storage_never_cross_serves_workspace_partitions()",
  "assert_ne!(workspace_cache_root_a, workspace_cache_root_b)",
  "assert_ne!(shard_a, shard_b)",
  "crossWorkspaceServe=false",
]) {
  assert.ok(
    bridgeSource.includes(standingInvariant),
    `the standing cross-workspace partition arm lost ${standingInvariant}`,
  );
}

if (!injectNewStoreCall) {
  const injectedRows = scanCensus(true);
  assert.equal(
    injectedRows.filter((row) => row.id === "bridge-shard-store").length,
    rows.filter((row) => row.id === "bridge-shard-store").length + 1,
    "the structural falsifier must expose an added production shard-store site",
  );
}
if (!injectCrossWorkspaceSharingPath) {
  assert.equal(
    scanCrossWorkspaceSharingSites(true).length,
    1,
    "the cross-workspace sharing falsifier must expose one identityless production storage path",
  );
}

// FALSIFIER: id=sif-shard-trust-census-new-store-call class=structuralEntailment via=--inject-new-store-call producer=can-fail owner=sif-shard-trust-census entry=git-ls-files-rust-production-shard-callset
// FALSIFIER: id=sif-shard-no-cross-workspace-sharing class=productionCallSiteMutation via=--inject-cross-workspace-sharing-path expected=RED owner=sif-shard-trust-census entry=git-ls-files-rust-production-sharing-callset
process.stdout.write(
  `SIF shard trust census OK: rows=${rows.length} producers=${rows.filter((row) => row.role === "producer").length} consumers=${rows.filter((row) => row.role === "consumer").length} bridgeLockReaders=0 bridgeSigstoreDependencies=0 crossWorkspaceSharingPaths=0 workspaceScopedGlobalBranches=3 crossWorkspaceServe=false\n${formatRows(rows)}\n`,
);

function scanCensus(injectStore: boolean): CensusRow[] {
  const observed: CensusRow[] = [];
  for (const sourcePath of trackedRustSources) {
    let source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
    if (injectStore && sourcePath === bridgeSourcePath) {
      const injection =
        "\nfn injected_external_sif_shard_store_site() { store_external_sif_cache_shard(); }\n";
      const testModule = source.search(/\n#\[cfg\(test\)\]\s*\nmod\s+/u);
      source =
        testModule < 0
          ? source + injection
          : source.slice(0, testModule) + injection + source.slice(testModule);
    }
    const production = productionRustPrefix(source);
    let containingFunction = "<module>";
    for (const [lineIndex, line] of production.split(/\r?\n/u).entries()) {
      const functionMatch = line.match(
        /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/u,
      );
      if (functionMatch?.[1]) containingFunction = functionMatch[1];
      for (const rule of rules) {
        if (!line.includes(rule.call)) continue;
        if (line.includes(`fn ${rule.call}`)) continue;
        if (!ruleAppliesToSite(rule.id, sourcePath, containingFunction)) continue;
        observed.push({
          ...rule,
          sourcePath,
          symbol: containingFunction,
          line: lineIndex + 1,
        });
      }
    }
  }
  return observed.toSorted((left, right) => {
    const leftKey = rowKey(left);
    const rightKey = rowKey(right);
    return leftKey < rightKey ? -1 : leftKey > rightKey ? 1 : 0;
  });
}

function scanCrossWorkspaceSharingSites(injectSharingPath: boolean): string[] {
  const sites: string[] = [];
  for (const sourcePath of trackedRustSources) {
    let source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
    if (injectSharingPath && sourcePath === bridgeSourcePath) {
      const injection =
        "\nfn injected_cross_workspace_sharing_path() { let _ = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(PathBuf::new()); }\n";
      const testModule = source.search(/\n#\[cfg\(test\)\]\s*\nmod\s+/u);
      source =
        testModule < 0
          ? source + injection
          : source.slice(0, testModule) + injection + source.slice(testModule);
    }
    const production = productionRustPrefix(source);
    for (const [lineIndex, line] of production.split(/\r?\n/u).entries()) {
      if (
        line.includes("OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(") ||
        line.includes("OmenaQueryExternalSifStorageV0::from_workspace_cache_root(") ||
        /(?:allow|enable|serve|share)_cross_workspace_(?:external_)?sif/iu.test(line) ||
        line.includes("crossWorkspaceServe=true")
      ) {
        sites.push(`${sourcePath}:${lineIndex + 1}:${line.trim()}`);
      }
    }
  }
  return sites.toSorted();
}

function countOccurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}

function ruleAppliesToSite(ruleId: string, sourcePath: string, symbol: string): boolean {
  if (ruleId === "cli-sif-artifact-writer") return symbol === "generate_sif";
  if (ruleId === "cli-lock-update-writer") return symbol === "lock_update";
  if (ruleId === "query-resolved-style-shard-consumer") {
    return sourcePath === "rust/crates/omena-query/src/source.rs";
  }
  if (ruleId === "lsp-in-process-style-shard-consumer") {
    return symbol === "resolve_in_process_external_sifs_for_lsp";
  }
  if (ruleId === "lsp-refresh-style-shard-consumer") {
    return symbol === "resolve_external_sifs_for_refresh_documents";
  }
  if (ruleId === "lsp-seed-pair-shard-consumer") {
    return (
      sourcePath === "rust/crates/omena-lsp-server/src/external_sif_loader.rs" &&
      symbol === "resolve_bridge_external_sifs_for_sources"
    );
  }
  return true;
}

function rowKey(row: Pick<CensusRow, "id" | "sourcePath" | "symbol">): string {
  return `${row.id}|${row.sourcePath}|${row.symbol}`;
}

function formatRows(observed: readonly CensusRow[]): string {
  return observed
    .map(
      (row) =>
        `${row.role}\t${row.id}\t${row.sourcePath}:${row.line}\t${row.symbol}\t${row.lockKnowledge}`,
    )
    .join("\n");
}

function isProductionRustSource(sourcePath: string): boolean {
  return (
    sourcePath.endsWith(".rs") &&
    !/(?:^|\/)tests?(?:\/|\.rs$)/u.test(sourcePath) &&
    !sourcePath.endsWith("_test.rs")
  );
}

function productionRustPrefix(source: string): string {
  return source.split(/\n#\[cfg\(test\)\]\s*\nmod\s+/u)[0] ?? source;
}

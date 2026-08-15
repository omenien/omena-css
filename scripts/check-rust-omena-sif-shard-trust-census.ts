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
const args = new Set(process.argv.slice(2));
const injectNewStoreCall = args.has("--inject-new-store-call");
assert.deepEqual(
  [...args].filter((arg) => arg.startsWith("--") && arg !== "--inject-new-store-call"),
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
    id: "bridge-shard-store",
    role: "producer",
    call: "store_external_sif_cache_shard(",
    lockKnowledge: "freshness-fingerprint-only; no-lock-or-verdict",
  },
  {
    id: "bridge-shard-load",
    role: "consumer",
    call: "load_external_sif_cache_shard(",
    lockKnowledge: "payload-digest+self-identity-only; no-recorded-verdict",
  },
  {
    id: "query-resolved-style-shard-consumer",
    role: "consumer",
    call: "generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(",
    lockKnowledge: "freshness-fingerprint+partitioned-cache; no-recorded-verdict",
  },
  {
    id: "lsp-in-process-style-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage(",
    lockKnowledge: "lock-sif-inputs+partitioned-cache; no-verified-verdict",
  },
  {
    id: "lsp-refresh-style-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage(",
    lockKnowledge: "lock-sif-inputs+partitioned-cache; no-verified-verdict",
  },
  {
    id: "lsp-seed-pair-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_cache_storage(",
    lockKnowledge: "workspace-owner+partitioned-cache; no-verified-verdict",
  },
];

const expectedKeys = [
  "cli-sif-artifact-writer|rust/crates/omena-cli/src/sif.rs|generate_sif",
  "cli-lock-update-writer|rust/crates/omena-cli/src/lock.rs|lock_update",
  "bridge-shard-store|rust/crates/omena-bridge/src/style_resolution.rs|generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage",
  "bridge-shard-load|rust/crates/omena-bridge/src/style_resolution.rs|generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage",
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
assert.ok(
  !/(?:omena\.lock|lock_entry|lockfile|sigstore[_-]verify)/iu.test(bridgeProduction),
  "omena-bridge production code must consume recorded verdicts rather than lock or Sigstore authority",
);
assert.ok(
  !/sigstore[_-]verify/iu.test(bridgeCargo),
  "omena-bridge must not acquire a Sigstore verification dependency",
);

if (!injectNewStoreCall) {
  const injectedRows = scanCensus(true);
  assert.equal(
    injectedRows.filter((row) => row.id === "bridge-shard-store").length,
    rows.filter((row) => row.id === "bridge-shard-store").length + 1,
    "the structural falsifier must expose an added production shard-store site",
  );
}

// FALSIFIER: id=sif-shard-trust-census-new-store-call class=structuralEntailment via=--inject-new-store-call producer=can-fail owner=sif-shard-trust-census entry=git-ls-files-rust-production-shard-callset
process.stdout.write(
  `SIF shard trust census OK: rows=${rows.length} producers=${rows.filter((row) => row.role === "producer").length} consumers=${rows.filter((row) => row.role === "consumer").length} bridgeLockReaders=0 bridgeSigstoreDependencies=0\n${formatRows(rows)}\n`,
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

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

type CensusRole = "artifact-input" | "trust-writer" | "shard-writer" | "consumer";

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
const injectDirectShardPrimitive = args.has("--inject-direct-shard-primitive");
const injectCrossWorkspaceSharingPath = args.has("--inject-cross-workspace-sharing-path");
const injectStructLiteralSharingPath = args.has("--inject-struct-literal-sharing-path");
const injectConstantIdentitySharingPath = args.has("--inject-constant-identity-sharing-path");
const injectDocCommentCfgDecoy = args.has("--inject-doc-comment-cfg-decoy");
assert.deepEqual(
  [...args].filter(
    (arg) =>
      arg.startsWith("--") &&
      arg !== "--inject-new-store-call" &&
      arg !== "--inject-direct-shard-primitive" &&
      arg !== "--inject-cross-workspace-sharing-path" &&
      arg !== "--inject-struct-literal-sharing-path" &&
      arg !== "--inject-constant-identity-sharing-path" &&
      arg !== "--inject-doc-comment-cfg-decoy",
  ),
  [],
  "unknown SIF shard trust census option",
);

const rules: readonly CensusRule[] = [
  {
    id: "cli-sif-artifact-writer",
    role: "artifact-input",
    call: "fs::write(&output_path, &sif_json)",
    lockKnowledge: "canonical-url+sif-hash-derivable; no-recorded-verdict",
  },
  {
    id: "cli-lock-update-writer",
    role: "artifact-input",
    call: "fs::write(&lockfile, &lock_json)",
    lockKnowledge: "lock-entry+default-tier; no-attestation-verdict",
  },
  {
    id: "cli-recorded-verdict-writer",
    role: "trust-writer",
    call: "write_recorded_shard_verdicts(",
    lockKnowledge: "verified-lock-entry+canonical-sif-hash+content-addressed-bundle",
  },
  {
    id: "cli-recorded-bundle-writer",
    role: "trust-writer",
    call: "write_recorded_sigstore_bundle(",
    lockKnowledge: "verified-sif+offline-sigstore-bundle",
  },
  {
    id: "cli-explicit-lock-sif-reader",
    role: "consumer",
    call: "read_omena_lock_json_v1(",
    lockKnowledge: "explicit-user-input+sifHash-check; fallback-behind-local-bridge",
  },
  {
    id: "bridge-shard-store",
    role: "shard-writer",
    call: "store_external_sif_cache_shard(",
    lockKnowledge: "recorded-verdict-sidecar-or-local-t1; no-lock-reader",
  },
  {
    id: "bridge-shard-atomic-write",
    role: "shard-writer",
    call: "write_external_sif_cache_shard_atomically(",
    lockKnowledge: "enumerated-disk-writer-under-validated-shard-store",
  },
  {
    id: "bridge-shard-load",
    role: "consumer",
    call: "load_external_sif_cache_shard(",
    lockKnowledge: "local-regeneration-equality+payload-digest+offline-published-subject",
  },
  {
    id: "query-resolved-style-shard-consumer",
    role: "consumer",
    call: "generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(",
    lockKnowledge: "offline-verified-advisory-tier+partitioned-cache; no-lock-reader",
  },
  {
    id: "query-published-style-shard-consumer",
    role: "consumer",
    call: "generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_cache_context_storage_and_trust(",
    lockKnowledge: "published-url+offline-verified-advisory-tier+partitioned-cache",
  },
  {
    id: "lsp-in-process-style-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(",
    lockKnowledge: "bridge-verified-advisory-tier+partitioned-cache; no-lock-reader",
  },
  {
    id: "lsp-refresh-style-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(",
    lockKnowledge: "bridge-verified-advisory-tier+partitioned-cache; no-lock-reader",
  },
  {
    id: "lsp-seed-pair-shard-consumer",
    role: "consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_cache_storage_and_trust(",
    lockKnowledge: "bridge-verified-advisory-tier+workspace-owner; no-lock-reader",
  },
];

const expectedKeys = [
  "cli-sif-artifact-writer|rust/crates/omena-cli/src/sif.rs|generate_sif",
  "cli-lock-update-writer|rust/crates/omena-cli/src/lock.rs|lock_update",
  "cli-recorded-verdict-writer|rust/crates/omena-cli/src/lock.rs|lock_verify_attestation",
  "cli-recorded-bundle-writer|rust/crates/omena-cli/src/lock.rs|lock_verify_attestation",
  "cli-explicit-lock-sif-reader|rust/crates/omena-cli/src/diagnostics.rs|read_lock_external_sifs",
  "bridge-shard-store|rust/crates/omena-bridge/src/style_resolution.rs|generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_impl",
  "bridge-shard-atomic-write|rust/crates/omena-bridge/src/style_resolution.rs|store_external_sif_cache_shard",
  "bridge-shard-load|rust/crates/omena-bridge/src/style_resolution.rs|generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_impl",
  "query-resolved-style-shard-consumer|rust/crates/omena-query/src/source.rs|enqueue_alias",
  "query-published-style-shard-consumer|rust/crates/omena-query/src/source.rs|enqueue_alias",
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
const rows = scanCensus({
  injectStore: injectNewStoreCall,
  injectPrimitive: injectDirectShardPrimitive,
  injectDocCommentDecoy: injectDocCommentCfgDecoy,
});
const observedKeys = rows.map(rowKey).toSorted();
assert.deepEqual(
  observedKeys,
  expectedKeys,
  `SIF shard trust call-site census drifted:\n${formatRows(rows)}`,
);

const bridgeProductionSources = trackedRustSources
  .filter((sourcePath) => sourcePath.startsWith("rust/crates/omena-bridge/src/"))
  .map((sourcePath) => ({
    sourcePath,
    source: maskCfgTestItems(fs.readFileSync(path.join(repoRoot, sourcePath), "utf8")),
  }));
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
const bridgeLockReaderSites = scanBridgeLockReaderSites(bridgeProductionSources);
const bridgeNetworkSites = scanBridgeNetworkSites(bridgeProductionSources);
const bridgeSigstoreVerifierCalls = bridgeProductionSources.flatMap(({ sourcePath, source }) =>
  source
    .split(/\r?\n/u)
    .flatMap((line, index) =>
      line.includes("sigstore_verify::verify(") ? [`${sourcePath}:${index + 1}`] : [],
    ),
);
const crossWorkspaceSharingSites = scanCrossWorkspaceSharingSites({
  injectUnscopedConstructor: injectCrossWorkspaceSharingPath,
  injectStructLiteral: injectStructLiteralSharingPath,
  injectDocCommentDecoy: injectDocCommentCfgDecoy,
});
const workspaceIdentityConstructorSites = scanWorkspaceIdentityConstructorSites(
  injectConstantIdentitySharingPath,
);
const expectedWorkspaceIdentityConstructorKeys = [
  "rust/crates/omena-lsp-server/src/external_sif_loader.rs|bridge_cache_storage_for_workspace_uri",
  "rust/crates/omena-lsp-server/src/external_sif_loader.rs|bridge_cache_storage_for_document",
].toSorted();
assert.deepEqual(bridgeLockReaderSites, [], "omena-bridge must not read omena.lock");
assert.deepEqual(
  bridgeNetworkSites,
  [],
  "omena-bridge offline verification must not use a network client",
);
assert.equal(
  bridgeSigstoreVerifierCalls.length,
  1,
  "omena-bridge must have exactly one native offline Sigstore verification call",
);
assert.match(
  bridgeCargo,
  /\[target\.'cfg\(not\(target_family = "wasm"\)\)'\.dependencies\][\s\S]*sigstore-verify/u,
  "Sigstore verification must remain a native-only bridge dependency",
);
assert.ok(
  lspBoundarySource.includes("workspaceLockBytesAreNotReadOrAdmittedAutomatically"),
  "the LSP boundary must name the automatic workspace-lock exclusion",
);
assert.ok(
  sassCompatibilitySource.includes("The automatic LSP path does not read workspace lock bytes"),
  "the Sass compatibility contract must document the no-read automatic lock boundary",
);
assert.deepEqual(
  crossWorkspaceSharingSites,
  [],
  `cross-workspace external-SIF sharing path exists:\n${crossWorkspaceSharingSites.join("\n")}`,
);
assert.deepEqual(
  workspaceIdentityConstructorSites.map((site) => site.key).toSorted(),
  expectedWorkspaceIdentityConstructorKeys,
  `workspace-identity constructor call-set drifted:\n${workspaceIdentityConstructorSites
    .map((site) => site.display)
    .join("\n")}`,
);
const workspaceScopedGlobalBranches = countOccurrences(
  bridgeCacheRootSource,
  "workspace: scoped_workspace_root(",
);
assert.equal(
  workspaceScopedGlobalBranches,
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
  const injectedRows = scanCensus({ injectStore: true });
  assert.equal(
    injectedRows.filter((row) => row.id === "bridge-shard-store").length,
    rows.filter((row) => row.id === "bridge-shard-store").length + 1,
    "the structural falsifier must expose an added production shard-store site",
  );
}
if (!injectDirectShardPrimitive) {
  const injectedRows = scanCensus({ injectPrimitive: true });
  assert.equal(
    injectedRows.filter((row) => row.id === "bridge-shard-atomic-write").length,
    rows.filter((row) => row.id === "bridge-shard-atomic-write").length + 1,
    "the primitive-writer falsifier must expose a direct production disk writer",
  );
}
if (!injectCrossWorkspaceSharingPath) {
  assert.equal(
    scanCrossWorkspaceSharingSites({ injectUnscopedConstructor: true }).length,
    1,
    "the cross-workspace sharing falsifier must expose one identityless production storage path",
  );
}
if (!injectStructLiteralSharingPath) {
  assert.equal(
    scanCrossWorkspaceSharingSites({ injectStructLiteral: true }).length,
    3,
    "the struct-literal falsifier must expose assigned, returned, and tail-expression storage paths",
  );
}
if (!injectConstantIdentitySharingPath) {
  assert.equal(
    scanWorkspaceIdentityConstructorSites(true).length,
    workspaceIdentityConstructorSites.length + 1,
    "the constant-identity falsifier must expose one new identity constructor site",
  );
}
if (!injectDocCommentCfgDecoy) {
  const decoyRows = scanCensus({ injectDocCommentDecoy: true });
  assert.equal(
    decoyRows.filter((row) => row.id === "bridge-shard-store").length,
    rows.filter((row) => row.id === "bridge-shard-store").length + 1,
    "a doc-comment cfg(test) decoy must not hide a production shard writer",
  );
  assert.equal(
    scanCrossWorkspaceSharingSites({ injectDocCommentDecoy: true }).length,
    1,
    "a doc-comment cfg(test) decoy must not hide an identityless storage constructor",
  );
}

// FALSIFIER: id=sif-shard-trust-census-post-test-store-call class=structuralEntailment via=--inject-new-store-call producer=can-fail owner=sif-shard-trust-census entry=git-ls-files-rust-production-shard-callset
// FALSIFIER: id=sif-shard-trust-census-direct-primitive-writer class=structuralEntailment via=--inject-direct-shard-primitive expected=RED owner=sif-shard-trust-census entry=enumerated-disk-writer-callset
// FALSIFIER: id=sif-shard-no-cross-workspace-sharing class=productionCallSiteMutation via=--inject-cross-workspace-sharing-path expected=RED owner=sif-shard-trust-census entry=git-ls-files-rust-production-sharing-callset
// FALSIFIER: id=sif-shard-no-struct-literal-sharing class=productionCallSiteMutation via=--inject-struct-literal-sharing-path expected=RED owner=sif-shard-trust-census entry=enumerated-storage-constructor-callset
// FALSIFIER: id=sif-shard-workspace-identity-callset class=productionCallSiteMutation via=--inject-constant-identity-sharing-path expected=RED owner=sif-shard-trust-census entry=enumerated-storage-constructor-callset
// FALSIFIER: id=sif-shard-cfg-comment-decoy class=lexicalMaskMutation via=--inject-doc-comment-cfg-decoy expected=RED owner=sif-shard-trust-census entry=comment-aware-cfg-test-mask
process.stdout.write(
  `SIF shard trust census OK: scope=git-ls-files-comment-aware-production-rust coverage=enumerated-writer-consumer-and-storage-constructor-callsets rows=${rows.length} artifactInputs=${rows.filter((row) => row.role === "artifact-input").length} trustWriters=${rows.filter((row) => row.role === "trust-writer").length} shardWriters=${rows.filter((row) => row.role === "shard-writer").length} consumers=${rows.filter((row) => row.role === "consumer").length} bridgeLockReaders=${bridgeLockReaderSites.length} bridgeNetworkSites=${bridgeNetworkSites.length} bridgeSigstoreVerifierCalls=${bridgeSigstoreVerifierCalls.length} crossWorkspaceSharingPaths=0 workspaceIdentityConstructorSites=${workspaceIdentityConstructorSites.length} workspaceScopedGlobalBranches=${workspaceScopedGlobalBranches} crossWorkspaceServe=false\n${formatRows(rows)}\n`,
);

type CensusInjection = {
  injectStore?: boolean;
  injectPrimitive?: boolean;
  injectDocCommentDecoy?: boolean;
};

function scanCensus(injection: CensusInjection = {}): CensusRow[] {
  const observed: CensusRow[] = [];
  for (const sourcePath of trackedRustSources) {
    let source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
    if (injection.injectStore && sourcePath === bridgeSourcePath) {
      const injectedSource =
        "\nfn injected_external_sif_shard_store_site() { store_external_sif_cache_shard(); }\n";
      source += injectedSource;
    }
    if (injection.injectPrimitive && sourcePath === bridgeSourcePath) {
      source +=
        "\nfn injected_direct_shard_primitive() { write_external_sif_cache_shard_atomically(); }\n";
    }
    if (injection.injectDocCommentDecoy && sourcePath === bridgeSourcePath) {
      source +=
        "\n/// #[cfg(test)]\nfn injected_cfg_comment_decoy() { store_external_sif_cache_shard(); let _ = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(PathBuf::new()); }\n";
    }
    const production = maskCfgTestItems(source);
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

type SharingInjection = {
  injectUnscopedConstructor?: boolean;
  injectStructLiteral?: boolean;
  injectDocCommentDecoy?: boolean;
};

function scanCrossWorkspaceSharingSites(injection: SharingInjection = {}): string[] {
  const sites: string[] = [];
  for (const sourcePath of trackedRustSources) {
    let source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
    if (injection.injectUnscopedConstructor && sourcePath === bridgeSourcePath) {
      const injectedSource =
        "\nfn injected_cross_workspace_sharing_path() { let _ = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(PathBuf::new()); }\n";
      source += injectedSource;
    }
    if (injection.injectStructLiteral && sourcePath === bridgeSourcePath) {
      source +=
        "\nfn injected_assigned_struct_literal_sharing_path() { let _storage = OmenaBridgeExternalSifStorageV0 { workspace_cache_root: PathBuf::new(), workspace_identity: None, recorded_verdict_dir: None }; }\nfn injected_returned_struct_literal_sharing_path() -> OmenaBridgeExternalSifStorageV0 { return OmenaBridgeExternalSifStorageV0\n{ workspace_cache_root: PathBuf::new(), workspace_identity: None, recorded_verdict_dir: None }; }\nfn injected_tail_struct_literal_sharing_path() -> OmenaBridgeExternalSifStorageV0 { OmenaBridgeExternalSifStorageV0\n{ workspace_cache_root: PathBuf::new(), workspace_identity: None, recorded_verdict_dir: None } }\n";
    }
    if (injection.injectDocCommentDecoy && sourcePath === bridgeSourcePath) {
      source +=
        "\n/// #[cfg(test)]\nfn injected_cfg_comment_decoy() { store_external_sif_cache_shard(); let _ = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(PathBuf::new()); }\n";
    }
    const production = maskCfgTestItems(source);
    const lexicalProduction = maskRustCommentsAndLiterals(production);
    const structLiteralLines = new Set<number>();
    for (const match of lexicalProduction.matchAll(
      /\bOmena(?:Bridge|Query)ExternalSifStorageV0\s*\{/gu,
    )) {
      const prefix = lexicalProduction.slice(Math.max(0, match.index - 32), match.index);
      if (/\b(?:struct|impl)\s*$/u.test(prefix)) continue;
      structLiteralLines.add(lexicalProduction.slice(0, match.index).split("\n").length - 1);
    }
    for (const [lineIndex, line] of production.split(/\r?\n/u).entries()) {
      if (
        line.includes("OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(") ||
        line.includes("OmenaQueryExternalSifStorageV0::from_workspace_cache_root(") ||
        structLiteralLines.has(lineIndex) ||
        /(?:allow|enable|serve|share)_cross_workspace_(?:external_)?sif/iu.test(line) ||
        line.includes("crossWorkspaceServe=true")
      ) {
        sites.push(`${sourcePath}:${lineIndex + 1}:${line.trim()}`);
      }
    }
  }
  return sites.toSorted();
}

type WorkspaceIdentityConstructorSite = { key: string; display: string };

function scanWorkspaceIdentityConstructorSites(
  injectConstantIdentity: boolean,
): WorkspaceIdentityConstructorSite[] {
  const sites: WorkspaceIdentityConstructorSite[] = [];
  for (const sourcePath of trackedRustSources) {
    let source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
    if (injectConstantIdentity && sourcePath === bridgeSourcePath) {
      source +=
        '\nfn injected_constant_workspace_identity() { let _ = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root_and_identity(PathBuf::new(), "global"); }\n';
    }
    const production = maskCfgTestItems(source);
    let containingFunction = "<module>";
    for (const [lineIndex, line] of production.split(/\r?\n/u).entries()) {
      const functionMatch = line.match(
        /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/u,
      );
      if (functionMatch?.[1]) containingFunction = functionMatch[1];
      if (!line.includes("from_workspace_cache_root_and_identity(")) continue;
      if (line.includes("fn from_workspace_cache_root_and_identity(")) continue;
      sites.push({
        key: `${sourcePath}|${containingFunction}`,
        display: `${sourcePath}:${lineIndex + 1}:${containingFunction}:${line.trim()}`,
      });
    }
  }
  return sites.toSorted((left, right) =>
    left.display < right.display ? -1 : left.display > right.display ? 1 : 0,
  );
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
  if (ruleId === "cli-explicit-lock-sif-reader") {
    return (
      sourcePath === "rust/crates/omena-cli/src/diagnostics.rs" &&
      symbol === "read_lock_external_sifs"
    );
  }
  if (ruleId === "lsp-lock-hint-reader") {
    return (
      sourcePath === "rust/crates/omena-lsp-server/src/external_sif_loader.rs" &&
      symbol === "read_lock_external_sifs"
    );
  }
  if (ruleId === "lsp-lock-bridge-reconciliation") {
    return sourcePath === "rust/crates/omena-lsp-server/src/external_sif_loader.rs";
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

function scanBridgeLockReaderSites(
  sources: readonly { sourcePath: string; source: string }[],
): string[] {
  const patterns = [
    /read_omena_lock_json_v1\s*\(/u,
    /verify_omena_lock_frozen_v1\s*\(/u,
    /OmenaLock(?:SifEntry)?V1/u,
    /(?:read|read_to_string|File::open)\s*\([^\n]*(?:lockfile|omena\.lock)/u,
  ];
  return sources
    .flatMap(({ sourcePath, source }) =>
      source
        .split(/\r?\n/u)
        .flatMap((line, index) =>
          patterns.some((pattern) => pattern.test(line))
            ? [`${sourcePath}:${index + 1}:${line.trim()}`]
            : [],
        ),
    )
    .toSorted();
}

function scanBridgeNetworkSites(
  sources: readonly { sourcePath: string; source: string }[],
): string[] {
  const pattern =
    /(?:std::net::|tokio::net::|reqwest::|ureq::|hyper::Client|TcpStream|UdpSocket|Command::new\("(?:cosign|gh|curl)")/u;
  return sources
    .flatMap(({ sourcePath, source }) =>
      source
        .split(/\r?\n/u)
        .flatMap((line, index) =>
          pattern.test(line) ? [`${sourcePath}:${index + 1}:${line.trim()}`] : [],
        ),
    )
    .toSorted();
}

function maskCfgTestItems(source: string): string {
  const masked = [...source];
  const lexicalSource = maskRustCommentsAndLiterals(source);
  const attribute = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/gu;
  for (const match of lexicalSource.matchAll(attribute)) {
    const start = match.index;
    const end = findAttributedRustItemEnd(source, start + match[0].length);
    assert.ok(end > start, "cfg(test) item must have a balanced item boundary");
    for (let index = start; index < end; index += 1) {
      if (masked[index] !== "\n" && masked[index] !== "\r") masked[index] = " ";
    }
  }
  return masked.join("");
}

function maskRustCommentsAndLiterals(source: string): string {
  const masked = [...source];
  let index = 0;
  while (index < source.length) {
    if (/\s/u.test(source[index] ?? "")) {
      index += 1;
      continue;
    }
    const end = skipRustTriviaOrLiteral(source, index);
    if (end <= index) {
      index += 1;
      continue;
    }
    for (let cursor = index; cursor < end; cursor += 1) {
      if (masked[cursor] !== "\n" && masked[cursor] !== "\r") masked[cursor] = " ";
    }
    index = end;
  }
  return masked.join("");
}

function findAttributedRustItemEnd(source: string, start: number): number {
  let index = start;
  while (index < source.length) {
    const skipped = skipRustTriviaOrLiteral(source, index);
    if (skipped > index) {
      index = skipped;
      continue;
    }
    const character = source[index];
    if (character === ";") return index + 1;
    if (character === "{") return findBalancedRustBraceEnd(source, index);
    index += 1;
  }
  return source.length;
}

function findBalancedRustBraceEnd(source: string, openingBrace: number): number {
  let depth = 0;
  let index = openingBrace;
  while (index < source.length) {
    const skipped = skipRustTriviaOrLiteral(source, index);
    if (skipped > index) {
      index = skipped;
      continue;
    }
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return index + 1;
    }
    index += 1;
  }
  throw new Error(
    `unbalanced cfg(test) Rust item near ${JSON.stringify(source.slice(Math.max(0, openingBrace - 80), openingBrace + 120))}`,
  );
}

function skipRustTriviaOrLiteral(source: string, index: number): number {
  if (/\s/u.test(source[index] ?? "")) {
    let cursor = index + 1;
    while (cursor < source.length && /\s/u.test(source[cursor] ?? "")) cursor += 1;
    return cursor;
  }
  if (source.startsWith("//", index)) {
    const newline = source.indexOf("\n", index + 2);
    return newline < 0 ? source.length : newline + 1;
  }
  if (source.startsWith("/*", index)) {
    let cursor = index + 2;
    let depth = 1;
    while (cursor < source.length && depth > 0) {
      if (source.startsWith("/*", cursor)) {
        depth += 1;
        cursor += 2;
      } else if (source.startsWith("*/", cursor)) {
        depth -= 1;
        cursor += 2;
      } else {
        cursor += 1;
      }
    }
    return cursor;
  }
  const charQuoteOffset = source.startsWith("b'", index) ? 1 : 0;
  if (source[index + charQuoteOffset] === "'") {
    let cursor = index + charQuoteOffset + 1;
    let escaped = false;
    while (cursor < source.length && cursor - index <= 12 && source[cursor] !== "\n") {
      if (!escaped && source[cursor] === "'") return cursor + 1;
      if (!escaped && source[cursor] === "\\") escaped = true;
      else escaped = false;
      cursor += 1;
    }
  }
  const raw = source.slice(index).match(/^(?:b?r)(#+)?"/u);
  if (raw) {
    const hashes = raw[1] ?? "";
    const terminator = `"${hashes}`;
    const end = source.indexOf(terminator, index + raw[0].length);
    return end < 0 ? source.length : end + terminator.length;
  }
  const quoteOffset = source.startsWith('b"', index) ? 1 : 0;
  if (source[index + quoteOffset] === '"') {
    let cursor = index + quoteOffset + 1;
    while (cursor < source.length) {
      if (source[cursor] === "\\") cursor += 2;
      else if (source[cursor] === '"') return cursor + 1;
      else cursor += 1;
    }
    return source.length;
  }
  return index;
}

const cfgMaskSelfTest = maskCfgTestItems(`
fn before() { store_external_sif_cache_shard(); }
/// #[cfg(test)]
fn after_doc_decoy() { store_external_sif_cache_shard(); }
#[cfg(test)]
mod tests { fn nested() { let _ = "}"; store_external_sif_cache_shard(); } }
fn after() { store_external_sif_cache_shard(); }
`);
assert.ok(cfgMaskSelfTest.includes("fn before()"));
assert.ok(cfgMaskSelfTest.includes("fn after_doc_decoy()"));
assert.ok(!cfgMaskSelfTest.includes("fn nested()"));
assert.ok(cfgMaskSelfTest.includes("fn after()"));

import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

type CensusRole = "artifact-input" | "trust-writer" | "shard-writer" | "consumer";

type CensusRule = {
  id: string;
  call: string;
};

type CensusRow = {
  id: string;
  role: CensusRole;
  sourcePath: string;
  symbol: string;
  line: number;
  call: string;
};

const repoRoot = process.cwd();
const args = new Set(process.argv.slice(2).filter((arg) => arg !== "--"));
const injectNewStoreCall = args.has("--inject-new-store-call");
const injectDirectShardPrimitive = args.has("--inject-direct-shard-primitive");
const injectCrossWorkspaceSharingPath = args.has("--inject-cross-workspace-sharing-path");
const injectStructLiteralSharingPath = args.has("--inject-struct-literal-sharing-path");
const injectConstantIdentitySharingPath = args.has("--inject-constant-identity-sharing-path");
const injectDocCommentCfgDecoy = args.has("--inject-doc-comment-cfg-decoy");
const injectCliSelectionMint = args.has("--inject-cli-selection-mint");
const injectCliSelectionLaundering = args.has("--inject-cli-selection-laundering");
const injectCliLockReader = args.has("--inject-cli-lock-reader");
const injectLspLockReaderAlias = args.has("--inject-lsp-lock-reader-alias");
const injectRoleFlip = args.has("--inject-role-flip");
assert.deepEqual(
  [...args].filter(
    (arg) =>
      arg.startsWith("--") &&
      arg !== "--inject-new-store-call" &&
      arg !== "--inject-direct-shard-primitive" &&
      arg !== "--inject-cross-workspace-sharing-path" &&
      arg !== "--inject-struct-literal-sharing-path" &&
      arg !== "--inject-constant-identity-sharing-path" &&
      arg !== "--inject-doc-comment-cfg-decoy" &&
      arg !== "--inject-cli-selection-mint" &&
      arg !== "--inject-cli-selection-laundering" &&
      arg !== "--inject-cli-lock-reader" &&
      arg !== "--inject-lsp-lock-reader-alias" &&
      arg !== "--inject-role-flip",
  ),
  [],
  "unknown SIF shard trust census option",
);

const rules: readonly CensusRule[] = [
  {
    id: "cli-sif-artifact-transaction",
    call: "commit_sif_json_output(",
  },
  {
    id: "cli-lock-update-writer",
    call: "fs::write(&lockfile, &lock_json)",
  },
  {
    id: "cli-recorded-verdict-writer",
    call: "write_recorded_shard_verdicts(",
  },
  {
    id: "cli-recorded-bundle-writer",
    call: "write_recorded_sigstore_bundle(",
  },
  {
    id: "cli-lock-json-reader",
    call: "read_omena_lock_json_v1(",
  },
  {
    id: "bridge-shard-store",
    call: "store_external_sif_cache_shard(",
  },
  {
    id: "bridge-shard-atomic-write",
    call: "write_external_sif_cache_shard_atomically(",
  },
  {
    id: "bridge-shard-load",
    call: "load_external_sif_cache_shard(",
  },
  {
    id: "query-resolved-style-shard-consumer",
    call: "generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(",
  },
  {
    id: "query-published-style-shard-consumer",
    call: "generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_cache_context_storage_and_trust(",
  },
  {
    id: "lsp-in-process-style-shard-consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(",
  },
  {
    id: "lsp-refresh-style-shard-consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(",
  },
  {
    id: "lsp-seed-pair-shard-consumer",
    call: "resolve_omena_query_bridge_external_sifs_for_seed_pairs_with_cache_storage_and_trust(",
  },
];

const expectedKeys = [
  "artifact-input|cli-sif-artifact-transaction|rust/crates/omena-cli/src/sif.rs|generate_sif",
  "artifact-input|cli-lock-update-writer|rust/crates/omena-cli/src/lock.rs|lock_update",
  "trust-writer|cli-recorded-verdict-writer|rust/crates/omena-cli/src/lock.rs|lock_verify_attestation",
  "trust-writer|cli-recorded-bundle-writer|rust/crates/omena-cli/src/lock.rs|lock_verify_attestation",
  "consumer|cli-lock-json-reader|rust/crates/omena-cli/src/external_sif_authority.rs|read_explicit_lock_external_sifs",
  "consumer|cli-lock-json-reader|rust/crates/omena-cli/src/lock.rs|lock_status",
  "consumer|cli-lock-json-reader|rust/crates/omena-cli/src/lock.rs|lock_verify",
  "consumer|cli-lock-json-reader|rust/crates/omena-cli/src/lock.rs|read_lockfile_or_empty",
  "consumer|cli-lock-json-reader|rust/crates/omena-cli/src/provenance.rs|provenance_status",
  "shard-writer|bridge-shard-store|rust/crates/omena-bridge/src/style_resolution.rs|generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_impl",
  "shard-writer|bridge-shard-atomic-write|rust/crates/omena-bridge/src/style_resolution.rs|store_external_sif_cache_shard",
  "consumer|bridge-shard-load|rust/crates/omena-bridge/src/style_resolution.rs|generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_impl",
  "consumer|query-resolved-style-shard-consumer|rust/crates/omena-query/src/source.rs|enqueue_alias",
  "consumer|query-published-style-shard-consumer|rust/crates/omena-query/src/source.rs|enqueue_alias",
  "consumer|lsp-in-process-style-shard-consumer|rust/crates/omena-lsp-server/src/external_sif_loader.rs|resolve_in_process_external_sifs_for_lsp",
  "consumer|lsp-refresh-style-shard-consumer|rust/crates/omena-lsp-server/src/external_sif_loader.rs|resolve_external_sifs_for_refresh_documents",
  "consumer|lsp-seed-pair-shard-consumer|rust/crates/omena-lsp-server/src/external_sif_loader.rs|resolve_bridge_external_sifs_for_sources",
].toSorted();

const trackedRustSources = evidenceScanSurface
  .execFileSync("git", ["ls-files", "-z", "--", "*.rs"], {
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
  injectRoleFlip,
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
const sassCompatibilitySource = fs.readFileSync(path.join(repoRoot, "docs/sass-compat.md"), "utf8");
const bridgeSource = fs.readFileSync(path.join(repoRoot, bridgeSourcePath), "utf8");
const bridgeCacheRootSource = fs.readFileSync(
  path.join(repoRoot, "rust/crates/omena-bridge/src/cache_root.rs"),
  "utf8",
);
const bridgeLockReaderSites = scanBridgeLockReaderSites(bridgeProductionSources);
const lspLockReaderSites = scanLspLockReaderSites(injectLspLockReaderAlias);
const cliLockReaderSites = scanCliLockReaderSites(injectCliLockReader);
const cliSelectionBoundaryViolations = scanCliSelectionBoundaryViolations({
  injectVisibleMint: injectCliSelectionMint,
  injectLaundering: injectCliSelectionLaundering,
});
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
  lspLockReaderSites,
  [],
  `omena-lsp-server must not read workspace lock bytes:\n${formatLockReaderSites(lspLockReaderSites)}`,
);
assert.deepEqual(
  cliLockReaderSites.map((site) => site.key),
  [
    "rust/crates/omena-cli/src/external_sif_authority.rs|read_explicit_lock_external_sifs",
    "rust/crates/omena-cli/src/lock.rs|lock_status",
    "rust/crates/omena-cli/src/lock.rs|lock_verify",
    "rust/crates/omena-cli/src/lock.rs|read_lockfile_or_empty",
    "rust/crates/omena-cli/src/provenance.rs|provenance_status",
  ],
  `CLI workspace-lock reader call-set drifted:\n${formatLockReaderSites(cliLockReaderSites)}`,
);
assert.deepEqual(
  cliSelectionBoundaryViolations,
  [],
  `CLI external-SIF parsed-argument provenance boundary drifted:\n${cliSelectionBoundaryViolations.join("\n")}`,
);
if (!injectLspLockReaderAlias) {
  assert.equal(
    scanLspLockReaderSites(true).length,
    1,
    "the alias-and-newline LSP falsifier must expose a future production reader",
  );
}
if (!injectCliLockReader) {
  assert.equal(
    scanCliLockReaderSites(true).length,
    cliLockReaderSites.length + 1,
    "the CLI future-file falsifier must expose an additional production lock reader",
  );
}
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
if (!injectRoleFlip) {
  assert.notDeepEqual(
    scanCensus({ injectRoleFlip: true }).map(rowKey),
    observedKeys,
    "the role mutation arm must change the validated census row projection",
  );
}
if (!injectCliSelectionMint && !injectCliSelectionLaundering) {
  const injectedViolations = scanCliSelectionBoundaryViolations({ injectVisibleMint: true });
  assert.equal(
    injectedViolations.length,
    1,
    "the CLI parsed-argument provenance falsifier must expose a publicly mintable token",
  );
  const launderingViolations = scanCliSelectionBoundaryViolations({ injectLaundering: true });
  assert.ok(
    launderingViolations.length >= 1,
    "the CLI function-item laundering falsifier must expose the additional dispatch helper",
  );
}

// FALSIFIER: id=sif-shard-trust-census-post-test-store-call class=structuralEntailment via=--inject-new-store-call producer=can-fail owner=sif-shard-trust-census entry=git-ls-files-rust-production-shard-callset
// FALSIFIER: id=sif-shard-trust-census-direct-primitive-writer class=structuralEntailment via=--inject-direct-shard-primitive expected=RED owner=sif-shard-trust-census entry=enumerated-disk-writer-callset
// FALSIFIER: id=sif-shard-no-cross-workspace-sharing class=productionCallSiteMutation via=--inject-cross-workspace-sharing-path expected=RED owner=sif-shard-trust-census entry=git-ls-files-rust-production-sharing-callset
// FALSIFIER: id=sif-shard-no-struct-literal-sharing class=productionCallSiteMutation via=--inject-struct-literal-sharing-path expected=RED owner=sif-shard-trust-census entry=enumerated-storage-constructor-callset
// FALSIFIER: id=sif-shard-workspace-identity-callset class=productionCallSiteMutation via=--inject-constant-identity-sharing-path expected=RED owner=sif-shard-trust-census entry=enumerated-storage-constructor-callset
// FALSIFIER: id=sif-shard-cfg-comment-decoy class=lexicalMaskMutation via=--inject-doc-comment-cfg-decoy expected=RED owner=sif-shard-trust-census entry=comment-aware-cfg-test-mask
// FALSIFIER: id=sif-shard-cli-selection-provenance class=productionVisibilityMutation via=--inject-cli-selection-mint expected=RED owner=sif-shard-trust-census entry=dispatch-private-token-mint
// FALSIFIER: id=sif-shard-cli-selection-laundering class=productionCallGraphMutation via=--inject-cli-selection-laundering expected=RED owner=sif-shard-trust-census entry=dispatch-private-selection-callset
// FALSIFIER: id=sif-shard-cli-lock-reader-future-file class=productionCallSiteMutation via=--inject-cli-lock-reader expected=RED owner=sif-shard-trust-census entry=git-ls-files-cli-lock-reader-callset
// FALSIFIER: id=sif-shard-lsp-lock-reader-alias class=productionAliasMutation via=--inject-lsp-lock-reader-alias expected=RED owner=sif-shard-trust-census entry=git-ls-files-lsp-lock-reader-callset
// FALSIFIER: id=sif-shard-role-disposition class=classificationMutation via=--inject-role-flip expected=RED owner=sif-shard-trust-census entry=validated-role-row-projection
process.stdout.write(
  `SIF shard trust census OK: scope=git-ls-files-comment-aware-production-rust coverage=enumerated-writer-consumer-lock-reader-and-spelled-storage-constructor-callsets rows=${rows.length} artifactInputs=${rows.filter((row) => row.role === "artifact-input").length} trustWriters=${rows.filter((row) => row.role === "trust-writer").length} shardWriters=${rows.filter((row) => row.role === "shard-writer").length} consumers=${rows.filter((row) => row.role === "consumer").length} bridgeLockReaders=${bridgeLockReaderSites.length} lspLockReaders=${lspLockReaderSites.length} cliLockReaders=${cliLockReaderSites.length} cliSelectionBoundaryViolations=${cliSelectionBoundaryViolations.length} bridgeNetworkSites=${bridgeNetworkSites.length} bridgeSigstoreVerifierCalls=${bridgeSigstoreVerifierCalls.length} crossWorkspaceSharingPaths=0 workspaceIdentityConstructorSites=${workspaceIdentityConstructorSites.length} workspaceScopedGlobalBranches=${workspaceScopedGlobalBranches} crossWorkspaceServe=false\n${formatRows(rows)}\n`,
);

type CensusInjection = {
  injectStore?: boolean;
  injectPrimitive?: boolean;
  injectDocCommentDecoy?: boolean;
  injectRoleFlip?: boolean;
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
          role:
            injection.injectRoleFlip && rule.id === "cli-lock-json-reader"
              ? "trust-writer"
              : censusRoleForRuleId(rule.id),
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
  // This is deliberately a spelled-type lexical call-set, not Rust type
  // inference: `Self { ... }` and aliases remain outside this census arm.
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

function censusRoleForRuleId(ruleId: string): CensusRole {
  switch (ruleId) {
    case "cli-sif-artifact-transaction":
    case "cli-lock-update-writer":
      return "artifact-input";
    case "cli-recorded-verdict-writer":
    case "cli-recorded-bundle-writer":
      return "trust-writer";
    case "bridge-shard-store":
    case "bridge-shard-atomic-write":
      return "shard-writer";
    case "cli-lock-json-reader":
    case "bridge-shard-load":
    case "query-resolved-style-shard-consumer":
    case "query-published-style-shard-consumer":
    case "lsp-in-process-style-shard-consumer":
    case "lsp-refresh-style-shard-consumer":
    case "lsp-seed-pair-shard-consumer":
      return "consumer";
    default:
      throw new Error(`SIF shard census rule has no derived role: ${ruleId}`);
  }
}

function ruleAppliesToSite(ruleId: string, sourcePath: string, symbol: string): boolean {
  if (ruleId === "cli-sif-artifact-transaction") return symbol === "generate_sif";
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
  if (ruleId === "cli-lock-json-reader") {
    return sourcePath.startsWith("rust/crates/omena-cli/src/");
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

function rowKey(row: Pick<CensusRow, "role" | "id" | "sourcePath" | "symbol">): string {
  return `${row.role}|${row.id}|${row.sourcePath}|${row.symbol}`;
}

function formatRows(observed: readonly CensusRow[]): string {
  return observed
    .map((row) => `${row.role}\t${row.id}\t${row.sourcePath}:${row.line}\t${row.symbol}`)
    .join("\n");
}

function isProductionRustSource(sourcePath: string): boolean {
  return (
    sourcePath.endsWith(".rs") &&
    !/(?:^|\/)tests?(?:\/|\.rs$)/u.test(sourcePath) &&
    !sourcePath.endsWith("_test.rs")
  );
}

type CliSelectionInjection = {
  injectVisibleMint?: boolean;
  injectLaundering?: boolean;
};

function scanCliSelectionBoundaryViolations(injection: CliSelectionInjection = {}): string[] {
  const sources = trackedRustSources
    .filter((sourcePath) => sourcePath.startsWith("rust/crates/omena-cli/src/"))
    .map((sourcePath) => ({
      sourcePath,
      source: fs.readFileSync(path.join(repoRoot, sourcePath), "utf8"),
    }));
  const dispatch = sources.find(
    ({ sourcePath }) => sourcePath === "rust/crates/omena-cli/src/dispatch.rs",
  );
  assert.ok(dispatch, "CLI dispatch source must be tracked");
  if (injection.injectVisibleMint) {
    dispatch.source = dispatch.source.replace(
      "    fn mint(explicit_sif_paths:",
      "    pub(crate) fn mint(explicit_sif_paths:",
    );
  }
  if (injection.injectLaundering) {
    const original = `fn external_sif_selection_from_parsed_arguments(
    explicit_sif_paths: Vec<PathBuf>,
    explicit_lockfile: Option<PathBuf>,
) -> CliExternalSifSelectionV0 {
    CliExternalSifSelectionV0::from_parsed_cli_arguments(ParsedCliExternalSifArgumentsV0::mint(
        explicit_sif_paths,
        explicit_lockfile,
    ))
}`;
    const laundering = `pub(crate) fn external_sif_selection_from_parsed_arguments(
    explicit_sif_paths: Vec<PathBuf>,
    explicit_lockfile: Option<PathBuf>,
) -> CliExternalSifSelectionV0 {
    let mint = ParsedCliExternalSifArgumentsV0::mint;
    let select = CliExternalSifSelectionV0::from_parsed_cli_arguments;
    let selected_lockfile = explicit_lockfile.or_else(|| Some(PathBuf::from("omena.lock")));
    select(mint(explicit_sif_paths, selected_lockfile))
}`;
    assert.ok(dispatch.source.includes(original), "dispatch selection helper fixture drifted");
    dispatch.source = dispatch.source.replace(original, laundering);
  }

  const violations: string[] = [];
  const productionSources = sources.map(({ sourcePath, source }) => ({
    sourcePath,
    source: maskCfgTestItems(source),
  }));
  const dispatchProduction = productionSources.find(
    ({ sourcePath }) => sourcePath === "rust/crates/omena-cli/src/dispatch.rs",
  )?.source;
  const authorityProduction = productionSources.find(
    ({ sourcePath }) => sourcePath === "rust/crates/omena-cli/src/external_sif_authority.rs",
  )?.source;
  assert.ok(dispatchProduction, "CLI dispatch production source must exist");
  assert.ok(authorityProduction, "CLI authority production source must exist");

  const dispatchLexical = maskRustCommentsAndLiterals(dispatchProduction);
  const authorityLexical = maskRustCommentsAndLiterals(authorityProduction);
  if (/\bpub(?:\([^)]*\))?\s+fn\s+mint\s*\(/u.test(dispatchLexical)) {
    violations.push("parsed-token-mint-is-visible-outside-dispatch");
  }
  const tokenBody = dispatchLexical.match(
    /struct\s+ParsedCliExternalSifArgumentsV0\s*\{(?<body>[\s\S]*?)\n\}/u,
  )?.groups?.body;
  assert.ok(tokenBody, "parsed CLI external-SIF token declaration must remain present");
  if (/\bpub(?:\([^)]*\))?\s+[A-Za-z_][A-Za-z0-9_]*\s*:/u.test(tokenBody)) {
    violations.push("parsed-token-field-is-visible-outside-dispatch");
  }
  const selectionBody = authorityLexical.match(
    /struct\s+CliExternalSifSelectionV0\s*\{(?<body>[\s\S]*?)\n\}/u,
  )?.groups?.body;
  assert.ok(selectionBody, "CLI external-SIF selection declaration must remain present");
  if (/\bpub(?:\([^)]*\))?\s+[A-Za-z_][A-Za-z0-9_]*\s*:/u.test(selectionBody)) {
    violations.push("selection-field-is-visible-outside-authority");
  }
  if (authorityLexical.includes("from_explicit_cli_arguments")) {
    violations.push("raw-path-selection-constructor-exists-in-production");
  }
  assert.match(
    authorityLexical,
    /fn\s+from_parsed_cli_arguments\s*\(\s*arguments\s*:\s*ParsedCliExternalSifArgumentsV0\s*\)/u,
    "the production selection constructor must require the dispatch-minted token",
  );

  const mintReferenceOwners = qualifiedReferenceOwners(
    productionSources,
    /ParsedCliExternalSifArgumentsV0\s*::\s*mint/gu,
  );
  const selectionReferenceOwners = qualifiedReferenceOwners(
    productionSources,
    /CliExternalSifSelectionV0\s*::\s*from_parsed_cli_arguments/gu,
  );
  const expectedConstructorOwner =
    "rust/crates/omena-cli/src/dispatch.rs|external_sif_selection_from_parsed_arguments";
  if (
    mintReferenceOwners.length !== 1 ||
    mintReferenceOwners[0]?.key !== expectedConstructorOwner
  ) {
    violations.push(
      `parsed-token-mint-owner-callset-drifted:${mintReferenceOwners.map((site) => site.key).join(",")}`,
    );
  }
  if (
    selectionReferenceOwners.length !== 1 ||
    selectionReferenceOwners[0]?.key !== expectedConstructorOwner
  ) {
    violations.push(
      `parsed-token-selection-owner-callset-drifted:${selectionReferenceOwners
        .map((site) => site.key)
        .join(",")}`,
    );
  }

  const dispatchFunctions = findRustFunctionSpans(dispatchProduction);
  const constructorFunction = dispatchFunctions.find(
    (candidate) => candidate.name === "external_sif_selection_from_parsed_arguments",
  );
  assert.ok(constructorFunction, "dispatch-private external-SIF selection helper must exist");
  if (constructorFunction.visibility !== "private") {
    violations.push("dispatch-selection-helper-is-visible-outside-dispatch");
  }
  const helperCallOwners = qualifiedReferenceOwners(
    productionSources,
    /\bexternal_sif_selection_from_parsed_arguments\s*\(/gu,
    "external_sif_selection_from_parsed_arguments",
  );
  if (
    helperCallOwners.length !== 3 ||
    helperCallOwners.some(
      (site) => site.key !== "rust/crates/omena-cli/src/dispatch.rs|run_with_exit",
    )
  ) {
    violations.push(
      `dispatch-selection-helper-callset-drifted:${helperCallOwners
        .map((site) => site.key)
        .join(",")}`,
    );
  }
  return violations.toSorted();
}

type RustFunctionSpan = {
  name: string;
  visibility: "private" | "visible";
  start: number;
  end: number;
};

type OwnedReference = { key: string; display: string };

function qualifiedReferenceOwners(
  sources: readonly { sourcePath: string; source: string }[],
  pattern: RegExp,
  declarationName?: string,
): OwnedReference[] {
  const sites: OwnedReference[] = [];
  for (const { sourcePath, source } of sources) {
    const lexical = maskRustCommentsAndLiterals(source);
    const functions = findRustFunctionSpans(source);
    for (const match of lexical.matchAll(pattern)) {
      const prefix = lexical.slice(Math.max(0, match.index - 24), match.index);
      if (declarationName && /\bfn\s*$/u.test(prefix)) {
        continue;
      }
      const owner = innermostRustFunctionAt(functions, match.index);
      const line = lexical.slice(0, match.index).split("\n").length;
      sites.push({
        key: `${sourcePath}|${owner?.name ?? "<module>"}`,
        display: `${sourcePath}:${line}:${owner?.name ?? "<module>"}`,
      });
    }
  }
  return sites;
}

function findRustFunctionSpans(source: string): RustFunctionSpan[] {
  const lexical = maskRustCommentsAndLiterals(source);
  const functions: RustFunctionSpan[] = [];
  const declaration =
    /(?<visibility>\bpub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(?<name>[A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu;
  for (const match of lexical.matchAll(declaration)) {
    const name = match.groups?.name;
    if (!name) continue;
    const openingBrace = match.index + match[0].lastIndexOf("{");
    functions.push({
      name,
      visibility: match.groups?.visibility ? "visible" : "private",
      start: match.index,
      end: findBalancedRustBraceEnd(source, openingBrace),
    });
  }
  return functions;
}

function innermostRustFunctionAt(
  functions: readonly RustFunctionSpan[],
  index: number,
): RustFunctionSpan | undefined {
  return functions
    .filter((candidate) => candidate.start <= index && index < candidate.end)
    .toSorted((left, right) => right.start - left.start)[0];
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

type LockReaderSite = { key: string; display: string };

function scanCliLockReaderSites(injectFutureSource: boolean): LockReaderSite[] {
  const sources = trackedRustSources
    .filter((sourcePath) => sourcePath.startsWith("rust/crates/omena-cli/src/"))
    .map((sourcePath) => ({
      sourcePath,
      source: fs.readFileSync(path.join(repoRoot, sourcePath), "utf8"),
    }));
  if (injectFutureSource) {
    sources.push({
      sourcePath: "rust/crates/omena-cli/src/future_workspace_lock_reader.rs",
      source: futureAliasedWorkspaceLockReaderSource(),
    });
  }
  return scanRustLockReaderSites(sources);
}

function scanLspLockReaderSites(injectFutureSource: boolean): LockReaderSite[] {
  const sources = trackedRustSources
    .filter((sourcePath) => sourcePath.startsWith("rust/crates/omena-lsp-server/src/"))
    .map((sourcePath) => ({
      sourcePath,
      source: fs.readFileSync(path.join(repoRoot, sourcePath), "utf8"),
    }));
  if (injectFutureSource) {
    sources.push({
      sourcePath: "rust/crates/omena-lsp-server/src/future_workspace_lock_reader.rs",
      source: futureAliasedWorkspaceLockReaderSource(),
    });
  }
  return scanRustLockReaderSites(sources);
}

function futureAliasedWorkspaceLockReaderSource(): string {
  return `
use omena_sif::read_omena_lock_json_v1 as parse_workspace_lock;
fn future_workspace_lock_reader(root: &Path) {
    let lockfile = root.join("omena.lock");
    let lock_source = std::fs::read_to_string(
        lockfile,
    ).unwrap();
    let _ = parse_workspace_lock(
        &lock_source,
    );
}
`;
}

function scanRustLockReaderSites(
  sources: readonly { sourcePath: string; source: string }[],
): LockReaderSite[] {
  const sites = new Map<string, LockReaderSite>();
  for (const { sourcePath, source } of sources) {
    const production = maskCfgTestItems(source);
    const lexical = maskRustCommentsAndLiterals(production);
    const commentless = maskRustCommentsPreservingLiterals(production);
    const functions = findRustFunctionSpans(production);
    const readerNames = new Set([
      "read_omena_lock_json_v1",
      "read_lock_external_sifs",
      "verify_omena_lock_frozen_v1",
    ]);
    for (const match of lexical.matchAll(
      /\b(?:read_omena_lock_json_v1|read_lock_external_sifs|verify_omena_lock_frozen_v1)\s+as\s+(?<alias>[A-Za-z_][A-Za-z0-9_]*)/gu,
    )) {
      const alias = match.groups?.alias;
      if (alias) readerNames.add(alias);
    }
    for (const readerName of readerNames) {
      const callPattern = new RegExp(`\\b${escapeRegExp(readerName)}\\s*\\(`, "gu");
      for (const match of lexical.matchAll(callPattern)) {
        const prefix = lexical.slice(Math.max(0, match.index - 16), match.index);
        if (/\bfn\s*$/u.test(prefix)) continue;
        recordLockReaderSite(sites, sourcePath, production, functions, match.index);
      }
    }

    const fileReadPattern =
      /\b(?:(?:std\s*::\s*)?fs\s*::\s*(?:read|read_to_string)|File\s*::\s*open)\s*\(/gu;
    for (const match of commentless.matchAll(fileReadPattern)) {
      const openingParen = match.index + match[0].lastIndexOf("(");
      const callEnd = findBalancedRustParenEnd(production, openingParen);
      const call = commentless.slice(match.index, callEnd);
      if (!/(?:\b(?:lockfile|lock_path|workspace_lock)\b|["']omena\.lock["'])/u.test(call)) {
        continue;
      }
      recordLockReaderSite(sites, sourcePath, production, functions, match.index);
    }
  }
  return [...sites.values()].toSorted((left, right) =>
    left.key < right.key ? -1 : left.key > right.key ? 1 : 0,
  );
}

function recordLockReaderSite(
  sites: Map<string, LockReaderSite>,
  sourcePath: string,
  source: string,
  functions: readonly RustFunctionSpan[],
  index: number,
): void {
  const owner = innermostRustFunctionAt(functions, index)?.name ?? "<module>";
  const key = `${sourcePath}|${owner}`;
  const line = source.slice(0, index).split("\n").length;
  sites.set(key, { key, display: `${sourcePath}:${line}:${owner}` });
}

function formatLockReaderSites(sites: readonly LockReaderSite[]): string {
  return sites.map((site) => site.display).join("\n");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
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

function maskRustCommentsPreservingLiterals(source: string): string {
  const masked = [...source];
  let index = 0;
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      const end = newline < 0 ? source.length : newline;
      for (let cursor = index; cursor < end; cursor += 1) masked[cursor] = " ";
      index = end;
      continue;
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
      for (let maskIndex = index; maskIndex < cursor; maskIndex += 1) {
        if (masked[maskIndex] !== "\n" && masked[maskIndex] !== "\r") masked[maskIndex] = " ";
      }
      index = cursor;
      continue;
    }
    const skipped = skipRustTriviaOrLiteral(source, index);
    if (skipped > index) {
      index = skipped;
      continue;
    }
    index += 1;
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

function findBalancedRustParenEnd(source: string, openingParen: number): number {
  let depth = 0;
  let index = openingParen;
  while (index < source.length) {
    const skipped = skipRustTriviaOrLiteral(source, index);
    if (skipped > index) {
      index = skipped;
      continue;
    }
    if (source[index] === "(") depth += 1;
    if (source[index] === ")") {
      depth -= 1;
      if (depth === 0) return index + 1;
    }
    index += 1;
  }
  throw new Error(
    `unbalanced Rust call near ${JSON.stringify(source.slice(Math.max(0, openingParen - 80), openingParen + 120))}`,
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

import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const ROOT = process.cwd();

const FILES = {
  queryTypes: "rust/crates/omena-query/src/types.rs",
  sourceProvider: "server/lsp-server/src/providers/diagnostics.ts",
  styleProvider: "server/lsp-server/src/providers/scss-diagnostics.ts",
  eslintShared: "packages/eslint-plugin/lib/_shared.cjs",
  stylelintShared: "packages/stylelint-plugin/lib/_shared.cjs",
  oxlintPlugin: "packages/oxlint-plugin/index.cjs",
  vitePlugin: "packages/vite-plugin/index.cjs",
  postcssPlugin: "packages/postcss-plugin/index.cjs",
  cssBuildAdapter: "packages/css-build-adapter/index.cjs",
  checkOrchestratorSrc: "packages/check-orchestrator/src",
  cmeCheckerShadow: "packages/cme-checker/src/shadow.ts",
  vitestCmeSrc: "packages/vitest-cme/src",
  cliDiagnostics: "rust/crates/omena-cli/src/diagnostics.rs",
  cliTests: "rust/crates/omena-cli/src/tests.rs",
  napi: "rust/crates/omena-napi/src/lib.rs",
  wasm: "rust/crates/omena-wasm/src/lib.rs",
  checkerContracts: "server/engine-core-ts/src/core/checker/contracts.ts",
  selectedQueryBackend: "server/engine-host-node/src/selected-query-backend.ts",
  engineCoreTs: "server/engine-core-ts/src",
  engineHostNode: "server/engine-host-node/src",
  checkerCli: "server/checker-cli/src",
  lspServerProviders: "server/lsp-server/src/providers",
  engineShadowRunner: "rust/crates/engine-shadow-runner/src",
  querySourceRefs: "rust/crates/omena-query/src/style/source_refs.rs",
  queryDynamicClassname: "rust/crates/omena-query/src/style/dynamic_classname.rs",
  queryStyleDiagnostics: "rust/crates/omena-query/src/style/diagnostics",
  queryStyleSourceUsage: "rust/crates/omena-query/src/style/diagnostics/source_usage.rs",
  querySourceSurfacesTests: "rust/crates/omena-query/src/tests/source_surfaces.rs",
  sourceConsumerGate: "scripts/check-source-diagnostics-query-consumer.ts",
  styleConsumerGate: "scripts/check-style-diagnostics-query-consumer.ts",
  sourceProviderTests: "test/unit/providers/diagnostics.test.ts",
  styleProviderTests: "test/unit/providers/scss-diagnostics.test.ts",
  queryDiagnosticsIdl: "server/engine-host-node/src/query-diagnostics-idl.generated.ts",
} as const;

const SOURCE_DIAGNOSTIC_CODE_PAIRS = [
  ["missingModule", "missing-module"],
  ["missingStaticClass", "missing-static-class"],
  ["missingTemplatePrefix", "missing-template-prefix"],
  ["missingResolvedClassValues", "missing-resolved-class-values"],
  ["missingResolvedClassDomain", "missing-resolved-class-domain"],
] as const;

const STYLE_DIAGNOSTIC_CODE_PAIRS = [
  ["unusedSelector", "unused-selector"],
  ["missingComposedModule", "missing-composed-module"],
  ["missingComposedSelector", "missing-composed-selector"],
  ["missingValueModule", "missing-value-module"],
  ["missingImportedValue", "missing-imported-value"],
  ["missingKeyframes", "missing-keyframes"],
  ["missingCustomProperty", "missing-custom-property"],
  ["missingSassSymbol", "missing-sass-symbol"],
] as const;

const CHECKER_DIAGNOSTIC_CODE_PAIRS = [
  ...SOURCE_DIAGNOSTIC_CODE_PAIRS,
  ...STYLE_DIAGNOSTIC_CODE_PAIRS,
] as const;

const REQUIRED_ADAPTER_EGRESS_CLOSURE = [
  ["eslint-plugin", "cli-query"],
  ["stylelint-plugin", "cli-query"],
  ["oxlint-plugin", "cli-query"],
  ["postcss-plugin", "build-transport"],
  ["vite-plugin", "build-transport"],
  ["css-build-adapter", "build-transport"],
  ["check-orchestrator", "non-producer"],
  ["cme-checker", "shadow-only"],
  ["vitest-cme", "test-only"],
  ["lsp-editor", "rust-query-owned+ts-current-honest-downgrade"],
] as const;

const ADAPTER_EGRESS_CLOSURE = [
  ["eslint-plugin", "cli-query"],
  ["stylelint-plugin", "cli-query"],
  ["oxlint-plugin", "cli-query"],
  ["postcss-plugin", "build-transport"],
  ["vite-plugin", "build-transport"],
  ["css-build-adapter", "build-transport"],
  ["check-orchestrator", "non-producer"],
  ["cme-checker", "shadow-only"],
  ["vitest-cme", "test-only"],
  ["lsp-editor", "rust-query-owned+ts-current-honest-downgrade"],
] as const;

const LEGACY_QUERY_MERGE_TOKENS = [
  "QUERY_TO_CHECKER_DIAGNOSTIC_CODE",
  "mergeQueryDiagnosticWithCheckerData",
  "mergeQueryOwnedSourceDiagnostics",
  "mergeQueryOwnedStyleDiagnostics",
] as const;

const ESLINT_LEGACY_FALLBACK_TOKENS = [
  "checkSourceDocument",
  "formatLegacyCheckerFinding",
  "formatCheckerFinding",
  "createWorkspaceAnalysisHost",
  "createWorkspaceStyleHost",
  "OMENA_ESLINT_QUERY_BACKEND",
] as const;

const STYLELINT_LEGACY_FALLBACK_TOKENS = [
  "readStyleCheckReport",
  "STYLE_CHECK_REPORT_CACHE",
  "check:workspace",
  "OMENA_STYLELINT_QUERY_BACKEND",
] as const;

const ADAPTER_LEGACY_DIAGNOSTIC_COMPUTE_TOKENS = [
  "CheckerFinding",
  "SourceCheckerFinding",
  "StyleCheckerFinding",
  "WorkspaceCheckerFinding",
  "formatCheckerFinding",
  "checkSourceDocument",
  "resolveSourceDiagnosticFindings",
  "resolveStyleDiagnosticFindings",
  "CheckerReportV1",
  "buildCheckerReportV1",
] as const;

const CHECKER_FINDING_CENSUS_TOKENS = [
  "CheckerFinding",
  "CheckerReportV1",
  "formatCheckerFinding",
  "checkSourceDocument",
  "resolveSourceDiagnosticFindings",
  "resolveStyleDiagnosticFindings",
  "CmeCheckerFinding",
  "CheckerFindingRecordV1",
] as const;

function main(): void {
  assertCheckerCodeMirror();
  assertFullAdapterEgressClosure();
  assertRustQueryDiagnosticProducerCoverage();
  assertLspProviderQueryDiagnosticCoverage();
  assertLspMergedOutputSnapshotOracleCoverage();
  assertLspSelectedQueryDiagnostics();
  assertLspTypescriptCurrentHonestDowngrade();
  assertQueryDiagnosticsShapeLock();
  assertCliNapiWasmQueryDiagnostics();
  assertLintPluginCliDiagnostics();
  assertOxlintCliDiagnostics();
  assertBuildAdaptersDoNotComputeDiagnostics();
  assertComparatorAndTestAdapters();
  assertCheckerFindingCensus();

  process.stdout.write(
    [
      "diagnostics single-family egress ok:",
      "lsp=rust-query-owned+ts-current-honest-downgrade",
      "cli=omena-query",
      "napi=omena-query",
      "wasm=omena-query",
      "eslint=cli-query",
      "stylelint=cli-query",
      "oxlint=cli-query",
      "postcss=build-transport",
      "vite=build-transport",
      "css-build=build-transport",
      "orchestrator=non-producer",
      "cme-checker=shadow-only",
      "vitest-cme=test-only",
      "closure=10",
      "shape-lock=cli-napi-wasm",
      "checker-code-mirror=13",
      "lsp-provider-code-coverage=13",
      "lsp-merged-output-snapshot-oracle=13",
      "query-diagnostics-idl=generated",
      "legacy-plugin-fallback=absent",
    ].join(" ") + "\n",
  );
}

function assertFullAdapterEgressClosure(): void {
  assert.deepEqual(
    ADAPTER_EGRESS_CLOSURE,
    REQUIRED_ADAPTER_EGRESS_CLOSURE,
    "diagnostics egress closure must enumerate the 9 packaged adapters plus the LSP editor surface",
  );
  assert.equal(
    ADAPTER_EGRESS_CLOSURE.length,
    10,
    "diagnostics egress closure must contain exactly 10 classified surfaces",
  );
}

function assertCheckerCodeMirror(): void {
  const checkerContracts = readRepoFile(FILES.checkerContracts);
  const eslint = readRepoFile(FILES.eslintShared);
  const stylelint = readRepoFile(FILES.stylelintShared);
  const checkerCodes = [...checkerContracts.matchAll(/readonly code: "([^"]+)"/gu)].map(
    (match) => match[1],
  );
  assert.deepEqual(
    checkerCodes.toSorted(),
    CHECKER_DIAGNOSTIC_CODE_PAIRS.map(([, checkerCode]) => checkerCode).toSorted(),
    `${FILES.checkerContracts} must declare the 13 mirrored checker diagnostic codes`,
  );

  assertExactDiagnosticMap(
    eslint,
    "OMENA_QUERY_SOURCE_DIAGNOSTIC_CODE_MAP",
    SOURCE_DIAGNOSTIC_CODE_PAIRS,
    FILES.eslintShared,
  );
  assertExactDiagnosticMap(
    stylelint,
    "OMENA_QUERY_STYLE_DIAGNOSTIC_CODE_MAP",
    STYLE_DIAGNOSTIC_CODE_PAIRS,
    FILES.stylelintShared,
  );
}

function assertRustQueryDiagnosticProducerCoverage(): void {
  const sourceProducers = [
    readRepoFile(FILES.querySourceRefs),
    readRepoFile(FILES.queryDynamicClassname),
  ].join("\n");
  const styleProducers = [
    readRepoTree(FILES.queryStyleDiagnostics),
    readRepoFile(FILES.queryStyleSourceUsage),
  ].join("\n");
  const provenanceEvidence = [
    readRepoFile(FILES.querySourceSurfacesTests),
    readRepoFile(FILES.sourceConsumerGate),
    readRepoFile(FILES.styleConsumerGate),
  ].join("\n");

  for (const [queryCode] of SOURCE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(sourceProducers, `"${queryCode}"`, "omena-query source producers");
  }
  for (const [queryCode] of STYLE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(styleProducers, `"${queryCode}"`, "omena-query style producers");
  }
  assertIncludes(
    sourceProducers,
    "omena-query.source-syntax-index",
    "omena-query source producers",
  );
  assertIncludes(
    sourceProducers,
    "omena-query.style-selector-definitions",
    "omena-query source producers",
  );
  assertIncludes(
    provenanceEvidence,
    "omena-query-checker-orchestrator.product-diagnostic-gate",
    "omena-query diagnostic provenance gates",
  );
  assertIncludes(
    provenanceEvidence,
    "omena-checker.rule-registry",
    "omena-query diagnostic provenance gates",
  );
}

function assertLspProviderQueryDiagnosticCoverage(): void {
  const sourceProviderTests = readRepoFile(FILES.sourceProviderTests);
  const styleProviderTests = readRepoFile(FILES.styleProviderTests);

  assertIncludes(
    sourceProviderTests,
    "snapshots the selected-query source merged-output oracle for every diagnostic code",
    FILES.sourceProviderTests,
  );
  assertIncludes(sourceProviderTests, "stableDiagnosticSnapshot", FILES.sourceProviderTests);
  assertIncludes(sourceProviderTests, "toMatchInlineSnapshot", FILES.sourceProviderTests);
  assertIncludes(
    sourceProviderTests,
    "selectedQuerySourceMergedOutputReferenceDiagnostics",
    FILES.sourceProviderTests,
  );
  assertIncludes(
    sourceProviderTests,
    "expect(stableDiagnosticSnapshot(diagnostics)).toEqual",
    FILES.sourceProviderTests,
  );
  assertIncludes(
    sourceProviderTests,
    "does not fall back to checker diagnostics in the selected-query source path",
    FILES.sourceProviderTests,
  );
  assertIncludes(
    styleProviderTests,
    "snapshots the selected-query style merged-output oracle for every diagnostic code",
    FILES.styleProviderTests,
  );
  assertIncludes(styleProviderTests, "stableDiagnosticSnapshot", FILES.styleProviderTests);
  assertIncludes(styleProviderTests, "toMatchInlineSnapshot", FILES.styleProviderTests);
  assertIncludes(
    styleProviderTests,
    "selectedQueryStyleMergedOutputReferenceDiagnostics",
    FILES.styleProviderTests,
  );
  assertIncludes(
    styleProviderTests,
    "expect(stableDiagnosticSnapshot(diagnostics)).toEqual",
    FILES.styleProviderTests,
  );
  for (const token of [
    "createModuleFile",
    "createSelector",
    "createValue",
    "createKeyframes",
    "createCustomProperty",
    "createSassSymbol",
  ]) {
    assertIncludes(styleProviderTests, token, FILES.styleProviderTests);
  }
  for (const [queryCode] of SOURCE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(sourceProviderTests, `"${queryCode}"`, FILES.sourceProviderTests);
  }
  for (const [queryCode] of STYLE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(styleProviderTests, `"${queryCode}"`, FILES.styleProviderTests);
  }
}

function assertLspMergedOutputSnapshotOracleCoverage(): void {
  const sourceProviderTests = readRepoFile(FILES.sourceProviderTests);
  const styleProviderTests = readRepoFile(FILES.styleProviderTests);

  assertIncludes(
    sourceProviderTests,
    "expect(stableDiagnosticSnapshot(diagnostics)).toMatchInlineSnapshot",
    FILES.sourceProviderTests,
  );
  assertIncludes(
    styleProviderTests,
    "expect(stableDiagnosticSnapshot(diagnostics)).toMatchInlineSnapshot",
    FILES.styleProviderTests,
  );
  assertIncludes(
    sourceProviderTests,
    "selectedQuerySourceMergedOutputReferenceDiagnostics(queryCodes)",
    FILES.sourceProviderTests,
  );
  assertIncludes(
    styleProviderTests,
    "selectedQueryStyleMergedOutputReferenceDiagnostics(queryCodes)",
    FILES.styleProviderTests,
  );

  for (const [queryCode] of SOURCE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(sourceProviderTests, `"code": "${queryCode}"`, FILES.sourceProviderTests);
  }
  for (const [queryCode] of STYLE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(styleProviderTests, `"code": "${queryCode}"`, FILES.styleProviderTests);
  }

  for (const [source, label] of [
    [sourceProviderTests, FILES.sourceProviderTests],
    [styleProviderTests, FILES.styleProviderTests],
  ] as const) {
    assertIncludes(source, "omena-query-checker-orchestrator.product-diagnostic-gate", label);
    assertIncludes(source, "omena-checker.rule-registry", label);
    assertIncludes(source, '"querySeverity":', label);
    assertIncludes(source, '"provenance":', label);
  }

  for (const token of [
    '"precision":',
    '"revisionAxis": "OmenaQuerySourceDiagnosticsForFileV0.input"',
    '"createSelector":',
    '"createModuleFile":',
  ]) {
    assertIncludes(sourceProviderTests, token, FILES.sourceProviderTests);
  }

  for (const token of [
    '"createModuleFile":',
    '"createSelector":',
    '"createValue":',
    '"createKeyframes":',
    '"createCustomProperty":',
    '"createSassSymbol":',
  ]) {
    assertIncludes(styleProviderTests, token, FILES.styleProviderTests);
  }
}

function assertLspSelectedQueryDiagnostics(): void {
  const sourceProvider = readRepoFile(FILES.sourceProvider);
  const styleProvider = readRepoFile(FILES.styleProvider);
  const queryDiagnosticsIdl = readRepoFile(FILES.queryDiagnosticsIdl);

  assertNone(sourceProvider, LEGACY_QUERY_MERGE_TOKENS, FILES.sourceProvider);
  assertNone(styleProvider, LEGACY_QUERY_MERGE_TOKENS, FILES.styleProvider);
  assertIncludes(
    queryDiagnosticsIdl,
    "OmenaQuerySourceDiagnosticsForFileV0Json",
    FILES.queryDiagnosticsIdl,
  );
  assertIncludes(
    queryDiagnosticsIdl,
    "OmenaQueryStyleDiagnosticsForFileV0Json",
    FILES.queryDiagnosticsIdl,
  );
  assertIncludes(sourceProvider, "OmenaQuerySourceDiagnosticsForFileV0Json", FILES.sourceProvider);
  assertIncludes(styleProvider, "OmenaQueryStyleDiagnosticsForFileV0Json", FILES.styleProvider);
  assertNotIncludes(sourceProvider, "interface QuerySourceDiagnostic", FILES.sourceProvider);
  assertNotIncludes(styleProvider, "interface QueryStyleDiagnostic", FILES.styleProvider);

  assertIncludes(
    sourceProvider,
    "SELECTED_QUERY_RUNNER_COMMANDS.sourceDiagnosticsForFile",
    FILES.sourceProvider,
  );
  assertIncludes(sourceProvider, "diagnostic.precision", FILES.sourceProvider);
  assertIncludes(
    styleProvider,
    "SELECTED_QUERY_RUNNER_COMMANDS.styleDiagnosticsForFile",
    FILES.styleProvider,
  );
  assertIncludes(
    sourceProvider,
    "const selectedBackendKind = resolveSelectedQueryBackendKind();",
    FILES.sourceProvider,
  );
  assertIncludes(
    sourceProvider,
    'if (!rustRunner && selectedBackendKind === "rust-selected-query")',
    FILES.sourceProvider,
  );
  assertIncludes(sourceProvider, "return [];", FILES.sourceProvider);
  assertIncludes(
    styleProvider,
    "const selectedBackendKind = resolveSelectedQueryBackendKind(runtimeDeps?.env);",
    FILES.styleProvider,
  );
  assertIncludes(styleProvider, "const hasStyleQueryRuntimeInputs = Boolean(", FILES.styleProvider);
  assertIncludes(styleProvider, "!hasStyleQueryRuntimeInputs", FILES.styleProvider);
  assertIncludes(styleProvider, "return [];", FILES.styleProvider);
  assertNotIncludes(
    sourceProvider,
    '.filter((diagnostic) => diagnostic.code !== "missingModule")',
    FILES.sourceProvider,
  );
  assertIncludes(sourceProvider, "findMissingModuleCreateFileData", FILES.sourceProvider);
}

function assertLspTypescriptCurrentHonestDowngrade(): void {
  const sourceProvider = readRepoFile(FILES.sourceProvider);
  const styleProvider = readRepoFile(FILES.styleProvider);
  const selectedQueryBackend = readRepoFile(FILES.selectedQueryBackend);

  assertIncludes(sourceProvider, "resolveSourceDiagnosticFindings(", FILES.sourceProvider);
  assertIncludes(sourceProvider, "resolveSourceDiagnosticFindingsAsync(", FILES.sourceProvider);
  assertIncludes(sourceProvider, "formatCheckerFinding(finding", FILES.sourceProvider);
  assertIncludes(sourceProvider, "finding satisfies never", FILES.sourceProvider);

  assertIncludes(styleProvider, "resolveStyleDiagnosticFindings(", FILES.styleProvider);
  assertIncludes(styleProvider, "resolveStyleDiagnosticFindingsAsync(", FILES.styleProvider);
  assertIncludes(styleProvider, "formatCheckerFinding(finding", FILES.styleProvider);
  assertIncludes(styleProvider, "finding satisfies never", FILES.styleProvider);

  assertIncludes(
    selectedQueryBackend,
    '? "rust-selected-query"\n      : "typescript-current"',
    FILES.selectedQueryBackend,
  );
  assertIncludes(selectedQueryBackend, ': "typescript-current";', FILES.selectedQueryBackend);
}

function assertQueryDiagnosticsShapeLock(): void {
  const queryTypes = readRepoFile(FILES.queryTypes);
  const cliTests = readRepoFile(FILES.cliTests);
  const napi = readRepoFile(FILES.napi);
  const wasm = readRepoFile(FILES.wasm);

  for (const token of [
    "pub struct OmenaQueryStyleDiagnosticV0",
    "pub struct OmenaQueryStyleDiagnosticsForFileV0",
    "pub struct OmenaQuerySourceDiagnosticV0",
    "pub struct OmenaQuerySourceDiagnosticsForFileV0",
    "pub schema_version: &'static str",
    "pub product: &'static str",
    "pub file_uri: String",
    "pub file_kind: &'static str",
    "pub diagnostic_count: usize",
    "pub diagnostics: Vec<OmenaQuery",
    "pub ready_surfaces: Vec<&'static str>",
    "pub provenance: Vec<&'static str>",
    "pub precision: Option<OmenaQueryAnalysisPrecisionV0>",
  ]) {
    assertIncludes(queryTypes, token, FILES.queryTypes);
  }

  for (const [source, label] of [
    [cliTests, FILES.cliTests],
    [wasm, FILES.wasm],
  ] as const) {
    assertIncludes(source, "schema_version", label);
    assertIncludes(source, "omena-query.diagnostics-for-file", label);
    assertIncludes(source, "diagnostic_count", label);
    assertIncludes(source, "ready_surfaces", label);
    assertIncludes(source, "provenance", label);
  }

  assertIncludes(napi, "assert_query_diagnostics_json_shape", FILES.napi);
  assertIncludes(napi, "schemaVersion", FILES.napi);
  assertIncludes(napi, "diagnosticCount", FILES.napi);
  assertIncludes(napi, "readySurfaces", FILES.napi);
  assertIncludes(napi, "provenance", FILES.napi);
  assertIncludes(napi, 'category").is_none()', FILES.napi);
}

function assertCliNapiWasmQueryDiagnostics(): void {
  const cli = readRepoFile(FILES.cliDiagnostics);
  const napi = readRepoFile(FILES.napi);
  const wasm = readRepoFile(FILES.wasm);

  for (const [label, source] of [
    [FILES.cliDiagnostics, cli],
    [FILES.napi, napi],
    [FILES.wasm, wasm],
  ] as const) {
    assertIncludes(source, "summarize_omena_query_source_diagnostics", label);
    assertIncludes(source, "summarize_omena_query_style_diagnostics", label);
  }

  assertIncludes(cli, "OmenaQuerySourceDiagnosticsForFileV0", FILES.cliDiagnostics);
  assertIncludes(napi, "OmenaNapiSourceDiagnosticsForFileV0", FILES.napi);
  assertIncludes(napi, "OmenaNapiStyleDiagnosticsForFileV0", FILES.napi);
  assertIncludes(wasm, "OmenaWasmSourceDiagnosticsForFileV0", FILES.wasm);
  assertIncludes(wasm, "OmenaWasmStyleDiagnosticsForFileV0", FILES.wasm);
}

function assertLintPluginCliDiagnostics(): void {
  const eslint = readRepoFile(FILES.eslintShared);
  const stylelint = readRepoFile(FILES.stylelintShared);

  assertNone(eslint, ESLINT_LEGACY_FALLBACK_TOKENS, FILES.eslintShared);
  assertNone(stylelint, STYLELINT_LEGACY_FALLBACK_TOKENS, FILES.stylelintShared);

  assertIncludes(eslint, '"source-diagnostics"', FILES.eslintShared);
  assertIncludes(eslint, "OMENA_CLI_BIN", FILES.eslintShared);
  assertIncludes(eslint, "OMENA_QUERY_SOURCE_DIAGNOSTIC_CODE_MAP", FILES.eslintShared);
  assertIncludes(stylelint, '"style-diagnostics"', FILES.stylelintShared);
  assertIncludes(stylelint, "OMENA_CLI_BIN", FILES.stylelintShared);
  assertIncludes(stylelint, "OMENA_QUERY_STYLE_DIAGNOSTIC_CODE_MAP", FILES.stylelintShared);
}

function assertOxlintCliDiagnostics(): void {
  const oxlint = readRepoFile(FILES.oxlintPlugin);

  assertNone(oxlint, ADAPTER_LEGACY_DIAGNOSTIC_COMPUTE_TOKENS, FILES.oxlintPlugin);
  assertIncludes(oxlint, "readOmenaSourceDiagnostics", FILES.oxlintPlugin);
  assertIncludes(oxlint, '"source-diagnostics"', FILES.oxlintPlugin);
  assertIncludes(oxlint, '"--json"', FILES.oxlintPlugin);
  assertIncludes(oxlint, "DIAGNOSTIC_RULES", FILES.oxlintPlugin);
  for (const [queryCode, checkerCode] of SOURCE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(oxlint, `queryCode: "${queryCode}"`, FILES.oxlintPlugin);
    assertIncludes(oxlint, `"${checkerCode}"`, FILES.oxlintPlugin);
  }
}

function assertBuildAdaptersDoNotComputeDiagnostics(): void {
  const vite = readRepoFile(FILES.vitePlugin);
  const postcss = readRepoFile(FILES.postcssPlugin);
  const cssBuildAdapter = readRepoFile(FILES.cssBuildAdapter);
  const checkOrchestrator = readRepoTree(FILES.checkOrchestratorSrc, [".ts"]);

  for (const [source, label] of [
    [vite, FILES.vitePlugin],
    [postcss, FILES.postcssPlugin],
    [cssBuildAdapter, FILES.cssBuildAdapter],
    [checkOrchestrator, FILES.checkOrchestratorSrc],
  ] as const) {
    assertNone(source, ADAPTER_LEGACY_DIAGNOSTIC_COMPUTE_TOKENS, label);
  }

  assertIncludes(vite, 'require("@omena/css-build-adapter")', FILES.vitePlugin);
  assertIncludes(vite, "rebuildAndCache", FILES.vitePlugin);
  assertIncludes(vite, "handleHotUpdate", FILES.vitePlugin);
  assertIncludes(vite, "devRuntimeUpdatePayload", FILES.vitePlugin);
  assertIncludes(postcss, 'require("@omena/css-build-adapter")', FILES.postcssPlugin);
  assertIncludes(postcss, "rebuildAndCache", FILES.postcssPlugin);
  assertIncludes(cssBuildAdapter, 'loadOptionalCjs("@omena/napi")', FILES.cssBuildAdapter);
  assertIncludes(cssBuildAdapter, 'loadOptionalEsm("@omena/wasm")', FILES.cssBuildAdapter);
  assertIncludes(cssBuildAdapter, "buildStyleSourcesWithContextJson", FILES.cssBuildAdapter);
  assertIncludes(
    cssBuildAdapter,
    "CLI/cargo fallback is intentionally not used on plugin hot paths",
    FILES.cssBuildAdapter,
  );
}

function assertComparatorAndTestAdapters(): void {
  const cmeCheckerShadow = readRepoFile(FILES.cmeCheckerShadow);
  const vitestCme = readRepoTree(FILES.vitestCmeSrc, [".ts"]);

  assertIncludes(cmeCheckerShadow, "CmeCheckerFindingLikeV0", FILES.cmeCheckerShadow);
  assertIncludes(cmeCheckerShadow, "compareCheckerFindings", FILES.cmeCheckerShadow);
  assertIncludes(cmeCheckerShadow, "diffCheckerCanonicalCandidate", FILES.cmeCheckerShadow);
  assertNone(vitestCme, ADAPTER_LEGACY_DIAGNOSTIC_COMPUTE_TOKENS, FILES.vitestCmeSrc);
  assertIncludes(vitestCme, "CmeWorkspace", FILES.vitestCmeSrc);
}

function assertCheckerFindingCensus(): void {
  const files = [
    ...readRepoTextFiles(FILES.engineCoreTs, [".ts"]),
    ...readRepoTextFiles(FILES.engineHostNode, [".ts"]),
    ...readRepoTextFiles(FILES.checkerCli, [".ts"]),
    ...readRepoTextFiles(FILES.lspServerProviders, [".ts"]),
    ...readRepoTextFiles("packages/cme-checker/src", [".ts"]),
    ...readRepoTextFiles("packages/vitest-cme/src", [".ts"]),
    ...readRepoTextFiles(FILES.engineShadowRunner, [".rs"]),
  ];
  const matches = files.filter(({ source }) =>
    CHECKER_FINDING_CENSUS_TOKENS.some((token) => source.includes(token)),
  );
  assert.ok(matches.length > 0, "CheckerFinding census must have classified references");

  const unclassified = matches
    .map(({ relativePath }) => relativePath)
    .filter((relativePath) => classifyCheckerFindingPath(relativePath) === null);
  assert.deepEqual(
    unclassified,
    [],
    `CheckerFinding census must classify every remaining reference: ${unclassified.join(", ")}`,
  );
}

function classifyCheckerFindingPath(relativePath: string): string | null {
  if (
    relativePath.startsWith("server/lsp-server/src/providers/diagnostics.ts") ||
    relativePath.startsWith("server/lsp-server/src/providers/scss-diagnostics.ts") ||
    relativePath.startsWith("server/engine-host-node/src/source-diagnostics-query.ts") ||
    relativePath.startsWith("server/engine-host-node/src/style-diagnostics-query.ts") ||
    relativePath.startsWith("server/engine-host-node/src/checker-host/") ||
    relativePath.startsWith("server/engine-host-node/src/engine-") ||
    relativePath.startsWith("server/engine-host-node/src/historical/") ||
    relativePath.startsWith("server/engine-core-ts/src/core/checker/") ||
    relativePath.startsWith("server/engine-core-ts/src/checker-surface/") ||
    relativePath.startsWith("server/engine-core-ts/src/contracts/") ||
    relativePath.startsWith("server/engine-core-ts/src/engine-") ||
    relativePath.startsWith("server/engine-core-ts/src/historical/") ||
    relativePath.startsWith("server/checker-cli/src/")
  ) {
    return "honest-downgrade-twin";
  }
  if (
    relativePath.startsWith("packages/cme-checker/src/") ||
    relativePath.startsWith("rust/crates/engine-shadow-runner/src/")
  ) {
    return "shadow-harness";
  }
  if (relativePath.startsWith("packages/vitest-cme/src/")) {
    return "test-only";
  }
  return null;
}

function assertExactDiagnosticMap(
  source: string,
  mapName: string,
  expectedPairs: readonly (readonly [string, string])[],
  label: string,
): void {
  const mapBody = new RegExp(
    `const\\s+${mapName}\\s*=\\s*new\\s+Map\\(\\[([\\s\\S]*?)\\]\\);`,
    "u",
  ).exec(source)?.[1];
  assert.ok(mapBody, `${label} must declare ${mapName}`);
  const actualPairs = [...mapBody.matchAll(/\["([^"]+)",\s*"([^"]+)"\]/gu)].map((match) => [
    match[1],
    match[2],
  ]);
  assert.deepEqual(
    actualPairs.toSorted(pairComparator),
    expectedPairs
      .map(([queryCode, checkerCode]) => [queryCode, checkerCode])
      .toSorted(pairComparator),
    `${label} ${mapName} must mirror the exact checker/query diagnostic code pairs`,
  );
}

function pairComparator(left: readonly string[], right: readonly string[]): number {
  return left.join("\0").localeCompare(right.join("\0"));
}

function assertNone(source: string, tokens: readonly string[], label: string): void {
  for (const token of tokens) {
    assertNotIncludes(source, token, label);
  }
}

function assertIncludes(source: string, token: string, label: string): void {
  assert.ok(source.includes(token), `${label} must include ${token}`);
}

function assertNotIncludes(source: string, token: string, label: string): void {
  assert.ok(!source.includes(token), `${label} must not include ${token}`);
}

function readRepoFile(relativePath: string): string {
  return readFileSync(path.join(ROOT, relativePath), "utf8");
}

function readRepoTree(relativePath: string, extensions: readonly string[] = [".rs"]): string {
  return readRepoTextFiles(relativePath, extensions)
    .map(({ source }) => source)
    .join("\n");
}

function readRepoTextFiles(
  relativePath: string,
  extensions: readonly string[],
): readonly { readonly relativePath: string; readonly source: string }[] {
  const absolutePath = path.join(ROOT, relativePath);
  const entries = readdirSync(absolutePath, { withFileTypes: true });
  return entries.flatMap((entry) => {
    const childPath = path.join(relativePath, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === "target" || entry.name === "dist") {
        return [];
      }
      return readRepoTextFiles(childPath, extensions);
    }
    if (entry.isFile() && extensions.some((extension) => childPath.endsWith(extension))) {
      return [{ relativePath: childPath, source: readRepoFile(childPath) }];
    }
    return [];
  });
}

main();

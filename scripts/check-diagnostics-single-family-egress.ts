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
  cliDiagnostics: "rust/crates/omena-cli/src/diagnostics.rs",
  cliTests: "rust/crates/omena-cli/src/tests.rs",
  napi: "rust/crates/omena-napi/src/lib.rs",
  wasm: "rust/crates/omena-wasm/src/lib.rs",
  checkerContracts: "server/engine-core-ts/src/core/checker/contracts.ts",
  querySourceRefs: "rust/crates/omena-query/src/style/source_refs.rs",
  queryDynamicClassname: "rust/crates/omena-query/src/style/dynamic_classname.rs",
  queryStyleDiagnostics: "rust/crates/omena-query/src/style/diagnostics",
  queryStyleSourceUsage: "rust/crates/omena-query/src/style/diagnostics/source_usage.rs",
  querySourceSurfacesTests: "rust/crates/omena-query/src/tests/source_surfaces.rs",
  sourceConsumerGate: "scripts/check-source-diagnostics-query-consumer.ts",
  styleConsumerGate: "scripts/check-style-diagnostics-query-consumer.ts",
  sourceProviderTests: "test/unit/providers/diagnostics.test.ts",
  styleProviderTests: "test/unit/providers/scss-diagnostics.test.ts",
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

function main(): void {
  assertCheckerCodeMirror();
  assertRustQueryDiagnosticProducerCoverage();
  assertLspProviderQueryDiagnosticCoverage();
  assertLspSelectedQueryDiagnostics();
  assertQueryDiagnosticsShapeLock();
  assertCliNapiWasmQueryDiagnostics();
  assertLintPluginCliDiagnostics();

  process.stdout.write(
    [
      "diagnostics single-family egress ok:",
      "lsp=selected-query-owned",
      "cli=omena-query",
      "napi=omena-query",
      "wasm=omena-query",
      "eslint=cli-query",
      "stylelint=cli-query",
      "shape-lock=cli-napi-wasm",
      "checker-code-mirror=13",
      "lsp-provider-code-coverage=13",
      "legacy-plugin-fallback=absent",
    ].join(" ") + "\n",
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
    "preserves every query-owned source diagnostic code",
    FILES.sourceProviderTests,
  );
  assertIncludes(sourceProviderTests, "stableDiagnosticSnapshot", FILES.sourceProviderTests);
  assertIncludes(sourceProviderTests, "toMatchInlineSnapshot", FILES.sourceProviderTests);
  assertIncludes(
    sourceProviderTests,
    "does not fall back to checker diagnostics in the selected-query source path",
    FILES.sourceProviderTests,
  );
  assertIncludes(
    styleProviderTests,
    "preserves every query-owned style diagnostic code",
    FILES.styleProviderTests,
  );
  assertIncludes(styleProviderTests, "stableDiagnosticSnapshot", FILES.styleProviderTests);
  assertIncludes(styleProviderTests, "toMatchInlineSnapshot", FILES.styleProviderTests);
  for (const [queryCode] of SOURCE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(sourceProviderTests, `"${queryCode}"`, FILES.sourceProviderTests);
  }
  for (const [queryCode] of STYLE_DIAGNOSTIC_CODE_PAIRS) {
    assertIncludes(styleProviderTests, `"${queryCode}"`, FILES.styleProviderTests);
  }
}

function assertLspSelectedQueryDiagnostics(): void {
  const sourceProvider = readRepoFile(FILES.sourceProvider);
  const styleProvider = readRepoFile(FILES.styleProvider);

  assertNone(sourceProvider, LEGACY_QUERY_MERGE_TOKENS, FILES.sourceProvider);
  assertNone(styleProvider, LEGACY_QUERY_MERGE_TOKENS, FILES.styleProvider);

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
    'resolveSelectedQueryBackendKind() === "rust-selected-query"',
    FILES.sourceProvider,
  );
  assertIncludes(sourceProvider, "return [];", FILES.sourceProvider);
  assertIncludes(
    styleProvider,
    'resolveSelectedQueryBackendKind(runtimeDeps.env) === "rust-selected-query"',
    FILES.styleProvider,
  );
  assertIncludes(styleProvider, "return [];", FILES.styleProvider);
  assertNotIncludes(
    sourceProvider,
    '.filter((diagnostic) => diagnostic.code !== "missingModule")',
    FILES.sourceProvider,
  );
  assertIncludes(sourceProvider, "findMissingModuleCreateFileData", FILES.sourceProvider);
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

function readRepoTree(relativePath: string): string {
  const absolutePath = path.join(ROOT, relativePath);
  const entries = readdirSync(absolutePath, { withFileTypes: true });
  return entries
    .flatMap((entry) => {
      const childPath = path.join(relativePath, entry.name);
      if (entry.isDirectory()) return [readRepoTree(childPath)];
      if (entry.isFile() && childPath.endsWith(".rs")) return [readRepoFile(childPath)];
      return [];
    })
    .join("\n");
}

main();

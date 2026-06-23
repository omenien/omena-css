import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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
} as const;

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
      "legacy-plugin-fallback=absent",
    ].join(" ") + "\n",
  );
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

main();

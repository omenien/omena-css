import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");
const writeMode = process.argv.includes("--write") || !checkOnly;
const workDir = fs.mkdtempSync(path.join(os.tmpdir(), "omena-engine-v2-idl-generate-"));
const schemaDir = path.join(workDir, "schema");
let formatCounter = 0;

const generatedFiles = [
  "server/engine-core-ts/src/contracts/engine-v2-input-idl.generated.ts",
  "server/engine-core-ts/src/contracts/engine-v2-output-idl.generated.ts",
  "server/engine-core-ts/src/contracts/parse-tree-idl.generated.ts",
  "server/engine-core-ts/src/contracts/external-corpus-envelope-idl.generated.ts",
  "server/engine-core-ts/src/contracts/engine-napi-boundary-idl.generated.ts",
  "server/engine-host-node/src/engine-output-v2-idl.generated.ts",
  "server/engine-host-node/src/engine-query-v2-idl.generated.ts",
  "server/engine-host-node/src/code-action-query-idl.generated.ts",
  "server/engine-host-node/src/query-diagnostics-idl.generated.ts",
  "rust/crates/omena-engine-input-producers/src/engine_contract_v2_idl_generated.rs",
  "rust/crates/omena-parser/src/parse_tree_contract_idl_generated.rs",
  "rust/crates/omena-diff-test/src/external_corpus_envelope_idl_generated.rs",
  "rust/crates/omena-napi/src/engine_napi_contract_idl_generated.rs",
] as const;

compileTypespecContract("contracts/engine-v2");
compileTypespecContract("contracts/parse-tree");
compileTypespecContract("contracts/external-corpus-envelope");
compileTypespecContract("contracts/engine-napi");

const outputs = new Map<string, string>([
  [
    "server/engine-core-ts/src/contracts/engine-v2-input-idl.generated.ts",
    renderGeneratedTypescript("EngineInputV2.json"),
  ],
  [
    "server/engine-core-ts/src/contracts/engine-v2-output-idl.generated.ts",
    renderGeneratedTypescript("EngineOutputV2.json"),
  ],
  [
    "server/engine-core-ts/src/contracts/parse-tree-idl.generated.ts",
    renderGeneratedTypescript("ParseTreeNodeV0.json", "contracts/parse-tree/main.tsp"),
  ],
  [
    "server/engine-core-ts/src/contracts/external-corpus-envelope-idl.generated.ts",
    renderGeneratedTypescript(
      "ExternalCorpusEnvelopeV1.json",
      "contracts/external-corpus-envelope/main.tsp",
    ),
  ],
  [
    "server/engine-core-ts/src/contracts/engine-napi-boundary-idl.generated.ts",
    renderGeneratedTypescript("EngineNapiBoundarySurfaceV0.json", "contracts/engine-napi/main.tsp"),
  ],
  ["server/engine-host-node/src/engine-output-v2-idl.generated.ts", renderHostOutputMirror()],
  ["server/engine-host-node/src/engine-query-v2-idl.generated.ts", renderHostQueryMirror()],
  [
    "server/engine-host-node/src/code-action-query-idl.generated.ts",
    renderGeneratedTypescript("OmenaQueryCodeActionPlanV0.json"),
  ],
  [
    "server/engine-host-node/src/query-diagnostics-idl.generated.ts",
    renderGeneratedTypescript("OmenaQueryDiagnosticsForFileV0.json"),
  ],
  [
    "rust/crates/omena-engine-input-producers/src/engine_contract_v2_idl_generated.rs",
    formatRust(renderRustContractModule()),
  ],
  [
    "rust/crates/omena-parser/src/parse_tree_contract_idl_generated.rs",
    formatRust(renderRustParseTreeContractModule()),
  ],
  [
    "rust/crates/omena-diff-test/src/external_corpus_envelope_idl_generated.rs",
    formatRust(renderRustExternalCorpusEnvelopeModule()),
  ],
  [
    "rust/crates/omena-napi/src/engine_napi_contract_idl_generated.rs",
    formatRust(renderRustEngineNapiBoundaryModule()),
  ],
]);

assert.deepEqual([...outputs.keys()], [...generatedFiles]);

const staleFiles: string[] = [];
for (const [relativePath, source] of outputs) {
  const absolutePath = path.join(repoRoot, relativePath);
  if (checkOnly) {
    const actual = fs.existsSync(absolutePath) ? fs.readFileSync(absolutePath, "utf8") : "";
    if (actual !== source) {
      staleFiles.push(relativePath);
    }
  } else if (writeMode) {
    fs.mkdirSync(path.dirname(absolutePath), { recursive: true });
    fs.writeFileSync(absolutePath, source, "utf8");
  }
}

if (staleFiles.length > 0) {
  assert.fail(
    `Engine V2 IDL generated files are stale:\n${staleFiles.map((file) => `- ${file}`).join("\n")}`,
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "engine-v2.contract-idl-generator",
      mode: checkOnly ? "check" : "write",
      source: [
        "contracts/engine-v2/main.tsp",
        "contracts/parse-tree/main.tsp",
        "contracts/external-corpus-envelope/main.tsp",
        "contracts/engine-napi/main.tsp",
      ],
      generatedFiles,
    },
    null,
    2,
  )}\n`,
);

function run(command: string, args: readonly string[]): void {
  execFileSync(command, [...args], { cwd: repoRoot, stdio: "inherit" });
}

function output(command: string, args: readonly string[]): string {
  return execFileSync(command, [...args], {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
}

function compileTypespecContract(contractDir: string): void {
  run("pnpm", [
    "exec",
    "tsp",
    "compile",
    contractDir,
    "--emit",
    "@typespec/json-schema",
    "--option",
    `@typespec/json-schema.emitter-output-dir=${schemaDir}`,
    "--option",
    "@typespec/json-schema.file-type=json",
    "--option",
    "@typespec/json-schema.emitAllModels=true",
    "--option",
    "@typespec/json-schema.polymorphic-models-strategy=oneOf",
    "--option",
    "@typespec/json-schema.seal-object-schemas=true",
  ]);
}

function renderGeneratedTypescript(
  schemaFile: string,
  sourceContract = "contracts/engine-v2/main.tsp",
): string {
  const generated = output("pnpm", [
    "exec",
    "json2ts",
    "-i",
    path.join(schemaDir, schemaFile),
    "--cwd",
    schemaDir,
    "--unknownAny",
    "false",
    "--no-additionalProperties",
  ]);
  const body = generated
    .replace(/^\/\* eslint-disable \*\/\n\/\*\*[\s\S]*?\*\/\n\n/u, "")
    .replace(/\bany\[\]/gu, "unknown[]")
    .replace(/\bany\b/gu, "unknown");
  return formatTypescript(
    withGeneratedHeader(applyReadonlyTypescriptSurface(body), sourceContract),
  );
}

function withGeneratedHeader(
  body: string,
  sourceContract = "contracts/engine-v2/main.tsp",
): string {
  return `/* eslint-disable */
// @generated by scripts/generate-engine-v2-contract-idl.ts from ${sourceContract}.
// Do not edit this file by hand.

${body.trimEnd()}
`;
}

function applyReadonlyTypescriptSurface(source: string): string {
  return source
    .split("\n")
    .map((line) => {
      const property = /^(\s{2})([A-Za-z_$][\w$]*\??): (.+);$/u.exec(line);
      if (!property) {
        return line;
      }
      const [, indent, name, type] = property;
      return `${indent}readonly ${name}: ${readonlyArrayType(type)};`;
    })
    .join("\n");
}

function readonlyArrayType(type: string): string {
  return type.replace(/\b([A-Za-z_$][\w$]*)\[\]/gu, "readonly $1[]");
}

function renderHostOutputMirror(): string {
  return formatTypescript(
    withGeneratedHeader(`export type {
  EngineOutputV2Json as HostEngineOutputV2Json,
  QueryResultV2Json as HostQueryResultV2Json,
  ExpressionSemanticsQueryResultV2Json as HostExpressionSemanticsQueryResultV2Json,
  SourceExpressionResolutionQueryResultV2Json as HostSourceExpressionResolutionQueryResultV2Json,
  SelectorUsageQueryResultV2Json as HostSelectorUsageQueryResultV2Json,
} from "../../engine-core-ts/src/contracts/engine-v2-output-idl.generated";
`),
  );
}

function renderHostQueryMirror(): string {
  return formatTypescript(
    withGeneratedHeader(`export type {
  QueryResultV2Json as EngineQueryResultV2Json,
  ExpressionSemanticsQueryResultV2Json,
  SourceExpressionResolutionQueryResultV2Json,
  SelectorUsageQueryResultV2Json,
} from "../../engine-core-ts/src/contracts/engine-v2-output-idl.generated";
`),
  );
}

function formatTypescript(source: string): string {
  formatCounter += 1;
  const tempFile = path.join(workDir, `generated-${formatCounter}.ts`);
  fs.writeFileSync(tempFile, source, "utf8");
  run("pnpm", ["exec", "oxfmt", tempFile]);
  return fs.readFileSync(tempFile, "utf8");
}

function formatRust(source: string): string {
  formatCounter += 1;
  const tempFile = path.join(workDir, `generated-${formatCounter}.rs`);
  fs.writeFileSync(tempFile, source, "utf8");
  run("rustfmt", ["--edition", "2024", tempFile]);
  return fs.readFileSync(tempFile, "utf8");
}

function renderRustContractModule(): string {
  return String.raw`// @generated by scripts/generate-engine-v2-contract-idl.ts from contracts/engine-v2/main.tsp.
// Do not edit this file by hand.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineWorkspaceV1Json {
    pub root: String,
    pub classname_transform: ClassnameTransformModeJson,
    pub settings_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClassnameTransformModeJson {
    #[serde(rename = "asIs")]
    AsIs,
    #[serde(rename = "camelCase")]
    CamelCase,
    #[serde(rename = "camelCaseOnly")]
    CamelCaseOnly,
    #[serde(rename = "dashes")]
    Dashes,
    #[serde(rename = "dashesOnly")]
    DashesOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAnalysisInputV2Json {
    pub file_path: String,
    pub document: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_graph: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleAnalysisInputV2Json {
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub document: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StringTypeFactKindV2Json {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "finiteSet")]
    FiniteSet,
    #[serde(rename = "constrained")]
    Constrained,
    #[serde(rename = "top")]
    Top,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StringConstraintKindV2Json {
    #[serde(rename = "prefix")]
    Prefix,
    #[serde(rename = "suffix")]
    Suffix,
    #[serde(rename = "prefixSuffix")]
    PrefixSuffix,
    #[serde(rename = "charInclusion")]
    CharInclusion,
    #[serde(rename = "composite")]
    Composite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StringTypeFactsV2Json {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_len: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_len: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_must: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_may: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub may_include_other_chars: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeFactControlFlowBlockV2Json {
    pub id: String,
    pub kind: String,
    pub transfer_kind: String,
    pub successor_block_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_ordinal: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facts: Option<StringTypeFactsV2Json>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeFactControlFlowGraphV2Json {
    pub entry_block_id: String,
    pub blocks: Vec<TypeFactControlFlowBlockV2Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeFactEntryV2Json {
    pub file_path: String,
    pub expression_id: String,
    pub facts: StringTypeFactsV2Json,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_flow_graph: Option<TypeFactControlFlowGraphV2Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInputV2Json {
    pub version: String,
    pub workspace: EngineWorkspaceV1Json,
    pub sources: Vec<SourceAnalysisInputV2Json>,
    pub styles: Vec<StyleAnalysisInputV2Json>,
    pub type_facts: Vec<TypeFactEntryV2Json>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueDomainKindV2Json {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "finiteSet")]
    FiniteSet,
    #[serde(rename = "constrained")]
    Constrained,
    #[serde(rename = "top")]
    Top,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertaintyShapeKindV2Json {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "boundedFinite")]
    BoundedFinite,
    #[serde(rename = "constrained")]
    Constrained,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueDomainDerivationStepV2Json {
    pub operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refinement_kind: Option<String>,
    pub result_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_provenance: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueDomainDerivationV2Json {
    pub schema_version: String,
    pub product: String,
    pub input_fact_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_constraint_kind: Option<String>,
    pub input_value_count: i32,
    pub reduced_kind: String,
    pub steps: Vec<ValueDomainDerivationStepV2Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueDomainProvenanceNodeV2Json {
    pub operation: String,
    pub result_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub reason: String,
    pub children: Vec<ValueDomainProvenanceNodeV2Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueDomainProvenanceTreeV2Json {
    pub schema_version: String,
    pub product: String,
    pub value_kind: String,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_provenance: Option<String>,
    pub root: ValueDomainProvenanceNodeV2Json,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpressionSemanticsPayloadV2Json {
    pub expression_id: String,
    pub expression_kind: String,
    pub style_file_path: Option<String>,
    pub selector_names: Vec<String>,
    pub candidate_names: Vec<String>,
    pub finite_values: Option<Vec<String>>,
    pub value_domain_kind: ValueDomainKindV2Json,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_constraint_kind: Option<StringConstraintKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_min_len: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_max_len: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_char_must: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_char_may: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_may_include_other_chars: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_domain_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_domain_derivation: Option<ValueDomainDerivationV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_domain_provenance_tree: Option<ValueDomainProvenanceTreeV2Json>,
    pub selector_certainty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_certainty_shape_kind: Option<CertaintyShapeKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_constraint_kind: Option<StringConstraintKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_certainty_shape_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_certainty_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty_shape_kind: Option<CertaintyShapeKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty_constraint_kind: Option<StringConstraintKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty_shape_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceExpressionResolutionPayloadV2Json {
    pub expression_id: String,
    pub style_file_path: Option<String>,
    pub selector_names: Vec<String>,
    pub finite_values: Option<Vec<String>>,
    pub selector_certainty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_certainty_shape_kind: Option<CertaintyShapeKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_constraint_kind: Option<StringConstraintKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_certainty_shape_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector_certainty_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty_shape_kind: Option<CertaintyShapeKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty_constraint_kind: Option<StringConstraintKindV2Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_min_len: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_max_len: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_char_must: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_char_may: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_may_include_other_chars: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty_shape_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_certainty_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorUsagePayloadV2Json {
    pub canonical_name: String,
    pub total_references: i32,
    pub direct_reference_count: i32,
    pub editable_direct_reference_count: i32,
    pub exact_reference_count: i32,
    pub inferred_or_better_reference_count: i32,
    pub has_expanded_references: bool,
    pub has_style_dependency_references: bool,
    pub has_any_references: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum QueryResultV2Json {
    #[serde(rename = "expression-semantics")]
    ExpressionSemantics {
        file_path: String,
        query_id: String,
        payload: ExpressionSemanticsPayloadV2Json,
    },
    #[serde(rename = "source-expression-resolution")]
    SourceExpressionResolution {
        file_path: String,
        query_id: String,
        payload: SourceExpressionResolutionPayloadV2Json,
    },
    #[serde(rename = "selector-usage")]
    SelectorUsage {
        file_path: String,
        query_id: String,
        payload: SelectorUsagePayloadV2Json,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPositionJsonV2Json {
    pub line: i32,
    pub character: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRangeJsonV2Json {
    pub start: TextPositionJsonV2Json,
    pub end: TextPositionJsonV2Json,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedTextEditJsonV2Json {
    pub uri: String,
    pub range: TextRangeJsonV2Json,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRewritePlanJsonV2Json {
    pub target: serde_json::Value,
    pub edits: Vec<PlannedTextEditJsonV2Json>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckerReportJsonV1Json {
    pub version: String,
    pub findings: Vec<serde_json::Value>,
    pub summary: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineOutputV2Json {
    pub version: String,
    pub query_results: Vec<QueryResultV2Json>,
    pub rewrite_plans: Vec<TextRewritePlanJsonV2Json>,
    pub checker_report: CheckerReportJsonV1Json,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CodeActionPlanKindJson {
    #[serde(rename = "quickfix")]
    Quickfix,
    #[serde(rename = "refactor.extract")]
    RefactorExtract,
    #[serde(rename = "refactor.inline")]
    RefactorInline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryWorkspaceTextEditV0Json {
    pub uri: String,
    pub range: serde_json::Value,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCodeActionV0Json {
    pub title: String,
    pub kind: CodeActionPlanKindJson,
    pub edits: Vec<OmenaQueryWorkspaceTextEditV0Json>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCodeActionPlanV0Json {
    pub schema_version: String,
    pub product: String,
    pub file_uri: String,
    pub file_kind: String,
    pub action_count: i32,
    pub actions: Vec<OmenaQueryCodeActionV0Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready_surfaces: Option<Vec<String>>,
}
`
    .trimEnd()
    .concat("\n");
}

function renderRustParseTreeContractModule(): string {
  return String.raw`// @generated by scripts/generate-engine-v2-contract-idl.ts from contracts/parse-tree/main.tsp.
// Do not edit this file by hand.

//! Rust projection of the parse-tree contract IDL used by parser boundaries.

#![allow(dead_code)]

use crate::{ParserByteSpanV0, ParserRangeV0};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Recursive parse-tree node projected from the parse-tree contract IDL.
pub struct ParseTreeNodeV0 {
    pub kind: String,
    pub byte_span: ParserByteSpanV0,
    pub range: ParserRangeV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bogus: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub children: Vec<ParseTreeNodeV0>,
}
`
    .trimEnd()
    .concat("\n");
}

function renderRustExternalCorpusEnvelopeModule(): string {
  return String.raw`// @generated by scripts/generate-engine-v2-contract-idl.ts from contracts/external-corpus-envelope/main.tsp.
// Do not edit this file by hand.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalCorpusStageV1Json {
    #[serde(rename = "stage1-advisory")]
    Stage1Advisory,
    #[serde(rename = "stage2-blocking")]
    Stage2Blocking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalCorpusDialectV1Json {
    #[serde(rename = "css")]
    Css,
    #[serde(rename = "scss")]
    Scss,
    #[serde(rename = "sass")]
    Sass,
    #[serde(rename = "less")]
    Less,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalCorpusExpectationKindV1Json {
    #[serde(rename = "static-must-match")]
    StaticMustMatch,
    #[serde(rename = "expected-sound-bail")]
    ExpectedSoundBail,
    #[serde(rename = "parser-recovery")]
    ParserRecovery,
    #[serde(rename = "out-of-scope")]
    OutOfScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusSourcePinV1Json {
    pub repository: String,
    pub pin: String,
    pub sparse_paths: Vec<String>,
    pub helper_classes: Vec<String>,
    pub layout_dependent_helpers_excluded: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusKnownFailurePolicyRefV1Json {
    pub path: String,
    pub schema_version: String,
    pub stage2_blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusGenerationProvenanceV1Json {
    pub tool: String,
    pub selection_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_pin_refs: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusSparsePathFixtureCountV1Json {
    pub sparse_path: String,
    pub fixture_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusChunkV1Json {
    pub chunk_id: String,
    pub path: String,
    pub stage: ExternalCorpusStageV1Json,
    pub sha256: String,
    pub fixture_count: i32,
    pub sparse_path_fixture_counts: Vec<ExternalCorpusSparsePathFixtureCountV1Json>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusProvenanceV1Json {
    pub generation_tool: String,
    pub selection_path: String,
    pub oracle_pin_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusGreenRunV1Json {
    pub date: String,
    pub commit: String,
    pub fixture_count: i32,
    pub chunk_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_olw: Option<i32>,
    pub critical_regression_count: i32,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusEnvelopeV1Json {
    pub schema_version: String,
    pub product: String,
    pub stage: ExternalCorpusStageV1Json,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dialect: Option<ExternalCorpusDialectV1Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expectation_kind: Option<ExternalCorpusExpectationKindV1Json>,
    pub source: ExternalCorpusSourcePinV1Json,
    pub known_failure_policy: ExternalCorpusKnownFailurePolicyRefV1Json,
    pub generation: ExternalCorpusGenerationProvenanceV1Json,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ExternalCorpusProvenanceV1Json>,
    pub sparse_path_fixture_counts: Vec<ExternalCorpusSparsePathFixtureCountV1Json>,
    pub chunks: Vec<ExternalCorpusChunkV1Json>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub green_runs: Option<Vec<ExternalCorpusGreenRunV1Json>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusDifferentialManifestV1Json {
    pub schema_version: String,
    pub product: String,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_verified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compilers: Option<serde_json::Value>,
    pub fixtures: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCorpusDisagreementLedgerV1Json {
    pub schema_version: String,
    pub product: String,
    pub allowlist_count: i32,
    pub entries: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCorpusSeedPolicyV1Toml {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    #[serde(rename = "corpus_manifest")]
    pub corpus_manifest: String,
    pub stage: String,
    #[serde(rename = "stage2_blocking")]
    pub stage2_blocking: bool,
    #[serde(rename = "source_pin")]
    pub source_pin: String,
    #[serde(rename = "review_interval_days")]
    pub review_interval_days: i32,
    #[serde(rename = "required_min_fixture_count_for_stage2")]
    pub required_min_fixture_count_for_stage2: i32,
    #[serde(rename = "required_consecutive_green_runs")]
    pub required_consecutive_green_runs: i32,
    #[serde(rename = "consecutive_green_runs")]
    pub consecutive_green_runs: i32,
    #[serde(default)]
    pub green_run: Vec<ExternalCorpusSeedPolicyGreenRunV1Toml>,
    #[serde(default)]
    pub subtest: Vec<ExternalCorpusSeedPolicySubtestV1Toml>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCorpusSeedPolicyGreenRunV1Toml {
    pub date: String,
    pub commit: String,
    #[serde(rename = "fixture_count")]
    pub fixture_count: i32,
    #[serde(rename = "chunk_sha256")]
    pub chunk_sha256: String,
    #[serde(rename = "outcome_olw")]
    pub outcome_olw: i32,
    #[serde(rename = "critical_regression_count")]
    pub critical_regression_count: i32,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCorpusSeedPolicySubtestV1Toml {
    pub fixture: String,
    pub name: String,
    pub status: String,
    pub reason: String,
    pub issue: String,
    pub since: String,
    #[serde(rename = "review_after")]
    pub review_after: String,
}
`
    .trimEnd()
    .concat("\n");
}

function renderRustEngineNapiBoundaryModule(): string {
  return String.raw`// @generated by scripts/generate-engine-v2-contract-idl.ts from contracts/engine-napi/main.tsp.
// Do not edit this file by hand.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub type EngineNapiTransformExecutionContextV0Json =
    omena_query::OmenaQueryTransformExecutionContextV0;
pub type EngineNapiTargetTransformOptionsV0Json =
    omena_query::OmenaQueryTargetTransformOptionsV0;
pub type EngineNapiStyleSourceInputV0Json = omena_query::OmenaQueryStyleSourceInputV0;
pub type EngineNapiStylePackageManifestV0Json =
    omena_query::OmenaQueryStylePackageManifestV0;
pub type EngineNapiSourceDocumentInputV0Json = omena_query::OmenaQuerySourceDocumentInputV0;
pub type EngineNapiSourceMissingSelectorDiagnosticCandidateV0Json =
    omena_query::OmenaQuerySourceMissingSelectorDiagnosticCandidateV0;
pub type EngineNapiClassnamesBindBindingsV0Json = Vec<String>;
pub type EngineNapiConsumerBuildSummaryV0Json = omena_query::OmenaQueryConsumerBuildSummaryV0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineNapiBoundaryErrorKindV0Json {
    #[serde(rename = "parse-error")]
    ParseError,
    #[serde(rename = "serialize-error")]
    SerializeError,
    #[serde(rename = "unsupported-mode")]
    UnsupportedMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineNapiBoundaryErrorV0Json {
    pub kind: EngineNapiBoundaryErrorKindV0Json,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineNapiEngineInputV2Json {
    pub version: String,
    pub workspace: serde_json::Value,
    pub sources: Vec<serde_json::Value>,
    pub styles: Vec<serde_json::Value>,
    pub type_facts: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineNapiSourceImportedStyleBindingInputV0Json {
    pub binding: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineNapiBoundarySurfaceV0Json {
    pub context: EngineNapiTransformExecutionContextV0Json,
    pub target_options: EngineNapiTargetTransformOptionsV0Json,
    pub input: EngineNapiEngineInputV2Json,
    pub sources: Vec<EngineNapiStyleSourceInputV0Json>,
    pub package_manifests: Vec<EngineNapiStylePackageManifestV0Json>,
    pub source_documents: Vec<EngineNapiSourceDocumentInputV0Json>,
    pub candidates: Vec<EngineNapiSourceMissingSelectorDiagnosticCandidateV0Json>,
    pub imported_style_bindings: Vec<EngineNapiSourceImportedStyleBindingInputV0Json>,
    pub classnames_bind_bindings: EngineNapiClassnamesBindBindingsV0Json,
    pub build_summary: serde_json::Value,
    pub boundary_error: EngineNapiBoundaryErrorV0Json,
}
`
    .trimEnd()
    .concat("\n");
}

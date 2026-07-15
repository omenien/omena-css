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
const packageJson = JSON.parse(fs.readFileSync(path.join(repoRoot, "package.json"), "utf8")) as {
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
};
const webrefCssVersion =
  packageJson.devDependencies?.["@webref/css"] ??
  packageJson.dependencies?.["@webref/css"] ??
  "unknown";

const propertyMetadataRows = [
  {
    propertyId: "background-color",
    canonicalName: "background-color",
    inherited: false,
    initialValue: "transparent",
  },
  {
    propertyId: "border-color",
    canonicalName: "border-color",
    inherited: false,
    initialValue: "transparent",
  },
  {
    propertyId: "border-style",
    canonicalName: "border-style",
    inherited: false,
    initialValue: "none",
  },
  {
    propertyId: "border-width",
    canonicalName: "border-width",
    inherited: false,
    initialValue: "0",
  },
  { propertyId: "box-shadow", canonicalName: "box-shadow", inherited: false, initialValue: "none" },
  {
    propertyId: "caret-color",
    canonicalName: "caret-color",
    inherited: false,
    initialValue: "transparent",
  },
  { propertyId: "color", canonicalName: "color", inherited: true, initialValue: "canvastext" },
  { propertyId: "cursor", canonicalName: "cursor", inherited: true, initialValue: "auto" },
  { propertyId: "direction", canonicalName: "direction", inherited: true, initialValue: "initial" },
  { propertyId: "display", canonicalName: "display", inherited: false, initialValue: "none" },
  { propertyId: "font", canonicalName: "font", inherited: true, initialValue: "initial" },
  {
    propertyId: "font-family",
    canonicalName: "font-family",
    inherited: true,
    initialValue: "serif",
  },
  { propertyId: "font-size", canonicalName: "font-size", inherited: true, initialValue: "medium" },
  {
    propertyId: "font-style",
    canonicalName: "font-style",
    inherited: true,
    initialValue: "normal",
  },
  {
    propertyId: "font-variant",
    canonicalName: "font-variant",
    inherited: true,
    initialValue: "normal",
  },
  {
    propertyId: "font-weight",
    canonicalName: "font-weight",
    inherited: true,
    initialValue: "normal",
  },
  {
    propertyId: "letter-spacing",
    canonicalName: "letter-spacing",
    inherited: true,
    initialValue: "normal",
  },
  {
    propertyId: "line-height",
    canonicalName: "line-height",
    inherited: true,
    initialValue: "normal",
  },
  { propertyId: "margin", canonicalName: "margin", inherited: false, initialValue: "0" },
  { propertyId: "opacity", canonicalName: "opacity", inherited: false, initialValue: "1" },
  {
    propertyId: "outline-color",
    canonicalName: "outline-color",
    inherited: false,
    initialValue: "transparent",
  },
  { propertyId: "padding", canonicalName: "padding", inherited: false, initialValue: "0" },
  { propertyId: "text-align", canonicalName: "text-align", inherited: true, initialValue: "start" },
  { propertyId: "text-indent", canonicalName: "text-indent", inherited: true, initialValue: "0" },
  {
    propertyId: "text-shadow",
    canonicalName: "text-shadow",
    inherited: false,
    initialValue: "none",
  },
  {
    propertyId: "text-transform",
    canonicalName: "text-transform",
    inherited: true,
    initialValue: "none",
  },
  {
    propertyId: "visibility",
    canonicalName: "visibility",
    inherited: true,
    initialValue: "visible",
  },
  {
    propertyId: "white-space",
    canonicalName: "white-space",
    inherited: true,
    initialValue: "normal",
  },
  {
    propertyId: "word-spacing",
    canonicalName: "word-spacing",
    inherited: true,
    initialValue: "normal",
  },
] as const;

const generatedFiles = [
  "server/engine-core-ts/src/contracts/engine-v2-input-idl.generated.ts",
  "server/engine-core-ts/src/contracts/engine-v2-output-idl.generated.ts",
  "server/engine-core-ts/src/contracts/parse-tree-idl.generated.ts",
  "server/engine-core-ts/src/contracts/external-corpus-envelope-idl.generated.ts",
  "server/engine-core-ts/src/contracts/property-metadata-idl.generated.ts",
  "server/engine-core-ts/src/contracts/engine-napi-boundary-idl.generated.ts",
  "server/engine-core-ts/src/contracts/engine-sdk-workflow-idl.generated.ts",
  "server/engine-host-node/src/engine-output-v2-idl.generated.ts",
  "server/engine-host-node/src/engine-query-v2-idl.generated.ts",
  "server/engine-host-node/src/code-action-query-idl.generated.ts",
  "server/engine-host-node/src/query-diagnostics-idl.generated.ts",
  "rust/crates/omena-engine-input-producers/src/engine_contract_v2_idl_generated.rs",
  "rust/crates/omena-parser/src/parse_tree_contract_idl_generated.rs",
  "rust/crates/omena-diff-test/src/external_corpus_envelope_idl_generated.rs",
  "rust/crates/omena-cascade/src/property_metadata_idl_generated.rs",
  "rust/crates/omena-napi/src/engine_napi_contract_idl_generated.rs",
  "rust/crates/omena-query/src/sdk_workflow_contract_idl_generated.rs",
] as const;

compileTypespecContract("contracts/engine-v2");
compileTypespecContract("contracts/parse-tree");
compileTypespecContract("contracts/external-corpus-envelope");
compileTypespecContract("contracts/property-metadata");
compileTypespecContract("contracts/engine-napi");
compileTypespecContract("contracts/engine-sdk-workflow");

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
    "server/engine-core-ts/src/contracts/property-metadata-idl.generated.ts",
    renderGeneratedTypescript("CssPropertyMetadataV1.json", "contracts/property-metadata/main.tsp"),
  ],
  [
    "server/engine-core-ts/src/contracts/engine-napi-boundary-idl.generated.ts",
    renderGeneratedTypescript("EngineNapiBoundarySurfaceV0.json", "contracts/engine-napi/main.tsp"),
  ],
  [
    "server/engine-core-ts/src/contracts/engine-sdk-workflow-idl.generated.ts",
    renderSdkWorkflowTypescript(),
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
    "rust/crates/omena-cascade/src/property_metadata_idl_generated.rs",
    formatRust(renderRustPropertyMetadataModule()),
  ],
  [
    "rust/crates/omena-napi/src/engine_napi_contract_idl_generated.rs",
    formatRust(renderRustEngineNapiBoundaryModule()),
  ],
  [
    "rust/crates/omena-query/src/sdk_workflow_contract_idl_generated.rs",
    formatRust(renderRustSdkWorkflowContractModule()),
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
        "contracts/property-metadata/main.tsp",
        "contracts/engine-napi/main.tsp",
        "contracts/engine-sdk-workflow/main.tsp",
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

function renderSdkWorkflowTypescript(): string {
  const source = renderGeneratedTypescript(
    "OmenaSdkWorkflowSurfaceV0.json",
    "contracts/engine-sdk-workflow/main.tsp",
  );
  const withTypedStringRecords = source.replace(
    "export interface RecordStringJson {}",
    "export type RecordStringJson = Readonly<Record<string, string>>;",
  );
  assert.ok(
    withTypedStringRecords !== source,
    "workflow contract must preserve string record value types",
  );
  const withoutInlineBoundaryTypes = withTypedStringRecords.replace(
    /export interface EngineNapi[A-Za-z0-9]+Json \{\n(?:  .*\n)*\}\n/gu,
    "",
  );
  assert.ok(
    withoutInlineBoundaryTypes !== withTypedStringRecords,
    "workflow contract must consume engine-napi boundary types",
  );
  assert.ok(
    !withoutInlineBoundaryTypes.includes("export interface EngineNapi"),
    "workflow contract must not redeclare engine-napi boundary types",
  );
  return formatTypescript(
    withoutInlineBoundaryTypes.replace(
      "// Do not edit this file by hand.\n",
      `// Do not edit this file by hand.\n\nimport type {\n  EngineNapiEngineInputV2Json,\n  EngineNapiStylePackageManifestV0Json,\n  EngineNapiStyleSourceInputV0Json,\n  EngineNapiTransformExecutionContextV0Json,\n} from "./engine-napi-boundary-idl.generated";\n`,
    ),
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

function rustString(value: string): string {
  return JSON.stringify(value);
}

function rustBool(value: boolean): string {
  return value ? "true" : "false";
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
  return String.raw`//! Generated external-corpus envelope contract types (sass-spec differential lanes).
// @generated by scripts/generate-engine-v2-contract-idl.ts from contracts/external-corpus-envelope/main.tsp.
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

function renderRustPropertyMetadataModule(): string {
  const rows = propertyMetadataRows
    .map(
      (row) => `    CssPropertyMetadataRecordStaticV1 {
        property_id: ${rustString(row.propertyId)},
        canonical_name: ${rustString(row.canonicalName)},
        inherited: ${rustBool(row.inherited)},
        initial_value: ${rustString(row.initialValue)},
    },`,
    )
    .join("\n");

  return String.raw`//! Generated CSS property-metadata contract types (webref-derived authority).
// @generated by scripts/generate-engine-v2-contract-idl.ts from contracts/property-metadata/main.tsp.
// Do not edit this file by hand.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssPropertyMetadataV1Json {
    pub schema_version: String,
    pub product: String,
    pub source: CssPropertyMetadataSourceV1Json,
    pub custom_property_policy: CssCustomPropertyPolicyV1Json,
    pub properties: Vec<CssPropertyMetadataRecordV1Json>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssPropertyMetadataSourceV1Json {
    pub package: String,
    pub version: String,
    pub source_path: String,
    pub tool: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssCustomPropertyPolicyV1Json {
    pub inherited: bool,
    pub initial_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssPropertyMetadataRecordV1Json {
    pub property_id: String,
    pub canonical_name: String,
    pub inherited: bool,
    pub initial_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssPropertyMetadataStaticV1 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source: CssPropertyMetadataSourceStaticV1,
    pub custom_property_policy: CssCustomPropertyPolicyStaticV1,
    pub properties: &'static [CssPropertyMetadataRecordStaticV1],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssPropertyMetadataSourceStaticV1 {
    pub package: &'static str,
    pub version: &'static str,
    pub source_path: &'static str,
    pub tool: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssCustomPropertyPolicyStaticV1 {
    pub inherited: bool,
    pub initial_value: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssPropertyMetadataRecordStaticV1 {
    pub property_id: &'static str,
    pub canonical_name: &'static str,
    pub inherited: bool,
    pub initial_value: &'static str,
}

/// Committed webref-derived property metadata authority (schema v1).
pub const CSS_PROPERTY_METADATA_V1: CssPropertyMetadataStaticV1 = CssPropertyMetadataStaticV1 {
    schema_version: "1",
    product: "omena-cascade.property-metadata",
    source: CssPropertyMetadataSourceStaticV1 {
        package: "@webref/css",
        version: ${rustString(webrefCssVersion)},
        source_path: "rust/crates/omena-spec-audit/data/webref-grammar.json",
        tool: "scripts/generate-engine-v2-contract-idl.ts",
    },
    custom_property_policy: CssCustomPropertyPolicyStaticV1 {
        inherited: true,
        initial_value: "guaranteed-invalid",
    },
    properties: CSS_PROPERTY_METADATA_RECORDS_V1,
};

/// Committed per-property records backing [CSS_PROPERTY_METADATA_V1].
pub const CSS_PROPERTY_METADATA_RECORDS_V1: &[CssPropertyMetadataRecordStaticV1] = &[
${rows}
];
`
    .trimEnd()
    .concat("\n");
}

function renderRustEngineNapiBoundaryModule(): string {
  return String.raw`// @generated by scripts/generate-engine-v2-contract-idl.ts from contracts/engine-napi/main.tsp.
// Do not edit this file by hand.

#![allow(dead_code)]

use napi_derive::napi;
use serde::{Deserialize, Serialize};

pub type EngineNapiQueryTransformExecutionContextV0Json =
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
pub type EngineNapiQueryConsumerBuildSummaryV0Json =
    omena_query::OmenaQueryConsumerBuildSummaryV0;

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

fn boundary_error(
    kind: EngineNapiBoundaryErrorKindV0Json,
    message: String,
    function_name: &'static str,
) -> napi::Error {
    let fallback = format!("{kind:?}: {message}");
    let body = EngineNapiBoundaryErrorV0Json {
        kind,
        message,
        function_name: Some(function_name.to_string()),
    };
    napi::Error::from_reason(serde_json::to_string(&body).unwrap_or(fallback))
}

fn parse_boundary_value<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
    function_name: &'static str,
) -> napi::Result<T> {
    serde_json::from_value(value).map_err(|error| {
        boundary_error(
            EngineNapiBoundaryErrorKindV0Json::ParseError,
            error.to_string(),
            function_name,
        )
    })
}

fn serialize_boundary_value<T: Serialize>(
    value: &T,
    function_name: &'static str,
) -> napi::Result<serde_json::Value> {
    serde_json::to_value(value).map_err(|error| {
        boundary_error(
            EngineNapiBoundaryErrorKindV0Json::SerializeError,
            error.to_string(),
            function_name,
        )
    })
}

fn parse_boundary_json<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
    function_name: &'static str,
) -> napi::Result<T> {
    parse_boundary_value(value, function_name)
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[napi(object)]
#[serde(default, rename_all = "camelCase")]
pub struct EngineNapiTransformExecutionContextV0Json {
    #[napi(js_name = "dropDarkModeMediaQueries")]
    pub drop_dark_mode_media_queries: Option<bool>,
    #[napi(js_name = "supportsTargetCapability")]
    pub supports_target_capability: Option<serde_json::Value>,
    #[napi(js_name = "vendorPrefixPolicy")]
    pub vendor_prefix_policy: Option<serde_json::Value>,
    #[napi(js_name = "reachableClassNames")]
    pub reachable_class_names: Option<Vec<String>>,
    #[napi(js_name = "reachableKeyframeNames")]
    pub reachable_keyframe_names: Option<Vec<String>>,
    #[napi(js_name = "reachableValueNames")]
    pub reachable_value_names: Option<Vec<String>>,
    #[napi(js_name = "reachableCustomPropertyNames")]
    pub reachable_custom_property_names: Option<Vec<String>>,
    #[napi(js_name = "scssModuleEvaluation")]
    pub scss_module_evaluation: Option<serde_json::Value>,
    #[napi(js_name = "lessModuleEvaluation")]
    pub less_module_evaluation: Option<serde_json::Value>,
    #[napi(js_name = "importInlines")]
    pub import_inlines: Option<Vec<serde_json::Value>>,
    #[napi(js_name = "classNameRewrites")]
    pub class_name_rewrites: Option<Vec<serde_json::Value>>,
    #[napi(js_name = "cssModuleComposesResolutions")]
    pub css_module_composes_resolutions: Option<Vec<serde_json::Value>>,
    #[napi(js_name = "cssModuleValueResolutions")]
    pub css_module_value_resolutions: Option<Vec<serde_json::Value>>,
    #[napi(js_name = "designTokenRoutes")]
    pub design_token_routes: Option<Vec<serde_json::Value>>,
}

impl EngineNapiTransformExecutionContextV0Json {
    pub fn from_boundary_value(value: serde_json::Value) -> napi::Result<Self> {
        parse_boundary_value(value, "buildStyleSourceWithContext")
    }

    pub fn try_into_query(
        self,
        function_name: &'static str,
    ) -> napi::Result<EngineNapiQueryTransformExecutionContextV0Json> {
        let value = serialize_boundary_value(&self, function_name)?;
        parse_boundary_json(value, function_name)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct EngineNapiConsumerBuildSummaryV0Json {
    #[napi(js_name = "schemaVersion")]
    pub schema_version: String,
    pub product: String,
    #[napi(js_name = "stylePath")]
    pub style_path: String,
    pub dialect: String,
    #[napi(js_name = "requestedPassIds")]
    pub requested_pass_ids: Vec<String>,
    #[napi(js_name = "targetQuery")]
    pub target_query: serde_json::Value,
    #[napi(js_name = "unknownPassIds")]
    pub unknown_pass_ids: Vec<String>,
    pub execution: serde_json::Value,
    #[napi(js_name = "semanticRemovalCount")]
    pub semantic_removal_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<serde_json::Value>,
    #[napi(js_name = "sourceMapV3")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map_v3: Option<serde_json::Value>,
    #[napi(js_name = "openWorldSnapshot")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_snapshot: Option<serde_json::Value>,
    #[napi(js_name = "readySurfaces")]
    pub ready_surfaces: Vec<String>,
}

impl EngineNapiConsumerBuildSummaryV0Json {
    pub fn from_query(
        value: EngineNapiQueryConsumerBuildSummaryV0Json,
        function_name: &'static str,
    ) -> napi::Result<Self> {
        let value = serialize_boundary_value(&value, function_name)?;
        parse_boundary_value(value, function_name)
    }
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

function renderRustSdkWorkflowContractModule(): string {
  return String.raw`// @generated by scripts/generate-engine-v2-contract-idl.ts from contracts/engine-sdk-workflow/main.tsp.
// Do not edit this file by hand.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaSdkResponsePartitionV0 {
    Public,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaErrorClassV0 {
    Input,
    Workspace,
    Resolution,
    Analysis,
    Transform,
    Unsupported,
    Internal,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaErrorSeverityV0 {
    Error,
    Warning,
    Information,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OmenaErrorRecoverabilityV0 {
    #[serde(rename = "retry")]
    Retry,
    #[serde(rename = "user-action")]
    UserAction,
    #[serde(rename = "not-recoverable")]
    NotRecoverable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaErrorEvidenceReferenceV0 {
    pub query_identity: String,
    pub input_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaErrorContextV0 {
    pub code: String,
    pub severity: OmenaErrorSeverityV0,
    pub recoverability: OmenaErrorRecoverabilityV0,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<OmenaErrorEvidenceReferenceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaErrorV0 {
    pub class: OmenaErrorClassV0,
    pub message: String,
    pub context: OmenaErrorContextV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkErrorEnvelopeV0 {
    pub error: OmenaErrorV0,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCliResponseEnvelopeV0 {
    pub schema_version: String,
    pub product: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_content_digest: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkSnapshotRequestV0 {
    pub workspace_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkSnapshotResponseV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub partition: OmenaSdkResponsePartitionV0,
    pub workspace_root: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkQueryRequestV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub query_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkQueryResponseV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub partition: OmenaSdkResponsePartitionV0,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkDiagnosticsRequestV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub style_path: String,
    pub style_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkDiagnosticsSummaryV0 {
    pub schema_version: String,
    pub product: String,
    pub style_path: String,
    pub dialect: String,
    pub token_count: u64,
    pub parser_error_count: u64,
    pub class_selector_count: u64,
    pub custom_property_count: u64,
    pub keyframe_count: u64,
    pub ready_surfaces: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkDiagnosticsResponseV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub partition: OmenaSdkResponsePartitionV0,
    pub summary: OmenaSdkDiagnosticsSummaryV0,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkDiagnosticsDebugReportV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub partition: OmenaSdkResponsePartitionV0,
    pub public_response: OmenaSdkDiagnosticsResponseV0,
    pub analysis: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkBuildRequestV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub style_path: String,
    pub style_source: String,
    pub pass_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<crate::OmenaQueryTransformExecutionContextV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkBuildResponseV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub partition: OmenaSdkResponsePartitionV0,
    pub summary: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkExplainPositionV0 {
    pub line: i32,
    pub character: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkExplainRequestV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub style_path: String,
    pub position: OmenaSdkExplainPositionV0,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSdkExplainResponseV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub partition: OmenaSdkResponsePartitionV0,
    pub report: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBundlerHostCapabilitiesV0 {
    pub protocol_version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBundlerHostResolveModuleRequestV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub style_path: String,
    pub style_sources: Vec<crate::OmenaQueryStyleSourceInputV0>,
    pub package_manifests: Vec<crate::OmenaQueryStylePackageManifestV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBundlerHostComposesEdgeV0 {
    pub exported_name: String,
    pub module_id: String,
    pub class_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBundlerHostDiagnosticV0 {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBundlerHostResolveModuleResponseV0 {
    pub snapshot_id: crate::OmenaWorkspaceSnapshotIdV0,
    pub protocol_version: String,
    pub module_id: String,
    pub class_map: BTreeMap<String, String>,
    pub named_exports: BTreeMap<String, String>,
    pub typescript_declaration: String,
    pub composes_edges: Vec<OmenaBundlerHostComposesEdgeV0>,
    pub diagnostics: Vec<OmenaBundlerHostDiagnosticV0>,
    pub ready: bool,
}
`
    .trimEnd()
    .concat("\n");
}

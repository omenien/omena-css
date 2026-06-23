import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workDir = fs.mkdtempSync(path.join(os.tmpdir(), "omena-engine-v2-idl-"));
const schemaDir = path.join(workDir, "schema");
const rustProofDir = path.join(workDir, "rust-proof");

function run(command: string, args: readonly string[], cwd = repoRoot): void {
  execFileSync(command, [...args], { cwd, stdio: "inherit" });
}

function output(command: string, args: readonly string[], cwd = repoRoot): string {
  return execFileSync(command, [...args], { cwd, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
}

function readJson(relativePath: string): unknown {
  return JSON.parse(fs.readFileSync(path.join(schemaDir, relativePath), "utf8")) as unknown;
}

function asRecord(value: unknown, label: string): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), `${label} must be object`);
  return value as Record<string, unknown>;
}

function required(schema: Record<string, unknown>): string[] {
  assert.ok(Array.isArray(schema.required), "schema must have required array");
  return schema.required.map(String);
}

function properties(schema: Record<string, unknown>): Record<string, unknown> {
  return asRecord(schema.properties, "schema.properties");
}

function assertRequired(schemaFile: string, fields: readonly string[]): void {
  const schema = asRecord(readJson(schemaFile), schemaFile);
  const actual = required(schema);
  for (const field of fields) {
    assert.ok(actual.includes(field), `${schemaFile} must require ${field}`);
  }
}

function assertProperty(schemaFile: string, field: string): void {
  const schema = asRecord(readJson(schemaFile), schemaFile);
  assert.ok(field in properties(schema), `${schemaFile} must define property ${field}`);
}

fs.mkdirSync(schemaDir, { recursive: true });

run("pnpm", [
  "exec",
  "tsp",
  "compile",
  "contracts/engine-v2",
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

assertRequired("EngineInputV2.json", ["version", "workspace", "sources", "styles", "typeFacts"]);
assertProperty("StringTypeFactsV2.json", "provenance");
assertRequired("EngineOutputV2.json", ["version", "queryResults", "rewritePlans", "checkerReport"]);
assertRequired("OmenaQueryCodeActionPlanV0.json", [
  "schemaVersion",
  "product",
  "fileUri",
  "fileKind",
  "actionCount",
  "actions",
]);

const queryResultSchema = asRecord(readJson("QueryResultV2.json"), "QueryResultV2.json");
assert.ok(Array.isArray(queryResultSchema.oneOf), "QueryResultV2 must emit oneOf");
assert.equal(queryResultSchema.oneOf.length, 3, "QueryResultV2 must emit three oneOf variants");

const engineInputTypes = output("pnpm", [
  "exec",
  "json2ts",
  "-i",
  path.join(schemaDir, "EngineInputV2.json"),
  "--cwd",
  schemaDir,
  "--unknownAny",
  "false",
  "--no-additionalProperties",
]);
const engineOutputTypes = output("pnpm", [
  "exec",
  "json2ts",
  "-i",
  path.join(schemaDir, "EngineOutputV2.json"),
  "--cwd",
  schemaDir,
  "--unknownAny",
  "false",
  "--no-additionalProperties",
]);
const codeActionTypes = output("pnpm", [
  "exec",
  "json2ts",
  "-i",
  path.join(schemaDir, "OmenaQueryCodeActionPlanV0.json"),
  "--cwd",
  schemaDir,
  "--unknownAny",
  "false",
  "--no-additionalProperties",
]);
const generatedRustContract = fs.readFileSync(
  path.join(
    repoRoot,
    "rust/crates/omena-engine-input-producers/src/engine_contract_v2_idl_generated.rs",
  ),
  "utf8",
);

assert.ok(engineInputTypes.includes("export interface EngineInputV2Json"));
assert.ok(engineInputTypes.includes('version: "2"'));
assert.ok(engineInputTypes.includes("workspace: EngineWorkspaceV1Json"));
assert.ok(engineInputTypes.includes('"camelCaseOnly"'));
assert.ok(engineInputTypes.includes("provenance?: string"));
assert.ok(!engineInputTypes.includes("[k: string]"));
assert.ok(generatedRustContract.includes("CamelCaseOnly"));
assert.ok(generatedRustContract.includes('#[serde(rename = "camelCaseOnly")]'));
assert.ok(generatedRustContract.includes("pub kind: String,"));
assert.ok(generatedRustContract.includes("pub constraint_kind: Option<String>,"));
assert.ok(generatedRustContract.includes("pub min_len: Option<usize>,"));
assert.ok(generatedRustContract.includes("pub max_len: Option<usize>,"));
assert.ok(generatedRustContract.includes("pub provenance: Option<String>,"));
assert.ok(generatedRustContract.includes("#[allow(clippy::large_enum_variant)]"));

assert.ok(engineOutputTypes.includes("export interface EngineOutputV2Json"));
assert.ok(engineOutputTypes.includes("queryResults: QueryResultV2Json[]"));
assert.ok(engineOutputTypes.includes("rewritePlans: TextRewritePlanJsonV2Json[]"));
assert.match(engineOutputTypes, /target: (unknown|any);/u);
assert.ok(engineOutputTypes.includes("edits: PlannedTextEditJsonV2Json[]"));
assert.ok(engineOutputTypes.includes("checkerReport: CheckerReportJsonV1Json"));
assert.ok(!engineOutputTypes.includes("[k: string]"));

assert.ok(codeActionTypes.includes("export interface OmenaQueryCodeActionPlanV0Json"));
assert.ok(codeActionTypes.includes('product: "omena-query.code-actions"'));
assert.ok(codeActionTypes.includes("actions: OmenaQueryCodeActionV0Json[]"));
assert.ok(!codeActionTypes.includes("[k: string]"));

fs.mkdirSync(path.join(rustProofDir, "src"), { recursive: true });
fs.writeFileSync(
  path.join(rustProofDir, "Cargo.toml"),
  `[package]
name = "omena-engine-v2-idl-rust-proof"
version = "0.0.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
`,
  "utf8",
);
fs.writeFileSync(path.join(rustProofDir, "src/lib.rs"), renderRustProofModule(), "utf8");

run("cargo", ["test", "--manifest-path", path.join(rustProofDir, "Cargo.toml"), "--quiet"]);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "engine-v2.contract-idl-toolchain",
      typespec: "@typespec/compiler + @typespec/json-schema",
      typescript: "json-schema-to-typescript",
      rust: "repo-owned serde emitter proof",
      schemaDir,
      checkedSchemas: [
        "EngineInputV2.json",
        "StringTypeFactsV2.json",
        "QueryResultV2.json",
        "EngineOutputV2.json",
        "OmenaQueryCodeActionPlanV0.json",
      ],
    },
    null,
    2,
  )}\n`,
);

function renderRustProofModule(): string {
  return String.raw`use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineWorkspaceV1 {
    pub root: String,
    pub classname_transform: String,
    pub settings_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAnalysisInputV2 {
    pub file_path: String,
    pub document: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_graph: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleAnalysisInputV2 {
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub document: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StringTypeFactKindV2 {
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
#[serde(rename_all = "camelCase")]
pub enum StringConstraintKindV2 {
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
pub struct StringTypeFactsV2 {
    pub kind: StringTypeFactKindV2,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_kind: Option<StringConstraintKindV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_len: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_len: Option<i32>,
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
pub struct TypeFactEntryV2 {
    pub file_path: String,
    pub expression_id: String,
    pub facts: StringTypeFactsV2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInputV2 {
    pub version: String,
    pub workspace: EngineWorkspaceV1,
    pub sources: Vec<SourceAnalysisInputV2>,
    pub styles: Vec<StyleAnalysisInputV2>,
    pub type_facts: Vec<TypeFactEntryV2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineOutputV2 {
    pub version: String,
    pub query_results: Vec<QueryResultV2>,
    pub rewrite_plans: Vec<TextRewritePlanJsonV2>,
    pub checker_report: CheckerReportJsonV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum QueryResultV2 {
    #[serde(rename = "expression-semantics")]
    ExpressionSemantics {
        #[serde(rename = "filePath")]
        file_path: String,
        #[serde(rename = "queryId")]
        query_id: String,
        payload: serde_json::Value,
    },
    #[serde(rename = "source-expression-resolution")]
    SourceExpressionResolution {
        #[serde(rename = "filePath")]
        file_path: String,
        #[serde(rename = "queryId")]
        query_id: String,
        payload: serde_json::Value,
    },
    #[serde(rename = "selector-usage")]
    SelectorUsage {
        #[serde(rename = "filePath")]
        file_path: String,
        #[serde(rename = "queryId")]
        query_id: String,
        payload: serde_json::Value,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRewritePlanJsonV2 {
    pub target: serde_json::Value,
    pub edits: Vec<PlannedTextEditJsonV2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedTextEditJsonV2 {
    pub uri: String,
    pub range: TextRangeJsonV2,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRangeJsonV2 {
    pub start: TextPositionJsonV2,
    pub end: TextPositionJsonV2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPositionJsonV2 {
    pub line: i32,
    pub character: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckerReportJsonV1 {
    pub version: String,
    pub findings: Vec<serde_json::Value>,
    pub summary: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryWorkspaceTextEditV0 {
    pub uri: String,
    pub range: serde_json::Value,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeActionPlanKind {
    #[serde(rename = "quickfix")]
    Quickfix,
    #[serde(rename = "refactor.extract")]
    RefactorExtract,
    #[serde(rename = "refactor.inline")]
    RefactorInline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCodeActionV0 {
    pub title: String,
    pub kind: CodeActionPlanKind,
    pub edits: Vec<OmenaQueryWorkspaceTextEditV0>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCodeActionPlanV0 {
    pub schema_version: String,
    pub product: String,
    pub file_uri: String,
    pub file_kind: String,
    pub action_count: i32,
    pub actions: Vec<OmenaQueryCodeActionV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready_surfaces: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn engine_input_round_trips_workspace_and_provenance() {
        let input: EngineInputV2 = serde_json::from_value(json!({
            "version": "2",
            "workspace": {
                "root": "/repo",
                "classnameTransform": "asIs",
                "settingsKey": "asIs:{}"
            },
            "sources": [],
            "styles": [],
            "typeFacts": [{
                "filePath": "/repo/src/App.tsx",
                "expressionId": "class-expr:0",
                "facts": {
                    "kind": "constrained",
                    "constraintKind": "prefix",
                    "prefix": "btn-",
                    "provenance": "finiteSetWideningPrefix"
                }
            }]
        }))
        .unwrap();
        let output = serde_json::to_value(input).unwrap();
        assert_eq!(output["workspace"]["settingsKey"], "asIs:{}");
        assert_eq!(output["typeFacts"][0]["facts"]["provenance"], "finiteSetWideningPrefix");
    }

    #[test]
    fn engine_output_round_trips_tagged_query_union() {
        let output: EngineOutputV2 = serde_json::from_value(json!({
            "version": "2",
            "queryResults": [{
                "kind": "selector-usage",
                "filePath": "/repo/src/App.module.scss",
                "queryId": "button",
                "payload": { "canonicalName": "button" }
            }],
            "rewritePlans": [{
                "target": { "kind": "selector", "name": "button" },
                "edits": [{
                    "uri": "file:///repo/src/App.module.scss",
                    "range": {
                        "start": { "line": 0, "character": 0 },
                        "end": { "line": 0, "character": 7 }
                    },
                    "newText": ".button"
                }]
            }],
            "checkerReport": {
                "version": "1",
                "findings": [],
                "summary": { "total": 0 }
            }
        }))
        .unwrap();
        let output = serde_json::to_value(output).unwrap();
        assert_eq!(output["queryResults"][0]["kind"], "selector-usage");
        assert_eq!(output["rewritePlans"][0]["edits"][0]["newText"], ".button");
    }

    #[test]
    fn code_action_plan_round_trips_rust_query_json_shape() {
        let plan: OmenaQueryCodeActionPlanV0 = serde_json::from_value(json!({
            "schemaVersion": "0",
            "product": "omena-query.code-actions",
            "fileUri": "file:///repo/src/App.module.scss",
            "fileKind": "style",
            "actionCount": 1,
            "actions": [{
                "title": "Extract custom property",
                "kind": "refactor.extract",
                "edits": [{
                    "uri": "file:///repo/src/App.module.scss",
                    "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                    "newText": ":root {}"
                }],
                "source": "omenaQueryStyleExtractCodeActions"
            }],
            "readySurfaces": ["productFacingCodeActions"]
        }))
        .unwrap();
        let output = serde_json::to_value(plan).unwrap();
        assert_eq!(output["product"], "omena-query.code-actions");
        assert_eq!(output["actions"][0]["kind"], "refactor.extract");
    }
}
`;
}

import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function assertIncludes(file: string, content: string, expected: string): void {
  assert.ok(content.includes(expected), `${file} must contain ${expected}`);
}

function assertMatches(file: string, content: string, pattern: RegExp): void {
  assert.ok(pattern.test(content), `${file} must match ${pattern}`);
}

const decisionDocPath = "docs/engine-v2-contract-idl-decisions.md";
const coreContractPath = "server/engine-core-ts/src/contracts/engine-v2.ts";
const generatedInputContractPath =
  "server/engine-core-ts/src/contracts/engine-v2-input-idl.generated.ts";
const hostOutputPath = "server/engine-host-node/src/engine-output-v2.ts";
const engineQueryPath = "server/engine-host-node/src/engine-query-v2.ts";
const codeActionQueryPath = "server/engine-host-node/src/code-action-query.ts";
const rustInputPath = "rust/crates/omena-engine-input-producers/src/lib.rs";
const generatedRustInputPath =
  "rust/crates/omena-engine-input-producers/src/engine_contract_v2_idl_generated.rs";
const shadowRunnerPath = "rust/crates/engine-shadow-runner/src/main.rs";

const decisionDoc = read(decisionDocPath);
const coreContract = read(coreContractPath);
const generatedInputContract = read(generatedInputContractPath);
const hostOutput = read(hostOutputPath);
const engineQuery = read(engineQueryPath);
const codeActionQuery = read(codeActionQueryPath);
const rustInput = read(rustInputPath);
const generatedRustInput = read(generatedRustInputPath);
const shadowRunner = read(shadowRunnerPath);

for (const expected of [
  "workspace",
  "provenance",
  "TypeFactEntryV2",
  "EngineOutputV2",
  "QueryResultV2",
  "code-action query JSON",
  "Rust shadow-runner",
]) {
  assertIncludes(decisionDocPath, decisionDoc, expected);
}

assertIncludes(coreContractPath, coreContract, "export type EngineInputV2");
assertIncludes(coreContractPath, coreContract, "readonly workspace: EngineWorkspaceV1");
assertIncludes(generatedInputContractPath, generatedInputContract, "readonly provenance?: string");
assertIncludes(coreContractPath, coreContract, "export type QueryResultV2");
assertIncludes(coreContractPath, coreContract, "export type EngineOutputV2");
assertIncludes(
  coreContractPath,
  coreContract,
  "readonly rewritePlans: readonly TextRewritePlan<unknown>[]",
);

assertIncludes(rustInputPath, rustInput, "pub struct EngineInputV2");
assertIncludes(
  rustInputPath,
  rustInput,
  "pub type TypeFactEntryV2 = engine_contract_v2_idl_generated::TypeFactEntryV2Json;",
);
assertIncludes(
  rustInputPath,
  rustInput,
  "pub type StringTypeFactsV2 = engine_contract_v2_idl_generated::StringTypeFactsV2Json;",
);
assertIncludes(rustInputPath, rustInput, '#[serde(rename_all = "camelCase")]');
assertIncludes(generatedRustInputPath, generatedRustInput, "pub struct TypeFactEntryV2Json");
assertIncludes(generatedRustInputPath, generatedRustInput, "pub struct StringTypeFactsV2Json");
assertIncludes(generatedRustInputPath, generatedRustInput, "pub provenance: Option<String>");

assertIncludes(hostOutputPath, hostOutput, "export interface BuildEngineOutputV2Options");
assertIncludes(hostOutputPath, hostOutput, "export function buildEngineOutputV2");
assertIncludes(
  hostOutputPath,
  hostOutput,
  "readonly rewritePlans?: readonly TextRewritePlan<unknown>[]",
);

assertIncludes(engineQueryPath, engineQuery, "export interface BuildSelectedQueryResultsV2Options");
assertIncludes(engineQueryPath, engineQuery, "): readonly QueryResultV2[]");
assertIncludes(engineQueryPath, engineQuery, "ExpressionSemanticsQueryResultV2");
assertIncludes(engineQueryPath, engineQuery, "SourceExpressionResolutionQueryResultV2");
assertIncludes(engineQueryPath, engineQuery, "SelectorUsageQueryResultV2");

assertIncludes(codeActionQueryPath, codeActionQuery, "interface OmenaQueryCodeActionPlanJson");
assertIncludes(codeActionQueryPath, codeActionQuery, "export type CodeActionPlan");

assertMatches(shadowRunnerPath, shadowRunner, /struct EngineOutputV2\s*\{/);
assertMatches(shadowRunnerPath, shadowRunner, /enum QueryResultV2\s*\{/);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "engine-v2.contract-idl-decisions",
      decisionDoc: decisionDocPath,
      checkedSurfaces: [
        coreContractPath,
        hostOutputPath,
        engineQueryPath,
        codeActionQueryPath,
        rustInputPath,
        generatedRustInputPath,
        shadowRunnerPath,
      ],
    },
    null,
    2,
  )}\n`,
);

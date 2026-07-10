import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function blockBody(source: string, marker: string, opening = "{"): string {
  const start = source.indexOf(marker);
  assert.ok(start >= 0, `missing ${marker}`);
  const open = source.indexOf(opening, start);
  assert.ok(open >= 0, `missing body for ${marker}`);
  const closing = opening === "{" ? "}" : ")";
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === opening) depth += 1;
    if (source[index] === closing) depth -= 1;
    if (depth === 0) return source.slice(open + 1, index);
  }
  throw new Error(`unterminated body for ${marker}`);
}

function topLevelEnumVariants(source: string, enumName: string): string[] {
  const body = blockBody(source, `pub enum ${enumName}`);
  const variants: string[] = [];
  let depth = 0;
  for (const line of body.split("\n")) {
    const trimmed = line.trim();
    if (depth === 0) {
      const match = trimmed.match(/^([A-Z][A-Za-z0-9]*)\b/u);
      if (match?.[1]) variants.push(match[1]);
    }
    depth += [...line].filter((char) => char === "{").length;
    depth -= [...line].filter((char) => char === "}").length;
  }
  return [...new Set(variants)];
}

function count(source: string, marker: string): number {
  return source.split(marker).length - 1;
}

const model = read("rust/crates/omena-transform-passes/src/model.rs");
const executor = read("rust/crates/omena-transform-passes/src/runtime/executor.rs");
const testModuleStart = executor.indexOf("#[cfg(test)]\nmod dispatch_table_tests");
assert.ok(testModuleStart > 0, "executor test module boundary must exist");
const productionExecutor = executor.slice(0, testModuleStart);

assert.deepEqual(topLevelEnumVariants(model, "TransformDecision"), [
  "Applied",
  "NoChange",
  "Blocked",
  "Rejected",
]);
assert.ok(model.includes("pub decisions: Vec<TransformDecision>"));
assert.ok(model.includes("pub fn compatibility_outcome(&self)"));

const untypedPlannedOnlyCallCount = count(
  productionExecutor,
  "TransformPassDispatchResultV0::planned_only(",
);
assert.equal(untypedPlannedOnlyCallCount, 0, "untyped planned-only dispatches must be retired");

const blockedCallCount = count(productionExecutor, "TransformPassDispatchResultV0::blocked(");
const profileOnlyCallCount = count(
  productionExecutor,
  "TransformPassDispatchResultV0::profile_only(",
);
const irRejectedCallCount = [...productionExecutor.matchAll(/input\s*\.\s*ir_rejected\(/gu)].length;
const precisionBlockedCallCount = [
  ...productionExecutor.matchAll(/input\s*\.\s*precision_blocker\(\)/gu),
].length;
const baselineBlockedCallCount = blockedCallCount - precisionBlockedCallCount;
const semanticRejectedCallCount = [
  ...productionExecutor.matchAll(/TransformRejectionReasonV0::SemanticPreservation/gu),
].length;
const classifiedCallCount =
  baselineBlockedCallCount + profileOnlyCallCount + irRejectedCallCount + semanticRejectedCallCount;

assert.ok(blockedCallCount > 0, "blocked decisions must be non-vacuous");
assert.ok(profileOnlyCallCount > 0, "profile-only decisions must remain distinguishable");
assert.ok(irRejectedCallCount > 0, "IR transaction rejections must be non-vacuous");
assert.ok(semanticRejectedCallCount > 0, "semantic rejections must be non-vacuous");
assert.equal(baselineBlockedCallCount, 13);
assert.equal(classifiedCallCount, 37, "every baseline planned-only branch must be classified");

const structuralStart = productionExecutor.indexOf("fn run_import_inline_structural");
const structuralEnd = productionExecutor.indexOf(
  "fn execute_transform_passes_on_source_with_active_lex_cache",
  structuralStart,
);
assert.ok(structuralStart >= 0 && structuralEnd > structuralStart);
const structuralHandlers = productionExecutor.slice(structuralStart, structuralEnd);
const irMutationCount = count(structuralHandlers, "input.ir_mutation_result(");
assert.equal(irRejectedCallCount, irMutationCount);

assert.ok(
  productionExecutor.includes("TransformRejectionReasonV0::IrTransaction { pass: self.kind }"),
  "structural transaction failures must carry the typed pass kind",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-transform.decision-census",
      baselineUntypedPlannedOnlyCallCount: 37,
      untypedPlannedOnlyCallCount,
      blockedCallCount,
      baselineBlockedCallCount,
      precisionBlockedCallCount,
      profileOnlyCallCount,
      irRejectedCallCount,
      semanticRejectedCallCount,
      classifiedCallCount,
      complete: true,
    },
    null,
    2,
  )}\n`,
);

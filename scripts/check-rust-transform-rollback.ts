import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function blockBody(source: string, marker: string): string {
  const start = source.indexOf(marker);
  assert.ok(start >= 0, `missing ${marker}`);
  const open = source.indexOf("{", start);
  assert.ok(open >= 0, `missing body for ${marker}`);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
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

const model = read("rust/crates/omena-transform-passes/src/model.rs");
const executor = read("rust/crates/omena-transform-passes/src/runtime/executor.rs");
const manifest = read("rust/crates/omena-transform-passes/Cargo.toml");

assert.deepEqual(topLevelEnumVariants(model, "RollbackScopeV0"), [
  "RejectPreservedInput",
  "InversePatch",
  "CommittedIrrecoverable",
]);
const receiptBody = blockBody(model, "pub struct RollbackReceiptV0");
for (const field of [
  "pass_id",
  "attempted_mutation_count",
  "input_content_signature",
  "output_preserved_content_signature",
  "restorable",
]) {
  assert.ok(receiptBody.includes(`pub ${field}:`), `rollback receipt is missing ${field}`);
}
assert.ok(!receiptBody.includes("epoch"), "rollback receipt must not introduce an epoch");
assert.ok(
  model.includes("pub fn covers_inverse_patch("),
  "the shared rollback authority must validate inverse-patch coverage",
);

const decisionBody = blockBody(model, "pub enum TransformDecision");
assert.match(
  decisionBody,
  /Applied\s*\{[^}]*rollback_receipt:\s*RollbackReceiptV0/su,
  "applied decisions must carry a rollback receipt",
);
assert.match(
  decisionBody,
  /Rejected\s*\{[^}]*rollback_receipt:\s*RollbackReceiptV0/su,
  "rejected decisions must carry a rollback receipt",
);

const finalizeBody = blockBody(executor, "fn finalize(");
assert.ok(finalizeBody.includes("RollbackScopeV0::CommittedIrrecoverable"));
assert.ok(finalizeBody.includes("RollbackScopeV0::RejectPreservedInput"));
assert.ok(finalizeBody.includes("assert_eq!(input_content_signature, preserved_output_signature)"));
assert.ok(executor.includes("blake3::hash(source.as_bytes())"));
assert.ok(manifest.includes("blake3.workspace = true"));

const testName = "rollback_receipts_distinguish_committed_rewrites_from_rejected_transactions";
const testBody = blockBody(executor, `fn ${testName}`);
assert.ok(testBody.includes("summarize_structural_ir_shadow_equivalence_v0"));
assert.ok(testBody.includes("perturbed_receipt"));

const test = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-transform-passes",
    testName,
    "--",
    "--nocapture",
  ],
  { cwd: repoRoot, encoding: "utf8" },
);
const testOutput = `${test.stdout ?? ""}\n${test.stderr ?? ""}`;
assert.equal(test.status, 0, testOutput);
assert.match(testOutput, /running 1 test/u);
assert.match(testOutput, /1 passed; 0 failed/u);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-transform.rollback-receipt",
      rollbackScopes: topLevelEnumVariants(model, "RollbackScopeV0"),
      appliedReceiptRequired: true,
      rejectedReceiptRequired: true,
      transactionFixtureCount: 1,
      structuralShadowCorpusPresent: true,
      perturbedReceiptRejected: true,
      complete: true,
    },
    null,
    2,
  )}\n`,
);

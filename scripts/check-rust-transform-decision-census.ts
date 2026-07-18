import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
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

function closingBraceIndex(source: string, open: number): number {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return index;
  }
  throw new Error("unterminated conditional block");
}

type DecisionSiteClass = "blocked" | "profileOnly" | "irRejected" | "semanticRejected";

const expectedDecisionSiteClasses: readonly DecisionSiteClass[] = [
  "blocked",
  "profileOnly",
  "blocked",
  "profileOnly",
  "blocked",
  "irRejected",
  "blocked",
  "irRejected",
  "blocked",
  "irRejected",
  "irRejected",
  "blocked",
  "irRejected",
  "irRejected",
  "irRejected",
  "irRejected",
  "irRejected",
  "irRejected",
  "blocked",
  "irRejected",
  "irRejected",
  "irRejected",
  "irRejected",
  "irRejected",
  "irRejected",
  "blocked",
  "irRejected",
  "blocked",
  "irRejected",
  "blocked",
  "irRejected",
  "blocked",
  "irRejected",
  "irRejected",
  "blocked",
  "blocked",
  "blocked",
  "semanticRejected",
];

const model = read("rust/crates/omena-transform-passes/src/model.rs");
const executor = read("rust/crates/omena-transform-passes/src/runtime/executor.rs");
const semanticPreservation = read(
  "rust/crates/omena-transform-passes/src/runtime/semantic_preservation.rs",
);
const cascadeProof = read("rust/crates/omena-transform-passes/src/runtime/cascade_proof.rs");
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
const blockedReasonBody = blockBody(model, "pub enum TransformBlockedReasonV0");
assert.match(
  blockedReasonBody,
  /DischargeMissing\s*\{[\s\S]*lookup_status:\s*Option<DischargeLedgerLookupStatusV0>[\s\S]*verdict:\s*Option<DischargeLedgerVerdictV0>/u,
  "discharge failures must preserve the typed ledger outcome",
);
const dischargeEvidenceBody = blockBody(model, "pub struct TransformDischargeEvidenceV0");
for (const field of [
  "evidence_node_key",
  "guarantee_family",
  "ledger_cell_key",
  "boundedness_kind",
]) {
  assert.ok(
    dischargeEvidenceBody.includes(`pub ${field}:`),
    `transform discharge evidence is missing ${field}`,
  );
}
const decisionBody = blockBody(model, "pub enum TransformDecision");
assert.match(
  decisionBody,
  /Applied\s*\{[\s\S]*discharge_evidence:\s*Vec<TransformDischargeEvidenceV0>/u,
  "applied decisions must carry ledger-backed evidence references when present",
);
assert.match(
  decisionBody,
  /Applied\s*\{[\s\S]*semantic_guarantee_tier:\s*Option<TransformSemanticGuaranteeTierV0>/u,
  "applied decisions must carry an optional typed semantic guarantee tier",
);
assert.ok(
  executor.includes("pass.filter(|pass| semantic_preservation_applies(*pass))") &&
    executor.includes("TransformSemanticGuaranteeTierV0::L0Observed"),
  "the baseline trust tier must be derived from the existing semantic observer",
);
assert.ok(
  model.includes("pub struct TransformWinnerEqualityObligationV0") &&
    model.includes("pub struct TransformWinnerEqualityAffectedPairV0") &&
    model.includes("pub struct TransformWinnerEqualityWitnessV0") &&
    model.includes("CascadeOutcome::Definite { winner, proof, .. }"),
  "winner equality obligations must carry authority-produced cascade witnesses",
);

const frozenAdmissionPasses = [
  "EmptyRuleRemoval",
  "LayerFlatten",
  "NestingUnwrap",
  "RuleDeduplication",
  "RuleMerging",
  "ScopeFlatten",
  "SelectorMerging",
  "TreeShakeClass",
  "TreeShakeCustomProperty",
  "TreeShakeKeyframes",
  "TreeShakeValue",
].toSorted();
const admissionBody = blockBody(semanticPreservation, "fn semantic_preservation_applies");
const observedAdmissionPasses = [
  ...new Set(
    [...admissionBody.matchAll(/TransformPassKind::([A-Z][A-Za-z0-9]+)/gu)].map(
      (match) => match[1],
    ),
  ),
].toSorted();
assert.deepEqual(
  observedAdmissionPasses,
  frozenAdmissionPasses,
  "semantic trust must not change the existing admission floor",
);
assert.ok(!decisionBody.includes("epoch"), "transform decisions must not introduce an epoch");
assert.ok(
  !dischargeEvidenceBody.includes("epoch"),
  "discharge evidence must use the existing evidence identity",
);
assert.ok(cascadeProof.includes("node.earned_via()"));
assert.ok(cascadeProof.includes("GuaranteeFamilyV0::LedgerBackedObligationDischarge"));
assert.ok(!cascadeProof.includes("FamilyStampV0"), "the transform consumer must not mint stamps");
assert.ok(
  !cascadeProof.includes("LedgerDischargeWitnessV0"),
  "the transform consumer must not mint ledger witnesses",
);

const untypedPlannedOnlyCallCount = count(
  productionExecutor,
  "TransformPassDispatchResultV0::planned_only(",
);
assert.equal(untypedPlannedOnlyCallCount, 0, "untyped planned-only dispatches must be retired");

const blockedMarker = "TransformPassDispatchResultV0::blocked(";
const precisionBlockedIndexes = new Set<number>();
for (const match of productionExecutor.matchAll(/input\s*\.\s*precision_blocker\(\)/gu)) {
  const open = productionExecutor.indexOf("{", match.index);
  assert.ok(open >= 0, "precision blocker must guard a block");
  const close = closingBraceIndex(productionExecutor, open);
  const blockedIndex = productionExecutor.indexOf(blockedMarker, open);
  assert.ok(
    blockedIndex > open && blockedIndex < close,
    "precision blocker must emit a typed blocked decision in the same branch",
  );
  precisionBlockedIndexes.add(blockedIndex);
}

const decisionSites: Array<{ index: number; class: DecisionSiteClass }> = [];
for (const match of productionExecutor.matchAll(/TransformPassDispatchResultV0::blocked\(/gu)) {
  if (!precisionBlockedIndexes.has(match.index)) {
    decisionSites.push({ index: match.index, class: "blocked" });
  }
}
for (const match of productionExecutor.matchAll(
  /TransformPassDispatchResultV0::profile_only\(/gu,
)) {
  decisionSites.push({ index: match.index, class: "profileOnly" });
}
for (const match of productionExecutor.matchAll(/input\s*\.\s*ir_rejected\(/gu)) {
  decisionSites.push({ index: match.index, class: "irRejected" });
}
for (const match of productionExecutor.matchAll(
  /TransformRejectionReasonV0::SemanticPreservation/gu,
)) {
  decisionSites.push({ index: match.index, class: "semanticRejected" });
}
decisionSites.sort((left, right) => left.index - right.index);

const discoveredDecisionSiteClasses = decisionSites.map((site) => site.class);
assert.deepEqual(
  discoveredDecisionSiteClasses,
  expectedDecisionSiteClasses,
  "typed decision sites must retain their checked-in per-ordinal classification",
);
const decisionSiteManifest = decisionSites.map((site, index) => ({
  file: "rust/crates/omena-transform-passes/src/runtime/executor.rs",
  ordinal: index + 1,
  class: site.class,
}));

const blockedCallCount = count(productionExecutor, blockedMarker);
const profileOnlyCallCount = decisionSites.filter((site) => site.class === "profileOnly").length;
const irRejectedCallCount = decisionSites.filter((site) => site.class === "irRejected").length;
const precisionBlockedCallCount = precisionBlockedIndexes.size;
const baselineBlockedCallCount = blockedCallCount - precisionBlockedCallCount;
const semanticRejectedCallCount = decisionSites.filter(
  (site) => site.class === "semanticRejected",
).length;
const classifiedCallCount = decisionSiteManifest.length;

assert.ok(blockedCallCount > 0, "blocked decisions must be non-vacuous");
assert.ok(profileOnlyCallCount > 0, "profile-only decisions must remain distinguishable");
assert.ok(irRejectedCallCount > 0, "IR transaction rejections must be non-vacuous");
assert.ok(semanticRejectedCallCount > 0, "semantic rejections must be non-vacuous");
assert.equal(baselineBlockedCallCount, 14);
assert.equal(classifiedCallCount, 38, "every baseline planned-only branch must be classified");

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

const dischargeTest = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-transform-passes",
    "discharge_decisions_block_stale_and_record_ledger_evidence",
    "--",
    "--nocapture",
  ],
  { cwd: repoRoot, encoding: "utf8" },
);
const dischargeTestOutput = `${dischargeTest.stdout ?? ""}\n${dischargeTest.stderr ?? ""}`;
assert.equal(dischargeTest.status, 0, dischargeTestOutput);
assert.match(dischargeTestOutput, /running 1 test/u);
assert.match(dischargeTestOutput, /1 passed; 0 failed/u);

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
      decisionSiteManifest,
      dischargeEvidenceRecorded: true,
      staleDischargeBlocked: true,
      transformMintsEvidenceStamp: false,
      complete: true,
    },
    null,
    2,
  )}\n`,
);

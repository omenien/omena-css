import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface Baseline {
  readonly schemaVersion: "0";
  readonly product: "rust.rewrite-obligation-family-closure-baseline";
  readonly untypedCarrierCeiling: number;
  readonly untypedProseArmCeiling: number;
  readonly hardClosure: boolean;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readBaseline(): Baseline {
  const baseline = JSON.parse(read("scripts/rewrite-obligation-family-closure-baseline.json"));
  assert.equal(baseline.schemaVersion, "0");
  assert.equal(baseline.product, "rust.rewrite-obligation-family-closure-baseline");
  assert.ok(Number.isInteger(baseline.untypedCarrierCeiling));
  assert.ok(Number.isInteger(baseline.untypedProseArmCeiling));
  assert.equal(typeof baseline.hardClosure, "boolean");
  return baseline;
}

function countEnumVariants(source: string, enumName: string): number {
  const match = source.match(new RegExp(`pub enum ${enumName} \\{([\\s\\S]*?)\\n\\}`));
  assert.ok(match, `missing enum ${enumName}`);
  return match[1]
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => /^[A-Z][A-Za-z0-9]*,?$/.test(line)).length;
}

function enumVariants(source: string, enumName: string): string[] {
  const match = source.match(new RegExp(`pub enum ${enumName} \\{([\\s\\S]*?)\\n\\}`));
  assert.ok(match, `missing enum ${enumName}`);
  return match[1]
    .split("\n")
    .map((line) => line.trim().replace(/,$/, ""))
    .filter((line) => /^[A-Z][A-Za-z0-9]*$/.test(line))
    .toSorted();
}

function extractStructBody(source: string, structName: string): string {
  const match = source.match(new RegExp(`pub struct ${structName} \\{([\\s\\S]*?)\\n\\}`));
  assert.ok(match, `missing struct ${structName}`);
  return match[1];
}

function extractFunctionBody(source: string, fnName: string): string {
  const start = source.indexOf(`pub const fn ${fnName}`);
  assert.ok(start >= 0, `missing function ${fnName}`);
  const open = source.indexOf("{", start);
  assert.ok(open >= 0, `missing function body for ${fnName}`);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") depth += 1;
    if (char === "}") depth -= 1;
    if (depth === 0) return source.slice(open + 1, index);
  }
  throw new Error(`unterminated function body for ${fnName}`);
}

function countUntypedCarrierFamilies(): number {
  const cascadeProof = read("rust/crates/omena-cascade-proof/src/lib.rs");
  const transformEgg = read("rust/crates/omena-transform-egg/src/lib.rs");
  const lawvere = read("rust/crates/omena-lawvere/src/lib.rs");

  const transformRewriteInputNew = cascadeProof.match(
    /impl TransformRewriteProofInputV0 \{[\s\S]*?pub fn new\(([\s\S]*?)\n    \) -> Self/,
  );
  assert.ok(transformRewriteInputNew, "missing TransformRewriteProofInputV0::new");
  const carrierOneTyped = transformRewriteInputNew[1].includes(
    "obligation_family: ObligationFamilyIdV0",
  );

  const eggProofBody = extractStructBody(transformEgg, "EggRewriteProofV0");
  const lawvereCertificateBody = extractStructBody(lawvere, "ReorderabilityCertificateV0");
  const carrierTwoTyped =
    eggProofBody.includes("obligation_family") &&
    lawvereCertificateBody.includes("obligation_family");

  return Number(!carrierOneTyped) + Number(!carrierTwoTyped);
}

function countUntypedProseArms(): number {
  const transformCst = read("rust/crates/omena-transform-cst/src/lib.rs");
  const functionBody = extractFunctionBody(transformCst, "cascade_safe_obligation");
  if (!functionBody.includes("match kind")) return 0;
  return [...functionBody.matchAll(/TransformPassKind::[A-Za-z0-9]+/g)].length;
}

const baseline = readBaseline();
const evidenceGraph = read("rust/crates/omena-evidence-graph/src/lib.rs");
const transformCst = read("rust/crates/omena-transform-cst/src/lib.rs");

for (const symbol of [
  "pub enum ObligationFamilyIdV0",
  "pub struct RewriteObligationFamilyDescriptorV0",
  "list_rewrite_obligation_families_v0",
  "summarize_rewrite_obligation_family_closure_v0",
]) {
  assert.ok(evidenceGraph.includes(symbol), `evidence graph registry must include ${symbol}`);
}

for (const forbidden of [
  "DifferentiallyValidated",
  "Certified",
  "Theorem",
  "theorem",
  "validated",
  "certified",
]) {
  assert.ok(
    !evidenceGraph.includes(forbidden),
    `obligation-family registry must not introduce ${forbidden}`,
  );
}

const guaranteeKindCount = countEnumVariants(evidenceGraph, "GuaranteeKindV0");
assert.equal(guaranteeKindCount, 7, "GuaranteeKindV0 variant count must stay fixed");

const obligationFamilyCount = countEnumVariants(evidenceGraph, "ObligationFamilyIdV0");
const registeredFamilies = enumVariants(evidenceGraph, "ObligationFamilyIdV0");
const countConstant = evidenceGraph.match(
  /REWRITE_OBLIGATION_FAMILY_COUNT_V0:\s*usize\s*=\s*(\d+);/,
);
assert.ok(countConstant, "missing obligation family count constant");
assert.equal(
  obligationFamilyCount,
  Number(countConstant[1]),
  "obligation family enum and count constant must stay aligned",
);
assert.equal(
  countConstant[1],
  String((evidenceGraph.match(/\.descriptor\(\)/g) ?? []).length),
  "obligation family descriptor table must cover every family exactly once",
);

const passKindCount = countEnumVariants(transformCst, "TransformPassKind");
assert.equal(passKindCount, 44, "transform pass catalog must stay at the expected width");

const untypedCarrierCount = countUntypedCarrierFamilies();
const untypedProseArmCount = countUntypedProseArms();
const carrierBoundFamilies = new Set(
  [
    "rust/crates/omena-cascade-proof/src/lib.rs",
    "rust/crates/omena-transform-cst/src/lib.rs",
    "rust/crates/omena-transform-egg/src/lib.rs",
    "rust/crates/omena-transform-egg/src/lawvere_analysis.rs",
    "rust/crates/omena-lawvere/src/lib.rs",
  ].flatMap((relativePath) =>
    [...read(relativePath).matchAll(/ObligationFamilyIdV0::([A-Z][A-Za-z0-9]+)/g)].map(
      (match) => match[1],
    ),
  ),
);
const orphanFamilies = registeredFamilies.filter((family) => !carrierBoundFamilies.has(family));
const extraCarrierFamilies = [...carrierBoundFamilies]
  .filter((family) => !registeredFamilies.includes(family))
  .toSorted();
assert.equal(
  [
    ...read("rust/crates/omena-transform-egg/src/lib.rs").matchAll(
      /(?:proof:\s*|=\s*)EggRewriteProofV0\s*\{/g,
    ),
  ].length,
  0,
  "EggRewriteProofV0 must be built through its typed constructor",
);
assert.ok(
  untypedCarrierCount <= baseline.untypedCarrierCeiling,
  `untyped carrier count ${untypedCarrierCount} exceeds ceiling ${baseline.untypedCarrierCeiling}`,
);
assert.ok(
  untypedProseArmCount <= baseline.untypedProseArmCeiling,
  `untyped prose arm count ${untypedProseArmCount} exceeds ceiling ${baseline.untypedProseArmCeiling}`,
);

if (baseline.hardClosure) {
  assert.equal(untypedCarrierCount, 0, "hard closure requires no untyped carrier families");
  assert.equal(untypedProseArmCount, 0, "hard closure requires no untyped prose arms");
  assert.deepEqual(orphanFamilies, [], "hard closure requires every family to have a carrier");
  assert.deepEqual(extraCarrierFamilies, [], "hard closure found unregistered carrier families");
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.rewrite-obligation-family-closure",
      obligationFamilyCount,
      guaranteeKindCount,
      transformPassKindCount: passKindCount,
      untypedCarrierCount,
      untypedProseArmCount,
      untypedCarrierCeiling: baseline.untypedCarrierCeiling,
      untypedProseArmCeiling: baseline.untypedProseArmCeiling,
      carrierBoundFamilyCount: carrierBoundFamilies.size,
      orphanFamilies,
      extraCarrierFamilies,
      hardClosure: baseline.hardClosure,
      closurePassed:
        untypedCarrierCount === 0 &&
        untypedProseArmCount === 0 &&
        orphanFamilies.length === 0 &&
        extraCarrierFamilies.length === 0 &&
        baseline.hardClosure,
    },
    null,
    2,
  )}\n`,
);

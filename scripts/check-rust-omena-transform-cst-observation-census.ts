import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function enumVariants(source: string, enumName: string): string[] {
  const match = source.match(new RegExp(`pub enum ${enumName} \\{([\\s\\S]*?)\\n\\}`));
  assert.ok(match, `missing enum ${enumName}`);
  return match[1]
    .split("\n")
    .map((line) => line.trim().replace(/,$/u, ""))
    .filter((line) => /^[A-Z][A-Za-z0-9]*$/u.test(line));
}

function extractFunctionBody(source: string, functionName: string): string {
  const start = source.indexOf(`pub fn ${functionName}`);
  assert.ok(start >= 0, `missing function ${functionName}`);
  const open = source.indexOf("{", start);
  assert.ok(open >= 0, `missing function body for ${functionName}`);

  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    const char = source[index];
    if (char === "{") depth += 1;
    if (char === "}") depth -= 1;
    if (depth === 0) return source.slice(open + 1, index);
  }
  throw new Error(`unterminated function body for ${functionName}`);
}

const transformCstSource = read("rust/crates/omena-transform-cst/src/lib.rs");
const passDescriptorSource = read("rust/crates/omena-transform-cst/src/pass_descriptor.rs");

const variants = enumVariants(transformCstSource, "TransformPassKind");
const functionBody = extractFunctionBody(passDescriptorSource, "pass_observation_contract");
const referencedVariants = [
  ...new Set(
    [...functionBody.matchAll(/TransformPassKind::([A-Z][A-Za-z0-9]+)/gu)].map((match) => match[1]),
  ),
].toSorted();
const declaredCount = [...functionBody.matchAll(/declared_observation_contract\s*\(/g)].length;
const unknownGapReasons = [...functionBody.matchAll(/UnknownGap\s*\{\s*reason:\s*"([^"]*)"/gu)].map(
  (match) => match[1],
);

assert.equal(variants.length, 44, "TransformPassKind catalog width must remain scan-derived");
assert.deepEqual(
  referencedVariants,
  variants.toSorted(),
  "pass_observation_contract must reference every TransformPassKind exactly once",
);
assert.ok(
  !/(^|[^\w])_\s*=>/u.test(functionBody),
  "pass_observation_contract must not use a catch-all arm",
);
assert.ok(declaredCount > 0, "observation census must include at least one declared surface");
assert.ok(
  unknownGapReasons.every((reason) => reason.trim().length > 0),
  "unknown observation gaps must carry a non-empty reason",
);
assert.ok(
  passDescriptorSource.includes("pub enum ObservationKindV0"),
  "ObservationKindV0 must be declared beside pass descriptors",
);
assert.ok(
  passDescriptorSource.includes("pub struct PassSemanticContractV0"),
  "PassSemanticContractV0 must be declared beside pass descriptors",
);
assert.ok(
  passDescriptorSource.includes("pub enum PassObservationSurfaceV0"),
  "PassObservationSurfaceV0 must be declared beside pass descriptors",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-transform-cst.observation-census",
      transformPassKindCount: variants.length,
      observationContractRowCount: referencedVariants.length,
      declaredSurfaceArmCount: declaredCount,
      unknownGapCount: unknownGapReasons.length,
      complete: referencedVariants.length === variants.length,
    },
    null,
    2,
  )}\n`,
);

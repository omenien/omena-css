import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

interface WitnessBasisSite {
  readonly sourcePath: string;
  readonly functionName: string;
}

interface WitnessBasisDisposition extends WitnessBasisSite {
  readonly disposition: "guardFixture";
  readonly authority: string;
}

interface PrecisionWitnessCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.precision-witness-census";
  readonly dormantBasis: "supersetProof";
  readonly classifiedReferences: readonly WitnessBasisDisposition[];
  readonly unclassifiedReferences: readonly WitnessBasisSite[];
  readonly productionReferenceCount: number;
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-precision-witness-census.json");
const writeMode = process.argv.includes("--write");
const supersetProofPattern = /OmenaAbstractValuePrecisionBasisV0::SupersetProof/gu;
const allowedReferences: readonly WitnessBasisDisposition[] = [
  {
    sourcePath: "rust/crates/omena-abstract-value/src/tests.rs",
    functionName: "superset_proof_basis_remains_dormant_without_producer_binding",
    disposition: "guardFixture",
    authority: "the fixture proves that an unbound future proof cannot promote precision",
  },
];

const references = rustSourcePaths(path.join(repoRoot, "rust/crates"))
  .flatMap((sourcePath) =>
    collectSites(
      path.relative(repoRoot, sourcePath),
      fs.readFileSync(sourcePath, "utf8"),
      supersetProofPattern,
    ),
  )
  .toSorted(compareSites);
const allowedKeys = new Set(allowedReferences.map(siteKey));
const unclassifiedReferences = references.filter((site) => !allowedKeys.has(siteKey(site)));
const productionReferenceCount = references.filter(
  (site) => !site.sourcePath.endsWith("/tests.rs") && !site.sourcePath.includes("/tests/"),
).length;

const injected = collectSites(
  "rust/crates/injected/src/producer.rs",
  "fn derive_precision() { let basis = OmenaAbstractValuePrecisionBasisV0::SupersetProof; }",
  supersetProofPattern,
);
assert.deepEqual(
  injected,
  [{ sourcePath: "rust/crates/injected/src/producer.rs", functionName: "derive_precision" }],
  "the predicate must detect a new SupersetProof producer",
);
assert.equal(
  injected.some((site) => allowedKeys.has(siteKey(site))),
  false,
  "an unknown SupersetProof producer must not match the guard fixture",
);
assert.deepEqual(
  references,
  allowedReferences.map(({ sourcePath, functionName }) => ({ sourcePath, functionName })),
  "SupersetProof must remain dormant outside its rejection fixture",
);
assert.equal(productionReferenceCount, 0, "SupersetProof has no producer-bound implementation yet");

const census: PrecisionWitnessCensus = {
  schemaVersion: "0",
  product: "omena.precision-witness-census",
  dormantBasis: "supersetProof",
  classifiedReferences: allowedReferences,
  unclassifiedReferences,
  productionReferenceCount,
};
const serialized = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
} else {
  assert.deepEqual(
    JSON.parse(fs.readFileSync(censusPath, "utf8")),
    census,
    "precision witness census is stale",
  );
}

process.stdout.write(
  `Omena precision witness census OK: dormant=${census.dormantBasis} production=${productionReferenceCount} unclassified=0\n`,
);

function collectSites(sourcePath: string, source: string, pattern: RegExp): WitnessBasisSite[] {
  return [...source.matchAll(pattern)].map((match) => ({
    sourcePath,
    functionName: enclosingFunctionName(source, match.index),
  }));
}

function enclosingFunctionName(source: string, offset: number): string {
  const prefix = source.slice(0, offset);
  const matches = [...prefix.matchAll(/\bfn\s+([a-zA-Z0-9_]+)\s*\(/gu)];
  return matches.at(-1)?.[1] ?? "moduleScope";
}

function siteKey(site: WitnessBasisSite): string {
  return `${site.sourcePath}#${site.functionName}`;
}

function compareSites(left: WitnessBasisSite, right: WitnessBasisSite): number {
  return (
    left.sourcePath.localeCompare(right.sourcePath) ||
    left.functionName.localeCompare(right.functionName)
  );
}

function rustSourcePaths(root: string): string[] {
  const paths: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      paths.push(...rustSourcePaths(entryPath));
    } else if (entry.name.endsWith(".rs")) {
      paths.push(entryPath);
    }
  }
  return paths.toSorted();
}

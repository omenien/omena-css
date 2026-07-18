import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

interface ReasonlessTopSite {
  readonly sourcePath: string;
  readonly functionName: string;
}

interface ReasonlessTopDisposition extends ReasonlessTopSite {
  readonly disposition: "legacyCacheKey" | "legacyRoundTripFixture" | "legacyShapeFixture";
  readonly authority: string;
}

interface TopProvenanceCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.top-provenance-census";
  readonly reasonlessTopDispositions: readonly ReasonlessTopDisposition[];
  readonly unclassifiedReasonlessTopSites: readonly ReasonlessTopSite[];
  readonly legacyUnitTopSites: readonly ReasonlessTopSite[];
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-top-provenance-census.json");
const writeMode = process.argv.includes("--write");
const reasonlessPattern = /AbstractClassValueV0::Top\s*\{\s*provenance:\s*None\s*\}/gu;
const legacyUnitPattern = /AbstractClassValueV0::Top(?!\s*\{)/gu;

const allowedReasonlessSites: readonly ReasonlessTopDisposition[] = [
  {
    sourcePath: "rust/crates/omena-abstract-value/src/tests.rs",
    functionName: "top_provenance_is_additive_to_the_legacy_json_shape",
    disposition: "legacyShapeFixture",
    authority: "the serialization fixture locks compatibility with the legacy Top JSON shape",
  },
  {
    sourcePath: "rust/crates/omena-streaming-ifds/src/lib.rs",
    functionName: "legacy_fact_from_key",
    disposition: "legacyCacheKey",
    authority: "the legacy top cache key carries no typed provenance payload",
  },
  {
    sourcePath: "rust/crates/omena-streaming-ifds/src/lib.rs",
    functionName: "representative_class_value",
    disposition: "legacyRoundTripFixture",
    authority: "the typed-cache fixture verifies the legacy reasonless representation",
  },
];

const rustSources = rustSourcePaths(path.join(repoRoot, "rust/crates"));
const reasonlessTopSites = rustSources
  .flatMap((sourcePath) =>
    collectSites(
      path.relative(repoRoot, sourcePath),
      fs.readFileSync(sourcePath, "utf8"),
      reasonlessPattern,
    ),
  )
  .toSorted(compareSites);
const legacyUnitTopSites = rustSources
  .flatMap((sourcePath) =>
    collectSites(
      path.relative(repoRoot, sourcePath),
      fs.readFileSync(sourcePath, "utf8"),
      legacyUnitPattern,
    ),
  )
  .toSorted(compareSites);

const allowedKeys = new Set(allowedReasonlessSites.map(siteKey));
const unclassifiedReasonlessTopSites = reasonlessTopSites.filter(
  (site) => !allowedKeys.has(siteKey(site)),
);

const injected = collectSites(
  "injected/new-producer.rs",
  "fn derive_value() { let value = AbstractClassValueV0::Top { provenance: None }; }",
  reasonlessPattern,
);
assert.deepEqual(
  injected,
  [{ sourcePath: "injected/new-producer.rs", functionName: "derive_value" }],
  "the predicate must detect a newly introduced reasonless Top producer",
);
assert.equal(
  injected.some((site) => allowedKeys.has(siteKey(site))),
  false,
  "an unknown reasonless Top producer must not match a disposition",
);

assert.deepEqual(
  reasonlessTopSites,
  allowedReasonlessSites.map(({ sourcePath, functionName }) => ({ sourcePath, functionName })),
  "reasonless Top sites must be explicitly classified",
);
assert.deepEqual(legacyUnitTopSites, [], "the retired unit Top constructor must not return");

const census: TopProvenanceCensus = {
  schemaVersion: "0",
  product: "omena.top-provenance-census",
  reasonlessTopDispositions: allowedReasonlessSites,
  unclassifiedReasonlessTopSites,
  legacyUnitTopSites,
};
const serialized = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
} else {
  assert.deepEqual(
    JSON.parse(fs.readFileSync(censusPath, "utf8")),
    census,
    "Top provenance census is stale",
  );
}

process.stdout.write(
  `Omena Top provenance census OK: classified=${reasonlessTopSites.length} unclassified=0 legacyUnit=0\n`,
);

function collectSites(sourcePath: string, source: string, pattern: RegExp): ReasonlessTopSite[] {
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

function siteKey(site: ReasonlessTopSite): string {
  return `${site.sourcePath}#${site.functionName}`;
}

function compareSites(left: ReasonlessTopSite, right: ReasonlessTopSite): number {
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

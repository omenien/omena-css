import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const repositoryRoot = resolve(dirname(scriptPath), "..");
const bundlerSourcePath = resolve(repositoryRoot, "rust/crates/omena-bundler/src/lib.rs");
const injectClosureClaim = process.argv.includes("--inject-resolution-closure-claim");
const legacyEntryPointLimit =
  "Legacy LinkedStylesheetV0 entry points do not expose dependency resolution provenance.";
const instanceAttributionLimit =
  "InstanceAttributed remains unproduced; path-union reachability is a disclosed over-approximation.";
const censusFalsifierToken = "ResolutionAuthorityCensusDrift";
const actualBundlerSource = readFileSync(bundlerSourcePath, "utf8");
const bundlerSource = injectClosureClaim
  ? actualBundlerSource.replace(
      legacyEntryPointLimit,
      "All LinkedStylesheetV0 entry points expose dependency resolution provenance.",
    )
  : actualBundlerSource;

// FALSIFIER: id=resolution-authority-legacy-api-honesty class=accounting via=--inject-resolution-closure-claim producer=can-fail owner=resolution-authority-honesty entry=legacy-api-asymmetry-disclosed
assert.ok(
  bundlerSource.includes(legacyEntryPointLimit),
  "legacy entry-point provenance asymmetry is not disclosed",
);
// FALSIFIER: id=resolution-authority-instance-hole-honesty class=accounting via=--inject-resolution-closure-claim producer=can-fail owner=resolution-authority-honesty entry=instance-attribution-hole-disclosed
assert.ok(
  bundlerSource.includes(instanceAttributionLimit),
  "instance-attributed reachability hole is not disclosed",
);

const run = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "configured_sass_edges_select_distinct_instances_without_reparsing",
    "--",
    "--nocapture",
  ],
  {
    cwd: repositoryRoot,
    encoding: "utf8",
  },
);
const transcript = [run.stdout, run.stderr].filter(Boolean).join("\n");
// FALSIFIER: id=resolution-authority-product-census-run class=accounting via=ResolutionAuthorityCensusDrift producer=can-fail owner=resolution-authority-honesty entry=query-producer-census-test-green
assert.equal(run.status, 0, `${censusFalsifierToken}\n${transcript}`);
const censusMatch =
  /resolution authority census: expectedEdges=(\d+) disclosedEdges=(\d+) legacyEdges=(\d+)/u.exec(
    transcript,
  );
// FALSIFIER: id=resolution-authority-product-census-transcript class=accounting via=ResolutionAuthorityCensusDrift producer=can-fail owner=resolution-authority-honesty entry=query-producer-census-transcript-present
assert.ok(censusMatch, `${censusFalsifierToken}\n${transcript}`);
const expectedEdges = Number(censusMatch?.[1]);
const disclosedEdges = Number(censusMatch?.[2]);
const legacyEdges = Number(censusMatch?.[3]);
// FALSIFIER: id=resolution-authority-product-census-complete class=accounting via=ResolutionAuthorityCensusDrift producer=can-fail owner=resolution-authority-honesty entry=one-disclosure-per-product-edge
assert.equal(disclosedEdges, expectedEdges);
// FALSIFIER: id=resolution-authority-product-fallback-structural class=structuralEntailment via=STRUCTURAL producer=entailed owner=resolution-authority-honesty entry=query-producer-fully-supplies-resolved-edges reentry=first-product-producer-under-supplies-a-resolved-edge
assert.equal(legacyEdges, 0);

console.log(
  JSON.stringify({
    expectedEdges,
    disclosedEdges,
    legacyEdges,
    legacyEntryPointProvenanceVisible: false,
    instanceAttributedProduced: false,
  }),
);

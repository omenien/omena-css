import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const read = (relativePath: string): string =>
  readFileSync(path.join(repoRoot, relativePath), "utf8");

const source = read("rust/crates/omena-refinement/src/lib.rs");
const readme = read("rust/crates/omena-refinement/README.md");
const normalizedReadme = readme.replace(/\s+/g, " ");

for (const requiredSource of [
  "CascadeDimensionalRefinementBridgeV0",
  "CascadeDimensionalRefinementContextEvaluationV0",
  "summarize_cascade_dimensional_refinement_bridge_v0",
  "omena-refinement.cascade-dimensional-refinement-bridge",
  "m6DimensionalRefinementBridgeSubstrate",
  "uses_existing_abstract_property_value_substrate: true",
  "uses_existing_cascade_family_substrate: true",
  "uses_existing_refinement_predicate_substrate: true",
  "forks_unit_system: false",
  "liquid_haskell_complete: false",
  "smt_complete: false",
  "theorem_claimed: false",
  "product_path_evidence_ready: true",
  "stronger_type_safety_claim_ready: false",
  "cascade_dimensional_refinement_bridge_reuses_existing_substrates",
]) {
  assertIncludes(source, requiredSource);
}

for (const requiredReadme of [
  "summarize_cascade_dimensional_refinement_bridge_v0",
  "CascadeValueFamilyV0",
  "RefinementPropertyPredicateV0",
  "research-staged #69 substrate only",
  "does not fork a unit system",
  "complete Liquid-Haskell-style inference",
  "complete SMT refinement",
  "claim a theorem",
]) {
  assertIncludes(normalizedReadme, requiredReadme);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.m6-dimensional-refinement",
      bridgeReady: true,
      usesExistingCascadeFamilySubstrate: true,
      usesExistingRefinementPredicateSubstrate: true,
      forksUnitSystem: false,
      liquidHaskellComplete: false,
      smtComplete: false,
      theoremClaimed: false,
      evidenceClaimLevel: "m6DimensionalRefinementBridgeSubstrate",
    },
    null,
    2,
  )}\n`,
);

function assertIncludes(haystack: string, needle: string): void {
  assert.ok(haystack.includes(needle), `expected M6 dimensional refinement surface to include ${needle}`);
}

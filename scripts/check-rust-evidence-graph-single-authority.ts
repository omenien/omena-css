import { strict as assert } from "node:assert";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface MigratedFamily {
  readonly family: string;
  readonly file: string;
  readonly requiredSymbols: readonly string[];
}

interface Survivor {
  readonly family: string;
  readonly file: string;
  readonly requiredSymbols: readonly string[];
  readonly forbiddenSymbols: readonly string[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function listRustFiles(relativeDir: string): string[] {
  const absoluteDir = path.join(repoRoot, relativeDir);
  return readdirSync(absoluteDir)
    .flatMap((entry) => {
      const relativePath = path.join(relativeDir, entry);
      const absolutePath = path.join(repoRoot, relativePath);
      if (statSync(absolutePath).isDirectory()) {
        return listRustFiles(relativePath);
      }
      return relativePath.endsWith(".rs") ? [relativePath] : [];
    })
    .sort();
}

function assertIncludes(source: string, needle: string, context: string): void {
  assert.ok(source.includes(needle), `${context} must include ${needle}`);
}

function countMatches(source: string, pattern: RegExp): number {
  return [...source.matchAll(pattern)].length;
}

const migratedFamilies: readonly MigratedFamily[] = [
  {
    family: "query diagnostics and precision",
    file: "rust/crates/omena-query/src/types.rs",
    requiredSymbols: [
      "use omena_evidence_graph::",
      "project_omena_query_evidence_node",
      "project_omena_query_diagnostic_provenance_from_evidence_graph",
      "source_diagnostic_precision_node",
      "diagnostic_provenance_projection_preserves_legacy_labels",
      "source_diagnostic_precision_projects_byte_identical_shape",
      "GuaranteeKindV0::for_label_less_family()",
    ],
  },
  {
    family: "transform pass provenance",
    file: "rust/crates/omena-transform-passes/src/model.rs",
    requiredSymbols: [
      "use omena_evidence_graph::",
      "impl TransformPassExecutionOutcomeV0",
      "impl TransformProvenanceDerivationNodeV0",
      "impl TransformProvenanceDerivationForestV0",
      "transform_outcome_evidence_graph_preserves_public_shape",
      "transform_derivation_forest_evidence_graph_preserves_public_shape",
      "GuaranteeKindV0::for_label_less_family()",
    ],
  },
  {
    family: "transform CST cascade safety",
    file: "rust/crates/omena-transform-cst/src/lib.rs",
    requiredSymbols: [
      "use omena_evidence_graph::",
      "impl CascadeSafetyWitnessV0",
      "cascade_safety_witness_evidence_graph_preserves_public_shape",
      "verified_artifact_builder_routes_through_typestate_report",
      "GuaranteeKindV0::for_label_less_family()",
    ],
  },
  {
    family: "cascade proof records",
    file: "rust/crates/omena-cascade-proof/src/lib.rs",
    requiredSymbols: [
      "use omena_evidence_graph::",
      "impl TransformRewriteProofInputV0",
      "impl CascadeSMTProofV0",
      "rewrite_proof_input_evidence_graph_preserves_public_shape",
      "cascade_proof_record_evidence_graph_preserves_public_shape",
      "GuaranteeKindV0::for_label_less_family()",
    ],
  },
  {
    family: "incremental layer evidence",
    file: "rust/crates/omena-incremental/src/lib.rs",
    requiredSymbols: [
      "use omena_evidence_graph::",
      "impl IncrementalAlphaEquivalenceHashV0",
      "impl IncrementalShadowDeltaOracleV0",
      "impl IncrementalEditDistancePriorityInputV0",
      "impl IncrementalInvalidationPriorityPlanV0",
      "impl IncrementalLayerEvidenceV0",
      "incremental_claim_levels_round_trip_to_guarantee_kinds",
      "incremental_layer_evidence_graph_preserves_public_shape",
      "GuaranteeKindV0::from_existing_label",
    ],
  },
  {
    family: "abstract class-value provenance",
    file: "rust/crates/omena-abstract-value/src/provenance.rs",
    requiredSymbols: [
      "use omena_evidence_graph::",
      "impl AbstractClassValueProvenanceTreeV0",
      "GuaranteeKindV0::for_label_less_family()",
    ],
  },
  {
    family: "abstract class-value provenance guard",
    file: "rust/crates/omena-abstract-value/src/tests.rs",
    requiredSymbols: ["abstract_value_provenance_tree_evidence_graph_preserves_public_shape"],
  },
];

const survivors: readonly Survivor[] = [
  {
    family: "cross-file polylog bound scope",
    file: "rust/crates/omena-cross-file-summary/src/lib.rs",
    requiredSymbols: ['polylog_bound_scope: "notClaimedExactTraversal"'],
    forbiddenSymbols: ["omena_evidence_graph::", "EvidenceGraphV0"],
  },
  {
    family: "streaming IFDS module claim note",
    file: "rust/crates/omena-streaming-ifds/src/lib.rs",
    requiredSymbols: ["//! claim_level:"],
    forbiddenSymbols: ["omena_evidence_graph::", "EvidenceGraphV0"],
  },
];

for (const family of migratedFamilies) {
  const source = read(family.file);
  for (const symbol of family.requiredSymbols) {
    assertIncludes(source, symbol, `${family.family} (${family.file})`);
  }
}

for (const survivor of survivors) {
  const source = read(survivor.file);
  for (const symbol of survivor.requiredSymbols) {
    assertIncludes(source, symbol, `${survivor.family} survivor (${survivor.file})`);
  }
  for (const symbol of survivor.forbiddenSymbols) {
    assert.ok(
      !source.includes(symbol),
      `${survivor.family} survivor (${survivor.file}) must stay outside the migrated evidence graph population`,
    );
  }
}

const graphSource = read("rust/crates/omena-evidence-graph/src/lib.rs");
for (const symbol of [
  "pub enum GuaranteeKindV0",
  "pub enum GuaranteeFamilyV0",
  "ByteIdentityOracle",
  "ExternalReplicaDifferential",
  "PropertyCorpusWitness",
  "TypedInvariantWitness",
  "ProseObligationDischarged",
  "FloorAssumption",
  "LedgerBackedObligationDischarge",
  "pub struct EvidenceGraphV0",
  "build_salsa_demand_evidence_graph_v0",
  "build_evidence_graph_from_edges_v0",
  "salsa_demand_graph_keys_on_edges_not_the_full_node_list",
  "salsa_demand_graph_rejects_fabricated_edges",
]) {
  assertIncludes(graphSource, symbol, "evidence graph authority");
}
assert.ok(
  !graphSource.includes("#[salsa::tracked]") && !graphSource.includes("#[salsa::input]"),
  "omena-evidence-graph must remain a pure builder/model crate without salsa re-architecture",
);

for (const forbiddenClaim of [
  "DifferentiallyValidated",
  "Certified",
  "Theorem",
  "theorem",
  "validated",
  "certified",
  "verified",
  "Verified",
  "SolverDischarged",
  "SolverChecked",
]) {
  assert.ok(
    !graphSource.includes(forbiddenClaim),
    `evidence graph guarantee vocabulary must not introduce ${forbiddenClaim}`,
  );
}

const queryTypesSource = read("rust/crates/omena-query/src/types.rs");
assert.ok(
  !queryTypesSource.includes("extend_omena_query_checker_product_gate_provenance"),
  "query diagnostics must not retain the legacy checker-product provenance extender",
);

const querySourceFiles = listRustFiles("rust/crates/omena-query/src");

const queryDiagnosticSeedSiteCount = querySourceFiles.reduce(
  (count, file) => count + countMatches(read(file), /provenance:\s*vec!\[/g),
  0,
);
assert.equal(
  queryDiagnosticSeedSiteCount,
  0,
  "query diagnostic and analysis-result provenance must be populated through the evidence graph authority",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.evidence-graph-single-authority",
      migratedFamilyCount: migratedFamilies.length,
      outOfMigrationSurvivorCount: survivors.length,
      queryDiagnosticSeedSiteCount,
      migratedFamilies: migratedFamilies.map((family) => family.family),
      outOfMigrationSurvivors: survivors.map((survivor) => survivor.family),
      violations: 0,
    },
    null,
    2,
  )}\n`,
);

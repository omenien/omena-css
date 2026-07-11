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

interface StampRequirement {
  readonly family: string;
  readonly file: string;
  readonly requiredSymbols: readonly string[];
  readonly forbiddenSymbols: readonly string[];
}

interface ClassifiedStampSite {
  readonly file: string;
  readonly ordinal: number;
  readonly family: string;
}

interface ProductionStampSite {
  readonly file: string;
  readonly ordinal: number;
  readonly symbol: string;
  readonly line: number;
}

interface FamilyStampCallerSite {
  readonly file: string;
  readonly line: number;
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

function lineNumberAt(source: string, index: number): number {
  return source.slice(0, index).split(/\r?\n/u).length;
}

function isInsideCfgTestModule(source: string, index: number): boolean {
  const prefix = source.slice(0, index);
  const cfgTestIndex = prefix.lastIndexOf("#[cfg(test)]");
  if (cfgTestIndex < 0) {
    return false;
  }
  const testModuleIndex = prefix.indexOf("mod tests", cfgTestIndex);
  if (testModuleIndex < 0) {
    return false;
  }
  const nextModuleIndex = prefix.indexOf("\nmod ", testModuleIndex + "mod tests".length);
  return nextModuleIndex < 0;
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

const stampRequirements: readonly StampRequirement[] = [
  {
    family: "cascade proof prose obligation evidence",
    file: "rust/crates/omena-cascade-proof/src/lib.rs",
    requiredSymbols: [
      "ProseObligationProvenanceV0::from_provenance_labels",
      "FamilyStampV0::prose_obligation_discharged",
      "EvidenceNodeSeedV0::with_family(",
    ],
    forbiddenSymbols: ["EvidenceNodeSeedV0::new("],
  },
  {
    family: "transform CST prose obligation evidence",
    file: "rust/crates/omena-transform-cst/src/lib.rs",
    requiredSymbols: [
      "ProseObligationProvenanceV0::from_provenance_labels",
      "FamilyStampV0::prose_obligation_discharged",
      "EvidenceNodeSeedV0::with_family(",
    ],
    forbiddenSymbols: ["EvidenceNodeSeedV0::new("],
  },
  {
    family: "incremental typed invariant evidence",
    file: "rust/crates/omena-incremental/src/lib.rs",
    requiredSymbols: [
      "TypedInvariantWitnessTokenV0::from_incremental_layer_evidence",
      "FamilyStampV0::typed_invariant_witness",
      "EvidenceNodeSeedV0::with_family(",
    ],
    forbiddenSymbols: ["EvidenceNodeSeedV0::new("],
  },
  {
    family: "diff-test property corpus witness evidence",
    file: "rust/crates/omena-diff-test/src/transform_pass_cascade_conformance.rs",
    requiredSymbols: [
      "PropertyCorpusWitnessTokenV0::from_conformance_ledger",
      "FamilyStampV0::property_corpus_witness",
      "EvidenceNodeSeedV0::with_family(",
      "GuaranteeKindV0::from_existing_label(",
    ],
    forbiddenSymbols: ["EvidenceNodeSeedV0::new("],
  },
];

const guaranteeFamilies = [
  "ByteIdentityOracle",
  "ExternalReplicaDifferential",
  "ExternalTool",
  "PropertyCorpusWitness",
  "TypedInvariantWitness",
  "ProseObligationDischarged",
  "FloorAssumption",
  "LedgerBackedObligationDischarge",
] as const;
const expectedGuaranteeFamilies =
  process.env.OMENA_EVIDENCE_GRAPH_TEST_DROP_EXTERNAL_TOOL_FAMILY === "1"
    ? guaranteeFamilies.filter((family) => family !== "ExternalTool")
    : [...guaranteeFamilies];

const classifiedStampSites: readonly ClassifiedStampSite[] = [
  {
    file: "rust/crates/omena-abstract-value/src/provenance.rs",
    ordinal: 0,
    family: "FloorAssumption",
  },
  {
    file: "rust/crates/omena-cascade-proof/src/lib.rs",
    ordinal: 0,
    family: "ProseObligationDischarged",
  },
  {
    file: "rust/crates/omena-cascade-proof/src/lib.rs",
    ordinal: 1,
    family: "ProseObligationDischarged",
  },
  {
    file: "rust/crates/omena-cascade-proof/src/lib.rs",
    ordinal: 2,
    family: "LedgerBackedObligationDischarge",
  },
  {
    file: "rust/crates/omena-cascade-proof/src/lib.rs",
    ordinal: 3,
    family: "ProseObligationDischarged",
  },
  {
    file: "rust/crates/omena-diff-test/src/transform_pass_cascade_conformance.rs",
    ordinal: 0,
    family: "PropertyCorpusWitness",
  },
  {
    file: "rust/crates/omena-incremental/src/lib.rs",
    ordinal: 0,
    family: "TypedInvariantWitness",
  },
  {
    file: "rust/crates/omena-query/src/types.rs",
    ordinal: 0,
    family: "FloorAssumption",
  },
  {
    file: "rust/crates/omena-transform-cst/src/lib.rs",
    ordinal: 0,
    family: "ProseObligationDischarged",
  },
  {
    file: "rust/crates/omena-transform-passes/src/model.rs",
    ordinal: 0,
    family: "FloorAssumption",
  },
  {
    file: "rust/crates/omena-transform-passes/src/model.rs",
    ordinal: 1,
    family: "FloorAssumption",
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

for (const requirement of stampRequirements) {
  const source = read(requirement.file);
  for (const symbol of requirement.requiredSymbols) {
    assertIncludes(source, symbol, `${requirement.family} stamp site (${requirement.file})`);
  }
  for (const symbol of requirement.forbiddenSymbols) {
    assert.ok(
      !source.includes(symbol),
      `${requirement.family} stamp site (${requirement.file}) must not default mechanism evidence through ${symbol}`,
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

for (const family of expectedGuaranteeFamilies) {
  assertIncludes(graphSource, family, "evidence graph guarantee family registry");
}

const guaranteeKindBlock = graphSource.match(/pub enum GuaranteeKindV0 \{([\s\S]*?)\n\}/u);
assert.ok(guaranteeKindBlock, "GuaranteeKindV0 enum block must be discoverable");
const guaranteeKindVariants = guaranteeKindBlock[1]
  .split(/\r?\n/u)
  .map((line) => line.trim().replace(/,$/u, ""))
  .filter((line) => line.length > 0);
assert.deepEqual(
  guaranteeKindVariants,
  [
    "Floor",
    "SampledFixtureWitness",
    "SchedulerPriorityFixtureWitness",
    "MetricInputFixtureWitness",
    "IncrementalLayerEvidenceOnly",
    "AlphaRenamingStableHashFixtureWitness",
    "NotClaimedExactTraversal",
  ],
  "GuaranteeKindV0 must stay at the existing seven variants",
);

const guaranteeFamilyBlock = graphSource.match(/pub enum GuaranteeFamilyV0 \{([\s\S]*?)\n\}/u);
assert.ok(guaranteeFamilyBlock, "GuaranteeFamilyV0 enum block must be discoverable");
const guaranteeFamilyVariants = guaranteeFamilyBlock[1]
  .split(/\r?\n/u)
  .map((line) => line.trim().replace(/,$/u, ""))
  .filter((line) => line.length > 0);
assert.deepEqual(
  guaranteeFamilyVariants,
  expectedGuaranteeFamilies,
  "GuaranteeFamilyV0 must stay at the closed mechanism-family set",
);

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

const productionStampSites = listRustFiles("rust/crates")
  .filter((file) => file !== "rust/crates/omena-evidence-graph/src/lib.rs")
  .flatMap((file) => {
    const source = read(file);
    const matches = [
      ...source.matchAll(/GuaranteeKindV0::for_label_less_family\s*\(/gu),
      ...source.matchAll(/GuaranteeKindV0::from_existing_label\s*\(/gu),
    ]
      .filter((match) => match.index !== undefined)
      .filter((match) => !isInsideCfgTestModule(source, match.index ?? 0))
      .sort((left, right) => (left.index ?? 0) - (right.index ?? 0));
    return matches.map((match, ordinal): ProductionStampSite => {
      const index = match.index ?? 0;
      return {
        file,
        ordinal,
        symbol: match[0].replace(/\s*\($/u, ""),
        line: lineNumberAt(source, index),
      };
    });
  })
  .sort((left, right) =>
    left.file === right.file ? left.ordinal - right.ordinal : left.file.localeCompare(right.file),
  );

const classifiedByKey = new Map(
  classifiedStampSites.map((site) => [`${site.file}:${site.ordinal}`, site] as const),
);
const discoveredByKey = new Map(
  productionStampSites.map((site) => [`${site.file}:${site.ordinal}`, site] as const),
);

assert.ok(productionStampSites.length > 0, "production guarantee stamp census must be non-vacuous");
assert.equal(
  productionStampSites.length,
  classifiedStampSites.length,
  `production guarantee stamp census mismatch: discovered ${JSON.stringify(productionStampSites)}`,
);
assert.equal(
  productionStampSites.length,
  11,
  "production guarantee stamp census must cover 11 sites",
);

for (const site of classifiedStampSites) {
  assert.ok(
    guaranteeFamilies.includes(site.family as (typeof guaranteeFamilies)[number]),
    `${site.file}:${site.ordinal} uses an unknown guarantee family ${site.family}`,
  );
  assert.ok(
    discoveredByKey.has(`${site.file}:${site.ordinal}`),
    `classified guarantee stamp site ${site.file}:${site.ordinal} must exist in production scan`,
  );
}

for (const site of productionStampSites) {
  assert.ok(
    classifiedByKey.has(`${site.file}:${site.ordinal}`),
    `production guarantee stamp site ${site.file}:${site.ordinal} must be classified`,
  );
}

const ledgerFamilySiteCount = classifiedStampSites.filter(
  (site) => site.family === "LedgerBackedObligationDischarge",
).length;
assert.equal(
  ledgerFamilySiteCount,
  1,
  "ledger-backed guarantee family must be classified at its lookup stamp site",
);

const ledgerStampCallerSites = listRustFiles("rust/crates")
  .filter((file) => file !== "rust/crates/omena-evidence-graph/src/lib.rs")
  .flatMap((file): FamilyStampCallerSite[] => {
    const source = read(file);
    return [...source.matchAll(/FamilyStampV0::ledger_backed_obligation_discharge\s*\(/g)].map(
      (match) => ({
        file,
        line: lineNumberAt(source, match.index ?? 0),
      }),
    );
  });
const ledgerStampCallerCount = ledgerStampCallerSites.length;
assert.equal(
  ledgerStampCallerCount,
  1,
  "ledger-backed guarantee stamp must have exactly one live caller",
);
assert.deepEqual(
  ledgerStampCallerSites.map((site) => site.file),
  ["rust/crates/omena-cascade-proof/src/lib.rs"],
  "ledger-backed guarantee stamp caller must stay in cascade proof records",
);

const externalToolFamilySiteCount = classifiedStampSites.filter(
  (site) => site.family === "ExternalTool",
).length;
assert.equal(
  externalToolFamilySiteCount,
  0,
  "external-tool guarantee family must remain dormant until a named consumer owns the stamp",
);

const externalToolStampCallerSites = listRustFiles("rust/crates")
  .filter((file) => file !== "rust/crates/omena-evidence-graph/src/lib.rs")
  .flatMap((file): FamilyStampCallerSite[] => {
    const source = read(file);
    return [...source.matchAll(/FamilyStampV0::external_tool\s*\(/g)].map((match) => ({
      file,
      line: lineNumberAt(source, match.index ?? 0),
    }));
  });
if (process.env.OMENA_EVIDENCE_GRAPH_TEST_INJECT_EXTERNAL_TOOL_CALLER === "1") {
  externalToolStampCallerSites.push({
    file: "rust/crates/injected-external-tool-consumer/src/lib.rs",
    line: 1,
  });
}
const externalToolStampCallerCount = externalToolStampCallerSites.length;
assert.equal(
  externalToolStampCallerCount,
  0,
  `external-tool guarantee stamp must remain dormant: ${JSON.stringify(externalToolStampCallerSites)}`,
);

const externalToolWitnessBlock = graphSource.match(
  /pub struct ExternalToolRunWitnessV0 \{([\s\S]*?)\n\}/u,
);
assert.ok(externalToolWitnessBlock, "external-tool invocation witness must be discoverable");
const externalToolWitnessFields = externalToolWitnessBlock[1]
  .split(/\r?\n/u)
  .map((line) => line.trim())
  .filter((line) => line.startsWith("pub "))
  .map((line) => line.replace(/^pub\s+/u, "").replace(/:.*$/u, ""));
assert.deepEqual(
  externalToolWitnessFields,
  ["tool_name", "tool_version", "input_digest", "exit_status"],
  "external-tool witness must contain invocation facts only",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.evidence-graph-single-authority",
      migratedFamilyCount: migratedFamilies.length,
      guaranteeFamilyCount: guaranteeFamilies.length,
      productionStampSiteCount: productionStampSites.length,
      classifiedStampSiteCount: classifiedStampSites.length,
      ledgerFamilySiteCount,
      ledgerStampCallerCount,
      externalToolFamilySiteCount,
      externalToolStampCallerCount,
      outOfMigrationSurvivorCount: survivors.length,
      queryDiagnosticSeedSiteCount,
      migratedFamilies: migratedFamilies.map((family) => family.family),
      guaranteeFamilies,
      productionStampSites,
      classifiedStampSites,
      outOfMigrationSurvivors: survivors.map((survivor) => survivor.family),
      violations: 0,
    },
    null,
    2,
  )}\n`,
);

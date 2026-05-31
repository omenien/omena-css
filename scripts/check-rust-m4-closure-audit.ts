import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

const root = process.cwd();
const packageJson = JSON.parse(read("package.json")) as {
  readonly scripts: Record<string, string>;
};

const readinessScript = requiredScript("check:rust-m4-readiness");
const axisBClosureAudit = read("scripts/check-rust-m4-axis-b-closure-audit.ts");

const theoryClaimStatuses = [
  "descriptorOnly",
  "fixtureRecordOnly",
  "partialPropertyTest",
  "propertyTestEnforced",
] as const;
type TheoryClaimStatus = (typeof theoryClaimStatuses)[number];
type TheoryClaimStage = "m4-alpha" | "m4-beta" | "m4-gamma";
type TheoryClaimFraming =
  | "stagedScaffold"
  | "fixtureBound"
  | "partialMechanism"
  | "enforcedProperty";

type TheoryClaimEntry = {
  readonly id: string;
  readonly stage: TheoryClaimStage;
  readonly status: TheoryClaimStatus;
  readonly framing: TheoryClaimFraming;
  readonly surface: string;
  readonly evidencePath: string;
  readonly evidenceMarkers: readonly string[];
  readonly nextAction?: string;
};

const requiredReadinessTargets = [
  "rust/m4-axis-a-readiness",
  "rust/m4-axis-b-readiness",
  "rust/m4-axis-c-readiness",
  "rust/m4-axis-d-readiness",
  "rust/z5-performance-baseline-readiness",
  "rust/m4-closure-audit",
] as const;
const requiredAxisClosureScripts = [
  "check:rust-m4-axis-a-closure-audit",
  "check:rust-m4-axis-b-closure-audit",
  "check:rust-m4-axis-c-closure-audit",
  "check:rust-m4-axis-d-closure-audit",
] as const;

for (const target of requiredReadinessTargets) {
  assertIncludes(readinessScript, target, `M4 readiness must include ${target}`);
}

for (const scriptName of requiredAxisClosureScripts) {
  requiredScript(scriptName);
}

for (const scriptName of [
  "check:rust-m4-axis-a-readiness",
  "check:rust-m4-axis-b-readiness",
  "check:rust-m4-axis-c-readiness",
  "check:rust-m4-axis-d-readiness",
  "check:rust-z5-performance-baseline-readiness",
] as const) {
  requiredScript(scriptName);
}

assertIncludes(
  axisBClosureAudit,
  "requiredForM4Close: false",
  "M4 aggregate audit must record #38 real-workspace acceptance as deferred, not blocking",
);
assertIncludes(
  axisBClosureAudit,
  "packagedGate",
  "M4 aggregate audit must retain packaged LSP protocol gate tracking for #38",
);
const theoryClaimGuard = buildTheoryClaimGuard();
assertTheoryClaimGuard(theoryClaimGuard);

const status = "m4Ready";

process.stdout.write(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.m4-closure-audit",
      status,
      m4Complete: true,
      readinessScript: "check:rust-m4-readiness",
      closureAudits: [...requiredAxisClosureScripts, "check:rust-m4-closure-audit"],
      axes: {
        axisA: {
          gate: "rust/m4-axis-a-readiness",
          scope: "automation-testkit-conformance",
          localGateRequired: true,
        },
        axisB: {
          gate: "rust/m4-axis-b-readiness",
          scope: "issue-61-resolver-perimeter-and-issue-38-lsp-regression",
          localGateRequired: true,
          externalWorkspaceAcceptanceRequiredForM4Close: false,
        },
        axisC: {
          gate: "rust/m4-axis-c-readiness",
          scope: "typed-provenance-and-cross-file-summary-edge-substrate",
          localGateRequired: true,
        },
        axisD: {
          gate: "rust/m4-axis-d-readiness",
          scope: "behavior-preserving-structural-splits",
          localGateRequired: true,
        },
      },
      benchmark: {
        gate: "rust/z5-performance-baseline-readiness",
        scope: "symmetric-benchmark-measurement-boundary",
        localGateRequired: true,
      },
      issue38: {
        githubIssue: "https://github.com/yongsk0066/css-module-explainer/issues/38",
        stateExpectedBeforeM4Close: "technical-regression-gates-green",
        currentLocalStatus: "root-cause-regression-gates-present",
        externalWorkspaceAcceptance: {
          requiredForM4Close: false,
          status: "deferred-to-maintainer-real-workspace-check",
        },
        packagedGate: "release/check/packaged-omena-lsp-server-type-fact-protocol",
      },
      theoryClaimGuard,
      nextPriorities: [
        "keepZkRealBackendDeferredBehindOptInFeature",
        "continueAxisARealCorpusAndSpecAuditExpansion",
        "continueAxisBResolverPerimeterEvidence",
      ],
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function read(relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}

function requiredScript(name: string): string {
  const script = packageJson.scripts[name];
  assert.equal(typeof script, "string", `${name} must be declared in package.json`);
  return script;
}

function assertIncludes(source: string, marker: string, message: string): void {
  assert.ok(source.includes(marker), message);
}

function buildTheoryClaimGuard(): {
  readonly ladder: readonly TheoryClaimStatus[];
  readonly legacyNotClaimed: Record<string, "notClaimed">;
  readonly stages: Record<TheoryClaimStage, readonly TheoryClaimEntry[]>;
  readonly summary: Record<TheoryClaimStatus, number>;
} {
  const entries: readonly TheoryClaimEntry[] = [
    {
      id: "m4-alpha.qtt-semiring-algebra",
      stage: "m4-alpha",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "QTT provenance semiring family",
      evidencePath: "rust/crates/omena-abstract-value/src/semiring.rs",
      evidenceMarkers: [
        "pub trait ProvenanceSemiringV0",
        "fn add(&self, lhs: &Self::Element, rhs: &Self::Element) -> Self::Element",
        "verify_provenance_semiring_laws_on_fixtures",
        "m4_alpha_provenance_semiring_law_reports_v0",
        "pub struct Lin01ProvenanceSemiringV0",
        "impl ProvenanceSemiringV0 for Lin01ProvenanceSemiringV0",
      ],
      nextAction:
        "keep partial-property wording until polynomial provenance and sheaf-valued lift land",
    },
    {
      id: "m4-alpha.grn-state-transition",
      stage: "m4-alpha",
      status: "propertyTestEnforced",
      framing: "enforcedProperty",
      surface: "GRN attractor basin proof over n <= 16",
      evidencePath: "rust/crates/omena-cascade/src/grn.rs",
      evidenceMarkers: [
        "transition_cascade_grn_state_v0",
        "enumerate_explicit_grn_attractor_v0",
        "GrnTransitionRecordV0",
        "prove_cascade_attractor_basin",
        "grn_explicit_attractor_basin_proof_covers_all_n_le_16",
        "grn_explicit_transition_function_enumerates_full_state_space",
      ],
    },
    {
      id: "m4-alpha.spin-glass-property-tests",
      stage: "m4-alpha",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "spin-glass 5-tier ultrametric corpus",
      evidencePath: "rust/crates/omena-cascade/src/statistics.rs",
      evidenceMarkers: [
        "CascadeUltrametricPathV0",
        "cascade_ultrametric_distance_v0",
        "verify_spin_glass_ultrametric_corpus_v0",
        "spin_glass_ultrametric_corpus_enforces_five_tier_strong_triangle",
      ],
      nextAction:
        "keep partial-property wording until the corpus is lifted from binary fixtures to real cascade topology extraction",
    },
    {
      id: "m4-alpha.mdl-differential-corpus",
      stage: "m4-alpha",
      status: "propertyTestEnforced",
      framing: "enforcedProperty",
      surface: "MDL 100-fixture differential corpus",
      evidencePath: "package.json",
      evidenceMarkers: [
        "check:rust-m4-alpha-mdl-differential",
        "mdl_default_ast_size_matches_100_fixture_differential_corpus",
      ],
    },
    {
      id: "m4-alpha.frame-rule-fuzz",
      stage: "m4-alpha",
      status: "propertyTestEnforced",
      framing: "enforcedProperty",
      surface: "frame-rule overapproximation fuzz gate",
      evidencePath: "package.json",
      evidenceMarkers: [
        "check:rust-m4-alpha-frame-rule-fuzz",
        "frame_rule_overapprox",
        "M4_ALPHA_FRAME_FUZZ_RUNS:-100000",
      ],
    },
    {
      id: "m4-beta.lawvere-equation-cluster",
      stage: "m4-beta",
      status: "descriptorOnly",
      framing: "stagedScaffold",
      surface: "Lawvere equation cluster catalog",
      evidencePath: "rust/crates/omena-lawvere/src/lib.rs",
      evidenceMarkers: ["LawvereEquationClusterV0", "lawvere_equation_clusters_v0"],
      nextAction:
        "keep staged-scaffold wording until a fixture corpus or semantic law checker lands",
    },
    {
      id: "m4-beta.lawvere-reorderability-certificate",
      stage: "m4-beta",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "Lawvere reorderability certificate with differential commutativity witness",
      evidencePath: "rust/crates/omena-lawvere/src/lib.rs",
      evidenceMarkers: [
        "ReorderabilityCertificateV0",
        "LawvereDifferentialCommutativityWitnessV0",
        "reorderability_certificate_from_differential_v0",
        "requiresDifferentialCommutativityWitness",
        "differentialCommutativityCorpus",
      ],
      nextAction: "expand corpus coverage before final Lawvere semantics wording",
    },
    {
      id: "m4-beta.rg-flow-fixed-point",
      stage: "m4-beta",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "RG beta vector, fixed-point metric, and coupling Jacobian spectrum",
      evidencePath: "rust/crates/omena-rg-flow/src/lib.rs",
      evidenceMarkers: [
        "BetaVectorV0",
        "CouplingJacobianSpectrumV0",
        "estimate_coupling_jacobian_spectrum_v0",
        "fixed_point_reached",
        "beta_vector_from_couplings",
      ],
      nextAction: "continue product wiring once downstream RG-flow consumers are selected",
    },
    {
      id: "m4-beta.hypergraph-ifds-summary",
      stage: "m4-beta",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "hypergraph IFDS projection seed",
      evidencePath: "rust/crates/omena-query/src/style/cross_file_hypergraph/reachability.rs",
      evidenceMarkers: [
        "OmenaUnifiedHypergraphConnectivityOracle",
        "tabulate_hypergraph_ifds_summary_edges",
      ],
      nextAction: "treat as seed substrate; full streaming IFDS lives in m4-gamma",
    },
    {
      id: "m4-beta.replica-ensemble-projection",
      stage: "m4-beta",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "replica overlap and spectral ensemble projection",
      evidencePath: "rust/crates/omena-ensemble/src/types.rs",
      evidenceMarkers: ["LocalTwoComponentEm", "AutoSpectral", "SpectralMethod"],
      nextAction: "keep empirical/projection wording until real EM or spectral inference lands",
    },
    {
      id: "m4-gamma.z3-backend-and-retired-descriptor-backends",
      stage: "m4-gamma",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "SMT backend selection surface with descriptor-only CVC5/Bitwuzla retired",
      evidencePath: "rust/crates/omena-smt/src/backend/z3.rs",
      evidenceMarkers: ["Z3SmtBackendV0", "smt-z3", "z3"],
      nextAction:
        "keep Z3 opt-in unless the product lane explicitly accepts the solver dependency",
    },
    {
      id: "m4-gamma.smt-bisimulation-fuzz",
      stage: "m4-gamma",
      status: "propertyTestEnforced",
      framing: "enforcedProperty",
      surface: "SMT bisimulation fuzz case",
      evidencePath: "rust/crates/omena-smt/src/fuzz.rs",
      evidenceMarkers: [
        "smt_bisimulation_fuzz_case_v0",
        "run_smt_bisimulation_fuzz_case_v0",
        "SmtBisimulationFuzzReportV0",
      ],
    },
    {
      id: "m4-gamma.zk-protocol-surface",
      stage: "m4-gamma",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "ZK audit setup/circuit protocol surface with opt-in arkworks proof roundtrip",
      evidencePath: "rust/crates/omena-zk-audit/src/lib.rs",
      evidenceMarkers: [
        "SetupKindV0::Halo2Ipa",
        "SetupKindV0::ArkworksGroth16",
        "CascadeZKAuditV0",
        "ZKBackendLinkPolicyV0",
        "ArkworksGroth16RoundTripV0",
        "prove_and_verify_cascade_smt_payload_with_arkworks_v0",
        "arkworks_groth16_roundtrip_generates_and_verifies_proof",
        "zk_backend_link_policy_keeps_real_backends_feature_gated",
        "heavy_dependencies_default_off",
      ],
      nextAction:
        "keep default path heavy-free and expand circuit coverage before final ZK wording",
    },
    {
      id: "m4-gamma.refinement-type-system",
      stage: "m4-gamma",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "refinement type wrapper and predicate interface",
      evidencePath: "rust/crates/omena-refinement/src/lib.rs",
      evidenceMarkers: [
        "RefinedAbstractPropertyValueV0",
        "RefinementPropertyPredicateV0",
        "evaluate_refinement_property_predicate_v0",
        "refinement_property_grammar_evaluates_exact_and_one_of_values",
        "refinement_predicate_composition_tracks_partial_and_negative_witnesses",
        "refinement_numeric_range_and_pseudo_state_predicates_are_evaluated",
        "refinement_context_digest_is_order_stable_and_invalidation_sensitive",
        "project_refined_to_legacy_v0",
      ],
      nextAction: "keep partial-property wording until SMT-backed predicate discharge lands",
    },
    {
      id: "m4-gamma.variational-posterior",
      stage: "m4-gamma",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "variational posterior and hover-budget surface",
      evidencePath: "rust/crates/omena-variational/src/lib.rs",
      evidenceMarkers: [
        "DesignerIntentPosteriorV0",
        "ProvenancePosteriorAnnotationV0",
        "mutates_existing_provenance_enum: false",
      ],
      nextAction:
        "keep stochastic inference disabled by default until deeper empirical calibration lands",
    },
    {
      id: "m4-gamma.categorical-fixture-evidence",
      stage: "m4-gamma",
      status: "propertyTestEnforced",
      framing: "enforcedProperty",
      surface: "fixture-backed categorical evidence endpoints",
      evidencePath: "rust/crates/omena-categorical/src/lib.rs",
      evidenceMarkers: [
        "CategoricalEndpointFixtureEvidenceV0",
        "categorical_fixture_evidence_for_endpoint_v0",
        "cascade_primitive_roles_v0",
      ],
    },
    {
      id: "m4-gamma.streaming-ifds-transfer-cache",
      stage: "m4-gamma",
      status: "partialPropertyTest",
      framing: "partialMechanism",
      surface: "streaming IFDS transfer and summary-cache substrate",
      evidencePath: "rust/crates/omena-streaming-ifds/src/lib.rs",
      evidenceMarkers: [
        "StreamingIFDSTransferFunctionV0",
        "StreamingIFDSSummaryCacheEntryV0",
        "PolylogDynamicConnectivityBackendV0",
      ],
      nextAction:
        "audit dynamic-connectivity algorithm depth separately from transfer/cache contracts",
    },
  ];

  for (const entry of entries) {
    assert.ok(
      theoryClaimStatuses.includes(entry.status),
      `${entry.id} has unknown theory claim status`,
    );
    assertTheoryClaimFraming(entry);
    const evidence = read(entry.evidencePath);
    for (const marker of entry.evidenceMarkers) {
      assertIncludes(
        evidence,
        marker,
        `${entry.id} evidence marker missing in ${entry.evidencePath}`,
      );
    }
  }

  const stages = groupTheoryClaimsByStage(entries);
  const summary = summarizeTheoryClaimStatuses(entries);

  return {
    ladder: theoryClaimStatuses,
    legacyNotClaimed: {
      dynamicDyck: "notClaimed",
      externalDatalog: "notClaimed",
      egglogExecution: "notClaimed",
      sheafOrModalTheorem: "notClaimed",
      fullPerceptualTooling: "notClaimed",
    },
    stages,
    summary,
  };
}

function assertTheoryClaimFraming(entry: TheoryClaimEntry): void {
  const allowedFramingByStatus: Record<TheoryClaimStatus, readonly TheoryClaimFraming[]> = {
    descriptorOnly: ["stagedScaffold"],
    fixtureRecordOnly: ["fixtureBound"],
    partialPropertyTest: ["partialMechanism"],
    propertyTestEnforced: ["enforcedProperty"],
  };
  assert.ok(
    allowedFramingByStatus[entry.status].includes(entry.framing),
    `${entry.id} has ${entry.status} but uses final/incompatible framing ${entry.framing}`,
  );
  if (entry.status !== "propertyTestEnforced") {
    assert.ok(
      entry.nextAction,
      `${entry.id} must name nextAction while it is not propertyTestEnforced`,
    );
  }
}

function assertTheoryClaimGuard(guard: ReturnType<typeof buildTheoryClaimGuard>): void {
  assert.deepEqual(
    guard.ladder,
    ["descriptorOnly", "fixtureRecordOnly", "partialPropertyTest", "propertyTestEnforced"],
    "theory claim ladder must keep the post-gamma four-tier order",
  );
  for (const stage of ["m4-alpha", "m4-beta", "m4-gamma"] as const) {
    assert.ok(guard.stages[stage].length >= 4, `${stage} must have explicit theory claim entries`);
  }
  for (const statusName of theoryClaimStatuses) {
    if (statusName === "fixtureRecordOnly") {
      continue;
    }
    assert.ok(
      guard.summary[statusName] > 0,
      `theory claim ladder must contain at least one ${statusName} entry`,
    );
  }
  assert.ok(
    guard.stages["m4-gamma"].some((entry) => entry.id === "m4-gamma.zk-protocol-surface"),
    "M4-gamma ZK protocol surface must stay explicitly classified until real backend work lands",
  );
  assert.ok(
    guard.stages["m4-alpha"].some((entry) => entry.id === "m4-alpha.qtt-semiring-algebra"),
    "M4-alpha semiring algebra risk must stay explicitly classified before ZK/refinement deepening",
  );
}

function groupTheoryClaimsByStage(
  entries: readonly TheoryClaimEntry[],
): Record<TheoryClaimStage, readonly TheoryClaimEntry[]> {
  return {
    "m4-alpha": entries.filter((entry) => entry.stage === "m4-alpha"),
    "m4-beta": entries.filter((entry) => entry.stage === "m4-beta"),
    "m4-gamma": entries.filter((entry) => entry.stage === "m4-gamma"),
  };
}

function summarizeTheoryClaimStatuses(
  entries: readonly TheoryClaimEntry[],
): Record<TheoryClaimStatus, number> {
  return Object.fromEntries(
    theoryClaimStatuses.map((statusName) => [
      statusName,
      entries.filter((entry) => entry.status === statusName).length,
    ]),
  ) as Record<TheoryClaimStatus, number>;
}

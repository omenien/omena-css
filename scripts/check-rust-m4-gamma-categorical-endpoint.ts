import { readFileSync } from "node:fs";
import { parseArgs } from "node:util";

const canonicalEndpoints = [
  "rust/omena-categorical/verify-cascade-section-aggregation-plan-stability",
  "rust/omena-categorical/verify-cascade-section-aggregation-covariance",
  "rust/omena-categorical/verify-beck-chevalley",
  "rust/omena-categorical/classify-omega-truth",
  "rust/omena-categorical/verify-s4-axioms",
  "rust/omena-categorical/verify-modal-imperative-equivalence",
  "rust/omena-categorical/verify-invariant-functoriality",
  "rust/omena-categorical/compare-design-system-theory",
  "rust/omena-categorical/summarize-kripke-frame",
  "rust/omena-categorical/verify-cross-project-symmetry",
] as const;

type CanonicalEndpoint = (typeof canonicalEndpoints)[number];

const expectedCanonicalFixtureIds: Record<CanonicalEndpoint, string> = {
  "rust/omena-categorical/verify-cascade-section-aggregation-plan-stability":
    "fixture.categorical.cascade-section-aggregation-plan-stability.v0",
  "rust/omena-categorical/verify-cascade-section-aggregation-covariance":
    "fixture.categorical.cascade-section-aggregation-covariance.v0",
  "rust/omena-categorical/verify-beck-chevalley": "fixture.categorical.beck-chevalley.v0",
  "rust/omena-categorical/classify-omega-truth": "fixture.categorical.omega-truth.v0",
  "rust/omena-categorical/verify-s4-axioms": "fixture.categorical.s4-axioms.v0",
  "rust/omena-categorical/verify-modal-imperative-equivalence":
    "fixture.categorical.modal-imperative-equivalence.v0",
  "rust/omena-categorical/verify-invariant-functoriality":
    "fixture.categorical.invariant-functoriality.v0",
  "rust/omena-categorical/compare-design-system-theory":
    "fixture.categorical.design-system-theory-compare.v0",
  "rust/omena-categorical/summarize-kripke-frame": "fixture.categorical.kripke-frame.v0",
  "rust/omena-categorical/verify-cross-project-symmetry":
    "fixture.categorical.cross-project-symmetry.v0",
};

/**
 * @deprecated Compatibility owner: omena-categorical maintainers. Removal condition:
 * not before 1.0; remove only after downstream migration and zero in-repo non-compat uses.
 */
const LEGACY_PLAN_ENDPOINT_V0 = "rust/omena-categorical/verify-site-stability";

/**
 * @deprecated Compatibility owner: omena-categorical maintainers. Removal condition:
 * not before 1.0; remove only after downstream migration and zero in-repo non-compat uses.
 */
const LEGACY_AGGREGATION_ENDPOINT_V0 = "rust/omena-categorical/verify-cosheaf-covariance";

/**
 * @deprecated Compatibility owner: omena-categorical maintainers. Removal condition:
 * not before 1.0; remove only after downstream migration and zero in-repo non-compat uses.
 */
const LEGACY_PLAN_FIXTURE_ID_V0 = "fixture.categorical.site-stability.v0";

/**
 * @deprecated Compatibility owner: omena-categorical maintainers. Removal condition:
 * not before 1.0; remove only after downstream migration and zero in-repo non-compat uses.
 */
const LEGACY_AGGREGATION_FIXTURE_ID_V0 = "fixture.categorical.cosheaf-covariance.v0";

type LegacyEndpointCompatibility = {
  readonly canonicalEndpoint: CanonicalEndpoint;
  readonly fixtureId: string;
};

const legacyEndpointCompatibility = new Map<string, LegacyEndpointCompatibility>([
  [
    LEGACY_PLAN_ENDPOINT_V0,
    {
      canonicalEndpoint: "rust/omena-categorical/verify-cascade-section-aggregation-plan-stability",
      fixtureId: LEGACY_PLAN_FIXTURE_ID_V0,
    },
  ],
  [
    LEGACY_AGGREGATION_ENDPOINT_V0,
    {
      canonicalEndpoint: "rust/omena-categorical/verify-cascade-section-aggregation-covariance",
      fixtureId: LEGACY_AGGREGATION_FIXTURE_ID_V0,
    },
  ],
]);

const { values } = parseArgs({
  options: {
    endpoint: { type: "string" },
  },
});

const requestedEndpoint = values.endpoint;
if (!requestedEndpoint) {
  throw new Error("Missing --endpoint");
}
const legacyCompatibility = legacyEndpointCompatibility.get(requestedEndpoint);
const canonicalEndpointCandidate = legacyCompatibility?.canonicalEndpoint ?? requestedEndpoint;
if (!canonicalEndpoints.includes(canonicalEndpointCandidate as CanonicalEndpoint)) {
  throw new Error(`Unknown M4-gamma categorical endpoint: ${requestedEndpoint}`);
}
const canonicalEndpoint = canonicalEndpointCandidate as CanonicalEndpoint;
const canonicalFixtureId = expectedCanonicalFixtureIds[canonicalEndpoint];
const fixtureId = legacyCompatibility?.fixtureId ?? canonicalFixtureId;

const categoricalSource = readFileSync("rust/crates/omena-categorical/src/lib.rs", "utf8");
const querySource = readFileSync("rust/crates/omena-query/src/types.rs", "utf8");
const lspSource = readFileSync("rust/crates/omena-lsp-server/src/lib.rs", "utf8");

for (const marker of new Set([
  requestedEndpoint,
  canonicalEndpoint,
  "CategoricalCascadeEvidenceV0",
  "CategoricalEndpointFixtureEvidenceV0",
  "CategoricalFixtureAssertionV0",
  "categorical_cascade_evidence_v0",
  "categorical_fixture_evidence_for_endpoint_v0",
  "endpoint_count: endpoints.len()",
  "fixture_evidence",
  "functor_applications",
  "apply_cascade_primitive_role_functor_v0",
  "primitive-role-composition-preservation",
  fixtureId,
  canonicalFixtureId,
])) {
  if (!categoricalSource.includes(marker)) {
    throw new Error(`omena-categorical endpoint fixture surface is missing ${marker}`);
  }
}

if (
  !querySource.includes("pub categorical_evidence:") ||
  !querySource.includes("Option<omena_query_checker_orchestrator::CategoricalCascadeEvidenceV0>")
) {
  throw new Error(
    "cascade-at-position query result must expose optional categorical evidence through the checker boundary",
  );
}

if (!lspSource.includes("includeCategoricalEvidence")) {
  throw new Error(
    "Rust LSP cascade-at-position request must expose the default-off categorical evidence gate",
  );
}

console.log(
  JSON.stringify(
    {
      product: "rust.omena-categorical.categorical-evidence-endpoint",
      endpoint: requestedEndpoint,
      evidenceProduct: "omena-categorical.cascade-evidence",
      fixtureId,
      fixtureBacked: true,
      schemaVersion: "0",
      layerMarker: "categorical-semantic",
      defaultEnabled: false,
    },
    null,
    2,
  ),
);

import { readFileSync } from "node:fs";
import { parseArgs } from "node:util";

const requiredEndpoints = [
  "rust/omena-categorical/verify-site-stability",
  "rust/omena-categorical/verify-cosheaf-covariance",
  "rust/omena-categorical/verify-beck-chevalley",
  "rust/omena-categorical/classify-omega-truth",
  "rust/omena-categorical/verify-s4-axioms",
  "rust/omena-categorical/verify-modal-imperative-equivalence",
  "rust/omena-categorical/verify-invariant-functoriality",
  "rust/omena-categorical/compare-design-system-theory",
  "rust/omena-categorical/summarize-kripke-frame",
  "rust/omena-categorical/verify-cross-project-symmetry",
] as const;

const { values } = parseArgs({
  options: {
    endpoint: { type: "string" },
  },
});

const endpoint = values.endpoint;
if (!endpoint) {
  throw new Error("Missing --endpoint");
}
if (!requiredEndpoints.includes(endpoint as (typeof requiredEndpoints)[number])) {
  throw new Error(`Unknown M4-gamma categorical endpoint: ${endpoint}`);
}

const categoricalSource = readFileSync("rust/crates/omena-categorical/src/lib.rs", "utf8");
const querySource = readFileSync("rust/crates/omena-query/src/types.rs", "utf8");
const lspSource = readFileSync("rust/crates/omena-lsp-server/src/lib.rs", "utf8");

for (const marker of [
  endpoint,
  "CategoricalCascadeEvidenceV0",
  "categorical_cascade_evidence_v0",
  "endpoint_count: endpoints.len()",
]) {
  if (!categoricalSource.includes(marker)) {
    throw new Error(`omena-categorical endpoint catalog is missing ${marker}`);
  }
}

if (!querySource.includes("pub categorical_evidence: Option<omena_categorical::CategoricalCascadeEvidenceV0>")) {
  throw new Error("cascade-at-position query result must expose optional categorical evidence");
}

if (!lspSource.includes("includeCategoricalEvidence")) {
  throw new Error("Rust LSP cascade-at-position request must expose the default-off categorical evidence gate");
}

console.log(
  JSON.stringify(
    {
      product: "rust.omena-categorical.categorical-evidence-endpoint",
      endpoint,
      evidenceProduct: "omena-categorical.cascade-evidence",
      schemaVersion: "0",
      layerMarker: "categorical-semantic",
      defaultEnabled: false,
    },
    null,
    2,
  ),
);

import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";

function read(path: string): string {
  return readFileSync(path, "utf8");
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

function assertIncludes(source: string, needle: string, message: string): void {
  assert(source.includes(needle), `${message}: missing ${needle}`);
}

const workspace = read("rust/Cargo.toml");
const packageJson = read("package.json");
const workspaceMembers = [...workspace.matchAll(/^\s*"([^"]+)",$/gmu)].map((match) => match[1]);
const gammaCrates = [
  "crates/omena-categorical",
  "crates/omena-smt",
  "crates/omena-zk-circuit",
  "crates/omena-zk-audit",
  "crates/omena-refinement",
  "crates/omena-refinement-trait",
  "crates/omena-variational",
  "crates/omena-streaming-ifds",
] as const;

assert(workspaceMembers.length === 41, `expected 41 workspace members, got ${workspaceMembers.length}`);
assert(
  workspaceMembers.filter((member) => member.includes("/omena-")).length === 38,
  "expected omena-* crate roster to be 38",
);
for (const cratePath of gammaCrates) {
  assert(workspaceMembers.includes(cratePath), `missing M4-gamma workspace member ${cratePath}`);
}

const heavyDependencyNames = [
  "ark-ff",
  "ark-poly",
  "ark-bn254",
  "ark-bls12-381",
  "halo2_proofs",
  "winterfell",
  "binius",
  "z3",
  "z3-sys",
  "cvc5",
  "bitwuzla",
  "bitwuzla-sys",
];

for (const cratePath of gammaCrates) {
  const manifestPath = `rust/${cratePath}/Cargo.toml`;
  const manifest = read(manifestPath);
  const dependencyBlock = /\[dependencies\]([\s\S]*?)(?:\n\[|$)/u.exec(manifest)?.[1] ?? "";
  for (const dependencyName of heavyDependencyNames) {
    const escapedDependencyName = dependencyName.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    assert(
      !new RegExp(`^\\s*${escapedDependencyName}\\s*=`, "mu").test(dependencyBlock),
      `${manifestPath} must not pull heavy dependency ${dependencyName} in default dependencies`,
    );
  }
}

const proofsBytes = readFileSync("rust/crates/omena-cascade/src/proofs.rs");
const proofsDigest = createHash("sha256").update(proofsBytes).digest("hex");
assert(
  proofsDigest === "24a402864688e9cf2e1a38e6c92031fab0a42a2016235524590a89c7703c6517",
  `legacy proofs.rs SHA-256 drifted: ${proofsDigest}`,
);

const categorical = read("rust/crates/omena-categorical/src/lib.rs");
const categoricalEndpointIds = [
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
for (const moduleName of [
  "site",
  "sheaf",
  "cosheaf",
  "colimit",
  "beck_chevalley",
  "omega",
  "modal",
  "kripke",
  "design_system_theory",
]) {
  assertIncludes(categorical, moduleName, "omena-categorical must expose all 9 required modules");
}
for (const primitive of [
  "prove_layer_flatten_candidate",
  "prove_scope_flatten_candidate",
  "prove_box_shorthand_combination",
  "evaluate_static_supports_condition",
]) {
  assertIncludes(categorical, primitive, "omena-categorical must map existing cascade primitive roles");
}
assertIncludes(categorical, "contract_count: 26", "omena-categorical must pin 26 V0 contracts");
assertIncludes(categorical, "CategoricalCascadeEvidenceV0", "omena-categorical must expose cascade evidence");
assertIncludes(
  categorical,
  "CategoricalEndpointFixtureEvidenceV0",
  "omena-categorical must expose fixture-backed endpoint evidence",
);
assertIncludes(
  categorical,
  "CategoricalFixtureAssertionV0",
  "omena-categorical must expose fixture assertions for endpoint evidence",
);
assertIncludes(
  categorical,
  "categorical_fixture_evidence_for_endpoint_v0",
  "omena-categorical must back endpoints with fixture evidence",
);
assertIncludes(
  categorical,
  "fixture_evidence",
  "categorical cascade evidence must include endpoint fixture evidence",
);
for (const endpointId of categoricalEndpointIds) {
  assertIncludes(categorical, endpointId, "omena-categorical must expose all 10 cme-check endpoints");
}
for (const fixtureId of [
  "fixture.categorical.site-stability.v0",
  "fixture.categorical.cosheaf-covariance.v0",
  "fixture.categorical.beck-chevalley.v0",
  "fixture.categorical.omega-truth.v0",
  "fixture.categorical.s4-axioms.v0",
  "fixture.categorical.modal-imperative-equivalence.v0",
  "fixture.categorical.invariant-functoriality.v0",
  "fixture.categorical.design-system-theory-compare.v0",
  "fixture.categorical.kripke-frame.v0",
  "fixture.categorical.cross-project-symmetry.v0",
]) {
  assertIncludes(categorical, fixtureId, "omena-categorical must expose fixture-backed evidence IDs");
}

const queryTypes = read("rust/crates/omena-query/src/types.rs");
assertIncludes(
  queryTypes,
  "pub categorical_evidence: Option<omena_categorical::CategoricalCascadeEvidenceV0>",
  "cascade-at-position response must carry optional categorical evidence",
);
const lspServer = read("rust/crates/omena-lsp-server/src/lib.rs");
assertIncludes(
  lspServer,
  "includeCategoricalEvidence",
  "Rust LSP cascade-at-position must keep categorical evidence default-off",
);

for (const smtPath of [
  "rust/crates/omena-smt/src/theory.rs",
  "rust/crates/omena-smt/src/encoder.rs",
  "rust/crates/omena-smt/src/obligations.rs",
  "rust/crates/omena-smt/src/proof.rs",
  "rust/crates/omena-smt/src/unsat_core.rs",
  "rust/crates/omena-smt/src/backend/stub.rs",
  "rust/crates/omena-smt/src/backend/z3.rs",
  "rust/crates/omena-smt/src/backend/cvc5.rs",
  "rust/crates/omena-smt/src/backend/bitwuzla.rs",
]) {
  assert(existsSync(smtPath), `missing SMT module ${smtPath}`);
}

const smt = read("rust/crates/omena-smt/src/lib.rs");
assertIncludes(smt, "proof_style_bisimulation_invariant_holds_for_all_l1_primitives", "SMT bisimulation invariant test must be present");
assertIncludes(smt, "static_supports_smt_equivalence_tracks_l1_verdict_shape", "SMT supports equivalence test must be present");
assertIncludes(smt, "smt_bisimulation_fuzz_seed_corpus_covers_m3_fixture_shapes", "SMT must cover the M3 fixture-shaped fuzz seed corpus");
const smtFuzz = read("rust/crates/omena-smt/src/fuzz.rs");
assertIncludes(smtFuzz, "SmtBisimulationFuzzCaseV0", "SMT must expose a bisimulation fuzz case contract");
assertIncludes(smtFuzz, "SmtBisimulationFuzzReportV0", "SMT must expose a bisimulation fuzz report contract");
assertIncludes(smtFuzz, "m3-cascade-proof-fixtures", "SMT fuzz evidence must identify the M3 cascade proof fixture suite");
assertIncludes(smtFuzz, "run_smt_bisimulation_fuzz_seed_corpus_v0", "SMT must expose deterministic fuzz seed corpus evidence");
const fuzzManifest = read("rust/fuzz/Cargo.toml");
assertIncludes(fuzzManifest, 'name = "smt_bisimulation"', "cargo-fuzz must expose the SMT bisimulation target");
assertIncludes(packageJson, "check:rust-m4-gamma-smt-verification", "M4-gamma readiness must exercise SMT verification");
assertIncludes(packageJson, "check:rust-m4-gamma-smt-fuzz-full", "M4-gamma must expose the full 1e6 SMT fuzz command");
assertIncludes(packageJson, "M4_GAMMA_SMT_FUZZ_RUNS:-1000000", "full SMT fuzz command must default to 1e6 runs");

const cascadeRefinement = read("rust/crates/omena-cascade/src/refinement.rs");
assertIncludes(cascadeRefinement, "legacy_proofs_rs_byte_untouched", "cascade refinement must enforce proofs.rs SHA-256 invariant");
assertIncludes(cascadeRefinement, "evaluate_static_supports_condition", "refinement must delegate to L1 supports evaluator");
assertIncludes(cascadeRefinement, "prove_scope_flatten_candidate", "refinement must delegate to L1 scope proof");
assertIncludes(cascadeRefinement, "prove_layer_flatten_candidate", "refinement must delegate to L1 layer proof");

const checker = read("rust/crates/omena-checker/src/lib.rs");
assertIncludes(checker, "CascadeSMTViolation", "checker must register S-tier SMT violation lint");
assertIncludes(checker, "DesignerIntentInconsistency", "checker must register variational lint");
assertIncludes(checker, "StreamingIfdsPrecisionParity", "checker must register streaming IFDS parity lint");
assertIncludes(checker, "pub ordinal: u16", "checker descriptors must expose stable rule ordinals");
assertIncludes(checker, "DesignerIntentInconsistency => 22", "variational lint must keep R3 ordinal 22");
assertIncludes(checker, "CascadeSMTViolation => 23", "SMT lint must keep R4 expansion ordinal 23");
assertIncludes(checker, "StreamingIfdsPrecisionParity => 25", "streaming IFDS lint must keep R4 expansion ordinal 25");
assertIncludes(
  checker,
  "resolve_omena_checker_rule_tier_for_smt_backend",
  "SMT lint must expose backend-aware tier resolution",
);
assertIncludes(checker, "OmenaCheckerSmtBackendKindV0::Stub => OmenaCheckerRuleTierV0::I", "stub SMT backend must resolve as I-tier");
assertIncludes(checker, "OmenaCheckerSmtBackendKindV0::Z3", "real SMT backends must be represented for S-tier resolution");

const zkAudit = read("rust/crates/omena-zk-audit/src/lib.rs");
assertIncludes(zkAudit, "SetupKindV0::Halo2Ipa", "ZK audit default must be Halo2+IPA");
assertIncludes(zkAudit, '"default", "zk-audit", "zk-audit-stark", "zk-audit-binius"', "ZK audit must expose four CI matrix cells");
assertIncludes(zkAudit, "zk_audit_fold_chain_v0", "ZK audit must expose fold-chain evidence");
assertIncludes(zkAudit, 'recursion_overhead: "O(1)"', "ZK audit fold-chain must pin O(1) recursion overhead");
assertIncludes(
  packageJson,
  "check:rust-m4-gamma-zk-audit-matrix",
  "M4-gamma readiness must exercise the four ZK audit matrix cells",
);
assertIncludes(packageJson, "--features zk-audit-stark", "ZK audit matrix must exercise the STARK cell");
assertIncludes(packageJson, "--features zk-audit-binius", "ZK audit matrix must exercise the Binius cell");
const omenaCliManifest = read("rust/crates/omena-cli/Cargo.toml");
const omenaCli = read("rust/crates/omena-cli/src/main.rs");
assertIncludes(omenaCliManifest, 'zk-audit = ["dep:omena-zk-audit"]', "omena-cli must gate ZK audit CLI behind feature");
assertIncludes(omenaCli, "AuditCommand", "omena-cli must expose audit subcommand behind feature");
assertIncludes(omenaCli, "ZkAuditCommand", "omena-cli must expose audit zk command group");
assertIncludes(omenaCli, "Prove", "omena audit zk must expose prove");
assertIncludes(omenaCli, "Verify", "omena audit zk must expose verify");
assertIncludes(omenaCli, "SetupStatus", "omena audit zk must expose setup-status");
assertIncludes(omenaCli, "omena-cli.audit.zk.setup-status", "ZK setup-status must return product evidence");

const variational = read("rust/crates/omena-variational/src/lib.rs");
assertIncludes(variational, "ProvenancePosteriorAnnotationV0", "variational sidecar annotation must exist");
assertIncludes(variational, "DesignerIntentPosteriorModeV0", "variational posterior mode must exist");
const variationalHover = read("rust/crates/omena-variational/src/hover.rs");
assertIncludes(variationalHover, "total_budget_ms: 25", "variational hover total budget must be 25ms");
assertIncludes(variationalHover, "fragment_budget_ms: 6", "variational hover fragment budget must be 6ms");
assertIncludes(variationalHover, "enabled_by_default: false", "variational hover must default disabled");

const streaming = read("rust/crates/omena-streaming-ifds/src/lib.rs");
assertIncludes(streaming, "OmenaUnifiedHypergraphConnectivityOracle", "streaming IFDS must consume M4-beta hypergraph oracle trait");
assertIncludes(streaming, "PolylogDynamicConnectivityBackendV0", "streaming IFDS must expose polylog backend type");
assertIncludes(streaming, "StreamingIFDSAnalysisReportV0", "streaming IFDS must expose a substantive analysis report");
assertIncludes(streaming, "StreamingIFDSTransferFunctionV0", "streaming IFDS must expose transfer functions, not reachability only");
assertIncludes(streaming, "StreamingIFDSSummaryCacheEntryV0", "streaming IFDS must expose a summary cache contract");
assertIncludes(streaming, "run_streaming_ifds_exact_v0", "streaming IFDS must run exact streaming fact propagation");
assertIncludes(streaming, "precision_parity_with_batch", "streaming IFDS must report batch precision parity");
assertIncludes(streaming, "streaming_ifds_frame_rule_bridge_policy_v0", "streaming IFDS must keep the frame-rule bridge feature-gated");
assertIncludes(streaming, "streaming_ifds_refinement_revision_bump_v0", "streaming IFDS must model refinement Salsa revision bump");
assertIncludes(streaming, "delta: 0", "streaming IFDS default delta must be 0");
assertIncludes(streaming, "epsilon: 0", "streaming IFDS default epsilon must be 0");
assertIncludes(
  packageJson,
  "check:rust-m4-gamma-streaming-ifds",
  "M4-gamma readiness must exercise default and frame-rule streaming IFDS cells",
);
assertIncludes(packageJson, "--features with-frame-rule", "streaming IFDS readiness must exercise with-frame-rule");

console.log(
  JSON.stringify(
    {
      product: "rust.m4-gamma.readiness",
      workspaceMembers: workspaceMembers.length,
      omenaCrates: workspaceMembers.filter((member) => member.includes("/omena-")).length,
      gammaCrates: gammaCrates.length,
      legacyProofsSha256: proofsDigest,
      heavyDefaultDependencies: 0,
    },
    null,
    2,
  ),
);

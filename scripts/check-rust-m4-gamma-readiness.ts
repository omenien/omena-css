import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";

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

function rustFilesUnder(directory: string): string[] {
  return readdirSync(directory).flatMap((entry) => {
    const path = `${directory}/${entry}`;
    return statSync(path).isDirectory()
      ? rustFilesUnder(path)
      : entry.endsWith(".rs")
        ? [path]
        : [];
  });
}

function matchingBraceOffset(source: string, openBraceOffset: number): number {
  let depth = 0;
  for (let index = openBraceOffset; index < source.length; index += 1) {
    if (source[index] === "{") {
      depth += 1;
    } else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        return index;
      }
    }
  }
  return -1;
}

function assertGammaV0StructHeaders(cratePaths: readonly string[]): void {
  const missingHeaders: string[] = [];
  const unitStructs: string[] = [];
  let checkedStructs = 0;

  for (const cratePath of cratePaths) {
    for (const filePath of rustFilesUnder(`rust/${cratePath}/src`)) {
      const source = read(filePath);
      const structPattern = /pub\s+struct\s+([A-Za-z0-9_]+V0)(?:<[^>{;]+>)?\s*([{:;])/gu;
      for (const match of source.matchAll(structPattern)) {
        const [, name, delimiter] = match;
        if (!name) {
          continue;
        }
        if (delimiter === ";") {
          unitStructs.push(`${filePath}:${name}`);
          continue;
        }

        const openBraceOffset = (match.index ?? 0) + match[0].length - 1;
        const closeBraceOffset = matchingBraceOffset(source, openBraceOffset);
        assert(closeBraceOffset >= 0, `could not parse V0 struct body for ${filePath}:${name}`);
        const body = source.slice(openBraceOffset, closeBraceOffset + 1);
        checkedStructs += 1;
        if (!/pub\s+schema_version\s*:/.test(body) || !/pub\s+layer_marker\s*:/.test(body)) {
          missingHeaders.push(`${filePath}:${name}`);
        }
      }
    }
  }

  assert(
    unitStructs.length === 0,
    `V0 unit structs cannot carry schema/layer headers: ${unitStructs.join(", ")}`,
  );
  assert(
    missingHeaders.length === 0,
    `V0 structs missing schema_version/layer_marker: ${missingHeaders.join(", ")}`,
  );
  assert(
    checkedStructs >= 80,
    `expected to audit at least 80 M4-gamma V0 structs, got ${checkedStructs}`,
  );
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

// Step 7 (omena-structure-design): workspace-roster integrity is DERIVED from the role
// manifest rather than hardcoded literal counts, so adding a member (e.g. the [U]
// umbrella crate) cannot break a `=== 45`/`=== 42` assert. Every workspace member must
// carry the [package.metadata.omena] role manifest; rust/role-boundaries is the
// authoritative role gate, this is the m4-gamma-side completeness cross-check.
const untaggedMembers = workspaceMembers.filter(
  (member) => !read(`rust/${member}/Cargo.toml`).includes("[package.metadata.omena]"),
);
assert(
  untaggedMembers.length === 0,
  `every workspace member must carry the [package.metadata.omena] role manifest; untagged: ${untaggedMembers.join(", ")}`,
);
for (const cratePath of gammaCrates) {
  assert(workspaceMembers.includes(cratePath), `missing M4-gamma workspace member ${cratePath}`);
  // gammaCrates is the m4-gamma audit SUBSET of the pillar-tagged theoretical-rigor
  // crates (pillar ⊋ gammaCrates — pillar also covers m4-alpha/beta theory crates), so
  // it is NOT derived from pillar; instead cross-check the manifest linkage one way.
  assert(
    read(`rust/${cratePath}/Cargo.toml`).includes('pillar = "theoretical-rigor"'),
    `M4-gamma crate ${cratePath} must be tagged pillar = "theoretical-rigor" in the role manifest`,
  );
}
assertGammaV0StructHeaders(gammaCrates);

const heavyDependencyNames = [
  "ark-ff",
  "ark-groth16",
  "ark-relations",
  "ark-std",
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
    const heavyDependencyLine = new RegExp(
      `^\\s*${escapedDependencyName}\\s*=([^\\n]+)$`,
      "mu",
    ).exec(dependencyBlock)?.[1];
    assert(
      !heavyDependencyLine || heavyDependencyLine.includes("optional = true"),
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
  "cascade_property",
  "prove_layer_flatten_candidate",
  "prove_scope_flatten_candidate",
  "prove_box_shorthand_combination",
  "evaluate_static_supports_condition",
]) {
  assertIncludes(
    categorical,
    primitive,
    "omena-categorical must map existing cascade primitive roles",
  );
}
assertIncludes(
  categorical,
  '"ranking"',
  "omena-categorical must classify cascade_property as a ranking primitive",
);
assertIncludes(
  categorical,
  "cosheaf colimit witness",
  "omena-categorical must map cascade_property to a cosheaf colimit witness",
);
assertIncludes(categorical, "contract_count: 26", "omena-categorical must pin 26 V0 contracts");
assertIncludes(
  categorical,
  "CategoricalCascadeEvidenceV0",
  "omena-categorical must expose cascade evidence",
);
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
  "CascadeFunctorApplicationV0",
  "omena-categorical must expose real functor application evidence",
);
assertIncludes(
  categorical,
  "apply_cascade_primitive_role_functor_v0",
  "omena-categorical must compute primitive-to-role functor evidence",
);
assertIncludes(
  categorical,
  "primitive-role-composition-preservation",
  "omena-categorical must replace invariant functoriality tautology with computed composition evidence",
);
assertIncludes(
  categorical,
  "fixture_evidence",
  "categorical cascade evidence must include endpoint fixture evidence",
);
for (const endpointId of categoricalEndpointIds) {
  assertIncludes(
    categorical,
    endpointId,
    "omena-categorical must expose all 10 cme-check endpoints",
  );
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
  assertIncludes(
    categorical,
    fixtureId,
    "omena-categorical must expose fixture-backed evidence IDs",
  );
}

const queryTypes = read("rust/crates/omena-query/src/types.rs");
assertIncludes(
  queryTypes,
  "pub categorical_evidence:",
  "cascade-at-position response must carry optional categorical evidence through the checker boundary",
);
assertIncludes(
  queryTypes,
  "Option<omena_query_checker_orchestrator::CategoricalCascadeEvidenceV0>",
  "categorical evidence must flow through the query-checker orchestrator boundary",
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
]) {
  assert(existsSync(smtPath), `missing SMT module ${smtPath}`);
}

const smt = read("rust/crates/omena-smt/src/lib.rs");
assertIncludes(
  smt,
  "proof_style_bisimulation_invariant_holds_for_all_l1_primitives",
  "SMT bisimulation invariant test must be present",
);
assertIncludes(
  smt,
  "static_supports_smt_equivalence_tracks_l1_verdict_shape",
  "SMT supports equivalence test must be present",
);
assertIncludes(
  smt,
  "smt_bisimulation_fuzz_seed_corpus_covers_m3_fixture_shapes",
  "SMT must cover the M3 fixture-shaped fuzz seed corpus",
);
assertIncludes(
  smt,
  "z3_backend_solves_canonical_box_shorthand_obligation",
  "SMT must expose a feature-gated Z3 solver round-trip test",
);
assertIncludes(smt, "canonical_smtlib2_script_v0", "SMT must expose SMT-LIB2 script encoding");
assertIncludes(
  smt,
  "smtlib2_script",
  "SMT backend checks must consume encoded SMT-LIB2 input rather than descriptor-only terms",
);
const smtManifest = read("rust/crates/omena-smt/Cargo.toml");
assertIncludes(smtManifest, 'z3 = { version = "0.20.0"', "SMT must link the Z3 crate");
assertIncludes(
  smtManifest,
  'smt-z3 = ["dep:z3", "z3/gh-release"]',
  "SMT Z3 backend must stay opt-in and self-contained",
);
const smtFuzz = read("rust/crates/omena-smt/src/fuzz.rs");
assertIncludes(
  smtFuzz,
  "SmtBisimulationFuzzCaseV0",
  "SMT must expose a bisimulation fuzz case contract",
);
assertIncludes(
  smtFuzz,
  "SmtBisimulationFuzzReportV0",
  "SMT must expose a bisimulation fuzz report contract",
);
assertIncludes(
  smtFuzz,
  "smt_bisimulation_fuzz_case_v0",
  "SMT fuzz cases must have a schema-zero constructor",
);
assertIncludes(
  smtFuzz,
  'product: "omena-smt.bisimulation-fuzz-case"',
  "SMT fuzz case V0 must carry product identity",
);
assertIncludes(
  smtFuzz,
  "schema_version: SMT_SCHEMA_VERSION_V0",
  "SMT fuzz case V0 must carry schema_version",
);
assertIncludes(
  smtFuzz,
  "layer_marker: SMT_LAYER_MARKER_V0",
  "SMT fuzz case V0 must carry layer_marker",
);
assertIncludes(
  smtFuzz,
  "feature_gate: SMT_FEATURE_GATE_V0",
  "SMT fuzz case V0 must carry feature_gate",
);
assertIncludes(
  smtFuzz,
  "m3-cascade-proof-fixtures",
  "SMT fuzz evidence must identify the M3 cascade proof fixture suite",
);
assertIncludes(
  smtFuzz,
  "run_smt_bisimulation_fuzz_seed_corpus_v0",
  "SMT must expose deterministic fuzz seed corpus evidence",
);
const fuzzManifest = read("rust/fuzz/Cargo.toml");
assertIncludes(
  fuzzManifest,
  'name = "smt_bisimulation"',
  "cargo-fuzz must expose the SMT bisimulation target",
);
assertIncludes(
  packageJson,
  "check:rust-m4-gamma-smt-verification",
  "M4-gamma readiness must exercise SMT verification",
);
assertIncludes(
  packageJson,
  "check:rust-m4-gamma-smt-fuzz-full",
  "M4-gamma must expose the full 1e6 SMT fuzz command",
);
assertIncludes(
  packageJson,
  "M4_GAMMA_SMT_FUZZ_RUNS:-1000000",
  "full SMT fuzz command must default to 1e6 runs",
);

const cascadeRefinement = read("rust/crates/omena-cascade/src/refinement.rs");
assertIncludes(
  cascadeRefinement,
  "legacy_proofs_rs_byte_untouched",
  "cascade refinement must enforce proofs.rs SHA-256 invariant",
);
assertIncludes(
  cascadeRefinement,
  "evaluate_static_supports_condition",
  "refinement must delegate to L1 supports evaluator",
);
assertIncludes(
  cascadeRefinement,
  "prove_scope_flatten_candidate",
  "refinement must delegate to L1 scope proof",
);
assertIncludes(
  cascadeRefinement,
  "prove_layer_flatten_candidate",
  "refinement must delegate to L1 layer proof",
);

const checker = read("rust/crates/omena-checker/src/lib.rs");
assertIncludes(checker, "CascadeSMTViolation", "checker must register S-tier SMT violation lint");
assertIncludes(checker, "DesignerIntentInconsistency", "checker must register variational lint");
assertIncludes(
  checker,
  "StreamingIfdsPrecisionParity",
  "checker must register streaming IFDS parity lint",
);
assertIncludes(checker, "pub ordinal: u16", "checker descriptors must expose stable rule ordinals");
assertIncludes(
  checker,
  "DesignerIntentInconsistency => 22",
  "variational lint must keep R3 ordinal 22",
);
assertIncludes(checker, "CascadeSMTViolation => 23", "SMT lint must keep R4 expansion ordinal 23");
assertIncludes(
  checker,
  "StreamingIfdsPrecisionParity => 25",
  "streaming IFDS lint must keep R4 expansion ordinal 25",
);
assertIncludes(
  checker,
  "resolve_omena_checker_rule_tier_for_smt_backend",
  "SMT lint must expose backend-aware tier resolution",
);
assertIncludes(
  checker,
  "OmenaCheckerSmtBackendKindV0::Stub => OmenaCheckerRuleTierV0::I",
  "stub SMT backend must resolve as I-tier",
);
assertIncludes(
  checker,
  "OmenaCheckerSmtBackendKindV0::Z3",
  "real SMT backends must be represented for S-tier resolution",
);

const zkAudit = read("rust/crates/omena-zk-audit/src/lib.rs");
assertIncludes(zkAudit, "SetupKindV0::Halo2Ipa", "ZK audit default must be Halo2+IPA");
assertIncludes(
  zkAudit,
  "ArkworksGroth16RoundTripV0",
  "ZK audit must expose an actual arkworks Groth16 proof round-trip result",
);
assertIncludes(
  zkAudit,
  "prove_and_verify_cascade_smt_payload_with_arkworks_v0",
  "ZK audit must link a real opt-in proof generation and verification path",
);
assertIncludes(
  zkAudit,
  "ZKBackendLinkStatusV0::RealBackendLinked",
  "ZK audit backend policy must distinguish real linked backends from protocol-only cells",
);
assertIncludes(
  zkAudit,
  '"default", "zk-audit", "zk-audit-stark", "zk-audit-binius"',
  "ZK audit must expose four CI matrix cells",
);
const zkAuditManifest = read("rust/crates/omena-zk-audit/Cargo.toml");
assertIncludes(
  zkAuditManifest,
  "ark-groth16",
  "ZK audit must link arkworks Groth16 behind a feature",
);
assertIncludes(
  zkAuditManifest,
  "zk-audit = [",
  "ZK audit real backend dependencies must remain opt-in through the zk-audit feature",
);
assertIncludes(zkAudit, "zk_audit_fold_chain_v0", "ZK audit must expose fold-chain evidence");
assertIncludes(
  zkAudit,
  'recursion_overhead: "O(1)"',
  "ZK audit fold-chain must pin O(1) recursion overhead",
);
assertIncludes(
  packageJson,
  "check:rust-m4-gamma-zk-audit-matrix",
  "M4-gamma readiness must exercise the four ZK audit matrix cells",
);
assertIncludes(
  packageJson,
  "--features zk-audit-stark",
  "ZK audit matrix must exercise the STARK cell",
);
assertIncludes(
  packageJson,
  "--features zk-audit-binius",
  "ZK audit matrix must exercise the Binius cell",
);
const omenaCliManifest = read("rust/crates/omena-cli/Cargo.toml");
const omenaCli = read("rust/crates/omena-cli/src/main.rs");
assertIncludes(
  omenaCliManifest,
  'zk-audit = ["dep:omena-zk-audit"',
  "omena-cli must gate ZK audit CLI behind feature",
);
assertIncludes(omenaCli, "AuditCommand", "omena-cli must expose audit subcommand behind feature");
assertIncludes(omenaCli, "ZkAuditCommand", "omena-cli must expose audit zk command group");
assertIncludes(omenaCli, "Prove", "omena audit zk must expose prove");
assertIncludes(omenaCli, "Verify", "omena audit zk must expose verify");
assertIncludes(omenaCli, "SetupStatus", "omena audit zk must expose setup-status");
assertIncludes(
  omenaCli,
  "omena-cli.audit.zk.setup-status",
  "ZK setup-status must return product evidence",
);

const variational = read("rust/crates/omena-variational/src/lib.rs");
assertIncludes(
  variational,
  "ProvenancePosteriorAnnotationV0",
  "variational sidecar annotation must exist",
);
assertIncludes(
  variational,
  "DesignerIntentPosteriorModeV0",
  "variational posterior mode must exist",
);
assertIncludes(
  variational,
  "PatternPriorKindV0::UniformDirichlet",
  "variational prior must expose uniform Dirichlet mode",
);
assertIncludes(
  variational,
  "dirichlet_alpha",
  "variational prior must carry Dirichlet alpha over intents",
);
assertIncludes(
  variational,
  "axis_a_schema_version",
  "variational calibration must pin Axis A schema version",
);
assertIncludes(
  variational,
  "RgUniversalityClassRefV0",
  "variational prior must carry RG universality-class hook",
);
assertIncludes(
  variational,
  "factor_count",
  "variational emission likelihood must report factor count",
);
assertIncludes(
  variational,
  "log_likelihood_bits",
  "variational likelihood must stay in bits at V0 boundary",
);
assertIncludes(
  variational,
  "ProvenancePosteriorNodeV0",
  "variational provenance sidecar must expose node annotations",
);
assertIncludes(
  variational,
  "mutates_existing_provenance_enum: false",
  "variational sidecar must not mutate existing provenance enum",
);
const variationalHover = read("rust/crates/omena-variational/src/hover.rs");
assertIncludes(
  variationalHover,
  "total_budget_ms: 25",
  "variational hover total budget must be 25ms",
);
assertIncludes(
  variationalHover,
  "fragment_budget_ms: 6",
  "variational hover fragment budget must be 6ms",
);
assertIncludes(
  variationalHover,
  "enabled_by_default: false",
  "variational hover must default disabled",
);

const streaming = read("rust/crates/omena-streaming-ifds/src/lib.rs");
assertIncludes(
  streaming,
  "OmenaUnifiedHypergraphConnectivityOracle",
  "streaming IFDS must consume M4-beta hypergraph oracle trait",
);
assertIncludes(
  streaming,
  "PolylogDynamicConnectivityBackendV0",
  "streaming IFDS must expose polylog backend type",
);
assertIncludes(
  streaming,
  "StreamingIFDSAnalysisReportV0",
  "streaming IFDS must expose a substantive analysis report",
);
assertIncludes(
  streaming,
  "StreamingIFDSTransferFunctionV0",
  "streaming IFDS must expose transfer functions, not reachability only",
);
assertIncludes(
  streaming,
  "StreamingIFDSSummaryCacheEntryV0",
  "streaming IFDS must expose a summary cache contract",
);
assertIncludes(
  streaming,
  "run_streaming_ifds_exact_v0",
  "streaming IFDS must run exact streaming fact propagation",
);
assertIncludes(
  streaming,
  "precision_parity_with_batch",
  "streaming IFDS must report batch precision parity",
);
assertIncludes(
  streaming,
  "streaming_ifds_frame_rule_bridge_policy_v0",
  "streaming IFDS must keep the frame-rule bridge feature-gated",
);
assertIncludes(
  streaming,
  "streaming_ifds_refinement_revision_bump_v0",
  "streaming IFDS must model refinement Salsa revision bump",
);
assertIncludes(streaming, "delta: 0", "streaming IFDS default delta must be 0");
assertIncludes(streaming, "epsilon: 0", "streaming IFDS default epsilon must be 0");
assertIncludes(
  packageJson,
  "check:rust-m4-gamma-streaming-ifds",
  "M4-gamma readiness must exercise default and frame-rule streaming IFDS cells",
);
assertIncludes(
  packageJson,
  "--features with-frame-rule",
  "streaming IFDS readiness must exercise with-frame-rule",
);

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

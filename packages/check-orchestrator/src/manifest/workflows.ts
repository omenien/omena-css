import { resolveScanSurfaceForScanner } from "../evidence/scan-surface-manifest";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { parse as parseYaml } from "yaml";
import { buildCheckPlan } from "./plan";
import { loadGatePolicy } from "./gate-policy";
import { BUNDLE_SHARDS, bundleShardNames } from "./shards";
import type { CheckCiTier, CheckDiagnostic, CheckGate } from "./types";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

const PNPM_SCRIPT_REF = /\bpnpm\s+(?:run\s+)?([A-Za-z0-9:_-]+)/g;
const OMENA_CHECK_TARGET_REF =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(run|bundle)\s+([A-Za-z0-9:_@/.-]+)/g;
const OMENA_CHECK_MATRIX_TARGET_REF = /^\s+target:\s+([A-Za-z0-9:_@/.-]+)\s*$/;
const OMENA_CHECK_MATRIX_TARGET_BINDING =
  /^\s*OMENA_CHECK_TARGET:\s*\$\{\{\s*matrix\.target\s*\}\}\s*$/m;
const OMENA_CHECK_MATRIX_TARGET_INVOCATION =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(run|bundle)\s+["']?\$OMENA_CHECK_TARGET\b/;
const NODE_SCRIPT_REF = /\bnode\b[^|;&\n]*?\.\/(scripts\/[A-Za-z0-9_./-]+\.ts)\b/g;
const WORKFLOW_CI_TIER_ANNOTATION = /^\s*#\s*omena-ci-tier:\s*([A-Za-z0-9_-]+)\s*$/;
const WORKFLOW_REQUIRED_ANNOTATION = /^\s*#\s*omena-ci-required:\s*(true|false)\s*$/;

const CI_REACHABILITY_ESCAPE_HATCH_FALLBACK = Object.freeze({
  maxGateCount: 156,
  owner: "check-orchestrator maintainers",
  reviewBy: "gate-policy.json escapeHatch.reviewAfter",
});

function escapeHatchPolicy(rootDir: string) {
  const policy = loadGatePolicy(rootDir);
  // Defensive against a malformed document (R5): the reachability sweep runs
  // BEFORE the gate-policy shape validator, so a deleted/null escapeHatch must
  // fall back here (the validator then reports the governed
  // gate-policy-invalid-shape error) instead of crashing with a TypeError.
  const hatch = policy?.escapeHatch;
  if (
    !hatch ||
    typeof hatch !== "object" ||
    typeof hatch.maxGateCount !== "number" ||
    typeof hatch.owner !== "string" ||
    typeof hatch.reviewAfter !== "string"
  ) {
    return CI_REACHABILITY_ESCAPE_HATCH_FALLBACK;
  }
  return {
    maxGateCount: hatch.maxGateCount,
    owner: hatch.owner,
    reviewBy: hatch.reviewAfter,
  };
}

const VALID_WORKFLOW_CI_TIERS = new Set<CheckCiTier>([
  "verify",
  "closure-fast",
  "rust-workspace",
  "package",
  "protocol",
  "native",
  "plugin",
  "extension-host",
  "release",
  "scheduled",
  "manual",
  "none",
]);

interface GovernedWorkflowNodeInvocation {
  readonly workflowPath: string;
  readonly scriptPath: string;
  readonly reason: string;
}

interface MatrixOmenaCheckTarget {
  readonly command: "run" | "bundle";
  readonly target: string;
}

const GOVERNED_WORKFLOW_NODE_INVOCATIONS: readonly GovernedWorkflowNodeInvocation[] = [
  {
    workflowPath: ".github/workflows/_publish-crate-train.yml",
    scriptPath: "scripts/check-rust-publish-train-closure.ts",
    reason: "The shell captures the script's JSON stdout for jq-driven publish ordering.",
  },
  {
    workflowPath: ".github/workflows/benchmark-regression.yml",
    scriptPath: "scripts/check-rust-z5-perf-gate-baseline.ts",
    reason: "The scheduled writer updates the benchmark baseline; CI gates consume it read-only.",
  },
];

const GOVERNED_WORKFLOW_NODE_INVOCATIONS_BY_KEY = new Map(
  GOVERNED_WORKFLOW_NODE_INVOCATIONS.map((classification) => [
    `${classification.workflowPath}#${classification.scriptPath}`,
    classification,
  ]),
);
if (
  GOVERNED_WORKFLOW_NODE_INVOCATIONS_BY_KEY.size !== GOVERNED_WORKFLOW_NODE_INVOCATIONS.length ||
  GOVERNED_WORKFLOW_NODE_INVOCATIONS.some((classification) => !classification.reason.trim())
) {
  throw new Error("governed workflow Node invocations must have unique keys and non-empty reasons");
}

type GovernedLeafCriterion =
  | "research-evidence"
  | "compat-alias-with-retirement-window"
  | "manual-tool-with-named-consumer";

interface GovernedLeafClassification {
  readonly id: string;
  readonly reason: string;
  readonly criterion: GovernedLeafCriterion;
}

const GOVERNED_CI_LEAF_CLASSIFICATIONS: readonly GovernedLeafClassification[] = [
  {
    id: "rust/benchmark/bundler-productization",
    reason: "Benchmark/profiling entrypoint; run manually when collecting performance evidence.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/benchmark/z5/macro",
    reason: "Benchmark/profiling entrypoint; run manually when collecting performance evidence.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/benchmark/z5/micro",
    reason: "Benchmark/profiling entrypoint; run manually when collecting performance evidence.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/benchmark/emitted-css-golden-gate:update",
    reason:
      "Golden snapshot regeneration command; the read-only emitted CSS gate is the CI validation surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/benchmark/transform-relex-baseline:update",
    reason:
      "Benchmark snapshot regeneration command; the read-only transform re-lex baseline gate is the CI validation surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/z5-perf-baseline:update",
    reason:
      "Perf baseline regeneration command; the read-only z5 perf baseline and regression gates are the CI validation surfaces.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-transform-target/generated-compat:update",
    reason:
      "Generated compat data regeneration command; the read-only generated-compat gate is the CI validation surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-coverage-gap-report:update",
    reason:
      "Generated coverage-gap report regeneration command; the read-only coverage-gap report gate is the CI validation surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/discharge-ledger:update",
    reason:
      "Generated discharge ledger regeneration command; the read-only discharge-ledger gate is the CI validation surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "release/changeset",
    reason: "Release authoring command; not a CI validation gate.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "tooling/cme-checker-boundary",
    reason:
      "Tooling helper gate retained for local orchestrator maintenance; canonical doctor/inventory gates run from verify CI.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "tooling/cme-checker-testkit-archetypes",
    reason:
      "Tooling helper gate retained for local orchestrator maintenance; canonical doctor/inventory gates run from verify CI.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "contract/parity-v1-golden",
    reason:
      "Contract fixture probe retained for manual compatibility checks outside the CI matrix.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "contract/parity-v1-smoke",
    reason:
      "Contract fixture probe retained for manual compatibility checks outside the CI matrix.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "contract/parity-v2-smoke",
    reason:
      "Contract fixture probe retained for manual compatibility checks outside the CI matrix.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "editor/editor-path-boundary",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "tooling/orchestrator-doctor",
    reason:
      "Tooling helper gate retained for local orchestrator maintenance; canonical doctor/inventory gates run from verify CI.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "tooling/omena-check",
    reason: "Check-orchestrator CLI entrypoint; workflow jobs validate the gates it runs.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "release/check/packaged-engine-shadow-runner",
    reason: "Legacy packaged-runner probe superseded by release/package/prepared in package CI.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "editor/provider-host-routing-boundary",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/checker/release-gate-shadow-review",
    reason:
      "Checker promotion/release probe retained for manual diagnosis; scheduled checker-release-gate carries the release shadow path.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/design-system/universality-class",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/expression-domain/candidates",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-domain/canonical-candidate",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-domain/canonical-producer",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-domain/compare",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-domain/evaluator-candidates",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-domain/fragments",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-domain/reduced-evaluator",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-semantics/candidates",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-semantics/canonical-candidate",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-semantics/canonical-producer",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-semantics/evaluator-candidates",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-semantics/fragments",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-semantics/match-fragments",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/expression-semantics/query-fragments",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/input-producers/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/lsp-runtime-loop",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/m4-alpha-frame-refresh-latency",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-alpha-frame-rule-fuzz",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-alpha-grn-explicit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-alpha-mdl-differential",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-alpha-qtt-semiring",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-alpha-spin-glass-policy",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-axis-a-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-axis-a-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-axis-b-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-axis-b-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-axis-c-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-axis-c-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-axis-d-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-axis-d-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-beta-ensemble",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-beta-hypergraph-monotone-fact-propagation",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-beta-transform-catalog",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-beta-multiscale-complexity-heuristic",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-gamma-categorical-evidence",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-gamma-refinement",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-gamma-smt-fuzz-full",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-gamma-smt-verification",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-gamma-demand-sliced-monotone-fact-propagation",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-gamma-zk-audit-matrix",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m4-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/m8-dynamic-classname-deepening",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-abstract-value/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/omena-bridge/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/omena-categorical/classify-omega-truth",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/compare-design-system-theory",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/summarize-kripke-frame",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/verify-beck-chevalley",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/verify-cascade-section-aggregation-covariance",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/verify-cross-project-symmetry",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/verify-invariant-functoriality",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/verify-modal-imperative-equivalence",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/verify-s4-axioms",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-categorical/verify-cascade-section-aggregation-plan-stability",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-checker/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/omena-incremental/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/omena-lsp-server/style-provider-parity",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-meta-macros-boundary",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-query/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/omena-query/dead-reexport-removal-measurement",
    reason:
      "Isolated published-baseline deletion experiment retained for manual API-removal evidence; the read-only facade and public-surface gates carry CI coverage.",
    criterion: "research-evidence",
  },
  {
    id: "rust/omena-resolver/fixture-suite",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-resolver/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/omena-semantic-observation-harness",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-semantic-split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/omena-spec-audit-boundary",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-conformance-dashboard:update",
    reason:
      "Local generator command retained for reviewed dashboard refreshes; the read-only conformance dashboard gate carries CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-diff-test-wpt-extraction:update",
    reason:
      "Local generator command retained for reviewed WPT extraction refreshes; committed extraction and generator gates carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-diff-test-wpt-extraction-source",
    reason:
      "Pinned-source maintenance gate requires an explicit external WPT checkout; committed extraction and generator gates carry CI coverage.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/omena-tsgo-client/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/parser/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
    criterion: "compat-alias-with-retirement-window",
  },
  {
    id: "rust/phase-2-swap-readiness",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/query-plan/compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/selector-usage/fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/selector-usage/plan-compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/selector-usage/query-fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/shadow/compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/shadow/smoke",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/source-resolution/candidates",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/source-resolution/canonical-candidate",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/source-resolution/canonical-producer",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/source-resolution/evaluator-candidates",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/source-resolution/fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/source-resolution/match-fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/source-resolution/plan-compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/source-resolution/query-fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/split/boundaries",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "rust/type-fact/compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "editor/selected-query-boundary",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "workspace/semantic-smoke",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "ts7/decision-ready",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "ts7/phase-a/decision-ready",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "ts7/phase-a/shadow-review",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "ts7/phase-b/readiness",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "tsgo/operational/shadow-review",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "workspace/check",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "editor/explain/expression",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "core/format",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "core/lint/fix",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "editor/omena",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "release/release/publish",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "test/bench",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "tooling/update/check-inventory",
    reason:
      "Tooling helper gate retained for local orchestrator maintenance; canonical doctor/inventory gates run from verify CI.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "contract/update:contract-parity-v1-golden",
    reason:
      "Contract fixture probe retained for manual compatibility checks outside the CI matrix.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "core/watch",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
    criterion: "manual-tool-with-named-consumer",
  },
  {
    id: "core/prepare",
    reason:
      "Lefthook git-hook installer; runs on local pnpm install only (skipped in CI) and is not a CI gate.",
    criterion: "manual-tool-with-named-consumer",
  },
];

const GOVERNED_CI_LEAF_CLASSIFICATIONS_BY_ID = new Map(
  GOVERNED_CI_LEAF_CLASSIFICATIONS.map((classification) => [classification.id, classification]),
);

export function findWorkflowBypassDiagnostics(
  rootDir: string,
  gates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const workflowsDir = path.join(rootDir, ".github/workflows");
  if (!existsSync(workflowsDir)) return [];

  const gatesByScriptName = new Map(gates.map((gate) => [gate.scriptName, gate]));
  const diagnostics: CheckDiagnostic[] = [];

  for (const fileName of evidenceScanSurface.readdirSync(workflowsDir).toSorted()) {
    if (!fileName.endsWith(".yml") && !fileName.endsWith(".yaml")) continue;

    const workflowPath = path.join(workflowsDir, fileName);
    const relativePath = path.relative(rootDir, workflowPath);
    const lines = readFileSync(workflowPath, "utf8").split(/\r?\n/);

    lines.forEach((line, index) => {
      for (const match of line.matchAll(OMENA_CHECK_TARGET_REF)) {
        const command = match[1];
        const target = match[2];
        if (!command || !target) continue;

        const gate = resolveWorkflowTarget(gates, target);
        if (!gate) {
          diagnostics.push({
            severity: "error",
            code: "workflow-unknown-omena-check-target",
            message: `${relativePath}:${index + 1} references unknown omena-check target "${target}".`,
          });
          continue;
        }

        if (target !== gate.id) {
          diagnostics.push({
            severity: "error",
            code: "workflow-non-canonical-omena-check-target",
            message: `${relativePath}:${index + 1} references omena-check target "${target}"; use canonical gate id "${gate.id}".`,
          });
        }

        if (command === "bundle" && gate.kind !== "bundle" && gate.kind !== "alias") {
          diagnostics.push({
            severity: "error",
            code: "workflow-non-bundle-omena-check-target",
            message: `${relativePath}:${index + 1} uses omena-check bundle for non-bundle target "${target}".`,
          });
        }
      }

      for (const match of line.matchAll(PNPM_SCRIPT_REF)) {
        const scriptName = match[1];
        if (!scriptName) continue;
        if (scriptName === "omena-check") continue;

        const gate = gatesByScriptName.get(scriptName);
        if (!gate) continue;

        const command = gate.kind === "bundle" || gate.kind === "alias" ? "bundle" : "run";
        diagnostics.push({
          severity: "error",
          code: "workflow-direct-script-call",
          message: `${relativePath}:${index + 1} calls "${scriptName}" directly; use "pnpm omena-check ${command} ${gate.id}".`,
        });
      }

      for (const match of line.matchAll(NODE_SCRIPT_REF)) {
        const scriptPath = match[1];
        if (!scriptPath) continue;
        const classification = GOVERNED_WORKFLOW_NODE_INVOCATIONS_BY_KEY.get(
          `${relativePath}#${scriptPath}`,
        );
        if (classification) continue;
        diagnostics.push({
          severity: "error",
          code: "workflow-direct-node-script-call",
          message: `${relativePath}:${index + 1} calls "${scriptPath}" through node directly; register and invoke a canonical omena-check gate.`,
        });
      }
    });

    for (const job of parseWorkflowJobs(lines)) {
      const block = lines.slice(job.start, job.end);
      for (const reference of findMatrixOmenaCheckTargets(block)) {
        const gate = resolveWorkflowTarget(gates, reference.target);
        if (!gate) {
          diagnostics.push({
            severity: "error",
            code: "workflow-unknown-omena-check-target",
            message: `${relativePath} matrix references unknown omena-check target "${reference.target}".`,
          });
          continue;
        }
        if (reference.target !== gate.id) {
          diagnostics.push({
            severity: "error",
            code: "workflow-non-canonical-omena-check-target",
            message: `${relativePath} matrix references omena-check target "${reference.target}"; use canonical gate id "${gate.id}".`,
          });
        }
        if (reference.command === "bundle" && gate.kind !== "bundle" && gate.kind !== "alias") {
          diagnostics.push({
            severity: "error",
            code: "workflow-non-bundle-omena-check-target",
            message: `${relativePath} matrix uses omena-check bundle for non-bundle target "${reference.target}".`,
          });
        }
      }
    }
  }

  return diagnostics;
}

export function findScheduledWorkflowEscalationDiagnostics(
  rootDir: string,
): readonly CheckDiagnostic[] {
  const workflowsDir = path.join(rootDir, ".github/workflows");
  if (!existsSync(workflowsDir)) return [];

  const diagnostics: CheckDiagnostic[] = [];
  for (const fileName of evidenceScanSurface.readdirSync(workflowsDir).toSorted()) {
    if (!fileName.endsWith(".yml") && !fileName.endsWith(".yaml")) continue;

    const workflowPath = path.join(workflowsDir, fileName);
    const relativePath = path.relative(rootDir, workflowPath);
    const workflowText = readFileSync(workflowPath, "utf8");
    if (!/^\s+schedule:\s*$/m.test(workflowText)) continue;

    if (!/^\s*issues:\s*write\s*$/m.test(workflowText)) {
      diagnostics.push({
        severity: "error",
        code: "scheduled-workflow-missing-issue-permission",
        message: `${relativePath} is scheduled but does not grant issues: write for failure escalation.`,
      });
    }

    if (!/if:\s*(?:\$\{\{\s*)?failure\(\)/.test(workflowText)) {
      diagnostics.push({
        severity: "error",
        code: "scheduled-workflow-missing-failure-condition",
        message: `${relativePath} is scheduled but has no failure() escalation condition.`,
      });
    }

    if (!/uses:\s+\.\/\.github\/actions\/escalate-ci-failure/.test(workflowText)) {
      diagnostics.push({
        severity: "error",
        code: "scheduled-workflow-missing-failure-escalation",
        message: `${relativePath} is scheduled but does not use ./.github/actions/escalate-ci-failure.`,
      });
    }
  }

  return diagnostics;
}

// R4 hardening: judge facts come from the PARSED job mapping, not line
// regexes. The YAML parser normalizes key spelling (`"if":`, `if :`) and
// block-scalar run bodies, so the rule governs semantics: does the job carry
// a job-level if of any form, and does a LIVE judge step (no step-level if,
// no continue-on-error) actually execute check-ci-required-results.mjs?
const JUDGE_COMMAND = /^node \.\/scripts\/check-ci-required-results\.mjs$/;

interface ParsedJobFacts {
  readonly hasJobLevelIf: boolean;
  readonly softFail: boolean;
  readonly judge: "live" | "inert" | "missing";
  readonly judgeInertReason: string;
}

function asRecord(value: unknown): Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

const JUDGE_NEEDS_BINDING = /^\$\{\{\s*toJson\(needs\)\s*\}\}$/;

// R5 hardening: liveness is SEMANTIC, not lexical. A judge step is live only
// when (a) the judge invocation is the step's ONLY effective command — a
// block-scalar wrapping it in `set +e` … `exit 0` swallows the exit code;
// (b) the step does not override shell: (which can drop errexit), carry a
// step-level if:, or soft-fail via continue-on-error; and (c) its env binds
// OMENA_CI_REQUIRED_RESULTS to `${{ toJson(needs) }}` — a stubbed or missing
// binding makes the judge judge nothing.
function judgeStepInertReason(
  step: Record<string, unknown>,
  effectiveLines: string[],
  inheritedShellOverride: boolean,
): string {
  if (effectiveLines.length !== 1) {
    return "the judge run body carries additional commands that can swallow the judge's exit code";
  }
  if (Object.hasOwn(step, "shell")) {
    return "the judge step overrides shell:, which can drop fail-on-error semantics";
  }
  if (inheritedShellOverride) {
    // R6 (confirm-lens prescription): GitHub honors defaults.run.shell at
    // workflow AND job scope — a wrapping shell there swallows the judge's
    // exit code exactly like a step-level shell: override would.
    return "a workflow- or job-level defaults.run.shell override covers the judge step, which can drop fail-on-error semantics";
  }
  if (Object.hasOwn(step, "if")) {
    return "the judge step carries a step-level if: and can be skipped";
  }
  if (Object.hasOwn(step, "continue-on-error") && step["continue-on-error"] !== false) {
    return "the judge step carries continue-on-error: and cannot fail the job";
  }
  const binding = asRecord(step["env"])["OMENA_CI_REQUIRED_RESULTS"];
  if (typeof binding !== "string" || !JUDGE_NEEDS_BINDING.test(binding.trim())) {
    return "the judge step does not bind OMENA_CI_REQUIRED_RESULTS to ${{ toJson(needs) }}, so it judges a stub instead of the needs";
  }
  return "";
}

function hasRunShellDefault(scope: Record<string, unknown>): boolean {
  return Object.hasOwn(asRecord(asRecord(scope["defaults"])["run"]), "shell");
}

function readParsedJobFacts(jobValue: unknown, workflowShellDefault = false): ParsedJobFacts {
  const job = asRecord(jobValue);
  const hasJobLevelIf = Object.hasOwn(job, "if");
  const softFail = Object.hasOwn(job, "continue-on-error") && job["continue-on-error"] !== false;
  const inheritedShellOverride = workflowShellDefault || hasRunShellDefault(job);
  let judge: ParsedJobFacts["judge"] = "missing";
  let judgeInertReason = "";
  const steps = Array.isArray(job["steps"]) ? job["steps"] : [];
  for (const stepValue of steps) {
    const step = asRecord(stepValue);
    const run = typeof step["run"] === "string" ? step["run"] : "";
    const effectiveLines = run
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line.length > 0 && !line.startsWith("#"));
    if (!effectiveLines.some((line) => JUDGE_COMMAND.test(line))) continue;
    const inertReason = judgeStepInertReason(step, effectiveLines, inheritedShellOverride);
    if (inertReason) {
      judge = "inert";
      judgeInertReason = inertReason;
      continue;
    }
    return { hasJobLevelIf, softFail, judge: "live", judgeInertReason: "" };
  }
  return { hasJobLevelIf, softFail, judge, judgeInertReason };
}

export function findCiRequiredAggregationDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const workflowPath = path.join(rootDir, ".github/workflows/ci.yml");
  if (!existsSync(workflowPath)) return [];

  const lines = readFileSync(workflowPath, "utf8").split(/\r?\n/);
  const jobs = parseWorkflowJobs(lines);
  const annotations = jobs.map((job) => ({
    job,
    required: parseWorkflowRequiredAnnotation(lines, job),
  }));
  if (annotations.every(({ required }) => required === null)) {
    // this was the ONE genuine fail-open — deleting EVERY
    // `# omena-ci-required:` annotation used to disable the whole contract
    // (single deletions and flips were already loud). A ci.yml with jobs and
    // zero annotations is now an error, not silence.
    if (jobs.length === 0) return [];
    return [
      {
        severity: "error",
        code: "ci-required-model-missing",
        message:
          ".github/workflows/ci.yml declares jobs but carries no `# omena-ci-required:` annotations; " +
          "the required-aggregation contract cannot be derived from nothing.",
      },
    ];
  }

  const diagnostics: CheckDiagnostic[] = [];

  // Hardening review (R3/R4): the strength derivation's skip-cascade premise
  // is broken exactly at job-level `if:` joints. EVERY ci-required
  // needs-ancestor that disables skip-propagation (ANY if form counts) must
  // judge its needs with a LIVE check-ci-required-results.mjs step, and no
  // job on the required path may soft-fail via job-level continue-on-error,
  // which converts a failing conclusion into success upstream of every other
  // rule. Facts come from the parsed document (see readParsedJobFacts).
  let parsedDoc: Record<string, unknown>;
  try {
    parsedDoc = asRecord(parseYaml(lines.join("\n")));
  } catch (error) {
    return [
      {
        severity: "error",
        code: "ci-workflow-yaml-invalid",
        message: `.github/workflows/ci.yml does not parse as YAML: ${
          error instanceof Error ? error.message : String(error)
        }`,
      },
    ];
  }
  const parsedJobs = asRecord(parsedDoc["jobs"]);
  const workflowShellDefault = hasRunShellDefault(parsedDoc);
  const needsByName = new Map(jobs.map((job) => [job.name, parseWorkflowJobNeeds(lines, job)]));
  const ancestors = new Set<string>();
  const queue = [...(needsByName.get("ci-required") ?? [])];
  while (queue.length > 0) {
    const name = queue.pop();
    if (!name || ancestors.has(name)) continue;
    ancestors.add(name);
    queue.push(...(needsByName.get(name) ?? []));
  }
  for (const name of new Set([...ancestors, "ci-required"])) {
    if (!Object.hasOwn(parsedJobs, name)) continue;
    const facts = readParsedJobFacts(parsedJobs[name], workflowShellDefault);
    if (facts.softFail) {
      diagnostics.push({
        severity: "error",
        code: "ci-required-soft-fail",
        message: `.github/workflows/ci.yml job "${name}" is on the ci-required path but carries job-level continue-on-error:, which converts a failing conclusion into success.`,
      });
    }
    // The root aggregator has its own mandatory-judge and mandatory-always
    // rules below; only the soft-fail arm applies to it here.
    if (name === "ci-required") continue;
    if (facts.hasJobLevelIf && facts.judge === "missing") {
      diagnostics.push({
        severity: "error",
        code: "ci-aggregator-judge-missing",
        message: `.github/workflows/ci.yml job "${name}" reaches ci-required and carries a job-level if: (any form disables skip-propagation) without judging its needs via check-ci-required-results.mjs; a failed need could silently become success.`,
      });
    }
    if (facts.hasJobLevelIf && facts.judge === "inert") {
      diagnostics.push({
        severity: "error",
        code: "ci-aggregator-judge-inert",
        message: `.github/workflows/ci.yml job "${name}" reaches ci-required and its judge step is inert (${facts.judgeInertReason}); a failed need could silently become success.`,
      });
    }
    if (facts.judge === "live" && !facts.hasJobLevelIf) {
      diagnostics.push({
        severity: "error",
        code: "ci-aggregator-missing-always",
        message: `.github/workflows/ci.yml job "${name}" judges its needs but lacks if: always(); the judge is skipped exactly when it matters.`,
      });
    }
  }

  for (const { job, required } of annotations) {
    if (required !== null) continue;
    diagnostics.push({
      severity: "error",
      code: "ci-required-job-unclassified",
      message: `.github/workflows/ci.yml job "${job.name}" must declare "# omena-ci-required: true" or "# omena-ci-required: false".`,
    });
  }

  const aggregator = jobs.find((job) => job.name === "ci-required");
  if (!aggregator) {
    diagnostics.push({
      severity: "error",
      code: "ci-required-aggregator-missing",
      message: '.github/workflows/ci.yml must define the "ci-required" aggregate job.',
    });
    return diagnostics;
  }

  const expected = annotations
    .filter(({ job, required }) => job.name !== "ci-required" && required === true)
    .map(({ job }) => job.name)
    .toSorted();
  const actual = parseWorkflowJobNeeds(lines, aggregator).toSorted();
  const missing = expected.filter((jobName) => !actual.includes(jobName));
  const extra = actual.filter((jobName) => !expected.includes(jobName));
  const duplicateNeeds = actual.filter(
    (jobName, index) => index > 0 && jobName === actual[index - 1],
  );

  if (missing.length > 0 || extra.length > 0) {
    diagnostics.push({
      severity: "error",
      code: "ci-required-needs-drift",
      message: `ci-required.needs must match the required job contract; missing=[${missing.join(", ")}], extra=[${extra.join(", ")}].`,
    });
  }
  if (duplicateNeeds.length > 0) {
    diagnostics.push({
      severity: "error",
      code: "ci-required-needs-duplicate",
      message: `ci-required.needs must not repeat job ids; duplicate=[${[
        ...new Set(duplicateNeeds),
      ].join(", ")}].`,
    });
  }

  // Root aggregator duties, judged on the PARSED job (R4: the root was the
  // last judge still governed by a substring test — a comment naming the
  // script satisfied it).
  const rootJob = asRecord(parsedJobs["ci-required"]);
  const rootIf = typeof rootJob["if"] === "string" ? rootJob["if"].trim() : "";
  if (!/^\$\{\{\s*always\(\)\s*\}\}$/.test(rootIf)) {
    diagnostics.push({
      severity: "error",
      code: "ci-required-missing-always",
      message:
        'The "ci-required" aggregate job must use "if: ${{ always() }}" so failed or cancelled dependencies are evaluated.',
    });
  }
  const rootFacts = readParsedJobFacts(parsedJobs["ci-required"], workflowShellDefault);
  if (rootFacts.judge === "missing") {
    diagnostics.push({
      severity: "error",
      code: "ci-required-result-check-missing",
      message:
        'The "ci-required" aggregate job must execute scripts/check-ci-required-results.mjs as a ' +
        "live run step (a comment or echo naming the script judges nothing).",
    });
  } else if (rootFacts.judge === "inert") {
    diagnostics.push({
      severity: "error",
      code: "ci-aggregator-judge-inert",
      message: `.github/workflows/ci.yml job "ci-required" is the root aggregator and its judge step is inert (${rootFacts.judgeInertReason}); a failed need could silently become success.`,
    });
  }

  return diagnostics;
}

// g131 stage-5 R2 (lens-A HIGH): every bundle that carries a committed shard
// table must have ci.yml consume EXACTLY that shard set — adding a named
// shard to the table without wiring the matrix silently removes its members
// from CI (reproduced by the lens with 9/13 gates dropped and every other
// surface green). Two sanctioned consumption forms:
//   (a) inline matrix + `--shard=${{ matrix.<key> }}` — the matrix values
//       must equal bundleShardNames(bundle);
//   (b) generated matrix + `--shard="$ENV"` (closure-fast) — the matrix is
//       produced by `omena-check shards --json` from the SAME table, so the
//       equality is structural; the aggregation-complete gate pins that form.

// R4-confirm lens (third round of one species): the fromJSON reference must
// (1) sit on a value line INSIDE this job's strategy.matrix section — not a
// comment, not an env mapping, not any other block; and (2) name an output
// whose PRODUCING job derives it from THIS bundle's shard table via
// `omena-check shards <bundleId> --json`. Position and provenance, both.
function jobConsumesGeneratedMatrix(
  blockLines: readonly string[],
  allLines: readonly string[],
  jobs: readonly { readonly name: string; readonly start: number; readonly end: number }[],
  bundleId: string,
): boolean {
  const valueRef =
    /^(\s+)[A-Za-z0-9_-]+:\s*\$\{\{\s*fromJSON\(needs\.([A-Za-z0-9_-]+)\.outputs\.([A-Za-z0-9_-]+)\)\s*\}\}\s*$/u;
  let inStrategy = false;
  let inMatrix = false;
  let strategyIndent = -1;
  let matrixIndent = -1;
  for (const line of blockLines) {
    if (/^\s*#/u.test(line) || /^\s*$/u.test(line)) continue;
    const indent = line.match(/^\s*/u)?.[0]?.length ?? 0;
    if (/^\s*strategy:\s*$/u.test(line)) {
      inStrategy = true;
      strategyIndent = indent;
      inMatrix = false;
      continue;
    }
    if (inStrategy && indent <= strategyIndent) {
      inStrategy = false;
      inMatrix = false;
    }
    if (inStrategy && /^\s*matrix:\s*$/u.test(line)) {
      inMatrix = true;
      matrixIndent = indent;
      continue;
    }
    if (inMatrix && indent <= matrixIndent) inMatrix = false;
    if (!inMatrix) continue;
    const match = valueRef.exec(line);
    if (!match) continue;
    const producerName = match[2] ?? "";
    const outputName = match[3] ?? "";
    const producer = jobs.find((candidate) => candidate.name === producerName);
    if (!producer) continue;
    const producerBlock = allLines
      .slice(producer.start, producer.end)
      .filter((producerLine) => !/^\s*#/u.test(producerLine));
    // STEP-ID JOIN (R5-confirm closure rider): the output must be mapped
    // from the SAME step whose EXECUTING run line derives THIS bundle's
    // table — independent existence checks let a two-bundle producer or an
    // unrelated-step mapping bind the wrong table, and a substring test let
    // a comment/name mention satisfy provenance.
    const outputEscaped = outputName.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    let producingStepId: string | null = null;
    for (const producerLine of producerBlock) {
      const mapping = new RegExp(
        `^\\s+${outputEscaped}:\\s*\\$\\{\\{\\s*steps\\.([A-Za-z0-9_-]+)\\.outputs\\.`,
        "u",
      ).exec(producerLine);
      if (mapping) {
        producingStepId = mapping[1] ?? null;
        break;
      }
    }
    if (!producingStepId) continue;
    const bundleEscaped = bundleId.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const shardsRun = new RegExp(
      `^\\s*(?:- )?run: .*omena-check shards ${bundleEscaped} --json`,
      "u",
    );
    let currentStepId: string | null = null;
    let stepDerives = false;
    for (const producerLine of producerBlock) {
      if (/^\s*- /u.test(producerLine)) currentStepId = null;
      const idLine = /^\s*(?:- )?id:\s*([A-Za-z0-9_-]+)\s*$/u.exec(producerLine);
      if (idLine) currentStepId = idLine[1] ?? null;
      // Strip a trailing comment before testing — `run: echo x # omena-check
      // shards ...` is a mention, not an execution.
      const executable = producerLine.split(" #")[0] ?? producerLine;
      if (currentStepId === producingStepId && shardsRun.test(executable)) {
        stepDerives = true;
        break;
      }
    }
    if (stepDerives) return true;
  }
  return false;
}

export function findBundleShardMatrixDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const workflowPath = path.join(rootDir, ".github/workflows/ci.yml");
  if (!existsSync(workflowPath)) return [];
  const lines = readFileSync(workflowPath, "utf8").split(/\r?\n/);
  const jobs = parseWorkflowJobs(lines);
  const diagnostics: CheckDiagnostic[] = [];
  for (const bundleId of Object.keys(BUNDLE_SHARDS)) {
    const expected = [...bundleShardNames(bundleId)].toSorted();
    const escaped = bundleId.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const inlineRef = new RegExp(
      `omena-check (?:run|bundle) ${escaped} --summary --shard=\\$\\{\\{ matrix\\.([A-Za-z0-9_-]+) \\}\\}`,
      "u",
    );
    const envRef = new RegExp(`omena-check (?:run|bundle) ${escaped} --summary --shard="\\$`, "u");
    // The env form is sanctioned ONLY for a generated matrix: the job must
    // consume a preflight-produced matrix via fromJSON(needs.*.outputs.*),
    // which is itself derived from this same shard table (R2-confirm lens:
    // an env-form invocation with a hand-shrunk inline matrix silently
    // dropped a whole shard — the env branch must not blanket-satisfy).

    let consumed = false;
    for (const job of jobs) {
      const blockLines = lines.slice(job.start, job.end);
      const block = blockLines.join("\n");
      const inline = inlineRef.exec(block);
      if (inline) {
        consumed = true;
        const key = inline[1] ?? "";
        const keyEscaped = key.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
        const matrixLine = blockLines.find((line) =>
          new RegExp(`^\\s+${keyEscaped}:\\s*\\[[^\\]]+\\]\\s*$`, "u").test(line),
        );
        const values = matrixLine
          ? (/\[([^\]]+)\]/u.exec(matrixLine)?.[1] ?? "")
              .split(",")
              .map((value) => value.trim().replace(/^["']|["']$/gu, ""))
              .filter(Boolean)
              .toSorted()
          : [];
        if (JSON.stringify(values) !== JSON.stringify(expected)) {
          diagnostics.push({
            severity: "error",
            code: "bundle-shard-matrix-drift",
            message:
              `ci.yml job "${job.name}" consumes bundle "${bundleId}" with matrix shards ` +
              `[${values.join(", ")}] but the committed shard table declares [${expected.join(", ")}]; ` +
              "a table shard the matrix never runs silently removes its members from CI.",
          });
        }
        continue;
      }
      if (envRef.test(block)) {
        if (jobConsumesGeneratedMatrix(blockLines, lines, jobs, bundleId)) {
          consumed = true;
        } else {
          diagnostics.push({
            severity: "error",
            code: "bundle-shard-matrix-drift",
            message:
              `ci.yml job "${job.name}" consumes bundle "${bundleId}" through an env-bound ` +
              "--shard without a strategy.matrix value generated from THIS bundle's shard " +
              "table (fromJSON of an output produced by `omena-check shards <bundleId> --json`); " +
              "the env form is sanctioned only for that generated-matrix shape.",
          });
        }
      }
    }
    if (!consumed) {
      diagnostics.push({
        severity: "error",
        code: "bundle-shard-matrix-drift",
        message:
          `bundle "${bundleId}" carries a committed shard table but no ci.yml job consumes it ` +
          "with the sanctioned `--summary --shard` invocation; every table shard must be wired " +
          "into a matrix (an off-form invocation counts as unconsumed — fix the invocation shape).",
      });
    }
  }
  return diagnostics;
}

export function findCiTierReachabilityDiagnostics(
  rootDir: string,
  gates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const reachableByTier = buildReachableGateIdsByTier(rootDir, gates, diagnostics);
  const reachableTiersByGate = buildReachableTiersByGate(reachableByTier);
  const escapeHatchReachableGateIds = buildEscapeHatchReachableGateIds(gates);
  const escapeHatchGateIds: string[] = [];

  // Criterion coherence runs over the WHOLE classification map, independent of
  // whether the gate also declares a ciTier — a declared tier must not exempt
  // an entry from its criterion's structural shape (hardening review).
  const gateById = new Map(gates.map((gate) => [gate.id, gate]));
  for (const classification of GOVERNED_CI_LEAF_CLASSIFICATIONS) {
    const gate = gateById.get(classification.id);
    if (!gate) continue;
    if (
      classification.criterion === "compat-alias-with-retirement-window" &&
      gate.kind !== "alias" &&
      !gate.deprecatedBy &&
      !gate.deprecatedAliases?.length &&
      !gate.tags?.includes("compat-split-boundary")
    ) {
      diagnostics.push({
        severity: "error",
        code: "governed-leaf-criterion-mismatch",
        message: `Governed leaf "${classification.id}" carries criterion compat-alias-with-retirement-window but is not an alias-shaped gate (kind=${gate.kind}, no deprecation linkage, no compat-split-boundary tag).`,
      });
    }
  }

  for (const gate of gates) {
    const reachableTierCount = reachableTiersByGate.get(gate.id)?.size ?? 0;
    if (!gate.ciTier) {
      if (gate.origin === "declared" || gate.origin === "package+declared") {
        diagnostics.push({
          severity: "error",
          code: "declared-gate-missing-ci-tier",
          message: `Declared gate "${gate.id}" must set ciTier explicitly.`,
        });
        continue;
      }

      if (reachableTierCount > 0) {
        continue;
      }

      if (escapeHatchReachableGateIds.has(gate.id)) {
        escapeHatchGateIds.push(gate.id);
        continue;
      }

      const classification = GOVERNED_CI_LEAF_CLASSIFICATIONS_BY_ID.get(gate.id);
      if (!classification) {
        diagnostics.push({
          severity: "error",
          code: "ci-tier-unclassified",
          message: `Package gate "${gate.id}" is not reachable from any workflow tier and has no governed leaf classification.`,
        });
        continue;
      }

      escapeHatchGateIds.push(gate.id);
      continue;
    }

    if (gate.ciTier === "none" && !gate.tags?.includes("ci-unreachable-allowed")) {
      diagnostics.push({
        severity: "error",
        code: "ci-tier-none-not-allowed",
        message: `Declared gate "${gate.id}" uses ciTier "none" without the ci-unreachable-allowed tag.`,
      });
    }

    if ((gate.ciTier === "none" || gate.ciTier === "manual") && !gate.ciReason?.trim()) {
      diagnostics.push({
        severity: "error",
        code: "ci-tier-escape-hatch-missing-reason",
        message: `Gate "${gate.id}" uses ciTier "${gate.ciTier}" without ciReason.`,
      });
    }

    if (gate.ciTier === "none" || gate.ciTier === "manual") {
      escapeHatchGateIds.push(gate.id);
      continue;
    }

    const reachableIds = reachableByTier.get(gate.ciTier) ?? new Set<string>();
    if (!reachableIds.has(gate.id)) {
      diagnostics.push({
        severity: "error",
        code: "ci-tier-unreachable",
        message: `Gate "${gate.id}" declares ciTier "${gate.ciTier}" but is not reachable from that workflow tier.`,
      });
    }
  }

  const hatch = escapeHatchPolicy(rootDir);
  if (escapeHatchGateIds.length > hatch.maxGateCount) {
    diagnostics.push({
      severity: "error",
      code: "ci-tier-escape-hatch-budget-exceeded",
      message:
        `CI reachability escape-hatch population ${escapeHatchGateIds.length} exceeds the governed maximum ` +
        `${hatch.maxGateCount}; owner=${hatch.owner}, ` +
        `reviewBy=${hatch.reviewBy}.`,
    });
  } else if (escapeHatchGateIds.length > 0) {
    const criterionCounts = new Map<string, number>();
    for (const gateId of escapeHatchGateIds) {
      const criterion =
        GOVERNED_CI_LEAF_CLASSIFICATIONS_BY_ID.get(gateId)?.criterion ?? "declared-none-or-manual";
      criterionCounts.set(criterion, (criterionCounts.get(criterion) ?? 0) + 1);
    }
    const policy = loadGatePolicy(rootDir);
    const pinnedCriteria = policy?.governedLeafCriteria;
    if (policy && !pinnedCriteria) {
      // Fail-closed (R4): deleting the key must not silently retire the
      // whole criterion governance — the exact fail-open species
      // ci-required-model-missing closed for annotations.
      diagnostics.push({
        severity: "error",
        code: "gate-policy-criteria-missing",
        message:
          "gate-policy.json exists but pins no governedLeafCriteria; deleting the key must not silently retire criterion governance.",
      });
    }
    if (pinnedCriteria) {
      // Union of pinned and live criteria (R4): a pinned criterion with zero
      // live members (ghost pin) must be as loud as an unpinned live one.
      const criteria = new Set([...criterionCounts.keys(), ...Object.keys(pinnedCriteria)]);
      for (const criterion of [...criteria].toSorted()) {
        const live = criterionCounts.get(criterion) ?? 0;
        const pinned = pinnedCriteria[criterion] ?? 0;
        if (live !== pinned) {
          diagnostics.push({
            severity: "error",
            code: "gate-policy-criterion-drift",
            message:
              `escape-hatch criterion "${criterion}" counts ${live} live but gate-policy.json pins ` +
              `${pinned}; relabels must land as a reviewed policy diff.`,
          });
        }
      }
    }
    // Counts alone are blind to count-preserving compensating relabels (R4);
    // the digest pins the (gateId, criterion) PAIRS themselves.
    const liveDigest = createHash("sha256")
      .update(
        [...escapeHatchGateIds]
          .toSorted()
          .map(
            (gateId) =>
              `${gateId}=${
                GOVERNED_CI_LEAF_CLASSIFICATIONS_BY_ID.get(gateId)?.criterion ??
                "declared-none-or-manual"
              }`,
          )
          .join("\n"),
      )
      .digest("hex");
    const pinnedDigest = policy?.governedLeafCriteriaDigest;
    if (policy && typeof pinnedDigest !== "string") {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-criteria-missing",
        message: `gate-policy.json exists but pins no governedLeafCriteriaDigest; pin the live digest ${liveDigest}.`,
      });
    } else if (policy && pinnedDigest !== liveDigest) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-criterion-digest-drift",
        message:
          `escape-hatch (gateId, criterion) pairs digest to ${liveDigest} but gate-policy.json pins ` +
          `${pinnedDigest}; content relabels must land as a reviewed policy diff (re-pin governedLeafCriteriaDigest deliberately).`,
      });
    }
    const criterionSummary = [...criterionCounts.entries()]
      .toSorted(([left], [right]) => left.localeCompare(right))
      .map(([criterion, count]) => `${criterion}=${count}`)
      .join(", ");
    if (escapeHatchGateIds.length < hatch.maxGateCount) {
      diagnostics.push({
        severity: "warning",
        code: "ci-tier-escape-hatch-slack",
        message:
          `CI reachability escape-hatch population ${escapeHatchGateIds.length} is below the cap ` +
          `${hatch.maxGateCount}; ratchet the cap down in gate-policy.json (decrease-only discipline).`,
      });
    }
    diagnostics.push({
      severity: "warning",
      code: "ci-tier-escape-hatch-summary",
      message:
        `CI reachability escape-hatch population: ${escapeHatchGateIds.length}/` +
        `${hatch.maxGateCount} gate(s); ` +
        `criteria: ${criterionSummary}; ` +
        `owner=${hatch.owner}, ` +
        `reviewBy=${hatch.reviewBy}.`,
    });
  }

  return diagnostics;
}

function resolveWorkflowTarget(gates: readonly CheckGate[], target: string): CheckGate | null {
  return (
    gates.find((gate) => gate.id === target || gate.scriptName === target) ??
    gates.find((gate) => gate.deprecatedAliases?.includes(target)) ??
    gates.find((gate) => gate.id.endsWith(`/${target}`)) ??
    null
  );
}

function buildReachableGateIdsByTier(
  rootDir: string,
  gates: readonly CheckGate[],
  diagnostics: CheckDiagnostic[],
): Map<CheckCiTier, Set<string>> {
  const workflowsDir = path.join(rootDir, ".github/workflows");
  const reachable = new Map<CheckCiTier, Set<string>>();
  if (!existsSync(workflowsDir)) return reachable;

  for (const fileName of evidenceScanSurface.readdirSync(workflowsDir).toSorted()) {
    if (!fileName.endsWith(".yml") && !fileName.endsWith(".yaml")) continue;

    const workflowPath = path.join(workflowsDir, fileName);
    const lines = readFileSync(workflowPath, "utf8").split(/\r?\n/);
    const workflowText = lines.join("\n");
    for (const job of parseWorkflowJobs(lines)) {
      const tier = inferWorkflowJobTier(fileName, workflowText, lines, job, diagnostics);
      if (!tier) continue;

      const ids = reachable.get(tier) ?? new Set<string>();
      const block = lines.slice(job.start, job.end);
      for (const line of block) {
        for (const match of line.matchAll(OMENA_CHECK_TARGET_REF)) {
          const target = match[2];
          if (!target) continue;
          const gate = resolveWorkflowTarget(gates, target);
          if (!gate) continue;
          for (const step of buildCheckPlan({ gates }, gate).steps) {
            ids.add(step.id);
          }
        }
      }
      for (const reference of findMatrixOmenaCheckTargets(block)) {
        const gate = resolveWorkflowTarget(gates, reference.target);
        if (!gate) continue;
        for (const step of buildCheckPlan({ gates }, gate).steps) {
          ids.add(step.id);
        }
      }
      reachable.set(tier, ids);
    }
  }

  for (const [tier, ids] of reachable) {
    let changed = true;
    while (changed) {
      changed = false;
      for (const gate of gates) {
        if (
          gate.ciTier !== tier ||
          !gate.referencedTargets?.length ||
          ids.has(gate.id) ||
          !gate.referencedTargets.every((target) =>
            ids.has(resolveWorkflowTarget(gates, target)?.id ?? ""),
          )
        ) {
          continue;
        }
        ids.add(gate.id);
        changed = true;
      }
    }
  }

  return reachable;
}

function findMatrixOmenaCheckTargets(lines: readonly string[]): readonly MatrixOmenaCheckTarget[] {
  const block = lines.join("\n");
  const invocation = OMENA_CHECK_MATRIX_TARGET_INVOCATION.exec(block);
  if (!OMENA_CHECK_MATRIX_TARGET_BINDING.test(block) || !invocation?.[1]) {
    return [];
  }
  const command = invocation[1] as "run" | "bundle";

  return lines.flatMap((line) => {
    const match = OMENA_CHECK_MATRIX_TARGET_REF.exec(line);
    return match?.[1] ? [{ command, target: match[1] }] : [];
  });
}

function buildReachableTiersByGate(
  reachableByTier: ReadonlyMap<CheckCiTier, ReadonlySet<string>>,
): Map<string, Set<CheckCiTier>> {
  const reachableTiersByGate = new Map<string, Set<CheckCiTier>>();
  for (const [tier, gateIds] of reachableByTier) {
    for (const gateId of gateIds) {
      const tiers = reachableTiersByGate.get(gateId) ?? new Set<CheckCiTier>();
      tiers.add(tier);
      reachableTiersByGate.set(gateId, tiers);
    }
  }
  return reachableTiersByGate;
}

function buildEscapeHatchReachableGateIds(gates: readonly CheckGate[]): Set<string> {
  const ids = new Set<string>();
  for (const gate of gates) {
    if (gate.ciTier !== "manual" && gate.ciTier !== "none") continue;
    for (const step of buildCheckPlan({ gates }, gate).steps) {
      ids.add(step.id);
    }
  }
  return ids;
}

interface WorkflowJobBlock {
  readonly name: string;
  readonly start: number;
  readonly end: number;
}

function parseWorkflowJobs(lines: readonly string[]): readonly WorkflowJobBlock[] {
  const jobsHeaderIndex = lines.findIndex((line) => /^jobs:\s*$/.test(line));
  if (jobsHeaderIndex < 0) return [];

  const jobs: Array<{ name: string; start: number; end: number }> = [];
  for (let index = jobsHeaderIndex + 1; index < lines.length; index += 1) {
    const line = lines[index] ?? "";
    if (/^\S/.test(line) && line.trim() !== "") break;

    const header = line.match(/^ {2}([A-Za-z0-9_-]+):\s*$/);
    const jobName = header?.[1];
    if (!jobName) continue;

    if (jobs.length > 0) {
      const previousJob = jobs.at(-1);
      if (previousJob) {
        previousJob.end = index;
      }
    }
    jobs.push({ name: jobName, start: index, end: lines.length });
  }
  return jobs;
}

function parseWorkflowRequiredAnnotation(
  lines: readonly string[],
  job: WorkflowJobBlock,
): boolean | null {
  for (const line of lines.slice(job.start + 1, job.end)) {
    const value = line.match(WORKFLOW_REQUIRED_ANNOTATION)?.[1];
    if (value === "true") return true;
    if (value === "false") return false;
  }
  return null;
}

function parseWorkflowJobNeeds(lines: readonly string[], job: WorkflowJobBlock): readonly string[] {
  const block = lines.slice(job.start + 1, job.end);
  const needsIndex = block.findIndex((line) => /^ {4}needs:\s*/.test(line));
  if (needsIndex < 0) return [];

  const needsLine = block[needsIndex] ?? "";
  const scalar = needsLine.match(/^ {4}needs:\s*([A-Za-z0-9_-]+)\s*$/)?.[1];
  if (scalar) return [scalar];
  const inline = needsLine.match(/^ {4}needs:\s*\[([^\]]*)\]\s*$/)?.[1];
  if (inline !== undefined) {
    return inline
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);
  }

  const needs: string[] = [];
  for (const line of block.slice(needsIndex + 1)) {
    const item = line.match(/^ {6}-\s*([A-Za-z0-9_-]+)\s*$/)?.[1];
    if (item) {
      needs.push(item);
      continue;
    }
    if (/^ {0,4}\S/.test(line)) break;
  }
  return needs;
}

function inferWorkflowJobTier(
  fileName: string,
  workflowText: string,
  lines: readonly string[],
  job: WorkflowJobBlock,
  diagnostics: CheckDiagnostic[],
): CheckCiTier | null {
  const annotatedTier = parseWorkflowJobTierAnnotation(fileName, lines, job, diagnostics);
  if (annotatedTier) return annotatedTier;
  if (/^\s+schedule:\s*$/m.test(workflowText)) return "scheduled";
  return null;
}

function parseWorkflowJobTierAnnotation(
  fileName: string,
  lines: readonly string[],
  job: WorkflowJobBlock,
  diagnostics: CheckDiagnostic[],
): CheckCiTier | null {
  const block = lines.slice(job.start + 1, job.end);
  for (const line of block) {
    const match = line.match(WORKFLOW_CI_TIER_ANNOTATION);
    const tier = match?.[1];
    if (!tier) continue;

    if (!VALID_WORKFLOW_CI_TIERS.has(tier as CheckCiTier)) {
      diagnostics.push({
        severity: "error",
        code: "workflow-unknown-ci-tier",
        message: `${fileName} job "${job.name}" declares unknown omena-ci-tier "${tier}".`,
      });
      return null;
    }
    return tier as CheckCiTier;
  }
  return null;
}

// --- Lifecycle façade (derived axes): a read-only view of workflow jobs, their
// needs edges, their plan-expanded gate ids, and each workflow's trigger
// facts. lifecycle.ts derives cadence×strength from this view; keeping the
// walker here reuses the private parsers without widening their surface.

export interface WorkflowTriggerFacts {
  readonly fileName: string;
  readonly hasBranchPush: boolean;
  readonly hasTagPush: boolean;
  readonly crons: readonly string[];
  readonly hasDispatch: boolean;
  readonly hasWorkflowCall: boolean;
  readonly reusableWorkflowUses: readonly string[];
}

export interface WorkflowJobView {
  readonly workflowFile: string;
  readonly jobName: string;
  readonly needs: readonly string[];
  readonly gateIds: ReadonlySet<string>;
}

export interface WorkflowLifecycleView {
  readonly jobs: readonly WorkflowJobView[];
  readonly triggers: readonly WorkflowTriggerFacts[];
}

export function collectWorkflowLifecycleView(
  rootDir: string,
  gates: readonly CheckGate[],
): WorkflowLifecycleView {
  const workflowsDir = path.join(rootDir, ".github/workflows");
  const jobs: WorkflowJobView[] = [];
  const triggers: WorkflowTriggerFacts[] = [];
  if (!existsSync(workflowsDir)) return { jobs, triggers };

  for (const fileName of evidenceScanSurface.readdirSync(workflowsDir).toSorted()) {
    if (!fileName.endsWith(".yml") && !fileName.endsWith(".yaml")) continue;
    const lines = readFileSync(path.join(workflowsDir, fileName), "utf8").split(/\r?\n/);
    const text = lines.join("\n");
    triggers.push(parseWorkflowTriggerFacts(fileName, text));

    for (const job of parseWorkflowJobs(lines)) {
      const block = lines.slice(job.start, job.end);
      const gateIds = new Set<string>();
      for (const line of block) {
        for (const match of line.matchAll(OMENA_CHECK_TARGET_REF)) {
          const target = match[2];
          if (!target) continue;
          const gate = resolveWorkflowTarget(gates, target);
          if (!gate) continue;
          for (const step of buildCheckPlan({ gates }, gate).steps) gateIds.add(step.id);
        }
      }
      for (const reference of findMatrixOmenaCheckTargets(block)) {
        const gate = resolveWorkflowTarget(gates, reference.target);
        if (!gate) continue;
        for (const step of buildCheckPlan({ gates }, gate).steps) gateIds.add(step.id);
      }
      jobs.push({
        workflowFile: fileName,
        jobName: job.name,
        needs: parseWorkflowJobNeeds(lines, job),
        gateIds,
      });
    }
  }
  return { jobs, triggers };
}

function parseWorkflowTriggerFacts(fileName: string, text: string): WorkflowTriggerFacts {
  const onBlock = /^on:\s*$([\s\S]*?)(?=^\S)/m.exec(text)?.[1] ?? "";
  const pushBlock = /^ {2}push:\s*$([\s\S]*?)(?=^ {2}\S|$(?![\s\S]))/m.exec(onBlock)?.[1] ?? "";
  const hasPushKey = /^ {2}push:/m.test(onBlock);
  return {
    fileName,
    hasBranchPush: hasPushKey && (!pushBlock.includes("tags:") || pushBlock.includes("branches:")),
    hasTagPush: hasPushKey && pushBlock.includes("tags:"),
    crons: [...onBlock.matchAll(/^\s*-\s*cron:\s*["']([^"']+)["']\s*$/gm)].map(
      (match) => match[1] ?? "",
    ),
    hasDispatch: /^ {2}workflow_dispatch:/m.test(onBlock),
    hasWorkflowCall: /^ {2}workflow_call:/m.test(onBlock),
    reusableWorkflowUses: [
      ...text.matchAll(/^\s*uses:\s*\.\/\.github\/workflows\/([A-Za-z0-9_.-]+)\s*$/gm),
    ].map((match) => match[1] ?? ""),
  };
}

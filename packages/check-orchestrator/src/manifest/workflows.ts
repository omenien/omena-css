import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { buildCheckPlan } from "./plan";
import type { CheckCiTier, CheckDiagnostic, CheckGate } from "./types";

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

const CI_REACHABILITY_ESCAPE_HATCH_POLICY = Object.freeze({
  maxGateCount: 157,
  owner: "check-orchestrator maintainers",
  reviewBy: "2026-10-31",
});

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

interface GovernedCiLeafClassification {
  readonly id: string;
  readonly reason: string;
}

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

const GOVERNED_CI_LEAF_CLASSIFICATIONS: readonly GovernedCiLeafClassification[] = [
  {
    id: "rust/benchmark/bundler-productization",
    reason: "Benchmark/profiling entrypoint; run manually when collecting performance evidence.",
  },
  {
    id: "rust/benchmark/z5/macro",
    reason: "Benchmark/profiling entrypoint; run manually when collecting performance evidence.",
  },
  {
    id: "rust/benchmark/z5/micro",
    reason: "Benchmark/profiling entrypoint; run manually when collecting performance evidence.",
  },
  {
    id: "rust/benchmark/emitted-css-golden-gate:update",
    reason:
      "Golden snapshot regeneration command; the read-only emitted CSS gate is the CI validation surface.",
  },
  {
    id: "rust/benchmark/transform-relex-baseline:update",
    reason:
      "Benchmark snapshot regeneration command; the read-only transform re-lex baseline gate is the CI validation surface.",
  },
  {
    id: "rust/z5-perf-baseline:update",
    reason:
      "Perf baseline regeneration command; the read-only z5 perf baseline and regression gates are the CI validation surfaces.",
  },
  {
    id: "rust/omena-transform-target/generated-compat:update",
    reason:
      "Generated compat data regeneration command; the read-only generated-compat gate is the CI validation surface.",
  },
  {
    id: "rust/omena-coverage-gap-report:update",
    reason:
      "Generated coverage-gap report regeneration command; the read-only coverage-gap report gate is the CI validation surface.",
  },
  {
    id: "rust/discharge-ledger:update",
    reason:
      "Generated discharge ledger regeneration command; the read-only discharge-ledger gate is the CI validation surface.",
  },
  {
    id: "release/changeset",
    reason: "Release authoring command; not a CI validation gate.",
  },
  {
    id: "tooling/cme-checker-boundary",
    reason:
      "Tooling helper gate retained for local orchestrator maintenance; canonical doctor/inventory gates run from verify CI.",
  },
  {
    id: "tooling/cme-checker-testkit-archetypes",
    reason:
      "Tooling helper gate retained for local orchestrator maintenance; canonical doctor/inventory gates run from verify CI.",
  },
  {
    id: "contract/parity-v1-golden",
    reason:
      "Contract fixture probe retained for manual compatibility checks outside the CI matrix.",
  },
  {
    id: "contract/parity-v1-smoke",
    reason:
      "Contract fixture probe retained for manual compatibility checks outside the CI matrix.",
  },
  {
    id: "contract/parity-v2-smoke",
    reason:
      "Contract fixture probe retained for manual compatibility checks outside the CI matrix.",
  },
  {
    id: "editor/editor-path-boundary",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
  },
  {
    id: "tooling/orchestrator-doctor",
    reason:
      "Tooling helper gate retained for local orchestrator maintenance; canonical doctor/inventory gates run from verify CI.",
  },
  {
    id: "tooling/omena-check",
    reason: "Check-orchestrator CLI entrypoint; workflow jobs validate the gates it runs.",
  },
  {
    id: "release/check/packaged-engine-shadow-runner",
    reason: "Legacy packaged-runner probe superseded by release/package/prepared in package CI.",
  },
  {
    id: "editor/provider-host-routing-boundary",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
  },
  {
    id: "rust/checker/release-gate-shadow-review",
    reason:
      "Checker promotion/release probe retained for manual diagnosis; scheduled checker-release-gate carries the release shadow path.",
  },
  {
    id: "rust/design-system/universality-class",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
  },
  {
    id: "rust/expression-domain/candidates",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-domain/canonical-candidate",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-domain/canonical-producer",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-domain/compare",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-domain/evaluator-candidates",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-domain/fragments",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-domain/reduced-evaluator",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-semantics/candidates",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-semantics/canonical-candidate",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-semantics/canonical-producer",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-semantics/evaluator-candidates",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-semantics/fragments",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-semantics/match-fragments",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/expression-semantics/query-fragments",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/input-producers/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/lsp-runtime-loop",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
  },
  {
    id: "rust/m4-alpha-frame-refresh-latency",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-alpha-frame-rule-fuzz",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-alpha-grn-explicit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-alpha-mdl-differential",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-alpha-qtt-semiring",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-alpha-spin-glass-policy",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-axis-a-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-axis-a-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-axis-b-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-axis-b-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-axis-c-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-axis-c-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-axis-d-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-axis-d-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-beta-ensemble",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-beta-hypergraph-monotone-fact-propagation",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-beta-transform-catalog",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-beta-multiscale-complexity-heuristic",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-closure-audit",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-gamma-categorical-evidence",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-gamma-refinement",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-gamma-smt-fuzz-full",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-gamma-smt-verification",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-gamma-demand-sliced-monotone-fact-propagation",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-gamma-zk-audit-matrix",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-readiness",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m8-dynamic-classname-deepening",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-abstract-value/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/omena-bridge/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/omena-categorical/classify-omega-truth",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/compare-design-system-theory",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/summarize-kripke-frame",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/verify-beck-chevalley",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/verify-cascade-section-aggregation-covariance",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/verify-cross-project-symmetry",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/verify-invariant-functoriality",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/verify-modal-imperative-equivalence",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/verify-s4-axioms",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-categorical/verify-cascade-section-aggregation-plan-stability",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/omena-checker/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/omena-incremental/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/omena-lsp-server/style-provider-parity",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
  },
  {
    id: "rust/omena-meta-macros-boundary",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
  },
  {
    id: "rust/omena-query/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/omena-resolver/fixture-suite",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
  },
  {
    id: "rust/omena-resolver/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/omena-semantic-observation-harness",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
  },
  {
    id: "rust/omena-semantic-split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/omena-spec-audit-boundary",
    reason:
      "Rust subsystem probe retained for targeted manual diagnosis; canonical boundary/readiness bundles carry CI coverage.",
  },
  {
    id: "rust/omena-conformance-dashboard:update",
    reason:
      "Local generator command retained for reviewed dashboard refreshes; the read-only conformance dashboard gate carries CI coverage.",
  },
  {
    id: "rust/omena-diff-test-wpt-extraction:update",
    reason:
      "Local generator command retained for reviewed WPT extraction refreshes; committed extraction and generator gates carry CI coverage.",
  },
  {
    id: "rust/omena-diff-test-wpt-extraction-source",
    reason:
      "Pinned-source maintenance gate requires an explicit external WPT checkout; committed extraction and generator gates carry CI coverage.",
  },
  {
    id: "rust/omena-tsgo-client/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/parser/split-boundary",
    reason:
      "Compatibility alias for split-boundary checks; canonical boundary bundles carry CI coverage.",
  },
  {
    id: "rust/phase-2-swap-readiness",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/query-plan/compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/selector-usage/fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/selector-usage/plan-compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/selector-usage/query-fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/shadow/compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/shadow/smoke",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/source-resolution/candidates",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/source-resolution/canonical-candidate",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/source-resolution/canonical-producer",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/source-resolution/evaluator-candidates",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/source-resolution/fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/source-resolution/match-fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/source-resolution/plan-compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/source-resolution/query-fragments",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/split/boundaries",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "rust/type-fact/compare",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "editor/selected-query-boundary",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
  },
  {
    id: "workspace/semantic-smoke",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "ts7/decision-ready",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "ts7/phase-a/decision-ready",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "ts7/phase-a/shadow-review",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "ts7/phase-b/readiness",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "tsgo/operational/shadow-review",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "workspace/check",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "editor/explain/expression",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
  },
  {
    id: "core/format",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "core/lint/fix",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "editor/omena",
    reason:
      "Editor/provider smoke probe retained for targeted manual diagnosis; product CI uses broader provider and extension-host gates.",
  },
  {
    id: "release/release/publish",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "test/bench",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "tooling/update/check-inventory",
    reason:
      "Tooling helper gate retained for local orchestrator maintenance; canonical doctor/inventory gates run from verify CI.",
  },
  {
    id: "contract/update:contract-parity-v1-golden",
    reason:
      "Contract fixture probe retained for manual compatibility checks outside the CI matrix.",
  },
  {
    id: "core/watch",
    reason:
      "Reviewed package-origin leaf retained for manual diagnosis outside the closed-world CI surface.",
  },
  {
    id: "core/prepare",
    reason:
      "Lefthook git-hook installer; runs on local pnpm install only (skipped in CI) and is not a CI gate.",
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

  for (const fileName of readdirSync(workflowsDir).toSorted()) {
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
  for (const fileName of readdirSync(workflowsDir).toSorted()) {
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

export function findCiRequiredAggregationDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const workflowPath = path.join(rootDir, ".github/workflows/ci.yml");
  if (!existsSync(workflowPath)) return [];

  const lines = readFileSync(workflowPath, "utf8").split(/\r?\n/);
  const jobs = parseWorkflowJobs(lines);
  const annotations = jobs.map((job) => ({
    job,
    required: parseWorkflowRequiredAnnotation(lines, job),
  }));
  if (annotations.every(({ required }) => required === null)) return [];

  const diagnostics: CheckDiagnostic[] = [];
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

  const block = lines.slice(aggregator.start, aggregator.end).join("\n");
  if (!/^\s+if:\s*\$\{\{\s*always\(\)\s*\}\}\s*$/m.test(block)) {
    diagnostics.push({
      severity: "error",
      code: "ci-required-missing-always",
      message:
        'The "ci-required" aggregate job must use "if: ${{ always() }}" so failed or cancelled dependencies are evaluated.',
    });
  }
  if (!block.includes("scripts/check-ci-required-results.mjs")) {
    diagnostics.push({
      severity: "error",
      code: "ci-required-result-check-missing",
      message:
        'The "ci-required" aggregate job must execute scripts/check-ci-required-results.mjs.',
    });
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

  if (escapeHatchGateIds.length > CI_REACHABILITY_ESCAPE_HATCH_POLICY.maxGateCount) {
    diagnostics.push({
      severity: "error",
      code: "ci-tier-escape-hatch-budget-exceeded",
      message:
        `CI reachability escape-hatch population ${escapeHatchGateIds.length} exceeds the governed maximum ` +
        `${CI_REACHABILITY_ESCAPE_HATCH_POLICY.maxGateCount}; owner=${CI_REACHABILITY_ESCAPE_HATCH_POLICY.owner}, ` +
        `reviewBy=${CI_REACHABILITY_ESCAPE_HATCH_POLICY.reviewBy}.`,
    });
  } else if (escapeHatchGateIds.length > 0) {
    diagnostics.push({
      severity: "warning",
      code: "ci-tier-escape-hatch-summary",
      message:
        `CI reachability escape-hatch population: ${escapeHatchGateIds.length}/` +
        `${CI_REACHABILITY_ESCAPE_HATCH_POLICY.maxGateCount} gate(s); ` +
        `owner=${CI_REACHABILITY_ESCAPE_HATCH_POLICY.owner}, ` +
        `reviewBy=${CI_REACHABILITY_ESCAPE_HATCH_POLICY.reviewBy}.`,
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

  for (const fileName of readdirSync(workflowsDir).toSorted()) {
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

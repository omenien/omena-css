import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { buildCheckPlan } from "./plan";
import type { CheckCiTier, CheckDiagnostic, CheckGate } from "./types";

const PNPM_SCRIPT_REF = /\bpnpm\s+(?:run\s+)?([A-Za-z0-9:_-]+)/g;
const OMENA_CHECK_TARGET_REF =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(run|bundle)\s+([A-Za-z0-9:_@/.-]+)/g;
const WORKFLOW_CI_TIER_ANNOTATION = /^\s*#\s*omena-ci-tier:\s*([A-Za-z0-9_-]+)\s*$/;

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
    id: "contract/parity-v2-golden",
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
    id: "rust/bundler-productization-benchmark",
    reason: "Benchmark/profiling entrypoint; run manually when collecting performance evidence.",
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
    id: "rust/m4-beta-hypergraph-ifds",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-beta-lawvere",
    reason:
      "Research evidence gate retained for manual review; not part of the current PR or scheduled CI surface.",
  },
  {
    id: "rust/m4-beta-rg-flow",
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
    id: "rust/m4-gamma-streaming-ifds",
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
    id: "rust/omena-categorical/verify-cosheaf-covariance",
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
    id: "rust/omena-categorical/verify-site-stability",
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
    });
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

  if (escapeHatchGateIds.length > 0) {
    diagnostics.push({
      severity: "warning",
      code: "ci-tier-escape-hatch-summary",
      message: `CI reachability escape-hatch population: ${escapeHatchGateIds.length} gate(s).`,
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
      reachable.set(tier, ids);
    }
  }

  return reachable;
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

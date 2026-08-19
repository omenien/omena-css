import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import type { CheckDiagnostic, CheckGate, GateLifecycle } from "./types";

/**
 * The mechanical tier rule, recomputed at LOAD time from the lane definition
 * the policy itself records: direct members of the named bundles, the named
 * single gates, and every `omena-check run|bundle` target inside the named
 * ci.yml jobs. Population completeness holds in BOTH directions against the
 * record set (both directions — adding a tier gate without a record is as loud as retiring one).
 */
export function expensiveTierMembers(
  rootDir: string,
  gates: readonly CheckGate[],
  lanes: GatePolicy["lanes"],
): ReadonlySet<string> {
  const ids = new Set<string>();
  const resolve = (target: string): CheckGate | undefined =>
    gates.find((gate) => gate.id === target || gate.scriptName === target) ??
    gates.find((gate) => gate.deprecatedAliases?.includes(target)) ??
    gates.find((gate) => gate.id.endsWith(`/${target}`));
  for (const bundleId of lanes.bundles) {
    const bundle = resolve(bundleId);
    if (!bundle) continue;
    ids.add(bundle.id);
    for (const target of bundle.referencedTargets ?? []) {
      const member = resolve(target);
      if (member) ids.add(member.id);
    }
  }
  for (const single of lanes.singles) {
    const gate = resolve(single);
    if (gate) ids.add(gate.id);
  }
  const ciPath = path.join(rootDir, ".github/workflows/ci.yml");
  if (existsSync(ciPath)) {
    const lines = readFileSync(ciPath, "utf8").split(/\r?\n/);
    let currentJob = "";
    for (const line of lines) {
      const header = line.match(/^ {2}([A-Za-z0-9_-]+):\s*$/);
      if (header) currentJob = header[1] ?? "";
      if (!lanes.ciJobs.includes(currentJob)) continue;
      const target = line.match(/omena-check (?:run|bundle) ([A-Za-z0-9:_@/.-]+)/)?.[1];
      if (!target) continue;
      const gate = resolve(target);
      if (gate) ids.add(gate.id);
    }
  }
  return ids;
}

// the WPT module-promotion mechanism generalized to the gate corpus.
// The expensive tier (static enumeration until the cost-ledger successor replaces
// it) carries per-gate lifecycle records with a review interval; the CI
// reachability escape hatch folds into the same file so its review date is
// COMPARED, not prose. The clock is env-injectable (WPT pattern) so the
// manifest stays deterministic under test.

export interface GatePolicyRecord {
  readonly gateId: string;
  readonly stage: "blocking" | "advisory";
  readonly reviewedAt: string;
  readonly reviewAfter: string;
  readonly evidence: string;
  readonly holdReason?: string;
}

export interface GatePolicy {
  readonly schemaVersion: string;
  readonly product: string;
  readonly template: {
    readonly reviewIntervalDays: number;
    readonly advisoryHoldReasons: readonly string[];
  };
  readonly costSource: string;
  readonly escapeHatch: {
    readonly maxGateCount: number;
    readonly owner: string;
    readonly reviewedAt: string;
    readonly reviewAfter: string;
  };
  readonly lanes: {
    readonly bundles: readonly string[];
    readonly singles: readonly string[];
    readonly ciJobs: readonly string[];
  };
  readonly records: readonly GatePolicyRecord[];
}

export function gatePolicyPath(rootDir: string): string {
  return path.join(rootDir, "packages/check-orchestrator/gate-policy.json");
}

export function loadGatePolicy(rootDir: string): GatePolicy | null {
  const policyPath = gatePolicyPath(rootDir);
  if (!existsSync(policyPath)) return null;
  return JSON.parse(readFileSync(policyPath, "utf8")) as GatePolicy;
}

export function gatePolicyToday(): string {
  return process.env.OMENA_GATE_POLICY_TODAY ?? new Date().toISOString().slice(0, 10);
}

const ISO_DATE = /^\d{4}-\d{2}-\d{2}$/;

export function findGatePolicyDiagnostics(
  rootDir: string,
  gates: readonly CheckGate[],
  lifecycleByGateId: ReadonlyMap<string, GateLifecycle>,
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const policy = loadGatePolicy(rootDir);
  if (!policy) {
    // The policy lives with the orchestrator package; roots without the
    // package (test fixtures) are out of scope, but a repo that HAS the
    // package must never silently lose the policy file (the vacuous-lock
    // species this program keeps meeting).
    if (existsSync(path.join(rootDir, "packages/check-orchestrator"))) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-missing",
        message: "packages/check-orchestrator/gate-policy.json is absent.",
      });
    }
    return diagnostics;
  }
  const today = gatePolicyToday();
  const gateIds = new Set(gates.map((gate) => gate.id));
  const holdVocabulary = new Set(policy.template.advisoryHoldReasons);
  const seen = new Set<string>();

  const checkWindow = (label: string, reviewedAt: string, reviewAfter: string): void => {
    if (!ISO_DATE.test(reviewedAt) || !ISO_DATE.test(reviewAfter)) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-invalid-date",
        message: `${label} carries a non-ISO review date.`,
      });
      return;
    }
    const days =
      (Date.parse(`${reviewAfter}T00:00:00Z`) - Date.parse(`${reviewedAt}T00:00:00Z`)) / 86_400_000;
    if (days !== policy.template.reviewIntervalDays) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-interval-drift",
        message: `${label} review interval is ${days}d; the template requires ${policy.template.reviewIntervalDays}d.`,
      });
    }
    if (today > reviewAfter) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-review-expired",
        message:
          `${label} review expired on ${reviewAfter} (today=${today}). Re-adjudicate the entry, ` +
          `set reviewedAt=today and reviewAfter=today+${policy.template.reviewIntervalDays}d in gate-policy.json.`,
      });
    }
  };

  checkWindow("escapeHatch", policy.escapeHatch.reviewedAt, policy.escapeHatch.reviewAfter);

  for (const record of policy.records) {
    if (seen.has(record.gateId)) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-duplicate-record",
        message: `gate-policy record "${record.gateId}" appears more than once.`,
      });
      continue;
    }
    seen.add(record.gateId);
    if (!gateIds.has(record.gateId)) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-unknown-gate",
        message: `gate-policy record "${record.gateId}" names a gate that no longer exists; retire or rename the record in the same commit as the gate change.`,
      });
      continue;
    }
    checkWindow(`record "${record.gateId}"`, record.reviewedAt, record.reviewAfter);
    if (record.holdReason !== undefined && !holdVocabulary.has(record.holdReason)) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-unknown-hold-reason",
        message: `gate-policy record "${record.gateId}" carries holdReason "${record.holdReason}" outside the template vocabulary.`,
      });
    }
    const derived = lifecycleByGateId.get(record.gateId)?.strength;
    if (derived && record.stage !== derived && !record.holdReason) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-stage-mismatch",
        message:
          `gate-policy record "${record.gateId}" is staged "${record.stage}" but the derived strength is ` +
          `"${derived}"; align the stage or carry a vocabulary holdReason.`,
      });
    }
  }

  const tier = expensiveTierMembers(rootDir, gates, policy.lanes);
  for (const memberId of tier) {
    if (!seen.has(memberId)) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-record-missing",
        message:
          `expensive-tier gate "${memberId}" has no gate-policy record; adding or moving a gate ` +
          `into a governed lane must land its record in the same commit.`,
      });
    }
  }
  for (const recordId of seen) {
    if (!tier.has(recordId) && gateIds.has(recordId)) {
      diagnostics.push({
        severity: "error",
        code: "gate-policy-record-not-in-tier",
        message:
          `gate-policy record "${recordId}" is no longer an expensive-tier member; retire the ` +
          `record in the same commit as the lane change.`,
      });
    }
  }
  return diagnostics;
}

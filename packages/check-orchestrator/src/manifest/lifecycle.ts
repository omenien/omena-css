import type { CheckDiagnostic, CheckGate, GateCadence, GateLifecycle, GateStrength } from "./types";
import {
  collectWorkflowLifecycleView,
  type WorkflowJobView,
  type WorkflowTriggerFacts,
} from "./workflows";

// g130-S1: cadence and strength are TOTAL, DERIVED axes.
// - cadence(gate) = the fastest cadence among the workflows whose jobs reach
//   the gate (push < nightly < weekly < release < manual); unreached => manual.
// - strength(gate) = blocking iff the gate is reached from a ci.yml job whose
//   result reaches `ci-required` through the needs graph (GitHub skip-cascade
//   makes every needs ancestor failure-propagating), plus the bundle fixpoint;
//   everything else => advisory.
// Declared values are optional overrides validated against the derivation.

const CADENCE_ORDER: readonly GateCadence[] = ["push", "nightly", "weekly", "release", "manual"];

const RELEASE_FILE_PREFIXES = ["_publish", "publish-", "release-"];

export interface GateLifecycleComputation {
  readonly byGateId: ReadonlyMap<string, GateLifecycle>;
  readonly workflowCadence: ReadonlyMap<string, GateCadence>;
}

export function computeGateLifecycles(
  rootDir: string,
  gates: readonly CheckGate[],
): GateLifecycleComputation {
  const view = collectWorkflowLifecycleView(rootDir, gates);
  const workflowCadence = classifyWorkflowCadences(view.triggers);
  const blockingGateIds = computeBlockingGateIds(view.jobs, gates);

  const derivedCadence = new Map<string, GateCadence>();
  for (const job of view.jobs) {
    const cadence = workflowCadence.get(job.workflowFile);
    if (!cadence) continue;
    for (const gateId of job.gateIds) {
      const current = derivedCadence.get(gateId);
      if (!current || CADENCE_ORDER.indexOf(cadence) < CADENCE_ORDER.indexOf(current)) {
        derivedCadence.set(gateId, cadence);
      }
    }
  }
  // Bundle fixpoint: a gate whose referenced targets are all reachable adopts
  // the fastest cadence among them (mirrors the tier-reachability fixpoint).
  let changed = true;
  while (changed) {
    changed = false;
    for (const gate of gates) {
      if (derivedCadence.has(gate.id) || !gate.referencedTargets?.length) continue;
      const memberCadences = gate.referencedTargets.map((target) =>
        derivedCadence.get(resolveGateIdForTarget(gates, target) ?? ""),
      );
      if (memberCadences.some((cadence) => cadence === undefined)) continue;
      const fastest = (memberCadences as GateCadence[]).toSorted(
        (left, right) => CADENCE_ORDER.indexOf(left) - CADENCE_ORDER.indexOf(right),
      )[0];
      if (!fastest) continue;
      derivedCadence.set(gate.id, fastest);
      changed = true;
    }
  }

  const byGateId = new Map<string, GateLifecycle>();
  for (const gate of gates) {
    const cadenceDerived: GateCadence = derivedCadence.get(gate.id) ?? "manual";
    const strengthDerived: GateStrength = blockingGateIds.has(gate.id) ? "blocking" : "advisory";
    const cadence = gate.cadence ?? cadenceDerived;
    const strength = gate.strength ?? strengthDerived;
    byGateId.set(gate.id, {
      cadence,
      strength,
      cadenceSource: gate.cadence !== undefined ? "declared-override" : "derived",
      strengthSource: gate.strength !== undefined ? "declared-override" : "derived",
    });
  }
  return { byGateId, workflowCadence };
}

export function findGateLifecycleDiagnostics(
  rootDir: string,
  gates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const view = collectWorkflowLifecycleView(rootDir, gates);
  const workflowCadence = classifyWorkflowCadences(view.triggers);
  const blockingGateIds = computeBlockingGateIds(view.jobs, gates);
  const derived = computeDerivedOnly(view.jobs, workflowCadence, blockingGateIds, gates);

  for (const gate of gates) {
    if (
      gate.axisException !== undefined &&
      gate.cadence === undefined &&
      gate.strength === undefined
    ) {
      diagnostics.push({
        severity: "error",
        code: "gate-axis-exception-unused",
        message: `Gate "${gate.id}" declares axisException without declaring a cadence or strength override.`,
      });
    }
    const derivedAxes = derived.get(gate.id);
    if (!derivedAxes) continue;
    for (const [axis, declaredValue, derivedValue] of [
      ["cadence", gate.cadence, derivedAxes.cadence],
      ["strength", gate.strength, derivedAxes.strength],
    ] as const) {
      if (declaredValue === undefined || declaredValue === derivedValue) continue;
      if (gate.axisException?.trim()) continue;
      diagnostics.push({
        severity: "error",
        code: "gate-axis-override-mismatch",
        message:
          `Gate "${gate.id}" declares ${axis} "${declaredValue}" but the derivation says ` +
          `"${derivedValue}"; agree with the derivation or carry axisException with a reason.`,
      });
    }
  }
  return diagnostics;
}

function computeDerivedOnly(
  jobs: readonly WorkflowJobView[],
  workflowCadence: ReadonlyMap<string, GateCadence>,
  blockingGateIds: ReadonlySet<string>,
  gates: readonly CheckGate[],
): Map<string, { cadence: GateCadence; strength: GateStrength }> {
  const cadences = new Map<string, GateCadence>();
  for (const job of jobs) {
    const cadence = workflowCadence.get(job.workflowFile);
    if (!cadence) continue;
    for (const gateId of job.gateIds) {
      const current = cadences.get(gateId);
      if (!current || CADENCE_ORDER.indexOf(cadence) < CADENCE_ORDER.indexOf(current)) {
        cadences.set(gateId, cadence);
      }
    }
  }
  const result = new Map<string, { cadence: GateCadence; strength: GateStrength }>();
  for (const gate of gates) {
    result.set(gate.id, {
      cadence: cadences.get(gate.id) ?? "manual",
      strength: blockingGateIds.has(gate.id) ? "blocking" : "advisory",
    });
  }
  return result;
}

export function classifyWorkflowCadences(
  triggers: readonly WorkflowTriggerFacts[],
): Map<string, GateCadence> {
  const byFile = new Map<string, GateCadence>();
  for (const facts of triggers) {
    byFile.set(facts.fileName, classifyOwnCadence(facts));
  }
  // workflow_call-only files inherit the fastest cadence among their callers.
  let changed = true;
  while (changed) {
    changed = false;
    for (const facts of triggers) {
      for (const used of facts.reusableWorkflowUses) {
        const caller = byFile.get(facts.fileName);
        const callee = byFile.get(used);
        if (!caller || !callee) continue;
        if (CADENCE_ORDER.indexOf(caller) < CADENCE_ORDER.indexOf(callee)) {
          byFile.set(used, caller);
          changed = true;
        }
      }
    }
  }
  return byFile;
}

function classifyOwnCadence(facts: WorkflowTriggerFacts): GateCadence {
  if (facts.hasBranchPush) return "push";
  const cronCadences = new Set(facts.crons.map(classifyCron));
  if (cronCadences.has("nightly")) return "nightly";
  if (cronCadences.has("weekly")) return "weekly";
  if (facts.hasTagPush) return "release";
  if (RELEASE_FILE_PREFIXES.some((prefix) => facts.fileName.startsWith(prefix))) return "release";
  return "manual";
}

function classifyCron(cron: string): GateCadence {
  const fields = cron.trim().split(/\s+/);
  const dayOfMonth = fields[2] ?? "*";
  const dayOfWeek = fields[4] ?? "*";
  if (dayOfMonth === "*" && dayOfWeek === "*") return "nightly";
  return "weekly";
}

function computeBlockingGateIds(
  jobs: readonly WorkflowJobView[],
  gates: readonly CheckGate[],
): Set<string> {
  const ciJobs = jobs.filter((job) => job.workflowFile === "ci.yml");
  const byName = new Map(ciJobs.map((job) => [job.jobName, job]));
  const blockingJobs = new Set<string>();
  const queue = ["ci-required"];
  while (queue.length > 0) {
    const name = queue.pop();
    if (!name || blockingJobs.has(name)) continue;
    blockingJobs.add(name);
    for (const need of byName.get(name)?.needs ?? []) queue.push(need);
  }
  blockingJobs.delete("ci-required");

  const ids = new Set<string>();
  for (const job of ciJobs) {
    if (!blockingJobs.has(job.jobName)) continue;
    for (const gateId of job.gateIds) ids.add(gateId);
  }
  // Limitation (disclosed): jobs living in a workflow_call-only file (today:
  // _build-native-binaries.yml) are not attributed to their calling ci.yml job
  // here. Every such job currently runs ZERO registered gates, so the strength
  // derivation is unaffected; if a reusable-workflow job ever gains a gate,
  // extend the walker to attribute the calling job before relying on it.
  let changed = true;
  while (changed) {
    changed = false;
    for (const gate of gates) {
      if (ids.has(gate.id) || !gate.referencedTargets?.length) continue;
      if (
        gate.referencedTargets.every((target) =>
          ids.has(resolveGateIdForTarget(gates, target) ?? ""),
        )
      ) {
        ids.add(gate.id);
        changed = true;
      }
    }
  }
  return ids;
}

function resolveGateIdForTarget(gates: readonly CheckGate[], target: string): string | undefined {
  return (
    gates.find((gate) => gate.id === target || gate.scriptName === target) ??
    gates.find((gate) => gate.deprecatedAliases?.includes(target)) ??
    gates.find((gate) => gate.id.endsWith(`/${target}`))
  )?.id;
}

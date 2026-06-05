import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { buildCheckPlan } from "./plan";
import type { CheckCiTier, CheckDiagnostic, CheckGate } from "./types";

const PNPM_SCRIPT_REF = /\bpnpm\s+(?:run\s+)?([A-Za-z0-9:_-]+)/g;
const OMENA_CHECK_TARGET_REF =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(run|bundle)\s+([A-Za-z0-9:_@/.-]+)/g;

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

export function findCiTierReachabilityDiagnostics(
  rootDir: string,
  gates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const declaredGates = gates.filter(
    (gate) => gate.origin === "declared" || gate.origin === "package+declared",
  );

  for (const gate of declaredGates) {
    if (!gate.ciTier) {
      diagnostics.push({
        severity: "error",
        code: "declared-gate-missing-ci-tier",
        message: `Declared gate "${gate.id}" must set ciTier explicitly.`,
      });
    }
    if (gate.ciTier === "none" && !gate.tags?.includes("ci-unreachable-allowed")) {
      diagnostics.push({
        severity: "error",
        code: "ci-tier-none-not-allowed",
        message: `Declared gate "${gate.id}" uses ciTier "none" without the ci-unreachable-allowed tag.`,
      });
    }
  }

  const reachableByTier = buildReachableGateIdsByTier(rootDir, gates);
  for (const gate of gates) {
    if (!gate.ciTier || gate.ciTier === "none" || gate.ciTier === "manual") {
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
      const tier = inferWorkflowJobTier(fileName, workflowText, job.name);
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
  jobName: string,
): CheckCiTier | null {
  if (fileName === "ci.yml" && jobName === "verify") return "verify";
  if (fileName === "ci.yml" && jobName === "closure-fast") return "closure-fast";
  if (/^\s+schedule:\s*$/m.test(workflowText)) return "scheduled";
  return null;
}

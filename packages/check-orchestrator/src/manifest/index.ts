import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import {
  buildDeclaredGates,
  DECLARED_CHECK_GATES,
  getDeprecatedPackageScriptReplacement,
} from "./declared";
import { renderCheckInventory } from "./inventory";
import { buildCheckPlan, renderCheckPlan } from "./plan";
import { classifyScript } from "./scopes";
import { buildCheckSurfaceReport, findAliasChains, renderCheckSurfaceReport } from "./surface";
import { findWorkflowBypassDiagnostics } from "./workflows";
import type {
  CheckBundle,
  CheckDiagnostic,
  CheckGate,
  CheckManifest,
  RootPackageJson,
} from "./types";

export type {
  CheckAliasChain,
  CheckBundle,
  CheckBundleSurface,
  CheckCiTier,
  CheckDiagnostic,
  CheckGate,
  CheckGateOrigin,
  CheckManifest,
  CheckPlan,
  CheckPlanStep,
  CheckScopeId,
  CheckSurfaceReport,
  DeclaredCheckGateV0,
} from "./types";
export {
  buildDeclaredGates,
  buildCheckPlan,
  buildCheckSurfaceReport,
  renderCheckInventory,
  renderCheckPlan,
  renderCheckSurfaceReport,
};

const PACKAGE_SCRIPT_REF = /\bpnpm\s+(?:run\s+)?([A-Za-z0-9:_-]+)/g;
const CHECK_ORCHESTRATOR_TARGET_REF =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(run|bundle)\s+([A-Za-z0-9:_@/.-]+)/g;

export function loadCheckManifest(rootDir = findRepoRoot()): CheckManifest {
  const packageJson = readRootPackageJson(rootDir);
  const scripts = packageJson.scripts ?? {};
  const diagnostics: CheckDiagnostic[] = [];
  const packageGates = Object.entries(scripts)
    .toSorted(([left], [right]) => left.localeCompare(right))
    .map(([scriptName, command]) => buildGate(scriptName, command, scripts, diagnostics));
  const declaredGates = shouldLoadRepoDeclaredGates(rootDir)
    ? buildDeclaredGates(packageGates, DECLARED_CHECK_GATES, diagnostics)
    : [];
  const gates = [...packageGates, ...declaredGates];

  diagnostics.push(...findDuplicateGateIds(gates));
  diagnostics.push(...findAliasChainDiagnostics(gates));
  diagnostics.push(...findCheckOrchestratorTargetDiagnostics(gates));
  diagnostics.push(...findWorkflowBypassDiagnostics(rootDir, gates));

  return {
    rootDir,
    gates,
    bundles: gates.filter(
      (gate): gate is CheckBundle => gate.kind === "bundle" || gate.kind === "alias",
    ),
    diagnostics,
  };
}

export function resolveGateTarget(
  manifest: Pick<CheckManifest, "gates">,
  target: string,
): CheckGate | null {
  return (
    manifest.gates.find((gate) => gate.id === target || gate.scriptName === target) ??
    manifest.gates.find((gate) => gate.deprecatedAliases?.includes(target)) ??
    manifest.gates.find((gate) => gate.id.endsWith(`/${target}`)) ??
    null
  );
}

export function runDoctor(
  manifest: Pick<CheckManifest, "diagnostics">,
): readonly CheckDiagnostic[] {
  return manifest.diagnostics;
}

export function findRepoRoot(startDir = process.cwd()): string {
  let dir = path.resolve(startDir);
  while (true) {
    try {
      const candidate = readRootPackageJson(dir);
      if (candidate.name === "omena-css") return dir;
    } catch {
      // Keep walking.
    }

    const parent = path.dirname(dir);
    if (parent === dir) {
      throw new Error(`Unable to locate omena-css repo root from ${startDir}`);
    }
    dir = parent;
  }
}

function buildGate(
  scriptName: string,
  command: string,
  scripts: Record<string, string>,
  diagnostics: CheckDiagnostic[],
): CheckGate {
  const scope = classifyScript(scriptName);
  if (!scope) {
    diagnostics.push({
      severity: "error",
      code: "unknown-script-scope",
      message: `Script "${scriptName}" is not covered by a check-orchestrator scope.`,
    });
  }

  const referencedScripts = extractReferencedScripts(command, scripts);

  const deprecatedBy = getDeprecatedPackageScriptReplacement(scriptName);

  return {
    id: scope?.toGateId(scriptName) ?? `unknown/${scriptName.replace(":", "/")}`,
    scriptName,
    command,
    scope: scope?.id ?? "tooling",
    kind: detectGateKind(scriptName, command, referencedScripts),
    origin: "package",
    referencedScripts,
    ...(deprecatedBy ? { deprecatedBy } : {}),
  };
}

function detectGateKind(
  scriptName: string,
  command: string,
  referencedScripts: readonly string[],
): CheckGate["kind"] {
  if (isAliasScript(command, referencedScripts)) return "alias";
  if (
    referencedScripts.length > 0 &&
    /(?:bundle|lane|readiness|decision-ready|shadow|verify|consumers|boundary|capability)$/.test(
      scriptName,
    )
  ) {
    return "bundle";
  }
  if (scriptName === "check" || scriptName.startsWith("check:") || scriptName.startsWith("test")) {
    return "gate";
  }
  return "command";
}

function isAliasScript(command: string, referencedScripts: readonly string[]): boolean {
  if (referencedScripts.length !== 1) return false;
  const trimmedCommand = command.trim();
  return (
    /^pnpm\s+(?:run\s+)?[A-Za-z0-9:_-]+\s*$/.test(trimmedCommand) ||
    /^pnpm\s+(?:run\s+)?omena-check\s+(?:run|bundle)\s+[A-Za-z0-9:_@/.-]+\s*$/.test(trimmedCommand)
  );
}

function extractReferencedScripts(
  command: string,
  scripts: Record<string, string>,
): readonly string[] {
  const refs = new Set<string>();
  const cmeTargetMatches = [...command.matchAll(CHECK_ORCHESTRATOR_TARGET_REF)];

  for (const match of cmeTargetMatches) {
    const target = match[2];
    const referencedScript = target ? resolveScriptNameFromTarget(target, scripts) : null;
    if (referencedScript) {
      refs.add(referencedScript);
    }
  }

  for (const match of command.matchAll(PACKAGE_SCRIPT_REF)) {
    const scriptName = match[1];
    if (scriptName === "omena-check" && cmeTargetMatches.length > 0) {
      continue;
    }
    if (scriptName && Object.hasOwn(scripts, scriptName)) {
      refs.add(scriptName);
    }
  }
  return [...refs].toSorted();
}

function findCheckOrchestratorTargetDiagnostics(
  gates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];

  for (const gate of gates) {
    for (const match of gate.command.matchAll(CHECK_ORCHESTRATOR_TARGET_REF)) {
      const command = match[1];
      const target = match[2];
      if (!command || !target) continue;

      const resolved = resolveGateTarget({ gates }, target);
      if (!resolved) {
        diagnostics.push({
          severity: "error",
          code: "unknown-omena-check-target",
          message: `Script "${gate.scriptName}" references unknown omena-check target "${target}".`,
        });
        continue;
      }

      if (target !== resolved.id) {
        diagnostics.push({
          severity: "error",
          code: "non-canonical-omena-check-target",
          message: `Script "${gate.scriptName}" references omena-check target "${target}"; use canonical gate id "${resolved.id}".`,
        });
      }

      if (command === "bundle" && resolved.kind !== "bundle" && resolved.kind !== "alias") {
        diagnostics.push({
          severity: "error",
          code: "non-bundle-omena-check-target",
          message: `Script "${gate.scriptName}" uses omena-check bundle for non-bundle target "${target}".`,
        });
      }
    }
  }

  return diagnostics;
}

function resolveScriptNameFromTarget(
  target: string,
  scripts: Record<string, string>,
): string | null {
  if (Object.hasOwn(scripts, target)) return target;

  for (const scriptName of Object.keys(scripts)) {
    const scope = classifyScript(scriptName);
    if (scope?.toGateId(scriptName) === target) {
      return scriptName;
    }
  }

  return null;
}

function findDuplicateGateIds(gates: readonly CheckGate[]): readonly CheckDiagnostic[] {
  const byId = new Map<string, string[]>();
  for (const gate of gates) {
    const scripts = byId.get(gate.id) ?? [];
    scripts.push(gate.scriptName);
    byId.set(gate.id, scripts);
  }

  return [...byId.entries()]
    .filter(([, scripts]) => scripts.length > 1)
    .map(([id, scripts]) => ({
      severity: "error" as const,
      code: "duplicate-gate-id",
      message: `Gate id "${id}" is shared by scripts: ${scripts.join(", ")}`,
    }));
}

function findAliasChainDiagnostics(gates: readonly CheckGate[]): readonly CheckDiagnostic[] {
  return findAliasChains(gates).map((chain) => ({
    severity: "warning" as const,
    code: "alias-chain",
    message: `Alias "${chain.aliasScriptName}" references alias "${chain.referencedAliasScriptName}"; point to "${chain.directTargetScripts.join(", ")}" directly or keep only one public alias.`,
  }));
}

function readRootPackageJson(rootDir: string): RootPackageJson {
  const packageJsonPath = path.join(rootDir, "package.json");
  return JSON.parse(readFileSync(packageJsonPath, "utf8")) as RootPackageJson;
}

function shouldLoadRepoDeclaredGates(rootDir: string): boolean {
  return existsSync(path.join(rootDir, "packages/check-orchestrator/src/manifest/declared.ts"));
}

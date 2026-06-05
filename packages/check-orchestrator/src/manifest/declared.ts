import type { CheckGate, CheckCiTier, CheckDiagnostic, DeclaredCheckGateV0 } from "./types";

const VALID_CI_TIERS = new Set<CheckCiTier>([
  "verify",
  "closure-fast",
  "scheduled",
  "manual",
  "none",
]);

export const DECLARED_CHECK_GATES = [
  {
    id: "rust/runtime-query-api-hardening",
    kind: "alias",
    scope: "rust",
    deps: ["rust/m1-runtime-query-api-hardening"],
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
    deprecatedAliases: [
      "rust/m1-runtime-query-api-hardening",
      "check:rust-m1-runtime-query-api-hardening",
    ],
  },
  {
    id: "rust/product-facing-capability",
    kind: "alias",
    scope: "rust",
    deps: ["rust/m2-product-facing-capability"],
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
    deprecatedAliases: [
      "rust/m2-product-facing-capability",
      "check:rust-m2-product-facing-capability",
    ],
  },
  {
    id: "rust/theory-generalization",
    kind: "alias",
    scope: "rust",
    deps: ["rust/m3-theoretical-moat-generalization"],
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
    deprecatedAliases: [
      "rust/m3-theoretical-moat-generalization",
      "check:rust-m3-theoretical-moat-generalization",
    ],
  },
] satisfies readonly DeclaredCheckGateV0[];

const LEGACY_PACKAGE_SCRIPT_REPLACEMENTS = new Map(
  DECLARED_CHECK_GATES.flatMap((gate) =>
    (gate.deprecatedAliases ?? [])
      .filter((alias) => alias.startsWith("check:"))
      .map((alias) => [alias, gate.id] as const),
  ),
);

export function getDeprecatedPackageScriptReplacement(scriptName: string): string | undefined {
  return LEGACY_PACKAGE_SCRIPT_REPLACEMENTS.get(scriptName);
}

export function buildDeclaredGates(
  packageGates: readonly CheckGate[],
  declarations: readonly DeclaredCheckGateV0[],
  diagnostics: CheckDiagnostic[],
): readonly CheckGate[] {
  const duplicateDeclaredIds = findDuplicateValues(declarations.map((gate) => gate.id));
  for (const id of duplicateDeclaredIds) {
    diagnostics.push({
      severity: "error",
      code: "duplicate-declared-gate-id",
      message: `Declared gate id "${id}" is defined more than once.`,
    });
  }

  const packageGateIds = new Set(packageGates.map((gate) => gate.id));
  for (const declaration of declarations) {
    if (packageGateIds.has(declaration.id)) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-gate-id-collision",
        message: `Declared gate id "${declaration.id}" collides with a package-derived gate id.`,
      });
    }
  }

  const declaredGates = declarations.map((declaration) =>
    buildDeclaredGate(declaration, diagnostics),
  );
  const allGates = [...packageGates, ...declaredGates];

  diagnostics.push(...findDeclaredDependencyDiagnostics(declarations, allGates));
  diagnostics.push(...findDeclaredCycleDiagnostics(declarations));

  return declaredGates.map((gate) =>
    Object.assign({}, gate, {
      referencedScripts: (gate.referencedTargets ?? [])
        .map((target) => resolveDeclaredDependency(allGates, target)?.scriptName)
        .filter((scriptName): scriptName is string => Boolean(scriptName)),
    }),
  );
}

function buildDeclaredGate(
  declaration: DeclaredCheckGateV0,
  diagnostics: CheckDiagnostic[],
): CheckGate {
  validateDeclaredShape(declaration, diagnostics);

  return {
    id: declaration.id,
    scriptName: `@declared/${declaration.id}`,
    command: declaration.command?.join(" ") ?? declaration.deps?.join(" && ") ?? "",
    scope: declaration.scope,
    kind: declaration.kind,
    origin: "declared",
    referencedTargets: declaration.deps ?? [],
    referencedScripts: [],
    ...(declaration.command ? { commandParts: declaration.command } : {}),
    ...(declaration.tags ? { tags: declaration.tags } : {}),
    ...(declaration.timeoutMinutes ? { timeoutMinutes: declaration.timeoutMinutes } : {}),
    ...(declaration.ciTier ? { ciTier: declaration.ciTier } : {}),
    ...(declaration.ciGroup ? { ciGroup: declaration.ciGroup } : {}),
    ...(declaration.deprecatedAliases ? { deprecatedAliases: declaration.deprecatedAliases } : {}),
  };
}

function validateDeclaredShape(
  declaration: DeclaredCheckGateV0,
  diagnostics: CheckDiagnostic[],
): void {
  const hasCommand = (declaration.command?.length ?? 0) > 0;
  const depCount = declaration.deps?.length ?? 0;

  if ((declaration.kind === "command" || declaration.kind === "gate") && !hasCommand) {
    diagnostics.push({
      severity: "error",
      code: "declared-gate-missing-command",
      message: `Declared ${declaration.kind} "${declaration.id}" must define command parts.`,
    });
  }

  if (declaration.kind === "bundle" && depCount === 0) {
    diagnostics.push({
      severity: "error",
      code: "declared-bundle-missing-deps",
      message: `Declared bundle "${declaration.id}" must define deps.`,
    });
  }

  if (declaration.kind === "alias" && depCount !== 1) {
    diagnostics.push({
      severity: "error",
      code: "declared-alias-invalid-deps",
      message: `Declared alias "${declaration.id}" must point to exactly one dep.`,
    });
  }

  if (declaration.ciTier && !VALID_CI_TIERS.has(declaration.ciTier)) {
    diagnostics.push({
      severity: "error",
      code: "declared-gate-unknown-ci-tier",
      message: `Declared gate "${declaration.id}" uses unknown ciTier "${declaration.ciTier}".`,
    });
  }
}

function findDeclaredDependencyDiagnostics(
  declarations: readonly DeclaredCheckGateV0[],
  gates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  for (const declaration of declarations) {
    for (const dep of declaration.deps ?? []) {
      if (!resolveDeclaredDependency(gates, dep)) {
        diagnostics.push({
          severity: "error",
          code: "declared-gate-unknown-dep",
          message: `Declared gate "${declaration.id}" references unknown dep "${dep}".`,
        });
      }
    }
  }
  return diagnostics;
}

function findDeclaredCycleDiagnostics(
  declarations: readonly DeclaredCheckGateV0[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const byId = new Map(declarations.map((gate) => [gate.id, gate]));
  const visiting = new Set<string>();
  const visited = new Set<string>();

  for (const declaration of declarations) {
    visit(declaration, []);
  }

  return diagnostics;

  function visit(declaration: DeclaredCheckGateV0, path: readonly string[]): void {
    if (visited.has(declaration.id)) return;
    if (visiting.has(declaration.id)) {
      diagnostics.push({
        severity: "error",
        code: "declared-gate-cycle",
        message: `Declared gate cycle detected: ${[...path, declaration.id].join(" -> ")}`,
      });
      return;
    }

    visiting.add(declaration.id);
    for (const dep of declaration.deps ?? []) {
      const depDeclaration = byId.get(dep);
      if (depDeclaration) {
        visit(depDeclaration, [...path, declaration.id]);
      }
    }
    visiting.delete(declaration.id);
    visited.add(declaration.id);
  }
}

function resolveDeclaredDependency(gates: readonly CheckGate[], target: string): CheckGate | null {
  return (
    gates.find((gate) => gate.id === target || gate.scriptName === target) ??
    gates.find((gate) => gate.deprecatedAliases?.includes(target)) ??
    gates.find((gate) => gate.id.endsWith(`/${target}`)) ??
    null
  );
}

function findDuplicateValues(values: readonly string[]): readonly string[] {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) {
      duplicates.add(value);
    }
    seen.add(value);
  }
  return [...duplicates].toSorted();
}

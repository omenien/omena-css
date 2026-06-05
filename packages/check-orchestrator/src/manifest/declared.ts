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
    id: "rust/closure-fast",
    kind: "bundle",
    scope: "rust",
    deps: [
      "rust/runtime-query-api-hardening",
      "rust/product-facing-capability",
      "rust/theory-generalization",
      "rust/omena-query/boundary",
      "rust/omena-lsp-server/boundary",
      "rust/omena-cascade/boundary",
      "rust/omena-diff-test-boundary",
      "rust/publish-train-closure",
      "rust/inter-crate-pin",
      "rust/role-boundaries",
      "rust/publish-flags",
      "rust/naming-consistency",
      "rust/no-split-repo-residue",
      "release/check/release-tag-grammar",
      "rust/closure-fast-aggregation-complete",
    ],
    tags: ["closure-fast", "ci-unreachable-allowed"],
    ciTier: "none",
    ciGroup: "closure-fast",
  },
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
  declaredClosurePackageGate("rust/omena-query/boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-lsp-server/boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-cascade/boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-diff-test-boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/publish-train-closure", "gate", "rust"),
  declaredClosurePackageGate("rust/inter-crate-pin", "gate", "rust"),
  declaredClosurePackageGate("rust/role-boundaries", "gate", "rust"),
  declaredClosurePackageGate("rust/publish-flags", "gate", "rust"),
  declaredClosurePackageGate("rust/naming-consistency", "gate", "rust"),
  declaredClosurePackageGate("rust/no-split-repo-residue", "gate", "rust"),
  declaredClosurePackageGate("release/check/release-tag-grammar", "gate", "release"),
  declaredClosurePackageGate("rust/closure-fast-aggregation-complete", "gate", "rust"),
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

function declaredClosurePackageGate(
  id: string,
  kind: DeclaredCheckGateV0["kind"],
  scope: DeclaredCheckGateV0["scope"],
): DeclaredCheckGateV0 {
  return {
    id,
    kind,
    scope,
    packageTarget: id,
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  };
}

export function applyDeclaredPackageMetadata(
  packageGates: readonly CheckGate[],
  declarations: readonly DeclaredCheckGateV0[],
  diagnostics: CheckDiagnostic[],
): readonly CheckGate[] {
  const byScriptName = new Map(packageGates.map((gate) => [gate.scriptName, gate]));

  for (const declaration of declarations) {
    if (!declaration.packageTarget) {
      continue;
    }

    validateDeclaredShape(declaration, diagnostics);
    const packageGate = resolveDeclaredDependency(packageGates, declaration.packageTarget);
    if (!packageGate) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-target-unknown",
        message: `Declared package metadata "${declaration.id}" references unknown package target "${declaration.packageTarget}".`,
      });
      continue;
    }

    if (packageGate.id !== declaration.id) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-target-id-mismatch",
        message: `Declared package metadata "${declaration.id}" points to package gate "${packageGate.id}".`,
      });
      continue;
    }

    byScriptName.set(packageGate.scriptName, mergeDeclaredMetadata(packageGate, declaration));
  }

  return packageGates.map((gate) => byScriptName.get(gate.scriptName) ?? gate);
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
    if (declaration.packageTarget) {
      continue;
    }
    if (packageGateIds.has(declaration.id)) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-gate-id-collision",
        message: `Declared gate id "${declaration.id}" collides with a package-derived gate id.`,
      });
    }
  }

  const executableDeclarations = declarations.filter((declaration) => !declaration.packageTarget);
  const declaredGates = executableDeclarations.map((declaration) =>
    buildDeclaredGate(declaration, diagnostics),
  );
  const allGates = [...packageGates, ...declaredGates];

  diagnostics.push(...findDeclaredDependencyDiagnostics(executableDeclarations, allGates));
  diagnostics.push(...findDeclaredCycleDiagnostics(executableDeclarations));

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
    ...(declaration.timeoutMinutes !== undefined
      ? { timeoutMinutes: declaration.timeoutMinutes }
      : {}),
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

  if (declaration.packageTarget) {
    if (hasCommand || depCount > 0) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-metadata-has-executable",
        message: `Declared package metadata "${declaration.id}" must not define command or deps.`,
      });
    }
    if (declaration.ciTier && !VALID_CI_TIERS.has(declaration.ciTier)) {
      diagnostics.push({
        severity: "error",
        code: "declared-gate-unknown-ci-tier",
        message: `Declared gate "${declaration.id}" uses unknown ciTier "${declaration.ciTier}".`,
      });
    }
    return;
  }

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

function mergeDeclaredMetadata(gate: CheckGate, declaration: DeclaredCheckGateV0): CheckGate {
  return {
    ...gate,
    origin: "package+declared",
    ...(declaration.tags ? { tags: mergeUnique(gate.tags ?? [], declaration.tags) } : {}),
    ...(declaration.timeoutMinutes !== undefined
      ? { timeoutMinutes: declaration.timeoutMinutes }
      : {}),
    ...(declaration.ciTier ? { ciTier: declaration.ciTier } : {}),
    ...(declaration.ciGroup ? { ciGroup: declaration.ciGroup } : {}),
    ...(declaration.deprecatedAliases
      ? {
          deprecatedAliases: mergeUnique(
            gate.deprecatedAliases ?? [],
            declaration.deprecatedAliases,
          ),
        }
      : {}),
  };
}

function mergeUnique(left: readonly string[], right: readonly string[]): readonly string[] {
  return [...new Set([...left, ...right])];
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

export type CheckScopeId =
  | "core"
  | "plugin"
  | "release"
  | "ts7"
  | "tsgo"
  | "rust"
  | "contract"
  | "editor"
  | "test"
  | "workspace"
  | "tooling";

export type CheckGateKind = "command" | "gate" | "bundle" | "alias";
export type CheckGateOrigin = "package" | "declared" | "package+declared";
export type CheckCiTier =
  | "verify"
  | "closure-fast"
  | "rust-workspace"
  | "scheduled"
  | "manual"
  | "none";

export type DeclaredCheckDepV0 =
  | string
  | {
      readonly target: string;
      readonly args?: readonly string[];
    };

export interface DeclaredCheckGateV0 {
  readonly id: string;
  readonly kind: CheckGateKind;
  readonly scope: CheckScopeId;
  readonly command?: readonly string[];
  readonly deps?: readonly DeclaredCheckDepV0[];
  readonly packageTarget?: string;
  readonly replacesPackageTarget?: string;
  readonly tags?: readonly string[];
  readonly timeoutMinutes?: number;
  readonly ciTier?: CheckCiTier;
  readonly ciGroup?: string;
  readonly deprecatedAliases?: readonly string[];
}

export interface CheckTargetRef {
  readonly target: string;
  readonly args?: readonly string[];
}

export interface CheckGate {
  readonly id: string;
  readonly scriptName: string;
  readonly command: string;
  readonly scope: CheckScopeId;
  readonly kind: CheckGateKind;
  readonly origin: CheckGateOrigin;
  readonly commandParts?: readonly string[];
  readonly referencedTargets?: readonly string[];
  readonly referencedTargetSpecs?: readonly CheckTargetRef[];
  readonly referencedScripts: readonly string[];
  readonly tags?: readonly string[];
  readonly timeoutMinutes?: number;
  readonly ciTier?: CheckCiTier;
  readonly ciGroup?: string;
  readonly deprecatedAliases?: readonly string[];
  readonly deprecatedBy?: string;
}

export interface CheckBundle extends CheckGate {
  readonly kind: "bundle" | "alias";
}

export type CheckDiagnosticSeverity = "error" | "warning";

export interface CheckDiagnostic {
  readonly severity: CheckDiagnosticSeverity;
  readonly code: string;
  readonly message: string;
}

export interface CheckManifest {
  readonly rootDir: string;
  readonly gates: readonly CheckGate[];
  readonly bundles: readonly CheckBundle[];
  readonly diagnostics: readonly CheckDiagnostic[];
}

export interface CheckPlanStep {
  readonly id: string;
  readonly scriptName: string;
  readonly scope: CheckScopeId;
  readonly kind: CheckGateKind;
  readonly depth: number;
  readonly referencedScripts: readonly string[];
  readonly repeated: boolean;
  readonly cycle: boolean;
}

export interface CheckPlan {
  readonly target: CheckGate;
  readonly steps: readonly CheckPlanStep[];
}

export interface CheckAliasChain {
  readonly aliasId: string;
  readonly aliasScriptName: string;
  readonly referencedAliasId: string;
  readonly referencedAliasScriptName: string;
  readonly directTargetScripts: readonly string[];
}

export interface CheckBundleSurface {
  readonly id: string;
  readonly scriptName: string;
  readonly scope: CheckScopeId;
  readonly kind: CheckGateKind;
  readonly uniqueLeafCount: number;
  readonly totalStepCount: number;
  readonly repeatedStepCount: number;
  readonly maxDepth: number;
}

export interface CheckSurfaceReport {
  readonly totalGates: number;
  readonly gateCount: number;
  readonly bundleCount: number;
  readonly aliasCount: number;
  readonly commandCount: number;
  readonly aliasChains: readonly CheckAliasChain[];
  readonly largestBundles: readonly CheckBundleSurface[];
}

export interface RootPackageJson {
  readonly name?: string;
  readonly scripts?: Record<string, string>;
}

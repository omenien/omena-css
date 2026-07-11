export {
  buildDeclaredGates,
  buildCheckPlan,
  buildCheckSurfaceReport,
  loadCheckManifest,
  renderCheckInventory,
  renderCheckPlan,
  renderCheckSurfaceReport,
  resolveGateTarget,
  runDoctor,
  type CheckAliasChain,
  type CheckBundle,
  type CheckBundleSurface,
  type CheckCiTier,
  type CheckDiagnostic,
  type CheckGate,
  type CheckGateOrigin,
  type CheckManifest,
  type CheckPlan,
  type CheckPlanStep,
  type CheckScopeId,
  type CheckSurfaceReport,
  type DeclaredCheckGateV0,
} from "./manifest/index";
export { buildAffectedCheckPlan } from "./affected";
export type { AffectedCheckPlan, AffectedCheckReason } from "./affected";
export { CI_PROBE_PROFILES, resolveCiProbeProfile } from "./probes";
export type { CiProbeProfile, CiProbeProfileId } from "./probes";

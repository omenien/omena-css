import "@typespec/json-schema";
import "../engine-napi/main.tsp";

using TypeSpec.JsonSchema;

@jsonSchema
namespace Omena.EngineSdkWorkflow;

model OmenaWorkspaceSnapshotIdV0 {
  value: safeint;
}

enum OmenaSdkResponsePartitionV0 {
  public: "public",
  debug: "debug",
}

enum OmenaErrorClassV0 {
  input: "input",
  workspace: "workspace",
  resolution: "resolution",
  analysis: "analysis",
  transform: "transform",
  unsupported: "unsupported",
  internal: "internal",
  unknownError: "unknown",
}

enum OmenaErrorSeverityV0 {
  error: "error",
  warning: "warning",
  information: "information",
}

enum OmenaErrorRecoverabilityV0 {
  retry: "retry",
  userAction: "user-action",
  notRecoverable: "not-recoverable",
}

model OmenaErrorEvidenceReferenceV0 {
  queryIdentity: string;
  inputIdentity: string;
}

model OmenaErrorContextV0 {
  code: string;
  severity: OmenaErrorSeverityV0;
  recoverability: OmenaErrorRecoverabilityV0;
  evidence?: OmenaErrorEvidenceReferenceV0[];
}

model OmenaErrorV0 {
  class: OmenaErrorClassV0;
  message: string;
  context: OmenaErrorContextV0;
}

model OmenaSdkErrorEnvelopeV0 {
  error: OmenaErrorV0;
}

model OmenaCliResponseEnvelopeV0 {
  schemaVersion: "0";
  product: string;
  configContentDigest?: string;
  payload: unknown;
}

model OmenaSdkSnapshotRequestV0 {
  workspaceRoot: string;
}

model OmenaSdkSnapshotResponseV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
  workspaceRoot: string;
}

model OmenaSdkQueryRequestV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  queryKind: string;
  input?: Omena.EngineNapi.EngineNapiEngineInputV2;
}

model OmenaSdkQueryResponseV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
  payload: unknown;
}

model OmenaSdkDiagnosticsRequestV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  stylePath: string;
  styleSource: string;
}

model OmenaSdkDiagnosticsSummaryV0 {
  schemaVersion: string;
  product: string;
  stylePath: string;
  dialect: string;
  tokenCount: safeint;
  parserErrorCount: safeint;
  classSelectorCount: safeint;
  customPropertyCount: safeint;
  keyframeCount: safeint;
  readySurfaces: string[];
}

model OmenaSdkDiagnosticsResponseV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
  summary: OmenaSdkDiagnosticsSummaryV0;
}

model OmenaSdkDiagnosticsDebugReportV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
  publicResponse: OmenaSdkDiagnosticsResponseV0;
  analysis: unknown;
}

enum OmenaSdkBuildVerificationProfileV0 {
  descriptive: "descriptive",
  strict: "strict",
}

enum OmenaSdkBuildVerificationReasonV0 {
  requiredAxisUnavailable: "requiredAxisUnavailable",
  cascadeEnvironmentUnavailable: "cascadeEnvironmentUnavailable",
  winnerChanged: "winnerChanged",
  observationUnavailable: "observationUnavailable",
  unknownPass: "unknownPass",
  closedWorldEvidenceUnavailable: "closedWorldEvidenceUnavailable",
  decisionCoverageIncomplete: "decisionCoverageIncomplete",
}

model OmenaSdkBuildVerificationEventV0 {
  passId: string;
  reasons: OmenaSdkBuildVerificationReasonV0[];
}

model OmenaSdkBuildVerificationSummaryV0 {
  profileId?: string;
  refusedCount: safeint;
  rolledBackCount: safeint;
  refusalReasons: OmenaSdkBuildVerificationEventV0[];
  rollbackReasons: OmenaSdkBuildVerificationEventV0[];
}

model OmenaSdkBuildRequestV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  stylePath: string;
  styleSource: string;
  passIds: string[];
  verificationProfile?: OmenaSdkBuildVerificationProfileV0;
  context?: Omena.EngineNapi.EngineNapiTransformExecutionContextV0;
}

model OmenaSdkBuildResponseV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
  verification: OmenaSdkBuildVerificationSummaryV0;
  summary: unknown;
}

model OmenaSdkExplainPositionV0 {
  line: int32;
  character: int32;
}

model OmenaSdkExplainRequestV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  stylePath: string;
  position: OmenaSdkExplainPositionV0;
}

model OmenaSdkExplainResponseV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
  report: unknown;
}

enum OmenaWorkspaceSessionOperationV0 {
  diagnostics: "diagnostics",
  format: "format",
  lint: "lint",
  check: "check",
  explain: "explain",
  replaceStyleSources: "replaceStyleSources",
  cancel: "cancel",
  shutdown: "shutdown",
}

model OmenaWorkspaceSessionLimitsV0 {
  deadlineMs: safeint;
  maxResponseBytes: safeint;
}

model OmenaWorkspaceSessionHandshakeRequestV0 {
  protocolVersion: "0";
  workspaceRoot: string;
  configContentDigest?: string;
  styleSources: Omena.EngineNapi.EngineNapiStyleSourceInputV0[];
  limits: OmenaWorkspaceSessionLimitsV0;
}

model OmenaWorkspaceSessionHandshakeResponseV0 {
  protocolVersion: "0";
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
  workspaceRoot: string;
  configContentDigest?: string;
  capabilities: string[];
}

model OmenaWorkspaceSessionRequestV0 {
  requestId: string;
  protocolVersion: "0";
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  operation: OmenaWorkspaceSessionOperationV0;
  limits: OmenaWorkspaceSessionLimitsV0;
  payload?: unknown;
}

model OmenaWorkspaceSessionResponseV0 {
  requestId: string;
  protocolVersion: "0";
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
  ok: boolean;
  payload?: unknown;
  error?: OmenaErrorV0;
}

model OmenaBundlerHostCapabilitiesV0 {
  protocolVersion: "0";
  capabilities: string[];
}

model OmenaBundlerHostResolveModuleRequestV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  stylePath: string;
  styleSources: Omena.EngineNapi.EngineNapiStyleSourceInputV0[];
  packageManifests: Omena.EngineNapi.EngineNapiStylePackageManifestV0[];
}

model OmenaBundlerHostComposesEdgeV0 {
  exportedName: string;
  moduleId: string;
  className: string;
}

model OmenaBundlerHostDiagnosticV0 {
  code: string;
  message: string;
}

model OmenaBundlerHostResolveModuleResponseV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  protocolVersion: "0";
  moduleId: string;
  classMap: Record<string>;
  namedExports: Record<string>;
  typescriptDeclaration: string;
  composesEdges: OmenaBundlerHostComposesEdgeV0[];
  diagnostics: OmenaBundlerHostDiagnosticV0[];
  ready: boolean;
}

model OmenaModuleInstanceKeyV0 {
  module: string;
  configuration: string;
}

model OmenaClosedWorldEmptyEntrypointsBlockerV0 {
  kind: "emptyEntrypoints";
}

model OmenaClosedWorldMissingEntrypointBlockerV0 {
  kind: "missingEntrypoint";
  sourcePath: string;
}

model OmenaClosedWorldAmbiguousModulePathBlockerV0 {
  kind: "ambiguousModulePath";
  sourcePath: string;
}

model OmenaClosedWorldMissingDependencyBlockerV0 {
  kind: "missingDependency";
  sourcePath: string;
  importSource: string;
}

model OmenaClosedWorldMissingModuleInstanceBlockerV0 {
  kind: "missingModuleInstance";
  module: OmenaModuleInstanceKeyV0;
}

model OmenaClosedWorldMissingModuleDependencyBlockerV0 {
  kind: "missingModuleDependency";
  module: OmenaModuleInstanceKeyV0;
  dependency: OmenaModuleInstanceKeyV0;
}

model OmenaClosedWorldPassUnavailableBlockerV0 {
  kind: "closedWorldPassUnavailable";
  requestedPassIds: string[];
}

union OmenaClosedWorldBlockerV0 {
  emptyEntrypoints: OmenaClosedWorldEmptyEntrypointsBlockerV0,
  missingEntrypoint: OmenaClosedWorldMissingEntrypointBlockerV0,
  ambiguousModulePath: OmenaClosedWorldAmbiguousModulePathBlockerV0,
  missingDependency: OmenaClosedWorldMissingDependencyBlockerV0,
  missingModuleInstance: OmenaClosedWorldMissingModuleInstanceBlockerV0,
  missingModuleDependency: OmenaClosedWorldMissingModuleDependencyBlockerV0,
  closedWorldPassUnavailable: OmenaClosedWorldPassUnavailableBlockerV0,
}

model OmenaModuleQualifiedSymbolSetV0 {
  moduleInstance: OmenaModuleInstanceKeyV0;
  reachable: boolean;
  classNames: string[];
  keyframeNames: string[];
  valueNames: string[];
  customPropertyNames: string[];
}

model OmenaClosedWorldReachabilityIndexV0 {
  moduleInstances: OmenaModuleInstanceKeyV0[];
  moduleQualifiedSymbols: OmenaModuleQualifiedSymbolSetV0[];
  classNames: string[];
  keyframeNames: string[];
  valueNames: string[];
  customPropertyNames: string[];
}

model OmenaClosedWorldInterfaceHashKnownV0 {
  status: "known";
  interfaceHash: string;
}

model OmenaClosedWorldInterfaceHashAbsentV0 {
  status: "absent";
}

union OmenaClosedWorldInterfaceHashAvailabilityV0 {
  known: OmenaClosedWorldInterfaceHashKnownV0,
  absent: OmenaClosedWorldInterfaceHashAbsentV0,
}

model OmenaClosedWorldInterfaceHashEntryV0 {
  moduleInstance: OmenaModuleInstanceKeyV0;
  availability: OmenaClosedWorldInterfaceHashAvailabilityV0;
}

model OmenaClosedWorldInterfaceHashSetV0 {
  entries: OmenaClosedWorldInterfaceHashEntryV0[];
}

model OmenaClosedWorldSourcePrecisionSummaryV0 {
  exactSourceCount: safeint;
  conservativeSourceCount: safeint;
  heuristicSourceCount: safeint;
  unknownSourceCount: safeint;
}

model OmenaClosedWorldBundleV0 {
  entrypoints: OmenaModuleInstanceKeyV0[];
  linkedModules: OmenaModuleInstanceKeyV0[];
  reachability: OmenaClosedWorldReachabilityIndexV0;
  closureHash: string;
  interfaceHashes?: OmenaClosedWorldInterfaceHashSetV0;
  sourcePrecision?: OmenaClosedWorldSourcePrecisionSummaryV0;
}

model OmenaClosedWorldClosedOutcomeV0 {
  status: "closed";
  bundle: OmenaClosedWorldBundleV0;
}

model OmenaClosedWorldOpenOutcomeV0 {
  status: "open";
  blockers: OmenaClosedWorldBlockerV0[];
}

union OmenaClosedWorldOutcomeV0 {
  closed: OmenaClosedWorldClosedOutcomeV0,
  open: OmenaClosedWorldOpenOutcomeV0,
}

model OmenaClosedWorldDecisionParityV0 {
  legacyOpenDecision: boolean;
  typedOutcomeOpen: boolean;
  equivalent: boolean;
}

model OmenaBundleEvidenceGateV0 {
  name: string;
  passed: boolean;
}

union OmenaGuaranteeKindV0 {
  floor: "floor",
  sampledFixtureWitness: "sampledFixtureWitness",
  schedulerPriorityFixtureWitness: "schedulerPriorityFixtureWitness",
  metricInputFixtureWitness: "metricInputFixtureWitness",
  incrementalLayerEvidenceOnly: "incrementalLayerEvidenceOnly",
  alphaRenamingStableHashFixtureWitness: "alphaRenamingStableHashFixtureWitness",
  notClaimedExactTraversal: "notClaimedExactTraversal",
}

model OmenaBundleReachabilityEvidenceV0 {
  guarantee: OmenaGuaranteeKindV0;
  interpretation: string;
  moduleInstances: OmenaModuleInstanceKeyV0[];
  closureHash: string;
}

model OmenaBundleEvidenceManifestV0 {
  schemaVersion: "0";
  product: "omena-query.bundle-evidence";
  stylePath: string;
  outcomeStatus: "closed" | "open";
  reachability: OmenaBundleReachabilityEvidenceV0 | null;
  gates: OmenaBundleEvidenceGateV0[];
  blockers: OmenaClosedWorldBlockerV0[];
  interfaceHashes: OmenaClosedWorldInterfaceHashEntryV0[];
  sourcePrecision: OmenaClosedWorldSourcePrecisionSummaryV0 | null;
}

model OmenaBundlerHostBundleAdmissionV0 {
  closedWorldOutcome: OmenaClosedWorldOutcomeV0;
  closedWorldDecisionParity: OmenaClosedWorldDecisionParityV0;
  evidence: OmenaBundleEvidenceManifestV0;
}

model OmenaBundlerHostPackageContractV0 {
  resolveModuleResponse: OmenaBundlerHostResolveModuleResponseV0;
  bundleAdmission: OmenaBundlerHostBundleAdmissionV0;
}

model OmenaSdkWorkflowSurfaceV0 {
  errorEnvelope: OmenaSdkErrorEnvelopeV0;
  cliResponseEnvelope: OmenaCliResponseEnvelopeV0;
  snapshotRequest: OmenaSdkSnapshotRequestV0;
  snapshotResponse: OmenaSdkSnapshotResponseV0;
  queryRequest: OmenaSdkQueryRequestV0;
  queryResponse: OmenaSdkQueryResponseV0;
  diagnosticsRequest: OmenaSdkDiagnosticsRequestV0;
  diagnosticsResponse: OmenaSdkDiagnosticsResponseV0;
  diagnosticsDebugReport: OmenaSdkDiagnosticsDebugReportV0;
  buildRequest: OmenaSdkBuildRequestV0;
  buildResponse: OmenaSdkBuildResponseV0;
  explainRequest: OmenaSdkExplainRequestV0;
  explainResponse: OmenaSdkExplainResponseV0;
  workspaceSessionHandshakeRequest: OmenaWorkspaceSessionHandshakeRequestV0;
  workspaceSessionHandshakeResponse: OmenaWorkspaceSessionHandshakeResponseV0;
  workspaceSessionRequest: OmenaWorkspaceSessionRequestV0;
  workspaceSessionResponse: OmenaWorkspaceSessionResponseV0;
  bundlerHostCapabilities: OmenaBundlerHostCapabilitiesV0;
  bundlerHostResolveModuleRequest: OmenaBundlerHostResolveModuleRequestV0;
  bundlerHostResolveModuleResponse: OmenaBundlerHostResolveModuleResponseV0;
  bundlerHostBundleAdmission: OmenaBundlerHostBundleAdmissionV0;
}

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

model OmenaSdkBuildRequestV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  stylePath: string;
  styleSource: string;
  passIds: string[];
  context?: Omena.EngineNapi.EngineNapiTransformExecutionContextV0;
}

model OmenaSdkBuildResponseV0 {
  snapshotId: OmenaWorkspaceSnapshotIdV0;
  partition: OmenaSdkResponsePartitionV0;
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

model OmenaSdkWorkflowSurfaceV0 {
  snapshotRequest: OmenaSdkSnapshotRequestV0;
  snapshotResponse: OmenaSdkSnapshotResponseV0;
  queryRequest: OmenaSdkQueryRequestV0;
  queryResponse: OmenaSdkQueryResponseV0;
  diagnosticsRequest: OmenaSdkDiagnosticsRequestV0;
  diagnosticsResponse: OmenaSdkDiagnosticsResponseV0;
  buildRequest: OmenaSdkBuildRequestV0;
  buildResponse: OmenaSdkBuildResponseV0;
  explainRequest: OmenaSdkExplainRequestV0;
  explainResponse: OmenaSdkExplainResponseV0;
}

import "@typespec/json-schema";

using TypeSpec.JsonSchema;

@jsonSchema
namespace Omena.ExternalCorpusEnvelope;

enum ExternalCorpusStageV1 {
  stage1Advisory: "stage1-advisory",
  stage2Blocking: "stage2-blocking",
}

enum ExternalCorpusDialectV1 {
  css: "css",
  scss: "scss",
  sass: "sass",
  less: "less",
}

enum ExternalCorpusExpectationKindV1 {
  staticMustMatch: "static-must-match",
  expectedSoundBail: "expected-sound-bail",
  parserRecovery: "parser-recovery",
  outOfScope: "out-of-scope",
  findingCensus: "finding-census",
}

model ExternalCorpusSourcePinV1 {
  kind: "pinned-repository";
  repository: string;
  pin: string;
  sparsePaths: string[];
  helperClasses: string[];
  layoutDependentHelpersExcluded: string[];
}

model ExternalCorpusLocalWorkspaceSourceV1 {
  kind: "local-workspace";
  workspacePath: string;
}

@oneOf
union ExternalCorpusSourceV1 {
  pinnedRepository: ExternalCorpusSourcePinV1,
  localWorkspace: ExternalCorpusLocalWorkspaceSourceV1,
}

model ExternalCorpusKnownFailurePolicyRefV1 {
  path: string;
  schemaVersion: string;
  stage2Blocking: boolean;
}

model ExternalCorpusGenerationProvenanceV1 {
  tool: string;
  selectionPath: string;
  oraclePinRefs?: string[];
}

model ExternalCorpusSparsePathFixtureCountV1 {
  sparsePath: string;
  fixtureCount: int32;
}

model ExternalCorpusChunkV1 {
  chunkId: string;
  path: string;
  stage: ExternalCorpusStageV1;
  sha256: string;
  fixtureCount: int32;
  sparsePathFixtureCounts?: ExternalCorpusSparsePathFixtureCountV1[];
}

model ExternalCorpusProvenanceV1 {
  generationTool: string;
  selectionPath: string;
  oraclePinRefs: string[];
}

model ExternalCorpusGreenRunV1 {
  date: string;
  commit: string;
  fixtureCount: int32;
  chunkSha256: string;
  outcomeOlw?: int32;
  criticalRegressionCount: int32;
  command: string;
}

model ExternalCorpusEnvelopeV1 {
  schemaVersion: string;
  product: string;
  stage: ExternalCorpusStageV1;
  dialect?: ExternalCorpusDialectV1;
  expectationKind?: ExternalCorpusExpectationKindV1;
  source: ExternalCorpusSourceV1;
  knownFailurePolicy?: ExternalCorpusKnownFailurePolicyRefV1;
  generation: ExternalCorpusGenerationProvenanceV1;
  provenance?: ExternalCorpusProvenanceV1;
  sparsePathFixtureCounts?: ExternalCorpusSparsePathFixtureCountV1[];
  chunks: ExternalCorpusChunkV1[];
  greenRuns?: ExternalCorpusGreenRunV1[];
}

model ExternalCorpusDifferentialManifestV1 {
  schemaVersion: string;
  product: string;
  mode: string;
  lastVerified?: string;
  compilers?: unknown;
  fixtures: unknown[];
}

model ExternalCorpusDisagreementLedgerV1 {
  schemaVersion: string;
  product: string;
  allowlistCount: int32;
  entries: unknown[];
}

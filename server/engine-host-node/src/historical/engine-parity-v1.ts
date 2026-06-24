import type {
  CheckerReportV1,
  EngineInputV1,
  EngineOutputV1,
} from "../../../engine-core-ts/src/contracts";
import { downcastEngineOutputV2ToV1 } from "../../../engine-core-ts/src/contracts";
import { buildEngineInputV1Async, type BuildEngineInputV1Options } from "./engine-input-v1";
import { buildSelectedQueryResultsV2 } from "../engine-query-v2";
import { buildEngineOutputV2 } from "../engine-output-v2";
import type { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../engine-core-ts/src/core/semantic/workspace-reference-index";
import type { WorkspaceStyleDependencyGraph } from "../../../engine-core-ts/src/core/semantic/style-dependency-graph";
import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import type { DocumentAnalysisCache } from "../../../engine-core-ts/src/core/indexing/document-analysis-cache";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import type { SourceDocumentSnapshot } from "../checker-host/workspace-check-support";

export interface EngineParitySnapshotV1 {
  readonly input: EngineInputV1;
  readonly output: EngineOutputV1;
}

export interface BuildCheckerEngineParitySnapshotV1Options extends BuildEngineInputV1Options {
  readonly checkerReport: CheckerReportV1;
  readonly sourceDocuments: readonly SourceDocumentSnapshot[];
  readonly styleFiles: readonly string[];
  readonly analysisCache: DocumentAnalysisCache;
  readonly styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null;
  readonly typeResolver: TypeResolver;
  readonly semanticReferenceIndex: WorkspaceSemanticWorkspaceReferenceIndex;
  readonly styleDependencyGraph: WorkspaceStyleDependencyGraph;
}

export async function buildCheckerEngineParitySnapshotV1(
  options: BuildCheckerEngineParitySnapshotV1Options,
): Promise<EngineParitySnapshotV1> {
  const outputV2 = buildEngineOutputV2({
    checkerReport: options.checkerReport,
    queryResults: buildSelectedQueryResultsV2({
      workspaceRoot: options.workspaceRoot,
      classnameTransform: options.classnameTransform,
      pathAlias: options.pathAlias,
      sourceDocuments: options.sourceDocuments,
      styleFiles: options.styleFiles,
      analysisCache: options.analysisCache,
      styleDocumentForPath: options.styleDocumentForPath,
      typeResolver: options.typeResolver,
      semanticReferenceIndex: options.semanticReferenceIndex,
      styleDependencyGraph: options.styleDependencyGraph,
    }),
  });

  return {
    input: await buildEngineInputV1Async(options),
    output: downcastEngineOutputV2ToV1(outputV2),
  };
}

import type {
  CheckerReportV1,
  EngineInputV2,
  EngineOutputV2,
} from "../../engine-core-ts/src/contracts";
import { buildEngineInputV2Async, type BuildEngineInputV2Options } from "./engine-input-v2";
import { buildSelectedQueryResultsV2 } from "./engine-query-v2";
import { buildEngineOutputV2 } from "./engine-output-v2";
import type { WorkspaceSemanticWorkspaceReferenceIndex } from "../../engine-core-ts/src/core/semantic/workspace-reference-index";
import type { WorkspaceStyleDependencyGraph } from "../../engine-core-ts/src/core/semantic/style-dependency-graph";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import type { DocumentAnalysisCache } from "../../engine-core-ts/src/core/indexing/document-analysis-cache";
import type { TypeResolver } from "../../engine-core-ts/src/core/ts/type-resolver";
import type { SourceDocumentSnapshot } from "./checker-host/workspace-check-support";

export interface EngineParitySnapshotV2 {
  readonly input: EngineInputV2;
  readonly output: EngineOutputV2;
}

export interface BuildCheckerEngineParitySnapshotV2Options extends BuildEngineInputV2Options {
  readonly checkerReport: CheckerReportV1;
  readonly sourceDocuments: readonly SourceDocumentSnapshot[];
  readonly styleFiles: readonly string[];
  readonly analysisCache: DocumentAnalysisCache;
  readonly styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null;
  readonly typeResolver: TypeResolver;
  readonly semanticReferenceIndex: WorkspaceSemanticWorkspaceReferenceIndex;
  readonly styleDependencyGraph: WorkspaceStyleDependencyGraph;
}

export async function buildCheckerEngineParitySnapshotV2(
  options: BuildCheckerEngineParitySnapshotV2Options,
): Promise<EngineParitySnapshotV2> {
  return {
    input: await buildEngineInputV2Async(options),
    output: buildEngineOutputV2({
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
    }),
  };
}

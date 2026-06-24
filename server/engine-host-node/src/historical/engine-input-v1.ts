import type {
  EngineInputV1,
  SourceAnalysisInputV1,
  StyleAnalysisInputV1,
} from "../../../engine-core-ts/src/contracts";
import {
  ENGINE_CONTRACT_VERSION_V1,
  buildSourceBindingGraphSnapshotV1,
} from "../../../engine-core-ts/src/contracts";
import type { DocumentAnalysisCache } from "../../../engine-core-ts/src/core/indexing/document-analysis-cache";
import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import type { ClassnameTransformMode } from "../../../engine-core-ts/src/core/scss/classname-transform";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import {
  workspaceSettingsKey,
  type SourceDocumentSnapshot,
} from "../checker-host/workspace-check-support";
import { selectTypeFactCollector } from "../type-fact-collector";
import type { TypeFactBackendKind } from "../type-backend";

export interface BuildEngineInputV1Options {
  readonly workspaceRoot: string;
  readonly classnameTransform: ClassnameTransformMode;
  readonly pathAlias: Readonly<Record<string, string>>;
  readonly sourceDocuments: readonly SourceDocumentSnapshot[];
  readonly styleFiles: readonly string[];
  readonly analysisCache: DocumentAnalysisCache;
  readonly styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null;
  readonly typeResolver?: TypeResolver;
  readonly typeBackend?: TypeFactBackendKind;
  readonly env?: NodeJS.ProcessEnv;
}

export function buildEngineInputV1(options: BuildEngineInputV1Options): EngineInputV1 {
  const sourceEntries = collectEngineInputSourceEntries(options);
  const { sources, styles, typeFactCollector } = buildEngineInputScaffold(options, sourceEntries);
  const typeFacts = typeFactCollector.collectV1({
    workspaceRoot: options.workspaceRoot,
    sourceEntries,
  });

  return finishEngineInputV1(options, sources, styles, typeFacts);
}

export async function buildEngineInputV1Async(
  options: BuildEngineInputV1Options,
): Promise<EngineInputV1> {
  const sourceEntries = collectEngineInputSourceEntries(options);
  const { sources, styles, typeFactCollector } = buildEngineInputScaffold(options, sourceEntries);
  const typeFacts = await typeFactCollector.collectV1Async({
    workspaceRoot: options.workspaceRoot,
    sourceEntries,
  });

  return finishEngineInputV1(options, sources, styles, typeFacts);
}

function collectEngineInputSourceEntries(options: BuildEngineInputV1Options) {
  return options.sourceDocuments.map((document) => ({
    document,
    analysis: options.analysisCache.get(
      document.uri,
      document.content,
      document.filePath,
      document.version,
    ),
  }));
}

function buildEngineInputScaffold(
  options: BuildEngineInputV1Options,
  sourceEntries: ReturnType<typeof collectEngineInputSourceEntries>,
) {
  const sources: SourceAnalysisInputV1[] = sourceEntries.map(({ document, analysis }) => ({
    filePath: document.filePath,
    document: analysis.sourceDocument,
    bindingGraph: buildSourceBindingGraphSnapshotV1(analysis.sourceBindingGraph),
  }));

  const styles: StyleAnalysisInputV1[] = options.styleFiles.flatMap((filePath) => {
    const document = options.styleDocumentForPath(filePath);
    return document ? [{ filePath, document }] : [];
  });

  const typeFactCollector = selectTypeFactCollector({
    ...(options.typeResolver ? { typeResolver: options.typeResolver } : {}),
    ...(options.typeBackend ? { typeBackend: options.typeBackend } : {}),
    ...(options.env ? { env: options.env } : {}),
  });
  return { sources, styles, typeFactCollector };
}

function finishEngineInputV1(
  options: BuildEngineInputV1Options,
  sources: readonly SourceAnalysisInputV1[],
  styles: readonly StyleAnalysisInputV1[],
  typeFacts: EngineInputV1["typeFacts"],
): EngineInputV1 {
  return {
    version: ENGINE_CONTRACT_VERSION_V1,
    workspace: {
      root: options.workspaceRoot,
      classnameTransform: options.classnameTransform,
      settingsKey: workspaceSettingsKey(options.classnameTransform, options.pathAlias),
    },
    sources,
    styles,
    typeFacts,
  };
}

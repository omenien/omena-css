import type {
  EngineInputV2,
  SourceAnalysisInputV1,
  StyleAnalysisInputV1,
} from "../../engine-core-ts/src/contracts";
import {
  ENGINE_CONTRACT_VERSION_V2,
  buildSourceBindingGraphSnapshotV1,
} from "../../engine-core-ts/src/contracts";
import type { DocumentAnalysisCache } from "../../engine-core-ts/src/core/indexing/document-analysis-cache";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import type { ClassnameTransformMode } from "../../engine-core-ts/src/core/scss/classname-transform";
import type { TypeResolver } from "../../engine-core-ts/src/core/ts/type-resolver";
import {
  workspaceSettingsKey,
  type SourceDocumentSnapshot,
} from "./checker-host/workspace-check-support";
import { selectTypeFactCollector } from "./type-fact-collector";
import type { TypeFactBackendKind } from "./type-backend";

export interface BuildEngineInputV2Options {
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

export function buildEngineInputV2(options: BuildEngineInputV2Options): EngineInputV2 {
  const sourceEntries = collectEngineInputSourceEntries(options);
  const { sources, styles, typeFactCollector } = buildEngineInputScaffold(options, sourceEntries);
  const typeFacts = typeFactCollector.collectV2({
    workspaceRoot: options.workspaceRoot,
    sourceEntries,
  });

  return finishEngineInputV2(options, sources, styles, typeFacts);
}

export async function buildEngineInputV2Async(
  options: BuildEngineInputV2Options,
): Promise<EngineInputV2> {
  const sourceEntries = collectEngineInputSourceEntries(options);
  const { sources, styles, typeFactCollector } = buildEngineInputScaffold(options, sourceEntries);
  const typeFacts = await typeFactCollector.collectV2Async({
    workspaceRoot: options.workspaceRoot,
    sourceEntries,
  });

  return finishEngineInputV2(options, sources, styles, typeFacts);
}

function collectEngineInputSourceEntries(options: BuildEngineInputV2Options) {
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
  options: BuildEngineInputV2Options,
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

function finishEngineInputV2(
  options: BuildEngineInputV2Options,
  sources: readonly SourceAnalysisInputV1[],
  styles: readonly StyleAnalysisInputV1[],
  typeFacts: EngineInputV2["typeFacts"],
): EngineInputV2 {
  return {
    version: ENGINE_CONTRACT_VERSION_V2,
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

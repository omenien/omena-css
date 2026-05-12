import {
  StyleIndexCache,
  type StyleDocumentBuilder,
} from "../../../engine-core-ts/src/core/scss/scss-index";
import { SourceFileCache } from "../../../engine-core-ts/src/core/ts/source-file-cache";
import {
  WorkspaceSemanticWorkspaceReferenceIndex,
  WorkspaceStyleDependencyGraph,
} from "../../../engine-core-ts/src/core/semantic";
import type {
  StyleSemanticGraphBatchOutputCache,
  StyleSemanticGraphCache,
} from "../style-semantic-graph-query-backend";
import type { SelectorUsagePayloadCache } from "../selector-usage-query-backend";

export interface SharedRuntimeCaches {
  readonly sourceFileCache: SourceFileCache;
  readonly styleIndexCache: StyleIndexCache;
  readonly semanticReferenceIndex: WorkspaceSemanticWorkspaceReferenceIndex;
  readonly styleDependencyGraph: WorkspaceStyleDependencyGraph;
  readonly styleSemanticGraphCache: StyleSemanticGraphCache;
  readonly styleSemanticGraphBatchOutputCache: StyleSemanticGraphBatchOutputCache;
  readonly selectorUsagePayloadCache: SelectorUsagePayloadCache;
}

export interface BuildSharedRuntimeCachesOptions {
  readonly buildStyleDocument?: StyleDocumentBuilder;
}

export function buildSharedRuntimeCaches(
  options: BuildSharedRuntimeCachesOptions = {},
): SharedRuntimeCaches {
  return {
    sourceFileCache: new SourceFileCache({ max: 200 }),
    styleIndexCache: new StyleIndexCache({
      max: 500,
      ...(options.buildStyleDocument ? { buildStyleDocument: options.buildStyleDocument } : {}),
    }),
    semanticReferenceIndex: new WorkspaceSemanticWorkspaceReferenceIndex(),
    styleDependencyGraph: new WorkspaceStyleDependencyGraph(),
    styleSemanticGraphCache: new Map(),
    styleSemanticGraphBatchOutputCache: new Map(),
    selectorUsagePayloadCache: new Map(),
  };
}

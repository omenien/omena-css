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

export interface PackageManifestTextCache {
  read(filePath: string, readFile: (filePath: string) => string | null): string | null;
  invalidate(filePath: string): void;
  clear(): void;
}

export interface SharedRuntimeCaches {
  readonly sourceFileCache: SourceFileCache;
  readonly styleIndexCache: StyleIndexCache;
  readonly semanticReferenceIndex: WorkspaceSemanticWorkspaceReferenceIndex;
  readonly styleDependencyGraph: WorkspaceStyleDependencyGraph;
  readonly styleSemanticGraphCache: StyleSemanticGraphCache;
  readonly styleSemanticGraphBatchOutputCache: StyleSemanticGraphBatchOutputCache;
  readonly selectorUsagePayloadCache: SelectorUsagePayloadCache;
  readonly packageManifestTextCache: PackageManifestTextCache;
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
    packageManifestTextCache: new RuntimePackageManifestTextCache(),
  };
}

export function createManifestCachedStyleFileReader(
  caches: Pick<SharedRuntimeCaches, "packageManifestTextCache">,
  readStyleFile: (filePath: string) => string | null,
): (filePath: string) => string | null {
  return (filePath) =>
    isPackageManifestPath(filePath)
      ? caches.packageManifestTextCache.read(filePath, readStyleFile)
      : readStyleFile(filePath);
}

export function isPackageManifestPath(filePath: string): boolean {
  return filePath.split(/[\\/]/u).pop() === "package.json";
}

class RuntimePackageManifestTextCache implements PackageManifestTextCache {
  private readonly entries = new Map<string, string | null>();

  read(filePath: string, readFile: (filePath: string) => string | null): string | null {
    if (this.entries.has(filePath)) return this.entries.get(filePath) ?? null;
    const text = readFile(filePath);
    this.entries.set(filePath, text);
    return text;
  }

  invalidate(filePath: string): void {
    this.entries.delete(filePath);
  }

  clear(): void {
    this.entries.clear();
  }
}

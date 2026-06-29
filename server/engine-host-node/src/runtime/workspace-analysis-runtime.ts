import type { AliasResolver } from "../../../engine-core-ts/src/core/cx/alias-resolver";
import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import { DocumentAnalysisCache } from "../../../engine-core-ts/src/core/indexing/document-analysis-cache";
import { collectSemanticReferenceContribution } from "../../../engine-core-ts/src/core/semantic";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import { fileUrlToPath } from "../../../engine-core-ts/src/core/util/text-utils";
import type { SharedRuntimeCaches } from "./shared-runtime-caches";
import { createRequiredRustSourceFrontendAnalysisProvider } from "../source-frontend-analysis-provider";
import { resolveSymbolValuesFromRustControlFlowWithTypescriptFallback } from "../type-fact-control-flow-graph";

export interface WorkspaceAnalysisRuntimeArgs {
  readonly caches: SharedRuntimeCaches;
  readonly typeResolver: TypeResolver;
  readonly workspaceRoot: string;
  readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
  readonly fileExists: (path: string) => boolean;
  readonly aliasResolver: () => AliasResolver;
  readonly settingsKey: () => string;
  readonly onReferencesChanged: () => void;
}

export function createWorkspaceAnalysisCache(
  args: WorkspaceAnalysisRuntimeArgs,
): DocumentAnalysisCache {
  const sourceFrontendAnalysis = createRequiredRustSourceFrontendAnalysisProvider({
    aliasResolver: () => args.aliasResolver(),
    fileExists: args.fileExists,
  });
  return new DocumentAnalysisCache({
    sourceFileCache: args.caches.sourceFileCache,
    sourceFrontendAnalysis,
    fileExists: args.fileExists,
    get aliasResolver(): AliasResolver {
      return args.aliasResolver();
    },
    max: 200,
    onAnalyze: (uri, entry) => {
      const semanticContribution = collectSemanticReferenceContribution(uri, entry, {
        styleDocumentForPath: args.styleDocumentForPath,
        typeResolver: args.typeResolver,
        workspaceRoot: args.workspaceRoot,
        filePath: fileUrlToPath(uri),
        settingsKey: args.settingsKey(),
        resolveSymbolValues: (expression) =>
          resolveSymbolValuesFromRustControlFlowWithTypescriptFallback({
            source: entry.sourceFile.text,
            sourcePath: fileUrlToPath(uri),
            expression,
          }),
      });
      args.caches.semanticReferenceIndex.record(
        uri,
        semanticContribution.referenceSites,
        semanticContribution.moduleUsages,
        semanticContribution.deps,
      );
      args.onReferencesChanged();
    },
  });
}

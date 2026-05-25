import type { AliasResolver } from "../../../engine-core-ts/src/core/cx/alias-resolver";
import { cssModulesClassnamesBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/binder-plugin";
import { cvaRecipeBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/cva-recipe-plugin";
import { tailwindUnoUtilityBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/tailwind-utility-plugin";
import { vanillaExtractRecipeBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/vanilla-extract-recipe-plugin";
import { vueStyleModuleBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/vue-style-module-plugin";
import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import { DocumentAnalysisCache } from "../../../engine-core-ts/src/core/indexing/document-analysis-cache";
import { collectSemanticReferenceContribution } from "../../../engine-core-ts/src/core/semantic";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import { fileUrlToPath } from "../../../engine-core-ts/src/core/util/text-utils";
import type { SharedRuntimeCaches } from "./shared-runtime-caches";

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
  return new DocumentAnalysisCache({
    sourceFileCache: args.caches.sourceFileCache,
    binderPlugins: [
      cssModulesClassnamesBinderPluginV0,
      tailwindUnoUtilityBinderPluginV0,
      vanillaExtractRecipeBinderPluginV0,
      cvaRecipeBinderPluginV0,
      vueStyleModuleBinderPluginV0,
    ],
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

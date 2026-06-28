import { existsSync, readFileSync } from "node:fs";
import fastGlob from "fast-glob";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import {
  buildStyleFileWatcherGlob,
  findLangForPath,
} from "../../../engine-core-ts/src/core/scss/lang-registry";
import { StyleIndexCache } from "../../../engine-core-ts/src/core/scss/scss-index";
import type { ClassnameTransformMode } from "../../../engine-core-ts/src/core/scss/classname-transform";
import {
  AliasResolver,
  loadWorkspaceBundlerPathAliases,
  loadWorkspaceTsconfigPathAliases,
} from "../../../engine-core-ts/src/core/cx/alias-resolver";
import { cssModulesClassnamesBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/binder-plugin";
import { cvaRecipeBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/cva-recipe-plugin";
import { tailwindUnoUtilityBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/tailwind-utility-plugin";
import { vanillaExtractRecipeBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/vanilla-extract-recipe-plugin";
import { vueStyleModuleBinderPluginV0 } from "../../../engine-core-ts/src/core/binder/vue-style-module-plugin";
import { DocumentAnalysisCache } from "../../../engine-core-ts/src/core/indexing/document-analysis-cache";
import { collectSemanticReferenceContribution } from "../../../engine-core-ts/src/core/semantic/reference-collector";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../engine-core-ts/src/core/semantic/workspace-reference-index";
import { WorkspaceStyleDependencyGraph } from "../../../engine-core-ts/src/core/semantic/style-dependency-graph";
import { SourceFileCache } from "../../../engine-core-ts/src/core/ts/source-file-cache";
import { type TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import { selectTypeResolver, type TypeFactBackendKind } from "../type-backend";
import {
  createDefaultRustSourceFrontendAnalysisProvider,
  resolveSourceFrontendBackendKind,
} from "../source-frontend-analysis-provider";
import { resolveSymbolValuesFromRustControlFlow } from "../type-fact-control-flow-graph";

const SOURCE_GLOB = "**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs}";
const DEFAULT_IGNORES = ["**/node_modules/**", "**/dist/**", "**/.git/**"] as const;

export interface WorkspaceCheckResolvedFiles {
  readonly sourceFiles: readonly string[];
  readonly styleFiles: readonly string[];
}

export interface WorkspaceStyleHost {
  readonly styleDependencyGraph: WorkspaceStyleDependencyGraph;
  readonly readStyleFile: (filePath: string) => string | null;
  readonly styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null;
  preloadStyleDocuments(): void;
}

export interface WorkspaceAnalysisHost {
  readonly analysisCache: DocumentAnalysisCache;
  readonly semanticReferenceIndex: WorkspaceSemanticWorkspaceReferenceIndex;
  readonly typeResolver: TypeResolver;
  readonly typeBackend: TypeFactBackendKind;
}

export interface SourceDocumentSnapshot {
  readonly uri: string;
  readonly filePath: string;
  readonly content: string;
  readonly version: number;
}

export async function resolveWorkspaceCheckFiles(params: {
  readonly workspaceRoot: string;
  readonly sourceFilePaths?: readonly string[];
  readonly styleFilePaths?: readonly string[];
}): Promise<WorkspaceCheckResolvedFiles> {
  const sourceFiles = (
    params.sourceFilePaths ?? (await collectWorkspaceFiles(params.workspaceRoot, SOURCE_GLOB))
  ).toSorted();
  const styleFiles = (
    params.styleFilePaths ??
    (await collectWorkspaceFiles(params.workspaceRoot, buildStyleFileWatcherGlob()))
  )
    .filter((filePath) => findLangForPath(filePath) !== null)
    .toSorted();

  return {
    sourceFiles,
    styleFiles,
  };
}

export function resolveWorkspaceCheckFilesSync(params: {
  readonly workspaceRoot: string;
  readonly sourceFilePaths?: readonly string[];
  readonly styleFilePaths?: readonly string[];
}): WorkspaceCheckResolvedFiles {
  const sourceFiles = (
    params.sourceFilePaths ?? collectWorkspaceFilesSync(params.workspaceRoot, SOURCE_GLOB)
  ).toSorted();
  const styleFiles = (
    params.styleFilePaths ??
    collectWorkspaceFilesSync(params.workspaceRoot, buildStyleFileWatcherGlob())
  )
    .filter((filePath) => findLangForPath(filePath) !== null)
    .toSorted();

  return {
    sourceFiles,
    styleFiles,
  };
}

export function createWorkspaceStyleHost(params: {
  readonly styleFiles: readonly string[];
  readonly classnameTransform: ClassnameTransformMode;
}): WorkspaceStyleHost {
  const styleIndexCache = new StyleIndexCache({ max: 1000 });
  const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
  const styleContents = new Map<string, string>();

  const readStyleFile = (filePath: string): string | null => {
    const cached = styleContents.get(filePath);
    if (cached !== undefined) return cached;
    try {
      const content = readFileSync(filePath, "utf8");
      styleContents.set(filePath, content);
      return content;
    } catch {
      return null;
    }
  };

  const styleDocumentForPath = (filePath: string): StyleDocumentHIR | null => {
    if (!findLangForPath(filePath)) return null;
    const content = readStyleFile(filePath);
    if (content === null) return null;
    const styleDocument = styleIndexCache.getStyleDocument(
      filePath,
      content,
      params.classnameTransform,
    );
    styleDependencyGraph.record(filePath, styleDocument);
    return styleDocument;
  };

  return {
    readStyleFile,
    styleDependencyGraph,
    styleDocumentForPath,
    preloadStyleDocuments(): void {
      for (const styleFile of params.styleFiles) {
        styleDocumentForPath(styleFile);
      }
    },
  };
}

export function createWorkspaceAnalysisHost(params: {
  readonly workspaceRoot: string;
  readonly classnameTransform: ClassnameTransformMode;
  readonly pathAlias: Readonly<Record<string, string>>;
  readonly styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null;
  readonly typeBackend?: TypeFactBackendKind;
  readonly env?: NodeJS.ProcessEnv;
}): WorkspaceAnalysisHost {
  const aliasResolver = new AliasResolver(
    params.workspaceRoot,
    params.pathAlias,
    loadWorkspaceTsconfigPathAliases(params.workspaceRoot),
    undefined,
    loadWorkspaceBundlerPathAliases(params.workspaceRoot),
  );
  const sourceFileCache = new SourceFileCache({ max: 500 });
  const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
  const typeResolverSelection = selectTypeResolver({
    ...(params.typeBackend ? { typeBackend: params.typeBackend } : {}),
    ...(params.env ? { env: params.env } : {}),
  });
  const typeResolver = typeResolverSelection.typeResolver;
  const sourceFrontendAnalysis =
    resolveSourceFrontendBackendKind(params.env) === "rust-source-frontend"
      ? createDefaultRustSourceFrontendAnalysisProvider({
          aliasResolver: () => aliasResolver,
          fileExists: existsSync,
        })
      : undefined;

  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    ...(sourceFrontendAnalysis ? { sourceFrontendAnalysis } : {}),
    binderPlugins: [
      cssModulesClassnamesBinderPluginV0,
      tailwindUnoUtilityBinderPluginV0,
      vanillaExtractRecipeBinderPluginV0,
      cvaRecipeBinderPluginV0,
      vueStyleModuleBinderPluginV0,
    ],
    fileExists: existsSync,
    aliasResolver,
    max: 500,
    onAnalyze: (uri, entry) => {
      const contribution = collectSemanticReferenceContribution(uri, entry, {
        styleDocumentForPath: params.styleDocumentForPath,
        typeResolver,
        workspaceRoot: params.workspaceRoot,
        filePath: fileURLToPath(uri),
        settingsKey: workspaceSettingsKey(params.classnameTransform, params.pathAlias),
        resolveSymbolValues: (expression) =>
          resolveSymbolValuesFromRustControlFlow({
            source: entry.sourceFile.text,
            sourcePath: fileURLToPath(uri),
            expression,
          }),
      });
      semanticReferenceIndex.record(
        uri,
        contribution.referenceSites,
        contribution.moduleUsages,
        contribution.deps,
      );
    },
  });

  return {
    analysisCache,
    semanticReferenceIndex,
    typeResolver,
    typeBackend: typeResolverSelection.backend,
  };
}

export function collectSourceDocuments(
  sourceFiles: readonly string[],
  analysisCache: DocumentAnalysisCache,
): readonly SourceDocumentSnapshot[] {
  return sourceFiles.map((filePath) => {
    const content = readFileSync(filePath, "utf8");
    const uri = pathToFileURL(filePath).href;
    const version = 1;
    analysisCache.get(uri, content, filePath, version);
    return { uri, filePath, content, version } as const;
  });
}

async function collectWorkspaceFiles(
  workspaceRoot: string,
  pattern: string,
): Promise<readonly string[]> {
  return fastGlob(pattern, {
    cwd: workspaceRoot,
    absolute: true,
    onlyFiles: true,
    followSymbolicLinks: false,
    ignore: [...DEFAULT_IGNORES],
  });
}

function collectWorkspaceFilesSync(workspaceRoot: string, pattern: string): readonly string[] {
  return fastGlob.sync(pattern, {
    cwd: workspaceRoot,
    absolute: true,
    onlyFiles: true,
    followSymbolicLinks: false,
    ignore: [...DEFAULT_IGNORES],
  });
}

export function workspaceSettingsKey(
  classnameTransform: ClassnameTransformMode,
  pathAlias: Readonly<Record<string, string>>,
): string {
  const aliases = Object.entries(pathAlias)
    .toSorted(([a], [b]) => a.localeCompare(b))
    .map(([key, value]) => `${key}=${value}`)
    .join("|");
  return `transform:${classnameTransform};alias:${aliases}`;
}

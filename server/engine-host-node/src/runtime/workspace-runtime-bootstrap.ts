import { readFileSync } from "node:fs";
import type { AliasResolver } from "../../../engine-core-ts/src/core/cx/alias-resolver";
import type {
  SassModuleUseHIR,
  SassSymbolKind,
  StyleDocumentHIR,
} from "../../../engine-core-ts/src/core/hir/style-types";
import type { FileTask } from "../../../engine-core-ts/src/core/indexing/indexer-worker";
import {
  listSassModuleExportedSymbolTargets,
  resolveSassModuleUseTarget,
  resolveSassModuleUseTargetFilePath,
} from "../../../engine-core-ts/src/core/query";
import { findStyleDocumentLangForPath } from "../../../engine-core-ts/src/core/scss/lang-registry";
import type { StyleIndexCache } from "../../../engine-core-ts/src/core/scss/scss-index";
import type { WorkspaceStyleDependencyGraph } from "../../../engine-core-ts/src/core/semantic";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import type { ResourceSettings } from "../../../engine-core-ts/src/settings";
import {
  selectTypeResolver,
  type SelectTypeResolverOptions,
  type TypeFactBackendKind,
} from "../type-backend";
import type { WorkspaceRuntimeIO } from "./workspace-runtime";

export interface RuntimeTypeResolverOptions extends SelectTypeResolverOptions {}

export interface StyleDocumentLookupArgs {
  readonly styleIndexCache: StyleIndexCache;
  readonly styleDependencyGraph: WorkspaceStyleDependencyGraph;
  readonly readOpenDocumentText: (path: string) => string | null;
  readonly readStyleFile: (path: string) => string | null;
  readonly fileExists: (path: string) => boolean;
  readonly aliasResolverForPath?: (path: string) => AliasResolver | null;
  readonly getModeForPath: (path: string) => ResourceSettings["scss"]["classnameTransform"];
}

export interface WorkspaceRuntimeIOOptions {
  readonly readStyleFile: (path: string) => string | null;
  readonly readOpenDocumentText?: (path: string) => string | null;
  readonly readStyleFileAsync?: (path: string) => Promise<string | null>;
  readonly fileSupplier?: () => AsyncIterable<FileTask>;
}

export function createRuntimeTypeResolver(options: RuntimeTypeResolverOptions): TypeResolver {
  return selectTypeResolver(options).typeResolver;
}

export function resolveRuntimeTypeBackend(
  options: RuntimeTypeResolverOptions,
): TypeFactBackendKind {
  return selectTypeResolver(options).backend;
}

export function createStyleDocumentLookup(
  args: StyleDocumentLookupArgs,
): (path: string) => StyleDocumentHIR | null {
  return (path: string): StyleDocumentHIR | null => {
    if (!findStyleDocumentLangForPath(path)) return null;
    const buffered = args.readOpenDocumentText(path);
    const mode = args.getModeForPath(path);
    if (buffered !== null) {
      const styleDocument = args.styleIndexCache.getStyleDocument(path, buffered, mode);
      const styleDocumentForPath = (targetPath: string): StyleDocumentHIR | null =>
        readStyleDocumentForGraph(args, path, styleDocument, targetPath);
      args.styleDependencyGraph.record(path, styleDocument, {
        resolveSassModuleUseTargetFilePath: (moduleUse) =>
          resolveSassModuleUseTargetFilePath(
            path,
            moduleUse,
            args.aliasResolverForPath?.(path) ?? undefined,
            args.fileExists,
            { readFile: args.readStyleFile },
          ),
        resolveSassModuleExportedSymbolTargets: (moduleUse, symbolKind, name) =>
          resolveSassModuleExportedSymbolTargets(
            path,
            moduleUse,
            symbolKind,
            name,
            styleDocumentForPath,
            args.aliasResolverForPath?.(path) ?? undefined,
            args.readStyleFile,
          ),
      });
      return styleDocument;
    }
    const content = args.readStyleFile(path);
    if (content === null) return null;
    const styleDocument = args.styleIndexCache.getStyleDocument(path, content, mode);
    const styleDocumentForPath = (targetPath: string): StyleDocumentHIR | null =>
      readStyleDocumentForGraph(args, path, styleDocument, targetPath);
    args.styleDependencyGraph.record(path, styleDocument, {
      resolveSassModuleUseTargetFilePath: (moduleUse) =>
        resolveSassModuleUseTargetFilePath(
          path,
          moduleUse,
          args.aliasResolverForPath?.(path) ?? undefined,
          args.fileExists,
          { readFile: args.readStyleFile },
        ),
      resolveSassModuleExportedSymbolTargets: (moduleUse, symbolKind, name) =>
        resolveSassModuleExportedSymbolTargets(
          path,
          moduleUse,
          symbolKind,
          name,
          styleDocumentForPath,
          args.aliasResolverForPath?.(path) ?? undefined,
          args.readStyleFile,
        ),
    });
    return styleDocument;
  };
}

function readStyleDocumentForGraph(
  args: StyleDocumentLookupArgs,
  currentPath: string,
  currentDocument: StyleDocumentHIR,
  targetPath: string,
): StyleDocumentHIR | null {
  if (targetPath === currentPath) return currentDocument;
  if (!findStyleDocumentLangForPath(targetPath)) return null;
  const mode = args.getModeForPath(targetPath);
  const buffered = args.readOpenDocumentText(targetPath);
  if (buffered !== null) {
    return args.styleIndexCache.getStyleDocument(targetPath, buffered, mode);
  }
  const content = args.readStyleFile(targetPath);
  return content === null ? null : args.styleIndexCache.getStyleDocument(targetPath, content, mode);
}

function resolveSassModuleExportedSymbolTargets(
  stylePath: string,
  moduleUse: SassModuleUseHIR,
  symbolKind: SassSymbolKind,
  name: string,
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  aliasResolver: AliasResolver | undefined,
  readFile: (path: string) => string | null,
): readonly { readonly filePath: string; readonly name: string }[] {
  const target = resolveSassModuleUseTarget(
    styleDocumentForPath,
    stylePath,
    moduleUse,
    aliasResolver,
    { readFile },
  );
  if (!target) return [];
  return listSassModuleExportedSymbolTargets(
    styleDocumentForPath,
    target.filePath,
    target.styleDocument,
    symbolKind,
    name,
    aliasResolver,
    new Set(),
    { readFile },
  ).map((exportedTarget) => ({
    filePath: exportedTarget.filePath,
    name: exportedTarget.decl.name,
  }));
}

export function createWorkspaceRuntimeIO(options: WorkspaceRuntimeIOOptions): WorkspaceRuntimeIO {
  return {
    readStyleFile: options.readStyleFile,
    ...(options.readOpenDocumentText ? { readOpenDocumentText: options.readOpenDocumentText } : {}),
    ...(options.readStyleFileAsync ? { readStyleFileAsync: options.readStyleFileAsync } : {}),
    ...(options.fileSupplier ? { fileSupplier: options.fileSupplier } : {}),
  };
}

export function defaultReadStyleFile(path: string): string | null {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return null;
  }
}

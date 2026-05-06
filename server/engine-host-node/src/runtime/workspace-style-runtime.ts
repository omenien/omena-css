import type { AliasResolver } from "../../../engine-core-ts/src/core/cx/alias-resolver";
import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import { scssFileSupplier } from "../../../engine-core-ts/src/core/indexing/file-supplier";
import type { FileTask } from "../../../engine-core-ts/src/core/indexing/indexer-worker";
import { IndexerWorker } from "../../../engine-core-ts/src/core/indexing/indexer-worker";
import {
  listSassModuleExportedSymbolTargets,
  resolveSassModuleUseTarget,
  resolveSassModuleUseTargetFilePath,
} from "../../../engine-core-ts/src/core/query";
import type {
  SassModuleUseHIR,
  SassSymbolKind,
} from "../../../engine-core-ts/src/core/hir/style-types";
import type { ResourceSettings } from "../../../engine-core-ts/src/settings";
import type { SharedRuntimeCaches } from "./shared-runtime-caches";
import { createScopedRuntimeLogger, type RuntimeSink } from "./runtime-sink";

export interface WorkspaceStyleRuntimeIO {
  readonly readStyleFile: (path: string) => string | null;
  readonly readStyleFileAsync?: (path: string) => Promise<string | null>;
  readonly fileSupplier?: () => AsyncIterable<FileTask>;
}

export interface WorkspaceStyleRuntimeArgs {
  readonly workspaceRoot: string;
  readonly caches: SharedRuntimeCaches;
  readonly io: WorkspaceStyleRuntimeIO;
  readonly sink: RuntimeSink;
  readonly serverName: string;
  readonly fileExists: (path: string) => boolean;
  readonly aliasResolver: () => AliasResolver;
  readonly getModeForStylePath: (path: string) => ResourceSettings["scss"]["classnameTransform"];
  readonly isOwnedStylePath: (stylePath: string) => boolean;
}

export interface WorkspaceStyleRuntime {
  readonly indexerReady: Promise<void>;
  invalidateStyle(stylePath: string): void;
  peekStyleDocument(
    stylePath: string,
    mode: ResourceSettings["scss"]["classnameTransform"],
  ): StyleDocumentHIR | null;
  buildStyleDocument(
    stylePath: string,
    content: string,
    mode: ResourceSettings["scss"]["classnameTransform"],
  ): StyleDocumentHIR;
  pushStyleFile(stylePath: string): void;
  stop(): void;
}

export function createWorkspaceStyleRuntime(
  args: WorkspaceStyleRuntimeArgs,
): WorkspaceStyleRuntime {
  const logger = createScopedRuntimeLogger(args.sink, `[${args.serverName}:indexer] `);
  const supplier =
    args.io.fileSupplier ??
    (() => scssFileSupplier(args.workspaceRoot, logger, args.isOwnedStylePath));
  const readFile = args.io.readStyleFileAsync ?? defaultReadStyleFileAsync;
  const indexerWorker = new IndexerWorker({
    supplier,
    readFile,
    onScssFile: (stylePath, content) => {
      const styleDocument = args.caches.styleIndexCache.getStyleDocument(
        stylePath,
        content,
        args.getModeForStylePath(stylePath),
      );
      args.caches.styleDependencyGraph.record(stylePath, styleDocument, {
        resolveSassModuleUseTargetFilePath: (moduleUse) =>
          resolveSassModuleUseTargetFilePath(
            stylePath,
            moduleUse,
            args.aliasResolver(),
            args.fileExists,
            { readFile: args.io.readStyleFile },
          ),
        resolveSassModuleExportedSymbolTargets: (moduleUse, symbolKind, name) =>
          resolveSassModuleExportedSymbolTargets(
            stylePath,
            moduleUse,
            symbolKind,
            name,
            (targetPath) => readIndexedStyleDocument(args, targetPath),
            args.aliasResolver(),
            args.io.readStyleFile,
          ),
      });
    },
    logger,
  });
  indexerWorker.start().catch((err: unknown) => {
    args.sink.error(
      `[${args.serverName}] indexer worker crashed: ${
        err instanceof Error ? err.message : String(err)
      }`,
    );
  });

  return {
    indexerReady: indexerWorker.ready,
    invalidateStyle(stylePath: string): void {
      args.caches.styleIndexCache.invalidate(stylePath);
      args.caches.styleDependencyGraph.forget(stylePath);
      args.caches.styleSemanticGraphCache.clear();
      args.caches.styleSemanticGraphBatchOutputCache.clear();
      args.caches.selectorUsagePayloadCache.clear();
    },
    peekStyleDocument(
      stylePath: string,
      mode: ResourceSettings["scss"]["classnameTransform"],
    ): StyleDocumentHIR | null {
      return args.caches.styleIndexCache.peekEntry(stylePath, mode)?.styleDocument ?? null;
    },
    buildStyleDocument(
      stylePath: string,
      content: string,
      mode: ResourceSettings["scss"]["classnameTransform"],
    ): StyleDocumentHIR {
      return args.caches.styleIndexCache.getStyleDocument(stylePath, content, mode);
    },
    pushStyleFile(stylePath: string): void {
      args.caches.styleSemanticGraphCache.clear();
      args.caches.styleSemanticGraphBatchOutputCache.clear();
      args.caches.selectorUsagePayloadCache.clear();
      indexerWorker.pushFile({ path: stylePath });
    },
    stop(): void {
      indexerWorker.stop();
    },
  };
}

async function defaultReadStyleFileAsync(stylePath: string): Promise<string | null> {
  const { readFile } = await import("node:fs/promises");
  try {
    return await readFile(stylePath, "utf8");
  } catch {
    return null;
  }
}

function readIndexedStyleDocument(
  args: WorkspaceStyleRuntimeArgs,
  stylePath: string,
): StyleDocumentHIR | null {
  if (!args.fileExists(stylePath)) return null;
  const mode = args.getModeForStylePath(stylePath);
  const cached = args.caches.styleIndexCache.peekEntry(stylePath, mode)?.styleDocument;
  if (cached) return cached;
  const content = args.io.readStyleFile(stylePath);
  return content === null
    ? null
    : args.caches.styleIndexCache.getStyleDocument(stylePath, content, mode);
}

function resolveSassModuleExportedSymbolTargets(
  stylePath: string,
  moduleUse: SassModuleUseHIR,
  symbolKind: SassSymbolKind,
  name: string,
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
  aliasResolver: AliasResolver,
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

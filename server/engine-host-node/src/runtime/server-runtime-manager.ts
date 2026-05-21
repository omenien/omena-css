import { existsSync } from "node:fs";
import { DEFAULT_RESOURCE_SETTINGS } from "../../../engine-core-ts/src/settings";
import type { FileTask } from "../../../engine-core-ts/src/core/indexing/indexer-worker";
import type { StyleDocumentBuilder } from "../../../engine-core-ts/src/core/scss/scss-index";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import type { WorkspaceRegistry } from "../workspace/workspace-registry";
import {
  buildSharedRuntimeCaches,
  createManifestCachedStyleFileReader,
} from "./shared-runtime-caches";
import {
  createRuntimeTypeResolver,
  createStyleDocumentLookup,
  createWorkspaceRuntimeIO,
} from "./workspace-runtime-bootstrap";
import {
  createWorkspaceRuntimeManager,
  type WorkspaceRuntimeManager,
} from "./workspace-runtime-manager";
import type { RuntimeSink } from "./runtime-sink";
import { resolveRuntimeStyleDocumentBuilder } from "../omena-parser-style-document-builder";

export interface ServerRuntimeManagerOptions {
  readonly typeResolver?: TypeResolver;
  readonly fileSupplier?: () => AsyncIterable<FileTask>;
  readonly readStyleFileAsync?: (path: string) => Promise<string | null>;
  readonly fileExists?: (path: string) => boolean;
  readonly buildStyleDocument?: StyleDocumentBuilder;
}

export interface CreateServerRuntimeManagerArgs {
  readonly options: ServerRuntimeManagerOptions;
  readonly readStyleFile: (path: string) => string | null;
  readonly readOpenDocumentText: (path: string) => string | null;
  readonly sink: RuntimeSink;
  readonly serverName: string;
}

export interface ServerRuntimeManagerBundle {
  readonly registry: WorkspaceRegistry;
  readonly runtimeManager: WorkspaceRuntimeManager;
}

export function createServerRuntimeManager(
  args: CreateServerRuntimeManagerArgs,
): ServerRuntimeManagerBundle {
  const buildStyleDocument =
    args.options.buildStyleDocument ?? resolveRuntimeStyleDocumentBuilder();
  const caches = buildSharedRuntimeCaches(buildStyleDocument ? { buildStyleDocument } : {});
  const typeResolver = args.options.typeResolver
    ? createRuntimeTypeResolver({ typeResolver: args.options.typeResolver })
    : createRuntimeTypeResolver({});
  const fileExists = args.options.fileExists ?? existsSync;
  const readStyleFile = createManifestCachedStyleFileReader(caches, args.readStyleFile);
  const runtimeIO = createWorkspaceRuntimeIO({
    readStyleFile,
    readOpenDocumentText: args.readOpenDocumentText,
    ...(args.options.readStyleFileAsync
      ? { readStyleFileAsync: args.options.readStyleFileAsync }
      : {}),
    ...(args.options.fileSupplier ? { fileSupplier: args.options.fileSupplier } : {}),
  });

  let runtimeManager: WorkspaceRuntimeManager | null = null;
  const styleDocumentForPath = createStyleDocumentLookup({
    styleIndexCache: caches.styleIndexCache,
    styleDependencyGraph: caches.styleDependencyGraph,
    readOpenDocumentText: args.readOpenDocumentText,
    readStyleFile,
    fileExists,
    aliasResolverForPath: (stylePath) =>
      runtimeManager?.getDepsForFilePath(stylePath)?.aliasResolver ?? null,
    getModeForPath: (stylePath) =>
      runtimeManager?.getDepsForFilePath(stylePath)?.settings.scss.classnameTransform ??
      DEFAULT_RESOURCE_SETTINGS.scss.classnameTransform,
  });

  runtimeManager = createWorkspaceRuntimeManager({
    caches,
    typeResolver,
    styleDocumentForPath,
    io: runtimeIO,
    sink: args.sink,
    fileExists,
    serverName: args.serverName,
    getModeForStylePath: (stylePath) =>
      runtimeManager?.getDepsForFilePath(stylePath)?.settings.scss.classnameTransform ??
      DEFAULT_RESOURCE_SETTINGS.scss.classnameTransform,
  });

  return {
    registry: runtimeManager.getRegistry(),
    runtimeManager,
  };
}

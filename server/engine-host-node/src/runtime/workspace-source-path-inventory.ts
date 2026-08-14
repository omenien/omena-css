import { readFileSync } from "node:fs";
import path from "node:path";
import type { FileTask } from "../../../engine-core-ts/src/core/indexing/indexer-worker";
import {
  isSourceFilePath,
  sourceFileSupplier,
} from "../../../engine-core-ts/src/core/indexing/file-supplier";
import type { RuntimeFileChangeType } from "./invalidation-planner";
import type { RuntimeSink } from "./runtime-sink";

export interface WorkspaceSourcePathInventoryArgs {
  readonly workspaceRoot: string;
  readonly supplier?: () => AsyncIterable<FileTask>;
  readonly readSourceFile?: (filePath: string) => string | null;
  readonly sink: RuntimeSink;
  readonly serverName: string;
  readonly maxCachedUtf8Bytes?: number;
  readonly maxCachedFiles?: number;
}

export interface SourceCorpusFileRead {
  readonly source: string;
  readonly utf8Bytes: number;
  readonly cacheHit: boolean;
}

export interface WorkspaceSourcePathInventory {
  readonly ready: Promise<void>;
  completePaths(): readonly string[] | null;
  readSourceFile(filePath: string): SourceCorpusFileRead | null;
  setDynamicWatcherCoverage(available: boolean): void;
  applyFileChange(filePath: string, changeType: RuntimeFileChangeType): void;
  stop(): void;
}

const DEFAULT_SOURCE_CACHE_MAX_UTF8_BYTES = 16 * 1024 * 1024;
const DEFAULT_SOURCE_CACHE_MAX_FILES = 4_096;

/**
 * Maintains the filesystem half of the source-corpus completeness proof used
 * by selected-query diagnostics. Open unsaved documents are merged by the LSP
 * scheduler at request time; this inventory owns only the complete initial
 * disk walk plus watched create/change/delete updates.
 */
export function createWorkspaceSourcePathInventory(
  args: WorkspaceSourcePathInventoryArgs,
): WorkspaceSourcePathInventory {
  const paths = new Set<string>();
  const sourceCache = new Map<string, { readonly source: string; readonly utf8Bytes: number }>();
  const maxCachedUtf8Bytes = Math.max(
    0,
    args.maxCachedUtf8Bytes ?? DEFAULT_SOURCE_CACHE_MAX_UTF8_BYTES,
  );
  const maxCachedFiles = Math.max(
    0,
    Math.floor(args.maxCachedFiles ?? DEFAULT_SOURCE_CACHE_MAX_FILES),
  );
  let cachedUtf8Bytes = 0;
  let walkComplete = false;
  let dynamicWatcherCoverage = false;
  let stopped = false;
  const supplier = args.supplier ?? (() => sourceFileSupplier(args.workspaceRoot));
  const readSourceFile = args.readSourceFile ?? readSourceFileFromDisk;
  const ready = (async (): Promise<void> => {
    try {
      for await (const task of supplier()) {
        if (stopped) return;
        if (isSourceFilePath(task.path)) paths.add(path.resolve(task.path));
      }
      if (!stopped) walkComplete = true;
    } catch (err: unknown) {
      const detail = err instanceof Error ? err.message : String(err);
      args.sink.error(`[${args.serverName}:source-inventory] source walk incomplete: ${detail}`);
    }
  })();

  return {
    ready,
    completePaths(): readonly string[] | null {
      return walkComplete && dynamicWatcherCoverage ? [...paths].toSorted() : null;
    },
    readSourceFile(filePath): SourceCorpusFileRead | null {
      const resolvedPath = path.resolve(filePath);
      const cached = dynamicWatcherCoverage ? sourceCache.get(resolvedPath) : undefined;
      if (cached) {
        sourceCache.delete(resolvedPath);
        sourceCache.set(resolvedPath, cached);
        return { ...cached, cacheHit: true };
      }
      const source = readSourceFile(resolvedPath);
      if (source === null) return null;
      const utf8Bytes = Buffer.byteLength(source, "utf8");
      if (dynamicWatcherCoverage && maxCachedFiles > 0 && utf8Bytes <= maxCachedUtf8Bytes) {
        const prior = sourceCache.get(resolvedPath);
        if (prior) cachedUtf8Bytes -= prior.utf8Bytes;
        sourceCache.delete(resolvedPath);
        sourceCache.set(resolvedPath, { source, utf8Bytes });
        cachedUtf8Bytes += utf8Bytes;
        while (cachedUtf8Bytes > maxCachedUtf8Bytes || sourceCache.size > maxCachedFiles) {
          const oldest = sourceCache.entries().next().value;
          if (!oldest) break;
          const [oldestPath, oldestEntry] = oldest;
          sourceCache.delete(oldestPath);
          cachedUtf8Bytes -= oldestEntry.utf8Bytes;
        }
      }
      return { source, utf8Bytes, cacheHit: false };
    },
    setDynamicWatcherCoverage(available): void {
      dynamicWatcherCoverage = available;
      if (!available) {
        sourceCache.clear();
        cachedUtf8Bytes = 0;
      }
    },
    applyFileChange(filePath, changeType): void {
      if (!isSourceFilePath(filePath)) return;
      const resolvedPath = path.resolve(filePath);
      const cached = sourceCache.get(resolvedPath);
      if (cached) cachedUtf8Bytes -= cached.utf8Bytes;
      sourceCache.delete(resolvedPath);
      if (changeType === "deleted") {
        paths.delete(resolvedPath);
      } else {
        paths.add(resolvedPath);
      }
    },
    stop(): void {
      stopped = true;
      walkComplete = false;
      dynamicWatcherCoverage = false;
      paths.clear();
      sourceCache.clear();
      cachedUtf8Bytes = 0;
    },
  };
}

function readSourceFileFromDisk(filePath: string): string | null {
  try {
    return readFileSync(filePath, "utf8");
  } catch {
    return null;
  }
}

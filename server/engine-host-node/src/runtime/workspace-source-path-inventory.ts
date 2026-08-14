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
}

export interface SourceCorpusFileRead {
  readonly source: string;
  readonly utf8Bytes: number;
  readonly cacheHit: boolean;
}

export interface SourceCorpusReadCounters {
  readonly diskReadFiles: number;
  readonly diskReadBytes: number;
  readonly cacheHitFiles: number;
}

export interface WorkspaceSourcePathInventory {
  readonly ready: Promise<void>;
  completePaths(): readonly string[] | null;
  readSourceFile(filePath: string): SourceCorpusFileRead | null;
  sourceCorpusReadCounters(): SourceCorpusReadCounters;
  setDynamicWatcherCoverage(available: boolean): void;
  applyFileChange(filePath: string, changeType: RuntimeFileChangeType): void;
  stop(): void;
}

/**
 * Maintains the filesystem half of the source-corpus completeness proof used
 * by selected-query diagnostics. Open unsaved documents are merged by the LSP
 * scheduler at request time; this inventory owns only the complete initial
 * disk walk plus watched create/delete updates.
 */
export function createWorkspaceSourcePathInventory(
  args: WorkspaceSourcePathInventoryArgs,
): WorkspaceSourcePathInventory {
  const paths = new Set<string>();
  const sourceCache = new Map<string, { readonly source: string; readonly utf8Bytes: number }>();
  let walkComplete = false;
  let dynamicWatcherCoverage = false;
  let diskReadFiles = 0;
  let diskReadBytes = 0;
  let cacheHitFiles = 0;
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
      const cached = sourceCache.get(resolvedPath);
      if (cached) {
        cacheHitFiles += 1;
        return { ...cached, cacheHit: true };
      }
      const source = readSourceFile(resolvedPath);
      if (source === null) return null;
      const utf8Bytes = Buffer.byteLength(source, "utf8");
      sourceCache.set(resolvedPath, { source, utf8Bytes });
      diskReadFiles += 1;
      diskReadBytes += utf8Bytes;
      return { source, utf8Bytes, cacheHit: false };
    },
    sourceCorpusReadCounters(): SourceCorpusReadCounters {
      return { diskReadFiles, diskReadBytes, cacheHitFiles };
    },
    setDynamicWatcherCoverage(available): void {
      dynamicWatcherCoverage = available;
    },
    applyFileChange(filePath, changeType): void {
      if (!isSourceFilePath(filePath)) return;
      const resolvedPath = path.resolve(filePath);
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

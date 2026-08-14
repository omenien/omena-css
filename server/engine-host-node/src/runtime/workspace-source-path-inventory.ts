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
  readonly sink: RuntimeSink;
  readonly serverName: string;
}

export interface WorkspaceSourcePathInventory {
  readonly ready: Promise<void>;
  completePaths(): readonly string[] | null;
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
  let complete = false;
  let stopped = false;
  const supplier = args.supplier ?? (() => sourceFileSupplier(args.workspaceRoot));
  const ready = (async (): Promise<void> => {
    try {
      for await (const task of supplier()) {
        if (stopped) return;
        if (isSourceFilePath(task.path)) paths.add(task.path);
      }
      if (!stopped) complete = true;
    } catch (err: unknown) {
      const detail = err instanceof Error ? err.message : String(err);
      args.sink.error(`[${args.serverName}:source-inventory] source walk incomplete: ${detail}`);
    }
  })();

  return {
    ready,
    completePaths(): readonly string[] | null {
      return complete ? [...paths].toSorted() : null;
    },
    applyFileChange(filePath, changeType): void {
      if (!isSourceFilePath(filePath)) return;
      if (changeType === "deleted") {
        paths.delete(filePath);
      } else {
        paths.add(filePath);
      }
    },
    stop(): void {
      stopped = true;
      complete = false;
      paths.clear();
    },
  };
}

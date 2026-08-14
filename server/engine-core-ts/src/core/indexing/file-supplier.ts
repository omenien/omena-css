import fastGlob from "fast-glob";
import { buildStyleFileWatcherGlob } from "../scss/lang-registry";
import type { FileTask } from "./indexer-worker";

export const SOURCE_FILE_EXTENSIONS = [
  "ts",
  "tsx",
  "js",
  "jsx",
  "mts",
  "cts",
  "mjs",
  "cjs",
  "d.ts",
] as const;

export const SOURCE_FILE_WATCHER_GLOB = `**/*.{${SOURCE_FILE_EXTENSIONS.join(",")}}`;

const WORKSPACE_FILE_IGNORES = ["**/node_modules/**", "**/dist/**", "**/.git/**"] as const;

/** Yields one FileTask per style module file in the workspace. */
export function scssFileSupplier(
  workspaceRoot: string,
  logger: { error: (msg: string) => void },
  shouldIncludePath: (path: string) => boolean = () => true,
): AsyncIterable<FileTask> {
  return {
    async *[Symbol.asyncIterator](): AsyncGenerator<FileTask> {
      const stream = fastGlob.stream(buildStyleFileWatcherGlob(), {
        cwd: workspaceRoot,
        absolute: true,
        onlyFiles: true,
        followSymbolicLinks: false,
        ignore: [...WORKSPACE_FILE_IGNORES],
      });
      try {
        for await (const entry of stream) {
          const path = String(entry);
          if (!shouldIncludePath(path)) continue;
          yield { path };
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        logger.error(`scssFileSupplier aborted mid-walk: ${message}`);
      }
    },
  };
}

/**
 * Yields every source file whose contents contribute to workspace-wide
 * selected-query negative facts. Unlike the style indexer supplier, errors
 * intentionally propagate: a partial walk must never be presented as a
 * complete source-corpus enumeration.
 */
export function sourceFileSupplier(workspaceRoot: string): AsyncIterable<FileTask> {
  return {
    async *[Symbol.asyncIterator](): AsyncGenerator<FileTask> {
      const stream = fastGlob.stream(SOURCE_FILE_WATCHER_GLOB, {
        cwd: workspaceRoot,
        absolute: true,
        onlyFiles: true,
        followSymbolicLinks: false,
        ignore: [...WORKSPACE_FILE_IGNORES],
      });
      for await (const entry of stream) {
        yield { path: String(entry) };
      }
    },
  };
}

export function isSourceFilePath(filePath: string): boolean {
  return SOURCE_FILE_EXTENSIONS.some((extension) => filePath.endsWith(`.${extension}`));
}

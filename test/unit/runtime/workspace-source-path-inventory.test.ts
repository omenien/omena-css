import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it, vi } from "vitest";
import type { FileTask } from "../../../server/engine-core-ts/src/core/indexing/indexer-worker";
import { createWorkspaceSourcePathInventory } from "../../../server/engine-host-node/src/runtime/workspace-source-path-inventory";
import type { RuntimeSink } from "../../../server/engine-host-node/src/runtime/runtime-sink";

describe("workspace source-path inventory", () => {
  it("publishes only a completed source walk with live watcher coverage", async () => {
    const inventory = createWorkspaceSourcePathInventory({
      workspaceRoot: "/fake/ws",
      supplier: () => tasks(["/fake/ws/src/App.tsx", "/fake/ws/src/not-source.json"]),
      sink: makeSink(),
      serverName: "test",
    });

    expect(inventory.completePaths()).toBeNull();
    await inventory.ready;
    expect(inventory.completePaths()).toBeNull();

    inventory.setDynamicWatcherCoverage(true);
    expect(inventory.completePaths()).toEqual(["/fake/ws/src/App.tsx"]);

    inventory.applyFileChange("/fake/ws/src/New.mts", "created");
    expect(inventory.completePaths()).toEqual(["/fake/ws/src/App.tsx", "/fake/ws/src/New.mts"]);
    inventory.applyFileChange("/fake/ws/src/App.tsx", "deleted");
    expect(inventory.completePaths()).toEqual(["/fake/ws/src/New.mts"]);

    inventory.setDynamicWatcherCoverage(false);
    expect(inventory.completePaths()).toBeNull();
  });

  it("invalidates only a changed source read", async () => {
    const readSourceFile = vi.fn(() => "const value = '핀';\n");
    const inventory = createWorkspaceSourcePathInventory({
      workspaceRoot: "/fake/ws",
      supplier: () => tasks(["/fake/ws/src/App.tsx"]),
      readSourceFile,
      sink: makeSink(),
      serverName: "test",
    });

    await inventory.ready;
    inventory.setDynamicWatcherCoverage(true);
    expect(inventory.readSourceFile("/fake/ws/src/App.tsx")).toMatchObject({
      cacheHit: false,
      utf8Bytes: Buffer.byteLength("const value = '핀';\n", "utf8"),
    });
    expect(inventory.readSourceFile("/fake/ws/src/App.tsx")).toMatchObject({ cacheHit: true });
    expect(readSourceFile).toHaveBeenCalledTimes(1);

    inventory.applyFileChange("/fake/ws/src/App.tsx", "changed");
    expect(inventory.readSourceFile("/fake/ws/src/App.tsx")?.cacheHit).toBe(false);
    expect(readSourceFile).toHaveBeenCalledTimes(2);

    inventory.setDynamicWatcherCoverage(false);
    expect(inventory.readSourceFile("/fake/ws/src/App.tsx")?.cacheHit).toBe(false);
    expect(readSourceFile).toHaveBeenCalledTimes(3);
  });

  it("bounds watcher-backed source bytes with least-recently-used eviction", async () => {
    const sources = new Map([
      [path.resolve("/fake/ws/src/A.tsx"), "aaaa"],
      [path.resolve("/fake/ws/src/B.tsx"), "bbbb"],
    ]);
    const readSourceFile = vi.fn((filePath: string) => sources.get(filePath) ?? null);
    const inventory = createWorkspaceSourcePathInventory({
      workspaceRoot: "/fake/ws",
      supplier: () => tasks([...sources.keys()]),
      readSourceFile,
      sink: makeSink(),
      serverName: "test",
      maxCachedUtf8Bytes: 4,
      maxCachedFiles: 1,
    });

    await inventory.ready;
    inventory.setDynamicWatcherCoverage(true);
    expect(inventory.readSourceFile("/fake/ws/src/A.tsx")?.cacheHit).toBe(false);
    expect(inventory.readSourceFile("/fake/ws/src/B.tsx")?.cacheHit).toBe(false);
    expect(inventory.readSourceFile("/fake/ws/src/A.tsx")?.cacheHit).toBe(false);
    expect(readSourceFile).toHaveBeenCalledTimes(3);
  });

  it("does not serve a stale cached source when dynamic watcher coverage is absent", async () => {
    const workspaceRoot = mkdtempSync(path.join(tmpdir(), "omena-source-cache-stale-"));
    try {
      mkdirSync(path.join(workspaceRoot, "src"));
      const sourcePath = path.join(workspaceRoot, "src/App.tsx");
      writeFileSync(sourcePath, "export const version = 'before';\n");
      const inventory = createWorkspaceSourcePathInventory({
        workspaceRoot,
        supplier: () => tasks([sourcePath]),
        sink: makeSink(),
        serverName: "test",
      });

      await inventory.ready;
      expect(inventory.readSourceFile(sourcePath)).toMatchObject({
        source: "export const version = 'before';\n",
        cacheHit: false,
      });
      writeFileSync(sourcePath, "export const version = 'after';\n");
      expect(inventory.readSourceFile(sourcePath)).toMatchObject({
        source: "export const version = 'after';\n",
        cacheHit: false,
      });
    } finally {
      rmSync(workspaceRoot, { recursive: true, force: true });
    }
  });

  it("runs the production supplier and proves a non-empty complete enumeration", async () => {
    const workspaceRoot = mkdtempSync(path.join(tmpdir(), "omena-source-inventory-"));
    try {
      mkdirSync(path.join(workspaceRoot, "src"));
      writeFileSync(path.join(workspaceRoot, "src/App.tsx"), "export const App = () => null;\n");
      writeFileSync(path.join(workspaceRoot, "src/ignored.json"), "{}\n");
      const inventory = createWorkspaceSourcePathInventory({
        workspaceRoot,
        sink: makeSink(),
        serverName: "test",
      });

      await inventory.ready;
      inventory.setDynamicWatcherCoverage(true);
      expect(inventory.completePaths()).toEqual([path.join(workspaceRoot, "src/App.tsx")]);
    } finally {
      rmSync(workspaceRoot, { recursive: true, force: true });
    }
  });

  it("keeps completeness unknown when the source walk fails", async () => {
    const errors: string[] = [];
    const inventory = createWorkspaceSourcePathInventory({
      workspaceRoot: "/fake/ws",
      supplier: () => ({
        async *[Symbol.asyncIterator](): AsyncGenerator<FileTask> {
          yield { path: "/fake/ws/src/App.tsx" };
          throw new Error("walk failed");
        },
      }),
      sink: makeSink(errors),
      serverName: "test",
    });

    await inventory.ready;
    expect(inventory.completePaths()).toBeNull();
    expect(errors).toEqual(["[test:source-inventory] source walk incomplete: walk failed"]);
  });
});

function tasks(paths: readonly string[]): AsyncIterable<FileTask> {
  return {
    async *[Symbol.asyncIterator](): AsyncGenerator<FileTask> {
      for (const filePath of paths) yield { path: filePath };
    },
  };
}

function makeSink(errors: string[] = []): RuntimeSink {
  return {
    info: () => {},
    error: (message) => errors.push(message),
    clearDiagnostics: () => {},
    requestCodeLensRefresh: () => {},
  };
}

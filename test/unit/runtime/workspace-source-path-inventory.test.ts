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

  it("invalidates only a changed source read while accounting files and bytes", async () => {
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
    expect(inventory.readSourceFile("/fake/ws/src/App.tsx")?.cacheHit).toBe(false);
    expect(inventory.readSourceFile("/fake/ws/src/App.tsx")?.cacheHit).toBe(true);
    expect(readSourceFile).toHaveBeenCalledTimes(1);

    inventory.applyFileChange("/fake/ws/src/App.tsx", "changed");
    expect(inventory.readSourceFile("/fake/ws/src/App.tsx")?.cacheHit).toBe(false);
    const utf8Bytes = Buffer.byteLength("const value = '핀';\n", "utf8");
    expect(inventory.sourceCorpusReadCounters()).toEqual({
      diskReadFiles: 2,
      diskReadBytes: utf8Bytes * 2,
      cacheHitFiles: 1,
    });
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

import { describe, expect, it } from "vitest";
import type { FileTask } from "../../../server/engine-core-ts/src/core/indexing/indexer-worker";
import { createWorkspaceSourcePathInventory } from "../../../server/engine-host-node/src/runtime/workspace-source-path-inventory";
import type { RuntimeSink } from "../../../server/engine-host-node/src/runtime/runtime-sink";

describe("workspace source-path inventory", () => {
  it("publishes only a completed source walk and tracks watched membership", async () => {
    const inventory = createWorkspaceSourcePathInventory({
      workspaceRoot: "/fake/ws",
      supplier: () => tasks(["/fake/ws/src/App.tsx", "/fake/ws/src/not-source.json"]),
      sink: makeSink(),
      serverName: "test",
    });

    expect(inventory.completePaths()).toBeNull();
    await inventory.ready;
    expect(inventory.completePaths()).toEqual(["/fake/ws/src/App.tsx"]);

    inventory.applyFileChange("/fake/ws/src/New.mts", "created");
    expect(inventory.completePaths()).toEqual(["/fake/ws/src/App.tsx", "/fake/ws/src/New.mts"]);
    inventory.applyFileChange("/fake/ws/src/App.tsx", "deleted");
    expect(inventory.completePaths()).toEqual(["/fake/ws/src/New.mts"]);
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
      for (const path of paths) yield { path };
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

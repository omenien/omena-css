import { describe, expect, it, vi } from "vitest";
import { WorkspaceRegistry } from "../../../server/engine-host-node/src/workspace/workspace-registry";
import type { WorkspaceProviderDeps } from "../../../server/engine-host-node/src/workspace/workspace-registry";
import { applyWatchedFileChanges } from "../../../server/engine-host-node/src/runtime/watched-file-application";
import { makeBaseDeps } from "../../_fixtures/test-helpers";

describe("applyWatchedFileChanges", () => {
  it("invalidates cached package manifests when package.json changes", () => {
    const registry = new WorkspaceRegistry();
    const invalidatePackageManifestCache = vi.fn();
    const deps = {
      ...makeBaseDeps({
        workspaceRoot: "/fake/ws",
        workspaceFolderUri: "file:///fake/ws",
      }),
      invalidatePackageManifestCache,
    } satisfies WorkspaceProviderDeps;

    registry.register({ uri: "file:///fake/ws", rootPath: "/fake/ws", name: "fake" }, deps);

    applyWatchedFileChanges({
      registry,
      documents: { all: () => [], get: () => undefined },
      events: [
        { uri: "file:///fake/ws/node_modules/@design/tokens/package.json", type: "changed" },
      ],
    });

    expect(invalidatePackageManifestCache).toHaveBeenCalledWith(
      "/fake/ws/node_modules/@design/tokens/package.json",
    );
  });
});

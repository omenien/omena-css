import { describe, expect, it } from "vitest";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { createServerRuntimeManager } from "../../../server/engine-host-node/src/runtime/server-runtime-manager";
import {
  buildSharedRuntimeCaches,
  createManifestCachedStyleFileReader,
} from "../../../server/engine-host-node/src/runtime/shared-runtime-caches";
import type { RuntimeSink } from "../../../server/engine-host-node/src/runtime/runtime-sink";

describe("buildSharedRuntimeCaches", () => {
  it("threads an injected style document builder into the runtime style index cache", () => {
    let buildCount = 0;
    const caches = buildSharedRuntimeCaches({
      buildStyleDocument: (filePath, content) => {
        buildCount += 1;
        return parseStyleDocument(content, filePath);
      },
    });

    const first = caches.styleIndexCache.getStyleDocument(
      "/fake/ws/src/Button.module.scss",
      ".button { color: red; }",
    );
    const second = caches.styleIndexCache.getStyleDocument(
      "/fake/ws/src/Button.module.scss",
      ".button { color: red; }",
    );

    expect(first.selectors.map((selector) => selector.name)).toEqual(["button"]);
    expect(second).toBe(first);
    expect(buildCount).toBe(1);
  });

  it("threads the parser builder seam through the server runtime manager", () => {
    const stylePath = "/fake/ws/src/Button.module.scss";
    const files = new Map([[stylePath, ".button { color: red; }"]]);
    let buildCount = 0;
    const { runtimeManager } = createServerRuntimeManager({
      options: {
        fileSupplier: async function* () {},
        fileExists: (filePath) => files.has(filePath),
        buildStyleDocument: (filePath, content) => {
          buildCount += 1;
          return parseStyleDocument(content, filePath);
        },
      },
      readStyleFile: (filePath) => files.get(filePath) ?? null,
      readOpenDocumentText: () => null,
      sink: makeSink(),
      serverName: "test",
    });

    runtimeManager.registerInitialFolders([
      { uri: "file:///fake/ws", rootPath: "/fake/ws", name: "fake" },
    ]);

    const deps = runtimeManager.getDepsForFilePath(stylePath);
    const document = deps?.styleDocumentForPath(stylePath);

    expect(document?.selectors.map((selector) => selector.name)).toEqual(["button"]);
    expect(buildCount).toBe(1);
    runtimeManager.disposeAll({ all: () => [] });
  });

  it("caches package manifest reads without caching normal style files", () => {
    const caches = buildSharedRuntimeCaches();
    const reads = new Map<string, number>();
    const files = new Map([
      ["/fake/ws/package.json", `{"style":"./index.css"}`],
      ["/fake/ws/src/Button.module.scss", ".button { color: red; }"],
    ]);
    const readStyleFile = createManifestCachedStyleFileReader(caches, (filePath) => {
      reads.set(filePath, (reads.get(filePath) ?? 0) + 1);
      return files.get(filePath) ?? null;
    });

    expect(readStyleFile("/fake/ws/package.json")).toBe(`{"style":"./index.css"}`);
    expect(readStyleFile("/fake/ws/package.json")).toBe(`{"style":"./index.css"}`);
    expect(readStyleFile("/fake/ws/src/Button.module.scss")).toBe(".button { color: red; }");
    expect(readStyleFile("/fake/ws/src/Button.module.scss")).toBe(".button { color: red; }");

    expect(reads.get("/fake/ws/package.json")).toBe(1);
    expect(reads.get("/fake/ws/src/Button.module.scss")).toBe(2);

    files.set("/fake/ws/package.json", `{"sass":"./index.scss"}`);
    caches.packageManifestTextCache.invalidate("/fake/ws/package.json");
    expect(readStyleFile("/fake/ws/package.json")).toBe(`{"sass":"./index.scss"}`);
    expect(reads.get("/fake/ws/package.json")).toBe(2);
  });
});

function makeSink(): RuntimeSink {
  return {
    info() {},
    error() {},
    clearDiagnostics() {},
    requestCodeLensRefresh() {},
  };
}

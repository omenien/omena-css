import { describe, expect, it } from "vitest";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { buildSharedRuntimeCaches } from "../../../server/engine-host-node/src/runtime/shared-runtime-caches";

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
});

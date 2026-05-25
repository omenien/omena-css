import { describe, expect, it } from "vitest";

import { buildSourceDocument } from "../../../server/engine-core-ts/src/core/hir/builders/ts-source-adapter";

describe("source language inference", () => {
  it("marks Vue SFC sources explicitly in HIR", () => {
    const document = buildSourceDocument({
      filePath: "/workspace/src/Card.vue",
      cxBindings: [],
      stylesBindings: new Map(),
      classUtilNames: [],
      classExpressions: [],
    });

    expect(document.language).toBe("vue");
  });
});

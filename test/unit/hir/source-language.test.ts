import { describe, expect, it } from "vitest";

import { buildSourceDocument } from "../../../server/engine-core-ts/src/core/hir/builders/ts-source-adapter";

function inferLanguage(filePath: string) {
  return buildSourceDocument({
    filePath,
    cxBindings: [],
    stylesBindings: new Map(),
    classUtilNames: [],
    classExpressions: [],
  }).language;
}

describe("source language inference", () => {
  it("marks Vue SFC sources explicitly in HIR", () => {
    expect(inferLanguage("/workspace/src/Card.vue")).toBe("vue");
  });

  it("mirrors Rust source-language families in HIR", () => {
    expect(inferLanguage("/workspace/src/Page.html")).toBe("html");
    expect(inferLanguage("/workspace/src/Page.svelte")).toBe("svelte");
    expect(inferLanguage("/workspace/src/Page.astro")).toBe("astro");
    expect(inferLanguage("/workspace/src/Notes.md")).toBe("markdown");
    expect(inferLanguage("/workspace/src/Notes.mdx")).toBe("markdown");
    expect(inferLanguage("/workspace/src/Card.liquid")).toBe("server-template");
    expect(inferLanguage("/workspace/src/Card.html.eex")).toBe("server-template");
    expect(inferLanguage("/workspace/src/card.rb")).toBe("unknown");
  });
});

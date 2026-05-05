import { describe, expect, it } from "vitest";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { WorkspaceStyleDependencyGraph } from "../../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import { infoAtLine, makeBaseDeps, semanticSiteAt } from "../../_fixtures/test-helpers";
import {
  readStyleRenameTargetAtCursor,
  planStyleRenameAtCursor,
} from "../../../server/engine-host-node/src/style-rename-query";
import type { StyleSemanticGraphSummaryV0 } from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";

const SCSS_PATH = "/fake/src/Button.module.scss";
const TOKENS_PATH = "/fake/src/tokens.module.scss";

describe("style rename query", () => {
  it("reads a target and plans SCSS plus source edits", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSiteAt("file:///fake/src/App.tsx", "indicator", 10, SCSS_PATH),
    ]);
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
      workspaceRoot: "/fake",
      semanticReferenceIndex,
    });
    const styleDocument = deps.styleDocumentForPath(SCSS_PATH);

    expect(styleDocument).not.toBeNull();
    const target = readStyleRenameTargetAtCursor(SCSS_PATH, 1, 3, styleDocument!, deps);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("indicator");

    const plan = planStyleRenameAtCursor(SCSS_PATH, 1, 3, styleDocument!, deps, "status");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits.map((edit) => edit.newText) : []).toEqual([
      "status",
      "status",
    ]);
  });

  it("returns null when no selector target exists", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
      workspaceRoot: "/fake",
    });
    const styleDocument = deps.styleDocumentForPath(SCSS_PATH);

    expect(styleDocument).not.toBeNull();
    expect(planStyleRenameAtCursor(SCSS_PATH, 99, 0, styleDocument!, deps, "status")).toBeNull();
  });

  it("renames selectors through direct cross-file composes references", () => {
    const basePath = "/fake/src/Base.module.scss";
    const baseScss = `.base { color: red; }\n`;
    const buttonScss = `.button {
  composes: base from "./Base.module.scss";
}
`;
    const baseDocument = parseStyleDocument(baseScss, basePath);
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(basePath, baseDocument);
    styleDependencyGraph.record(SCSS_PATH, buttonDocument);
    const deps = makeBaseDeps({
      styleDependencyGraph,
      styleDocumentForPath: (filePath) => {
        if (filePath === basePath) return baseDocument;
        if (filePath === SCSS_PATH) return buttonDocument;
        return null;
      },
    });

    const plan = planStyleRenameAtCursor(basePath, 0, 2, baseDocument, deps, "surface");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      {
        uri: "file:///fake/src/Base.module.scss",
        range: { start: { line: 0, character: 1 }, end: { line: 0, character: 5 } },
        newText: "surface",
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: { start: { line: 1, character: 12 }, end: { line: 1, character: 16 } },
        newText: "surface",
      },
    ]);
  });

  it("blocks selector rename when composes references are transitive", () => {
    const basePath = "/fake/src/Base.module.scss";
    const resetPath = "/fake/src/Reset.module.scss";
    const baseScss = `.base { color: red; }\n`;
    const resetScss = `.reset {
  composes: base from "./Base.module.scss";
}
`;
    const buttonScss = `.button {
  composes: reset from "./Reset.module.scss";
}
`;
    const baseDocument = parseStyleDocument(baseScss, basePath);
    const resetDocument = parseStyleDocument(resetScss, resetPath);
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(basePath, baseDocument);
    styleDependencyGraph.record(resetPath, resetDocument);
    styleDependencyGraph.record(SCSS_PATH, buttonDocument);
    const deps = makeBaseDeps({
      styleDependencyGraph,
      styleDocumentForPath: (filePath) => {
        if (filePath === basePath) return baseDocument;
        if (filePath === resetPath) return resetDocument;
        if (filePath === SCSS_PATH) return buttonDocument;
        return null;
      },
    });

    const target = readStyleRenameTargetAtCursor(basePath, 0, 2, baseDocument, deps);
    expect(target).toEqual({ kind: "blocked", reason: "styleDependencyReferences" });
    expect(planStyleRenameAtCursor(basePath, 0, 2, baseDocument, deps, "surface")).toBeNull();
  });

  it("reads and plans same-file Sass variable rename edits", () => {
    const scss = `$gap: 1rem;
.button {
  color: $gap;
  margin: $gap;
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const deps = makeBaseDeps({
      styleDocumentForPath: (filePath) => (filePath === SCSS_PATH ? styleDocument : null),
    });

    const target = readStyleRenameTargetAtCursor(SCSS_PATH, 2, 10, styleDocument, deps);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("$gap");

    const plan = planStyleRenameAtCursor(SCSS_PATH, 2, 10, styleDocument, deps, "space");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      { uri: "file:///fake/src/Button.module.scss", newText: "$space" },
      { uri: "file:///fake/src/Button.module.scss", newText: "$space" },
      { uri: "file:///fake/src/Button.module.scss", newText: "$space" },
    ]);
  });

  it("reads and plans same-file @value declaration rename edits", () => {
    const scss = `@value primary: #fff;
@value accent: primary;
.button {
  color: primary;
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const deps = makeBaseDeps({
      styleDocumentForPath: (filePath) => (filePath === SCSS_PATH ? styleDocument : null),
    });

    const target = readStyleRenameTargetAtCursor(SCSS_PATH, 3, 10, styleDocument, deps);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("primary");

    const plan = planStyleRenameAtCursor(SCSS_PATH, 3, 10, styleDocument, deps, "brand");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      { uri: "file:///fake/src/Button.module.scss", newText: "brand" },
      { uri: "file:///fake/src/Button.module.scss", newText: "brand" },
      { uri: "file:///fake/src/Button.module.scss", newText: "brand" },
    ]);
  });

  it("renames imported @value local aliases without touching the source binding", () => {
    const scss = `@value secondary as accent from "./tokens.module.scss";
.button {
  color: accent;
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const deps = makeBaseDeps({
      styleDocumentForPath: (filePath) => (filePath === SCSS_PATH ? styleDocument : null),
    });

    const target = readStyleRenameTargetAtCursor(SCSS_PATH, 2, 10, styleDocument, deps);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("accent");

    const plan = planStyleRenameAtCursor(SCSS_PATH, 2, 10, styleDocument, deps, "brand");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      {
        uri: "file:///fake/src/Button.module.scss",
        range: { start: { line: 0, character: 20 }, end: { line: 0, character: 26 } },
        newText: "brand",
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: { start: { line: 2, character: 9 }, end: { line: 2, character: 15 } },
        newText: "brand",
      },
    ]);
  });

  it("renames source @value declarations through aliased imports", () => {
    const tokensScss = `@value secondary: #fff;\n`;
    const buttonScss = `@value secondary as accent from "./tokens.module.scss";
.button { color: accent; }
`;
    const tokensDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(TOKENS_PATH, tokensDocument);
    styleDependencyGraph.record(SCSS_PATH, buttonDocument);
    const deps = makeBaseDeps({
      styleDependencyGraph,
      styleDocumentForPath: (filePath) => {
        if (filePath === TOKENS_PATH) return tokensDocument;
        if (filePath === SCSS_PATH) return buttonDocument;
        return null;
      },
    });

    const plan = planStyleRenameAtCursor(TOKENS_PATH, 0, 8, tokensDocument, deps, "brand");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      {
        uri: "file:///fake/src/tokens.module.scss",
        range: { start: { line: 0, character: 7 }, end: { line: 0, character: 16 } },
        newText: "brand",
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: { start: { line: 0, character: 7 }, end: { line: 0, character: 16 } },
        newText: "brand",
      },
    ]);
  });

  it("renames source @value declarations through unaliased imports and importer refs", () => {
    const tokensScss = `@value secondary: #fff;\n`;
    const buttonScss = `@value secondary from "./tokens.module.scss";
.button { color: secondary; }
`;
    const tokensDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const buttonDocument = parseStyleDocument(buttonScss, SCSS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(TOKENS_PATH, tokensDocument);
    styleDependencyGraph.record(SCSS_PATH, buttonDocument);
    const deps = makeBaseDeps({
      styleDependencyGraph,
      styleDocumentForPath: (filePath) => {
        if (filePath === TOKENS_PATH) return tokensDocument;
        if (filePath === SCSS_PATH) return buttonDocument;
        return null;
      },
    });

    const plan = planStyleRenameAtCursor(TOKENS_PATH, 0, 8, tokensDocument, deps, "brand");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      {
        uri: "file:///fake/src/tokens.module.scss",
        range: { start: { line: 0, character: 7 }, end: { line: 0, character: 16 } },
        newText: "brand",
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: { start: { line: 0, character: 7 }, end: { line: 0, character: 16 } },
        newText: "brand",
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: { start: { line: 1, character: 17 }, end: { line: 1, character: 26 } },
        newText: "brand",
      },
    ]);
  });

  it("reads and plans same-file Less variable rename edits", () => {
    const lessPath = "/fake/src/Button.module.less";
    const less = `@gap: 1rem;
.button {
  color: @gap;
  margin: @gap;
}
`;
    const styleDocument = parseStyleDocument(less, lessPath);
    const deps = makeBaseDeps({
      styleDocumentForPath: (filePath) => (filePath === lessPath ? styleDocument : null),
    });

    const target = readStyleRenameTargetAtCursor(lessPath, 2, 10, styleDocument, deps);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("@gap");

    const plan = planStyleRenameAtCursor(lessPath, 2, 10, styleDocument, deps, "space");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      { uri: "file:///fake/src/Button.module.less", newText: "@space" },
      { uri: "file:///fake/src/Button.module.less", newText: "@space" },
      { uri: "file:///fake/src/Button.module.less", newText: "@space" },
    ]);
  });

  it("reads and plans same-file Sass mixin rename edits", () => {
    const scss = `@mixin raised() {}
.button {
  @include raised();
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const deps = makeBaseDeps({
      styleDocumentForPath: (filePath) => (filePath === SCSS_PATH ? styleDocument : null),
    });

    const target = readStyleRenameTargetAtCursor(SCSS_PATH, 2, 13, styleDocument, deps);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("raised");

    const plan = planStyleRenameAtCursor(SCSS_PATH, 2, 13, styleDocument, deps, "elevated");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      { uri: "file:///fake/src/Button.module.scss", newText: "elevated" },
      { uri: "file:///fake/src/Button.module.scss", newText: "elevated" },
    ]);
  });

  it("reads and plans same-file Sass function rename edits", () => {
    const scss = `@function tone($value) { @return $value; }
.button {
  color: tone(red);
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const deps = makeBaseDeps({
      styleDocumentForPath: (filePath) => (filePath === SCSS_PATH ? styleDocument : null),
    });

    const target = readStyleRenameTargetAtCursor(SCSS_PATH, 2, 10, styleDocument, deps);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("tone");

    const plan = planStyleRenameAtCursor(SCSS_PATH, 2, 10, styleDocument, deps, "theme-tone");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      { uri: "file:///fake/src/Button.module.scss", newText: "theme-tone" },
      { uri: "file:///fake/src/Button.module.scss", newText: "theme-tone" },
    ]);
  });

  it("renames local Sass variables without touching same-name file-scope variables", () => {
    const scss = `$gap: 1rem;
.one {
  $gap: 2rem;
  color: $gap;
}
.two {
  color: $gap;
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const deps = makeBaseDeps({
      styleDocumentForPath: (filePath) => (filePath === SCSS_PATH ? styleDocument : null),
    });

    const plan = planStyleRenameAtCursor(SCSS_PATH, 3, 10, styleDocument, deps, "space");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits : []).toMatchObject([
      {
        uri: "file:///fake/src/Button.module.scss",
        range: { start: { line: 2, character: 2 }, end: { line: 2, character: 6 } },
        newText: "$space",
      },
      {
        uri: "file:///fake/src/Button.module.scss",
        range: { start: { line: 3, character: 9 }, end: { line: 3, character: 13 } },
        newText: "$space",
      },
    ]);
  });

  it("uses rust selector-usage payloads for rename safety blocking", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
      workspaceRoot: "/fake",
    });
    const styleDocument = deps.styleDocumentForPath(SCSS_PATH);

    expect(styleDocument).not.toBeNull();
    const target = readStyleRenameTargetAtCursor(SCSS_PATH, 1, 3, styleDocument!, deps, {
      env: { CME_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
      readRustSelectorUsagePayloadForWorkspaceTarget: () => ({
        canonicalName: "indicator",
        totalReferences: 2,
        directReferenceCount: 1,
        editableDirectReferenceCount: 1,
        exactReferenceCount: 1,
        inferredOrBetterReferenceCount: 2,
        hasExpandedReferences: true,
        hasStyleDependencyReferences: false,
        hasAnyReferences: true,
      }),
    });

    expect(target).toEqual({
      kind: "blocked",
      reason: "expandedReferences",
    });
  });

  it("uses rust style semantic graph selector identity to block unsafe rename targets", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
      workspaceRoot: "/fake",
    });
    const styleDocument = deps.styleDocumentForPath(SCSS_PATH);

    expect(styleDocument).not.toBeNull();
    const target = readStyleRenameTargetAtCursor(SCSS_PATH, 1, 3, styleDocument!, deps, {
      env: { CME_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
      readRustStyleSemanticGraphForWorkspaceTarget: () => makeGraph("blocked"),
      readRustSelectorUsagePayloadForWorkspaceTarget: () => null,
    });

    expect(target).toEqual({
      kind: "blocked",
      reason: "unsafeSelectorShape",
    });
  });

  it("uses rust style semantic graph reference sites for direct source rewrite edits", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
      workspaceRoot: "/fake",
    });
    const styleDocument = deps.styleDocumentForPath(SCSS_PATH);

    expect(styleDocument).not.toBeNull();
    const plan = planStyleRenameAtCursor(SCSS_PATH, 1, 3, styleDocument!, deps, "status", {
      env: { CME_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
      readRustStyleSemanticGraphForWorkspaceTarget: () => makeGraph("safe"),
      readRustSelectorUsagePayloadForWorkspaceTarget: () => null,
    });

    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits.map((edit) => edit.newText) : []).toEqual([
      "status",
      "status",
    ]);
    expect(plan?.kind === "plan" ? plan.plan.edits[1]?.uri : null).toBe("file:///fake/src/App.tsx");
  });

  it("uses rust selector-usage payloads for direct source rewrite sites", () => {
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
      workspaceRoot: "/fake",
    });
    const styleDocument = deps.styleDocumentForPath(SCSS_PATH);

    expect(styleDocument).not.toBeNull();
    const plan = planStyleRenameAtCursor(SCSS_PATH, 1, 3, styleDocument!, deps, "status", {
      env: { CME_SELECTED_QUERY_BACKEND: "rust-selector-usage" } as NodeJS.ProcessEnv,
      readRustSelectorUsagePayloadForWorkspaceTarget: () => ({
        canonicalName: "indicator",
        totalReferences: 1,
        directReferenceCount: 1,
        editableDirectReferenceCount: 1,
        exactReferenceCount: 1,
        inferredOrBetterReferenceCount: 1,
        hasExpandedReferences: false,
        hasStyleDependencyReferences: false,
        hasAnyReferences: true,
        editableDirectSites: [
          {
            filePath: "/fake/src/App.tsx",
            range: {
              start: { line: 10, character: 10 },
              end: { line: 10, character: 19 },
            },
            className: "indicator",
          },
        ],
      }),
    });

    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits.map((edit) => edit.newText) : []).toEqual([
      "status",
      "status",
    ]);
    expect(plan?.kind === "plan" ? plan.plan.edits[1]?.uri : null).toBe("file:///fake/src/App.tsx");
  });
});

function makeGraph(rewriteSafety: "safe" | "blocked"): StyleSemanticGraphSummaryV0 {
  return {
    schemaVersion: "0",
    product: "omena-semantic.style-semantic-graph",
    language: "scss",
    parserFacts: {},
    semanticFacts: {},
    selectorIdentityEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-identity",
      canonicalIdCount: 1,
      canonicalIds: [
        {
          canonicalId: "selector:indicator",
          localName: "indicator",
          identityKind: "localClass",
          rewriteSafety,
          blockers: rewriteSafety === "blocked" ? ["nested-expansion"] : [],
        },
      ],
      rewriteSafety: {
        allCanonicalIdsRewriteSafe: rewriteSafety === "safe",
        safeCanonicalIds: rewriteSafety === "safe" ? ["selector:indicator"] : [],
        blockedCanonicalIds: rewriteSafety === "blocked" ? ["selector:indicator"] : [],
        blockers: rewriteSafety === "blocked" ? ["nested-expansion"] : [],
      },
    },
    selectorReferenceEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-references",
      stylePath: SCSS_PATH,
      selectorCount: 1,
      referencedSelectorCount: 1,
      unreferencedSelectorCount: 0,
      totalReferenceSites: 1,
      selectors: [
        {
          canonicalId: "selector:indicator",
          filePath: SCSS_PATH,
          localName: "indicator",
          totalReferences: 1,
          directReferenceCount: 1,
          editableDirectReferenceCount: 1,
          exactReferenceCount: 1,
          inferredOrBetterReferenceCount: 1,
          hasExpandedReferences: false,
          hasStyleDependencyReferences: false,
          hasAnyReferences: true,
          sites: [
            {
              filePath: "/fake/src/App.tsx",
              range: {
                start: { line: 10, character: 10 },
                end: { line: 10, character: 19 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
          ],
          editableDirectSites: [
            {
              filePath: "/fake/src/App.tsx",
              range: {
                start: { line: 10, character: 10 },
                end: { line: 10, character: 19 },
              },
              className: "indicator",
            },
          ],
        },
      ],
    },
    sourceInputEvidence: {},
    promotionEvidence: {},
    losslessCstContract: {},
  };
}

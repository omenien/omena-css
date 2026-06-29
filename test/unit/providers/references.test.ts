import { describe, expect, it, vi } from "vitest";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { findSelectorAtCursor } from "../../../server/engine-core-ts/src/core/query";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { WorkspaceStyleDependencyGraph } from "../../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { handleReferences } from "../../../server/lsp-server/src/providers/references";
import {
  cursorFixture,
  textDocumentPositionFixture,
  textDocumentPositionFromCursor,
  workspace,
  type CmeWorkspace,
  type Range,
} from "../../../packages/vitest-cme/src";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  infoAtLine,
  makeBaseDeps,
  semanticSiteAt,
} from "../../_fixtures/test-helpers";
import {
  buildStyleDocumentFromSelectorMap,
  expandSelectorMapWithTransform,
  parseStyleSelectorMap,
} from "../../_fixtures/style-documents";

function makeDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
  return makeBaseDeps({
    selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 5)]]),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

const SOURCE_PATH = "/fake/src/App.tsx";
const SOURCE_URI = "file:///fake/src/App.tsx";
const STYLE_PATH = "/fake/src/Button.module.scss";
const STYLE_URI = "file:///fake/src/Button.module.scss";
const NON_STYLE_PATH = "/fake/src/Button.tsx";
const NON_STYLE_URI = "file:///fake/src/Button.tsx";
const NON_STYLE_WORKSPACE = workspace({
  [NON_STYLE_PATH]: "/*|*/const value = 1;\n",
});
const SOURCE_WORKSPACE = workspace({
  [SOURCE_PATH]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const /*<binding>*/cx/*</binding>*/ = classNames.bind(styles);
const a = cx('/*<class>*/ind/*|*/icator/*</class>*/');
`,
});
const STYLE_WORKSPACE = workspace({
  [STYLE_PATH]: `
.unused {}
.unused {}
.unused {}
.unused {}
./*|*/indicator {}
`,
});
const NO_SELECTOR_STYLE_WORKSPACE = workspace({
  [STYLE_PATH]: "/*|*/.unused {}\n",
});
const SOURCE_BINDING_RANGE = SOURCE_WORKSPACE.range("binding", SOURCE_PATH).range;
const SOURCE_CLASS_RANGE = SOURCE_WORKSPACE.range("class", SOURCE_PATH).range;

const SOURCE_BINDING: CxBinding = {
  cxVarName: "cx",
  stylesVarName: "styles",
  scssModulePath: STYLE_PATH,
  classNamesImportName: "classNames",
  bindingRange: SOURCE_BINDING_RANGE,
};

function sourceCursor(fixture: CmeWorkspace = SOURCE_WORKSPACE, markerName = "cursor") {
  return cursorFixture({
    workspace: fixture,
    filePath: SOURCE_PATH,
    documentUri: SOURCE_URI,
    markerName,
    version: 1,
  });
}

function sourceReferenceParams(cursor = sourceCursor()) {
  return {
    ...textDocumentPositionFromCursor(cursor),
    context: { includeDeclaration: false },
  };
}

function styleReferenceParams(
  fixture: CmeWorkspace = STYLE_WORKSPACE,
  options: {
    readonly uri?: string;
    readonly markerName?: string;
    readonly filePath?: string;
    readonly includeDeclaration?: boolean;
  } = {},
) {
  return {
    ...textDocumentPositionFixture({
      workspace: fixture,
      documentUri: options.uri ?? STYLE_URI,
      markerName: options.markerName ?? "cursor",
      filePath: options.filePath,
    }),
    context: { includeDeclaration: options.includeDeclaration ?? true },
  };
}

function makeSourceDeps(expressionRange: Range = SOURCE_CLASS_RANGE): ProviderDeps {
  const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
  semanticReferenceIndex.record(SOURCE_URI, [
    semanticSiteAt(SOURCE_URI, "indicator", expressionRange.start.line, STYLE_PATH, "indicator", {
      start: expressionRange.start.character,
      end: expressionRange.end.character,
    }),
  ]);

  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: () => ({
      stylesBindings: new Map([
        ["styles", { kind: "resolved" as const, absolutePath: SOURCE_BINDING.scssModulePath }],
      ]),
      bindings: [SOURCE_BINDING],
    }),
    parseClassExpressions: (_sf, bindings) =>
      buildTestClassExpressions({
        filePath: SOURCE_PATH,
        bindings,
        expressions: [
          {
            kind: "literal",
            origin: "cxCall",
            className: "indicator",
            range: expressionRange,
            scssModulePath: SOURCE_BINDING.scssModulePath,
          },
        ],
      }),
  });
  const analysisCache = new DocumentAnalysisCache({
    sourceFrontendAnalysis,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    max: 10,
  });

  return makeBaseDeps({
    analysisCache,
    semanticReferenceIndex,
    selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
    workspaceRoot: "/fake",
  });
}

describe("handleReferences", () => {
  it("returns null for non-style files", () => {
    const result = handleReferences(
      styleReferenceParams(NON_STYLE_WORKSPACE, { uri: NON_STYLE_URI }),
      makeDeps(),
    );
    expect(result).toBeNull();
  });

  it("returns source-side locations when invoked from a TSX class expression cursor", () => {
    const cursor = sourceCursor();
    const result = handleReferences(
      sourceReferenceParams(cursor),
      makeSourceDeps(SOURCE_CLASS_RANGE),
      cursor,
    );

    expect(result).toEqual([
      {
        uri: SOURCE_URI,
        range: SOURCE_CLASS_RANGE,
      },
    ]);
  });

  it("returns null when cursor is not on a class selector", () => {
    const result = handleReferences(styleReferenceParams(NO_SELECTOR_STYLE_WORKSPACE), makeDeps());
    expect(result).toBeNull();
  });

  it("returns Location[] when references exist", () => {
    const idx = new WorkspaceSemanticWorkspaceReferenceIndex();
    idx.record("file:///fake/src/App.tsx", [
      semanticSiteAt("file:///fake/src/App.tsx", "indicator", 10, "/fake/src/Button.module.scss"),
    ]);
    const result = handleReferences(
      styleReferenceParams(),
      makeDeps({ semanticReferenceIndex: idx }),
    );
    expect(result).not.toBeNull();
    expect(result).toHaveLength(1);
    expect(result![0]!.uri).toBe("file:///fake/src/App.tsx");
  });

  it("prefers semantic reference sites when available", () => {
    const idx = new WorkspaceSemanticWorkspaceReferenceIndex();
    const semanticWorkspace = workspace({
      [SOURCE_PATH]: "const a = cx('/*<class>*/indicator/*</class>*/');",
    });
    const expectedRange = semanticWorkspace.range("class", SOURCE_PATH).range;
    const expectedSite = semanticSiteAt(
      "file:///fake/src/App.tsx",
      "indicator",
      expectedRange.start.line,
      "/fake/src/Button.module.scss",
      "indicator",
      {
        start: expectedRange.start.character,
        end: expectedRange.end.character,
      },
    );
    idx.record("file:///fake/src/App.tsx", [expectedSite]);
    const result = handleReferences(
      styleReferenceParams(),
      makeDeps({ semanticReferenceIndex: idx }),
    );

    expect(result).not.toBeNull();
    expect(result).toHaveLength(1);
    expect(result![0]).toEqual({
      uri: "file:///fake/src/App.tsx",
      range: expectedRange,
    });
  });

  // Find-references keeps expanded sites. Rename filters them;
  // Find Refs does not.
  it("find-references STILL surfaces template-expanded sites", () => {
    const SCSS_PATH = "/fake/src/Button.module.scss";
    const SCSS_URI = "file:///fake/src/Button.module.scss";
    const TEMPLATE_URI = "file:///fake/src/App.tsx";
    const TEMPLATE_WORKSPACE = workspace({
      [SOURCE_PATH]: `const a = cx(/*<template>*/\`btn-\${weight}\`/*</template>*/);`,
    });
    const TEMPLATE_RANGE = TEMPLATE_WORKSPACE.range("template", SOURCE_PATH).range;

    const idx = new WorkspaceSemanticWorkspaceReferenceIndex();
    idx.record(TEMPLATE_URI, [
      semanticSiteAt(TEMPLATE_URI, "btn-small", TEMPLATE_RANGE.start.line, SCSS_PATH, "btn-small", {
        start: TEMPLATE_RANGE.start.character,
        end: TEMPLATE_RANGE.end.character,
        certainty: "inferred",
        reason: "templatePrefix",
      }),
      semanticSiteAt(TEMPLATE_URI, "btn-large", TEMPLATE_RANGE.start.line, SCSS_PATH, "btn-large", {
        start: TEMPLATE_RANGE.start.character,
        end: TEMPLATE_RANGE.end.character,
        certainty: "inferred",
        reason: "templatePrefix",
      }),
    ]);

    const result = handleReferences(
      styleReferenceParams(workspace({ [SCSS_PATH]: "\n./*|*/btn-small {}\n.btn-large {}\n" }), {
        uri: SCSS_URI,
        filePath: SCSS_PATH,
      }),
      makeBaseDeps({
        selectorMapForPath: () =>
          new Map([
            ["btn-small", infoAtLine("btn-small", 1)],
            ["btn-large", infoAtLine("btn-large", 3)],
          ]),
        workspaceRoot: "/fake",
        semanticReferenceIndex: idx,
      }),
    );

    expect(result).not.toBeNull();
    expect(result!.length).toBeGreaterThanOrEqual(1);
    const matched = result!.find(
      (loc) =>
        loc.uri === TEMPLATE_URI &&
        loc.range.start.line === TEMPLATE_RANGE.start.line &&
        loc.range.start.character === TEMPLATE_RANGE.start.character,
    );
    expect(matched).toBeDefined();
  });

  it("includes CSS-side references that arrive through composes reachability", () => {
    const BASE_PATH = "/fake/src/base.module.scss";
    const BASE_URI = "file:///fake/src/base.module.scss";
    const BUTTON_PATH = "/fake/src/button.module.scss";
    const graph = new WorkspaceStyleDependencyGraph();
    graph.record(
      BUTTON_PATH,
      buildStyleDocumentFromSelectorMap(
        BUTTON_PATH,
        new Map([
          [
            "button",
            {
              ...infoAtLine("button", 5),
              composes: [{ classNames: ["base"], from: "./base.module.scss" }],
            },
          ],
        ]),
      ),
    );

    const result = handleReferences(
      styleReferenceParams(workspace({ [BASE_PATH]: "\n\n\n\n\n./*|*/base {}\n" }), {
        uri: BASE_URI,
        filePath: BASE_PATH,
      }),
      makeBaseDeps({
        selectorMapForPath: (path) => {
          if (path === BASE_PATH) return new Map([["base", infoAtLine("base", 5)]]);
          if (path === BUTTON_PATH) return new Map([["button", infoAtLine("button", 5)]]);
          return null;
        },
        workspaceRoot: "/fake",
        styleDependencyGraph: graph,
      }),
    );

    expect(result).not.toBeNull();
    const buttonInfo = infoAtLine("button", 5);
    expect(result).toContainEqual({
      uri: "file:///fake/src/button.module.scss",
      range: buttonInfo.range,
    });
  });

  // findSelectorAtCursor prefers the BEM-suffix range when present.
  // Cursor on `&--primary` resolves to the nested class entry that
  // the resolved-name fallback range would miss.
  it("findSelectorAtCursor prefers bemSuffix.rawTokenRange over resolved range", () => {
    const fixture = workspace({
      [STYLE_PATH]: `.button {
  /*<raw>*/&--primary/*</raw>*/ {}
}
`,
    });
    const rawTokenRange = fixture.range("raw", STYLE_PATH).range;
    const resolvedName = "button--primary";
    const resolvedRange: Range = {
      start: rawTokenRange.start,
      end: {
        line: rawTokenRange.start.line,
        character: rawTokenRange.start.character + resolvedName.length,
      },
    };
    // Fixture: `.button { &--primary {} }` on two lines.
    // Line 0: `.button {`
    // Line 1: `  &--primary {}`
    // The synthesized resolved `range` points at a fallback column
    // on line 1, but it only covers `button--primary`'s ghost span
    // — the cursor on the `&` column (line 1, char 2) falls INSIDE
    // bemSuffix.rawTokenRange {start:{line:1,char:2}, end:{line:1,char:12}}.
    const selector = {
      ...infoAtLine(resolvedName, rawTokenRange.start.line),
      range: resolvedRange,
      bemSuffix: {
        rawTokenRange,
        rawToken: "&--primary",
        parentResolvedName: "button",
      },
      nestedSafety: "bemSuffixSafe" as const,
    };
    const classMap = new Map([["button--primary", selector]]);
    const styleDocument = buildStyleDocumentFromSelectorMap(
      "/fake/src/Button.module.scss",
      classMap,
    );

    // Cursor on the `&` character at (line 1, character 2). The
    // rawTokenRange covers exactly this position; the test locks
    // down that findSelectorAtCursor prefers it.
    const hit = findSelectorAtCursor(
      styleDocument,
      rawTokenRange.start.line,
      rawTokenRange.start.character,
    );
    expect(hit).not.toBeNull();
    expect(hit!.name).toBe(resolvedName);

    // Cursor past the rawTokenRange's end (character 11 is the
    // last char `y`; 12 is still INCLUSIVE at the end per the
    // codebase's rangeContains convention). Character 13 is past.
    const miss = findSelectorAtCursor(
      styleDocument,
      rawTokenRange.end.line,
      rawTokenRange.end.character + 1,
    );
    expect(miss).toBeNull();
  });

  it("classnameTransform: finds alias-form TSX access from SCSS cursor on original selector", async () => {
    const SCSS_PATH = "/fake/Button.module.scss";
    const SCSS_URI = "file:///fake/Button.module.scss";
    const fixture = workspace({
      [SCSS_PATH]: `./*|*/btn-primary { color: red; }`,
    });
    const base = parseStyleSelectorMap(fixture.file(SCSS_PATH).content, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCase");

    const idx = new WorkspaceSemanticWorkspaceReferenceIndex();
    idx.record("file:///fake/App.tsx", [
      semanticSiteAt("file:///fake/App.tsx", "btnPrimary", 5, SCSS_PATH, "btn-primary", {
        reason: "styleAccess",
        origin: "styleAccess",
      }),
    ]);

    // Cursor sits on `.btn-primary` in the SCSS file. Under camelCase
    // mode the class map holds both `btn-primary` (original) and
    // `btnPrimary` (alias) entries. The semantic index stores the
    // alias access under the canonical `btn-primary` selector, so
    // the provider must route through `originalName` to find it.
    const result = handleReferences(
      styleReferenceParams(fixture, { uri: SCSS_URI }),
      makeBaseDeps({
        selectorMapForPath: () => classMap,
        workspaceRoot: "/fake",
        semanticReferenceIndex: idx,
      }),
    );

    expect(result).not.toBeNull();
    expect(result).toHaveLength(1);
    expect(result![0]!.uri).toBe("file:///fake/App.tsx");
  });

  it("classnameTransform (camelCaseOnly): alias-only entry still resolves to the canonical selector bucket", async () => {
    const SCSS_PATH = "/fake/Button.module.scss";
    const SCSS_URI = "file:///fake/Button.module.scss";
    const fixture = workspace({
      [SCSS_PATH]: `./*|*/btn-primary { color: red; }`,
    });
    const base = parseStyleSelectorMap(fixture.file(SCSS_PATH).content, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCaseOnly");
    // Under `camelCaseOnly` only the alias entry remains in the map;
    // the cursor falls on the alias entry's range but its
    // `originalName` still points at `btn-primary`.
    expect(classMap.has("btn-primary")).toBe(false);
    expect(classMap.has("btnPrimary")).toBe(true);

    const idx = new WorkspaceSemanticWorkspaceReferenceIndex();
    idx.record("file:///fake/App.tsx", [
      semanticSiteAt("file:///fake/App.tsx", "btnPrimary", 7, SCSS_PATH, "btn-primary", {
        reason: "styleAccess",
        origin: "styleAccess",
      }),
    ]);

    const result = handleReferences(
      styleReferenceParams(fixture, { uri: SCSS_URI }),
      makeBaseDeps({
        selectorMapForPath: () => classMap,
        workspaceRoot: "/fake",
        semanticReferenceIndex: idx,
      }),
    );

    expect(result).not.toBeNull();
    expect(result).toHaveLength(1);
    expect(result![0]!.uri).toBe("file:///fake/App.tsx");
  });

  it("logs and returns null on exception", () => {
    const logError = vi.fn();
    const result = handleReferences(
      styleReferenceParams(),
      makeDeps({
        selectorMapForPath: () => {
          throw new Error("boom");
        },
        logError,
      }),
    );
    expect(result).toBeNull();
    expect(logError).toHaveBeenCalledTimes(1);
  });
});

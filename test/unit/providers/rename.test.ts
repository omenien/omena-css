import { describe, expect, it, vi } from "vitest";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { findSelectorAtCursor } from "../../../server/engine-core-ts/src/core/query";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import type {
  CursorParams,
  ProviderDeps,
} from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { handlePrepareRename, handleRename } from "../../../server/lsp-server/src/providers/rename";
import { DEFAULT_SETTINGS } from "../../../server/engine-core-ts/src/settings";
import type { Settings } from "../../../server/engine-core-ts/src/settings";
import {
  cursorFixture,
  scenario,
  targetFixture,
  textDocumentPositionFixture,
  textDocumentPositionFromCursor,
  textDocumentRenameFromCursor,
  textDocumentRenameFixture,
  workspace,
  type CmeWorkspace,
  type Range,
} from "../../../packages/vitest-cme/src";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  infoAtLine as info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";
import {
  buildStyleDocumentFromSelectorMap,
  expandSelectorMapWithTransform,
  parseStyleSelectorMap,
} from "../../_fixtures/style-documents";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";

const SCSS_PATH = "/fake/src/Button.module.scss";
const SCSS_URI = "file:///fake/src/Button.module.scss";
const DEFAULT_STYLE_WORKSPACE = workspace({
  [SCSS_PATH]: `
./*|*/indicator {}

.active {}
`,
});
const NO_SELECTOR_STYLE_WORKSPACE = workspace({
  [SCSS_PATH]: "/*|*/.unused {}\n",
});
const NON_STYLE_PATH = "/fake/src/Button.tsx";
const NON_STYLE_URI = "file:///fake/src/Button.tsx";
const NON_STYLE_WORKSPACE = workspace({
  [NON_STYLE_PATH]: "/*|*/const value = 1;\n",
});
const SEMANTIC_SOURCE_PATH = "/fake/src/SemanticSite.tsx";

function semanticSourceRange(sourceToken: string): Range {
  return workspace({
    [SEMANTIC_SOURCE_PATH]: `const value = cx('/*<class>*/${sourceToken}/*</class>*/');\n`,
  }).range("class", SEMANTIC_SOURCE_PATH).range;
}

function semanticSite(args: {
  uri: string;
  canonicalName: string;
  className?: string;
  line?: number;
  start?: number;
  end?: number;
  certainty?: "exact" | "inferred" | "possible";
  reason?: "literal" | "styleAccess" | "templatePrefix" | "typeUnion";
  origin?: "cxCall" | "styleAccess";
  expansion?: "direct" | "expanded";
}) {
  const certainty = args.certainty ?? "exact";
  const sourceToken = args.className ?? args.canonicalName;
  const fallbackRange = semanticSourceRange(sourceToken);
  const line = args.line ?? fallbackRange.start.line;
  const start = args.start ?? fallbackRange.start.character;
  const range: Range = {
    start: { line, character: start },
    end: { line, character: args.end ?? fallbackRange.end.character },
  };
  const expansion = args.expansion ?? (certainty === "exact" ? "direct" : "expanded");
  return {
    refId: `ref:${args.uri}:${line}:${start}`,
    selectorId: `selector:${SCSS_PATH}:${args.canonicalName}`,
    filePath: args.uri.replace("file://", ""),
    uri: args.uri,
    range,
    origin: args.origin ?? "cxCall",
    scssModulePath: SCSS_PATH,
    selectorFilePath: SCSS_PATH,
    canonicalName: args.canonicalName,
    className: args.className ?? args.canonicalName,
    certainty,
    reason: args.reason ?? "literal",
    expansion,
  } as const;
}

function makeDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
  return makeBaseDeps({
    selectorMapForPath: () =>
      new Map([
        ["indicator", info("indicator", 1)],
        ["active", info("active", 3)],
      ]),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

function expectPrepareRenameBlocked(
  run: () => ReturnType<typeof handlePrepareRename>,
  message: string,
): void {
  expect(run).toThrow(message);
}

function styleWorkspace(filePath: string, content: string): CmeWorkspace {
  return workspace({ [filePath]: content });
}

function styleClassMap(fixture: CmeWorkspace, filePath: string) {
  return parseStyleSelectorMap(fixture.file(filePath).content, filePath);
}

function stylePositionParams(
  fixture: CmeWorkspace,
  uri: string,
  markerName = "cursor",
  filePath?: string,
) {
  return textDocumentPositionFixture({
    workspace: fixture,
    documentUri: uri,
    markerName,
    filePath,
  });
}

function styleRenameParams(
  fixture: CmeWorkspace,
  uri: string,
  newName: string,
  markerName = "cursor",
  filePath?: string,
) {
  return textDocumentRenameFixture({
    workspace: fixture,
    documentUri: uri,
    markerName,
    filePath,
    newName,
  });
}

describe("handlePrepareRename", () => {
  it("returns range and placeholder for a selector at cursor in SCSS file", () => {
    const result = handlePrepareRename(
      stylePositionParams(DEFAULT_STYLE_WORKSPACE, SCSS_URI),
      makeDeps(),
    );
    expect(result).not.toBeNull();
    expect(result).toHaveProperty("range");
    expect(result).toHaveProperty("placeholder", "indicator");
  });

  it("returns null when cursor is not on a selector", () => {
    const result = handlePrepareRename(
      stylePositionParams(NO_SELECTOR_STYLE_WORKSPACE, SCSS_URI),
      makeDeps(),
    );
    expect(result).toBeNull();
  });

  it("returns null for non-style files", () => {
    const result = handlePrepareRename(
      stylePositionParams(NON_STYLE_WORKSPACE, NON_STYLE_URI),
      makeDeps(),
    );
    expect(result).toBeNull();
  });

  it("rejects nested child selectors that parse as standalone classes", () => {
    const fixture = styleWorkspace(
      SCSS_PATH,
      `
.button {
  .c/*|*/hild {
    color: red;
  }
}
`,
    );
    const styleDocument = buildStyleDocumentFromSelectorMap(
      SCSS_PATH,
      expandSelectorMapWithTransform(
        parseStyleSelectorMap(fixture.file(SCSS_PATH).content, SCSS_PATH),
        "asIs",
      ),
    );
    expect(() =>
      handlePrepareRename(
        stylePositionParams(fixture, SCSS_URI),
        makeDeps({ styleDocumentForPath: () => styleDocument }),
      ),
    ).toThrow("Only flat selectors and safe BEM suffix selectors can be renamed.");
  });
});

describe("handleRename", () => {
  it("renames a Sass module member reference across the target declaration and incoming use site", () => {
    const tokensPath = "/fake/src/_tokens.scss";
    const tokensUri = "file:///fake/src/_tokens.scss";
    const fixture = workspace({
      [tokensPath]: `$brand: red;\n`,
      [SCSS_PATH]: `@use "./tokens" as tokens;\n.button { color: tokens.$br/*at:ref*/and; }\n`,
    });
    const tokensDocument = parseStyleDocument(fixture.file(tokensPath).content, tokensPath);
    const sourceDocument = parseStyleDocument(fixture.file(SCSS_PATH).content, SCSS_PATH);
    const styleContents = new Map([
      [tokensPath, fixture.file(tokensPath).content],
      [SCSS_PATH, fixture.file(SCSS_PATH).content],
    ]);
    const styleDocuments = new Map([
      [tokensPath, tokensDocument],
      [SCSS_PATH, sourceDocument],
    ]);
    const incomingRef = sourceDocument.sassModuleMemberRefs[0]!;

    const deps = makeDeps({
      styleDocumentForPath: (path) => styleDocuments.get(path) ?? null,
      readStyleFile: (path) => styleContents.get(path) ?? null,
      styleDependencyGraph: {
        record: vi.fn(),
        forget: vi.fn(),
        forgetWithinRoot: vi.fn(),
        getIncoming: () => [],
        getOutgoing: () => [],
        getIncomingSassModuleMemberRefs: () => [
          {
            filePath: SCSS_PATH,
            namespace: "tokens",
            symbolKind: "variable",
            name: "brand",
            range: incomingRef.range,
          },
        ],
        getAllCustomPropertyDecls: () => [],
        getCustomPropertyDecls: () => [],
        getCustomPropertyRefs: () => [],
      },
    });

    const prep = handlePrepareRename(stylePositionParams(fixture, SCSS_URI, "ref"), deps);
    expect(prep?.placeholder).toBe("$brand");

    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "accent", "ref"), deps);

    expect(result?.changes?.[tokensUri]).toEqual([
      {
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 6 },
        },
        newText: "$accent",
      },
    ]);
    expect(result?.changes?.[SCSS_URI]).toEqual([
      {
        range: incomingRef.range,
        newText: "$accent",
      },
    ]);
  });

  it("builds WorkspaceEdit with SCSS selector and TS/TSX reference edits", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "indicator",
      }),
    ]);
    semanticReferenceIndex.record("file:///fake/src/Other.tsx", [
      semanticSite({
        uri: "file:///fake/src/Other.tsx",
        canonicalName: "indicator",
      }),
    ]);
    const result = handleRename(
      styleRenameParams(DEFAULT_STYLE_WORKSPACE, SCSS_URI, "status"),
      makeDeps({ semanticReferenceIndex }),
    );
    expect(result).not.toBeNull();
    const changes = result!.changes!;

    // SCSS file edit
    expect(changes[SCSS_URI]).toHaveLength(1);
    expect(changes[SCSS_URI]![0]!.newText).toBe("status");

    // TS/TSX edits
    expect(changes["file:///fake/src/App.tsx"]).toHaveLength(1);
    expect(changes["file:///fake/src/App.tsx"]![0]!.newText).toBe("status");
    expect(changes["file:///fake/src/Other.tsx"]).toHaveLength(1);
    expect(changes["file:///fake/src/Other.tsx"]![0]!.newText).toBe("status");
  });

  it("returns WorkspaceEdit with only SCSS edit when no references exist", () => {
    const result = handleRename(
      styleRenameParams(DEFAULT_STYLE_WORKSPACE, SCSS_URI, "status"),
      makeDeps(),
    );
    expect(result).not.toBeNull();
    const changes = result!.changes!;
    expect(Object.keys(changes)).toEqual([SCSS_URI]);
    expect(changes[SCSS_URI]).toHaveLength(1);
  });

  it("returns null when cursor is not on a selector", () => {
    const result = handleRename(
      styleRenameParams(NO_SELECTOR_STYLE_WORKSPACE, SCSS_URI, "status"),
      makeDeps(),
    );
    expect(result).toBeNull();
  });

  it("returns null for non-style files (TS/TSX side not yet wired)", () => {
    const result = handleRename(
      styleRenameParams(NON_STYLE_WORKSPACE, NON_STYLE_URI, "status"),
      makeDeps(),
    );
    expect(result).toBeNull();
  });
});

it("logs and returns null on exception in prepareRename", () => {
  const logError = vi.fn();
  const result = handlePrepareRename(
    stylePositionParams(DEFAULT_STYLE_WORKSPACE, SCSS_URI),
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

it("logs and returns null on exception in rename", () => {
  const logError = vi.fn();
  const result = handleRename(
    styleRenameParams(DEFAULT_STYLE_WORKSPACE, SCSS_URI, "status"),
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

// ── TS/TSX side tests ──

const TSX_PATH = "/fake/src/App.tsx";
const TSX_URI = "file:///fake/src/App.tsx";
const TSX_WORKSPACE = workspace({
  [TSX_PATH]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const /*<binding>*/cx/*</binding>*/ = classNames.bind(styles);
const a = cx('/*<class>*/ind/*|*/icator/*</class>*/');
`,
});
const TSX_BINDING_RANGE = TSX_WORKSPACE.range("binding", TSX_PATH).range;
const TSX_CLASS_RANGE = TSX_WORKSPACE.range("class", TSX_PATH).range;

const BINDING: CxBinding = {
  cxVarName: "cx",
  stylesVarName: "styles",
  scssModulePath: "/fake/src/Button.module.scss",
  classNamesImportName: "classNames",
  bindingRange: TSX_BINDING_RANGE,
};

function makeTsxDeps(
  overrides: Partial<ProviderDeps> = {},
  expressionRange: Range = TSX_CLASS_RANGE,
): ProviderDeps {
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: () => ({
      stylesBindings: new Map([
        ["styles", { kind: "resolved" as const, absolutePath: BINDING.scssModulePath }],
      ]),
      bindings: [
        {
          ...BINDING,
          bindingRange: TSX_BINDING_RANGE,
        },
      ],
    }),
    parseClassExpressions: (_sf, bindings) =>
      buildTestClassExpressions({
        filePath: TSX_PATH,
        bindings,
        expressions:
          bindings.length === 0
            ? []
            : [
                {
                  kind: "literal",
                  origin: "cxCall",
                  className: "indicator",
                  range: expressionRange,
                  scssModulePath: bindings[0]!.scssModulePath,
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
    selectorMapForPath: () => new Map([["indicator", info("indicator", 1)]]),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

function sourceCursorParams(
  fixture: CmeWorkspace,
  markerName = "cursor",
  filePath = TSX_PATH,
  documentUri = TSX_URI,
): CursorParams {
  return cursorFixture({
    workspace: fixture,
    filePath,
    documentUri,
    markerName,
    version: 1,
  });
}

describe("handlePrepareRename from TS/TSX", () => {
  it("returns range and placeholder for cursor on cx('indicator')", async () => {
    const spec = scenario({
      name: "prepareRename/static-cx-literal",
      workspace: TSX_WORKSPACE,
      actions: {
        prepareRename: ({ workspace: testWorkspace, target }) => {
          const cursor = cursorFixture({
            workspace: testWorkspace,
            filePath: TSX_PATH,
            documentUri: TSX_URI,
            markerName: target.name,
            version: 1,
          });
          return handlePrepareRename(
            textDocumentPositionFromCursor(cursor),
            makeTsxDeps({}, testWorkspace.range("class", TSX_PATH).range),
            cursor,
          );
        },
      },
    });

    const result = await spec.prepareRename("cursor", TSX_PATH);
    expect(result).not.toBeNull();
    expect(result!.placeholder).toBe("indicator");
    expect(result!.range).toEqual(TSX_CLASS_RANGE);
  });

  it("throws a message when the cursor is on a dynamic class expression", () => {
    const dynamicWorkspace = workspace({
      [TSX_PATH]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const a = cx(/*<expr>*/si/*|*/ze/*</expr>*/);
`,
    });
    const expressionRange = dynamicWorkspace.range("expr", TSX_PATH).range;
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: () => ({
        stylesBindings: new Map([
          ["styles", { kind: "resolved" as const, absolutePath: BINDING.scssModulePath }],
        ]),
        bindings: [
          {
            ...BINDING,
            bindingRange: TSX_BINDING_RANGE,
          },
        ],
      }),
      parseClassExpressions: (_sf, bindings) =>
        buildTestClassExpressions({
          filePath: "/fake/src/App.tsx",
          bindings,
          expressions: [
            {
              kind: "symbolRef",
              origin: "cxCall",
              rawReference: "size",
              range: expressionRange,
              scssModulePath: bindings[0]!.scssModulePath,
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
    const deps = makeBaseDeps({
      analysisCache,
      selectorMapForPath: () => new Map([["indicator", info("indicator")]]),
      workspaceRoot: "/fake",
    });
    const cursorParams = sourceCursorParams(dynamicWorkspace);

    expect(() =>
      handlePrepareRename(textDocumentPositionFromCursor(cursorParams), deps, cursorParams),
    ).toThrow("Dynamic class expressions cannot be renamed safely.");
  });
});

describe("handleRename from TS/TSX", () => {
  it("builds WorkspaceEdit when renaming from cx('indicator') in TSX", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "indicator",
        line: TSX_CLASS_RANGE.start.line,
        start: TSX_CLASS_RANGE.start.character,
        end: TSX_CLASS_RANGE.end.character,
      }),
    ]);
    semanticReferenceIndex.record("file:///fake/src/Other.tsx", [
      semanticSite({
        uri: "file:///fake/src/Other.tsx",
        canonicalName: "indicator",
      }),
    ]);
    const cursorParams = sourceCursorParams(TSX_WORKSPACE);
    const result = handleRename(
      textDocumentRenameFromCursor(cursorParams, "status"),
      makeTsxDeps({ semanticReferenceIndex }),
      cursorParams,
    );
    expect(result).not.toBeNull();
    const changes = result!.changes!;

    // SCSS file edit
    const scssUri = "file:///fake/src/Button.module.scss";
    expect(changes[scssUri]).toHaveLength(1);
    expect(changes[scssUri]![0]!.newText).toBe("status");

    // TS/TSX edits
    expect(changes["file:///fake/src/App.tsx"]).toHaveLength(1);
    expect(changes["file:///fake/src/Other.tsx"]).toHaveLength(1);
  });
});

// Rename must not corrupt template/variable reverse-index sites.
// When a rename rewrites a class whose reverse index also contains
// "expanded" template/variable entries at the same range, those
// synthesized entries must be skipped — rewriting them would
// destroy the dynamic expression source.
describe("rename template corruption guard", () => {
  // Shared fixture: `cx(`btn-${weight}`)` at range R in App.tsx, where
  // `btn-` resolves against SCSS class map containing `btn-small` and
  // `btn-large`. The reverse index holds:
  //   - one "direct" template site at R (kind: "template")
  //   - two "expanded" static sites at R (one per class name)
  // This is exactly the shape `collectCallSites` emits with a
  // CallSiteResolverContext available.
  const TEMPLATE_URI = "file:///fake/src/App.tsx";
  const TEMPLATE_WORKSPACE = workspace({
    [TSX_PATH]: "const a = cx(/*<template>*/`btn-${weight}`/*</template>*/);\n",
  });
  const TEMPLATE_RANGE = TEMPLATE_WORKSPACE.range("template", TSX_PATH).range;
  const BTN_STYLE_WORKSPACE = styleWorkspace(
    SCSS_PATH,
    `
./*|*/btn-small {}

.btn-large {}
`,
  );

  function buildTemplateSemanticIndex(): WorkspaceSemanticWorkspaceReferenceIndex {
    const idx = new WorkspaceSemanticWorkspaceReferenceIndex();
    idx.record(TEMPLATE_URI, [
      semanticSite({
        uri: TEMPLATE_URI,
        canonicalName: "btn-small",
        className: "btn-small",
        line: TEMPLATE_RANGE.start.line,
        start: TEMPLATE_RANGE.start.character,
        end: TEMPLATE_RANGE.end.character,
        certainty: "inferred",
        reason: "templatePrefix",
      }),
      semanticSite({
        uri: TEMPLATE_URI,
        canonicalName: "btn-large",
        className: "btn-large",
        line: TEMPLATE_RANGE.start.line,
        start: TEMPLATE_RANGE.start.character,
        end: TEMPLATE_RANGE.end.character,
        certainty: "inferred",
        reason: "templatePrefix",
      }),
    ]);
    return idx;
  }

  function btnScssDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
    return makeBaseDeps({
      selectorMapForPath: () =>
        new Map([
          ["btn-small", info("btn-small", 1)],
          ["btn-large", info("btn-large", 3)],
        ]),
      workspaceRoot: "/fake",
      ...overrides,
    });
  }

  it("rename template-literal class is blocked when expanded references exist", () => {
    const semanticReferenceIndex = buildTemplateSemanticIndex();
    const result = handleRename(
      styleRenameParams(BTN_STYLE_WORKSPACE, SCSS_URI, "btn-tiny"),
      btnScssDeps({ semanticReferenceIndex }),
    );
    expect(result).toBeNull();
  });

  it("SCSS-side prepareRename rejects class with template/variable references", () => {
    const semanticReferenceIndex = buildTemplateSemanticIndex();
    expectPrepareRenameBlocked(
      () =>
        handlePrepareRename(
          stylePositionParams(BTN_STYLE_WORKSPACE, SCSS_URI),
          btnScssDeps({ semanticReferenceIndex }),
        ),
      "Rename is blocked because inferred or expanded references would make the edit unsafe.",
    );
  });

  it("SCSS-side prepareRename rejects class with semantic inferred references", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "btn-small",
        line: TEMPLATE_RANGE.start.line,
        start: TEMPLATE_RANGE.start.character,
        end: TEMPLATE_RANGE.end.character,
        certainty: "inferred",
        reason: "templatePrefix",
      }),
    ]);
    expectPrepareRenameBlocked(
      () =>
        handlePrepareRename(
          stylePositionParams(BTN_STYLE_WORKSPACE, SCSS_URI),
          btnScssDeps({ semanticReferenceIndex }),
        ),
      "Rename is blocked because inferred or expanded references would make the edit unsafe.",
    );
  });

  it("SCSS-side prepareRename rejects exact-but-expanded semantic references", () => {
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "btn-small",
        line: TEMPLATE_RANGE.start.line,
        start: TEMPLATE_RANGE.start.character,
        end: TEMPLATE_RANGE.end.character,
        certainty: "exact",
        reason: "templatePrefix",
        expansion: "expanded",
      }),
    ]);
    expectPrepareRenameBlocked(
      () =>
        handlePrepareRename(
          stylePositionParams(BTN_STYLE_WORKSPACE, SCSS_URI),
          btnScssDeps({ semanticReferenceIndex }),
        ),
      "Rename is blocked because inferred or expanded references would make the edit unsafe.",
    );
  });

  it("source-side prepareRename rejects classes with expanded semantic references", () => {
    const fixture = workspace({
      [TSX_PATH]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const a = cx('/*<class>*/btn-/*|*/small/*</class>*/');
`,
    });
    const classRange = fixture.range("class", TSX_PATH).range;
    const semanticReferenceIndex = buildTemplateSemanticIndex();
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: () => ({
        stylesBindings: new Map([
          ["styles", { kind: "resolved" as const, absolutePath: BINDING.scssModulePath }],
        ]),
        bindings: [BINDING],
      }),
      parseClassExpressions: (_sf, bindings) =>
        buildTestClassExpressions({
          filePath: "/fake/src/App.tsx",
          bindings,
          expressions: [
            {
              kind: "literal",
              origin: "cxCall",
              className: "btn-small",
              range: classRange,
              scssModulePath: bindings[0]!.scssModulePath,
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
    const cursorParams = sourceCursorParams(fixture);
    expectPrepareRenameBlocked(
      () =>
        handlePrepareRename(
          textDocumentPositionFromCursor(cursorParams),
          makeBaseDeps({
            analysisCache,
            semanticReferenceIndex,
            selectorMapForPath: () =>
              new Map([
                ["btn-small", info("btn-small", 1)],
                ["btn-large", info("btn-large", 3)],
              ]),
            workspaceRoot: "/fake",
          }),
          cursorParams,
        ),
      "Rename is blocked because inferred or expanded references would make the edit unsafe.",
    );
  });
});

describe("prepareRename through real parseStyleSelectorMap (regression)", () => {
  // These tests exercise prepareRenameFromScss against selector maps
  // built by the real `parseStyleSelectorMap`, to catch regressions where
  // a nested rule silently changes a flat parent's nested-safety state and
  // causes rename to be rejected on the flat parent.
  it("`.button { &:hover {} }` — rename is accepted on the flat .button", async () => {
    const fixture = styleWorkspace(
      SCSS_PATH,
      `./*|*/button {\n  color: red;\n  &:hover { color: blue; }\n}`,
    );
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
    });
    const result = handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps);
    expect(result).not.toBeNull();
    expect(result).toHaveProperty("placeholder", "button");
  });

  it("`.button { &--primary {} }` — both flat .button and nested &--primary are renameable", async () => {
    const fixture = styleWorkspace(
      SCSS_PATH,
      `./*at:flat*/button {\n  color: red;\n  &/*at:nested*/--primary { background: blue; }\n}`,
    );
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
    });
    // Cursor on `.button` at the flat rule — accepted.
    const flat = handlePrepareRename(stylePositionParams(fixture, SCSS_URI, "flat"), deps);
    expect(flat).not.toBeNull();
    expect(flat).toHaveProperty("placeholder", "button");

    // Cursor on the nested `&--primary` — accepted via BEM suffix
    // rename. Placeholder is the resolved class name
    // `"button--primary"`; range covers the `&--primary` slice
    // (10 chars) on its line.
    const nested = handlePrepareRename(stylePositionParams(fixture, SCSS_URI, "nested"), deps);
    expect(nested).not.toBeNull();
    expect(nested).toHaveProperty("placeholder", "button--primary");
    expect((nested as { range: Range }).range.end.character).toBe(
      targetFixture({ workspace: fixture, markerName: "nested" }).character + 9,
    );
  });
});

describe("&-nested BEM suffix rename", () => {
  // Positive cases (4): strict red→green for BEM-safe shapes.
  it("prepareRename on `&--primary` returns range covering only `&--primary` (10 chars)", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/--primary { color: white; }\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
    });
    const result = handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps);
    expect(result).not.toBeNull();
    expect(result).toHaveProperty("placeholder", "button--primary");
    const range = (result as { range: Range }).range;
    expect(range.end.character - range.start.character).toBe(10);
  });

  it("rename `button--primary → button--tiny`: SCSS edits only `--primary` slice, TSX full", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/--primary { color: white; }\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "button--primary",
        className: "button--primary",
      }),
    ]);
    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      semanticReferenceIndex,
    });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "button--tiny"), deps);
    expect(result).not.toBeNull();
    const edits = (
      result as {
        changes: Record<
          string,
          Array<{
            range: Range;
            newText: string;
          }>
        >;
      }
    ).changes;
    // SCSS: only the `--primary` slice (9 chars from column +1) becomes `--tiny`.
    const scssEdits = edits[SCSS_URI]!;
    expect(scssEdits).toHaveLength(1);
    expect(scssEdits[0]!.newText).toBe("--tiny");
    const target = targetFixture({ workspace: fixture });
    expect(scssEdits[0]!.range.start.character).toBe(target.character);
    expect(scssEdits[0]!.range.end.character).toBe(target.character + 9);
    // TSX: full `button--primary` → `button--tiny`.
    const tsxEdits = edits["file:///fake/src/App.tsx"]!;
    expect(tsxEdits).toHaveLength(1);
    expect(tsxEdits[0]!.newText).toBe("button--tiny");
  });

  it("rename `&__icon`: edits only the `__icon` slice", async () => {
    const cardPath = "/fake/src/Card.module.scss";
    const cardUri = "file:///fake/src/Card.module.scss";
    const fixture = styleWorkspace(cardPath, `.card {\n  &/*|*/__icon { width: 16px; }\n}`);
    const classMap = styleClassMap(fixture, cardPath);
    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
    });
    const result = handleRename(styleRenameParams(fixture, cardUri, "card__glyph"), deps);
    expect(result).not.toBeNull();
    const edits = (result as { changes: Record<string, Array<{ newText: string }>> }).changes;
    const scssEdits = edits[cardUri]!;
    expect(scssEdits[0]!.newText).toBe("__glyph");
  });

  it("double-nested `.card { &__icon { &--small {} } }` edits only innermost `--small`", async () => {
    const cardPath = "/fake/src/Card.module.scss";
    const cardUri = "file:///fake/src/Card.module.scss";
    const fixture = styleWorkspace(
      cardPath,
      `.card {\n  &__icon {\n    &/*|*/--small { font-size: 12px; }\n  }\n}`,
    );
    const classMap = styleClassMap(fixture, cardPath);
    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
    });
    const result = handleRename(styleRenameParams(fixture, cardUri, "card__icon--xs"), deps);
    expect(result).not.toBeNull();
    const edits = (result as { changes: Record<string, Array<{ newText: string }>> }).changes;
    const scssEdits = edits[cardUri]!;
    expect(scssEdits).toHaveLength(1);
    expect(scssEdits[0]!.newText).toBe("--xs");
  });

  // Negative cases (12): new guards in Commit 4.
  it("rejects cross-parent rename `button--primary → banner--tiny`", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/--primary {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "banner--tiny"), deps);
    expect(result).toBeNull();
  });

  it("rejects empty-suffix rename `button--primary → button`", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/--primary {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "button"), deps);
    expect(result).toBeNull();
  });

  it("rejects no-op rename `button--primary → button--primary`", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/--primary {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "button--primary"), deps);
    expect(result).toBeNull();
  });

  it("rejects interpolated rawToken (guard test — synthetic selector)", () => {
    // Parser cannot produce this shape, but the guard is load-bearing
    // if a future parser change weakens interpolation filtering.
    const fixture = styleWorkspace(
      SCSS_PATH,
      `.btn {
  /*<raw>*/&/*|*/--#{$mod}/*</raw>*/ {}
}
`,
    );
    const rawTokenRange = fixture.range("raw", SCSS_PATH).range;
    const resolvedName = "btn--primary";
    const resolvedRange: Range = {
      start: rawTokenRange.start,
      end: {
        line: rawTokenRange.start.line,
        character: rawTokenRange.start.character + resolvedName.length,
      },
    };
    const synthetic = {
      ...info(resolvedName, rawTokenRange.start.line),
      range: resolvedRange,
      bemSuffix: {
        rawTokenRange,
        rawToken: "&--#{$mod}",
        parentResolvedName: "btn",
      },
      nestedSafety: "bemSuffixSafe" as const,
      declarations: "",
      ruleRange: resolvedRange,
    };
    const deps = makeBaseDeps({
      selectorMapForPath: () => new Map([["btn--primary", synthetic]]),
      workspaceRoot: "/fake",
    });
    expectPrepareRenameBlocked(
      () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
      "Selectors containing interpolation cannot be renamed safely.",
    );
  });

  it("rejects non-bare parent `.card:hover { &--primary {} }`", async () => {
    const classMap = parseStyleSelectorMap(`.card:hover {\n  &--primary {}\n}`, "/f.module.scss");
    // `card--primary` never exists because extractClassNames strips
    // `:hover--primary` greedily. Any cursor on line 1 falls through.
    expect(classMap.has("card--primary")).toBe(false);
  });

  it("rejects grouped parent `.a, .b { &--c {} }`", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.a, .b {\n  &/*|*/--c {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const entry = classMap.get("a--c") ?? classMap.get("b--c");
    expect(entry).toBeDefined();
    // bemSuffix undefined because parentCtx.isGrouped === true
    expect(entry!.bemSuffix).toBeUndefined();
    // prepareRename must refuse
    const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
    expectPrepareRenameBlocked(
      () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
      "Only flat selectors and safe BEM suffix selectors can be renamed.",
    );
  });

  it("rejects grouped-nested child `.btn { &--a, &--b {} }`", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.btn {\n  &/*|*/--a, &--b {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const a = classMap.get("btn--a");
    expect(a).toBeDefined();
    expect(a!.bemSuffix).toBeUndefined();
    const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
    expectPrepareRenameBlocked(
      () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
      "Only flat selectors and safe BEM suffix selectors can be renamed.",
    );
  });

  it("rejects multi-`&` `.btn { & + &--x {} }`", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.btn {\n  & + &/*|*/--x {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const entry = classMap.get("btn--x");
    if (entry !== undefined) {
      expect(entry.bemSuffix).toBeUndefined();
      const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
      expectPrepareRenameBlocked(
        () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
        "Only flat selectors and safe BEM suffix selectors can be renamed.",
      );
    }
  });

  it("rejects compound `.button { &.active {} }` (active entry has no trio)", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/.active {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const active = classMap.get("active")!;
    expect(active.nestedSafety).toBe("nestedUnsafe");
    expect(active.bemSuffix).toBeUndefined();
    const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
    const result = handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps);
    expect(result).toBeNull();
  });

  it("rejects invalid newName (empty string)", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/--primary {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, ""), deps);
    expect(result).toBeNull();
  });

  it("rejects invalid newName (numeric start)", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/--primary {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const deps = makeBaseDeps({ selectorMapForPath: () => classMap, workspaceRoot: "/fake" });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "123xyz"), deps);
    expect(result).toBeNull();
  });

  it("regression: nested `&--primary` + template `cx(\\`button--${x}\\`)` still rejects via expanded-sites", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.button {\n  &/*|*/--primary {}\n}`);
    const classMap = styleClassMap(fixture, SCSS_PATH);
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "button--primary",
        className: "button--primary",
        certainty: "inferred",
        reason: "templatePrefix",
      }),
    ]);
    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      semanticReferenceIndex,
    });
    expectPrepareRenameBlocked(
      () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
      "Rename is blocked because inferred or expanded references would make the edit unsafe.",
    );
  });
});

/**
 * Alias-first iteration order. Production `expandSelectorMapWithTransform`
 * puts originals before aliases, which makes SCSS-cursor always hit
 * the original. To exercise the alias-selector code path from
 * the SCSS side, we flip the insertion order. The code under test
 * must be robust regardless of iteration order — and this helper
 * forces the alias branch.
 */
const camelOf = (name: string): string =>
  name
    .split(/[\s_-]+/)
    .filter(Boolean)
    .map((p, i) =>
      i === 0 ? p.charAt(0).toLowerCase() + p.slice(1) : p.charAt(0).toUpperCase() + p.slice(1),
    )
    .join("");

function aliasFirstCamelCaseMap(base: ReadonlyMap<string, ReturnType<typeof info>>) {
  const expanded = new Map<string, ReturnType<typeof info>>();
  for (const [name, entry] of base) {
    const alias = camelOf(name);
    if (alias !== name && !expanded.has(alias)) {
      expanded.set(alias, { ...entry, name: alias, originalName: name, canonicalName: name });
    }
  }
  for (const [name, entry] of base) {
    expanded.set(name, entry);
  }
  return expanded;
}

function withTransformMode(mode: Settings["scss"]["classnameTransform"]): Settings {
  return {
    ...DEFAULT_SETTINGS,
    scss: { ...DEFAULT_SETTINGS.scss, classnameTransform: mode },
  };
}

describe("classnameTransform alias-aware rename", () => {
  it("camelCase: alias cursor rewrites SCSS via original entry's range", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = aliasFirstCamelCaseMap(base);
    // sanity: alias iterates first
    const firstKey = classMap.keys().next().value;
    expect(firstKey).toBe("btnPrimary");
    const original = base.get("btn-primary")!;

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      settings: withTransformMode("camelCase"),
    });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "btn-hero"), deps);
    expect(result).not.toBeNull();
    const scssEdits = result!.changes![SCSS_URI]!;
    expect(scssEdits).toHaveLength(1);
    expect(scssEdits[0]!.newText).toBe("btn-hero");
    // Edit range equals the ORIGINAL entry's range, even though the
    // cursor resolved to the alias via alias-first iteration.
    expect(scssEdits[0]!.range).toEqual({
      start: { line: original.range.start.line, character: original.range.start.character },
      end: { line: original.range.end.line, character: original.range.end.character },
    });
  });

  it("camelCase: alias cursor on deep-nested BEM edits only the suffix slice", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `.btn-primary {\n  &/*|*/--xl {}\n}`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = aliasFirstCamelCaseMap(base);
    // sanity: alias entries exist
    expect(classMap.has("btnPrimaryXl")).toBe(true);
    expect(classMap.get("btnPrimaryXl")!.originalName).toBe("btn-primary--xl");
    const target = targetFixture({ workspace: fixture });

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      settings: withTransformMode("camelCase"),
    });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "btn-primary--huge"), deps);
    expect(result).not.toBeNull();
    const scssEdits = result!.changes![SCSS_URI]!;
    expect(scssEdits).toHaveLength(1);
    // Slice edit: `--xl` at offset+1 replaced by `--huge`. Parent
    // stays untouched; the edit length is the original suffix length.
    expect(scssEdits[0]!.newText).toBe("--huge");
    expect(scssEdits[0]!.range.start).toEqual({
      line: target.line,
      character: target.character,
    });
    expect(scssEdits[0]!.range.end).toEqual({
      line: target.line,
      character: target.character + "--xl".length,
    });
  });

  it("camelCase: original-name cursor (non-alias path) edits same range", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    // Production expansion (original-first) — findSelectorAtCursor
    // returns the non-alias entry.
    const classMap = expandSelectorMapWithTransform(base, "camelCase");
    const original = base.get("btn-primary")!;

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      settings: withTransformMode("camelCase"),
    });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "btn-hero"), deps);
    expect(result).not.toBeNull();
    const scssEdits = result!.changes![SCSS_URI]!;
    expect(scssEdits).toHaveLength(1);
    expect(scssEdits[0]!.newText).toBe("btn-hero");
    expect(scssEdits[0]!.range).toEqual({
      start: { line: original.range.start.line, character: original.range.start.character },
      end: { line: original.range.end.line, character: original.range.end.character },
    });
  });

  it("camelCaseOnly: alias rename is rejected at prepareRename", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    // camelCaseOnly drops the original; only `btnPrimary` alias remains.
    const classMap = expandSelectorMapWithTransform(base, "camelCaseOnly");
    expect(classMap.has("btn-primary")).toBe(false);
    expect(classMap.get("btnPrimary")!.originalName).toBe("btn-primary");

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      settings: withTransformMode("camelCaseOnly"),
    });
    expectPrepareRenameBlocked(
      () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
      "Alias selector views cannot be renamed under the current classnameTransform mode.",
    );
  });

  it("dashesOnly: alias rename is rejected at prepareRename", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "dashesOnly");
    expect(classMap.get("btnPrimary")!.originalName).toBe("btn-primary");

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      settings: withTransformMode("dashesOnly"),
    });
    expectPrepareRenameBlocked(
      () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
      "Alias selector views cannot be renamed under the current classnameTransform mode.",
    );
  });

  it("camelCase: canonical-form and alias-form sites both rewrite with per-site format", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCase");

    // Two real-world access patterns against the same SCSS class:
    //   - `cx('btn-primary')` in App.tsx — canonical form
    //   - `styles.btnPrimary`  in Other.tsx — alias form
    // Both access forms are keyed under the canonical selector.
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "btn-primary",
        className: "btn-primary",
      }),
    ]);
    semanticReferenceIndex.record("file:///fake/src/Other.tsx", [
      semanticSite({
        uri: "file:///fake/src/Other.tsx",
        canonicalName: "btn-primary",
        className: "btnPrimary",
        reason: "styleAccess",
        origin: "styleAccess",
      }),
    ]);

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      semanticReferenceIndex,
      settings: withTransformMode("camelCase"),
    });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "btn-hero"), deps);
    expect(result).not.toBeNull();
    const changes = result!.changes!;
    // Canonical-form site writes the raw dashed name.
    expect(changes["file:///fake/src/App.tsx"]).toHaveLength(1);
    expect(changes["file:///fake/src/App.tsx"]![0]!.newText).toBe("btn-hero");
    // Alias-form site writes the camelCase form of the new name so
    // `styles.btnPrimary` becomes `styles.btnHero`, not the invalid
    // `styles.btn-hero`.
    expect(changes["file:///fake/src/Other.tsx"]).toHaveLength(1);
    expect(changes["file:///fake/src/Other.tsx"]![0]!.newText).toBe("btnHero");
  });

  it("camelCase: semantic direct sites rewrite with per-site format", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCase");

    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "btn-primary",
        className: "btn-primary",
      }),
    ]);
    semanticReferenceIndex.record("file:///fake/src/Other.tsx", [
      semanticSite({
        uri: "file:///fake/src/Other.tsx",
        canonicalName: "btn-primary",
        className: "btnPrimary",
        reason: "styleAccess",
        origin: "styleAccess",
      }),
    ]);

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      semanticReferenceIndex,
      settings: withTransformMode("camelCase"),
    });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "btn-hero"), deps);
    expect(result).not.toBeNull();
    const changes = result!.changes!;
    expect(changes["file:///fake/src/App.tsx"]).toHaveLength(1);
    expect(changes["file:///fake/src/App.tsx"]![0]!.newText).toBe("btn-hero");
    expect(changes["file:///fake/src/Other.tsx"]).toHaveLength(1);
    expect(changes["file:///fake/src/Other.tsx"]![0]!.newText).toBe("btnHero");
  });

  it("camelCase: SCSS cursor on original still finds alias-form TSX sites via canonical key", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCase");

    // Only an alias-form TSX site exists — no direct `cx('btn-primary')`.
    // SCSS cursor rename on the original must still rewrite the
    // alias access because the semantic index keys it under the
    // canonical selector.
    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "btn-primary",
        className: "btnPrimary",
        reason: "styleAccess",
        origin: "styleAccess",
      }),
    ]);

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      semanticReferenceIndex,
      settings: withTransformMode("camelCase"),
    });
    const result = handleRename(styleRenameParams(fixture, SCSS_URI, "btn-hero"), deps);
    expect(result).not.toBeNull();
    const tsxEdits = result!.changes!["file:///fake/src/App.tsx"]!;
    expect(tsxEdits).toHaveLength(1);
    expect(tsxEdits[0]!.newText).toBe("btnHero");
  });

  it("camelCase: SCSS cursor on original-first expansion returns the non-alias entry", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = expandSelectorMapWithTransform(base, "camelCase");
    // Both keys present.
    expect(classMap.has("btn-primary")).toBe(true);
    expect(classMap.has("btnPrimary")).toBe(true);
    const target = targetFixture({ workspace: fixture });

    const hit = findSelectorAtCursor(
      buildStyleDocumentFromSelectorMap(SCSS_PATH, classMap),
      target.line,
      target.character,
    );
    expect(hit).not.toBeNull();
    // Locks in that cursoring through the SCSS source still lands
    // on the original entry, not the alias copy — regression guard
    // against a future iteration-order change.
    expect(hit!.name).toBe("btn-primary");
    expect(hit!.originalName).toBeUndefined();
  });

  it("regression: expanded site on original key rejects rename via alias cursor", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = aliasFirstCamelCaseMap(base);

    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "btn-primary",
        className: "btn-primary",
        certainty: "inferred",
        reason: "templatePrefix",
      }),
    ]);

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      semanticReferenceIndex,
      settings: withTransformMode("camelCase"),
    });
    // Cursor hits the alias first (alias-first iteration). Without
    // the union, the single-key check against `btnPrimary` would
    // miss the expanded site keyed on `btn-primary` and allow the
    // rename — rewriting the template and destroying the source.
    expectPrepareRenameBlocked(
      () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
      "Rename is blocked because inferred or expanded references would make the edit unsafe.",
    );
  });

  it("regression: semantic expanded site on original key rejects rename via alias cursor", async () => {
    const fixture = styleWorkspace(SCSS_PATH, `./*|*/btn-primary { color: red; }`);
    const base = styleClassMap(fixture, SCSS_PATH);
    const classMap = aliasFirstCamelCaseMap(base);

    const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
    semanticReferenceIndex.record("file:///fake/src/App.tsx", [
      semanticSite({
        uri: "file:///fake/src/App.tsx",
        canonicalName: "btn-primary",
        className: "btn-primary",
        certainty: "inferred",
        reason: "templatePrefix",
      }),
    ]);

    const deps = makeBaseDeps({
      selectorMapForPath: () => classMap,
      workspaceRoot: "/fake",
      semanticReferenceIndex,
      settings: withTransformMode("camelCase"),
    });
    expectPrepareRenameBlocked(
      () => handlePrepareRename(stylePositionParams(fixture, SCSS_URI), deps),
      "Rename is blocked because inferred or expanded references would make the edit unsafe.",
    );
  });
});

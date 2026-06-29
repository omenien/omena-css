import { describe, expect, it, vi } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import {
  withSourceExpressionAtCursor,
  type ProviderDeps,
} from "../../../server/lsp-server/src/providers/cursor-dispatch";
import {
  cursorFixture,
  workspace,
  type CmeWorkspace,
  type Range,
} from "../../../packages/vitest-cme/src";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";

const SOURCE_PATH = "/fake/a.tsx";
const SOURCE_URI = "file:///fake/a.tsx";
const SCSS_PATH = "/fake/src/Button.module.scss";

const SOURCE_WORKSPACE = workspace({
  [SOURCE_PATH]: `/*at:outside*/import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const /*<binding>*/cx/*</binding>*/ = classNames.bind(styles);
const el = cx('/*<literal>*/ind/*|*/icator/*</literal>*/');
const el2 = <div className={styles./*<active>*/ac/*at:styleCursor*/tive/*</active>*/} />;
`,
});

const TSX = SOURCE_WORKSPACE.file(SOURCE_PATH).content;
const CX_BINDING_RANGE = SOURCE_WORKSPACE.range("binding", SOURCE_PATH).range;
const LITERAL_RANGE = SOURCE_WORKSPACE.range("literal", SOURCE_PATH).range;
const ACTIVE_RANGE = SOURCE_WORKSPACE.range("active", SOURCE_PATH).range;

const detectCxBindings = (sourceFile: ts.SourceFile): CxBinding[] =>
  sourceFile.text.includes("classnames/bind") && sourceFile.text.includes(".module.")
    ? [
        {
          cxVarName: "cx",
          stylesVarName: "styles",
          scssModulePath: SCSS_PATH,
          classNamesImportName: "classNames",
          bindingRange: CX_BINDING_RANGE,
        },
      ]
    : [];

function sourceCursor(fixture: CmeWorkspace = SOURCE_WORKSPACE, markerName = "cursor") {
  return cursorFixture({
    workspace: fixture,
    filePath: SOURCE_PATH,
    documentUri: SOURCE_URI,
    markerName,
    version: 1,
  });
}

function makeDeps(
  overrides: Partial<ProviderDeps> = {},
  literalRange: Range = LITERAL_RANGE,
  activeRange: Range = ACTIVE_RANGE,
): ProviderDeps {
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf) => ({
      stylesBindings: new Map([["styles", { kind: "resolved" as const, absolutePath: SCSS_PATH }]]),
      bindings: detectCxBindings(sf),
    }),
    parseClassExpressions: (_sf, bindings, stylesBindings) =>
      buildTestClassExpressions({
        filePath: "/fake/a.tsx",
        bindings,
        stylesBindings,
        expressions: [
          {
            kind: "literal",
            origin: "cxCall",
            className: "indicator",
            range: literalRange,
            scssModulePath: SCSS_PATH,
          },
          {
            kind: "styleAccess",
            className: "active",
            range: activeRange,
            scssModulePath: SCSS_PATH,
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
    selectorMapForPath: () =>
      new Map([
        ["indicator", info("indicator")],
        ["active", info("active")],
      ]),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

describe("withSourceExpressionAtCursor / entry gating", () => {
  it("returns null when the analyzed entry has no class expressions", () => {
    const emptyWorkspace = workspace({
      [SOURCE_PATH]: "/*|*/const x = 1;",
    });
    const cursor = sourceCursor(emptyWorkspace);
    const deps = makeDeps();
    const transform = vi.fn();
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: cursor.content,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      transform,
    );
    expect(result).toBeNull();
    expect(transform).not.toHaveBeenCalled();
  });

  it("returns null when class expressions are empty for an otherwise-matching file", () => {
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: () => ({ stylesBindings: new Map(), bindings: [] }),
      parseClassExpressions: () => [],
    });
    const analysisCache = new DocumentAnalysisCache({
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    const deps = makeBaseDeps({
      analysisCache,
      selectorMapForPath: () => null,
      workspaceRoot: "/fake",
    });
    const transform = vi.fn();
    const cursor = sourceCursor();
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: TSX,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      transform,
    );
    expect(result).toBeNull();
    expect(transform).not.toHaveBeenCalled();
  });

  it("returns null when the cursor is outside every class expression range", () => {
    const deps = makeDeps();
    const transform = vi.fn();
    const cursor = sourceCursor(SOURCE_WORKSPACE, "outside");
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: TSX,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      transform,
    );
    expect(result).toBeNull();
    expect(transform).not.toHaveBeenCalled();
  });

  it("returns null when the style document cannot be resolved for the expression path", () => {
    const deps = makeDeps({ selectorMapForPath: () => null });
    const transform = vi.fn();
    const cursor = sourceCursor();
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: TSX,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      transform,
    );
    expect(result).toBeNull();
    expect(transform).not.toHaveBeenCalled();
  });
});

describe("withSourceExpressionAtCursor / dispatch", () => {
  it("passes a literal cx expression when cursor lies inside it", () => {
    const deps = makeDeps();
    const spy = vi.fn((ctx) => ({ kind: ctx.expression.kind, origin: ctx.expression.origin }));
    const cursor = sourceCursor();
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: TSX,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      spy,
    );
    expect(spy).toHaveBeenCalledTimes(1);
    expect(result).toEqual({ kind: "literal", origin: "cxCall" });
  });

  it("passes a styleAccess expression when cursor is on a styles.x access", () => {
    const deps = makeDeps();
    const spy = vi.fn((ctx) => ({ kind: ctx.expression.kind, origin: ctx.expression.origin }));
    const cursor = sourceCursor(SOURCE_WORKSPACE, "styleCursor");
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: TSX,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      spy,
    );
    expect(spy).toHaveBeenCalledTimes(1);
    expect(result).toEqual({ kind: "styleAccess", origin: "styleAccess" });
  });

  it("allows the provider transform to resolve asynchronously", async () => {
    const deps = makeDeps();
    const spy = vi.fn(async (ctx) => ({ kind: ctx.expression.kind }));
    const cursor = sourceCursor();
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: TSX,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      spy,
    );
    await expect(result).resolves.toEqual({ kind: "literal" });
    expect(spy).toHaveBeenCalledTimes(1);
  });

  it("passes the AnalysisEntry so providers can skip a second cache lookup", () => {
    const deps = makeDeps();
    const spy = vi.fn((ctx) => ctx.entry);
    const cursor = sourceCursor();
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: TSX,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      spy,
    );
    expect(result).toBeTruthy();
    expect(
      (result as { sourceDocument: { classExpressions: readonly unknown[] } }).sourceDocument
        .classExpressions,
    ).toHaveLength(2);
  });

  it("chooses the most specific expression when ranges overlap", () => {
    const overlapWorkspace = workspace({
      [SOURCE_PATH]: `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const /*<binding>*/cx/*</binding>*/ = classNames.bind(styles);
const el = cx(/*<outer>*/'/*<inner>*/ind/*|*/icator/*</inner>*/'/*</outer>*/);
`,
    });
    const outerRange = overlapWorkspace.range("outer", SOURCE_PATH).range;
    const innerRange = overlapWorkspace.range("inner", SOURCE_PATH).range;
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: (sf) => ({
        stylesBindings: new Map([
          ["styles", { kind: "resolved" as const, absolutePath: SCSS_PATH }],
        ]),
        bindings: detectCxBindings(sf),
      }),
      parseClassExpressions: (_sf, bindings, stylesBindings) =>
        buildTestClassExpressions({
          filePath: SOURCE_PATH,
          bindings,
          stylesBindings,
          expressions: [
            {
              kind: "symbolRef",
              origin: "cxCall",
              rawReference: "size",
              range: outerRange,
              scssModulePath: SCSS_PATH,
            },
            {
              kind: "literal",
              origin: "cxCall",
              className: "indicator",
              range: innerRange,
              scssModulePath: SCSS_PATH,
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
    const spy = vi.fn((ctx) => ({ kind: ctx.expression.kind }));
    const cursor = sourceCursor(overlapWorkspace);
    const result = withSourceExpressionAtCursor(
      {
        documentUri: SOURCE_URI,
        content: overlapWorkspace.file(SOURCE_PATH).content,
        filePath: SOURCE_PATH,
        line: cursor.line,
        character: cursor.character,
        version: 1,
      },
      deps,
      spy,
    );
    expect(spy).toHaveBeenCalledTimes(1);
    expect(result).toEqual({ kind: "literal" });
  });
});

import { describe, expect, it, vi } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../../../server/engine-host-node/src/selected-query-backend";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { handleHover } from "../../../server/lsp-server/src/providers/hover";
import {
  cursorFixture,
  scenario,
  workspace,
  type CmeWorkspace,
  type Range,
} from "../../../packages/vitest-cme/src";
import { FakeTypeResolver } from "../../_fixtures/fake-type-resolver";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";
import { makeDesignTokenDefinitionGraph } from "../../_fixtures/style-semantic-graph";

const SOURCE_PATH = "/fake/ws/src/Button.tsx";
const SOURCE_URI = "file:///fake/ws/src/Button.tsx";
const STYLE_PATH = "/fake/ws/src/Button.module.scss";

const STATIC_HOVER_WORKSPACE = workspace({
  [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const /*<binding>*/cx/*</binding>*/ = classNames.bind(styles);
const el = cx('/*<class>*/ind/*|*/icator/*</class>*/');
`,
});
const STATIC_BINDING_RANGE = STATIC_HOVER_WORKSPACE.range("binding", SOURCE_PATH).range;
const STATIC_CLASS_RANGE = STATIC_HOVER_WORKSPACE.range("class", SOURCE_PATH).range;

const detectCxBindings = (_sourceFile: ts.SourceFile): CxBinding[] => [
  {
    cxVarName: "cx",
    stylesVarName: "styles",
    scssModulePath: STYLE_PATH,
    classNamesImportName: "classNames",
    bindingRange: STATIC_BINDING_RANGE,
  },
];

function makeDeps(
  overrides: Partial<ProviderDeps> = {},
  expressionRange: Range = STATIC_CLASS_RANGE,
): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf) }),
    parseClassExpressions: (_sf, bindings) =>
      buildTestClassExpressions({
        filePath: SOURCE_PATH,
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
    sourceFileCache,
    sourceFrontendAnalysis,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () => new Map([["indicator", info("indicator")]]),
    ...overrides,
  });
}

function hoverCursor(fixture: CmeWorkspace = STATIC_HOVER_WORKSPACE, markerName = "cursor") {
  return cursorFixture({
    workspace: fixture,
    filePath: SOURCE_PATH,
    documentUri: SOURCE_URI,
    markerName,
    version: 1,
  });
}

describe("handleHover", () => {
  const baseParams = hoverCursor();

  it("returns a Hover with markdown for a static call", async () => {
    const spec = scenario({
      name: "hover/static-cx-literal",
      workspace: STATIC_HOVER_WORKSPACE,
      actions: {
        hover: ({ workspace: testWorkspace, target }) => {
          const cursor = hoverCursor(testWorkspace, target.name);
          return handleHover(cursor, makeDeps({}, testWorkspace.range("class", SOURCE_PATH).range));
        },
      },
    });

    const hover = await spec.hover("cursor", SOURCE_PATH);
    expect(hover).not.toBeNull();
    expect(hover!.contents).toHaveProperty("kind", "markdown");
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("`.indicator`");
    expect(value).toContain("Button.module.scss");
    expect(hover!.range).toEqual(STATIC_CLASS_RANGE);
  });

  it("resolves hover on a continuation line of a multi-line cx() call", () => {
    const multiLineWorkspace = workspace({
      [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx(
  '/*<class>*/ind/*|*/icator/*</class>*/',
  { active: true },
);
`,
    });
    const expressionRange = multiLineWorkspace.range("class", SOURCE_PATH).range;
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: (sf) => ({
        stylesBindings: new Map(),
        bindings: detectCxBindings(sf),
      }),
      parseClassExpressions: (_sf, bindings) =>
        buildTestClassExpressions({
          filePath: "/fake/ws/src/Button.tsx",
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
    const deps = makeDeps({
      analysisCache: new DocumentAnalysisCache({
        sourceFileCache: new SourceFileCache({ max: 10 }),
        sourceFrontendAnalysis,
        fileExists: () => true,
        aliasResolver: EMPTY_ALIAS_RESOLVER,
        max: 10,
      }),
    });
    // Cursor on line 5 (the 'indicator' line) — no "(" on this line
    const hover = handleHover(hoverCursor(multiLineWorkspace), deps);
    expect(hover).not.toBeNull();
    expect((hover!.contents as { value: string }).value).toContain("`.indicator`");
  });

  it("returns null when the classMap has no match", () => {
    const hover = handleHover(baseParams, makeDeps({ selectorMapForPath: () => new Map() }));
    expect(hover).toBeNull();
  });

  it("includes dynamic hover explanation for symbol refs resolved from type unions", () => {
    const unionWorkspace = workspace({
      [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const size = choose();
const el = cx(/*<expr>*/si/*|*/ze/*</expr>*/);
`,
    });
    const expressionRange = unionWorkspace.range("expr", SOURCE_PATH).range;
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: (sf) => ({
        stylesBindings: new Map(),
        bindings: detectCxBindings(sf),
      }),
      parseClassExpressions: (_sf, bindings) =>
        buildTestClassExpressions({
          filePath: "/fake/ws/src/Button.tsx",
          bindings,
          expressions:
            bindings.length === 0
              ? []
              : [
                  {
                    kind: "symbolRef",
                    origin: "cxCall",
                    rawReference: "size",
                    rootName: "size",
                    pathSegments: [],
                    range: expressionRange,
                    scssModulePath: bindings[0]!.scssModulePath,
                  },
                ],
        }),
    });
    const deps = makeDeps({
      analysisCache: new DocumentAnalysisCache({
        sourceFileCache: new SourceFileCache({ max: 10 }),
        sourceFrontendAnalysis,
        fileExists: () => true,
        aliasResolver: EMPTY_ALIAS_RESOLVER,
        max: 10,
      }),
      selectorMapForPath: () =>
        new Map([
          ["indicator", info("indicator")],
          ["active", info("active")],
        ]),
      typeResolver: new FakeTypeResolver(["indicator", "active"]),
    });

    const hover = handleHover(hoverCursor(unionWorkspace), deps);

    expect(hover).not.toBeNull();
    const value = (hover!.contents as { value: string }).value;
    expect(value).toContain("Resolved from `size` via TypeScript string-literal union analysis.");
    expect(value).toContain("Value certainty: inferred.");
    expect(value).toContain("Value certainty shape: bounded finite (2).");
    expect(value).toContain(
      "Value certainty reason: TypeScript exposed multiple string-literal candidates.",
    );
    expect(value).toContain("Selector certainty: exact.");
    expect(value).toContain("Value domain: finite set (2).");
    expect(value).toContain("Candidates: `active`, `indicator`.");
  });

  it("logs and returns null when the underlying transform raises", () => {
    const logError = vi.fn();
    const hover = handleHover(
      baseParams,
      makeDeps({
        styleDocumentForPath: () => {
          throw new Error("boom");
        },
        logError,
      }),
    );
    expect(hover).toBeNull();
    expect(logError).toHaveBeenCalledTimes(1);
    expect(logError).toHaveBeenCalledWith("hover handler failed", expect.any(Error));
  });

  it("uses the async Rust design-token ranking path for style hovers", async () => {
    const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
    process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
    try {
      const styleWorkspace = workspace({
        [STYLE_PATH]: `.button {
  --brand: green;
  color: var(--br/*|*/and);
}
`,
      });
      const styleDocument = parseStyleDocument(styleWorkspace.file(STYLE_PATH).content, STYLE_PATH);
      const winnerRange = {
        start: { line: 1, character: 2 },
        end: { line: 1, character: 9 },
      };
      const runner: RustSelectedQueryBackendJsonRunnerAsync = async <T>() =>
        makeDesignTokenDefinitionGraph({
          referenceName: "--brand",
          winnerDeclarationFilePath: STYLE_PATH,
          winnerDeclarationRange: winnerRange,
        }) as T;
      const deps = makeBaseDeps({
        styleDocumentForPath: (filePath) => (filePath === STYLE_PATH ? styleDocument : null),
        readStyleFile: (filePath) =>
          filePath === STYLE_PATH ? styleWorkspace.file(STYLE_PATH).content : null,
        runRustSelectedQueryBackendJsonAsync: runner,
      } as Partial<ProviderDeps> & {
        readonly runRustSelectedQueryBackendJsonAsync: RustSelectedQueryBackendJsonRunnerAsync;
      });

      const hover = await handleHover(
        cursorFixture({
          workspace: styleWorkspace,
          filePath: STYLE_PATH,
          documentUri: "file:///fake/ws/src/Button.module.scss",
          markerName: "cursor",
          version: 1,
        }),
        deps,
      );

      expect(hover).not.toBeNull();
      const value = (hover!.contents as { value: string }).value;
      expect(value).toContain("`--brand`");
      expect(value).toContain("Cascade ranking");
      expect(value).toContain("Button.module.scss");
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
    }
  });
});

describe("handleHover / styles.x without classnames/bind (L8 fix)", () => {
  it("returns hover for styles.indicator in a clsx-only file", () => {
    const clsxWorkspace = workspace({
      [SOURCE_PATH]: `
import clsx from 'clsx';
import styles from './Button.module.scss';
const el = <div className={clsx(styles./*<class>*/ind/*|*/icator/*</class>*/)} />;
`,
    });
    const expressionRange = clsxWorkspace.range("class", SOURCE_PATH).range;
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const indicatorInfo = info("indicator");
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: () => ({
        stylesBindings: new Map([
          [
            "styles",
            { kind: "resolved" as const, absolutePath: "/fake/ws/src/Button.module.scss" },
          ],
        ]),
        bindings: [],
      }),
      parseClassExpressions: (_sf, _bindings, stylesBindings) =>
        buildTestClassExpressions({
          filePath: "/fake/ws/src/Button.tsx",
          stylesBindings,
          bindings: [],
          expressions: stylesBindings.has("styles")
            ? [
                {
                  kind: "styleAccess",
                  className: "indicator",
                  scssModulePath: "/fake/ws/src/Button.module.scss",
                  range: expressionRange,
                },
              ]
            : [],
        }),
    });
    const analysisCache = new DocumentAnalysisCache({
      sourceFileCache,
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    const deps = makeDeps({
      analysisCache,
      selectorMapForPath: (path: string) => {
        if (path === "/fake/ws/src/Button.module.scss") {
          return new Map([["indicator", indicatorInfo]]);
        }
        return null;
      },
    });

    const hover = handleHover(hoverCursor(clsxWorkspace), deps);

    expect(hover).not.toBeNull();
    expect((hover!.contents as { value: string }).value).toContain("`.indicator`");
  });

  it("returns hover for styles['btn-primary'] bracket access", async () => {
    const { parseClassExpressions } =
      await import("../../../server/engine-core-ts/src/core/cx/class-ref-parser");
    const { scanCxImports } =
      await import("../../../server/engine-core-ts/src/core/cx/binding-detector");
    const bracketWorkspace = workspace({
      [SOURCE_PATH]: `
import styles from './Button.module.scss';
const el = <div className={styles['btn-/*|*/primary']} />;
`,
    });
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const btnInfo = info("btn-primary");
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports,
      parseClassExpressions,
    });
    const analysisCache = new DocumentAnalysisCache({
      sourceFileCache,
      sourceFrontendAnalysis,
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      max: 10,
    });
    const deps = makeDeps({
      analysisCache,
      selectorMapForPath: (path: string) => {
        if (path === "/fake/ws/src/Button.module.scss") {
          return new Map([["btn-primary", btnInfo]]);
        }
        return null;
      },
    });

    const hover = handleHover(hoverCursor(bracketWorkspace), deps);

    expect(hover).not.toBeNull();
    expect((hover!.contents as { value: string }).value).toContain("`.btn-primary`");
  });
});

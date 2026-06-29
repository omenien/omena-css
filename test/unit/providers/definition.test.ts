import { describe, expect, it, vi } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { handleDefinition } from "../../../server/lsp-server/src/providers/definition";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../../../server/engine-host-node/src/selected-query-backend";
import {
  cursorFixture,
  scenario,
  workspace,
  type CmeWorkspace,
} from "../../../packages/vitest-cme/src";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";
import { makeDesignTokenDefinitionGraph } from "../../_fixtures/style-semantic-graph";

const SOURCE_PATH = "/fake/src/Button.tsx";
const SOURCE_URI = "file:///fake/src/Button.tsx";
const STYLE_PATH = "/fake/src/Button.module.scss";

const STATIC_DEFINITION_WORKSPACE = workspace({
  [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const /*<binding>*/cx/*</binding>*/ = classNames.bind(styles);
const el = cx('/*<class>*/ind/*|*/icator/*</class>*/');
`,
});
const STATIC_BINDING_RANGE = STATIC_DEFINITION_WORKSPACE.range("binding", SOURCE_PATH).range;
const STATIC_CLASS_RANGE = STATIC_DEFINITION_WORKSPACE.range("class", SOURCE_PATH).range;
const INDICATOR_INFO = info("indicator");

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
    selectorMapForPath: () => new Map([["indicator", INDICATOR_INFO]]),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

function definitionCursor(
  fixture: CmeWorkspace = STATIC_DEFINITION_WORKSPACE,
  markerName = "cursor",
) {
  return cursorFixture({
    workspace: fixture,
    filePath: SOURCE_PATH,
    documentUri: SOURCE_URI,
    markerName,
    version: 1,
  });
}

describe("handleDefinition", () => {
  const baseParams = definitionCursor();

  it("returns a LocationLink pointing at the SCSS rule for a static call", async () => {
    const spec = scenario({
      name: "definition/static-cx-literal",
      workspace: STATIC_DEFINITION_WORKSPACE,
      actions: {
        definition: ({ workspace: testWorkspace, target }) => {
          const cursor = definitionCursor(testWorkspace, target.name);
          return handleDefinition(
            cursor,
            makeDeps({}, testWorkspace.range("class", SOURCE_PATH).range),
          );
        },
      },
    });

    const result = await spec.definition("cursor", SOURCE_PATH);
    expect(result).not.toBeNull();
    expect(result).toHaveLength(1);
    const link = result![0]!;
    expect(link.targetUri).toMatch(/Button\.module\.scss$/);
    expect(link.targetUri.startsWith("file://")).toBe(true);
    expect(link.originSelectionRange).toEqual(STATIC_CLASS_RANGE);
    expect(link.targetRange).toEqual(INDICATOR_INFO.ruleRange);
    expect(link.targetSelectionRange).toEqual(INDICATOR_INFO.range);
  });

  it("returns null when the cursor is not on a cx call", () => {
    const notOnCallWorkspace = workspace({
      [SOURCE_PATH]: `
/*|*/import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx('/*<class>*/indicator/*</class>*/');
`,
    });
    const deps = makeDeps();
    const result = handleDefinition(definitionCursor(notOnCallWorkspace), deps);
    expect(result).toBeNull();
  });

  it("returns null when classMap has no match for the class name", () => {
    const deps = makeDeps({
      selectorMapForPath: () => new Map(),
    });
    const result = handleDefinition(baseParams, deps);
    expect(result).toBeNull();
  });

  it("returns all LocationLinks for a template-literal prefix match", () => {
    const templateWorkspace = workspace({
      [SOURCE_PATH]: `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx(/*<class>*/\`btn-/*|*/\${variant}\`/*</class>*/);
`,
    });
    const expressionRange = templateWorkspace.range("class", SOURCE_PATH).range;
    const sourceFileCache = new SourceFileCache({ max: 10 });
    const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
      fileExists: () => true,
      aliasResolver: EMPTY_ALIAS_RESOLVER,
      scanCxImports: (sf) => ({
        stylesBindings: new Map(),
        bindings: detectCxBindings(sf),
      }),
      parseClassExpressions: (_sf, bindings) =>
        buildTestClassExpressions({
          filePath: "/fake/src/Button.tsx",
          bindings,
          expressions:
            bindings.length === 0
              ? []
              : [
                  {
                    kind: "template",
                    origin: "cxCall",
                    rawTemplate: "btn-${variant}",
                    staticPrefix: "btn-",
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
    const deps: ProviderDeps = makeBaseDeps({
      analysisCache,
      selectorMapForPath: () =>
        new Map([
          ["btn", info("btn")],
          ["btn-primary", info("btn-primary")],
          ["btn-secondary", info("btn-secondary")],
          ["indicator", info("indicator")],
        ]),
      workspaceRoot: "/fake",
    });
    const result = handleDefinition(definitionCursor(templateWorkspace), deps);
    expect(result).not.toBeNull();
    expect(result).toHaveLength(2);
    expect(result!.every((l) => l.targetUri.startsWith("file://"))).toBe(true);
  });

  it("uses the async Rust design-token winner path for style definitions", async () => {
    const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
    process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
    try {
      const winnerPath = "/fake/src/generated.tokens.css";
      const winnerRange = {
        start: { line: 3, character: 8 },
        end: { line: 3, character: 15 },
      };
      const styleWorkspace = workspace({
        [STYLE_PATH]: `.button {
  color: var(--br/*|*/and);
}
`,
      });
      const styleDocument = parseStyleDocument(styleWorkspace.file(STYLE_PATH).content, STYLE_PATH);
      const runner: RustSelectedQueryBackendJsonRunnerAsync = async <T>() =>
        makeDesignTokenDefinitionGraph({
          referenceName: "--brand",
          winnerDeclarationFilePath: winnerPath,
          winnerDeclarationRange: winnerRange,
        }) as T;
      const deps = makeBaseDeps({
        styleDocumentForPath: (filePath) => (filePath === STYLE_PATH ? styleDocument : null),
        readStyleFile: (filePath) =>
          filePath === STYLE_PATH ? styleWorkspace.file(STYLE_PATH).content : null,
        workspaceRoot: "/fake",
        runRustSelectedQueryBackendJsonAsync: runner,
      } as Partial<ProviderDeps> & {
        readonly runRustSelectedQueryBackendJsonAsync: RustSelectedQueryBackendJsonRunnerAsync;
      });

      const result = await handleDefinition(
        cursorFixture({
          workspace: styleWorkspace,
          filePath: STYLE_PATH,
          documentUri: "file:///fake/src/Button.module.scss",
          markerName: "cursor",
          version: 1,
        }),
        deps,
      );

      expect(result).toEqual([
        expect.objectContaining({
          targetUri: "file:///fake/src/generated.tokens.css",
          targetRange: winnerRange,
          targetSelectionRange: winnerRange,
        }),
      ]);
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
    }
  });

  it("logs and returns null when the underlying transform raises", () => {
    const logError = vi.fn();
    const deps = makeDeps({
      styleDocumentForPath: () => {
        throw new Error("boom");
      },
      logError,
    });
    expect(() => handleDefinition(baseParams, deps)).not.toThrow();
    expect(handleDefinition(baseParams, deps)).toBeNull();
    expect(logError).toHaveBeenCalledWith("definition handler failed", expect.any(Error));
  });
});

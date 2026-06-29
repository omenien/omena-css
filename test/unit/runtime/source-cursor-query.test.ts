import { describe, expect, it } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { resolveSourceExpressionContextAtCursor } from "../../../server/engine-host-node/src/source-cursor-query";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";

const SCSS_PATH = "/fake/src/Button.module.scss";

const TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx('indicator');
`;

const detectCxBindings = (_sourceFile: ts.SourceFile): CxBinding[] => [
  {
    cxVarName: "cx",
    stylesVarName: "styles",
    scssModulePath: SCSS_PATH,
    classNamesImportName: "classNames",
    bindingRange: {
      start: { line: 3, character: 6 },
      end: { line: 3, character: 8 },
    },
  },
];

describe("resolveSourceExpressionContextAtCursor", () => {
  it("returns the expression, style document, and analysis entry at the cursor", () => {
    const deps = makeDeps();
    const ctx = resolveSourceExpressionContextAtCursor(
      {
        documentUri: "file:///fake/src/Button.tsx",
        content: TSX,
        filePath: "/fake/src/Button.tsx",
        line: 4,
        character: 18,
        version: 1,
      },
      deps,
    );

    expect(ctx).not.toBeNull();
    expect(ctx?.expression).toMatchObject({
      kind: "literal",
      className: "indicator",
      scssModulePath: SCSS_PATH,
    });
    expect(ctx?.styleDocument.filePath).toBe(SCSS_PATH);
    expect(ctx?.entry.sourceDocument.classExpressions).toHaveLength(1);
  });

  it("returns null when the expression style document is unavailable", () => {
    const deps = makeDeps({ missingStyleDocument: true });
    const ctx = resolveSourceExpressionContextAtCursor(
      {
        documentUri: "file:///fake/src/Button.tsx",
        content: TSX,
        filePath: "/fake/src/Button.tsx",
        line: 4,
        character: 18,
        version: 1,
      },
      deps,
    );

    expect(ctx).toBeNull();
  });
});

function makeDeps(options: { readonly missingStyleDocument?: boolean } = {}) {
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sourceFile) => ({
      stylesBindings: new Map(),
      bindings: detectCxBindings(sourceFile),
    }),
    parseClassExpressions: (_sourceFile, bindings) =>
      buildTestClassExpressions({
        filePath: "/fake/src/Button.tsx",
        bindings,
        expressions: [
          {
            kind: "literal",
            origin: "cxCall",
            className: "indicator",
            range: {
              start: { line: 4, character: 15 },
              end: { line: 4, character: 24 },
            },
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
      options.missingStyleDocument ? null : new Map([["indicator", info("indicator")]]),
  });
}

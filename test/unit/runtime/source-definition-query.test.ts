import { describe, expect, it } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { readSourceExpressionContextAtCursor } from "../../../server/engine-core-ts/src/core/query";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import {
  resolveSourceExpressionDefinitionTargets,
  resolveSourceExpressionDefinitionTargetsAsync,
} from "../../../server/engine-host-node/src/source-definition-query";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";

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
    scssModulePath: "/fake/src/Button.module.scss",
    classNamesImportName: "classNames",
    bindingRange: {
      start: { line: 3, character: 6 },
      end: { line: 3, character: 8 },
    },
  },
];

function makeDepsForExpressions(
  expressions: Parameters<typeof buildTestClassExpressions>[0]["expressions"],
): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf, fp) }),
    parseClassExpressions: (_sf, bindings) =>
      buildTestClassExpressions({
        filePath: "/fake/src/Button.tsx",
        bindings,
        expressions,
      }),
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () =>
      new Map([
        ["indicator", info("indicator")],
        ["btn-primary", info("btn-primary")],
        ["btn-secondary", info("btn-secondary")],
      ]),
    workspaceRoot: "/fake",
  });
}

const baseParams = {
  documentUri: "file:///fake/src/Button.tsx",
  content: TSX,
  filePath: "/fake/src/Button.tsx",
  line: 4,
  character: 18,
  version: 1,
};

describe("resolveSourceExpressionDefinitionTargets", () => {
  it("returns a target for a static class expression", () => {
    const deps = makeDepsForExpressions([
      {
        kind: "literal",
        origin: "cxCall",
        className: "indicator",
        range: {
          start: { line: 4, character: 15 },
          end: { line: 4, character: 24 },
        },
        scssModulePath: "/fake/src/Button.module.scss",
      },
    ]);
    const ctx = readSourceExpressionContextAtCursor(baseParams, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const targets = resolveSourceExpressionDefinitionTargets(ctx!, baseParams, deps);

    expect(targets).toHaveLength(1);
    expect(targets[0]).toEqual({
      originRange: {
        start: { line: 4, character: 15 },
        end: { line: 4, character: 24 },
      },
      targetFilePath: "/fake/src/Button.module.scss",
      targetRange: {
        start: { line: 10, character: 0 },
        end: { line: 13, character: 1 },
      },
      targetSelectionRange: {
        start: { line: 11, character: 2 },
        end: { line: 11, character: 11 },
      },
    });
  });

  it("returns every matched target for a prefix-constrained expression", () => {
    const deps = makeDepsForExpressions([
      {
        kind: "template",
        origin: "cxCall",
        rawTemplate: "btn-${variant}",
        staticPrefix: "btn-",
        range: {
          start: { line: 4, character: 15 },
          end: { line: 4, character: 28 },
        },
        scssModulePath: "/fake/src/Button.module.scss",
      },
    ]);
    const ctx = readSourceExpressionContextAtCursor(baseParams, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const targets = resolveSourceExpressionDefinitionTargets(ctx!, baseParams, deps);

    expect(targets).toHaveLength(2);
    expect(
      targets.every((target) => target.targetFilePath === "/fake/src/Button.module.scss"),
    ).toBe(true);
  });

  it("can read targets through the rust source-resolution backend", () => {
    const deps = makeDepsForExpressions([
      {
        kind: "literal",
        origin: "cxCall",
        className: "indicator",
        range: {
          start: { line: 4, character: 15 },
          end: { line: 4, character: 24 },
        },
        scssModulePath: "/fake/src/Button.module.scss",
      },
    ]);
    const ctx = readSourceExpressionContextAtCursor(baseParams, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const targets = resolveSourceExpressionDefinitionTargets(ctx!, baseParams, deps, {
      env: {
        CME_SELECTED_QUERY_BACKEND: "rust-source-resolution",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionSelectorMatch: (_document) => ({
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: ["indicator"],
      }),
    });

    expect(targets).toHaveLength(1);
    expect(targets[0]?.targetSelectionRange).toEqual({
      start: { line: 11, character: 2 },
      end: { line: 11, character: 11 },
    });
  });

  it("keeps exact style accesses on the local definition path before rust", async () => {
    const deps = makeDepsForExpressions([
      {
        kind: "styleAccess",
        className: "indicator",
        range: {
          start: { line: 4, character: 15 },
          end: { line: 4, character: 24 },
        },
        scssModulePath: "/fake/src/Button.module.scss",
      },
    ]);
    const ctx = readSourceExpressionContextAtCursor(baseParams, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });
    let rustRunnerCalled = false;

    expect(ctx).not.toBeNull();
    const targets = await resolveSourceExpressionDefinitionTargetsAsync(ctx!, baseParams, deps, {
      env: {
        CME_SELECTED_QUERY_BACKEND: "rust-source-resolution",
      } as NodeJS.ProcessEnv,
      runRustSelectedQueryBackendJsonAsync: async () => {
        rustRunnerCalled = true;
        return { evaluatorCandidates: { results: [] } };
      },
    });

    expect(targets).toHaveLength(1);
    expect(targets[0]?.targetFilePath).toBe("/fake/src/Button.module.scss");
    expect(rustRunnerCalled).toBe(false);
  });
});

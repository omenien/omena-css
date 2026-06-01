import { describe, expect, it } from "vitest";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { readSourceExpressionContextAtCursor } from "../../../server/engine-core-ts/src/core/query";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import {
  planSourceExpressionRename,
  readSourceExpressionRenameTarget,
} from "../../../server/engine-host-node/src/source-rename-query";
import type { StyleSemanticGraphSummaryV0 } from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  infoAtLine,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";

const BINDING: CxBinding = {
  cxVarName: "cx",
  stylesVarName: "styles",
  scssModulePath: "/fake/src/Button.module.scss",
  classNamesImportName: "classNames",
  bindingRange: {
    start: { line: 2, character: 6 },
    end: { line: 2, character: 8 },
  },
};

const TSX = `import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const a = cx('indicator');
`;

function makeTsxDeps(
  expressions: Parameters<typeof buildTestClassExpressions>[0]["expressions"],
): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
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
        expressions,
      }),
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () => new Map([["indicator", infoAtLine("indicator", 1)]]),
    workspaceRoot: "/fake",
  });
}

describe("source rename query", () => {
  it("returns a rename target for a static source expression and plans edits", () => {
    const deps = makeTsxDeps([
      {
        kind: "literal",
        origin: "cxCall",
        className: "indicator",
        range: {
          start: { line: 3, character: 14 },
          end: { line: 3, character: 23 },
        },
        scssModulePath: BINDING.scssModulePath,
      },
    ]);
    const cursor = {
      documentUri: "file:///fake/src/App.tsx",
      content: TSX,
      filePath: "/fake/src/App.tsx",
      line: 3,
      character: 16,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const target = readSourceExpressionRenameTarget(ctx!, cursor, deps);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("indicator");

    const plan = planSourceExpressionRename(ctx!, cursor, deps, "status");
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits[0]?.newText : null).toBe("status");
  });

  it("blocks dynamic source expressions", () => {
    const deps = makeTsxDeps([
      {
        kind: "symbolRef",
        origin: "cxCall",
        rawReference: "size",
        range: {
          start: { line: 3, character: 14 },
          end: { line: 3, character: 18 },
        },
        scssModulePath: BINDING.scssModulePath,
      },
    ]);
    const cursor = {
      documentUri: "file:///fake/src/App.tsx",
      content: TSX.replace("cx('indicator')", "cx(size)"),
      filePath: "/fake/src/App.tsx",
      line: 3,
      character: 16,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const target = readSourceExpressionRenameTarget(ctx!, cursor, deps);
    expect(target).toEqual({ kind: "blocked", reason: "dynamicExpression" });
    expect(planSourceExpressionRename(ctx!, cursor, deps, "status")).toBeNull();
  });

  it("can read a rename target through the rust source-resolution backend", () => {
    const deps = makeTsxDeps([
      {
        kind: "literal",
        origin: "cxCall",
        className: "indicator",
        range: {
          start: { line: 3, character: 14 },
          end: { line: 3, character: 23 },
        },
        scssModulePath: BINDING.scssModulePath,
      },
    ]);
    const cursor = {
      documentUri: "file:///fake/src/App.tsx",
      content: TSX,
      filePath: "/fake/src/App.tsx",
      line: 3,
      character: 16,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const options = {
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-source-resolution",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionSelectorMatch: () => ({
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: ["indicator"],
      }),
    };
    const target = readSourceExpressionRenameTarget(ctx!, cursor, deps, options);
    expect(target.kind).toBe("target");
    expect(target.kind === "target" ? target.target.placeholder : null).toBe("indicator");

    const plan = planSourceExpressionRename(ctx!, cursor, deps, "status", options);
    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits[0]?.newText : null).toBe("status");
  });

  it("uses rust style semantic graph reference sites for source rename edits", () => {
    const deps = makeTsxDeps([
      {
        kind: "literal",
        origin: "cxCall",
        className: "indicator",
        range: {
          start: { line: 3, character: 14 },
          end: { line: 3, character: 23 },
        },
        scssModulePath: BINDING.scssModulePath,
      },
    ]);
    const cursor = {
      documentUri: "file:///fake/src/App.tsx",
      content: TSX,
      filePath: "/fake/src/App.tsx",
      line: 3,
      character: 16,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const plan = planSourceExpressionRename(ctx!, cursor, deps, "status", {
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionSelectorMatch: () => ({
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: ["indicator"],
      }),
      readRustStyleSemanticGraphForWorkspaceTarget: () => makeGraph("direct"),
    });

    expect(plan?.kind).toBe("plan");
    expect(plan?.kind === "plan" ? plan.plan.edits.map((edit) => edit.newText) : []).toEqual([
      "status",
      "status",
    ]);
    expect(plan?.kind === "plan" ? plan.plan.edits[1]?.uri : null).toBe(
      "file:///fake/src/FromGraph.tsx",
    );
  });

  it("uses rust style semantic graph blockers for source rename safety", () => {
    const deps = makeTsxDeps([
      {
        kind: "literal",
        origin: "cxCall",
        className: "indicator",
        range: {
          start: { line: 3, character: 14 },
          end: { line: 3, character: 23 },
        },
        scssModulePath: BINDING.scssModulePath,
      },
    ]);
    const cursor = {
      documentUri: "file:///fake/src/App.tsx",
      content: TSX,
      filePath: "/fake/src/App.tsx",
      line: 3,
      character: 16,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const target = readSourceExpressionRenameTarget(ctx!, cursor, deps, {
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionSelectorMatch: () => ({
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: ["indicator"],
      }),
      readRustStyleSemanticGraphForWorkspaceTarget: () => makeGraph("expanded"),
    });

    expect(target).toEqual({
      kind: "blocked",
      reason: "expandedReferences",
    });
  });
});

function makeGraph(referenceMode: "direct" | "expanded"): StyleSemanticGraphSummaryV0 {
  const hasExpandedReferences = referenceMode === "expanded";
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
          rewriteSafety: "safe",
          blockers: [],
        },
      ],
      rewriteSafety: {
        allCanonicalIdsRewriteSafe: true,
        safeCanonicalIds: ["selector:indicator"],
        blockedCanonicalIds: [],
        blockers: [],
      },
    },
    selectorReferenceEngine: {
      schemaVersion: "0",
      product: "omena-semantic.selector-references",
      stylePath: "/fake/src/Button.module.scss",
      selectorCount: 1,
      referencedSelectorCount: 1,
      unreferencedSelectorCount: 0,
      totalReferenceSites: hasExpandedReferences ? 2 : 1,
      selectors: [
        {
          canonicalId: "selector:indicator",
          filePath: "/fake/src/Button.module.scss",
          localName: "indicator",
          totalReferences: hasExpandedReferences ? 2 : 1,
          directReferenceCount: 1,
          editableDirectReferenceCount: 1,
          exactReferenceCount: 1,
          inferredOrBetterReferenceCount: hasExpandedReferences ? 2 : 1,
          hasExpandedReferences,
          hasStyleDependencyReferences: false,
          hasAnyReferences: true,
          sites: [
            {
              filePath: "/fake/src/FromGraph.tsx",
              range: {
                start: { line: 9, character: 4 },
                end: { line: 9, character: 13 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
          ],
          editableDirectSites: [
            {
              filePath: "/fake/src/FromGraph.tsx",
              range: {
                start: { line: 9, character: 4 },
                end: { line: 9, character: 13 },
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

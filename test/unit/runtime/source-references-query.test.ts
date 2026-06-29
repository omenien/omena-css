import { describe, expect, it } from "vitest";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { WorkspaceSemanticWorkspaceReferenceIndex } from "../../../server/engine-core-ts/src/core/semantic/workspace-reference-index";
import { readSourceExpressionContextAtCursor } from "../../../server/engine-core-ts/src/core/query";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { resolveSourceExpressionReferences } from "../../../server/engine-host-node/src/source-references-query";
import type { StyleSemanticGraphSummaryV0 } from "../../../server/engine-host-node/src/style-semantic-graph-query-backend";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  infoAtLine,
  makeBaseDeps,
  semanticSiteAt,
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
  const semanticReferenceIndex = new WorkspaceSemanticWorkspaceReferenceIndex();
  semanticReferenceIndex.record("file:///fake/src/App.tsx", [
    semanticSiteAt(
      "file:///fake/src/App.tsx",
      "indicator",
      3,
      "/fake/src/Button.module.scss",
      "indicator",
      { start: 14, end: 23 },
    ),
  ]);

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
        expressions,
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

const cursor = {
  documentUri: "file:///fake/src/App.tsx",
  content: TSX,
  filePath: "/fake/src/App.tsx",
  line: 3,
  character: 16,
  version: 1,
};

describe("resolveSourceExpressionReferences", () => {
  it("returns source-side reference locations for a static class expression", () => {
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
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    expect(resolveSourceExpressionReferences(ctx!, cursor, deps)).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 3, character: 14 },
          end: { line: 3, character: 23 },
        },
      },
    ]);
  });

  it("can resolve source targets through the rust source-resolution backend", () => {
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
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    expect(
      resolveSourceExpressionReferences(ctx!, cursor, deps, {
        env: {
          OMENA_SELECTED_QUERY_BACKEND: "rust-source-resolution",
        } as NodeJS.ProcessEnv,
        readRustSourceResolutionSelectorMatch: () => ({
          styleFilePath: "/fake/src/Button.module.scss",
          selectorNames: ["indicator"],
        }),
      }),
    ).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 3, character: 14 },
          end: { line: 3, character: 23 },
        },
      },
    ]);
  });

  it("can source usage locations through the rust selector-usage backend", () => {
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
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    expect(
      resolveSourceExpressionReferences(ctx!, cursor, deps, {
        env: {
          OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage",
        } as NodeJS.ProcessEnv,
        readRustSelectorUsagePayloadForWorkspaceTarget: () => ({
          canonicalName: "indicator",
          totalReferences: 2,
          directReferenceCount: 1,
          editableDirectReferenceCount: 1,
          exactReferenceCount: 1,
          inferredOrBetterReferenceCount: 1,
          hasExpandedReferences: false,
          hasStyleDependencyReferences: true,
          hasAnyReferences: true,
          allSites: [
            {
              filePath: "/fake/src/App.tsx",
              range: {
                start: { line: 3, character: 14 },
                end: { line: 3, character: 23 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
            {
              filePath: "/fake/src/Other.module.scss",
              range: {
                start: { line: 2, character: 1 },
                end: { line: 2, character: 10 },
              },
              expansion: "direct",
              referenceKind: "styleDependency",
            },
          ],
        }),
      }),
    ).toEqual([
      {
        uri: "file:///fake/src/App.tsx",
        range: {
          start: { line: 3, character: 14 },
          end: { line: 3, character: 23 },
        },
      },
      {
        uri: "file:///fake/src/Other.module.scss",
        range: {
          start: { line: 2, character: 1 },
          end: { line: 2, character: 10 },
        },
      },
    ]);
  });

  it("can use the unified rust-selected-query backend for source targets and usage locations", () => {
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
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    expect(
      resolveSourceExpressionReferences(ctx!, cursor, deps, {
        env: {
          OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
        } as NodeJS.ProcessEnv,
        readRustSourceResolutionSelectorMatch: () => ({
          styleFilePath: "/fake/src/Button.module.scss",
          selectorNames: ["indicator"],
        }),
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
          allSites: [
            {
              filePath: "/fake/src/FromRust.tsx",
              range: {
                start: { line: 9, character: 4 },
                end: { line: 9, character: 13 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
          ],
        }),
      }),
    ).toEqual([
      {
        uri: "file:///fake/src/FromRust.tsx",
        range: {
          start: { line: 9, character: 4 },
          end: { line: 9, character: 13 },
        },
      },
    ]);
  });

  it("rechecks rust selector-usage when rust graph has an empty selector reference summary", () => {
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
    const ctx = readSourceExpressionContextAtCursor(cursor, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    expect(
      resolveSourceExpressionReferences(ctx!, cursor, deps, {
        env: {
          OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
        } as NodeJS.ProcessEnv,
        readRustSourceResolutionSelectorMatch: () => ({
          styleFilePath: "/fake/src/Button.module.scss",
          selectorNames: ["indicator"],
        }),
        readRustStyleSemanticGraphForWorkspaceTarget: () => makeEmptyReferenceGraph(),
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
          allSites: [
            {
              filePath: "/fake/src/FromRustUsage.tsx",
              range: {
                start: { line: 9, character: 4 },
                end: { line: 9, character: 13 },
              },
              expansion: "direct",
              referenceKind: "source",
            },
          ],
        }),
      }),
    ).toEqual([
      {
        uri: "file:///fake/src/FromRustUsage.tsx",
        range: {
          start: { line: 9, character: 4 },
          end: { line: 9, character: 13 },
        },
      },
    ]);
  });
});

function makeEmptyReferenceGraph(): StyleSemanticGraphSummaryV0 {
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
      referencedSelectorCount: 0,
      unreferencedSelectorCount: 1,
      totalReferenceSites: 0,
      selectors: [
        {
          canonicalId: "selector:indicator",
          filePath: "/fake/src/Button.module.scss",
          localName: "indicator",
          totalReferences: 0,
          directReferenceCount: 0,
          editableDirectReferenceCount: 0,
          exactReferenceCount: 0,
          inferredOrBetterReferenceCount: 0,
          hasExpandedReferences: false,
          hasStyleDependencyReferences: false,
          hasAnyReferences: false,
          sites: [],
          editableDirectSites: [],
        },
      ],
    },
    sourceInputEvidence: {
      referenceSiteIdentity: {
        status: "ready",
        referenceSiteCount: 1,
      },
    },
    promotionEvidence: {},
    losslessCstContract: {},
  };
}

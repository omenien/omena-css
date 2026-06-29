import { describe, expect, it } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { readSourceExpressionContextAtCursor } from "../../../server/engine-core-ts/src/core/query";
import type { TypeResolver } from "../../../server/engine-core-ts/src/core/ts/type-resolver";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { resolveSourceExpressionHoverResult } from "../../../server/engine-host-node/src/source-hover-query";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";

const TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const el = cx('indicator');
`;

const SYMBOL_REF_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const size = pick();
const el = cx(size);
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

function makeDeps(): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf) }),
    parseClassExpressions: (_sf, bindings) =>
      buildTestClassExpressions({
        filePath: "/fake/src/Button.tsx",
        bindings,
        expressions:
          bindings.length === 0
            ? []
            : [
                {
                  kind: "literal",
                  origin: "cxCall",
                  className: "indicator",
                  range: {
                    start: { line: 4, character: 15 },
                    end: { line: 4, character: 24 },
                  },
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
    workspaceRoot: "/fake",
  });
}

function makeSymbolRefDeps(overrides: Partial<ProviderDeps> = {}): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const sourceFrontendAnalysis = createTestSourceFrontendAnalysis({
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf) }),
    parseClassExpressions: (_sf, bindings) =>
      buildTestClassExpressions({
        filePath: "/fake/src/Button.tsx",
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
                  range: {
                    start: { line: 5, character: 15 },
                    end: { line: 5, character: 19 },
                  },
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
    selectorMapForPath: () =>
      new Map([
        ["indicator", info("indicator")],
        ["active", info("active")],
      ]),
    workspaceRoot: "/fake",
    ...overrides,
  });
}

function throwingTypeResolver(message: string): TypeResolver {
  return {
    resolve: () => {
      throw new Error(message);
    },
    invalidate: () => {},
    clear: () => {},
  };
}

describe("resolveSourceExpressionHoverResult", () => {
  it("returns selectors, explanation, and selector dependency summaries", () => {
    const deps = makeDeps();
    const params = {
      documentUri: "file:///fake/src/Button.tsx",
      content: TSX,
      filePath: "/fake/src/Button.tsx",
      line: 4,
      character: 18,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(params, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const result = resolveSourceExpressionHoverResult(ctx!, params, deps);

    expect(result.selectors.map((selector) => selector.name)).toEqual(["indicator"]);
    expect(result.dynamicExplanation).toBeNull();
    expect(Array.from(result.styleDependenciesBySelector.keys())).toEqual(["indicator"]);
  });

  it("can source selectors from the rust source-resolution backend", () => {
    const deps = makeDeps();
    const params = {
      documentUri: "file:///fake/src/Button.tsx",
      content: TSX,
      filePath: "/fake/src/Button.tsx",
      line: 4,
      character: 18,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(params, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const result = resolveSourceExpressionHoverResult(ctx!, params, deps, {
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-source-resolution",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionSelectorMatch: () => ({
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: ["indicator"],
      }),
    });

    expect(result.selectors.map((selector) => selector.name)).toEqual(["indicator"]);
    expect(Array.from(result.styleDependenciesBySelector.keys())).toEqual(["indicator"]);
  });

  it("does not touch the TypeScript resolver when rust source-resolution returns a match", () => {
    const deps = makeSymbolRefDeps({
      typeResolver: throwingTypeResolver("TypeScript resolver should not run"),
    });
    const params = {
      documentUri: "file:///fake/src/Button.tsx",
      content: SYMBOL_REF_TSX,
      filePath: "/fake/src/Button.tsx",
      line: 5,
      character: 17,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(params, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const result = resolveSourceExpressionHoverResult(ctx!, params, deps, {
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-source-resolution",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionSelectorMatch: () => ({
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: ["active"],
      }),
    });

    expect(result.selectors.map((selector) => selector.name)).toEqual(["active"]);
    expect(result.dynamicExplanation).toBeNull();
  });

  it("can source selectors and explanation from the rust expression-semantics backend", () => {
    const deps = makeSymbolRefDeps();
    const params = {
      documentUri: "file:///fake/src/Button.tsx",
      content: SYMBOL_REF_TSX,
      filePath: "/fake/src/Button.tsx",
      line: 5,
      character: 17,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(params, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const result = resolveSourceExpressionHoverResult(ctx!, params, deps, {
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-expression-semantics",
      } as NodeJS.ProcessEnv,
      readRustExpressionSemanticsPayload: () => ({
        expressionId: "expr-1",
        expressionKind: "symbolRef",
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: ["indicator"],
        candidateNames: ["indicator", "active"],
        finiteValues: ["indicator", "active"],
        valueDomainKind: "finiteSet",
        selectorCertainty: "inferred",
        valueCertainty: "inferred",
        selectorCertaintyShapeKind: "boundedFinite",
        selectorCertaintyShapeLabel: "bounded selector set (1)",
        valueCertaintyShapeKind: "boundedFinite",
        valueCertaintyShapeLabel: "bounded finite (2)",
        valueDomainDerivation: sampleValueDomainDerivation(),
        valueDomainProvenanceTree: sampleValueDomainProvenanceTree(),
      }),
    });

    expect(result.selectors.map((selector) => selector.name)).toEqual(["indicator"]);
    expect(result.dynamicExplanation).toEqual(
      expect.objectContaining({
        kind: "symbolRef",
        subject: "size",
        candidates: ["indicator", "active"],
        valueDomainDerivationLabel: "finiteSet reduced to finiteSet via baseFromFacts",
        valueDomainProvenanceLabel:
          "finiteSetDomain: finiteSet (the class value is one of a bounded set)",
        valueCertainty: "inferred",
        valueCertaintyShapeLabel: "bounded finite (2)",
        valueCertaintyReasonLabel: "analysis preserved multiple finite candidate values",
        selectorCertainty: "inferred",
        selectorCertaintyShapeLabel: "bounded selector set (1)",
        reasonLabel: "branched local flow analysis",
      }),
    );
    expect(Array.from(result.styleDependenciesBySelector.keys())).toEqual(["indicator"]);
  });

  it("applies selected-query selector projections to hover semantics", () => {
    const deps = makeSymbolRefDeps({
      selectorMapForPath: () => new Map([["btn-primary", info("btn-primary")]]),
      typeResolver: throwingTypeResolver("TypeScript resolver should not run"),
    });
    const params = {
      documentUri: "file:///fake/src/Button.tsx",
      content: SYMBOL_REF_TSX,
      filePath: "/fake/src/Button.tsx",
      line: 5,
      character: 17,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(params, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });
    let expressionId = "";
    let projectionReads = 0;

    expect(ctx).not.toBeNull();
    const result = resolveSourceExpressionHoverResult(ctx!, params, deps, {
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
      } as NodeJS.ProcessEnv,
      readRustExpressionSemanticsPayload: (_document, id) => {
        expressionId = id;
        return {
          expressionId: id,
          expressionKind: "symbolRef",
          styleFilePath: "/fake/src/Button.module.scss",
          selectorNames: [],
          candidateNames: ["btn-primary"],
          finiteValues: ["btn-primary"],
          valueDomainKind: "finiteSet",
          selectorCertainty: "inferred",
          valueCertainty: "inferred",
          selectorCertaintyShapeKind: "unknown",
          selectorCertaintyShapeLabel: "unknown",
          valueCertaintyShapeKind: "boundedFinite",
          valueCertaintyShapeLabel: "bounded finite (1)",
          valueDomainDerivation: sampleValueDomainDerivation(),
        };
      },
      readRustExpressionDomainSelectorProjections: (_document, scssModulePath) => {
        projectionReads += 1;
        return [
          {
            graphId: "/fake/src/Button.tsx:expression-domain-flow",
            filePath: "/fake/src/Button.tsx",
            nodeId: expressionId,
            targetStylePaths: [scssModulePath],
            valueKind: "composite",
            selectorNames: ["btn-primary"],
            certainty: "inferred",
          },
        ];
      },
    });

    expect(projectionReads).toBe(1);
    expect(result.selectors.map((selector) => selector.name)).toEqual(["btn-primary"]);
    expect(result.dynamicExplanation).toEqual(
      expect.objectContaining({
        kind: "symbolRef",
        subject: "size",
        candidates: ["btn-primary"],
        valueCertainty: "inferred",
        valueCertaintyShapeLabel: "exact",
      }),
    );
    expect(Array.from(result.styleDependenciesBySelector.keys())).toEqual(["btn-primary"]);
  });

  it("falls back when unified rust expression semantics has no hoverable selectors", () => {
    const deps = makeDeps();
    const params = {
      documentUri: "file:///fake/src/Button.tsx",
      content: TSX,
      filePath: "/fake/src/Button.tsx",
      line: 4,
      character: 18,
      version: 1,
    };
    const ctx = readSourceExpressionContextAtCursor(params, {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
    });

    expect(ctx).not.toBeNull();
    const result = resolveSourceExpressionHoverResult(ctx!, params, deps, {
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
      } as NodeJS.ProcessEnv,
      readRustExpressionSemanticsPayload: () => ({
        expressionId: "expr-1",
        expressionKind: "literal",
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: [],
        candidateNames: [],
        finiteValues: [],
        valueDomainKind: "finiteSet",
        selectorCertainty: "exact",
        valueCertainty: "exact",
        selectorCertaintyShapeKind: "boundedFinite",
        selectorCertaintyShapeLabel: "bounded selector set (0)",
        valueCertaintyShapeKind: "boundedFinite",
        valueCertaintyShapeLabel: "bounded finite (0)",
      }),
      readRustExpressionDomainSelectorProjections: () => [],
      readRustSourceResolutionSelectorMatch: () => null,
    });

    expect(result.selectors.map((selector) => selector.name)).toEqual(["indicator"]);
    expect(Array.from(result.styleDependenciesBySelector.keys())).toEqual(["indicator"]);
  });
});

function sampleValueDomainDerivation() {
  return {
    schemaVersion: "0",
    product: "omena-abstract-value.reduced-class-value-derivation",
    inputFactKind: "finiteSet",
    inputValueCount: 2,
    reducedKind: "finiteSet",
    steps: [
      {
        operation: "baseFromFacts",
        resultKind: "finiteSet",
        reason: "mapped input facts to the base abstract value",
      },
    ],
  };
}

function sampleValueDomainProvenanceTree() {
  return {
    schemaVersion: "0",
    product: "omena-abstract-value.provenance-tree",
    valueKind: "finiteSet" as const,
    value: { kind: "finiteSet" as const, values: ["indicator", "active"] },
    root: {
      operation: "finiteSetDomain",
      resultKind: "finiteSet",
      reason: "the class value is one of a bounded set",
      children: [],
    },
  };
}

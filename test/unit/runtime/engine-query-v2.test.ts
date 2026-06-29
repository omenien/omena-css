import { describe, expect, it } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import type { TypeResolver } from "../../../server/engine-core-ts/src/core/ts/type-resolver";
import { buildSelectedQueryResultsV2 } from "../../../server/engine-host-node/src/engine-query-v2";
import {
  EMPTY_ALIAS_RESOLVER,
  buildTestClassExpressions,
  createTestSourceFrontendAnalysis,
  info,
  makeBaseDeps,
} from "../../_fixtures/test-helpers";

const SYMBOL_REF_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const size = pick();
const el = cx(size);
`;

const MULTI_SYMBOL_REF_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const size = pick();
const tone = pickTone();
const el = cx(size, tone);
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

function makeDeps(options: { readonly multiSymbolRefs?: boolean } = {}) {
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
                ...(options.multiSymbolRefs
                  ? [
                      {
                        kind: "symbolRef" as const,
                        origin: "cxCall" as const,
                        rawReference: "size",
                        rootName: "size",
                        pathSegments: [],
                        range: {
                          start: { line: 5, character: 15 },
                          end: { line: 5, character: 19 },
                        },
                        scssModulePath: bindings[0]!.scssModulePath,
                      },
                    ]
                  : []),
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
  });
}

function buildResults(options: Parameters<typeof buildSelectedQueryResultsV2>[0]) {
  return buildSelectedQueryResultsV2(options);
}

describe("buildSelectedQueryResultsV2", () => {
  it("can source source-resolution query results from the rust backend", () => {
    const deps = makeDeps();
    const sourceDocuments = [
      {
        uri: "file:///fake/src/Button.tsx",
        content: MULTI_SYMBOL_REF_TSX,
        filePath: "/fake/src/Button.tsx",
        version: 1,
      },
    ] as const;

    const results = buildResults({
      workspaceRoot: "/fake",
      classnameTransform: "asIs",
      pathAlias: {},
      sourceDocuments,
      styleFiles: ["/fake/src/Button.module.scss"],
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      semanticReferenceIndex: deps.semanticReferenceIndex,
      styleDependencyGraph: deps.styleDependencyGraph,
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-source-resolution",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionPayload: () => ({
        expressionId: "expr-1",
        styleFilePath: "/fake/src/Button.module.scss",
        selectorNames: ["indicator"],
        finiteValues: ["indicator", "active"],
        selectorCertainty: "inferred",
        valueCertainty: "inferred",
        selectorCertaintyShapeKind: "boundedFinite",
        selectorCertaintyShapeLabel: "bounded selector set (1)",
        valueCertaintyShapeKind: "boundedFinite",
        valueCertaintyShapeLabel: "bounded finite (2)",
      }),
    });

    expect(results).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "source-expression-resolution",
          filePath: "/fake/src/Button.tsx",
          payload: expect.objectContaining({
            selectorNames: ["indicator"],
            finiteValues: ["indicator", "active"],
            selectorCertaintyShapeKind: "boundedFinite",
            valueCertaintyShapeKind: "boundedFinite",
          }),
        }),
      ]),
    );
  });

  it("can source expression-semantics query results from the rust backend", () => {
    const deps = makeDeps();
    const sourceDocuments = [
      {
        uri: "file:///fake/src/Button.tsx",
        content: SYMBOL_REF_TSX,
        filePath: "/fake/src/Button.tsx",
        version: 1,
      },
    ] as const;

    const results = buildResults({
      workspaceRoot: "/fake",
      classnameTransform: "asIs",
      pathAlias: {},
      sourceDocuments,
      styleFiles: ["/fake/src/Button.module.scss"],
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      semanticReferenceIndex: deps.semanticReferenceIndex,
      styleDependencyGraph: deps.styleDependencyGraph,
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

    expect(results).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "expression-semantics",
          filePath: "/fake/src/Button.tsx",
          payload: expect.objectContaining({
            selectorNames: ["indicator"],
            candidateNames: ["indicator", "active"],
            valueDomainKind: "finiteSet",
            valueDomainDerivation: expect.objectContaining({
              product: "omena-abstract-value.reduced-class-value-derivation",
              reducedKind: "finiteSet",
            }),
            valueDomainProvenanceTree: expect.objectContaining({
              product: "omena-abstract-value.provenance-tree",
              valueKind: "finiteSet",
            }),
            selectorCertaintyShapeKind: "boundedFinite",
            valueCertaintyShapeKind: "boundedFinite",
          }),
        }),
      ]),
    );
  });

  it("can source selector-usage query results from the rust backend", () => {
    const deps = makeDeps();
    const sourceDocuments = [
      {
        uri: "file:///fake/src/Button.tsx",
        content: SYMBOL_REF_TSX,
        filePath: "/fake/src/Button.tsx",
        version: 1,
      },
    ] as const;

    const results = buildResults({
      workspaceRoot: "/fake",
      classnameTransform: "asIs",
      pathAlias: {},
      sourceDocuments,
      styleFiles: ["/fake/src/Button.module.scss"],
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      semanticReferenceIndex: deps.semanticReferenceIndex,
      styleDependencyGraph: deps.styleDependencyGraph,
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-selector-usage",
      } as NodeJS.ProcessEnv,
      readRustSelectorUsagePayload: (_options, filePath, canonicalName) => ({
        canonicalName,
        totalReferences: filePath.endsWith("Button.module.scss") ? 3 : 0,
        directReferenceCount: 2,
        editableDirectReferenceCount: 1,
        exactReferenceCount: 1,
        inferredOrBetterReferenceCount: 2,
        hasExpandedReferences: true,
        hasStyleDependencyReferences: true,
        hasAnyReferences: true,
      }),
    });

    expect(results).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "selector-usage",
          filePath: "/fake/src/Button.module.scss",
          payload: expect.objectContaining({
            canonicalName: "indicator",
            totalReferences: 3,
            directReferenceCount: 2,
            editableDirectReferenceCount: 1,
            exactReferenceCount: 1,
            inferredOrBetterReferenceCount: 2,
            hasExpandedReferences: true,
            hasStyleDependencyReferences: true,
            hasAnyReferences: true,
          }),
        }),
      ]),
    );
  });

  it("reuses rust payload lists while building selected query results", () => {
    const deps = makeDeps({ multiSymbolRefs: true });
    const sourceDocuments = [
      {
        uri: "file:///fake/src/Button.tsx",
        content: SYMBOL_REF_TSX,
        filePath: "/fake/src/Button.tsx",
        version: 1,
      },
    ] as const;
    let sourceResolutionPayloadReads = 0;
    let expressionSemanticsPayloadReads = 0;
    let selectorUsagePayloadReads = 0;

    const results = buildResults({
      workspaceRoot: "/fake",
      classnameTransform: "asIs",
      pathAlias: {},
      sourceDocuments,
      styleFiles: ["/fake/src/Button.module.scss"],
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      semanticReferenceIndex: deps.semanticReferenceIndex,
      styleDependencyGraph: deps.styleDependencyGraph,
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionPayloads: () => {
        sourceResolutionPayloadReads += 1;
        return [
          makeSourceResolutionPayload("class-expr:0", "indicator"),
          makeSourceResolutionPayload("class-expr:1", "active"),
        ];
      },
      readRustExpressionSemanticsPayloads: () => {
        expressionSemanticsPayloadReads += 1;
        return [
          makeExpressionSemanticsPayload("class-expr:0", "indicator"),
          makeExpressionSemanticsPayload("class-expr:1", "active"),
        ];
      },
      readRustSelectorUsagePayloads: () => {
        selectorUsagePayloadReads += 1;
        return [
          makeSelectorUsageCandidate("/fake/src/Button.module.scss", "indicator"),
          makeSelectorUsageCandidate("/fake/src/Button.module.scss", "active"),
        ];
      },
    });

    expect(sourceResolutionPayloadReads).toBe(1);
    expect(expressionSemanticsPayloadReads).toBe(1);
    expect(selectorUsagePayloadReads).toBe(1);
    expect(results.filter((result) => result.kind === "source-expression-resolution")).toHaveLength(
      2,
    );
    expect(results.filter((result) => result.kind === "expression-semantics")).toHaveLength(2);
    expect(results.filter((result) => result.kind === "selector-usage")).toHaveLength(2);
  });

  it("does not compute TypeScript fallbacks when rust-selected-query returns source payloads", () => {
    const deps = makeDeps();
    const sourceDocuments = [
      {
        uri: "file:///fake/src/Button.tsx",
        content: SYMBOL_REF_TSX,
        filePath: "/fake/src/Button.tsx",
        version: 1,
      },
    ] as const;

    const results = buildResults({
      workspaceRoot: "/fake",
      classnameTransform: "asIs",
      pathAlias: {},
      sourceDocuments,
      styleFiles: ["/fake/src/Button.module.scss"],
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: throwingTypeResolver("TypeScript fallback should not run"),
      semanticReferenceIndex: deps.semanticReferenceIndex,
      styleDependencyGraph: deps.styleDependencyGraph,
      env: {
        OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
      } as NodeJS.ProcessEnv,
      readRustSourceResolutionPayloads: () => [
        makeSourceResolutionPayload("class-expr:0", "indicator"),
      ],
      readRustExpressionSemanticsPayloads: () => [
        makeExpressionSemanticsPayload("class-expr:0", "indicator"),
      ],
      readRustSelectorUsagePayloads: () => [
        makeSelectorUsageCandidate("/fake/src/Button.module.scss", "indicator"),
        makeSelectorUsageCandidate("/fake/src/Button.module.scss", "active"),
      ],
    });

    expect(results.filter((result) => result.kind === "source-expression-resolution")).toHaveLength(
      1,
    );
    expect(results.filter((result) => result.kind === "expression-semantics")).toHaveLength(1);
  });
});

function makeSourceResolutionPayload(expressionId: string, selectorName: string) {
  return {
    expressionId,
    styleFilePath: "/fake/src/Button.module.scss",
    selectorNames: [selectorName],
    finiteValues: [selectorName],
    selectorCertainty: "inferred",
    valueCertainty: "inferred",
    selectorCertaintyShapeKind: "boundedFinite",
    selectorCertaintyShapeLabel: "bounded selector set (1)",
    valueCertaintyShapeKind: "boundedFinite",
    valueCertaintyShapeLabel: "bounded finite (1)",
  };
}

function makeExpressionSemanticsPayload(expressionId: string, selectorName: string) {
  return {
    expressionId,
    expressionKind: "symbolRef",
    styleFilePath: "/fake/src/Button.module.scss",
    selectorNames: [selectorName],
    candidateNames: [selectorName],
    finiteValues: [selectorName],
    valueDomainKind: "finiteSet",
    selectorCertainty: "inferred",
    valueCertainty: "inferred",
    selectorCertaintyShapeKind: "boundedFinite",
    selectorCertaintyShapeLabel: "bounded selector set (1)",
    valueCertaintyShapeKind: "boundedFinite",
    valueCertaintyShapeLabel: "bounded finite (1)",
    valueDomainDerivation: sampleValueDomainDerivation(),
    valueDomainProvenanceTree: sampleValueDomainProvenanceTree(),
  };
}

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
        inputKind: "finiteSet",
        resultKind: "finiteSet",
        reason: "finite type facts are preserved as exact candidate sets",
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

function makeSelectorUsageCandidate(filePath: string, canonicalName: string) {
  return {
    kind: "selector-usage" as const,
    filePath,
    queryId: canonicalName,
    payload: {
      canonicalName,
      totalReferences: 1,
      directReferenceCount: 1,
      editableDirectReferenceCount: 1,
      exactReferenceCount: 1,
      inferredOrBetterReferenceCount: 1,
      hasExpandedReferences: false,
      hasStyleDependencyReferences: false,
      hasAnyReferences: true,
    },
  };
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

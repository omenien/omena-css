import { describe, expect, it } from "vitest";
import type ts from "typescript";
import type { CxBinding } from "../../../server/engine-core-ts/src/core/cx/cx-types";
import type { ResolvedCxBinding } from "../../../server/engine-core-ts/src/core/cx/resolved-bindings";
import { DocumentAnalysisCache } from "../../../server/engine-core-ts/src/core/indexing/document-analysis-cache";
import { SourceFileCache } from "../../../server/engine-core-ts/src/core/ts/source-file-cache";
import type { TypeResolver } from "../../../server/engine-core-ts/src/core/ts/type-resolver";
import type { ProviderDeps } from "../../../server/lsp-server/src/providers/cursor-dispatch";
import { resolveSourceDiagnosticFindings } from "../../../server/engine-host-node/src/source-diagnostics-query";
import { FakeTypeResolver } from "../../_fixtures/fake-type-resolver";
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
const value = cx('unknonw');
`;

const SYMBOL_REF_TSX = `
import classNames from 'classnames/bind';
import styles from './Button.module.scss';
const cx = classNames.bind(styles);
const size = pick();
const value = cx(size);
`;

const detectCxBindings = (_sourceFile: ts.SourceFile): CxBinding[] => [
  {
    cxVarName: "cx",
    stylesVarName: "styles",
    scssModulePath: "/fake/ws/src/Button.module.scss",
    classNamesImportName: "classNames",
    bindingRange: {
      start: { line: 3, character: 6 },
      end: { line: 3, character: 8 },
    },
  },
];

const parseClassExpressions = (_sf: ts.SourceFile, bindings: readonly ResolvedCxBinding[]) =>
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
              className: "unknonw",
              range: { start: { line: 4, character: 17 }, end: { line: 4, character: 26 } },
              scssModulePath: bindings[0]!.scssModulePath,
            },
          ],
  });

function makeDeps(): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf, fp) }),
    parseClassExpressions,
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () =>
      new Map([
        ["indicator", info("indicator")],
        ["unknown", info("unknown")],
      ]),
    typeResolver: new FakeTypeResolver(),
    workspaceRoot: "/fake/ws",
  });
}

function makeSymbolRefDeps(options: { readonly typeResolver?: TypeResolver } = {}): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf, fp) }),
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
                  range: { start: { line: 5, character: 17 }, end: { line: 5, character: 21 } },
                  scssModulePath: bindings[0]!.scssModulePath,
                },
              ],
      }),
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () => new Map([["unknown", info("unknown")]]),
    typeResolver: options.typeResolver ?? new FakeTypeResolver(["small", "large"]),
    workspaceRoot: "/fake/ws",
  });
}

function makeProjectedSymbolRefDeps(): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf, fp) }),
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
                  range: { start: { line: 5, character: 17 }, end: { line: 5, character: 21 } },
                  scssModulePath: bindings[0]!.scssModulePath,
                },
              ],
      }),
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () => new Map([["btn-primary", info("btn-primary")]]),
    typeResolver: throwingTypeResolver(),
    workspaceRoot: "/fake/ws",
  });
}

function makeMultiSymbolRefDeps(): ProviderDeps {
  const sourceFileCache = new SourceFileCache({ max: 10 });
  const analysisCache = new DocumentAnalysisCache({
    sourceFileCache,
    fileExists: () => true,
    aliasResolver: EMPTY_ALIAS_RESOLVER,
    scanCxImports: (sf, fp) => ({ stylesBindings: new Map(), bindings: detectCxBindings(sf, fp) }),
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
                  range: { start: { line: 5, character: 17 }, end: { line: 5, character: 21 } },
                  scssModulePath: bindings[0]!.scssModulePath,
                },
                {
                  kind: "symbolRef",
                  origin: "cxCall",
                  rawReference: "tone",
                  rootName: "tone",
                  pathSegments: [],
                  range: { start: { line: 6, character: 17 }, end: { line: 6, character: 21 } },
                  scssModulePath: bindings[0]!.scssModulePath,
                },
              ],
      }),
    max: 10,
  });
  return makeBaseDeps({
    analysisCache,
    selectorMapForPath: () => new Map([["unknown", info("unknown")]]),
    typeResolver: new FakeTypeResolver(["small", "large"]),
    workspaceRoot: "/fake/ws",
  });
}

describe("resolveSourceDiagnosticFindings", () => {
  it("returns source checker findings through the host boundary", () => {
    const findings = resolveSourceDiagnosticFindings(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: TSX,
        filePath: "/fake/ws/src/Button.tsx",
        version: 1,
      },
      makeDeps(),
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      code: "missing-static-class",
      className: "unknonw",
      suggestion: "unknown",
    });
  });

  it("can source symbol-ref diagnostics from the rust expression-semantics backend", () => {
    const findings = resolveSourceDiagnosticFindings(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: SYMBOL_REF_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        version: 1,
      },
      makeSymbolRefDeps(),
      {
        env: {
          OMENA_SELECTED_QUERY_BACKEND: "rust-expression-semantics",
        } as NodeJS.ProcessEnv,
        readRustExpressionSemanticsPayload: () => ({
          expressionId: "expr-1",
          expressionKind: "symbolRef",
          styleFilePath: "/fake/ws/src/Button.module.scss",
          selectorNames: [],
          candidateNames: ["small", "large"],
          finiteValues: ["small", "large"],
          valueDomainKind: "finiteSet",
          selectorCertainty: "possible",
          valueCertainty: "inferred",
          selectorCertaintyShapeKind: "unknown",
          selectorCertaintyShapeLabel: "unknown",
          valueCertaintyShapeKind: "boundedFinite",
          valueCertaintyShapeLabel: "bounded finite (2)",
          valueDomainDerivation: sampleValueDomainDerivation(),
        }),
      },
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      code: "missing-resolved-class-values",
      missingValues: ["small", "large"],
      valueCertainty: "inferred",
      selectorCertainty: "possible",
      reason: "flowBranch",
      valueDomainDerivation: sampleValueDomainDerivation(),
    });
  });

  it("does not read the TypeScript fallback when rust semantics are complete", () => {
    const findings = resolveSourceDiagnosticFindings(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: SYMBOL_REF_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        version: 1,
      },
      makeSymbolRefDeps({ typeResolver: throwingTypeResolver() }),
      {
        env: {
          OMENA_SELECTED_QUERY_BACKEND: "rust-expression-semantics",
        } as NodeJS.ProcessEnv,
        readRustExpressionSemanticsPayload: () => ({
          expressionId: "expr-1",
          expressionKind: "symbolRef",
          styleFilePath: "/fake/ws/src/Button.module.scss",
          selectorNames: [],
          candidateNames: ["small", "large"],
          finiteValues: ["small", "large"],
          valueDomainKind: "finiteSet",
          selectorCertainty: "possible",
          valueCertainty: "inferred",
          selectorCertaintyShapeKind: "unknown",
          selectorCertaintyShapeLabel: "unknown",
          valueCertaintyShapeKind: "boundedFinite",
          valueCertaintyShapeLabel: "bounded finite (2)",
        }),
      },
    );

    expect(findings).toHaveLength(1);
    expect(findings[0]).toMatchObject({
      code: "missing-resolved-class-values",
      missingValues: ["small", "large"],
    });
  });

  it("reuses rust expression-semantics payloads across symbol refs in the same style module", () => {
    let payloadReads = 0;

    const findings = resolveSourceDiagnosticFindings(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: SYMBOL_REF_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        version: 1,
      },
      makeMultiSymbolRefDeps(),
      {
        env: {
          OMENA_SELECTED_QUERY_BACKEND: "rust-expression-semantics",
        } as NodeJS.ProcessEnv,
        readRustExpressionSemanticsPayloads: () => {
          payloadReads += 1;
          return [
            {
              expressionId: "class-expr:0",
              expressionKind: "symbolRef",
              styleFilePath: "/fake/ws/src/Button.module.scss",
              selectorNames: [],
              candidateNames: ["small"],
              finiteValues: ["small"],
              valueDomainKind: "finiteSet",
              selectorCertainty: "possible",
              valueCertainty: "inferred",
              selectorCertaintyShapeKind: "unknown",
              selectorCertaintyShapeLabel: "unknown",
              valueCertaintyShapeKind: "boundedFinite",
              valueCertaintyShapeLabel: "bounded finite (1)",
            },
            {
              expressionId: "class-expr:1",
              expressionKind: "symbolRef",
              styleFilePath: "/fake/ws/src/Button.module.scss",
              selectorNames: [],
              candidateNames: ["large"],
              finiteValues: ["large"],
              valueDomainKind: "finiteSet",
              selectorCertainty: "possible",
              valueCertainty: "inferred",
              selectorCertaintyShapeKind: "unknown",
              selectorCertaintyShapeLabel: "unknown",
              valueCertaintyShapeKind: "boundedFinite",
              valueCertaintyShapeLabel: "bounded finite (1)",
            },
          ];
        },
      },
    );

    expect(payloadReads).toBe(1);
    expect(findings).toHaveLength(2);
    expect(findings.map((finding) => finding.code)).toEqual([
      "missing-resolved-class-values",
      "missing-resolved-class-values",
    ]);
  });

  it("uses selected-query selector projections before reporting missing rust domains", () => {
    let projectionReads = 0;

    const findings = resolveSourceDiagnosticFindings(
      {
        documentUri: "file:///fake/ws/src/Button.tsx",
        content: SYMBOL_REF_TSX,
        filePath: "/fake/ws/src/Button.tsx",
        version: 1,
      },
      makeProjectedSymbolRefDeps(),
      {
        env: {
          OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
        } as NodeJS.ProcessEnv,
        readRustExpressionSemanticsPayload: () => ({
          expressionId: "class-expr:0",
          expressionKind: "symbolRef",
          styleFilePath: "/fake/ws/src/Button.module.scss",
          selectorNames: [],
          candidateNames: [],
          valueDomainKind: "constrained",
          selectorCertainty: "possible",
          valueCertainty: "inferred",
          selectorCertaintyShapeKind: "unknown",
          selectorCertaintyShapeLabel: "unknown",
          valueCertaintyShapeKind: "constrained",
          valueCertaintyShapeLabel: "constrained composite",
          valueConstraintKind: "composite",
          valuePrefix: "btn-",
          valueMinLen: "btn-primary".length,
        }),
        readRustExpressionDomainSelectorProjections: () => {
          projectionReads += 1;
          return [
            {
              graphId: "/fake/ws/src/Button.tsx:expression-domain-flow",
              filePath: "/fake/ws/src/Button.tsx",
              nodeId: "class-expr:0",
              targetStylePaths: ["/fake/ws/src/Button.module.scss"],
              valueKind: "composite",
              selectorNames: ["btn-primary"],
              certainty: "inferred",
            },
          ];
        },
      },
    );

    expect(projectionReads).toBe(1);
    expect(findings).toEqual([]);
  });
});

function throwingTypeResolver(): TypeResolver {
  return {
    resolve: () => {
      throw new Error("unexpected TypeScript fallback");
    },
    invalidate: () => {},
    clear: () => {},
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
        reason: "preserved finite string literal facts",
      },
    ],
  } as const;
}

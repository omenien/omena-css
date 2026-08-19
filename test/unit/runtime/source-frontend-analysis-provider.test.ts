import { pathToFileURL } from "node:url";
import { describe, expect, it, vi } from "vitest";
import {
  createDefaultRustSourceFrontendAnalysisProvider,
  createRequiredRustSourceFrontendAnalysisProvider,
  resolveSourceFrontendBackendKind,
  SourceFrontendAnalysisError,
  SourceFrontendAnalysisOutcomeCountersV0,
  type SourceFrontendAnalysisOutcomeV0,
} from "../../../server/engine-host-node/src/source-frontend-analysis-provider";
import type { OmenaNapiSourceFrontendBinding } from "../../../server/engine-host-node/src/omena-napi-source-frontend-binding";
import { EMPTY_ALIAS_RESOLVER } from "../../_fixtures/test-helpers";

describe("source frontend analysis provider", () => {
  it("defaults to the Rust source frontend and rejects the retired TS override", () => {
    expect(resolveSourceFrontendBackendKind({})).toBe("rust-source-frontend");
    expect(
      resolveSourceFrontendBackendKind({ OMENA_SOURCE_FRONTEND_BACKEND: "rust-source-frontend" }),
    ).toBe("rust-source-frontend");
    expect(() =>
      resolveSourceFrontendBackendKind({ OMENA_SOURCE_FRONTEND_BACKEND: "typescript-current" }),
    ).toThrow("Unknown source frontend backend: typescript-current");
  });

  it("requires native Rust frontend facts for supported source files", () => {
    const provider = createRequiredRustSourceFrontendAnalysisProvider({
      aliasResolver: () => EMPTY_ALIAS_RESOLVER,
      fileExists: () => true,
      loadBinding: () => null,
    });

    expect(() =>
      provider({
        filePath: "/fake/ws/src/Button.tsx",
        content: 'const el = <div className="button" />;',
      }),
    ).toThrow(SourceFrontendAnalysisError);
    expect(
      provider({
        filePath: "/fake/ws/src/Button.module.scss",
        content: ".button {}",
      }),
    ).toBeNull();
  });

  it("resolves only Rust-declared import identities before projecting the binding index", () => {
    const source = [
      'import bind from "classnames/bind";',
      'import styles from "./Button.module.scss";',
      "const cx = bind.bind(styles);",
      'const el = cx("indicator");',
      "",
    ].join("\n");
    const sourcePath = "/fake/ws/src/Button.tsx";
    const styleUri = pathToFileURL("/fake/ws/src/Button.module.scss").href;
    const styleDeclaration = rustImportDeclaration(source, "styles", "./Button.module.scss");
    const cxDeclarationId = rustDeclarationId(source, "localVar", "cx", "const cx");
    const importSummary = rustImportSummary([
      rustImportDeclaration(source, "bind", "classnames/bind"),
      styleDeclaration,
    ]);
    const variantSpan = byteSpanFor(source, "indicator", 'cx("indicator"');
    const readSourceBindingIndexJson = vi.fn(
      (
        _sourcePath: string,
        _source: string,
        _sourceLanguage: string,
        styleImportResolutionsJson: string,
      ) => {
        expect(JSON.parse(styleImportResolutionsJson)).toEqual([
          { declarationId: styleDeclaration.declarationId, styleUri },
        ]);
        return JSON.stringify({
          ...emptyBindingIndex(source),
          bindingDecls: [
            {
              kind: "import",
              name: "styles",
              importPath: "./Button.module.scss",
              byteSpan: byteSpanFor(source, "styles", 'styles from "./Button.module.scss"'),
            },
            {
              kind: "localVar",
              name: "cx",
              byteSpan: byteSpanFor(source, "cx", "const cx"),
            },
          ],
          scopeContainsDecls: [
            scopeContains(source, "import", "styles", "./Button.module.scss"),
            scopeContains(source, "localVar", "cx"),
          ],
          styleImportBindings: [
            {
              declarationId: styleDeclaration.declarationId,
              localName: "styles",
              styleUri,
            },
          ],
          declaresStyleImports: [{ declName: "styles", stylesLocalName: "styles", styleUri }],
          styleImportResolvesModules: [{ stylesLocalName: "styles", styleUri }],
          classExpressionNodes: [
            {
              kind: "literal",
              byteSpan: variantSpan,
              targetStyleUri: styleUri,
            },
          ],
          expressionTargetsModules: [{ byteSpan: variantSpan, targetStyleUri: styleUri }],
          classnamesBindUtilityBindings: [
            {
              declarationId: cxDeclarationId,
              localName: "cx",
              stylesLocalName: "styles",
              styleUri,
              classnamesImportName: "bind",
            },
          ],
          declaresUtilityBindings: [
            {
              declarationId: cxDeclarationId,
              declName: "cx",
              utilityLocalName: "cx",
              utilityKind: "classnamesBind",
            },
          ],
          utilityUsesStyleImports: [
            { utilityLocalName: "cx", stylesLocalName: "styles", styleUri },
          ],
        });
      },
    );
    const readSourceSyntaxIndexJson = vi.fn(() =>
      JSON.stringify({
        classValueUniverses: [
          {
            pluginId: "cva-recipe-domain",
            domain: "cva-recipe",
            ownerName: "button",
            classNames: ["button", "button-primary"],
            axes: [{ axisName: "tone", values: ["primary"] }],
            byteSpan: variantSpan,
          },
        ],
        domainClassReferences: [
          {
            byteSpan: variantSpan,
            pluginId: "cva-recipe-domain",
            domain: "cva-recipe",
            ownerName: "button",
            axisName: "tone",
            optionName: "primary",
          },
        ],
      }),
    );
    const provider = createDefaultRustSourceFrontendAnalysisProvider({
      aliasResolver: () => EMPTY_ALIAS_RESOLVER,
      fileExists: () => true,
      loadBinding: () => ({
        readSourceImportDeclarations: () => importSummary,
        readSourceBindingIndexJson,
        readSourceSyntaxIndexJson,
      }),
    });

    const result = successResult(provider({ filePath: sourcePath, content: source }));

    expect(readSourceBindingIndexJson).toHaveBeenCalledTimes(1);
    expect(result.sourceDocument.styleImports).toMatchObject([
      { localName: "styles", resolved: { absolutePath: "/fake/ws/src/Button.module.scss" } },
    ]);
    expect(result.sourceDocument.utilityBindings).toMatchObject([
      { kind: "classnamesBind", localName: "cx", stylesLocalName: "styles" },
    ]);
    expect(result.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "indicator" },
    ]);
    expect(result.sourceDocument.domainClassReferences).toMatchObject([
      { matchKind: "literal", className: "button.tone.primary", domain: "cva-recipe" },
    ]);
    expect(result.classValueUniverses).toMatchObject([
      {
        pluginId: "cva-recipe-domain",
        domain: "cva-recipe",
        ownerName: "button",
        universe: { kind: "finite", classNames: ["button", "button-primary"] },
      },
    ]);
  });

  it("preserves unresolved Rust-declared style imports for missing-module diagnostics", () => {
    const source = [
      'import styles from "./Missing.module.scss";',
      "export const Button = () => <div className={styles.root}>hi</div>;",
      "",
    ].join("\n");
    const sourcePath = "/fake/ws/src/Button.tsx";
    const missingPath = "/fake/ws/src/Missing.module.scss";
    const declaration = rustImportDeclaration(source, "styles", "./Missing.module.scss");
    const provider = createDefaultRustSourceFrontendAnalysisProvider({
      aliasResolver: () => EMPTY_ALIAS_RESOLVER,
      fileExists: () => false,
      loadBinding: () => ({
        readSourceImportDeclarations: () => rustImportSummary([declaration]),
        readSourceBindingIndexJson: (_path, _source, _language, resolutionsJson) => {
          expect(JSON.parse(resolutionsJson)).toEqual([]);
          return JSON.stringify(emptyBindingIndex(source));
        },
      }),
    });

    const result = successResult(provider({ filePath: sourcePath, content: source }));

    expect(result.sourceDocument.styleImports).toMatchObject([
      {
        localName: "styles",
        resolved: {
          kind: "missing",
          absolutePath: missingPath,
          specifier: "./Missing.module.scss",
        },
      },
    ]);
    expect(result.sourceDocument.styleImports[0]?.range).toEqual({
      start: { line: 0, character: source.indexOf("./Missing.module.scss") },
      end: {
        line: 0,
        character: source.indexOf("./Missing.module.scss") + "./Missing.module.scss".length,
      },
    });
  });

  it("distinguishes and counts N-API, JSON, and projection failures", () => {
    const counters = new SourceFrontendAnalysisOutcomeCountersV0();
    const sourceInput = {
      filePath: "/fake/ws/src/Button.tsx",
      content: 'import styles from "./Button.module.scss";',
    };
    const outcomes = [
      providerOutcome(counters, {
        readSourceImportDeclarations: () => {
          throw new Error("native exploded");
        },
        readSourceBindingIndexJson: () => "{}",
      })(sourceInput),
      providerOutcome(counters, {
        readSourceImportDeclarations: () => rustImportSummary([]),
        readSourceBindingIndexJson: () => "{not-json",
      })(sourceInput),
      providerOutcome(counters, {
        readSourceImportDeclarations: () => rustImportSummary([]),
        readSourceBindingIndexJson: () => "{}",
      })(sourceInput),
    ];

    expect(outcomes.map((outcome) => outcome.kind)).toEqual([
      "napiFailure",
      "jsonFailure",
      "projectionFailure",
    ]);
    expect(counters.snapshot()).toEqual({
      total: 3,
      success: 0,
      unsupportedLanguage: 0,
      bindingUnavailable: 0,
      napiFailure: 1,
      jsonFailure: 1,
      projectionFailure: 1,
    });
  });
});

function providerOutcome(
  outcomeCounters: SourceFrontendAnalysisOutcomeCountersV0,
  binding: OmenaNapiSourceFrontendBinding,
) {
  return createDefaultRustSourceFrontendAnalysisProvider({
    aliasResolver: () => EMPTY_ALIAS_RESOLVER,
    fileExists: () => true,
    loadBinding: () => binding,
    outcomeCounters,
  });
}

function successResult(outcome: SourceFrontendAnalysisOutcomeV0) {
  if (outcome.kind !== "success") {
    throw new Error(
      `Expected source frontend success, received ${outcome.kind}: ${outcome.detail}`,
    );
  }
  return outcome.result;
}

function rustImportDeclaration(source: string, binding: string, specifier: string) {
  const bindingStart = source.indexOf(binding);
  const specifierStart = source.indexOf(specifier);
  if (bindingStart < 0 || specifierStart < 0) {
    throw new Error(`Missing import fixture token: ${binding} from ${specifier}`);
  }
  return {
    declarationId: `rust-decl:import:${binding}:${bindingStart}:${bindingStart + binding.length}:${specifier}`,
    binding,
    specifier,
    specifierByteSpan: {
      start: Buffer.byteLength(source.slice(0, specifierStart), "utf8"),
      end: Buffer.byteLength(source.slice(0, specifierStart + specifier.length), "utf8"),
    },
  };
}

function rustImportSummary(imports: ReturnType<typeof rustImportDeclaration>[]) {
  return {
    schemaVersion: "0",
    product: "omena-bridge.source-import-declarations",
    importCount: imports.length,
    imports,
  };
}

function rustDeclarationId(
  source: string,
  kind: "import" | "localVar",
  name: string,
  searchContext: string,
  importPath = "",
) {
  const contextStart = source.indexOf(searchContext);
  if (contextStart < 0) throw new Error(`Missing declaration context: ${searchContext}`);
  const start = source.indexOf(name, contextStart);
  if (start < 0) throw new Error(`Missing declaration fixture token: ${name}`);
  return `rust-decl:${kind}:${name}:${start}:${start + name.length}:${importPath}`;
}

function emptyBindingIndex(source: string) {
  return {
    schemaVersion: "0",
    product: "omena.source-binding-index",
    bindingScopes: [{ kind: "sourceFile", byteSpan: byteSpanForWholeSource(source) }],
    scopeParentEdges: [],
    bindingDecls: [],
    scopeContainsDecls: [],
    styleImportBindings: [],
    declaresStyleImports: [],
    styleImportResolvesModules: [],
    classExpressionNodes: [],
    expressionTargetsModules: [],
    classnamesBindUtilityBindings: [],
    classUtilBindings: [],
    declaresUtilityBindings: [],
    utilityUsesStyleImports: [],
    styleAccessUsesStyleImports: [],
    symbolRefUsesDecls: [],
  };
}

function scopeContains(
  source: string,
  declKind: "import" | "localVar",
  declName: string,
  importPath?: string,
) {
  return {
    scopeKind: "sourceFile",
    scopeByteSpan: byteSpanForWholeSource(source),
    declKind,
    declName,
    declByteSpan: byteSpanFor(source, declName),
    ...(importPath ? { importPath } : {}),
  };
}

function byteSpanForWholeSource(source: string) {
  return { start: 0, end: Buffer.byteLength(source, "utf8") };
}

function byteSpanFor(source: string, token: string, searchContext?: string) {
  const contextStart = searchContext ? source.indexOf(searchContext) : 0;
  if (contextStart < 0) throw new Error(`Missing search context: ${searchContext}`);
  const start = source.indexOf(token, contextStart);
  if (start < 0) throw new Error(`Missing token: ${token}`);
  return {
    start: Buffer.byteLength(source.slice(0, start), "utf8"),
    end: Buffer.byteLength(source.slice(0, start + token.length), "utf8"),
  };
}

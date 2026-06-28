import { pathToFileURL } from "node:url";
import { describe, expect, it, vi } from "vitest";
import {
  createDefaultRustSourceFrontendAnalysisProvider,
  resolveSourceFrontendBackendKind,
} from "../../../server/engine-host-node/src/source-frontend-analysis-provider";
import { EMPTY_ALIAS_RESOLVER } from "../../_fixtures/test-helpers";

describe("source frontend analysis provider", () => {
  it("defaults to the rust source frontend and keeps an explicit TS fallback", () => {
    expect(resolveSourceFrontendBackendKind({})).toBe("rust-source-frontend");
    expect(
      resolveSourceFrontendBackendKind({ OMENA_SOURCE_FRONTEND_BACKEND: "rust-source-frontend" }),
    ).toBe("rust-source-frontend");
    expect(
      resolveSourceFrontendBackendKind({ OMENA_SOURCE_FRONTEND_BACKEND: "typescript-current" }),
    ).toBe("typescript-current");
  });

  it("projects native binding index output into the analysis cache contract", () => {
    const source = [
      'import bind from "classnames/bind";',
      'import styles from "./Button.module.scss";',
      "const cx = bind.bind(styles);",
      'const el = cx("indicator");',
      "",
    ].join("\n");
    const sourcePath = "/fake/ws/src/Button.tsx";
    const styleUri = pathToFileURL("/fake/ws/src/Button.module.scss").href;
    const variantSpan = byteSpanFor(source, "indicator", 'cx("indicator"');
    const readSourceBindingIndexJson = vi.fn(
      (
        _sourcePath: string,
        _source: string,
        _sourceLanguage: string,
        importedStyleBindingsJson: string,
        classnamesBindBindingsJson: string,
      ) => {
        expect(JSON.parse(importedStyleBindingsJson)).toEqual([{ binding: "styles", styleUri }]);
        expect(JSON.parse(classnamesBindBindingsJson)).toEqual(["bind"]);
        return JSON.stringify({
          schemaVersion: "0",
          product: "omena.source-binding-index",
          bindingScopes: [{ kind: "sourceFile", byteSpan: byteSpanForWholeSource(source) }],
          scopeParentEdges: [],
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
          styleImportBindings: [{ localName: "styles", styleUri }],
          declaresStyleImports: [{ declName: "styles", stylesLocalName: "styles", styleUri }],
          styleImportResolvesModules: [{ stylesLocalName: "styles", styleUri }],
          classExpressionNodes: [
            {
              kind: "literal",
              byteSpan: variantSpan,
              targetStyleUri: styleUri,
            },
          ],
          expressionTargetsModules: [
            {
              byteSpan: variantSpan,
              targetStyleUri: styleUri,
            },
          ],
          classnamesBindUtilityBindings: [
            {
              localName: "cx",
              stylesLocalName: "styles",
              styleUri,
              classnamesImportName: "bind",
            },
          ],
          classUtilBindings: [],
          declaresUtilityBindings: [
            { declName: "cx", utilityLocalName: "cx", utilityKind: "classnamesBind" },
          ],
          utilityUsesStyleImports: [
            { utilityLocalName: "cx", stylesLocalName: "styles", styleUri },
          ],
          styleAccessUsesStyleImports: [],
          symbolRefUsesDecls: [],
        });
      },
    );
    const readSourceSyntaxIndexJson = vi.fn(() =>
      JSON.stringify({
        schemaVersion: "0",
        product: "omena.source-syntax-index",
        importedStyleBindings: [{ binding: "styles", styleUri }],
        classStringLiterals: [],
        stylePropertyAccesses: [],
        inlineStyleDeclarations: [],
        selectorReferences: [],
        typeFactTargets: [],
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
      loadBinding: () => ({ readSourceBindingIndexJson, readSourceSyntaxIndexJson }),
    });

    const projected = provider({ filePath: sourcePath, content: source });

    expect(readSourceBindingIndexJson).toHaveBeenCalledTimes(1);
    expect(projected?.sourceDocument.styleImports).toMatchObject([
      { localName: "styles", resolved: { absolutePath: "/fake/ws/src/Button.module.scss" } },
    ]);
    expect(projected?.sourceDocument.utilityBindings).toMatchObject([
      { kind: "classnamesBind", localName: "cx", stylesLocalName: "styles" },
    ]);
    expect(projected?.sourceDocument.classExpressions).toMatchObject([
      { kind: "literal", className: "indicator" },
    ]);
    expect(projected?.sourceDocument.domainClassReferences).toMatchObject([
      { matchKind: "literal", className: "button.tone.primary", domain: "cva-recipe" },
    ]);
    expect(projected?.classValueUniverses).toMatchObject([
      {
        pluginId: "cva-recipe-domain",
        domain: "cva-recipe",
        ownerName: "button",
        universe: { kind: "finite", classNames: ["button", "button-primary"] },
      },
    ]);
  });
});

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

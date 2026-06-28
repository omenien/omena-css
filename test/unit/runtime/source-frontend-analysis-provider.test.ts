import { pathToFileURL } from "node:url";
import { describe, expect, it, vi } from "vitest";
import {
  createDefaultRustSourceFrontendAnalysisProvider,
  resolveSourceFrontendBackendKind,
} from "../../../server/engine-host-node/src/source-frontend-analysis-provider";
import { EMPTY_ALIAS_RESOLVER } from "../../_fixtures/test-helpers";

describe("source frontend analysis provider", () => {
  it("requires an explicit rust source frontend backend selection", () => {
    expect(resolveSourceFrontendBackendKind({})).toBe("typescript-current");
    expect(
      resolveSourceFrontendBackendKind({ OMENA_SOURCE_FRONTEND_BACKEND: "rust-source-frontend" }),
    ).toBe("rust-source-frontend");
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
              byteSpan: byteSpanFor(source, "indicator", 'cx("indicator"'),
              targetStyleUri: styleUri,
            },
          ],
          expressionTargetsModules: [
            {
              byteSpan: byteSpanFor(source, "indicator", 'cx("indicator"'),
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
    const provider = createDefaultRustSourceFrontendAnalysisProvider({
      aliasResolver: () => EMPTY_ALIAS_RESOLVER,
      fileExists: () => true,
      loadBinding: () => ({ readSourceBindingIndexJson }),
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

import { describe, expect, it } from "vitest";
import {
  findImportBindingDeclId,
  resolveBindingAtOffset,
} from "../../../server/engine-core-ts/src/core/binder/source-binding-graph";
import { projectRustSourceBindingIndexV0 } from "../../../server/engine-core-ts/src/core/source-frontend/rust-binding-index-projection";

const styleUri = "file:///fake/ws/src/Card.module.scss";
const stylePath = "/fake/ws/src/Card.module.scss";

describe("projectRustSourceBindingIndexV0", () => {
  it("projects Rust binding facts into the TS binder and graph contracts", () => {
    const source = [
      "// 한글 prefix keeps byte offsets different from UTF-16 offsets",
      'import bind from "classnames/bind";',
      'import styles from "./Card.module.scss";',
      "const cx = bind.bind(styles);",
      "export function Card() {",
      '  const size = "card";',
      '  return <div className={cx("card", size, styles.icon)} />;',
      "}",
      "",
    ].join("\n");
    const cxDeclarationId = rustDeclarationId(source, "localVar", "cx", "const cx");
    const projected = projectRustSourceBindingIndexV0({
      filePath: "/fake/ws/src/Card.tsx",
      source,
      language: "typescriptreact",
      index: {
        bindingScopes: [
          { kind: "sourceFile", byteSpan: byteSpanForWholeSource(source) },
          { kind: "function", byteSpan: byteSpanFor(source, "function Card() {", "function Card") },
        ],
        scopeParentEdges: [
          {
            childKind: "function",
            childByteSpan: byteSpanFor(source, "function Card() {", "function Card"),
            parentKind: "sourceFile",
            parentByteSpan: byteSpanForWholeSource(source),
          },
        ],
        bindingDecls: [
          {
            kind: "import",
            name: "bind",
            importPath: "classnames/bind",
            byteSpan: byteSpanFor(source, "bind", 'bind from "classnames/bind"'),
          },
          {
            kind: "import",
            name: "styles",
            importPath: "./Card.module.scss",
            byteSpan: byteSpanFor(source, "styles", 'styles from "./Card.module.scss"'),
          },
          {
            kind: "localVar",
            name: "cx",
            byteSpan: byteSpanFor(source, "cx", "const cx"),
          },
          {
            kind: "localVar",
            name: "size",
            byteSpan: byteSpanFor(source, "size", "const size"),
          },
        ],
        scopeContainsDecls: [
          scopeContains(source, "import", "bind", "classnames/bind", 'bind from "classnames/bind"'),
          scopeContains(
            source,
            "import",
            "styles",
            "./Card.module.scss",
            'styles from "./Card.module.scss"',
          ),
          scopeContains(source, "localVar", "cx", undefined, "const cx"),
          scopeContains(source, "localVar", "size", undefined, "const size"),
        ],
        styleImportBindings: [{ localName: "styles", styleUri }],
        declaresStyleImports: [{ declName: "styles", stylesLocalName: "styles", styleUri }],
        styleImportResolvesModules: [{ stylesLocalName: "styles", styleUri }],
        classExpressionNodes: [
          {
            kind: "literal",
            byteSpan: byteSpanFor(source, "card", 'cx("card"'),
            targetStyleUri: styleUri,
          },
          {
            kind: "symbolRef",
            byteSpan: byteSpanFor(source, "size", ", size,"),
            targetStyleUri: styleUri,
          },
          {
            kind: "styleAccess",
            byteSpan: byteSpanFor(source, "icon", "styles.icon"),
            targetStyleUri: styleUri,
          },
        ],
        expressionTargetsModules: [
          { byteSpan: byteSpanFor(source, "card", 'cx("card"'), targetStyleUri: styleUri },
          { byteSpan: byteSpanFor(source, "size", ", size,"), targetStyleUri: styleUri },
          { byteSpan: byteSpanFor(source, "icon", "styles.icon"), targetStyleUri: styleUri },
        ],
        classnamesBindUtilityBindings: [
          {
            declarationId: cxDeclarationId,
            localName: "cx",
            stylesLocalName: "styles",
            styleUri,
            classnamesImportName: "bind",
          },
        ],
        classUtilBindings: [],
        declaresUtilityBindings: [
          {
            declarationId: cxDeclarationId,
            declName: "cx",
            utilityLocalName: "cx",
            utilityKind: "classnamesBind",
          },
        ],
        utilityUsesStyleImports: [{ utilityLocalName: "cx", stylesLocalName: "styles", styleUri }],
        styleAccessUsesStyleImports: [
          {
            byteSpan: byteSpanFor(source, "icon", "styles.icon"),
            declName: "styles",
            stylesLocalName: "styles",
            styleUri,
          },
        ],
        symbolRefUsesDecls: [
          {
            byteSpan: byteSpanFor(source, "size", ", size,"),
            rawReference: "size",
            rootName: "size",
            declName: "size",
            styleUri,
          },
        ],
      },
    });

    expect(projected.sourceBinder.decls.find((decl) => decl.name === "bind")?.span.start).toBe(
      source.indexOf("bind"),
    );
    expect(projected.sourceDocument.styleImports).toMatchObject([
      {
        localName: "styles",
        resolved: { kind: "resolved", absolutePath: stylePath },
      },
    ]);
    expect(projected.sourceDocument.utilityBindings).toMatchObject([
      {
        kind: "classnamesBind",
        localName: "cx",
        stylesLocalName: "styles",
        scssModulePath: stylePath,
        classNamesImportName: "bind",
      },
    ]);
    expect(projected.sourceDocument.classExpressions.map((expression) => expression.kind)).toEqual([
      "literal",
      "styleAccess",
      "symbolRef",
    ]);
    expect(findImportBindingDeclId(projected.sourceBindingGraph, "styles")).toBeTruthy();
    expect(
      resolveBindingAtOffset(projected.sourceBindingGraph, "size", source.indexOf("size,")),
    ).toMatchObject({ declId: expect.stringContaining("size") });
    expect(
      resolveBindingAtOffset(projected.sourceBindingGraph, "styles", source.indexOf("styles.icon")),
    ).toMatchObject({ declId: expect.stringContaining("styles") });
    expect(projected.sourceBindingGraph.edges.map((edge) => edge.kind)).toEqual(
      expect.arrayContaining([
        "declaresStyleImport",
        "declaresUtilityBinding",
        "expressionTargetsModule",
        "expressionUsesDecl",
        "styleImportResolvesModule",
        "utilityUsesStyleImport",
      ]),
    );
  });

  it("uses the span-owned utility declaration identity when names are shadowed", () => {
    const source = [
      'import bind from "classnames/bind";',
      'import styles from "./Card.module.scss";',
      'const cx = "unrelated";',
      "export function Card() {",
      "  const cx = bind.bind(styles);",
      '  return <div className={cx("card")} />;',
      "}",
      "",
    ].join("\n");
    const outerSpan = byteSpanFor(source, "cx", 'const cx = "unrelated"');
    const innerSpan = byteSpanFor(source, "cx", "const cx = bind.bind");
    const innerDeclarationId = rustDeclarationId(source, "localVar", "cx", "const cx = bind.bind");
    const projected = projectRustSourceBindingIndexV0({
      filePath: "/fake/ws/src/Card.tsx",
      source,
      language: "typescriptreact",
      index: {
        bindingScopes: [{ kind: "sourceFile", byteSpan: byteSpanForWholeSource(source) }],
        scopeParentEdges: [],
        bindingDecls: [
          { kind: "localVar", name: "cx", byteSpan: outerSpan },
          { kind: "localVar", name: "cx", byteSpan: innerSpan },
        ],
        scopeContainsDecls: [
          {
            scopeKind: "sourceFile",
            scopeByteSpan: byteSpanForWholeSource(source),
            declKind: "localVar",
            declName: "cx",
            declByteSpan: outerSpan,
          },
          {
            scopeKind: "sourceFile",
            scopeByteSpan: byteSpanForWholeSource(source),
            declKind: "localVar",
            declName: "cx",
            declByteSpan: innerSpan,
          },
        ],
        styleImportBindings: [],
        declaresStyleImports: [],
        styleImportResolvesModules: [],
        classExpressionNodes: [],
        expressionTargetsModules: [],
        classnamesBindUtilityBindings: [
          {
            declarationId: innerDeclarationId,
            localName: "cx",
            stylesLocalName: "styles",
            styleUri,
            classnamesImportName: "bind",
          },
        ],
        classUtilBindings: [],
        declaresUtilityBindings: [
          {
            declarationId: innerDeclarationId,
            declName: "cx",
            utilityLocalName: "cx",
            utilityKind: "classnamesBind",
          },
        ],
        utilityUsesStyleImports: [],
        styleAccessUsesStyleImports: [],
        symbolRefUsesDecls: [],
      },
    });

    expect(projected.sourceDocument.utilityBindings[0]?.bindingDeclId).toBe(innerDeclarationId);
    expect(
      projected.sourceBindingGraph.edges.find((edge) => edge.kind === "declaresUtilityBinding")
        ?.from,
    ).toBe(`decl:${innerDeclarationId}`);
    expect(innerDeclarationId).not.toContain(`${outerSpan.start}:${outerSpan.end}`);
  });
});

function scopeContains(
  source: string,
  declKind: "import" | "localVar" | "parameter",
  declName: string,
  importPath: string | undefined,
  searchContext: string,
) {
  return {
    scopeKind: "sourceFile" as const,
    scopeByteSpan: byteSpanForWholeSource(source),
    declKind,
    declName,
    declByteSpan: byteSpanFor(source, declName, searchContext),
    ...(importPath ? { importPath } : {}),
  };
}

function byteSpanForWholeSource(source: string) {
  return { start: 0, end: Buffer.byteLength(source, "utf8") };
}

function byteSpanFor(source: string, token: string, searchContext: string) {
  const contextStart = source.indexOf(searchContext);
  if (contextStart === -1) throw new Error(`missing search context: ${searchContext}`);
  const start = source.indexOf(token, contextStart);
  if (start === -1) throw new Error(`missing token: ${token}`);
  return {
    start: Buffer.byteLength(source.slice(0, start), "utf8"),
    end: Buffer.byteLength(source.slice(0, start + token.length), "utf8"),
  };
}

function rustDeclarationId(
  source: string,
  kind: "import" | "localVar" | "parameter",
  name: string,
  searchContext: string,
  importPath = "",
) {
  const contextStart = source.indexOf(searchContext);
  if (contextStart === -1) throw new Error(`missing search context: ${searchContext}`);
  const start = source.indexOf(name, contextStart);
  if (start === -1) throw new Error(`missing declaration name: ${name}`);
  return `rust-decl:${kind}:${name}:${start}:${start + name.length}:${importPath}`;
}

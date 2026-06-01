import { describe, expect, it } from "vitest";
import ts from "typescript";
import { buildSourceBinder } from "../../../server/engine-core-ts/src/core/binder/binder-builder";
import {
  buildSourceBindingGraph,
  listStyleModulePaths,
} from "../../../server/engine-core-ts/src/core/binder/source-binding-graph";
import {
  makeClassUtilBinding,
  makeSourceDocumentHIR,
  makeStyleAccessClassExpression,
  makeStyleImportBinding,
  makeSymbolRefClassExpression,
} from "../../../server/engine-core-ts/src/core/hir/source-types";
import type { StyleImport } from "@omena/shared";

describe("buildSourceBindingGraph", () => {
  it("materializes declaration-backed source facts and style-module targets", () => {
    const sourceText = `
import classNames from "classnames/bind";
import styles from "./Button.module.scss";
const cx = classNames.bind(styles);
function render(size: string) {
  return [cx(size), styles.indicator];
}
`;
    const sourceFile = ts.createSourceFile(
      "/fake/ws/src/Button.tsx",
      sourceText,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );
    const binder = buildSourceBinder(sourceFile);
    const stylesDeclId = binder.decls.find((decl) => decl.name === "styles")?.id;
    const cxDeclId = binder.decls.find((decl) => decl.name === "cx")?.id;
    const sizeDeclId = binder.decls.find((decl) => decl.name === "size")?.id;

    expect(stylesDeclId).toBeDefined();
    expect(cxDeclId).toBeDefined();
    expect(sizeDeclId).toBeDefined();

    const sourceDocument = makeSourceDocumentHIR({
      filePath: "/fake/ws/src/Button.tsx",
      language: "tsx",
      styleImports: [
        makeStyleImportBinding("style:styles", "styles", stylesDeclId!, {
          kind: "resolved",
          absolutePath: "/fake/ws/src/Button.module.scss",
        } satisfies StyleImport),
      ],
      utilityBindings: [makeClassUtilBinding("util:cx", "cx", cxDeclId!)],
      classExpressions: [
        makeSymbolRefClassExpression(
          "ref:size",
          "cxCall",
          "/fake/ws/src/Button.module.scss",
          "size",
          "size",
          [],
          rangeAt(sourceFile, "size", "last"),
          sizeDeclId!,
        ),
        makeStyleAccessClassExpression(
          "ref:indicator",
          "/fake/ws/src/Button.module.scss",
          stylesDeclId!,
          "indicator",
          ["indicator"],
          rangeAt(sourceFile, "indicator", "last"),
        ),
      ],
    });

    const graph = buildSourceBindingGraph(sourceDocument, binder);

    expect(listStyleModulePaths(graph)).toEqual(["/fake/ws/src/Button.module.scss"]);
    expect(
      graph.edges.some(
        (edge) =>
          edge.kind === "declaresStyleImport" &&
          edge.from === `decl:${stylesDeclId}` &&
          edge.to === "styleImport:style:styles",
      ),
    ).toBe(true);
    expect(
      graph.edges.some(
        (edge) =>
          edge.kind === "expressionUsesDecl" &&
          edge.from === "expression:ref:size" &&
          edge.to === `decl:${sizeDeclId}`,
      ),
    ).toBe(true);
    expect(
      graph.edges.some(
        (edge) =>
          edge.kind === "expressionUsesDecl" &&
          edge.from === "expression:ref:indicator" &&
          edge.to === `decl:${stylesDeclId}`,
      ),
    ).toBe(true);
  });
});

function rangeAt(sourceFile: ts.SourceFile, token: string, occurrence: "first" | "last") {
  const start =
    occurrence === "last" ? sourceFile.text.lastIndexOf(token) : sourceFile.text.indexOf(token);
  if (start === -1) throw new Error(`Token not found: ${token}`);
  const end = start + token.length;
  const startLc = sourceFile.getLineAndCharacterOfPosition(start);
  const endLc = sourceFile.getLineAndCharacterOfPosition(end);
  return {
    start: { line: startLc.line, character: startLc.character },
    end: { line: endLc.line, character: endLc.character },
  };
}

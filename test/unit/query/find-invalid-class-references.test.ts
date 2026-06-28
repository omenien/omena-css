import ts from "typescript";
import { describe, expect, it } from "vitest";
import { reducedProductClassValueUniverseV0 } from "../../../server/engine-core-ts/src/core/abstract-value/class-value-universe";
import {
  finiteSetClassValue,
  prefixClassValue,
} from "../../../server/engine-core-ts/src/core/abstract-value/class-value-domain";
import type { ClassExpressionHIR } from "../../../server/engine-core-ts/src/core/hir/source-types";
import { findInvalidClassReference } from "../../../server/engine-core-ts/src/core/query/find-invalid-class-references";
import { FakeTypeResolver } from "../../_fixtures/fake-type-resolver";
import { info } from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";

const SCSS_PATH = "/fake/ws/src/Button.module.scss";

function styleDocument(selectors: ReadonlyMap<string, ReturnType<typeof info>>) {
  return buildStyleDocumentFromSelectorMap(SCSS_PATH, selectors);
}

describe("findInvalidClassReference", () => {
  it("reports a missing static class with a suggestion", () => {
    const sourceFile = ts.createSourceFile(
      "/fake/ws/src/Button.tsx",
      "cx('indicaror');",
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );
    const expression: ClassExpressionHIR = {
      kind: "literal",
      id: "expr:literal",
      origin: "cxCall",
      className: "indicaror",
      range: rangeForToken(sourceFile, "indicaror"),
      scssModulePath: SCSS_PATH,
    };

    expect(
      findInvalidClassReference(
        expression,
        styleDocument(new Map([["indicator", info("indicator")]])),
        {
          typeResolver: new FakeTypeResolver(),
          filePath: "/fake/ws/src/Button.tsx",
          workspaceRoot: "/fake/ws",
        },
      ),
    ).toMatchObject({
      kind: "missingStaticClass",
      suggestion: "indicator",
    });
  });

  it("uses provided flow-like symbol values before consulting the type resolver", () => {
    const sourceText = ["const size = enabled ? 'indicator' : 'missing';", "cx(size);"].join("\n");
    const sourceFile = ts.createSourceFile(
      "/fake/ws/src/Button.tsx",
      sourceText,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );
    const expression: ClassExpressionHIR = {
      kind: "symbolRef",
      id: "expr:symbol",
      origin: "cxCall",
      rawReference: "size",
      rootName: "size",
      pathSegments: [],
      range: rangeForLastToken(sourceFile, "size"),
      scssModulePath: SCSS_PATH,
    };

    expect(
      findInvalidClassReference(
        expression,
        styleDocument(new Map([["indicator", info("indicator")]])),
        {
          typeResolver: new FakeTypeResolver(),
          filePath: "/fake/ws/src/Button.tsx",
          workspaceRoot: "/fake/ws",
          resolveSymbolValues: () => ({
            abstractValue: finiteSetClassValue(["indicator", "missing"]),
            valueCertainty: "inferred",
            reason: "flowBranch",
          }),
        },
      ),
    ).toMatchObject({
      kind: "missingResolvedClassValues",
      missingValues: ["missing"],
      reason: "flowBranch",
      valueCertainty: "inferred",
      selectorCertainty: "inferred",
    });
  });

  it("falls back to type-union values when flow cannot resolve the symbol", () => {
    const sourceFile = ts.createSourceFile(
      "/fake/ws/src/Button.tsx",
      "cx(size);",
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );
    const expression: ClassExpressionHIR = {
      kind: "symbolRef",
      id: "expr:symbol",
      origin: "cxCall",
      rawReference: "size",
      rootName: "size",
      pathSegments: [],
      range: rangeForToken(sourceFile, "size"),
      scssModulePath: SCSS_PATH,
    };

    expect(
      findInvalidClassReference(expression, styleDocument(new Map([["small", info("small")]])), {
        typeResolver: new FakeTypeResolver(["small", "large"]),
        filePath: "/fake/ws/src/Button.tsx",
        workspaceRoot: "/fake/ws",
      }),
    ).toMatchObject({
      kind: "missingResolvedClassValues",
      missingValues: ["large"],
      reason: "typeUnion",
      valueCertainty: "inferred",
      selectorCertainty: "inferred",
    });
  });

  it("reports unresolved non-finite domains when no selector matches the resolved prefix", () => {
    const sourceFile = ts.createSourceFile(
      "/fake/ws/src/Button.tsx",
      "cx(size);",
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );
    const expression: ClassExpressionHIR = {
      kind: "symbolRef",
      id: "expr:symbol",
      origin: "cxCall",
      rawReference: "size",
      rootName: "size",
      pathSegments: [],
      range: rangeForToken(sourceFile, "size"),
      scssModulePath: SCSS_PATH,
    };

    expect(
      findInvalidClassReference(expression, styleDocument(new Map([["button", info("button")]])), {
        typeResolver: new FakeTypeResolver(),
        filePath: "/fake/ws/src/Button.tsx",
        workspaceRoot: "/fake/ws",
        resolveSymbolValues: () => ({
          abstractValue: prefixClassValue("ghost-"),
          values: [],
          valueCertainty: "inferred",
          reason: "flowBranch",
        }),
      }),
    ).toMatchObject({
      kind: "missingResolvedClassDomain",
      abstractValue: { kind: "prefix", prefix: "ghost-" },
      reason: "flowBranch",
      valueCertainty: "inferred",
      selectorCertainty: "possible",
    });
  });

  it("scopes symbol projection through owner-matched provider universes", () => {
    const sourceFile = ts.createSourceFile(
      "/fake/ws/src/Button.tsx",
      "cx(button);",
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TSX,
    );
    const expression: ClassExpressionHIR = {
      kind: "symbolRef",
      id: "expr:symbol",
      origin: "cxCall",
      rawReference: "button",
      rootName: "button",
      pathSegments: [],
      range: rangeForToken(sourceFile, "button"),
      scssModulePath: SCSS_PATH,
    };

    expect(
      findInvalidClassReference(
        expression,
        styleDocument(
          new Map([
            ["button_base", info("button_base")],
            ["global_unknown", info("global_unknown")],
          ]),
        ),
        {
          typeResolver: new FakeTypeResolver(),
          filePath: "/fake/ws/src/Button.tsx",
          workspaceRoot: "/fake/ws",
          classValueUniverses: [
            {
              id: "universe:button",
              pluginId: "cva-recipe-domain",
              domain: "cva-recipe",
              ownerName: "button",
              universe: reducedProductClassValueUniverseV0({
                baseClassNames: ["button_base"],
                axes: [],
              }),
            },
          ],
          resolveSymbolValues: () => ({
            abstractValue: prefixClassValue("global_"),
            valueCertainty: "inferred",
            reason: "flowBranch",
          }),
        },
      ),
    ).toMatchObject({
      kind: "missingResolvedClassDomain",
      abstractValue: { kind: "prefix", prefix: "global_" },
      reason: "flowBranch",
      valueCertainty: "inferred",
      selectorCertainty: "possible",
    });
  });
});

function rangeForToken(sourceFile: ts.SourceFile, token: string) {
  const start = sourceFile.text.indexOf(token);
  if (start === -1) throw new Error(`Token not found: ${token}`);
  const end = start + token.length;
  return toRange(sourceFile, start, end);
}

function rangeForLastToken(sourceFile: ts.SourceFile, token: string) {
  const start = sourceFile.text.lastIndexOf(token);
  if (start === -1) throw new Error(`Token not found: ${token}`);
  const end = start + token.length;
  return toRange(sourceFile, start, end);
}

function toRange(sourceFile: ts.SourceFile, start: number, end: number) {
  const startLc = sourceFile.getLineAndCharacterOfPosition(start);
  const endLc = sourceFile.getLineAndCharacterOfPosition(end);
  return {
    start: { line: startLc.line, character: startLc.character },
    end: { line: endLc.line, character: endLc.character },
  };
}

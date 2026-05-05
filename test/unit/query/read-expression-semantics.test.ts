import ts from "typescript";
import { describe, expect, it } from "vitest";
import {
  compositeClassValue,
  prefixClassValue,
} from "../../../server/engine-core-ts/src/core/abstract-value/class-value-domain";
import { readExpressionSemantics } from "../../../server/engine-core-ts/src/core/query/read-expression-semantics";
import type { ClassExpressionHIR } from "../../../server/engine-core-ts/src/core/hir/source-types";
import { FakeTypeResolver } from "../../_fixtures/fake-type-resolver";
import { info } from "../../_fixtures/test-helpers";
import { buildStyleDocumentFromSelectorMap } from "../../_fixtures/style-documents";

const SCSS_PATH = "/fake/ws/src/Button.module.scss";

function styleDocument(names: string[]) {
  return buildStyleDocumentFromSelectorMap(
    SCSS_PATH,
    new Map(names.map((name) => [name, info(name)])),
  );
}

describe("readExpressionSemantics", () => {
  it("surfaces finite candidate names for union-like symbol values", () => {
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

    const semantics = readExpressionSemantics(
      {
        expression,
        sourceFile,
        styleDocument: styleDocument(["small", "large"]),
      },
      {
        typeResolver: new FakeTypeResolver(["small", "large"]),
        filePath: "/fake/ws/src/Button.tsx",
        workspaceRoot: "/fake/ws",
      },
    );

    expect(semantics.valueDomainKind).toBe("finiteSet");
    expect(semantics.candidateNames).toEqual(["large", "small"]);
    expect(semantics.selectorNames).toEqual(["large", "small"]);
  });

  it("surfaces selector candidates for non-finite prefix domains", () => {
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

    const semantics = readExpressionSemantics(
      {
        expression,
        sourceFile,
        styleDocument: styleDocument(["btn-primary", "btn-secondary", "card"]),
      },
      {
        typeResolver: new FakeTypeResolver(),
        filePath: "/fake/ws/src/Button.tsx",
        workspaceRoot: "/fake/ws",
        resolveSymbolValues: () => ({
          abstractValue: prefixClassValue("btn-"),
          valueCertainty: "inferred",
          reason: "flowBranch",
        }),
      },
    );

    expect(semantics.valueDomainKind).toBe("prefix");
    expect(semantics.finiteValues).toBeNull();
    expect(semantics.candidateNames).toEqual(["btn-primary", "btn-secondary"]);
    expect(semantics.selectorNames).toEqual(["btn-primary", "btn-secondary"]);
  });

  it("preserves constrained reduced-product derivation metadata", () => {
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

    const semantics = readExpressionSemantics(
      {
        expression,
        sourceFile,
        styleDocument: styleDocument(["btn-primary-active", "btn-secondary-active", "card"]),
      },
      {
        typeResolver: new FakeTypeResolver(),
        filePath: "/fake/ws/src/Button.tsx",
        workspaceRoot: "/fake/ws",
        resolveSymbolValues: () => ({
          abstractValue: compositeClassValue({
            prefix: "btn-",
            suffix: "-active",
            minLength: 18,
            mustChars: "-abceintv",
            mayChars: "-abcdeinoprstv",
            mayIncludeOtherChars: true,
            provenance: "compositeConcat",
          }),
          valueCertainty: "inferred",
          reason: "flowBranch",
        }),
      },
    );

    expect(semantics.valueDomainKind).toBe("constrained");
    expect(semantics.valueDomainDerivation).toMatchObject({
      inputFactKind: "constrained",
      inputConstraintKind: "composite",
      reducedKind: "constrained",
      steps: [
        {
          operation: "baseFromFacts",
          resultKind: "composite",
          resultProvenance: "compositeConcat",
        },
      ],
    });
    expect(semantics.valueDomainProvenanceTree).toMatchObject({
      product: "omena-abstract-value.provenance-tree",
      valueKind: "composite",
      valueProvenance: "compositeConcat",
      root: {
        operation: "reducedProductConcat",
        resultKind: "composite",
        resultProvenance: "compositeConcat",
      },
    });
    expect(semantics.valueDomainProvenanceTree?.root.children.map((child) => child.detail)).toEqual(
      expect.arrayContaining(["prefix=btn-", "suffix=-active", "minLength=18"]),
    );
    expect(semantics.selectorNames).toEqual(["btn-primary-active", "btn-secondary-active"]);
  });
});

function rangeForToken(sourceFile: ts.SourceFile, token: string) {
  const start = sourceFile.text.indexOf(token);
  if (start === -1) throw new Error(`Token not found: ${token}`);
  const end = start + token.length;
  const startLc = sourceFile.getLineAndCharacterOfPosition(start);
  const endLc = sourceFile.getLineAndCharacterOfPosition(end);
  return {
    start: { line: startLc.line, character: startLc.character },
    end: { line: endLc.line, character: endLc.character },
  };
}

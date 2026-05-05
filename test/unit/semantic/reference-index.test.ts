import { describe, expect, it } from "vitest";
import { buildSourceSemanticGraph } from "../../_support/semantic-graph-fixture";
import { buildSemanticReferenceIndex } from "../../_support/semantic-reference-index-fixture";
import { loadSourceScenario, loadStyleScenario } from "../../_fixtures/scenario-corpus";

describe("buildSemanticReferenceIndex", () => {
  it("returns direct and inferred selector references from the source graph", () => {
    const sourceScenario = loadSourceScenario({
      id: "01-basic",
      sourcePath: "01-basic/BasicScenario.tsx",
    });
    const styleScenario = loadStyleScenario({
      id: "01-basic-style",
      stylePath: "01-basic/Button.module.scss",
    });

    const graph = buildSourceSemanticGraph({
      sourceDocument: sourceScenario.sourceDocument,
      styleDocumentsByPath: new Map([[styleScenario.filePath, styleScenario.styleDocument]]),
      resolveSymbolValues: (ref) =>
        ref.rootName === "size"
          ? {
              abstractValue: { kind: "finiteSet", values: ["lg", "md", "sm"] },
              valueCertainty: "inferred",
              reason: "typeUnion",
            }
          : null,
    });
    const index = buildSemanticReferenceIndex(graph);

    expect(index.findSelectorReferences(styleScenario.filePath, "button")).toEqual([
      expect.objectContaining({
        refId: "class-expr:0",
        origin: "cxCall",
        canonicalName: "button",
        className: "button",
        selectorCertainty: "exact",
        reason: "literal",
        expansion: "direct",
        abstractValue: { kind: "exact", value: "button" },
      }),
      expect.objectContaining({
        refId: "class-expr:4",
        origin: "cxCall",
        canonicalName: "button",
        className: "button",
        selectorCertainty: "exact",
        reason: "literal",
        expansion: "direct",
      }),
      expect.objectContaining({
        refId: "class-expr:7",
        origin: "cxCall",
        canonicalName: "button",
        className: "button",
        selectorCertainty: "exact",
        reason: "literal",
        expansion: "direct",
      }),
    ]);

    expect(index.findSelectorReferences(styleScenario.filePath, "sm")).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          refId: "class-expr:2",
          canonicalName: "sm",
          className: "sm",
          selectorCertainty: "exact",
          reason: "typeUnion",
          expansion: "expanded",
          abstractValue: { kind: "finiteSet", values: ["lg", "md", "sm"] },
        }),
        expect.objectContaining({
          refId: "class-expr:6",
          canonicalName: "sm",
          className: "sm",
          selectorCertainty: "exact",
          reason: "typeUnion",
          expansion: "expanded",
        }),
      ]),
    );
    expect(index.countSelectorReferences(styleScenario.filePath, "sm")).toBe(2);
    expect(
      index.findSelectorReferences(styleScenario.filePath, "sm", {
        minimumSelectorCertainty: "exact",
      }),
    ).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          refId: "class-expr:2",
          selectorCertainty: "exact",
          expansion: "expanded",
        }),
      ]),
    );
    expect(index.findTargetsForRef("class-expr:2")).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          canonicalName: "sm",
          selectorCertainty: "exact",
          reason: "typeUnion",
          abstractValue: { kind: "finiteSet", values: ["lg", "md", "sm"] },
        }),
        expect.objectContaining({
          canonicalName: "md",
          selectorCertainty: "exact",
          reason: "typeUnion",
        }),
        expect.objectContaining({
          canonicalName: "lg",
          selectorCertainty: "exact",
          reason: "typeUnion",
        }),
      ]),
    );
  });

  it("resolves style-access aliases back to the canonical selector", () => {
    const sourceScenario = loadSourceScenario({
      id: "02-style-access",
      sourcePath: "02-multi-binding/StyleAccessDemo.tsx",
    });
    const styleScenario = loadStyleScenario({
      id: "02-button-camel-case",
      stylePath: "02-multi-binding/Button.module.scss",
      mode: "camelCase",
    });

    const graph = buildSourceSemanticGraph({
      sourceDocument: sourceScenario.sourceDocument,
      styleDocumentsByPath: new Map([[styleScenario.filePath, styleScenario.styleDocument]]),
    });
    const index = buildSemanticReferenceIndex(graph);

    expect(index.findSelectorReferences(styleScenario.filePath, "button--primary")).toEqual([
      expect.objectContaining({
        refId: "class-expr:1",
        origin: "styleAccess",
        canonicalName: "button--primary",
        className: "buttonPrimary",
        selectorCertainty: "exact",
        reason: "styleAccess",
        expansion: "direct",
        abstractValue: { kind: "exact", value: "buttonPrimary" },
      }),
    ]);
  });

  it("collects inferred selector matches for template prefixes while keeping them expanded", () => {
    const sourceScenario = loadSourceScenario({
      id: "04-dynamic",
      sourcePath: "04-dynamic/DynamicScenario.tsx",
    });
    const styleScenario = loadStyleScenario({
      id: "04-dynamic-style",
      stylePath: "04-dynamic/DynamicKeys.module.scss",
    });

    const graph = buildSourceSemanticGraph({
      sourceDocument: sourceScenario.sourceDocument,
      styleDocumentsByPath: new Map([[styleScenario.filePath, styleScenario.styleDocument]]),
    });
    const index = buildSemanticReferenceIndex(graph);

    expect(index.findTargetsForRef("class-expr:0")).toEqual([
      expect.objectContaining({
        canonicalName: "btn-danger",
        selectorCertainty: "inferred",
        reason: "templatePrefix",
        abstractValue: { kind: "prefix", prefix: "btn-" },
      }),
      expect.objectContaining({
        canonicalName: "btn-primary",
        selectorCertainty: "inferred",
        reason: "templatePrefix",
      }),
      expect.objectContaining({
        canonicalName: "btn-secondary",
        selectorCertainty: "inferred",
        reason: "templatePrefix",
      }),
    ]);

    expect(index.findSelectorReferences(styleScenario.filePath, "btn-primary")).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          refId: "class-expr:0",
          selectorCertainty: "inferred",
          expansion: "expanded",
        }),
      ]),
    );

    expect(
      index.findAllForScssPath(styleScenario.filePath, {
        minimumSelectorCertainty: "inferred",
      }),
    ).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          refId: "class-expr:0",
          canonicalName: "btn-primary",
          selectorCertainty: "inferred",
        }),
      ]),
    );
  });

  it("indexes possible top-domain references across the whole selector universe", () => {
    const styleScenario = loadStyleScenario({
      id: "04-dynamic-style",
      stylePath: "04-dynamic/DynamicKeys.module.scss",
    });
    const sourceDocument = {
      kind: "source" as const,
      filePath: "/fake/ws/src/App.tsx",
      language: "tsx" as const,
      styleImports: [],
      utilityBindings: [],
      classExpressions: [
        {
          kind: "symbolRef" as const,
          id: "class-expr:0",
          origin: "cxCall" as const,
          rawReference: "key",
          rootName: "key",
          pathSegments: [],
          range: {
            start: { line: 1, character: 3 },
            end: { line: 1, character: 6 },
          },
          scssModulePath: styleScenario.filePath,
        },
      ],
      domainClassReferences: [],
    };

    const graph = buildSourceSemanticGraph({
      sourceDocument,
      styleDocumentsByPath: new Map([[styleScenario.filePath, styleScenario.styleDocument]]),
      resolveSymbolValues: () => ({
        abstractValue: { kind: "top" },
        valueCertainty: "possible",
        reason: "flowBranch",
      }),
    });
    const index = buildSemanticReferenceIndex(graph);

    expect(index.findTargetsForRef("class-expr:0")).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          canonicalName: "btn-primary",
          selectorCertainty: "possible",
          reason: "flowBranch",
          abstractValue: { kind: "top" },
        }),
        expect.objectContaining({
          canonicalName: "btn-secondary",
          selectorCertainty: "possible",
          reason: "flowBranch",
          abstractValue: { kind: "top" },
        }),
      ]),
    );
  });
});

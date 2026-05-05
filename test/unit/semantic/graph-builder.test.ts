import { describe, expect, it } from "vitest";
import {
  buildSourceSemanticGraph,
  buildStyleSemanticGraph,
} from "../../_support/semantic-graph-fixture";
import type {
  SemanticEdge,
  SemanticGraph,
  SemanticNode,
} from "../../_support/semantic-graph-types";
import { loadSourceScenario, loadStyleScenario } from "../../_fixtures/scenario-corpus";
import { buildSourceDocumentFixture } from "../../_fixtures/source-documents";

describe("buildStyleSemanticGraph", () => {
  it("emits canonical selector nodes plus alias-view edges for camelCase style docs", () => {
    const styleScenario = loadStyleScenario({
      id: "02-button-camel-case",
      stylePath: "02-multi-binding/Button.module.scss",
      mode: "camelCase",
    });

    const graph = buildStyleSemanticGraph(styleScenario.styleDocument);
    const normalized = normalizeGraph(graph);

    expect(normalized.nodes).toEqual(
      expect.arrayContaining([
        {
          kind: "selector",
          canonicalName: "button--primary",
          filePath: styleScenario.filePath,
          id: `selector:${styleScenario.filePath}:button--primary`,
        },
        {
          kind: "selectorView",
          name: "buttonPrimary",
          canonicalName: "button--primary",
          viewKind: "alias",
          filePath: styleScenario.filePath,
          id: `selector-view:${styleScenario.filePath}:buttonPrimary`,
          nestedSafety: "bemSuffixSafe",
          originalName: "button--primary",
        },
      ]),
    );
    expect(normalized.edges).toEqual(
      expect.arrayContaining([
        {
          reason: "aliasCanonicalization",
          certainty: "exact",
          from: `selector-view:${styleScenario.filePath}:buttonPrimary`,
          to: `selector:${styleScenario.filePath}:button--primary`,
        },
      ]),
    );
  });
});

describe("buildSourceSemanticGraph", () => {
  it("emits exact literal edges and inferred type-union edges for the basic scenario", () => {
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
              values: ["sm", "md", "lg"],
              certainty: "inferred",
              reason: "typeUnion",
            }
          : null,
    });

    const normalized = normalizeGraph(graph);
    expect(normalized.edges).toEqual(
      expect.arrayContaining([
        {
          reason: "literal",
          certainty: "exact",
          abstractValue: { kind: "exact", value: "button" },
          from: "class-expr:0",
          to: `selector:${styleScenario.filePath}:button`,
        },
        {
          reason: "typeUnion",
          certainty: "exact",
          abstractValue: { kind: "finiteSet", values: ["lg", "md", "sm"] },
          from: "class-expr:2",
          to: `selector:${styleScenario.filePath}:sm`,
        },
        {
          reason: "typeUnion",
          certainty: "exact",
          abstractValue: { kind: "finiteSet", values: ["lg", "md", "sm"] },
          from: "class-expr:2",
          to: `selector:${styleScenario.filePath}:md`,
        },
        {
          reason: "typeUnion",
          certainty: "exact",
          abstractValue: { kind: "finiteSet", values: ["lg", "md", "sm"] },
          from: "class-expr:2",
          to: `selector:${styleScenario.filePath}:lg`,
        },
      ]),
    );
  });

  it("emits inferred template-prefix edges for the dynamic scenario", () => {
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

    expect(normalizeGraph(graph)).toMatchObject({
      edges: expect.arrayContaining([
        {
          reason: "templatePrefix",
          certainty: "inferred",
          abstractValue: { kind: "prefix", prefix: "btn-" },
          from: "class-expr:0",
          to: `selector:${styleScenario.filePath}:btn-primary`,
        },
        {
          reason: "templatePrefix",
          certainty: "inferred",
          abstractValue: { kind: "prefix", prefix: "btn-" },
          from: "class-expr:0",
          to: `selector:${styleScenario.filePath}:btn-secondary`,
        },
        {
          reason: "templatePrefix",
          certainty: "inferred",
          abstractValue: { kind: "prefix", prefix: "btn-" },
          from: "class-expr:0",
          to: `selector:${styleScenario.filePath}:btn-danger`,
        },
      ]),
    });
  });

  it("emits possible edges for top-valued symbol refs across the full selector universe", () => {
    const styleScenario = loadStyleScenario({
      id: "04-dynamic-style",
      stylePath: "04-dynamic/DynamicKeys.module.scss",
    });
    const sourceDocument = buildSourceDocumentFixture({
      filePath: "/fake/ws/src/App.tsx",
      bindings: [],
      stylesBindings: new Map(),
      classUtilNames: [],
      expressions: [
        {
          kind: "symbolRef",
          origin: "cxCall",
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
    });

    const graph = buildSourceSemanticGraph({
      sourceDocument,
      styleDocumentsByPath: new Map([[styleScenario.filePath, styleScenario.styleDocument]]),
      resolveSymbolValues: () => ({
        abstractValue: { kind: "top" },
        values: [],
        certainty: "possible",
        reason: "flowBranch",
      }),
    });

    expect(normalizeGraph(graph)).toMatchObject({
      edges: expect.arrayContaining([
        {
          reason: "flowBranch",
          certainty: "possible",
          abstractValue: { kind: "top" },
          from: "class-expr:0",
          to: `selector:${styleScenario.filePath}:btn-primary`,
        },
        {
          reason: "flowBranch",
          certainty: "possible",
          abstractValue: { kind: "top" },
          from: "class-expr:0",
          to: `selector:${styleScenario.filePath}:btn-secondary`,
        },
        {
          reason: "flowBranch",
          certainty: "possible",
          abstractValue: { kind: "top" },
          from: "class-expr:0",
          to: `selector:${styleScenario.filePath}:btn-danger`,
        },
      ]),
    });
  });
});

function normalizeGraph(graph: SemanticGraph): {
  readonly nodes: readonly unknown[];
  readonly edges: readonly unknown[];
} {
  return {
    nodes: graph.nodes.map(normalizeNode),
    edges: graph.edges.map(normalizeEdge),
  };
}

function normalizeNode(node: SemanticNode): unknown {
  switch (node.kind) {
    case "document":
      return {
        id: node.id,
        kind: node.kind,
        documentKind: node.documentKind,
        filePath: node.filePath,
      };
    case "styleImport":
      return {
        id: node.id,
        kind: node.kind,
        localName: node.localName,
        resolved: node.resolved.kind,
      };
    case "utilityBinding":
      return {
        id: node.id,
        kind: node.kind,
        bindingKind: node.bindingKind,
        localName: node.localName,
        ...(node.scssModulePath ? { scssModulePath: node.scssModulePath } : {}),
        ...(node.stylesLocalName ? { stylesLocalName: node.stylesLocalName } : {}),
      };
    case "ref":
      return {
        id: node.id,
        kind: node.kind,
        expressionKind: node.expressionKind,
        origin: node.origin,
        scssModulePath: node.scssModulePath,
        ...(node.className ? { className: node.className } : {}),
        ...(node.staticPrefix ? { staticPrefix: node.staticPrefix } : {}),
        ...(node.rawReference ? { rawReference: node.rawReference } : {}),
      };
    case "selector":
      return {
        id: node.id,
        kind: node.kind,
        filePath: node.filePath,
        canonicalName: node.canonicalName,
      };
    case "selectorView":
      return {
        id: node.id,
        kind: node.kind,
        name: node.name,
        canonicalName: node.canonicalName,
        viewKind: node.viewKind,
        nestedSafety: node.nestedSafety,
        filePath: node.filePath,
        ...(node.originalName ? { originalName: node.originalName } : {}),
      };
    default:
      node satisfies never;
      return node;
  }
}

function normalizeEdge(edge: SemanticEdge): unknown {
  return {
    from: edge.from,
    to: edge.to,
    reason: edge.reason,
    certainty: edge.certainty,
    ...(edge.abstractValue ? { abstractValue: edge.abstractValue } : {}),
  };
}

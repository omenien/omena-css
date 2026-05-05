import { describe, expect, it } from "vitest";
import type { ClassExpressionHIR } from "../../../server/engine-core-ts/src/core/hir/source-types";
import type { SelectorDeclHIR } from "../../../server/engine-core-ts/src/core/hir/style-types";
import type { SelectorUsageSummary } from "../../../server/engine-core-ts/src/core/query/read-selector-usage";
import type { SelectorStyleDependencySummary } from "../../../server/engine-core-ts/src/core/query/read-selector-style-dependencies";
import {
  renderCustomPropertyHover,
  renderHover,
  renderSelectorHover,
} from "../../../server/lsp-server/src/providers/hover-renderer";
import { workspace, type Range } from "../../../packages/vitest-cme/src";

const SCSS_PATH = "/fake/ws/src/Button.module.scss";
const SOURCE_PATH = "/fake/ws/src/Button.tsx";
const SELECTOR_COLUMN = 2;
const RULE_START_COLUMN = 0;
const RULE_END_COLUMN = 1;

function sourceRange(content: string, markerName: string): Range {
  return workspace({ [SOURCE_PATH]: content }).range(markerName, SOURCE_PATH).range;
}

const staticExpression: ClassExpressionHIR = {
  kind: "literal",
  id: "expr:literal",
  className: "indicator",
  range: sourceRange("const el = cx('/*<class>*/indicator/*</class>*/');", "class"),
  scssModulePath: SCSS_PATH,
  origin: "cxCall",
};

const templateExpression: ClassExpressionHIR = {
  kind: "template",
  id: "expr:template",
  rawTemplate: "btn-${variant}",
  staticPrefix: "btn-",
  range: sourceRange("const el = cx(/*<template>*/`btn-${variant}`/*</template>*/);", "template"),
  scssModulePath: SCSS_PATH,
  origin: "cxCall",
};

const symbolExpression: ClassExpressionHIR = {
  kind: "symbolRef",
  id: "expr:symbol",
  rawReference: "size",
  rootName: "size",
  pathSegments: [],
  range: sourceRange("const el = cx(/*<symbol>*/size/*</symbol>*/);", "symbol"),
  scssModulePath: SCSS_PATH,
  origin: "cxCall",
};

function selector(name: string, line: number, declarations: string): SelectorDeclHIR {
  const range: Range = {
    start: { line, character: SELECTOR_COLUMN },
    end: { line, character: SELECTOR_COLUMN + name.length },
  };
  const ruleRange: Range = {
    start: { line, character: RULE_START_COLUMN },
    end: { line: line + 3, character: RULE_END_COLUMN },
  };
  return {
    kind: "selector",
    id: `selector:${name}:${line}`,
    name,
    canonicalName: name,
    viewKind: "canonical",
    range,
    fullSelector: `.${name}`,
    declarations,
    ruleRange,
    composes: [],
    nestedSafety: "flat",
  };
}

function usageSummary(overrides: Partial<SelectorUsageSummary> = {}): SelectorUsageSummary {
  return {
    allSites: [],
    directSites: [],
    editableDirectSites: [],
    exactSites: [],
    inferredOrBetterSites: [],
    totalReferences: 0,
    directReferenceCount: 0,
    hasExpandedReferences: false,
    hasStyleDependencyReferences: false,
    hasAnyReferences: false,
    ...overrides,
  };
}

describe("renderHover", () => {
  it("returns null when no selectors match", () => {
    expect(
      renderHover({
        expression: staticExpression,
        scssModulePath: SCSS_PATH,
        selectors: [],
        workspaceRoot: "/fake/ws",
      }),
    ).toBeNull();
  });

  it("renders a single-match card with workspace-relative location", () => {
    const markdown = renderHover({
      expression: staticExpression,
      scssModulePath: SCSS_PATH,
      selectors: [selector("indicator", 11, "color: red; font-size: 14px")],
      workspaceRoot: "/fake/ws",
    });
    expect(markdown).toContain("**`.indicator`**");
    expect(markdown).toContain("src/Button.module.scss:12");
    expect(markdown).toContain("```scss");
    expect(markdown).toContain("  color: red;");
    expect(markdown).toContain("  font-size: 14px;");
  });

  it("renders a multi-match template card", () => {
    const markdown = renderHover({
      expression: templateExpression,
      scssModulePath: SCSS_PATH,
      selectors: [
        selector("btn-primary", 10, "color: white"),
        selector("btn-secondary", 14, "color: gray"),
      ],
      workspaceRoot: "/fake/ws",
    });
    expect(markdown).toContain("**2 matches** for `cx(");
    expect(markdown).toContain("btn-${...}");
    expect(markdown).toContain(".btn-primary");
    expect(markdown).toContain(".btn-secondary");
    expect(markdown).toContain("---"); // section separator
  });

  it("renders a multi-match variable card", () => {
    const markdown = renderHover({
      expression: symbolExpression,
      scssModulePath: SCSS_PATH,
      selectors: [
        selector("small", 10, "font-size: 12px"),
        selector("medium", 14, "font-size: 16px"),
      ],
      workspaceRoot: "/fake/ws",
    });
    expect(markdown).toContain("**2 matches** for `cx(size)`");
  });

  it("renders abstract value summaries for dynamic explanations", () => {
    const markdown = renderHover({
      expression: symbolExpression,
      scssModulePath: SCSS_PATH,
      selectors: [
        selector("small", 10, "font-size: 12px"),
        selector("medium", 14, "font-size: 16px"),
      ],
      workspaceRoot: "/fake/ws",
      dynamicExplanation: {
        kind: "symbolRef",
        subject: "size",
        candidates: ["small", "medium"],
        valueDomainLabel: "finite set (2)",
        valueDomainReasonLabel: "finite candidates widened to a shared prefix",
        valueCertainty: "exact",
        valueDomainProvenanceLabel:
          "finiteSetWidening: composite/finiteSetWideningComposite (large finite set widened to preserved edge and character constraints)",
        valueDomainProvenanceStepLabels: [
          "1. prefixConstraint: prefix=btn- (constraint retained by the abstract value domain)",
          "2. suffixConstraint: suffix=-active (constraint retained by the abstract value domain)",
        ],
        valueCertaintyShapeLabel: "exact",
        selectorCertainty: "inferred",
        selectorCertaintyShapeLabel: "bounded selector set (2)",
        selectorCertaintyReasonLabel: "finite candidate values matched a bounded selector set",
        reasonLabel: "TypeScript string-literal union analysis",
      },
    });
    expect(markdown).toContain("Value domain: finite set (2).");
    expect(markdown).toContain(
      "Value domain reason: finite candidates widened to a shared prefix.",
    );
    expect(markdown).toContain("Value domain provenance: finiteSetWidening");
    expect(markdown).toContain("Value domain constraints: 1. prefixConstraint");
    expect(markdown).toContain("Value certainty: exact.");
    expect(markdown).toContain("Value certainty shape: exact.");
    expect(markdown).toContain("Selector certainty: inferred.");
    expect(markdown).toContain("Selector certainty shape: bounded selector set (2).");
    expect(markdown).toContain(
      "Selector certainty reason: finite candidate values matched a bounded selector set.",
    );
  });

  it("renders value certainty reasons when present", () => {
    const markdown = renderHover({
      expression: symbolExpression,
      scssModulePath: SCSS_PATH,
      selectors: [selector("active", 10, "font-size: 12px")],
      workspaceRoot: "/fake/ws",
      dynamicExplanation: {
        kind: "symbolRef",
        subject: "size",
        candidates: ["active", "indicator"],
        valueCertainty: "inferred",
        valueCertaintyShapeLabel: "bounded finite (2)",
        valueCertaintyReasonLabel: "TypeScript exposed multiple string-literal candidates",
        selectorCertainty: "inferred",
        selectorCertaintyShapeLabel: "bounded selector set (1)",
        selectorCertaintyReasonLabel: "finite candidate values matched a bounded selector set",
        reasonLabel: "TypeScript string-literal union analysis",
      },
    });
    expect(markdown).toContain("Value certainty shape: bounded finite (2).");
    expect(markdown).toContain(
      "Value certainty reason: TypeScript exposed multiple string-literal candidates.",
    );
    expect(markdown).toContain("Selector certainty shape: bounded selector set (1).");
    expect(markdown).toContain(
      "Selector certainty reason: finite candidate values matched a bounded selector set.",
    );
  });

  it("caps multi-match at MAX_CANDIDATES=10 with a tail summary", () => {
    const many = Array.from({ length: 15 }, (_, i) => selector(`item-${i}`, i + 1, "color: red"));
    const markdown = renderHover({
      expression: templateExpression,
      scssModulePath: SCSS_PATH,
      selectors: many,
      workspaceRoot: "/fake/ws",
    });
    expect(markdown).toContain("**15 matches**");
    expect(markdown).toContain("…and 5 more");
    // First 10 are shown, 11th (item-10) should NOT appear in full
    expect(markdown).toContain(".item-0");
    expect(markdown).toContain(".item-9");
    expect(markdown).not.toContain(".item-10 {");
  });

  it("falls back to the raw scss path when workspaceRoot equals scssModulePath", () => {
    const markdown = renderHover({
      expression: staticExpression,
      scssModulePath: "/same",
      selectors: [selector("only", 3, "color: red")],
      workspaceRoot: "/same",
    });
    // relative("/same", "/same") === "" → fallback to raw path
    expect(markdown).toContain("/same:4");
  });

  it("handles an empty declarations string with empty braces", () => {
    const markdown = renderHover({
      expression: staticExpression,
      scssModulePath: SCSS_PATH,
      selectors: [selector("empty", 5, "")],
      workspaceRoot: "/fake/ws",
    });
    expect(markdown).toContain(".empty {}");
  });

  it("renders incoming composes dependencies", () => {
    const dependencies = new Map<string, SelectorStyleDependencySummary>([
      [
        "indicator",
        {
          incoming: [
            {
              canonicalName: "card",
              filePath: "/fake/ws/src/Card.module.scss",
              reason: "crossFileComposes",
            },
          ],
          outgoing: [],
        },
      ],
    ]);

    const markdown = renderHover({
      expression: staticExpression,
      scssModulePath: SCSS_PATH,
      selectors: [selector("indicator", 11, "color: red;")],
      styleDependenciesBySelector: dependencies,
      workspaceRoot: "/fake/ws",
    });

    expect(markdown).toContain("Composed by:");
    expect(markdown).toContain("`card` in `src/Card.module.scss`");
  });

  it("renders selector hover usage and dependency context", () => {
    const markdown = renderSelectorHover({
      selector: selector("indicator", 11, "color: red;"),
      scssModulePath: SCSS_PATH,
      usageSummary: usageSummary({
        totalReferences: 3,
        directReferenceCount: 1,
        hasExpandedReferences: true,
        hasStyleDependencyReferences: true,
        hasAnyReferences: true,
      }),
      styleDependencies: {
        incoming: [
          {
            canonicalName: "card",
            filePath: "/fake/ws/src/Card.module.scss",
            reason: "crossFileComposes",
          },
        ],
        outgoing: [
          {
            canonicalName: "base",
            filePath: "/fake/ws/src/Base.module.scss",
            reason: "crossFileComposes",
          },
        ],
      },
      workspaceRoot: "/fake/ws",
    });

    expect(markdown).toContain("References: 3 total (1 direct).");
    expect(markdown).toContain("Expanded/dynamic and composed-style references present.");
    expect(markdown).toContain("Composed by:");
    expect(markdown).toContain("Composes:");
    expect(markdown).toContain("`card` in `src/Card.module.scss`");
    expect(markdown).toContain("`base` in `src/Base.module.scss`");
  });

  it("renders design token cascade ranking context for custom properties", () => {
    const markdown = renderCustomPropertyHover({
      customPropertyDecl: {
        name: "--brand",
        value: "green",
        range: {
          start: { line: 2, character: 10 },
          end: { line: 2, character: 17 },
        },
      },
      scssModulePath: SCSS_PATH,
      referenceCount: 1,
      workspaceRoot: "/fake/ws",
      designTokenRanking: {
        shadowedDeclarationSourceOrders: [0, 1],
        shadowedDeclarations: [
          {
            range: {
              start: { line: 0, character: 8 },
              end: { line: 0, character: 15 },
            },
          },
          {
            range: {
              start: { line: 1, character: 9 },
              end: { line: 1, character: 16 },
            },
          },
        ],
      },
    });

    expect(markdown).toContain(
      "Cascade ranking: source-order winner; shadows 2 earlier same-file declarations.",
    );
  });

  it("renders workspace design token cascade candidate context for custom properties", () => {
    const markdown = renderCustomPropertyHover({
      customPropertyDecl: {
        name: "--brand",
        value: "red",
        range: {
          start: { line: 2, character: 10 },
          end: { line: 2, character: 17 },
        },
      },
      scssModulePath: SCSS_PATH,
      referenceCount: 1,
      workspaceRoot: "/fake/ws",
      designTokenRanking: {
        winnerDeclarationFilePath: "/fake/ws/src/tokens.scss",
        shadowedDeclarationSourceOrders: [],
        crossFileCandidateDeclarationCount: 1,
        crossFileShadowedDeclarationCount: 0,
      },
    });

    expect(markdown).toContain(
      "Cascade ranking: workspace candidate winner from `src/tokens.scss`; 1 cross-file candidate observed.",
    );
  });

  it("renders import graph design token cascade candidate context for custom properties", () => {
    const markdown = renderCustomPropertyHover({
      customPropertyDecl: {
        name: "--brand",
        value: "red",
        range: {
          start: { line: 2, character: 10 },
          end: { line: 2, character: 17 },
        },
      },
      scssModulePath: SCSS_PATH,
      referenceCount: 1,
      workspaceRoot: "/fake/ws",
      designTokenRanking: {
        winnerDeclarationFilePath: "/fake/ws/src/tokens.scss",
        crossFileCandidateScope: "cross-file-import-candidate",
        shadowedDeclarationSourceOrders: [],
        crossFileCandidateDeclarationCount: 1,
        crossFileShadowedDeclarationCount: 0,
      },
    });

    expect(markdown).toContain(
      "Cascade ranking: import graph candidate winner from `src/tokens.scss`; 1 cross-file candidate observed.",
    );
  });
});

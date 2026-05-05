import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { explainExpressionAtLocation } from "../../../server/engine-host-node/src/explain-expression";

const tempDirs: string[] = [];

afterEach(() => {
  for (const dir of tempDirs.splice(0)) {
    rmSync(dir, { recursive: true, force: true });
  }
});

describe("explainExpressionAtLocation", () => {
  it("returns dynamic explanation details for a local flow expression", () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "export function App(enabled: boolean) {",
        "  const size = enabled ? 'small' : 'large';",
        "  return <div className={cx(size)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".small {}",
    });

    const result = explainExpressionAtLocation({
      workspaceRoot,
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      line: 5,
      character: 28,
    });

    expect(result).toEqual(
      expect.objectContaining({
        expressionKind: "symbolRef",
        styleFilePath: path.join(workspaceRoot, "src/Button.module.scss"),
        selectorNames: ["small"],
        analysisV2: expect.objectContaining({
          valueDomainKind: "finiteSet",
          valueCertaintyShapeKind: "boundedFinite",
          selectorCertaintyShapeKind: "boundedFinite",
        }),
        dynamicExplanation: expect.objectContaining({
          subject: "size",
          valueCertainty: "inferred",
          valueCertaintyShapeLabel: "bounded finite (2)",
          valueCertaintyReasonLabel: "analysis preserved multiple finite candidate values",
          selectorCertainty: "inferred",
          selectorCertaintyShapeLabel: "bounded selector set (1)",
        }),
      }),
    );
  });

  it("surfaces bundle-1 constrained metadata for prefix-suffix expressions", () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        'import classNames from "classnames/bind";',
        'import styles from "./Button.module.scss";',
        "const cx = classNames.bind(styles);",
        "export function App(variant: string) {",
        '  const className = "btn-" + variant + "-chip";',
        "  return <div className={cx(className)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": [
        ".btn-idle-chip {}",
        ".btn-busy-chip {}",
        ".btn-error-chip {}",
      ].join("\n"),
    });

    const result = explainExpressionAtLocation({
      workspaceRoot,
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      line: 5,
      character: 28,
    });

    expect(result).toEqual(
      expect.objectContaining({
        selectorNames: ["btn-idle-chip", "btn-busy-chip", "btn-error-chip"],
        analysisV2: expect.objectContaining({
          valueDomainKind: "constrained",
          valueConstraintKind: "prefixSuffix",
          valueCertaintyShapeKind: "constrained",
          valueCertaintyConstraintKind: "prefixSuffix",
          selectorCertaintyShapeKind: "constrained",
        }),
      }),
    );
  });

  it("surfaces bundle-2 char-inclusion metadata", () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        'import classNames from "classnames/bind";',
        'import styles from "./Button.module.scss";',
        "const cx = classNames.bind(styles);",
        "function resolveState(value: number) {",
        "  switch (value) {",
        '    case 1: return "stateOne";',
        '    case 2: return "stateTwo";',
        '    case 3: return "stateThree";',
        '    case 4: return "stateFour";',
        '    case 5: return "stateFive";',
        '    case 6: return "stateSix";',
        '    case 7: return "stateSeven";',
        '    case 8: return "stateEight";',
        '    default: return "stateNine";',
        "  }",
        "}",
        "export function App(value: number) {",
        "  const state = resolveState(value);",
        "  return <div className={cx(state)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".stateOne {}",
    });

    const result = explainExpressionAtLocation({
      workspaceRoot,
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      line: 18,
      character: 28,
    });

    expect(result).toEqual(
      expect.objectContaining({
        analysisV2: expect.objectContaining({
          valueDomainKind: "constrained",
          valueConstraintKind: "charInclusion",
          valueCharMust: "aest",
          valueCharMay: "EFNOSTaeghinorstuvwx",
          valueCertaintyShapeKind: "constrained",
          valueCertaintyConstraintKind: "charInclusion",
        }),
      }),
    );
  });

  it("surfaces bundle-3 composite metadata", () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        'import classNames from "classnames/bind";',
        'import styles from "./Button.module.scss";',
        "const cx = classNames.bind(styles);",
        "function resolveVariant(value: number) {",
        "  switch (value) {",
        '    case 1: return "btn-primary";',
        '    case 2: return "btn-secondary";',
        '    case 3: return "btn-danger";',
        '    case 4: return "btn-success";',
        '    case 5: return "btn-warning";',
        '    case 6: return "btn-info";',
        '    case 7: return "btn-muted";',
        '    case 8: return "btn-ghost";',
        '    default: return "btn-outline";',
        "  }",
        "}",
        "export function App(value: number) {",
        "  const variant = resolveVariant(value);",
        "  return <div className={cx(variant)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".btn-primary {}",
    });

    const result = explainExpressionAtLocation({
      workspaceRoot,
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      line: 18,
      character: 28,
    });

    expect(result).toEqual(
      expect.objectContaining({
        analysisV2: expect.objectContaining({
          valueDomainKind: "constrained",
          valueConstraintKind: "composite",
          valuePrefix: "btn-",
          valueMinLen: 8,
          valueCharMust: "-bnt",
          valueCharMay: "-abcdefghilmnoprstuwy",
          valueCertaintyShapeKind: "constrained",
          valueCertaintyConstraintKind: "composite",
          selectorCertaintyShapeKind: "constrained",
          valueDomainProvenanceTree: expect.objectContaining({
            root: expect.objectContaining({
              operation: "finiteSetWidening",
              resultProvenance: "finiteSetWideningComposite",
              children: expect.arrayContaining([
                expect.objectContaining({
                  operation: "prefixConstraint",
                  detail: "prefix=btn-",
                }),
                expect.objectContaining({
                  operation: "characterMustConstraint",
                  detail: "mustChars=-bnt",
                }),
              ]),
            }),
          }),
        }),
      }),
    );
  });

  it("returns null when the cursor does not hit an explainable class expression", () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "const ok = cx('small');",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".small {}",
    });

    const result = explainExpressionAtLocation({
      workspaceRoot,
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      line: 0,
      character: 0,
    });

    expect(result).toBeNull();
  });

  it("can source expression semantics from the rust selected-query backend", () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "export function App(enabled: boolean) {",
        "  const size = enabled ? 'small' : 'large';",
        "  return <div className={cx(size)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".small {}\n.large {}",
    });

    const result = explainExpressionAtLocation({
      workspaceRoot,
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      line: 5,
      character: 28,
      env: {
        CME_SELECTED_QUERY_BACKEND: "rust-expression-semantics",
      } as NodeJS.ProcessEnv,
      readRustExpressionSemanticsPayload: () => ({
        expressionId: "expr-1",
        expressionKind: "symbolRef",
        styleFilePath: path.join(workspaceRoot, "src/Button.module.scss"),
        selectorNames: ["small"],
        candidateNames: ["small", "large"],
        finiteValues: ["small", "large"],
        valueDomainKind: "finiteSet",
        selectorCertainty: "inferred",
        valueCertainty: "inferred",
        selectorCertaintyShapeKind: "boundedFinite",
        selectorCertaintyShapeLabel: "bounded selector set (1)",
        valueCertaintyShapeKind: "boundedFinite",
        valueCertaintyShapeLabel: "bounded finite (2)",
        valueDomainDerivation: sampleValueDomainDerivation(),
      }),
    });

    expect(result).toEqual(
      expect.objectContaining({
        selectorNames: ["small"],
        analysisV2: expect.objectContaining({
          valueDomainKind: "finiteSet",
          valueCertaintyShapeKind: "boundedFinite",
          selectorCertaintyShapeKind: "boundedFinite",
          valueDomainDerivation: expect.objectContaining({
            reducedKind: "finiteSet",
          }),
        }),
        dynamicExplanation: expect.objectContaining({
          subject: "size",
          candidates: ["small", "large"],
          valueDomainDerivationLabel: "finiteSet reduced to finiteSet via baseFromFacts",
          valueCertainty: "inferred",
          valueCertaintyShapeLabel: "bounded finite (2)",
          valueCertaintyReasonLabel: "analysis preserved multiple finite candidate values",
          selectorCertainty: "inferred",
          selectorCertaintyShapeLabel: "bounded selector set (1)",
        }),
      }),
    );
  });

  it("applies selected-query selector projections to explain expression results", () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        'import classNames from "classnames/bind";',
        'import styles from "./Button.module.scss";',
        "const cx = classNames.bind(styles);",
        "export function App(variant: string) {",
        '  const className = "btn-" + variant;',
        "  return <div className={cx(className)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".btn-primary {}",
    });
    let expressionId = "";
    let projectionReads = 0;

    const result = explainExpressionAtLocation({
      workspaceRoot,
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      line: 5,
      character: 28,
      env: {
        CME_SELECTED_QUERY_BACKEND: "rust-selected-query",
      } as NodeJS.ProcessEnv,
      readRustExpressionSemanticsPayload: (_document, id, scssModulePath) => {
        expressionId = id;
        return {
          expressionId: id,
          expressionKind: "symbolRef",
          styleFilePath: scssModulePath,
          selectorNames: [],
          candidateNames: [],
          finiteValues: [],
          valueDomainKind: "none",
          selectorCertainty: "possible",
          valueCertainty: "possible",
          selectorCertaintyShapeKind: "unknown",
          selectorCertaintyShapeLabel: "unknown",
          valueCertaintyShapeKind: "unknown",
          valueCertaintyShapeLabel: "unknown",
        };
      },
      readRustExpressionDomainSelectorProjections: (_document, scssModulePath) => {
        projectionReads += 1;
        return [
          {
            graphId: `${workspaceRoot}/src/App.tsx:expression-domain-flow`,
            filePath: path.join(workspaceRoot, "src/App.tsx"),
            nodeId: expressionId,
            targetStylePaths: [scssModulePath],
            valueKind: "composite",
            selectorNames: ["btn-primary"],
            certainty: "inferred",
          },
        ];
      },
    });

    expect(projectionReads).toBe(1);
    expect(result).toEqual(
      expect.objectContaining({
        selectorNames: ["btn-primary"],
      }),
    );
  });

  it("falls back to TypeScript semantics when rust payload is non-informative", () => {
    const workspaceRoot = makeWorkspace({
      "src/App.tsx": [
        "import classNames from 'classnames/bind';",
        "import styles from './Button.module.scss';",
        "const cx = classNames.bind(styles);",
        "export function App(enabled: boolean) {",
        "  const size = enabled ? 'small' : 'large';",
        "  return <div className={cx(size)} />;",
        "}",
        "",
      ].join("\n"),
      "src/Button.module.scss": ".small {}",
    });

    const result = explainExpressionAtLocation({
      workspaceRoot,
      filePath: path.join(workspaceRoot, "src/App.tsx"),
      line: 5,
      character: 28,
      env: {
        CME_SELECTED_QUERY_BACKEND: "rust-selected-query",
      } as NodeJS.ProcessEnv,
      readRustExpressionSemanticsPayload: () => ({
        expressionId: "expr-1",
        expressionKind: "symbolRef",
        styleFilePath: path.join(workspaceRoot, "src/Button.module.scss"),
        selectorNames: [],
        candidateNames: [],
        finiteValues: [],
        valueDomainKind: "none",
        selectorCertainty: "possible",
        valueCertainty: "possible",
        selectorCertaintyShapeKind: "unknown",
        selectorCertaintyShapeLabel: "unknown",
        valueCertaintyShapeKind: "unknown",
        valueCertaintyShapeLabel: "unknown",
      }),
      readRustExpressionDomainSelectorProjections: () => [],
    });

    expect(result).toEqual(
      expect.objectContaining({
        selectorNames: ["small"],
        analysisV2: expect.objectContaining({
          valueDomainKind: "finiteSet",
          valueCertaintyShapeKind: "boundedFinite",
        }),
        dynamicExplanation: expect.objectContaining({
          subject: "size",
          candidates: expect.arrayContaining(["small", "large"]),
        }),
      }),
    );
  });
});

function makeWorkspace(files: Record<string, string>): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "explain-expression-"));
  tempDirs.push(root);
  for (const [relativePath, content] of Object.entries(files)) {
    const absolutePath = path.join(root, relativePath);
    mkdirSync(path.dirname(absolutePath), { recursive: true });
    writeFileSync(absolutePath, content, "utf8");
  }
  return root;
}

function sampleValueDomainDerivation() {
  return {
    schemaVersion: "0",
    product: "omena-abstract-value.reduced-class-value-derivation",
    inputFactKind: "finiteSet",
    inputValueCount: 2,
    reducedKind: "finiteSet",
    steps: [
      {
        operation: "baseFromFacts",
        resultKind: "finiteSet",
        reason: "mapped input facts to the base abstract value",
      },
    ],
  };
}

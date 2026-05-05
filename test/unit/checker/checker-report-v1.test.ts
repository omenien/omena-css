import { describe, expect, it } from "vitest";
import { buildCheckerReportV1 } from "../../../server/engine-core-ts/src/checker-surface/checker-report-v1";
import type { WorkspaceCheckerFinding } from "../../../server/engine-core-ts/src/core/checker/contracts";

describe("buildCheckerReportV1", () => {
  it("preserves value-domain derivation on source missing findings", () => {
    const valueDomainDerivation = {
      schemaVersion: "0",
      product: "omena-abstract-value.reduced-class-value-derivation",
      inputFactKind: "finiteSet",
      inputValueCount: 2,
      reducedKind: "finiteSet",
      steps: [
        {
          operation: "baseFromFacts",
          inputKind: "finiteSet",
          resultKind: "finiteSet",
          resultProvenance: "finiteSetWideningComposite",
          reason: "preserved finite string literal facts",
        },
      ],
    } as const;
    const findings: readonly WorkspaceCheckerFinding[] = [
      {
        filePath: "/fake/ws/src/App.tsx",
        finding: {
          category: "source",
          code: "missing-resolved-class-values",
          severity: "warning",
          range: { start: { line: 3, character: 10 }, end: { line: 3, character: 14 } },
          scssModulePath: "/fake/ws/src/Button.module.scss",
          missingValues: ["large"],
          abstractValue: { kind: "finiteSet", values: ["small", "large"] },
          valueCertainty: "inferred",
          selectorCertainty: "possible",
          reason: "flowBranch",
          valueDomainDerivation,
        },
      },
    ];

    const report = buildCheckerReportV1(findings, { warnings: 1, hints: 0, total: 1 }, "/fake/ws");

    expect(report.findings[0]).toMatchObject({
      code: "missing-resolved-class-values",
      valueDomainDerivationLabel: "finiteSet reduced to finiteSet via baseFromFacts",
      valueDomainDerivationStepLabels: [
        "1. baseFromFacts: finiteSet -> finiteSet/finiteSetWideningComposite (preserved finite string literal facts)",
      ],
      valueDomainDerivation,
    });
  });
});

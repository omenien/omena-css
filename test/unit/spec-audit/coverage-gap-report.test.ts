import { describe, expect, it } from "vitest";

import {
  buildCoverageGapReport,
  buildCoverageGapReportFromRepo,
  extractEngineFoldSurface,
  extractEngineRecognitionSurface,
  findCoverageGapRow,
  loadCoverageGapEngineSources,
  mathRecognitionResidue,
  serializeCoverageGapReport,
  type WebrefGrammarSnapshot,
} from "../../../scripts/coverage-gap-report";

const sources = loadCoverageGapEngineSources(process.cwd());
const recognition = extractEngineRecognitionSurface(sources);
const fold = extractEngineFoldSurface(sources);
const report = buildCoverageGapReportFromRepo(process.cwd());

describe("coverage gap report", () => {
  it("extracts recognition surfaces from parser source text", () => {
    expect(recognition.specializedArms).toEqual(["attr", "calc", "env", "if", "var"]);
    expect(Object.keys(recognition.valueNameTables).toSorted()).toEqual([
      "CSS_COLOR_FUNCTION_NAMES",
      "CSS_FILTER_FUNCTION_NAMES",
      "CSS_GRADIENT_FUNCTION_NAMES",
      "CSS_IMAGE_FUNCTION_NAMES",
      "CSS_SHAPE_FUNCTION_NAMES",
      "CSS_TRANSFORM_FUNCTION_NAMES",
      "VALUES_L4_MATH_FUNCTION_NAMES",
    ]);
    expect(recognition.functions).toEqual(expect.arrayContaining(["atan2", "if", "translate"]));
    expect(recognition.functions.every((name) => !name.endsWith("()"))).toBe(true);
    expect(recognition.atrules).toEqual(
      expect.arrayContaining(["@container", "@top-left", "@use"]),
    );
  });

  it("keeps fold surfaces separate from parser recognition", () => {
    expect(fold.cssFunctions).toEqual(
      expect.arrayContaining(["if", "calc", "translate", "rgb", "blur", "linear-gradient"]),
    );
    expect(fold.lessFunctions).toEqual(["acos", "asin", "atan", "cos", "sin", "tan"]);
    for (const lessOnly of fold.lessFunctions) {
      expect(fold.cssFunctions).not.toContain(lessOnly);
    }

    const mathResidue = mathRecognitionResidue(recognition, fold);
    expect(mathResidue).toEqual(
      expect.arrayContaining(["sin", "cos", "tan", "asin", "acos", "atan", "atan2"]),
    );
    expect(fold.cssFunctionsFromExplicitSurfaces.filter((name) => name !== "if")).toEqual(
      fold.cssFunctionsFromDomainSweep,
    );
  });

  it("computes recognition and fold gaps as separate advisory dimensions", () => {
    for (const foldedWitness of ["if", "translate", "rgb", "blur", "linear-gradient"]) {
      expect(findCoverageGapRow(report, "functions", foldedWitness)).toBeUndefined();
    }
    for (const residue of ["sin", "cos", "tan", "asin", "acos", "atan", "atan2"]) {
      expect(findCoverageGapRow(report, "functions", residue, "fold")?.tier).toBe("fold");
    }
    for (const contextualArm of ["var", "env", "attr"]) {
      expect(findCoverageGapRow(report, "functions", contextualArm, "fold")).toBeUndefined();
    }
    expect(report.policy.notDiffedCategories).toEqual(["properties", "selectors", "types"]);
    expect(report.policy.advisory).toBe(true);
  });

  it("makes folded witnesses reappear when their fold surface is removed", () => {
    for (const foldedWitness of ["if", "translate", "rgb", "blur", "linear-gradient"]) {
      const reducedFold = {
        ...fold,
        cssFunctions: fold.cssFunctions.filter((name) => name !== foldedWitness),
      };
      const reducedReport = buildCoverageGapReport({
        grammar: minimalGrammar([`${foldedWitness}()`], []),
        webFeaturesData: { features: {} },
        recognition,
        fold: reducedFold,
      });
      expect(findCoverageGapRow(reducedReport, "functions", foldedWitness, "fold")?.tier).toBe(
        "fold",
      );
    }
  });

  it("ranks synthetic recognition gaps from pinned baseline data", () => {
    const rankedReport = buildCoverageGapReport({
      grammar: minimalGrammar([], ["@limited-probe", "@widely-available-probe"]),
      webFeaturesData: {
        features: {
          "limited-probe": {
            name: "limited probe",
            compat_features: ["css.at-rules.limited-probe"],
            status: { baseline: false },
          },
          "widely-available-probe": {
            name: "widely available probe",
            compat_features: ["css.at-rules.widely-available-probe"],
            status: {
              baseline: "high",
              baseline_high_date: "2000-01-01",
              baseline_low_date: "1999-01-01",
            },
          },
        },
      },
      recognition,
      fold,
    });
    const firstRecognitionRow = rankedReport.rows.find((row) => row.tier === "recognition");
    expect(firstRecognitionRow?.name).toBe("@widely-available-probe");
    expect(firstRecognitionRow?.baseline.status).toBe("high");
  });

  it("serializes deterministically without timestamp-shaped fields", () => {
    const first = serializeCoverageGapReport(report);
    const second = serializeCoverageGapReport(buildCoverageGapReportFromRepo(process.cwd()));
    expect(second).toBe(first);
    expect(first).not.toMatch(/generatedAt|timestamp|last_changed|Date/u);

    const shuffledReport = buildCoverageGapReport({
      grammar: minimalGrammar(["sin()", "if()", "atan2()"].toReversed(), ["@apply", "@container"]),
      webFeaturesData: { features: {} },
      recognition,
      fold,
    });
    const unshuffledReport = buildCoverageGapReport({
      grammar: minimalGrammar(["atan2()", "if()", "sin()"], ["@container", "@apply"]),
      webFeaturesData: { features: {} },
      recognition,
      fold,
    });
    expect(serializeCoverageGapReport(shuffledReport)).toBe(
      serializeCoverageGapReport(unshuffledReport),
    );
  });
});

function minimalGrammar(
  functions: readonly string[],
  atrules: readonly string[],
): WebrefGrammarSnapshot {
  return {
    schemaVersion: "0",
    product: "omena-spec-audit.webref-grammar",
    source: { package: "@webref/css", version: "fixture", gitHead: "0".repeat(40) },
    generation: { tool: "fixture" },
    entryCount: functions.length + atrules.length,
    categories: {
      functions: functions.map((name) => ({ name, syntax: `${name} <value>` })),
      atrules: atrules.map((name) => ({ name, syntax: `${name} { <rule-list> }` })),
      properties: [],
      selectors: [],
      types: [],
    },
  };
}

import { describe, expect, it } from "vitest";

import {
  buildCoverageGapReport,
  buildCoverageGapReportFromRepo,
  extractEngineFoldSurface,
  extractEngineRecognitionSurface,
  extractSpecializedFunctionArms,
  findCoverageGapRows,
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
  it("recognizes specialized function arms through the shared case matcher", () => {
    expect(
      extractSpecializedFunctionArms(`
        fn specialized_function_kind(text: &str) -> Option<SyntaxKind> {
          if matches_ignore_ascii_case(text, &["if"]) {
            return Some(SyntaxKind::IfFunction);
          }
          None
        }
      `),
    ).toEqual(["if"]);
  });

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

  it("publishes every registry axis without turning implementation evidence into validation", () => {
    expect(report.summary.rowCount).toBe(1717);
    expect(report.summary.categoryCounts).toEqual({
      atrules: 56,
      functions: 162,
      properties: 815,
      selectors: 159,
      types: 525,
    });
    expect(report.summary.tierCounts.T2).toBe(0);
    expect(report.summary.tierCounts.T3).toBe(0);
    expect(report.summary.tierCounts.T4).toBe(0);
    expect(report.summary.categoryTierCounts.properties).toEqual({
      T0: 815,
      T1: 0,
      T2: 0,
      T3: 0,
      T4: 0,
    });
    expect(
      Object.values(report.summary.namedReasonCounts).reduce((total, count) => total + count, 0),
    ).toBe(1717);
    for (const foldedWitness of ["if", "translate", "rgb", "blur", "linear-gradient"]) {
      const rows = findCoverageGapRows(report, "functions", foldedWitness);
      expect(rows.length).toBeGreaterThan(0);
      expect(rows.every((row) => row.capabilityTier === "T1")).toBe(true);
      expect(rows.every((row) => row.measurements.staticallyReduced)).toBe(true);
    }
    for (const residue of ["sin", "cos", "tan", "asin", "acos", "atan", "atan2"]) {
      const rows = findCoverageGapRows(report, "functions", residue);
      expect(rows.length).toBeGreaterThan(0);
      expect(rows.every((row) => row.capabilityTier === "T1")).toBe(true);
      expect(rows.every((row) => !row.measurements.staticallyReduced)).toBe(true);
    }
    for (const contextualArm of ["var", "env", "attr"]) {
      const rows = findCoverageGapRows(report, "functions", contextualArm);
      expect(rows.every((row) => !row.measurements.staticallyReduced)).toBe(true);
    }
    expect(JSON.stringify(report)).not.toContain("notDiffedCategories");
    expect(report.policy.advisory).toBe(true);
  });

  it("records static reduction independently from the capability tier", () => {
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
      const rows = findCoverageGapRows(reducedReport, "functions", foldedWitness);
      expect(rows.every((row) => row.capabilityTier === "T1")).toBe(true);
      expect(rows.every((row) => !row.measurements.staticallyReduced)).toBe(true);
    }
  });

  it("ranks synthetic unrecognized rows from pinned baseline data", () => {
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
    const firstRecognitionRow = rankedReport.rows.find(
      (row) => row.category === "atrules" && row.name === "@widely-available-probe",
    );
    expect(firstRecognitionRow?.name).toBe("@widely-available-probe");
    expect(firstRecognitionRow?.baseline.status).toBe("high");
  });

  it("serializes deterministically without timestamp-shaped fields", async () => {
    const first = await serializeCoverageGapReport(report);
    const second = await serializeCoverageGapReport(buildCoverageGapReportFromRepo(process.cwd()));
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
    expect(await serializeCoverageGapReport(shuffledReport)).toBe(
      await serializeCoverageGapReport(unshuffledReport),
    );
  });

  it("rejects missing tiers and free-text reasons", () => {
    expect(() =>
      buildCoverageGapReportFromRepo(process.cwd(), { injectUntieredRow: true }),
    ).toThrow(/registered capability tier/u);
    expect(() =>
      buildCoverageGapReportFromRepo(process.cwd(), { injectFreeTextReason: true }),
    ).toThrow(/registered reason/u);
  });
});

function minimalGrammar(
  functions: readonly string[],
  atrules: readonly string[],
): WebrefGrammarSnapshot {
  return {
    schemaVersion: "1",
    product: "omena-spec-audit.webref-grammar",
    source: { package: "@webref/css", version: "fixture", gitHead: "0".repeat(40) },
    generation: { tool: "fixture" },
    entryCount: functions.length + atrules.length,
    categories: {
      functions: functions.map((name) =>
        fixtureGrammarRow(name, functions.toSorted().indexOf(name), `${name} <value>`),
      ),
      atrules: atrules.map((name) =>
        fixtureGrammarRow(name, atrules.toSorted().indexOf(name), `${name} { <rule-list> }`),
      ),
      properties: [],
      selectors: [],
      types: [],
    },
  };
}

function fixtureGrammarRow(name: string, sourceOrdinal: number, syntax: string) {
  return {
    name,
    href: `https://example.test/${sourceOrdinal}`,
    sourceOrdinal,
    syntax,
    boundary: { classification: "in-boundary" as const },
  };
}

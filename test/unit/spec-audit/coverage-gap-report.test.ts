import { readFileSync } from "node:fs";

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
  validateCoverageGapReport,
  type CoverageGapReport,
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
    expect(recognition.genericFunctions).toBe(true);
    expect(recognition.genericProperties).toBe(true);
    expect(recognition.selectorForms).toEqual([
      "combinator",
      "nesting",
      "pseudo-class",
      "pseudo-element",
    ]);
    expect(recognition.types).toEqual(expect.arrayContaining(["color", "number", "selector-list"]));
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

  it("publishes every registry axis and derives value tiers from matcher evidence", () => {
    expect(report.summary.rowCount).toBe(1715);
    expect(report.summary.categoryCounts).toEqual({
      atrules: 56,
      functions: 162,
      properties: 815,
      selectors: 158,
      types: 524,
    });
    expect(report.summary.tierCounts.T2).toBe(3);
    expect(report.summary.tierCounts.T3).toBe(0);
    expect(report.summary.tierCounts.T4).toBe(0);
    expect(report.summary.tierCounts.T1).toBe(1);
    expect(report.summary.categoryTierCounts.properties).toEqual({
      T0: 811,
      T1: 1,
      T2: 3,
      T3: 0,
      T4: 0,
    });
    expect(report.summary.categoryUnassignedCounts.properties).toBe(0);
    expect(report.summary.recognizedCounts).toEqual({
      atrules: 47,
      functions: 162,
      properties: 815,
      selectors: 158,
      types: expect.any(Number),
    });
    expect(report.summary.recognizedCounts.types).toBeGreaterThan(0);
    expect(report.summary.recognizedCounts.types).toBeLessThan(524);
    expect(findCoverageGapRows(report, "properties", "color")[0]?.capabilityTier).toBe("T1");
    for (const property of ["border-top", "font-family", "transform"]) {
      const [row] = findCoverageGapRows(report, "properties", property);
      expect(row?.capabilityTier).toBe("T2");
      expect(row?.measurements.typedProjectionEvidence).toBe(true);
      expect(row?.measurements.grammarValidationEvidence).toBe(true);
    }
    expect(
      Object.values(report.summary.namedReasonCounts).reduce((total, count) => total + count, 0),
    ).toBe(1715);
    for (const foldedWitness of ["if", "translate", "rgb", "blur", "linear-gradient"]) {
      const rows = findCoverageGapRows(report, "functions", foldedWitness);
      expect(rows.length).toBeGreaterThan(0);
      expect(rows.every((row) => row.capabilityTier === "T0")).toBe(true);
      expect(rows.every((row) => row.measurements.staticallyReduced)).toBe(true);
    }
    for (const residue of ["sin", "cos", "tan", "asin", "acos", "atan", "atan2"]) {
      const rows = findCoverageGapRows(report, "functions", residue);
      expect(rows.length).toBeGreaterThan(0);
      expect(rows.every((row) => row.capabilityTier === "T0")).toBe(true);
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
      expect(rows.every((row) => row.capabilityTier === "T0")).toBe(true);
      expect(rows.every((row) => !row.measurements.staticallyReduced)).toBe(true);
    }
  });

  it("rejects a value-tier promotion without matcher evidence", () => {
    const tampered = JSON.parse(JSON.stringify(report)) as CoverageGapReport;
    const row = tampered.rows.find(
      (candidate) => candidate.category === "properties" && candidate.name === "display",
    );
    expect(row).toBeDefined();
    Object.assign(row ?? {}, { capabilityTier: "T2" });
    expect(() =>
      validateCoverageGapReport(
        tampered,
        JSON.parse(
          readFileSync("rust/crates/omena-spec-audit/data/webref-grammar.json", "utf8"),
        ) as WebrefGrammarSnapshot,
        recognition,
        fold,
        sources.valueGrammarEvidence,
      ),
    ).toThrow(/cannot claim a value tier without matcher evidence/u);
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
    expect(firstRecognitionRow?.capabilityTier).toBeNull();
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
    ).toThrow(/capability tier or a registered reason/u);
    expect(() =>
      buildCoverageGapReportFromRepo(process.cwd(), { injectFreeTextReason: true }),
    ).toThrow(/unknown reason/u);
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

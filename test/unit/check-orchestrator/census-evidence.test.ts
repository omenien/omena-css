import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  CENSUS_ROW_IDS,
  assertCensusPopulation,
  assertNoLiteralRowSelection,
  checkerInventory,
} from "../../../scripts/lib/census-instrument-evidence";
import { hasRuntimeEvidence } from "../../../scripts/lib/rust-semver-intent";

const root = path.resolve(__dirname, "../../..");
const tick = String.fromCharCode(96);

describe("census evidence operands", () => {
  it("requires every externally declared row, naming each removal", () => {
    expect(() => assertCensusPopulation(CENSUS_ROW_IDS)).not.toThrow();
    for (const missing of CENSUS_ROW_IDS) {
      expect(() => assertCensusPopulation(CENSUS_ROW_IDS.filter((id) => id !== missing))).toThrow(
        "census row missing " + missing,
      );
    }
    expect(() => assertCensusPopulation([...CENSUS_ROW_IDS, "invented"])).toThrow(
      "unexpected census row invented",
    );
  });

  it.each([
    'row.id == "a1"',
    'row.id === "a1"',
    '"a1" === row.id',
    '(row.id) === ("a1")',
    '["a1"].includes(row.id)',
    'row.id.includes("a1")',
    "row.id === " + tick + "a1" + tick,
    tick + "a1" + tick + " === (row.id)",
    'row["id"] === "a1"',
    'row.expected.refusal === "binding does not exercise"',
    '"binding does not exercise" === (row.expected.refusal)',
    '["binding does not exercise"].includes(row.expected.refusal)',
    "row.expected.refusalPrefix === " + tick + "production reaches test constructor " + tick,
  ])("refuses literal selection through %s", (expression) => {
    expect(() =>
      assertNoLiteralRowSelection("function select(row) { return " + expression + "; }"),
    ).toThrow("per-row literal selection is forbidden");
  });

  it("refuses switches and local aliases, while permitting typed mechanism dispatch", () => {
    for (const source of [
      'switch ((row.id)) { case "a1": break; }',
      'const prefix = row.expected.refusalPrefix; if (prefix === "unregistered") {}',
      'const id = (row.id); const alias = id; if ("f" == alias) {}',
      'const id = "a1"; if (row.id === id) {}',
      'const ids = ["a1"]; if (ids.includes(row.id)) {}',
      'const text = "binding does not exercise"; if (row.expected.refusal === text) {}',
      'const id = "a1"; const alias = id; if (alias === row.id) {}',
      'const id = "a1"; function nested(row) { return row.id === id; }',
      'function nested(row) { const id = "a1"; { return row.id === id; } }',
    ])
      expect(() => assertNoLiteralRowSelection(source)).toThrow("per-row literal selection");
    expect(() =>
      assertNoLiteralRowSelection(
        'switch (row.gate.kind) { case "compiler": break; } const x = row.expected.refusal;',
      ),
    ).not.toThrow();
    expect(() =>
      assertNoLiteralRowSelection(
        'const id = "a1"; function nested(row, id) { return row.id === id; }',
      ),
    ).not.toThrow();
    expect(() =>
      assertNoLiteralRowSelection(
        'function nested(row, id) { return row.id === id; } const id = "a1";',
      ),
    ).not.toThrow();
    expect(() =>
      assertNoLiteralRowSelection(
        'const id = "a1"; function nested(row, input) { { const id = input; return row.id === id; } }',
      ),
    ).not.toThrow();
    expect(() =>
      assertNoLiteralRowSelection(
        readFileSync(path.join(root, "scripts/check-rust-census-instrument-s0.ts"), "utf8"),
      ),
    ).not.toThrow();
  });

  it("keys actual calls and the helper definition by their complete source content", () => {
    const source =
      'function blockBody(x) { return x; }\nassert.match(value, /required/);\nblockBody("x");\n';
    const initial = checkerInventory("checker.ts", source);
    const changed = checkerInventory("checker.ts", source.replace("/required/", "/(?:)/"));
    expect(initial).toHaveLength(3);
    expect(initial.filter((row) => row.kind === "blockBody")).toHaveLength(2);
    expect(changed.find((row) => row.kind === "assert")!.identity).not.toEqual(
      initial.find((row) => row.kind === "assert")!.identity,
    );
    expect(
      checkerInventory("checker.ts", source + '// assert.ok(true); blockBody("decoy");\n'),
    ).toEqual(initial);
  });
});

describe("semver declaration evidence", () => {
  const needle = "provider_completeness: ProviderCompletenessV1";
  const declared = "pub struct AnalysisPrecisionV1 {\n    " + needle + ",\n}";
  const decoys = [
    "fn from_axes(" + needle + ") {}",
    "/// " + needle,
    'const DECOY: &str = "' + needle + '";',
    "struct DifferentType {\n    " + needle + ",\n}",
  ].join("\n");
  it("requires the field in the certified struct even when every decoy survives", () => {
    expect(hasRuntimeEvidence(declared + "\n" + decoys, needle, "AnalysisPrecisionV1")).toBe(true);
    expect(
      hasRuntimeEvidence(
        declared.replace(needle + ",", "") + "\n" + decoys,
        needle,
        "AnalysisPrecisionV1",
      ),
    ).toBe(false);
    expect(() => hasRuntimeEvidence(declared, needle)).toThrow("struct declaration operand");
  });
  it("does not accept a comment or string transcription as runtime evidence", () => {
    const value = "WorldAssumptionV1::Open";
    expect(hasRuntimeEvidence("// " + value + '\nconst X: &str = "' + value + '";', value)).toBe(
      false,
    );
    expect(hasRuntimeEvidence("let world = " + value + ";", value)).toBe(true);
    expect(
      hasRuntimeEvidence(
        '#[serde(rename = "k-cfa")]\nKLimitedCallSite,',
        '#[serde(rename = "k-cfa")]',
      ),
    ).toBe(true);
  });
});

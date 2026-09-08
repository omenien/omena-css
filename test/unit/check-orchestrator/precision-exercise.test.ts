import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  assertPrecisionFlip,
  precisionBindingSpans,
  precisionExerciseInventory,
  readPrecisionVector,
  sha256,
  type PrecisionExerciseAuthority,
  type PrecisionVector,
} from "../../../scripts/lib/precision-exercise";
import { PRECISION_EXERCISE_CASES } from "../../../scripts/lib/precision-exercise-cases";

describe("precision exercise evidence", () => {
  const pair = PRECISION_EXERCISE_CASES[0]!;
  const axes = {
    valueDomain: "unknown",
    flowSensitivity: "unknown",
    contextSensitivity: "unknown",
    providerCompleteness: "unresolved",
    worldAssumption: "open",
    revisionAxis: "current",
  };
  const before: PrecisionVector = {
    probeId: pair.id,
    testId: pair.testPath,
    fixtureFile: pair.fixtureFile.replace(/^rust\//u, ""),
    fixtureSource: readFileSync(path.resolve(__dirname, "../../..", pair.fixtureFile), "utf8"),
    actualAxes: axes,
    expectedAxes: axes,
  };
  const after = { ...before, actualAxes: { ...axes, providerCompleteness: "complete" } };
  const failure = `${pair.testPath} ... FAILED\nemitted precision vector ${pair.id}`;
  const envelope = (row: PrecisionVector) => `OMENA_PRECISION_VECTOR ${JSON.stringify(row)}\n`;

  it("binds a complete vector to the independently registered fixture and test", () => {
    expect(sha256(before.fixtureSource)).toBe(pair.fixtureSha256);
    expect(readPrecisionVector(envelope(before), pair)).toEqual(before);
    expect(assertPrecisionFlip(pair, before, after, 101, failure)).toEqual([
      "providerCompleteness",
    ]);
  });

  it.each([
    ["probe identity", { ...before, probeId: "fabricated" }],
    ["test identity", { ...before, testId: "unrelated::test" }],
    ["fixture path", { ...before, fixtureFile: "unrelated.rs" }],
    ["external digest", { ...before, fixtureSource: "// same id, no product assertion" }],
    ["all axes", { ...before, actualAxes: { valueDomain: "unknown" } }],
  ])("refuses a mismatched %s", (_label, row) => {
    expect(() => readPrecisionVector(envelope(row as PrecisionVector), pair)).toThrow();
  });

  it("rejects zero or duplicate executions", () => {
    expect(() => readPrecisionVector("test result: ok. 0 passed", pair)).toThrow(
      "missing or ambiguous",
    );
    expect(() => readPrecisionVector(envelope(before) + envelope(before), pair)).toThrow(
      "missing or ambiguous",
    );
  });

  it("rejects a function-execution receipt with unchanged precision", () => {
    expect(() => assertPrecisionFlip(pair, before, before, 101, failure)).toThrow(
      "probe does not move precision",
    );
  });

  it("rejects an edited oracle and unrelated failures", () => {
    expect(() =>
      assertPrecisionFlip(pair, before, { ...after, expectedAxes: after.actualAxes }, 101, failure),
    ).toThrow("fixture expectation changed");
    expect(() => assertPrecisionFlip(pair, before, after, 0, failure)).toThrow(
      "product test did not fail",
    );
    expect(() => assertPrecisionFlip(pair, before, after, 127, failure)).toThrow(
      "product test did not fail",
    );
    expect(() => assertPrecisionFlip(pair, before, after, 101, "error: could not compile")).toThrow(
      "precision assertion did not fail",
    );
    expect(() =>
      assertPrecisionFlip(pair, before, after, 101, `emitted precision vector ${pair.id}`),
    ).toThrow("named product test did not fail");
  });
});

describe("item-local precision bindings", () => {
  function withFixture(body: (root: string, authority: PrecisionExerciseAuthority) => void) {
    const root = mkdtempSync(path.join(os.tmpdir(), "precision-spans-"));
    try {
      writeFileSync(
        path.join(root, "fixture.rs"),
        "fn first() { first_axis(); }\nfn second() { second_axis(); }\n",
      );
      body(root, {
        precisionEmissionPoints: [
          {
            id: "point",
            sourcePath: "fixture.rs",
            function: "second",
            disposition: { bindingId: "probe" },
          },
        ],
        mutationProbes: [],
        bindingProbes: [
          { id: "probe", sourcePath: "fixture.rs", from: "second_axis()", to: "changed_axis()" },
        ],
      });
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  }

  it("accepts an authored span independently of legacy mutation records", () =>
    withFixture((root, authority) => {
      expect(precisionBindingSpans(root, authority).get("probe")).toEqual(new Set(["point"]));
    }));

  it("rejects same-file repoints and missing ids", () =>
    withFixture((root, authority) => {
      const wrong = {
        ...authority,
        bindingProbes: [
          { id: "probe", sourcePath: "fixture.rs", from: "first_axis()", to: "changed_axis()" },
        ],
      };
      expect(() => precisionBindingSpans(root, wrong)).toThrow("binding does not exercise point");
      expect(() => precisionBindingSpans(root, { ...authority, bindingProbes: [] })).toThrow(
        "binding does not exercise point",
      );
    }));

  it("requires the whole mutation span inside the function body", () =>
    withFixture((root, authority) => {
      expect(() =>
        precisionBindingSpans(root, {
          ...authority,
          bindingProbes: [
            {
              id: "probe",
              sourcePath: "fixture.rs",
              from: "second_axis(); }\n",
              to: "changed_axis(); }\n",
            },
          ],
        }),
      ).toThrow("binding does not exercise point");
    }));

  it("refuses an unrelated gated exclusion", () =>
    withFixture((root, authority) => {
      expect(() =>
        precisionBindingSpans(root, {
          ...authority,
          gatedExclusions: [{ id: "exception", pointId: "other", probe: "probe" }],
        }),
      ).toThrow("gated exclusion exception not exercised by probe");
    }));
});

describe("precision identity floors", () => {
  const root = path.resolve(__dirname, "../../..");
  const authority = JSON.parse(
    readFileSync(path.join(root, "rust/omena-precision-floor-authority.json"), "utf8"),
  ) as PrecisionExerciseAuthority;

  it("prints every point-bearing crate and the bounded residue", () => {
    const inventory = precisionExerciseInventory(root, authority);
    expect(Object.values(inventory.perCrate).every((n) => n >= 1)).toBe(true);
    expect(inventory.pairs.length).toBe(PRECISION_EXERCISE_CASES.length);
    expect(inventory.noProbe).toHaveLength(1);
    expect(inventory.censusOnly).toHaveLength(9);
  });

  it("does not replace an identity floor with an equal count", () => {
    const changed = structuredClone(authority);
    Reflect.set(changed.precisionEmissionPoints[0]!, "id", "replacement-identity");
    expect(() => precisionExerciseInventory(root, changed)).toThrow(
      "census floor unmet emissionPoints",
    );
  });

  it("does not discharge a checker-only row by changing its command label", () => {
    const changed = structuredClone(authority);
    const probe = changed.mutationProbes.find(({ command }) => command?.[0] !== "cargo")!;
    Reflect.set(probe, "command", ["cargo", "test"]);
    expect(precisionExerciseInventory(root, changed).censusOnly).toContain(
      `probe-census-only:${probe.id}`,
    );
  });

  it("rejects a removed pair while other pairs remain in that crate", () => {
    expect(() =>
      precisionExerciseInventory(root, authority, PRECISION_EXERCISE_CASES.slice(1)),
    ).toThrow("census floor unmet authoredPairs");
  });

  it("refuses counting all points instead of exercising them", () => {
    expect(() => precisionExerciseInventory(root, authority, [])).toThrow(
      "authored pairs per crate >= 1",
    );
  });
});

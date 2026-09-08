import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import { PRECISION_EXERCISE_BIRTH_IDS, PRECISION_PICKUP_PIN } from "./precision-exercise-baseline";
import { rustNamedFunctions } from "./rust-write-authority";
import {
  PRECISION_EXERCISE_CASES,
  PRECISION_UNOBSERVABLE_POINTS,
  type PrecisionExerciseCase,
} from "./precision-exercise-cases";

export interface PrecisionEmissionPoint {
  readonly id: string;
  readonly sourcePath: string;
  readonly function: string;
  readonly disposition?: { readonly bindingId?: string };
}

export interface PrecisionProbe {
  readonly id: string;
  readonly sourcePath?: string;
  readonly from?: string;
  readonly to?: string;
  readonly changes?: readonly {
    readonly sourcePath: string;
    readonly from: string;
    readonly to: string;
  }[];
  readonly command?: readonly string[];
}

export interface PrecisionExerciseAuthority {
  readonly precisionEmissionPoints: readonly PrecisionEmissionPoint[];
  readonly mutationProbes: readonly PrecisionProbe[];
  readonly bindingProbes?: readonly PrecisionProbe[];
  readonly gatedExclusions?: readonly {
    readonly id: string;
    readonly pointId: string;
    readonly probe: string;
  }[];
}

export function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

export function precisionProbeChanges(probe: PrecisionProbe) {
  if (probe.changes) {
    assert.ok(probe.changes.length > 0, `probe span not resolvable ${probe.id}`);
    return probe.changes;
  }
  assert.ok(
    probe.sourcePath && probe.from && probe.to !== undefined,
    `probe span not resolvable ${probe.id}`,
  );
  return [{ sourcePath: probe.sourcePath, from: probe.from, to: probe.to }];
}

export function precisionBindingSpans(
  root: string,
  authority: PrecisionExerciseAuthority,
): Map<string, Set<string>> {
  const points = authority.precisionEmissionPoints.map((point) => {
    const source = readFileSync(path.join(root, point.sourcePath), "utf8");
    const functions = rustNamedFunctions(source, "crate").filter(
      ({ shortName }) => shortName === point.function,
    );
    assert.equal(functions.length, 1, `point item not resolvable ${point.id}`);
    return { point, region: functions[0]! };
  });
  const result = new Map<string, Set<string>>();
  for (const probe of [...authority.mutationProbes, ...(authority.bindingProbes ?? [])]) {
    assert.ok(!result.has(probe.id), `duplicate precision probe ${probe.id}`);
    const matched = new Set<string>();
    for (const change of precisionProbeChanges(probe)) {
      const source = readFileSync(path.join(root, change.sourcePath), "utf8");
      assert.ok(change.from.length > 0, `probe span not resolvable ${probe.id}`);
      assert.equal(
        source.split(change.from).length - 1,
        1,
        `probe span not resolvable ${probe.id}`,
      );
      const start = source.indexOf(change.from);
      const end = start + change.from.length;
      if (!change.sourcePath.endsWith(".rs")) continue;
      for (const { point, region } of points) {
        if (point.sourcePath === change.sourcePath && region.bodyStart < start && end < region.end)
          matched.add(point.id);
      }
    }
    result.set(probe.id, matched);
  }
  for (const point of authority.precisionEmissionPoints) {
    const id = point.disposition?.bindingId;
    assert.ok(id && result.get(id)?.has(point.id), `binding does not exercise ${point.id}`);
  }
  for (const row of authority.gatedExclusions ?? []) {
    assert.ok(
      result.get(row.probe)?.has(row.pointId),
      `gated exclusion ${row.id} not exercised by ${row.probe}`,
    );
  }
  return result;
}

export function precisionExerciseInventory(
  root: string,
  authority: PrecisionExerciseAuthority,
  cases: readonly PrecisionExerciseCase[] = PRECISION_EXERCISE_CASES,
) {
  const spans = precisionBindingSpans(root, authority);
  const points = new Map(authority.precisionEmissionPoints.map((point) => [point.id, point]));
  assert.equal(
    points.size,
    authority.precisionEmissionPoints.length,
    "duplicate emission point identity",
  );
  const baseline = JSON.parse(
    execFileSync(
      "git",
      ["show", `${PRECISION_PICKUP_PIN}:rust/omena-precision-floor-authority.json`],
      { cwd: root, encoding: "utf8" },
    ),
  ) as PrecisionExerciseAuthority;
  for (const point of baseline.precisionEmissionPoints)
    assert.ok(points.has(point.id), `census floor unmet emissionPoints ${point.id}`);
  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--manifest-path", "rust/Cargo.toml", "--no-deps", "--format-version", "1"],
      { cwd: root, encoding: "utf8" },
    ),
  ) as { readonly packages: readonly { readonly name: string; readonly manifest_path: string }[] };
  const ownerOf = (relative: string): string => {
    const file = path.resolve(root, relative);
    const owners = metadata.packages.filter((pkg) => {
      const memberPath = path.relative(path.dirname(pkg.manifest_path), file);
      return (
        memberPath.length > 0 &&
        !memberPath.startsWith(`..${path.sep}`) &&
        memberPath !== ".." &&
        !path.isAbsolute(memberPath)
      );
    });
    assert.equal(owners.length, 1, `point owner not resolvable ${relative}`);
    return owners[0]!.name;
  };
  const perCrate = new Map<string, number>();
  for (const point of points.values()) {
    const owner = ownerOf(point.sourcePath);
    perCrate.set(owner, 0);
  }
  const pairs: PrecisionExerciseCase[] = [];
  for (const pair of cases) {
    const point = points.get(pair.pointId);
    assert.ok(point, `census floor unmet emissionPoints ${pair.pointId}`);
    assert.equal(
      point.disposition?.bindingId,
      pair.id,
      `probe census-only ${point.disposition?.bindingId}`,
    );
    assert.ok(spans.get(pair.id)?.has(point.id), `binding does not exercise ${point.id}`);
    assert.equal(
      ownerOf(point.sourcePath),
      pair.owningCrate,
      `probe crate does not own point ${pair.id}`,
    );
    assert.equal(
      ownerOf(pair.fixtureFile),
      pair.owningCrate,
      `fixture crate does not own point ${pair.id}`,
    );
    const fixture = readFileSync(path.join(root, pair.fixtureFile), "utf8");
    assert.equal(
      sha256(fixture),
      pair.fixtureSha256,
      `authored fixture digest mismatch ${pair.id}`,
    );
    const probe = authority.bindingProbes?.find(({ id }) => id === pair.id);
    assert.deepEqual(
      probe,
      pair.mutation,
      `authored mutation differs from execution set ${pair.id}`,
    );
    perCrate.set(pair.owningCrate, (perCrate.get(pair.owningCrate) ?? 0) + 1);
    pairs.push(pair);
  }
  for (const [crate, count] of perCrate)
    assert.ok(count >= 1, `authored pairs per crate >= 1: ${crate}`);
  const pairIds = new Set(pairs.map(({ id }) => id));
  assert.equal(pairIds.size, pairs.length, "duplicate authored pair identity");
  for (const id of PRECISION_EXERCISE_BIRTH_IDS)
    assert.ok(pairIds.has(id), `census floor unmet authoredPairs ${id}`);
  const covered = new Set(pairs.map(({ pointId }) => pointId));
  const noProbe = [...points.keys()].filter((id) => !covered.has(id));
  for (const id of noProbe)
    assert.ok(
      (PRECISION_UNOBSERVABLE_POINTS as readonly string[]).includes(id),
      `no-probe population grew ${id}`,
    );
  for (const id of PRECISION_UNOBSERVABLE_POINTS)
    assert.ok(points.has(id), `census floor unmet emissionPoints ${id}`);
  // A changed command label cannot promote a checker-only probe to product evidence.
  // Preserve only the unchanged product probes measured at pickup outside this suite.
  const existingProductProbes = baseline.mutationProbes.filter(
    ({ command }) => command?.[0] === "cargo" && command[1] === "test",
  );
  const censusOnly = authority.mutationProbes
    .filter((probe) => !existingProductProbes.some((known) => isDeepStrictEqual(known, probe)))
    .map(({ id }) => `probe-census-only:${id}`);
  const counted = [...censusOnly, ...noProbe.map((id) => `precision-no-probe:${id}`)].toSorted();
  const instrument = JSON.parse(
    readFileSync(path.join(root, "rust/census-instrument-s0.json"), "utf8"),
  ) as {
    readonly countedResidue: readonly {
      readonly identity: string;
      readonly kind: string;
      readonly owner: string;
    }[];
  };
  const baselineInstrument = JSON.parse(
    execFileSync("git", ["show", `${PRECISION_PICKUP_PIN}:rust/census-instrument-s0.json`], {
      cwd: root,
      encoding: "utf8",
    }),
  ) as typeof instrument;
  const censusOnlyBirth = new Set(
    baselineInstrument.countedResidue
      .filter(({ kind }) => kind === "probe-census-only")
      .map(({ identity }) => identity),
  );
  for (const id of censusOnly)
    assert.ok(censusOnlyBirth.has(id), `probe census-only population grew ${id}`);
  const rows = instrument.countedResidue.filter(
    ({ kind }) => kind === "probe-census-only" || kind === "precision-no-probe",
  );
  assert.deepEqual(
    rows.map(({ identity }) => identity).toSorted(),
    counted,
    "precision counted residue not enumerated",
  );
  for (const row of rows)
    assert.ok(row.owner.trim().length > 0, `precision counted owner missing ${row.identity}`);
  assert.ok(
    instrument.countedResidue.some(
      ({ identity, owner }) =>
        identity === "writer-rehoming-deferred:rust/omena-precision-floor-authority.json" &&
        owner.trim().length > 0,
    ),
    "precision W3 authority re-homing residue missing",
  );
  return { pairs, perCrate: Object.fromEntries(perCrate), noProbe, censusOnly, countedRows: rows };
}

const AXES = [
  "contextSensitivity",
  "flowSensitivity",
  "providerCompleteness",
  "revisionAxis",
  "valueDomain",
  "worldAssumption",
] as const;

export interface PrecisionVector {
  readonly probeId: string;
  readonly testId: string;
  readonly fixtureFile: string;
  readonly fixtureSource: string;
  readonly actualAxes: Readonly<Record<string, string>>;
  readonly expectedAxes: Readonly<Record<string, string>>;
}

export function readPrecisionVector(
  transcript: string,
  pair: PrecisionExerciseCase,
): PrecisionVector {
  const prefix = "OMENA_PRECISION_VECTOR ";
  const rows = transcript
    .split(/\r?\n/u)
    .filter((line) => line.startsWith(prefix))
    .map((line) => JSON.parse(line.slice(prefix.length)) as PrecisionVector);
  assert.equal(rows.length, 1, `emitted precision vector missing or ambiguous ${pair.id}`);
  const row = rows[0]!;
  assert.equal(row.probeId, pair.id, `execution probe identity mismatch ${pair.id}`);
  assert.equal(row.testId, pair.testPath, `execution test identity mismatch ${pair.id}`);
  assert.equal(
    row.fixtureFile,
    pair.fixtureFile.replace(/^rust\//u, ""),
    `execution fixture path mismatch ${pair.id}`,
  );
  assert.equal(
    sha256(row.fixtureSource),
    pair.fixtureSha256,
    `compiled fixture digest mismatch ${pair.id}`,
  );
  for (const vector of [row.actualAxes, row.expectedAxes]) {
    assert.deepEqual(Object.keys(vector).toSorted(), AXES, `emitted axes incomplete ${pair.id}`);
    assert.ok(
      Object.values(vector).every((value) => typeof value === "string" && value.length > 0),
      `emitted axes invalid ${pair.id}`,
    );
  }
  return row;
}

export function assertPrecisionFlip(
  pair: PrecisionExerciseCase,
  before: PrecisionVector,
  after: PrecisionVector,
  exit: number | null,
  transcript: string,
): readonly string[] {
  assert.deepEqual(
    before.actualAxes,
    before.expectedAxes,
    `baseline precision mismatch ${pair.id}`,
  );
  assert.deepEqual(
    after.expectedAxes,
    before.expectedAxes,
    `fixture expectation changed ${pair.id}`,
  );
  const changed = AXES.filter((axis) => before.actualAxes[axis] !== after.actualAxes[axis]);
  assert.ok(changed.length > 0, `probe does not move precision ${pair.id}`);
  assert.equal(exit, 101, `product test did not fail ${pair.id}`);
  assert.ok(
    transcript.includes(`emitted precision vector ${pair.id}`),
    `precision assertion did not fail ${pair.id}`,
  );
  assert.ok(
    transcript.includes(`${pair.testPath} ... FAILED`),
    `named product test did not fail ${pair.id}`,
  );
  return changed;
}

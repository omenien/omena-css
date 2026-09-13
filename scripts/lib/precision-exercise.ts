import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import { PRECISION_PICKUP_PIN, PRECISION_EXERCISE_BIRTH_PIN } from "./precision-exercise-baseline";
import { compilerApi as ts } from "../../server/engine-core-ts/src/ts-facade";
import {
  maskRustCommentsAndLiterals,
  matchingRustDelimiter,
  rustNamedFunctions,
  type CargoPackage,
} from "./rust-write-authority";
import { PRECISION_EXERCISE_CASES, type PrecisionExerciseCase } from "./precision-exercise-cases";

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

// Parse committed declarations, never evaluate candidate code as a ratchet operand.
export function precisionExerciseBirth(root: string) {
  const committed = (file: string) =>
    execFileSync("git", ["show", `${PRECISION_EXERCISE_BIRTH_PIN}:${file}`], {
      cwd: root,
      encoding: "utf8",
    });
  const literalArray = (file: string, name: string): readonly string[] => {
    const tree = ts.createSourceFile(file, committed(file), ts.ScriptTarget.Latest, true);
    const declarations = tree.statements.flatMap((statement) =>
      ts.isVariableStatement(statement)
        ? [...statement.declarationList.declarations].filter(
            (declaration) => ts.isIdentifier(declaration.name) && declaration.name.text === name,
          )
        : [],
    );
    assert.equal(declarations.length, 1, `committed precision operand missing ${name}`);
    let value = declarations[0]!.initializer;
    if (value && ts.isAsExpression(value)) value = value.expression;
    assert.ok(value && ts.isArrayLiteralExpression(value), `nonliteral precision operand ${name}`);
    return value.elements.map((element) => {
      assert.ok(ts.isStringLiteral(element), `nonliteral precision identity ${name}`);
      return element.text;
    });
  };
  const instrument = JSON.parse(committed("rust/census-instrument-s0.json")) as {
    countedResidue: { kind: string; identity: string }[];
  };
  return {
    pin: PRECISION_EXERCISE_BIRTH_PIN,
    pairIds: literalArray(
      "scripts/lib/precision-exercise-baseline.ts",
      "PRECISION_EXERCISE_BIRTH_IDS",
    ),
    noProbe: literalArray(
      "scripts/lib/precision-exercise-cases.ts",
      "PRECISION_UNOBSERVABLE_POINTS",
    ),
    censusOnly: instrument.countedResidue
      .filter(({ kind }) => kind === "probe-census-only")
      .map(({ identity }) => identity),
  };
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
  ) as { readonly packages: readonly CargoPackage[] };
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
      pair.producerPath.split("::").at(-1),
      point.function,
      `point producer not resolvable ${pair.pointId}`,
    );
    assertPrecisionProducer(fixture, pair);
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
  const birth = precisionExerciseBirth(root);
  const covered = new Set(pairs.map(({ pointId }) => pointId));
  const noProbe = [...points.keys()].filter((id) => !covered.has(id));
  for (const id of noProbe) assert.ok(birth.noProbe.includes(id), `no-probe population grew ${id}`);
  for (const id of birth.pairIds)
    assert.ok(pairIds.has(id), `census floor unmet authoredPairs ${id}`);
  for (const id of birth.noProbe)
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
      readonly reason?: string;
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
  const legacyRows = instrument.countedResidue.filter(
    ({ kind }) => kind === "legacy-product-probe",
  );
  assert.deepEqual(
    legacyRows.map(({ identity }) => identity).toSorted(),
    existingProductProbes.map(({ id }) => `legacy-product-probe:${id}`).toSorted(),
    "legacy product probes not counted",
  );
  for (const row of legacyRows)
    assert.ok(
      row.owner.trim() && row.reason?.trim(),
      `legacy product probe disposition missing ${row.identity}`,
    );
  assert.ok(
    instrument.countedResidue.some(
      ({ identity, owner }) =>
        identity === "writer-rehoming-deferred:rust/omena-precision-floor-authority.json" &&
        owner.trim().length > 0,
    ),
    "precision W3 authority re-homing residue missing",
  );
  return {
    pairs,
    perCrate: Object.fromEntries(perCrate),
    noProbe,
    censusOnly,
    countedRows: [...rows, ...legacyRows],
  };
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

// A deliberately closed expression grammar for compiled probe fixtures. Fully
// qualified Rust calls are resolved against the module walk by the inventory.
// Only immutable bindings and value-preserving field/Option/iterator projections
// may connect that call to the assertion's actual argument. Unknown syntax fails.
export function assertPrecisionProducer(source: string, pair: PrecisionExerciseCase): void {
  const failure = `asserted precision producer mismatch ${pair.pointId}`;
  const code = maskRustCommentsAndLiterals(source);
  const functions = rustNamedFunctions(source, "crate").filter(
    ({ shortName }) => shortName === pair.testPath.split("::").at(-1),
  );
  assert.equal(functions.length, 1, failure);
  const fn = functions[0]!;
  const body = code.slice(fn.bodyStart + 1, fn.end - 1);
  const split = (text: string, separator: string): string[] => {
    const parts: string[] = [];
    let start = 0;
    for (let offset = 0; offset < text.length; offset += 1) {
      const close = { "(": ")", "[": "]", "{": "}" }[text[offset]!];
      if (close) offset = matchingRustDelimiter(text, offset, text[offset]!, close);
      else if (text[offset] === separator) {
        parts.push(text.slice(start, offset).trim());
        start = offset + 1;
      }
    }
    parts.push(text.slice(start).trim());
    return parts;
  };
  const statements = split(body, ";");
  const assertions = statements.filter((statement) => /^assert_emitted_axes\s*\(/u.test(statement));
  assert.equal(assertions.length, 1, failure);
  const assertion = assertions[0]!;
  const assertionIndex = statements.indexOf(assertion);
  const open = assertion.indexOf("(");
  const close = matchingRustDelimiter(assertion, open, "(", ")");
  assert.equal(assertion.slice(close + 1).trim(), "", failure);
  const args = split(assertion.slice(open + 1, close), ",");
  assert.ok(args.length === 4 && args[3] === "", failure);
  const bindings = new Map<string, { expression: string; index: number; mutable: boolean }>();
  for (const [index, statement] of statements.slice(0, assertionIndex).entries()) {
    const binding = statement.match(/^let\s+(mut\s+)?([a-z_][a-z0-9_]*)\s*=\s*([\s\S]+)$/u);
    if (!binding) continue;
    assert.ok(!bindings.has(binding[2]!), failure);
    bindings.set(binding[2]!, { expression: binding[3]!, index, mutable: !!binding[1] });
  }
  const resolving = new Set<string>();
  const trace = (expression: string, beforeIndex: number, parameter?: string): void => {
    const text = expression.trim();
    const head = text.match(/^(crate(?:::[A-Za-z_][A-Za-z0-9_]*)+|[a-z_][a-z0-9_]*)/u);
    assert.ok(head, failure);
    const name = head[0];
    let offset = name.length;
    while (/\s/u.test(text[offset] ?? "") && offset < text.length) offset += 1;
    if (text[offset] === "(") {
      assert.equal(name, pair.producerPath, failure);
      offset = matchingRustDelimiter(text, offset, "(", ")") + 1;
    } else if (name !== parameter) {
      const binding = bindings.get(name);
      assert.ok(
        binding && binding.index < beforeIndex && !binding.mutable && !resolving.has(name),
        failure,
      );
      // An assignment, mutable borrow or shadow would break the immutable origin.
      const subsequent = statements.slice(binding.index + 1, assertionIndex).join(";");
      assert.ok(
        !new RegExp(
          `\\b${name}\\s*(?:\\.[a-z_][a-z0-9_]*\\s*)*(?:=(?!=)|[+*/-]=)|&\\s*mut\\s+${name}\\b`,
          "u",
        ).test(subsequent),
        failure,
      );
      resolving.add(name);
      trace(binding.expression, binding.index);
      resolving.delete(name);
    }
    while (offset < text.length) {
      const tail = text.slice(offset);
      if (/^\s*\?\s*$/u.test(tail)) return;
      const projection = tail.match(/^\s*\.\s*([a-z_][a-z0-9_]*)\s*/u);
      assert.ok(projection, failure);
      const member = projection[1]!;
      offset += projection[0].length;
      if (text[offset] !== "(") {
        assert.ok(["axes", "precision", "diagnostics"].includes(member), failure);
        continue;
      }
      const end = matchingRustDelimiter(text, offset, "(", ")");
      const argument = text.slice(offset + 1, end).trim();
      if (["first", "as_ref", "iter"].includes(member)) assert.equal(argument, "", failure);
      else if (member === "ok_or") {
        // A masked literal only controls the error branch; it cannot replace Ok.
        assert.equal(argument, "", failure);
      } else if (member === "and_then") {
        const closure = argument.match(/^\|([a-z_][a-z0-9_]*)\|\s*([\s\S]+)$/u);
        assert.ok(closure, failure);
        trace(closure[2]!, beforeIndex, closure[1]!);
      } else if (member === "find") {
        // find selects an original item; forbid a callback with side effects.
        assert.ok(/^\|([a-z_][a-z0-9_]*)\|\s*\1\.code\s*==\s*$/u.test(argument), failure);
      } else assert.fail(failure);
      offset = end + 1;
    }
  };
  trace(args[1]!, assertionIndex);
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
  assertPrecisionProducer(row.fixtureSource, pair);
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

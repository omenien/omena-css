import { execFileSync, spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

interface ExpectedCargoSemverDiagnostic {
  readonly lint: string;
  readonly level?: "major" | "minor";
  readonly evidenceNeedles: readonly string[];
  readonly witnessLinePrefix?: string;
  readonly expectedWitnesses?: readonly string[];
}

interface ExpectedRuntimeValueChange {
  readonly id: string;
  readonly kind:
    | "wire-value-order-preserving"
    | "wire-order-changing"
    | "selection-outcome-changing";
  readonly surface: string;
  readonly reason: string;
  readonly evidence: readonly {
    readonly sourcePath: string;
    readonly needles: readonly string[];
  }[];
}

interface RustSemverIntent {
  readonly crate: string;
  readonly releaseClass: "pre1MinorBreaking";
  readonly reason: string;
  readonly expectedFailures: readonly ExpectedCargoSemverDiagnostic[];
  readonly expectedWarnings?: readonly ExpectedCargoSemverDiagnostic[];
  readonly expectedRuntimeValueChanges?: readonly ExpectedRuntimeValueChange[];
}

interface RustSemverIntentRegister {
  readonly schemaVersion: "0";
  readonly product: "omena-rust-semver-intent";
  readonly baselineWorkspaceVersion: string;
  readonly targetReleaseVersion: string;
  readonly intents: readonly RustSemverIntent[];
}

interface RuntimeHonestySurface {
  readonly owner: string;
  readonly changeId: string;
  readonly surface: string;
  readonly evidenceNeedles: readonly string[];
}

interface AuthoredTextEgressHonestyTable {
  readonly derivation: "final-head-byte-differential";
  readonly orderBranchSurfaces: readonly RuntimeHonestySurface[];
  readonly valueBranchSurfaces: readonly RuntimeHonestySurface[];
  readonly zeroBranchSurfaces: readonly {
    readonly surface: string;
    readonly evidenceGate: string;
  }[];
}

interface RunDeclaredRustSemverCheckOptions {
  readonly repoRoot: string;
  readonly crate: string;
  readonly workspaceVersion: string;
  readonly baselineArgs: readonly string[];
  readonly allFeatures: boolean;
}

export interface DeclaredRustSemverCheckResult {
  readonly policy: "steady-state-patch" | "declared-pre1-minor-breaking";
  readonly declaredFailureCount: number;
  readonly declaredWarningCount: number;
  readonly declaredRuntimeValueChangeCount: number;
  readonly declaredReleaseVersion: string | null;
}

const registerRelativePath = "rust/omena-rust-semver-intent.json";
const diagnosticHeaderPattern = /^--- (failure|warning) ([a-z0-9_]+):[^\n]*$/gmu;
const runtimeValueChangeKinds = new Set<ExpectedRuntimeValueChange["kind"]>([
  "wire-value-order-preserving",
  "wire-order-changing",
  "selection-outcome-changing",
]);

export function declaredRustSemverIntentCrates(repoRoot: string): readonly string[] {
  return readRegister(repoRoot).intents.map((intent) => intent.crate);
}

export function validateRustSemverIntentRegister(repoRoot: string): {
  readonly intentCount: number;
  readonly runtimeValueChangeCount: number;
  readonly honestyOrderSurfaceCount: number;
  readonly honestyValueSurfaceCount: number;
  readonly honestyZeroSurfaceCount: number;
  readonly honestySelftestMutationCount: 2;
} {
  const register = readRegister(repoRoot);
  const workspaceVersion = readWorkspaceVersion(repoRoot);
  assertReleaseWindow(
    register.baselineWorkspaceVersion,
    register.targetReleaseVersion,
    workspaceVersion,
  );

  let runtimeValueChangeCount = 0;
  for (const intent of register.intents) {
    assert.ok(intent.reason.trim().length > 0, `${intent.crate} semver intent requires a reason`);
    const runtimeChanges = intent.expectedRuntimeValueChanges ?? [];
    assert.ok(
      intent.expectedFailures.length > 0 || runtimeChanges.length > 0,
      `${intent.crate} semver intent must name a cargo diagnostic or runtime value change`,
    );
    assert.equal(
      new Set(runtimeChanges.map((change) => change.id)).size,
      runtimeChanges.length,
      `${intent.crate} runtime value change ids must be unique`,
    );
    for (const change of runtimeChanges) {
      assert.ok(
        runtimeValueChangeKinds.has(change.kind),
        `${intent.crate}:${change.id} has unsupported runtime value change kind ${JSON.stringify(change.kind)}`,
      );
      assert.ok(change.reason.trim().length > 0, `${intent.crate}:${change.id} requires a reason`);
      assert.ok(
        change.surface.trim().length > 0,
        `${intent.crate}:${change.id} requires a surface`,
      );
      assert.ok(change.evidence.length > 0, `${intent.crate}:${change.id} requires evidence`);
      for (const evidence of change.evidence) {
        assert.ok(
          evidence.sourcePath.startsWith("rust/"),
          `${intent.crate}:${change.id} evidence must stay in the Rust product surface`,
        );
        assert.ok(
          evidence.needles.length > 0,
          `${intent.crate}:${change.id} evidence requires source needles`,
        );
        const source = readFileSync(path.join(repoRoot, evidence.sourcePath), "utf8");
        for (const needle of evidence.needles) {
          assert.ok(
            source.includes(needle),
            `${intent.crate}:${change.id} is missing evidence ${JSON.stringify(needle)} in ${evidence.sourcePath}`,
          );
        }
      }
    }
    runtimeValueChangeCount += runtimeChanges.length;
  }

  const honestyTable = readAuthoredTextEgressHonestyTable(repoRoot);
  const honestyCounts = validateRuntimeHonestyTable(register.intents, honestyTable);
  runRuntimeHonestyTableSelftest();

  return {
    intentCount: register.intents.length,
    runtimeValueChangeCount,
    ...honestyCounts,
    honestySelftestMutationCount: 2,
  };
}

function validateRuntimeHonestyTable(
  intents: readonly RustSemverIntent[],
  table: AuthoredTextEgressHonestyTable,
): {
  readonly honestyOrderSurfaceCount: number;
  readonly honestyValueSurfaceCount: number;
  readonly honestyZeroSurfaceCount: number;
} {
  const declared = new Map(
    intents.flatMap((intent) =>
      (intent.expectedRuntimeValueChanges ?? []).map(
        (change) => [`${intent.crate}\0${change.id}`, change] as const,
      ),
    ),
  );
  const branchRows = [
    ...table.orderBranchSurfaces.map((surface) => ({
      surface,
      expectedKind: "wire-order-changing" as const,
      branch: "order" as const,
    })),
    ...table.valueBranchSurfaces.map((surface) => ({
      surface,
      expectedKind: "selection-outcome-changing" as const,
      branch: "value" as const,
    })),
  ];
  assert.equal(
    new Set(branchRows.map(({ surface }) => `${surface.owner}\0${surface.changeId}`)).size,
    branchRows.length,
    "authored-text egress honesty rows must have unique owner/change ids",
  );
  for (const { surface, expectedKind, branch } of branchRows) {
    const change = declared.get(`${surface.owner}\0${surface.changeId}`);
    assert.ok(
      change,
      `${branch}-branch honesty surface ${surface.owner}:${surface.changeId} requires a runtime value change row`,
    );
    assert.equal(
      change.kind,
      expectedKind,
      `${surface.owner}:${surface.changeId} ${branch}-branch honesty surface requires ${expectedKind}`,
    );
    assert.equal(
      change.surface,
      surface.surface,
      `${surface.owner}:${surface.changeId} honesty surface text drifted`,
    );
    assert.ok(
      surface.evidenceNeedles.length > 0,
      `${surface.owner}:${surface.changeId} honesty surface requires a manual identity implementation needle`,
    );
    const declaredNeedles = new Set(change.evidence.flatMap((evidence) => evidence.needles));
    for (const needle of surface.evidenceNeedles) {
      assert.ok(
        declaredNeedles.has(needle),
        `${surface.owner}:${surface.changeId} honesty surface is missing manual identity evidence ${JSON.stringify(needle)}`,
      );
    }
  }
  assert.equal(
    new Set(table.zeroBranchSurfaces.map((surface) => surface.surface)).size,
    table.zeroBranchSurfaces.length,
    "authored-text zero-branch surfaces must be unique",
  );
  for (const surface of table.zeroBranchSurfaces) {
    assert.ok(surface.surface.trim().length > 0, "zero-branch honesty surface requires a name");
    assert.ok(
      surface.evidenceGate.trim().length > 0,
      `${surface.surface} zero-branch honesty surface requires an evidence gate`,
    );
  }
  return {
    honestyOrderSurfaceCount: table.orderBranchSurfaces.length,
    honestyValueSurfaceCount: table.valueBranchSurfaces.length,
    honestyZeroSurfaceCount: table.zeroBranchSurfaces.length,
  };
}

function runRuntimeHonestyTableSelftest(): void {
  const orderSurface: RuntimeHonestySurface = {
    owner: "fixture-order",
    changeId: "manual-order",
    surface: "fixture order surface",
    evidenceNeedles: ["impl Ord for FixtureOrder"],
  };
  const valueSurface: RuntimeHonestySurface = {
    owner: "fixture-value",
    changeId: "manual-value",
    surface: "fixture value surface",
    evidenceNeedles: ["impl PartialEq for FixtureValue"],
  };
  const change = (
    id: string,
    kind: ExpectedRuntimeValueChange["kind"],
    surface: string,
    needle: string,
  ): ExpectedRuntimeValueChange => ({
    id,
    kind,
    surface,
    reason: "validator selftest fixture",
    evidence: [{ sourcePath: "rust/selftest.rs", needles: [needle] }],
  });
  const intent = (crate: string, runtimeChange: ExpectedRuntimeValueChange): RustSemverIntent => ({
    crate,
    releaseClass: "pre1MinorBreaking",
    reason: "validator selftest fixture",
    expectedFailures: [],
    expectedRuntimeValueChanges: [runtimeChange],
  });
  const table: AuthoredTextEgressHonestyTable = {
    derivation: "final-head-byte-differential",
    orderBranchSurfaces: [orderSurface],
    valueBranchSurfaces: [valueSurface],
    zeroBranchSurfaces: [],
  };
  const validIntents = [
    intent(
      orderSurface.owner,
      change(
        orderSurface.changeId,
        "wire-order-changing",
        orderSurface.surface,
        orderSurface.evidenceNeedles[0]!,
      ),
    ),
    intent(
      valueSurface.owner,
      change(
        valueSurface.changeId,
        "selection-outcome-changing",
        valueSurface.surface,
        valueSurface.evidenceNeedles[0]!,
      ),
    ),
  ];
  validateRuntimeHonestyTable(validIntents, table);
  assert.throws(
    () =>
      validateRuntimeHonestyTable(
        [
          intent(
            orderSurface.owner,
            change(
              orderSurface.changeId,
              "wire-value-order-preserving",
              orderSurface.surface,
              orderSurface.evidenceNeedles[0]!,
            ),
          ),
          validIntents[1]!,
        ],
        table,
      ),
    /requires wire-order-changing/u,
    "order-branch honesty selftest must reject a missing wire-order-changing row",
  );
  assert.throws(
    () => validateRuntimeHonestyTable([validIntents[0]!], table),
    /value-branch honesty surface .* requires a runtime value change row/u,
    "value-branch honesty selftest must reject a missing selection-outcome-changing row",
  );
}

export function runDeclaredRustSemverCheck(
  options: RunDeclaredRustSemverCheckOptions,
): DeclaredRustSemverCheckResult {
  const register = readRegister(options.repoRoot);
  const intents = register.intents.filter((intent) => intent.crate === options.crate);
  assert.ok(intents.length <= 1, `duplicate semver intents for ${options.crate}`);

  const args = [
    "semver-checks",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    options.crate,
    ...options.baselineArgs,
    ...(options.allFeatures ? ["--all-features"] : []),
    "--release-type",
    "patch",
    "--color",
    "never",
  ];
  const intent = intents[0];
  if (!intent) {
    execFileSync("cargo", args, { cwd: options.repoRoot, stdio: "inherit" });
    return {
      policy: "steady-state-patch",
      declaredFailureCount: 0,
      declaredWarningCount: 0,
      declaredRuntimeValueChangeCount: 0,
      declaredReleaseVersion: null,
    };
  }

  assertReleaseWindow(
    register.baselineWorkspaceVersion,
    register.targetReleaseVersion,
    options.workspaceVersion,
  );
  assert.ok(intent.reason.trim().length > 0, `${options.crate} semver intent requires a reason`);
  const runtimeValueChanges = intent.expectedRuntimeValueChanges ?? [];
  assert.ok(
    intent.expectedFailures.length > 0 || runtimeValueChanges.length > 0,
    `${options.crate} breaking semver intent must name a cargo diagnostic or runtime value change`,
  );
  validateRustSemverIntentRegister(options.repoRoot);

  const patchCheck = spawnSync("cargo", args, {
    cwd: options.repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const output = `${patchCheck.stdout ?? ""}${patchCheck.stderr ?? ""}`;
  process.stdout.write(output);
  if (intent.expectedFailures.length > 0) {
    assert.notEqual(
      patchCheck.status,
      0,
      `${options.crate} semver intent is stale because the steady-state patch check passed`,
    );
  } else {
    assert.equal(
      patchCheck.status,
      0,
      `${options.crate} runtime-value-only intent has undeclared cargo-semver-checks failures`,
    );
  }

  const diagnosticSections = parseDiagnosticSections(output);
  const observedFailures = diagnosticSections
    .filter((section) => section.kind === "failure")
    .map((section) => section.lint);
  const expectedFailures = intent.expectedFailures.map((failure) => failure.lint);
  assert.deepEqual(
    [...new Set(observedFailures)].sort(),
    [...new Set(expectedFailures)].sort(),
    `${options.crate} cargo-semver-checks failure set drifted from the declared intent`,
  );
  assert.equal(
    observedFailures.length,
    new Set(observedFailures).size,
    `${options.crate} cargo-semver-checks emitted duplicate failure sections`,
  );
  const expectedWarnings = intent.expectedWarnings ?? [];
  const observedWarnings = diagnosticSections
    .filter((section) => section.kind === "warning")
    .map((section) => section.lint);
  assert.deepEqual(
    [...new Set(observedWarnings)].sort(),
    [...new Set(expectedWarnings.map((warning) => warning.lint))].sort(),
    `${options.crate} cargo-semver-checks warning set drifted from the declared intent`,
  );
  assert.equal(
    observedWarnings.length,
    new Set(observedWarnings).size,
    `${options.crate} cargo-semver-checks emitted duplicate warning sections`,
  );

  const summary = output.match(
    /Summary semver requires new (major|minor) version: (\d+) major and (\d+) minor checks failed/u,
  );
  if (intent.expectedFailures.length > 0) {
    assert.ok(
      summary,
      `${options.crate} cargo-semver-checks output is missing the failure summary`,
    );
    const expectedMajorFailureCount = intent.expectedFailures.filter(
      (failure) => (failure.level ?? "major") === "major",
    ).length;
    const expectedMinorFailureCount = intent.expectedFailures.filter(
      (failure) => failure.level === "minor",
    ).length;
    const requiredReleaseWord = summary[1];
    const observedMajorFailureCount = Number(summary[2]);
    const observedMinorFailureCount = Number(summary[3]);
    assert.ok(
      (requiredReleaseWord === "major" && observedMajorFailureCount > 0) ||
        (requiredReleaseWord === "minor" &&
          observedMajorFailureCount === 0 &&
          observedMinorFailureCount > 0),
      `${options.crate} cargo-semver-checks release requirement disagrees with its failure counts`,
    );
    assert.equal(
      observedMajorFailureCount,
      expectedMajorFailureCount,
      `${options.crate} major failure count drifted`,
    );
    assert.equal(
      observedMinorFailureCount,
      expectedMinorFailureCount,
      `${options.crate} minor failure count drifted`,
    );
  } else {
    assert.equal(
      summary,
      null,
      `${options.crate} runtime-value-only intent emitted undeclared failures`,
    );
  }
  const warningSummary = output.match(
    /Warning produced (\d+) major and (\d+) minor level warnings/u,
  );
  if (expectedWarnings.length > 0) {
    assert.ok(warningSummary, `${options.crate} cargo-semver-checks warning summary is missing`);
    assert.equal(
      Number(warningSummary[1]),
      expectedWarnings.length,
      `${options.crate} major warning count drifted`,
    );
    assert.equal(Number(warningSummary[2]), 0, `${options.crate} has undeclared minor warnings`);
  } else {
    assert.equal(
      warningSummary,
      null,
      `${options.crate} emitted warnings without a declared warning policy`,
    );
  }

  for (const diagnostic of [...intent.expectedFailures, ...expectedWarnings]) {
    const section = diagnosticSections.find(
      (candidate) =>
        candidate.kind === (intent.expectedFailures.includes(diagnostic) ? "failure" : "warning") &&
        candidate.lint === diagnostic.lint,
    );
    assert.ok(section, `${options.crate} ${diagnostic.lint} diagnostic section is missing`);
    assert.ok(
      diagnostic.evidenceNeedles.length > 0,
      `${options.crate} ${diagnostic.lint} intent requires evidence needles`,
    );
    for (const needle of diagnostic.evidenceNeedles) {
      assert.ok(
        section.body.includes(needle),
        `${options.crate} ${diagnostic.lint} output is missing declared evidence ${JSON.stringify(needle)}`,
      );
    }
    if (diagnostic.witnessLinePrefix !== undefined) {
      assert.ok(
        diagnostic.expectedWitnesses !== undefined,
        `${options.crate} ${diagnostic.lint} witness prefix requires an expected witness set`,
      );
      const observedWitnesses = section.body
        .split("\n")
        .filter((line) => line.startsWith(diagnostic.witnessLinePrefix!))
        .map((line) => line.slice(diagnostic.witnessLinePrefix!.length).split(" in ")[0]);
      assert.deepEqual(
        observedWitnesses.toSorted(),
        [...diagnostic.expectedWitnesses].toSorted(),
        `${options.crate} ${diagnostic.lint} witness set drifted`,
      );
    }
  }

  return {
    policy: "declared-pre1-minor-breaking",
    declaredFailureCount: intent.expectedFailures.length,
    declaredWarningCount: expectedWarnings.length,
    declaredRuntimeValueChangeCount: runtimeValueChanges.length,
    declaredReleaseVersion: register.targetReleaseVersion,
  };
}

interface CargoSemverDiagnosticSection {
  readonly kind: "failure" | "warning";
  readonly lint: string;
  readonly body: string;
}

function parseDiagnosticSections(output: string): readonly CargoSemverDiagnosticSection[] {
  const matches = [...output.matchAll(diagnosticHeaderPattern)];
  return matches.map((match, index) => {
    const kind = match[1];
    const lint = match[2];
    assert.ok(kind === "failure" || kind === "warning");
    assert.ok(lint);
    const bodyStart = (match.index ?? 0) + match[0].length;
    const bodyEnd = matches[index + 1]?.index ?? output.length;
    return {
      kind,
      lint,
      body: output.slice(bodyStart, bodyEnd),
    };
  });
}

function readRegister(repoRoot: string): RustSemverIntentRegister {
  const register = JSON.parse(
    readFileSync(path.join(repoRoot, registerRelativePath), "utf8"),
  ) as RustSemverIntentRegister;
  assert.equal(register.schemaVersion, "0", `${registerRelativePath} schemaVersion`);
  assert.equal(register.product, "omena-rust-semver-intent", `${registerRelativePath} product`);
  assert.ok(Array.isArray(register.intents), `${registerRelativePath} intents`);
  assert.equal(
    new Set(register.intents.map((intent) => intent.crate)).size,
    register.intents.length,
    `${registerRelativePath} must contain at most one intent per crate`,
  );
  return register;
}

function readAuthoredTextEgressHonestyTable(repoRoot: string): AuthoredTextEgressHonestyTable {
  const census = JSON.parse(
    readFileSync(path.join(repoRoot, "rust/omena-identifier-authority-census.json"), "utf8"),
  ) as {
    readonly propertyIdentity?: {
      readonly egressHonestyTable?: AuthoredTextEgressHonestyTable;
    };
  };
  const table = census.propertyIdentity?.egressHonestyTable;
  assert.ok(
    table,
    "identifier-authority census must record the authored-text egress honesty table",
  );
  assert.equal(
    table.derivation,
    "final-head-byte-differential",
    "authored-text egress honesty table derivation",
  );
  assert.ok(Array.isArray(table.orderBranchSurfaces), "honesty order-branch surfaces");
  assert.ok(Array.isArray(table.valueBranchSurfaces), "honesty value-branch surfaces");
  assert.ok(Array.isArray(table.zeroBranchSurfaces), "honesty zero-branch surfaces");
  return table;
}

function readWorkspaceVersion(repoRoot: string): string {
  const source = readFileSync(path.join(repoRoot, "rust/Cargo.toml"), "utf8");
  const match = source.match(/\[workspace\.package\][\s\S]*?\bversion\s*=\s*"([^"]+)"/u);
  assert.ok(match, "rust/Cargo.toml must define workspace.package.version");
  return match[1]!;
}

function assertReleaseWindow(baseline: string, target: string, current: string): void {
  const baselineParts = parseVersion(baseline);
  const targetParts = parseVersion(target);
  assert.equal(baselineParts.major, 0, "pre-1.0 semver intent requires a 0.x baseline");
  assert.deepEqual(
    targetParts,
    {
      major: 0,
      minor: baselineParts.minor + 1,
      patch: 0,
    },
    "declared release version must advance the baseline 0.x minor exactly once",
  );
  assert.ok(
    current === baseline || current === target,
    `${registerRelativePath} applies only while the workspace is ${baseline} or ${target}`,
  );
}

function parseVersion(version: string): {
  readonly major: number;
  readonly minor: number;
  readonly patch: number;
} {
  const match = /^(\d+)\.(\d+)\.(\d+)$/u.exec(version);
  assert.ok(match, `unsupported workspace version ${version}`);
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
  };
}

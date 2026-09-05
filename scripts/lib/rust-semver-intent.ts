import { execFileSync, spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { maskRustCommentsAndLiterals, matchingRustDelimiter } from "./rust-write-authority";

interface ExpectedCargoSemverDiagnostic {
  readonly lint: string;
  readonly level?: "major" | "minor";
  readonly evidenceNeedles: readonly string[];
  readonly witnessLinePrefix?: string;
  readonly expectedWitnesses?: readonly string[];
  readonly expectedWitnessesByFeaturePlane?: {
    readonly defaultFeatures: readonly string[];
    readonly allFeatures: readonly string[];
  };
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
    readonly struct?: string;
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

interface PublishedRustReleaseBaseline {
  readonly version: string;
  readonly tag: string;
}

interface PublishedRustReleaseBaselineRegister {
  readonly schemaVersion: "0";
  readonly product: "omena-published-rust-release-baselines";
  readonly releases: readonly PublishedRustReleaseBaseline[];
}

interface RuntimeHonestySurface {
  readonly owner: string;
  readonly changeId: string;
  readonly surface: string;
  readonly evidenceNeedles: readonly string[];
}

interface AuthoredTextEgressHonestyTable {
  readonly basis: "registered-zero-branch-evidence-gates";
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
  readonly featurePlane: "default-features" | "all-features";
  readonly declaredWitnessCount: number;
}

const registerRelativePath = "rust/omena-rust-semver-intent.json";
const publishedBaselineRelativePath = "rust/omena-published-release-baselines.json";
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
  readonly lifecycleSelftestMutationCount: 3;
} {
  const register = readRegister(repoRoot);
  const publishedBaselines = readPublishedBaselines(repoRoot);
  const workspaceVersion = readWorkspaceVersion(repoRoot);
  assertReleaseLifecycle(register, publishedBaselines, workspaceVersion);

  let runtimeValueChangeCount = 0;
  for (const intent of register.intents) {
    assert.ok(intent.reason.trim().length > 0, `${intent.crate} semver intent requires a reason`);
    for (const diagnostic of [...intent.expectedFailures, ...(intent.expectedWarnings ?? [])]) {
      validateDiagnosticWitnessPolicy(intent.crate, diagnostic);
    }
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
            hasRuntimeEvidence(source, needle, evidence.struct),
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
  runReleaseLifecycleSelftest();

  return {
    intentCount: register.intents.length,
    runtimeValueChangeCount,
    ...honestyCounts,
    honestySelftestMutationCount: 2,
    lifecycleSelftestMutationCount: 3,
  };
}

export function hasRuntimeEvidence(source: string, needle: string, struct?: string): boolean {
  const structural = maskRustCommentsAndLiterals(source);
  const field = /^(?:pub\s+)?[a-z_][a-z0-9_]*\s*:\s*[A-Za-z_][A-Za-z0-9_:]*$/u.test(needle);
  if (field) {
    assert.ok(
      struct && /^[A-Za-z_][A-Za-z0-9_]*$/u.test(struct),
      "field evidence requires a struct declaration operand",
    );
    const declarations = [
      ...structural.matchAll(new RegExp("\\bstruct\\s+" + struct + "\\s*\\{", "gu")),
    ];
    if (declarations.length !== 1) return false;
    const declaration = declarations[0]!;
    const open = declaration.index + declaration[0].lastIndexOf("{");
    const close = matchingRustDelimiter(structural, open, "{", "}");
    const body = structural.slice(open + 1, close);
    const escaped = needle.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    return new RegExp("^\\s*" + escaped + "\\s*,?\\s*$", "mu").test(body);
  }
  // A source needle must begin in executable/declarative Rust, never a comment,
  // doctest, or a string containing a transcription of the claimed evidence.
  const activeSource = maskRustCommentsAndLiterals(source, true);
  let start = activeSource.indexOf(needle);
  while (start >= 0) {
    if (needle.startsWith('"') || structural.slice(start, start + 1) === needle.slice(0, 1))
      return true;
    start = activeSource.indexOf(needle, start + needle.length);
  }
  return false;
}

function validateDiagnosticWitnessPolicy(
  crate: string,
  diagnostic: ExpectedCargoSemverDiagnostic,
): void {
  const flatWitnesses = diagnostic.expectedWitnesses;
  const featurePlaneWitnesses = diagnostic.expectedWitnessesByFeaturePlane;
  const witnessPolicyCount =
    Number(flatWitnesses !== undefined) + Number(featurePlaneWitnesses !== undefined);

  if (diagnostic.witnessLinePrefix === undefined) {
    assert.equal(
      witnessPolicyCount,
      0,
      `${crate} ${diagnostic.lint} witness sets require a witness line prefix`,
    );
    return;
  }

  assert.equal(
    witnessPolicyCount,
    1,
    `${crate} ${diagnostic.lint} witness prefix requires exactly one witness policy`,
  );
  assert.ok(
    diagnostic.witnessLinePrefix.length > 0,
    `${crate} ${diagnostic.lint} witness line prefix must be non-empty`,
  );

  const witnessSets = featurePlaneWitnesses
    ? [
        ["default-features", featurePlaneWitnesses.defaultFeatures] as const,
        ["all-features", featurePlaneWitnesses.allFeatures] as const,
      ]
    : ([["shared", flatWitnesses!]] as const);
  for (const [featurePlane, witnesses] of witnessSets) {
    assert.ok(
      witnesses.length > 0,
      `${crate} ${diagnostic.lint} ${featurePlane} witness set must be non-empty`,
    );
    assert.equal(
      new Set(witnesses).size,
      witnesses.length,
      `${crate} ${diagnostic.lint} ${featurePlane} witness set must be unique`,
    );
    for (const witness of witnesses) {
      assert.ok(
        witness.trim().length > 0 && witness === witness.trim(),
        `${crate} ${diagnostic.lint} ${featurePlane} witnesses must be trimmed and non-empty`,
      );
    }
  }
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
    basis: "registered-zero-branch-evidence-gates",
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

  const featurePlane = options.allFeatures ? "all-features" : "default-features";
  const featureArgs = options.allFeatures
    ? ["--all-features"]
    : process.argv.includes("--inject-default-semver-feature-evasion")
      ? []
      : ["--only-explicit-features"];

  const args = [
    "semver-checks",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    options.crate,
    ...options.baselineArgs,
    ...featureArgs,
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
      featurePlane,
      declaredWitnessCount: 0,
    };
  }

  assertReleaseLifecycle(
    register,
    readPublishedBaselines(options.repoRoot),
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

  let declaredWitnessCount = 0;
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
      const expectedWitnesses =
        diagnostic.expectedWitnessesByFeaturePlane?.[
          featurePlane === "all-features" ? "allFeatures" : "defaultFeatures"
        ] ?? diagnostic.expectedWitnesses;
      assert.ok(
        expectedWitnesses !== undefined,
        `${options.crate} ${diagnostic.lint} witness prefix requires an expected witness set`,
      );
      const observedWitnesses = section.body
        .split("\n")
        .filter(
          (line) =>
            line.startsWith(diagnostic.witnessLinePrefix!) &&
            (line.includes(", previously in file ") || line.includes(" in file ")),
        )
        .map(
          (line) =>
            line
              .slice(diagnostic.witnessLinePrefix!.length)
              .split(", previously in file ")[0]!
              .split(" in file ")[0]!,
        );
      assert.deepEqual(
        observedWitnesses.toSorted(),
        [...expectedWitnesses].toSorted(),
        `${options.crate} ${diagnostic.lint} ${featurePlane} witness set drifted`,
      );
      declaredWitnessCount += expectedWitnesses.length;
    }
  }

  return {
    policy: "declared-pre1-minor-breaking",
    declaredFailureCount: intent.expectedFailures.length,
    declaredWarningCount: expectedWarnings.length,
    declaredRuntimeValueChangeCount: runtimeValueChanges.length,
    declaredReleaseVersion: register.targetReleaseVersion,
    featurePlane,
    declaredWitnessCount,
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

function readPublishedBaselines(repoRoot: string): PublishedRustReleaseBaselineRegister {
  const register = JSON.parse(
    readFileSync(path.join(repoRoot, publishedBaselineRelativePath), "utf8"),
  ) as PublishedRustReleaseBaselineRegister;
  assert.equal(register.schemaVersion, "0", `${publishedBaselineRelativePath} schemaVersion`);
  assert.equal(
    register.product,
    "omena-published-rust-release-baselines",
    `${publishedBaselineRelativePath} product`,
  );
  assert.ok(Array.isArray(register.releases), `${publishedBaselineRelativePath} releases`);
  assert.ok(register.releases.length > 0, `${publishedBaselineRelativePath} must not be empty`);
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
    table.basis,
    "registered-zero-branch-evidence-gates",
    "authored-text egress honesty table basis",
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

function assertReleaseLifecycle(
  intent: Pick<RustSemverIntentRegister, "baselineWorkspaceVersion" | "targetReleaseVersion">,
  published: PublishedRustReleaseBaselineRegister,
  current: string,
): void {
  const versions = published.releases.map((release) => release.version);
  assert.equal(
    new Set(versions).size,
    versions.length,
    `${publishedBaselineRelativePath} versions must be unique`,
  );
  for (const [index, release] of published.releases.entries()) {
    const parts = parseVersion(release.version);
    assert.equal(
      release.tag,
      `release-v${release.version}`,
      `${publishedBaselineRelativePath} tag must bind its exact version`,
    );
    if (index > 0) {
      const previous = parseVersion(published.releases[index - 1]!.version);
      assert.ok(
        parts.major > previous.major ||
          (parts.major === previous.major && parts.minor > previous.minor) ||
          (parts.major === previous.major &&
            parts.minor === previous.minor &&
            parts.patch > previous.patch),
        `${publishedBaselineRelativePath} releases must be strictly version-ordered`,
      );
    }
  }
  const latest = published.releases.at(-1)!;
  assert.equal(
    intent.baselineWorkspaceVersion,
    latest.version,
    `${registerRelativePath} baseline must rotate to the latest published release`,
  );
  assertReleaseWindow(intent.baselineWorkspaceVersion, intent.targetReleaseVersion, current);
}

function runReleaseLifecycleSelftest(): void {
  const published: PublishedRustReleaseBaselineRegister = {
    schemaVersion: "0",
    product: "omena-published-rust-release-baselines",
    releases: [
      { version: "0.4.0", tag: "release-v0.4.0" },
      { version: "0.5.0", tag: "release-v0.5.0" },
    ],
  };
  const rotated = {
    baselineWorkspaceVersion: "0.5.0",
    targetReleaseVersion: "0.6.0",
  } as const;
  assert.doesNotThrow(() => assertReleaseLifecycle(rotated, published, "0.5.0"));
  assert.throws(
    () =>
      assertReleaseLifecycle(
        { baselineWorkspaceVersion: "0.4.0", targetReleaseVersion: "0.5.0" },
        published,
        "0.5.0",
      ),
    /baseline must rotate to the latest published release/u,
    "post-publication lifecycle selftest must reject the completed train as the active window",
  );
  assert.throws(
    () =>
      assertReleaseLifecycle(
        rotated,
        {
          ...published,
          releases: [published.releases[0]!, { version: "0.5.0", tag: "release-v0.4.0" }],
        },
        "0.5.0",
      ),
    /tag must bind its exact version/u,
    "release lifecycle selftest must reject a tag/version split",
  );
  assert.throws(
    () =>
      assertReleaseLifecycle(
        { baselineWorkspaceVersion: "0.5.0", targetReleaseVersion: "0.7.0" },
        published,
        "0.5.0",
      ),
    /advance the baseline 0.x minor exactly once/u,
    "release lifecycle selftest must reject a skipped active minor window",
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

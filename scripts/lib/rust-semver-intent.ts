import { execFileSync, spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

interface ExpectedCargoSemverDiagnostic {
  readonly lint: string;
  readonly evidenceNeedles: readonly string[];
  readonly witnessLinePrefix?: string;
  readonly expectedWitnesses?: readonly string[];
}

interface RustSemverIntent {
  readonly crate: string;
  readonly releaseClass: "pre1MinorBreaking";
  readonly reason: string;
  readonly expectedFailures: readonly ExpectedCargoSemverDiagnostic[];
  readonly expectedWarnings?: readonly ExpectedCargoSemverDiagnostic[];
}

interface RustSemverIntentRegister {
  readonly schemaVersion: "0";
  readonly product: "omena-rust-semver-intent";
  readonly baselineWorkspaceVersion: string;
  readonly targetReleaseVersion: string;
  readonly intents: readonly RustSemverIntent[];
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
  readonly declaredReleaseVersion: string | null;
}

const registerRelativePath = "rust/omena-rust-semver-intent.json";
const diagnosticHeaderPattern = /^--- (failure|warning) ([a-z0-9_]+):[^\n]*$/gmu;

export function declaredRustSemverIntentCrates(repoRoot: string): readonly string[] {
  return readRegister(repoRoot).intents.map((intent) => intent.crate);
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
      declaredReleaseVersion: null,
    };
  }

  assertReleaseWindow(
    register.baselineWorkspaceVersion,
    register.targetReleaseVersion,
    options.workspaceVersion,
  );
  assert.ok(intent.reason.trim().length > 0, `${options.crate} semver intent requires a reason`);
  assert.ok(
    intent.expectedFailures.length > 0,
    `${options.crate} breaking semver intent must name at least one cargo-semver-checks failure`,
  );

  const patchCheck = spawnSync("cargo", args, {
    cwd: options.repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const output = `${patchCheck.stdout ?? ""}${patchCheck.stderr ?? ""}`;
  process.stdout.write(output);
  assert.notEqual(
    patchCheck.status,
    0,
    `${options.crate} semver intent is stale because the steady-state patch check passed`,
  );

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
    /Summary semver requires new major version: (\d+) major and (\d+) minor checks failed/u,
  );
  assert.ok(summary, `${options.crate} cargo-semver-checks output is missing the failure summary`);
  assert.equal(
    Number(summary[1]),
    intent.expectedFailures.length,
    `${options.crate} major failure count drifted`,
  );
  assert.equal(Number(summary[2]), 0, `${options.crate} has undeclared minor semver failures`);
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

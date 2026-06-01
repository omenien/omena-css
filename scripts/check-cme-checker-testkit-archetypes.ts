import { spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { strict as assert } from "node:assert";
import {
  assertCheckerCanonicalCandidateEqual,
  OMENA_CHECKER_TESTKIT_ARCHETYPES,
  deriveCheckerCanonicalCandidate,
  type CmeCheckerBundleV0,
  type CmeCheckerTestkitArchetypeV0,
} from "../packages/cme-checker/src";
import type { ContractParityEntry } from "./contract-parity-corpus-v1";
import { buildContractParitySnapshot } from "./contract-parity-runtime";
import {
  runShadowCheckerSourceMissingCanonicalCandidate,
  runShadowCheckerStyleRecoveryCanonicalCandidate,
  runShadowCheckerStyleUnusedCanonicalCandidate,
} from "./rust-shadow-shared";

interface ParsedFixtureV0 {
  readonly schemaVersion: "0";
  readonly files: readonly ParsedFixtureFileV0[];
  readonly expectations: readonly ParsedFixtureExpectationV0[];
}

interface ParsedFixtureFileV0 {
  readonly path: string;
  readonly metadata: readonly ParsedFixtureMetadataV0[];
  readonly markers: readonly unknown[];
  readonly source: string;
}

interface ParsedFixtureMetadataV0 {
  readonly key: string;
  readonly value: string;
}

interface ParsedFixtureExpectationV0 {
  readonly key: string;
  readonly value: string;
}

const SOURCE_MISSING_CODES = new Set([
  "missing-module",
  "missing-static-class",
  "missing-template-prefix",
  "missing-resolved-class-values",
  "missing-resolved-class-domain",
]);
const STYLE_RECOVERY_CODES = new Set([
  "missing-composed-module",
  "missing-composed-selector",
  "missing-value-module",
  "missing-imported-value",
  "missing-keyframes",
  "missing-sass-symbol",
]);
const STYLE_UNUSED_CODES = new Set(["unused-selector"]);

void (async () => {
  const reports = [];
  for (const archetype of OMENA_CHECKER_TESTKIT_ARCHETYPES) {
    // oxlint-disable-next-line no-await-in-loop
    reports.push(await validateArchetype(archetype));
  }

  process.stdout.write(
    JSON.stringify(
      {
        product: "cme-checker.testkit-archetypes",
        fixtureGrammar: "cme-fixture-v0",
        archetypeCount: reports.length,
        bundleCount: new Set(reports.map((report) => report.bundle)).size,
        reports,
      },
      null,
      2,
    ),
  );
  process.stdout.write("\n");
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

async function validateArchetype(archetype: CmeCheckerTestkitArchetypeV0) {
  const parsed = parseFixtureWithOmenaTestkit(archetype.fixture);
  const workspaceRoot = materializeFixtureWorkspace(parsed, archetype.label);
  try {
    assertExpectation(parsed, "code", archetype.expectedCode);
    const entry = contractEntryFromFixture(archetype, parsed, workspaceRoot);
    const snapshot = await buildContractParitySnapshot(entry);
    const expected = deriveCheckerCanonicalCandidate(snapshot, {
      bundle: archetype.bundle,
      category: archetype.category,
      codes: codesForBundle(archetype.bundle),
      extraFields: extraFieldsForBundle(archetype.bundle),
    });
    const actual = await runRustCanonicalCandidate(archetype.bundle, snapshot);
    assertCheckerCanonicalCandidateEqual(
      actual,
      expected,
      `${archetype.label}: cme-checker testkit archetype mismatch`,
    );
    assert.equal(
      actual.summary.total,
      1,
      `${archetype.label}: expected exactly one checker finding`,
    );
    assert.equal(
      actual.findings[0]?.code,
      archetype.expectedCode,
      `${archetype.label}: unexpected checker code`,
    );

    return {
      label: archetype.label,
      bundle: archetype.bundle,
      category: archetype.category,
      expectedCode: archetype.expectedCode,
      fileCount: parsed.files.length,
      metadataCount: parsed.files.reduce((sum, file) => sum + file.metadata.length, 0),
      markerCount: parsed.files.reduce((sum, file) => sum + file.markers.length, 0),
      findingCount: actual.summary.total,
    };
  } finally {
    rmSync(workspaceRoot, { recursive: true, force: true });
  }
}

function parseFixtureWithOmenaTestkit(raw: string): ParsedFixtureV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-testkit",
      "--bin",
      "omena-testkit-parse-fixture",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: raw,
      maxBuffer: 8 * 1024 * 1024,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.error, undefined);
  const parsed = JSON.parse(result.stdout) as ParsedFixtureV0;
  assert.equal(parsed.schemaVersion, "0");
  assert.ok(parsed.files.length > 0);
  assert.ok(parsed.expectations.length > 0);
  return parsed;
}

function materializeFixtureWorkspace(parsed: ParsedFixtureV0, label: string): string {
  const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), `${label}-`));
  for (const file of parsed.files) {
    assert.ok(!path.isAbsolute(file.path), `${label}: fixture path must be workspace-relative`);
    const targetPath = path.join(workspaceRoot, file.path);
    assert.ok(
      path.relative(workspaceRoot, targetPath).startsWith("..") === false,
      `${label}: fixture path escapes workspace`,
    );
    mkdirSync(path.dirname(targetPath), { recursive: true });
    writeFileSync(targetPath, file.source);
  }
  return workspaceRoot;
}

function contractEntryFromFixture(
  archetype: CmeCheckerTestkitArchetypeV0,
  parsed: ParsedFixtureV0,
  workspaceRoot: string,
): ContractParityEntry {
  const sourceFilePaths = parsed.files
    .filter((file) => fileIsSource(file))
    .map((file) => path.join(workspaceRoot, file.path));
  const styleFilePaths = parsed.files
    .filter((file) => fileIsStyle(file))
    .map((file) => path.join(workspaceRoot, file.path));

  assert.ok(sourceFilePaths.length > 0 || styleFilePaths.length > 0);

  return {
    label: archetype.label,
    workspace: {
      workspaceRoot,
      sourceFilePaths,
      styleFilePaths,
    },
    filters: {
      preset: archetype.category === "source" ? "changed-source" : "changed-style",
      category: archetype.category,
      severity: "all",
      includeBundles: [archetype.bundle],
      includeCodes: [],
      excludeCodes: [],
    },
  };
}

function fileIsSource(file: ParsedFixtureFileV0): boolean {
  const dialect = metadataValue(file, "dialect");
  return (
    dialect === "ts" ||
    dialect === "tsx" ||
    dialect === "js" ||
    dialect === "jsx" ||
    /\.(?:[cm]?[jt]sx?)$/.test(file.path)
  );
}

function fileIsStyle(file: ParsedFixtureFileV0): boolean {
  const dialect = metadataValue(file, "dialect");
  return (
    dialect === "css" ||
    dialect === "scss" ||
    dialect === "less" ||
    /\.(?:s?css|less)$/.test(file.path)
  );
}

function metadataValue(file: ParsedFixtureFileV0, key: string): string | undefined {
  return file.metadata.find((metadata) => metadata.key === key)?.value;
}

function assertExpectation(parsed: ParsedFixtureV0, key: string, value: string): void {
  assert.ok(
    parsed.expectations.some(
      (expectation) => expectation.key === key && expectation.value === value,
    ),
    `expected fixture expectation ${key}=${value}`,
  );
}

function codesForBundle(bundle: CmeCheckerBundleV0): ReadonlySet<string> {
  if (bundle === "source-missing") return SOURCE_MISSING_CODES;
  if (bundle === "style-recovery") return STYLE_RECOVERY_CODES;
  return STYLE_UNUSED_CODES;
}

function extraFieldsForBundle(bundle: CmeCheckerBundleV0): readonly string[] {
  if (bundle === "source-missing") {
    return ["analysisReason", "valueCertaintyShapeLabel", "valueDomainDerivation"];
  }
  return ["analysisReason", "valueCertaintyShapeLabel"];
}

async function runRustCanonicalCandidate(bundle: CmeCheckerBundleV0, snapshot: unknown) {
  if (bundle === "source-missing") {
    return runShadowCheckerSourceMissingCanonicalCandidate(snapshot);
  }
  if (bundle === "style-recovery") {
    return runShadowCheckerStyleRecoveryCanonicalCandidate(snapshot);
  }
  return runShadowCheckerStyleUnusedCanonicalCandidate(snapshot);
}

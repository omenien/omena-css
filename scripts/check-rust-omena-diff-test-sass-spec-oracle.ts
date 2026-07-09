import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

type SassDialectV0 = "scss" | "sass";
type ExternalCorpusExpectationKindV1 =
  | "static-must-match"
  | "expected-sound-bail"
  | "parser-recovery"
  | "out-of-scope";

interface PackageJsonV0 {
  readonly devDependencies?: Record<string, string>;
}

interface ImportedSassSpecChunkV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly chunkId: string;
  readonly sourcePin: string;
  readonly fixtures: readonly ImportedSassSpecFixtureV0[];
}

interface ImportedSassSpecFixtureV0 {
  readonly id: string;
  readonly upstreamPath: string;
  readonly dialect: SassDialectV0;
  readonly expectationKind: ExternalCorpusExpectationKindV1;
  readonly source: string;
  readonly expectedCss?: string;
  readonly expectedError?: string;
  readonly expectedWarning?: string;
}

interface DeclarationValuePairV0 {
  readonly property: string;
  readonly value: string;
}

interface SassSpecOracleCaptureV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly sourcePin: string;
  readonly compiler: {
    readonly name: "dart-sass";
    readonly package: "sass";
    readonly version: "1.101.0";
    readonly hostMode: "dart-sass-cli";
  };
  readonly chunkId: string;
  readonly fixtureCount: number;
  readonly records: readonly SassSpecOracleRecordV0[];
}

interface SassSpecOracleRecordV0 {
  readonly fixtureId: string;
  readonly upstreamPath: string;
  readonly dialect: SassDialectV0;
  readonly compiled: boolean;
  readonly cssSha256?: string;
  readonly declarationValuePairs?: readonly DeclarationValuePairV0[];
  readonly stderr?: string;
}

interface RunStatusResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/sass-spec-corpus");
const chunkPath = path.join(corpusRoot, "imported-smoke.json");
const capturePath = path.join(corpusRoot, "imported-smoke-oracle.json");
const packageJson = readJson<PackageJsonV0>(path.join(repoRoot, "package.json"));

assert.equal(packageJson.devDependencies?.sass, "1.101.0");
const dartSassVersion = run("pnpm", ["exec", "sass", "--version"]).stdout.trim();
assert.match(
  dartSassVersion,
  /^1\.101\.0\b/u,
  `dart-sass oracle must resolve to 1.101.0, got ${dartSassVersion}`,
);

const chunk = readJson<ImportedSassSpecChunkV0>(chunkPath);
assert.equal(chunk.schemaVersion, "0");
assert.equal(chunk.product, "omena-diff-test.sass-spec-imported-corpus.chunk");
assert.ok(chunk.fixtures.length > 0, "imported sass-spec chunk must not be empty");

const primaryCapture = captureChunk(chunk);
const primarySource = stableJson(primaryCapture);
const secondSource = stableJson(captureChunk(chunk));
assert.equal(
  sha256(primarySource),
  sha256(secondSource),
  "dart-sass oracle capture must be deterministic for the same imported chunk",
);

if (checkOnly) {
  assert.equal(readFileSync(capturePath, "utf8"), primarySource);
} else {
  writeFileSync(capturePath, primarySource);
}

process.stdout.write(
  stableJson({
    product: "omena-diff-test.sass-spec-dart-sass-oracle",
    mode: checkOnly ? "check" : "write",
    sourcePin: chunk.sourcePin,
    hostMode: "dart-sass-cli",
    fixtureCount: chunk.fixtures.length,
    captureSha256: sha256(primarySource),
    recordCount: primaryCapture.records.length,
    compiledCount: primaryCapture.records.filter((record) => record.compiled).length,
    generatedFiles: [path.basename(capturePath)],
  }),
);

function captureChunk(chunk: ImportedSassSpecChunkV0): SassSpecOracleCaptureV0 {
  return {
    schemaVersion: "0",
    product: "omena-diff-test.sass-spec-dart-sass-oracle-capture",
    sourcePin: chunk.sourcePin,
    compiler: {
      name: "dart-sass",
      package: "sass",
      version: "1.101.0",
      hostMode: "dart-sass-cli",
    },
    chunkId: chunk.chunkId,
    fixtureCount: chunk.fixtures.length,
    records: chunk.fixtures.map(captureFixture),
  };
}

function captureFixture(fixture: ImportedSassSpecFixtureV0): SassSpecOracleRecordV0 {
  if (fixture.expectationKind === "out-of-scope") {
    return {
      fixtureId: fixture.id,
      upstreamPath: fixture.upstreamPath,
      dialect: fixture.dialect,
      compiled: false,
      stderr: "excluded: fixture requires the upstream suite layout",
    };
  }

  const outputRoot = mkdtempSync(path.join(tmpdir(), "omena-sass-spec-oracle-"));
  const inputPath = path.join(outputRoot, `input.${fixture.dialect}`);
  const outputPath = path.join(outputRoot, "output.css");
  try {
    writeFileSync(inputPath, fixture.source);
    const result = runStatus("pnpm", [
      "exec",
      "sass",
      "--no-source-map",
      "--style",
      "expanded",
      inputPath,
      outputPath,
    ]);
    if (result.status !== 0) {
      assert.ok(fixture.expectedError !== undefined, `${fixture.id} unexpectedly failed`);
      return {
        fixtureId: fixture.id,
        upstreamPath: fixture.upstreamPath,
        dialect: fixture.dialect,
        compiled: false,
        stderr: normalizeCompilerStderr(result.stderr, outputRoot),
      };
    }

    const css = readFileSync(outputPath, "utf8");
    if (fixture.expectedCss !== undefined) {
      assert.equal(
        normalizeCssForOracleComparison(css),
        normalizeCssForOracleComparison(fixture.expectedCss),
        `${fixture.id} output.css mismatch`,
      );
    }
    assert.equal(fixture.expectedError, undefined, `${fixture.id} expected an error`);
    return {
      fixtureId: fixture.id,
      upstreamPath: fixture.upstreamPath,
      dialect: fixture.dialect,
      compiled: true,
      cssSha256: sha256(css),
      declarationValuePairs: collectDeclarationValuePairs(css),
    };
  } finally {
    rmSync(outputRoot, { force: true, recursive: true });
  }
}

function collectDeclarationValuePairs(css: string): DeclarationValuePairV0[] {
  const pairs: DeclarationValuePairV0[] = [];
  const declarationPattern = /(?<property>[-_a-zA-Z][-_a-zA-Z0-9]*)\s*:\s*(?<value>[^;{}]+);/gu;
  for (const match of css.matchAll(declarationPattern)) {
    const property = match.groups?.property;
    const value = match.groups?.value;
    assert.ok(property, `missing declaration property in ${match[0]}`);
    assert.ok(value, `missing declaration value in ${match[0]}`);
    pairs.push({
      property,
      value: value.trim().replace(/\s+/gu, " "),
    });
  }
  return pairs.sort((left, right) => {
    const propertyOrder = left.property.localeCompare(right.property);
    if (propertyOrder !== 0) {
      return propertyOrder;
    }
    return left.value.localeCompare(right.value);
  });
}

function normalizeCssForOracleComparison(css: string): string {
  return `${css.trimEnd()}\n`;
}

function normalizeCompilerStderr(stderr: string, outputRoot: string): string {
  return stderr.replaceAll(outputRoot, "<oracle-workdir>");
}

function run(command: string, args: readonly string[]): RunStatusResult {
  const result = runStatus(command, args);
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return result;
}

function runStatus(command: string, args: readonly string[]): RunStatusResult {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 16,
  });
  return {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function stableJson(value: unknown): string {
  return `${inlineStringArrays(JSON.stringify(value, null, 2))}\n`;
}

function inlineStringArrays(source: string): string {
  return source.replace(/\[\n((?:\s+"(?:[^"\\]|\\.)*"(?:,\n)?)+)\s+\]/g, (_match, body) => {
    const values = String(body)
      .trim()
      .split(/\n/)
      .map((line) => line.trim().replace(/,$/, ""));
    const inlineArray = `[${values.join(", ")}]`;
    return inlineArray.length <= 80 ? inlineArray : String(_match);
  });
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

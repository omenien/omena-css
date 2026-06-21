import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

type Dialect = "scss" | "sass" | "less";
type ExternalCompiler = "dart-sass" | "lessc";
type DivergenceKind = "none" | "compilerError" | "nativeUnavailable" | "valueMismatch";

interface ExternalDifferentialManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly mode: string;
  readonly lastVerified: string;
  readonly compilers: {
    readonly dartSassPackage: string;
    readonly dartSassVersion: string;
    readonly lesscPackage: string;
    readonly lesscVersion: string;
  };
  readonly fixtures: readonly ExternalDifferentialFixtureV0[];
}

interface ExternalDifferentialFixtureV0 {
  readonly id: string;
  readonly dialect: Dialect;
  readonly compiler: ExternalCompiler;
  readonly source: string;
  readonly expectedDivergence: DivergenceKind;
}

interface PackageJsonV0 {
  readonly devDependencies?: Record<string, string>;
}

interface OmenaBuildSummaryV0 {
  readonly execution?: {
    readonly cssModuleEvaluation?: {
      readonly productOutputSource?: string;
      readonly nativeEditOutput?: string;
    };
  };
}

interface DeclarationValuePairV0 {
  readonly property: string;
  readonly value: string;
}

interface ExternalDifferentialRecordV0 {
  readonly fixtureId: string;
  readonly dialect: Dialect;
  readonly compiler: ExternalCompiler;
  readonly compilerCompiled: boolean;
  readonly nativeEvaluationAvailable: boolean;
  readonly compilerValuePairs: readonly DeclarationValuePairV0[];
  readonly nativeValuePairs: readonly DeclarationValuePairV0[];
  readonly comparedValuePairCount: number;
  readonly divergence: DivergenceKind;
}

interface RunStatusResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

const repoRoot = process.cwd();
const corpusRoot = path.join(
  repoRoot,
  "rust/crates/omena-diff-test/static-stylesheet-external-differential",
);
const manifest = readJson<ExternalDifferentialManifestV0>(path.join(corpusRoot, "manifest.json"));
const packageJson = readJson<PackageJsonV0>(path.join(repoRoot, "package.json"));

assert.equal(manifest.schemaVersion, "0");
assert.equal(
  manifest.product,
  "omena-scss-eval.static-stylesheet-external-differential-corpus",
);
assert.equal(manifest.mode, "externalDifferential");
assert.equal(manifest.lastVerified, "2026-06-21");
assert.ok(manifest.fixtures.length > 0, "external differential corpus must not be empty");
assert.equal(packageJson.devDependencies?.sass, "^1.100.0");
assert.equal(packageJson.devDependencies?.less, "4.6.4");

const dartSassVersion = run("pnpm", ["exec", "sass", "--version"]).stdout.trim();
const lesscVersion = run("pnpm", ["exec", "lessc", "--version"]).stdout.trim();
assert.match(
  dartSassVersion,
  /^1\.100\.0\b/u,
  `dart-sass oracle must resolve to 1.100.0, got ${dartSassVersion}`,
);
assert.match(
  lesscVersion,
  /^lessc 4\.6\.4\b/u,
  `lessc oracle must resolve to 4.6.4, got ${lesscVersion}`,
);

const records = manifest.fixtures.map(evaluateFixture);
const divergenceCount = records.filter((record) => record.divergence !== "none").length;
for (const record of records) {
  const expected = manifest.fixtures.find((fixture) => fixture.id === record.fixtureId);
  assert.equal(
    record.divergence,
    expected?.expectedDivergence,
    `${record.fixtureId} expected ${expected?.expectedDivergence}, got ${record.divergence}`,
  );
}

const report = {
  schemaVersion: "0",
  product: "omena-scss-eval.static-stylesheet-external-differential",
  mode: "externalDifferential",
  lastVerified: manifest.lastVerified,
  dartSassVersion,
  lesscVersion,
  fixtureCount: records.length,
  comparedFixtureCount: records.filter((record) => record.comparedValuePairCount > 0).length,
  comparedValuePairCount: records.reduce(
    (count, record) => count + record.comparedValuePairCount,
    0,
  ),
  divergenceCount,
  records,
};

assert.equal(divergenceCount, 0, JSON.stringify(report, null, 2));
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);

function evaluateFixture(fixture: ExternalDifferentialFixtureV0): ExternalDifferentialRecordV0 {
  const outputRoot = mkdtempSync(path.join(tmpdir(), "omena-scss-eval-external-"));
  const inputPath = path.join(outputRoot, `input.${fixture.dialect}`);
  const outputPath = path.join(outputRoot, "external.css");
  try {
    writeFileSync(inputPath, fixture.source);
    const compilerResult = runExternalCompiler(fixture, inputPath, outputPath);
    const compilerCompiled = compilerResult.status === 0;
    const compilerCss = compilerCompiled ? readFileSync(outputPath, "utf8") : "";
    const nativeOutput = readOmenaNativeEditOutput(inputPath);
    const nativeEvaluationAvailable = nativeOutput !== undefined;
    const compilerValuePairs = compilerCompiled
      ? collectDeclarationValuePairs(compilerCss)
      : [];
    const nativeValuePairs =
      nativeOutput === undefined ? [] : collectDeclarationValuePairs(nativeOutput);
    const comparedValuePairCount = Math.min(compilerValuePairs.length, nativeValuePairs.length);
    const divergence = classifyDivergence(
      compilerCompiled,
      nativeEvaluationAvailable,
      compilerValuePairs,
      nativeValuePairs,
    );

    return {
      fixtureId: fixture.id,
      dialect: fixture.dialect,
      compiler: fixture.compiler,
      compilerCompiled,
      nativeEvaluationAvailable,
      compilerValuePairs,
      nativeValuePairs,
      comparedValuePairCount,
      divergence,
    };
  } finally {
    rmSync(outputRoot, { force: true, recursive: true });
  }
}

function runExternalCompiler(
  fixture: ExternalDifferentialFixtureV0,
  inputPath: string,
  outputPath: string,
): RunStatusResult {
  if (fixture.compiler === "lessc") {
    return runStatus("pnpm", ["exec", "lessc", inputPath, outputPath]);
  }
  return runStatus("pnpm", [
    "exec",
    "sass",
    "--no-source-map",
    "--style",
    "expanded",
    inputPath,
    outputPath,
  ]);
}

function readOmenaNativeEditOutput(inputPath: string): string | undefined {
  const summary = JSON.parse(
    run(
      "cargo",
      [
        "run",
        "--quiet",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "omena-cli",
        "--bin",
        "omena-cli",
        "--",
        "build",
        inputPath,
        "--json",
      ],
      1024 * 1024 * 32,
    ).stdout,
  ) as OmenaBuildSummaryV0;
  const evaluation = summary.execution?.cssModuleEvaluation;
  assert.equal(evaluation?.productOutputSource, "nativeEditOutput");
  return evaluation.nativeEditOutput;
}

function classifyDivergence(
  compilerCompiled: boolean,
  nativeEvaluationAvailable: boolean,
  compilerValuePairs: readonly DeclarationValuePairV0[],
  nativeValuePairs: readonly DeclarationValuePairV0[],
): DivergenceKind {
  if (!compilerCompiled) {
    return "compilerError";
  }
  if (!nativeEvaluationAvailable) {
    return "nativeUnavailable";
  }
  if (JSON.stringify(compilerValuePairs) !== JSON.stringify(nativeValuePairs)) {
    return "valueMismatch";
  }
  return "none";
}

function collectDeclarationValuePairs(css: string): DeclarationValuePairV0[] {
  const pairs: DeclarationValuePairV0[] = [];
  const declarationPattern =
    /(?<property>[-_a-zA-Z][-_a-zA-Z0-9]*)\s*:\s*(?<value>[^;{}]+);/gu;
  for (const match of css.matchAll(declarationPattern)) {
    const property = match.groups?.property;
    const value = match.groups?.value;
    assert.ok(property, `missing declaration property in ${match[0]}`);
    assert.ok(value, `missing declaration value in ${match[0]}`);
    pairs.push({
      property,
      value: canonicalizeDeclarationValue(value),
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

function canonicalizeDeclarationValue(value: string): string {
  return value.trim().replace(/\s+/gu, " ");
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function run(
  command: string,
  args: readonly string[],
  maxBuffer = 1024 * 1024,
): { readonly stdout: string } {
  const result = runStatus(command, args, maxBuffer);
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} exited ${result.status}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}

function runStatus(
  command: string,
  args: readonly string[],
  maxBuffer = 1024 * 1024,
): RunStatusResult {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer,
  });
  if (result.error) {
    throw result.error;
  }
  return {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

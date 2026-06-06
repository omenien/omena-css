import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

type DivergenceKind = "none" | "typeA" | "typeB" | "typeC";

interface SassDifferentialManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly mode: string;
  readonly fixtures: readonly SassDifferentialFixtureV0[];
}

interface SassDifferentialFixtureV0 {
  readonly id: string;
  readonly entrypoint: string;
  readonly sources: readonly string[];
  readonly expectedDivergence: DivergenceKind;
}

interface PackageJsonV0 {
  readonly devDependencies?: Record<string, string>;
}

interface StyleDiagnosticsSummary {
  readonly diagnostics: readonly StyleDiagnostic[];
}

interface StyleDiagnostic {
  readonly code: string;
  readonly message: string;
}

interface DifferentialRecord {
  readonly fixtureId: string;
  readonly dartSassCompiled: boolean;
  readonly missingSassSymbolCount: number;
  readonly divergence: DivergenceKind;
}

const repoRoot = process.cwd();
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/sass-differential");
const manifest = readJson<SassDifferentialManifestV0>(path.join(corpusRoot, "manifest.json"));
const packageJson = readJson<PackageJsonV0>(path.join(repoRoot, "package.json"));

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-diff-test.sass-differential-corpus");
assert.equal(manifest.mode, "sass-compilability");
assert.ok(manifest.fixtures.length > 0, "sass differential corpus must not be empty");
assert.match(
  packageJson.devDependencies?.sass ?? "",
  /^\^?1\./u,
  "dart-sass oracle must be pinned to the 1.x package line",
);

const sassVersion = run("pnpm", ["exec", "sass", "--version"]).stdout.trim();
assert.match(sassVersion, /^1\./u, `dart-sass executable must resolve to 1.x, got ${sassVersion}`);

const records = manifest.fixtures.map((fixture) => evaluateFixture(fixture));
const typeACount = records.filter((record) => record.divergence === "typeA").length;
assert.equal(
  typeACount,
  0,
  `dart-sass compiled ${typeACount} fixture(s) where omena still emitted missingSassSymbol`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-diff-test.sass-differential",
      dartSassVersion: sassVersion,
      fixtureCount: records.length,
      divergenceCounts: countDivergences(records),
      records,
    },
    null,
    2,
  )}\n`,
);

function evaluateFixture(fixture: SassDifferentialFixtureV0): DifferentialRecord {
  assertFixturePath(fixture.entrypoint, `${fixture.id} entrypoint`);
  for (const source of fixture.sources) {
    assertFixturePath(source, `${fixture.id} source`);
  }

  const entrypoint = path.join(corpusRoot, fixture.entrypoint);
  const sourcePaths = fixture.sources.map((source) => path.join(corpusRoot, source));
  const outputRoot = mkdtempSync(path.join(tmpdir(), "omena-sass-differential-"));
  try {
    const sassResult = runStatus("pnpm", [
      "exec",
      "sass",
      "--no-source-map",
      entrypoint,
      path.join(outputRoot, `${fixture.id}.css`),
    ]);
    const dartSassCompiled = sassResult.status === 0;
    const summary = runStyleDiagnostics(entrypoint, sourcePaths);
    const missingSassSymbols = summary.diagnostics.filter(
      (diagnostic) => diagnostic.code === "missingSassSymbol",
    );
    const divergence = classifyDivergence(dartSassCompiled, missingSassSymbols.length);

    assert.equal(
      divergence,
      fixture.expectedDivergence,
      `${fixture.id} expected ${fixture.expectedDivergence}, got ${divergence}\nsass stderr=${sassResult.stderr}\ndiagnostics=${summary.diagnostics
        .map((diagnostic) => `${diagnostic.code}: ${diagnostic.message}`)
        .join("\n")}`,
    );

    return {
      fixtureId: fixture.id,
      dartSassCompiled,
      missingSassSymbolCount: missingSassSymbols.length,
      divergence,
    };
  } finally {
    rmSync(outputRoot, { force: true, recursive: true });
  }
}

function classifyDivergence(
  dartSassCompiled: boolean,
  missingSassSymbolCount: number,
): DivergenceKind {
  if (dartSassCompiled && missingSassSymbolCount === 0) {
    return "none";
  }
  if (dartSassCompiled) {
    return "typeA";
  }
  if (missingSassSymbolCount === 0) {
    return "typeB";
  }
  return "typeC";
}

function countDivergences(
  differentialRecords: readonly DifferentialRecord[],
): Record<DivergenceKind, number> {
  const counts: Record<DivergenceKind, number> = {
    none: 0,
    typeA: 0,
    typeB: 0,
    typeC: 0,
  };
  for (const record of differentialRecords) {
    counts[record.divergence] += 1;
  }
  return counts;
}

function runStyleDiagnostics(
  entrypoint: string,
  sourcePaths: readonly string[],
): StyleDiagnosticsSummary {
  const args = [
    "style-diagnostics",
    entrypoint,
    ...sourcePaths.flatMap((sourcePath) => ["--source", sourcePath]),
    "--json",
  ];
  return JSON.parse(
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
        ...args,
      ],
      1024 * 1024 * 64,
    ).stdout,
  ) as StyleDiagnosticsSummary;
}

function assertFixturePath(relativePath: string, label: string): void {
  assert.ok(!path.isAbsolute(relativePath), `${label} must be relative`);
  assert.ok(!relativePath.includes(".."), `${label} must stay inside the corpus root`);
  assert.ok(
    existsSync(path.join(corpusRoot, relativePath)),
    `${label} does not exist: ${relativePath}`,
  );
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function run(
  command: string,
  args: readonly string[],
  maxBuffer = 1024 * 1024,
): { stdout: string } {
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
): {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
} {
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

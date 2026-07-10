import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { strict as assert } from "node:assert";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Surface = "napi" | "wasm" | "cli";

interface Fixture {
  readonly id: string;
  readonly sourcePath: string;
  readonly logicalPath: string;
  readonly source: string;
  readonly sha256: string;
}

interface FixtureOutput {
  readonly fixtureId: string;
  readonly outputs: Readonly<Record<Surface, unknown>>;
}

interface ParityBaseline {
  readonly schemaVersion: "0";
  readonly product: "omena-sdk.cross-surface-parity";
  readonly captureCommit: string;
  readonly fixtures: readonly Omit<Fixture, "source">[];
  readonly coverage: {
    readonly coveredSurfaces: readonly string[];
    readonly coveredWorkflows: readonly string[];
    readonly uncoveredSurfaces: readonly string[];
    readonly uncoveredWorkflows: readonly string[];
    readonly uncoveredCountCeiling: number;
  };
  readonly knownDivergences: readonly {
    readonly id: string;
    readonly description: string;
  }[];
  readonly transferredErrorPaths: readonly {
    readonly id: string;
    readonly description: string;
  }[];
  readonly goldens: readonly FixtureOutput[];
}

interface ProgramApiResidualLedger {
  readonly schemaVersion: "0";
  readonly product: "omena-sdk.program-api-residuals";
  readonly closureNote: string;
  readonly entries: readonly {
    readonly id: string;
    readonly kind:
      | "workspaceRuntime"
      | "paritySurface"
      | "errorPath"
      | "inputNormalization"
      | "evidenceBinding"
      | "sourceMapParity"
      | "oracleFamily";
    readonly status: "open";
    readonly owner: string;
    readonly scope: string;
    readonly uncoveredSurfaces?: readonly string[];
    readonly uncoveredWorkflows?: readonly string[];
    readonly knownDivergenceIds?: readonly string[];
    readonly transferredErrorPathIds?: readonly string[];
  }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const fixtureDir = path.join(repoRoot, "test/_fixtures/sdk-cross-surface-parity");
const baselinePath = path.join(repoRoot, "rust/omena-cross-surface-parity-golden.json");
const residualLedgerPath = path.join(repoRoot, "rust/omena-sdk-program-api-residuals.json");
const writeMode = process.argv.includes("--write");
const fullMode = writeMode || process.argv.includes("--full");
const workDir = fs.mkdtempSync(path.join(os.tmpdir(), "omena-cross-surface-parity-"));
const workspaceDir = path.join(workDir, "workspace");
const targetDir = path.join(repoRoot, "rust/target/cross-surface-parity");
const fixtures = loadFixtures();

assert.ok(fixtures.length >= 3, "cross-surface parity requires at least three fixtures");
assert.deepEqual(
  new Set(fixtures.map((fixture) => path.extname(fixture.logicalPath))),
  new Set([".css", ".scss", ".less"]),
  "cross-surface parity fixtures must cover CSS, SCSS, and Less",
);

materializeCliWorkspace(fixtures);
const cliBinary = buildCli();
const cliOutputs = runCliFixtures(cliBinary, fixtures);
const outputBySurface: Partial<Record<Surface, readonly unknown[]>> = { cli: cliOutputs };

if (fullMode) {
  const napiModule = buildNapiModule();
  const wasmModule = buildWasmModule();
  outputBySurface.napi = runNodeSurface("napi", napiModule, fixtures);
  outputBySurface.wasm = runNodeSurface("wasm", wasmModule, fixtures);
}

if (writeMode) {
  const baseline = buildBaseline(requireAllSurfaces(outputBySurface));
  fs.writeFileSync(baselinePath, `${JSON.stringify(baseline, null, 2)}\n`);
  process.stdout.write(`Wrote ${path.relative(repoRoot, baselinePath)}\n`);
  process.exit(0);
}

let baseline = JSON.parse(fs.readFileSync(baselinePath, "utf8")) as ParityBaseline;
if (process.env.OMENA_CROSS_SURFACE_PARITY_TEST_DROP_DIVERGENCE === "1") {
  baseline = { ...baseline, knownDivergences: [] };
}
if (process.env.OMENA_CROSS_SURFACE_PARITY_TEST_ADD_UNCOVERED_SURFACE === "1") {
  baseline = {
    ...baseline,
    coverage: {
      ...baseline.coverage,
      uncoveredSurfaces: [...baseline.coverage.uncoveredSurfaces, "unregistered-surface"],
    },
  };
}
assertBaselineContract(baseline);
let residualLedger = JSON.parse(
  fs.readFileSync(residualLedgerPath, "utf8"),
) as ProgramApiResidualLedger;
if (process.env.OMENA_SDK_RESIDUAL_LEDGER_TEST_DROP_OWNER === "1") {
  residualLedger = {
    ...residualLedger,
    entries: residualLedger.entries.map((entry, index) =>
      index === 0 ? { ...entry, owner: "" } : entry,
    ),
  };
}
assertResidualLedgerContract(residualLedger, baseline);
assert.deepEqual(
  baseline.fixtures,
  fixtures.map(({ source: _source, ...fixture }) => fixture),
  "cross-surface parity fixture corpus drifted",
);

const baselineByFixture = new Map(baseline.goldens.map((golden) => [golden.fixtureId, golden]));
for (const [surface, outputs] of Object.entries(outputBySurface) as [
  Surface,
  readonly unknown[],
][]) {
  for (let index = 0; index < fixtures.length; index += 1) {
    const fixture = fixtures[index];
    const golden = baselineByFixture.get(fixture.id);
    assert.ok(golden, `missing golden for ${fixture.id}`);
    assert.deepEqual(
      canonicalize(outputs[index]),
      canonicalize(golden.outputs[surface]),
      `${surface} output drifted for ${fixture.id}`,
    );
  }
}

if (fullMode) {
  const all = requireAllSurfaces(outputBySurface);
  for (let index = 0; index < fixtures.length; index += 1) {
    const expected = canonicalize(all.cli[index]);
    assert.deepEqual(
      canonicalize(all.napi[index]),
      expected,
      `NAPI parity failed for ${fixtures[index].id}`,
    );
    assert.deepEqual(
      canonicalize(all.wasm[index]),
      expected,
      `WASM parity failed for ${fixtures[index].id}`,
    );
  }
}

process.stdout.write(
  `Omena cross-surface parity OK: mode=${fullMode ? "full" : "cli-smoke"} fixtures=${fixtures.length}\n`,
);

function loadFixtures(): Fixture[] {
  if (process.env.OMENA_CROSS_SURFACE_PARITY_TEST_EMPTY_CORPUS === "1") return [];
  return fs
    .readdirSync(fixtureDir)
    .filter((name) => [".css", ".scss", ".less"].includes(path.extname(name)))
    .toSorted()
    .map((name) => {
      const sourcePath = path.relative(repoRoot, path.join(fixtureDir, name));
      const source = fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
      return {
        id: path.basename(name, path.extname(name)),
        sourcePath,
        logicalPath: `src/${name}`,
        source,
        sha256: createHash("sha256").update(source).digest("hex"),
      };
    });
}

function materializeCliWorkspace(entries: readonly Fixture[]): void {
  for (const fixture of entries) {
    const target = path.join(workspaceDir, fixture.logicalPath);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, fixture.source);
  }
}

function buildCli(): string {
  run("cargo", [
    "build",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cli",
    "--bin",
    "omena-cli",
  ]);
  return path.join(repoRoot, "rust/target/debug/omena-cli");
}

function buildNapiModule(): string {
  const env = rustBuildEnv();
  run(
    "cargo",
    ["build", "--manifest-path", "rust/Cargo.toml", "-p", "omena-napi", "--release"],
    env,
  );
  const extension =
    process.platform === "darwin" ? "dylib" : process.platform === "win32" ? "dll" : "so";
  const libraryName =
    process.platform === "win32" ? "omena_napi.dll" : `libomena_napi.${extension}`;
  const source = path.join(targetDir, "release", libraryName);
  const target = path.join(workDir, "napi", "omena.node");
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(source, target);
  return target;
}

function buildWasmModule(): string {
  const outputDir = path.join(workDir, "wasm");
  run("wasm-pack", [
    "build",
    "rust/crates/omena-wasm",
    "--target",
    "nodejs",
    "--release",
    "--out-dir",
    outputDir,
  ]);
  return path.join(outputDir, "omena_wasm.js");
}

function runCliFixtures(binary: string, entries: readonly Fixture[]): readonly unknown[] {
  return entries.map((fixture) => {
    const stdout = execFileSync(binary, ["check", fixture.logicalPath, "--json"], {
      cwd: workspaceDir,
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    });
    const parsed = JSON.parse(stdout) as unknown;
    if (process.env.OMENA_CROSS_SURFACE_PARITY_TEST_WRAP_CLI === "1") {
      return { check: parsed };
    }
    return parsed;
  });
}

function runNodeSurface(
  surface: "napi" | "wasm",
  modulePath: string,
  entries: readonly Fixture[],
): readonly unknown[] {
  const inputPath = path.join(workDir, `${surface}-input.json`);
  fs.writeFileSync(inputPath, JSON.stringify(entries));
  const script =
    surface === "napi"
      ? `const fs=require("fs");const m=require(process.argv[1]);const f=JSON.parse(fs.readFileSync(process.argv[2],"utf8"));process.stdout.write(JSON.stringify(f.map(x=>JSON.parse(m.checkStyleSourceJson(x.source,x.logicalPath)))));`
      : `const fs=require("fs");const m=require(process.argv[1]);const f=JSON.parse(fs.readFileSync(process.argv[2],"utf8"));process.stdout.write(JSON.stringify(f.map(x=>m.checkStyleSource(x.source,x.logicalPath))));`;
  return JSON.parse(
    execFileSync(process.execPath, ["-e", script, modulePath, inputPath], {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    }),
  ) as readonly unknown[];
}

function buildBaseline(outputs: Record<Surface, readonly unknown[]>): ParityBaseline {
  const captureCommit = output("git", ["rev-parse", "HEAD"]).trim();
  return {
    schemaVersion: "0",
    product: "omena-sdk.cross-surface-parity",
    captureCommit,
    fixtures: fixtures.map(({ source: _source, ...fixture }) => fixture),
    coverage: expectedCoverage(),
    knownDivergences: expectedKnownDivergences(),
    transferredErrorPaths: expectedTransferredErrorPaths(),
    goldens: fixtures.map((fixture, index) => ({
      fixtureId: fixture.id,
      outputs: {
        napi: canonicalize(outputs.napi[index]),
        wasm: canonicalize(outputs.wasm[index]),
        cli: canonicalize(outputs.cli[index]),
      },
    })),
  };
}

function assertBaselineContract(baseline: ParityBaseline): void {
  assert.equal(baseline.schemaVersion, "0");
  assert.equal(baseline.product, "omena-sdk.cross-surface-parity");
  assert.deepEqual(baseline.coverage, expectedCoverage(), "parity coverage ledger drifted");
  assert.deepEqual(
    baseline.knownDivergences,
    expectedKnownDivergences(),
    "known-divergence ledger drifted",
  );
  assert.deepEqual(
    baseline.transferredErrorPaths,
    expectedTransferredErrorPaths(),
    "transferred error-path ledger drifted",
  );
  assert.ok(
    baseline.coverage.uncoveredSurfaces.length + baseline.coverage.uncoveredWorkflows.length <=
      baseline.coverage.uncoveredCountCeiling,
    "uncovered parity obligations exceeded the committed ceiling",
  );
  const captureCommit =
    process.env.OMENA_CROSS_SURFACE_PARITY_TEST_RECENT_CAPTURE === "1"
      ? output("git", ["rev-parse", "HEAD"]).trim()
      : baseline.captureCommit;
  const ancestry = spawnSync("git", ["merge-base", "--is-ancestor", captureCommit, "HEAD"], {
    cwd: repoRoot,
  });
  assert.equal(ancestry.status, 0, "parity goldens must come from an ancestor commit");
  if (process.env.OMENA_CROSS_SURFACE_PARITY_TEST_RECENT_CAPTURE === "1") {
    assert.notEqual(
      captureCommit,
      output("git", ["rev-parse", "HEAD"]).trim(),
      "parity goldens must predate the parity harness",
    );
  }
}

function assertResidualLedgerContract(
  ledger: ProgramApiResidualLedger,
  baseline: ParityBaseline,
): void {
  assert.equal(ledger.schemaVersion, "0");
  assert.equal(ledger.product, "omena-sdk.program-api-residuals");
  assert.ok(ledger.closureNote.length > 0, "program API closure note must be present");
  assert.equal(new Set(ledger.entries.map((entry) => entry.id)).size, ledger.entries.length);
  assert.ok(ledger.entries.length > 0, "program API residual ledger must be non-empty");
  assert.deepEqual(
    ledger.entries.map((entry) => entry.kind).toSorted(),
    [
      "errorPath",
      "evidenceBinding",
      "inputNormalization",
      "oracleFamily",
      "paritySurface",
      "sourceMapParity",
      "workspaceRuntime",
    ],
    "program API residual categories must remain closed and complete",
  );
  for (const entry of ledger.entries) {
    assert.equal(entry.status, "open", `residual ${entry.id} must remain explicitly open`);
    assert.ok(entry.owner.length > 0, `residual ${entry.id} must name an owner`);
    assert.ok(entry.scope.length > 0, `residual ${entry.id} must describe its scope`);
  }

  const referenced = (key: keyof ProgramApiResidualLedger["entries"][number]) =>
    ledger.entries.flatMap((entry) => {
      const value = entry[key];
      return Array.isArray(value) ? value : [];
    });
  assert.deepEqual(
    referenced("uncoveredSurfaces").toSorted(),
    [...baseline.coverage.uncoveredSurfaces].toSorted(),
    "residual ledger must cover every uncovered surface exactly once",
  );
  assert.deepEqual(
    referenced("uncoveredWorkflows").toSorted(),
    [...baseline.coverage.uncoveredWorkflows].toSorted(),
    "residual ledger must cover every uncovered workflow exactly once",
  );
  assert.deepEqual(
    referenced("knownDivergenceIds").toSorted(),
    baseline.knownDivergences.map((entry) => entry.id).toSorted(),
    "residual ledger must cover every known divergence exactly once",
  );
  assert.deepEqual(
    referenced("transferredErrorPathIds").toSorted(),
    baseline.transferredErrorPaths.map((entry) => entry.id).toSorted(),
    "residual ledger must cover every transferred error path exactly once",
  );
}

function expectedCoverage(): ParityBaseline["coverage"] {
  return {
    coveredSurfaces: ["napi", "wasm", "cli"],
    coveredWorkflows: ["diagnostics"],
    uncoveredSurfaces: ["lsp"],
    uncoveredWorkflows: ["snapshot", "query", "build", "explain"],
    uncoveredCountCeiling: 5,
  };
}

function expectedKnownDivergences(): ParityBaseline["knownDivergences"] {
  return [
    {
      id: "empty-path-normalization",
      description:
        "NAPI and WASM normalize an empty style path to style.css; the file-fed CLI requires a real path.",
    },
  ];
}

function expectedTransferredErrorPaths(): ParityBaseline["transferredErrorPaths"] {
  return [
    {
      id: "build-context-json",
      description:
        "Cross-surface error parity starts with the fallible NAPI parse_context_json and WASM parse_context_value build workflow.",
    },
  ];
}

function requireAllSurfaces(
  outputs: Partial<Record<Surface, readonly unknown[]>>,
): Record<Surface, readonly unknown[]> {
  assert.ok(
    outputs.napi && outputs.wasm && outputs.cli,
    "full parity requires NAPI, WASM, and CLI",
  );
  return { napi: outputs.napi, wasm: outputs.wasm, cli: outputs.cli };
}

function canonicalize(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalize);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>)
      .toSorted(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => [key, canonicalize(entry)]),
  );
}

function rustBuildEnv(): NodeJS.ProcessEnv {
  const env = { ...process.env, CARGO_TARGET_DIR: targetDir };
  const stableDeveloperDir = "/Applications/Xcode.app/Contents/Developer";
  if (process.platform === "darwin" && fs.existsSync(stableDeveloperDir)) {
    env.DEVELOPER_DIR = stableDeveloperDir;
  }
  return env;
}

function run(command: string, args: readonly string[], env = process.env): void {
  execFileSync(command, [...args], { cwd: repoRoot, env, stdio: "inherit" });
}

function output(command: string, args: readonly string[]): string {
  return execFileSync(command, [...args], { cwd: repoRoot, encoding: "utf8" });
}

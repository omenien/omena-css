import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import { declaredRustSemverIntentCrates } from "./lib/rust-semver-intent.ts";

type SurfaceDisposition = "snapshotGated" | "trainSemverOnly" | "noRustConsumerSurface";
type RegistryBaseline = "present" | "firstPublish";

interface SurfaceRegisterRow {
  readonly crate: string;
  readonly disposition: SurfaceDisposition;
  readonly registryBaselineAtRegistration: RegistryBaseline;
  readonly snapshot?: string;
  readonly reason?: string;
  readonly namedSurfaceGate?: string;
}

interface SurfaceRegister {
  readonly schemaVersion: "0";
  readonly product: "omena-published-crate-surface-register";
  readonly domainAuthority: "rust.publish-train-closure.canonicalPublishOrder";
  readonly dispositions: readonly SurfaceDisposition[];
  readonly rows: readonly SurfaceRegisterRow[];
}

interface PublishTrainClosure {
  readonly canonicalPublishOrder: readonly string[];
  readonly trainCrateCount: number;
}

interface CargoPackage {
  readonly name: string;
  readonly manifest_path: string;
}

interface RegistryState {
  readonly registered: readonly string[];
  readonly unregistered: readonly string[];
}

const repoRoot = process.cwd();
const registerPath = path.join(repoRoot, "rust/omena-published-crate-surface-register.json");
const writeSnapshots = process.argv.includes("--write-snapshots");
const initializeFrom = readArg("--initialize-from");
const requiredSnapshotCrates = new Set([
  "omena-query-transform-runner",
  "omena-tsgo-client",
  "omena-wasm",
]);
const dispositions: readonly SurfaceDisposition[] = [
  "snapshotGated",
  "trainSemverOnly",
  "noRustConsumerSurface",
];

const closure = readPublishTrainClosure();
const packages = readCargoPackages();
const packageByName = new Map(packages.map((pkg) => [pkg.name, pkg]));

if (initializeFrom) {
  initializeRegister(initializeFrom);
  process.stdout.write(
    `Initialized ${path.relative(repoRoot, registerPath)} from the measured registry state.\n`,
  );
  process.exit(0);
}

const register = JSON.parse(readFileSync(registerPath, "utf8")) as SurfaceRegister;
assert.equal(register.schemaVersion, "0", "published-crate register schemaVersion");
assert.equal(
  register.product,
  "omena-published-crate-surface-register",
  "published-crate register product",
);
assert.equal(
  register.domainAuthority,
  "rust.publish-train-closure.canonicalPublishOrder",
  "published-crate register domain authority",
);
assert.deepEqual(register.dispositions, dispositions, "published-crate disposition vocabulary");
// The publish closure can add, remove, or reorder crates, so a stale register
// makes this comparison false without relying on a hand-maintained count.
assert.deepEqual(
  register.rows.map((row) => row.crate),
  closure.canonicalPublishOrder,
  "published-crate register must cover the computed publish train in canonical order",
);
assert.equal(
  new Set(register.rows.map((row) => row.crate)).size,
  closure.trainCrateCount,
  "published-crate register must contain one row per train crate",
);

const releaseWorkflow = readFileSync(
  path.join(repoRoot, ".github/workflows/_publish-crate-train.yml"),
  "utf8",
);
assert.match(
  releaseWorkflow,
  /cargo-semver-checks steady-state gate/u,
  "trainSemverOnly requires the release train semver gate",
);
assert.match(
  releaseWorkflow,
  /Skipping cargo-semver-checks for first-publish crate/u,
  "first-publish skip policy must remain explicit",
);
assert.match(
  releaseWorkflow,
  /omena-check run rust\/release-semver/u,
  "the release train must consume the declared semver intent policy",
);

for (const crate of requiredSnapshotCrates) {
  assert.equal(
    register.rows.find((row) => row.crate === crate)?.disposition,
    "snapshotGated",
    `${crate} must remain snapshot-gated`,
  );
}

for (const crate of declaredRustSemverIntentCrates(repoRoot)) {
  const row = register.rows.find((candidate) => candidate.crate === crate);
  assert(row, `${crate} semver intent must refer to a published-crate register row`);
  assert.equal(
    row.registryBaselineAtRegistration,
    "present",
    `${crate} cannot declare a compatibility break before its first registry publication`,
  );
}

const snapshotRows: SurfaceRegisterRow[] = [];
for (const row of register.rows) {
  assert(
    dispositions.includes(row.disposition),
    `unknown published-crate disposition: ${row.crate}:${row.disposition}`,
  );
  if (
    row.registryBaselineAtRegistration === "firstPublish" &&
    row.disposition === "trainSemverOnly"
  ) {
    // Registry measurement can emit this state for a new crate; the release
    // train has no previous version against which to run its semver check.
    throw new Error(
      `${row.crate} cannot use trainSemverOnly without a registry baseline; use snapshotGated`,
    );
  }
  if (row.disposition === "snapshotGated") {
    assert(row.snapshot, `${row.crate} snapshotGated row must name a snapshot`);
    snapshotRows.push(row);
    continue;
  }
  if (row.disposition === "noRustConsumerSurface") {
    assert(row.reason?.trim(), `${row.crate} noRustConsumerSurface row must state a reason`);
    assert(
      row.namedSurfaceGate?.trim(),
      `${row.crate} noRustConsumerSurface row must name its replacement gate`,
    );
    continue;
  }
  assert.equal(row.snapshot, undefined, `${row.crate} trainSemverOnly row cannot name a snapshot`);
}

for (const row of snapshotRows) {
  const actual = renderPublicApi(row.crate);
  const snapshotPath = path.join(repoRoot, row.snapshot!);
  if (writeSnapshots) {
    mkdirSync(path.dirname(snapshotPath), { recursive: true });
    writeFileSync(snapshotPath, actual);
    continue;
  }
  assert(existsSync(snapshotPath), `${row.crate} public API snapshot is missing`);
  const expected = normalizeOutput(readFileSync(snapshotPath, "utf8"));
  if (actual !== expected) {
    throw new Error(
      `${row.crate} public API changed without updating ${row.snapshot}; ` +
        `first difference ${firstDifferingLine(expected, actual)}`,
    );
  }
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.published-crate-surface-register",
      trainCrateCount: closure.trainCrateCount,
      snapshotGatedCount: snapshotRows.length,
      trainSemverOnlyCount: register.rows.filter((row) => row.disposition === "trainSemverOnly")
        .length,
      noRustConsumerSurfaceCount: register.rows.filter(
        (row) => row.disposition === "noRustConsumerSurface",
      ).length,
      firstPublishCount: register.rows.filter(
        (row) => row.registryBaselineAtRegistration === "firstPublish",
      ).length,
    },
    null,
    2,
  )}\n`,
);

function initializeRegister(registryStatePath: string): void {
  const registry = JSON.parse(readFileSync(registryStatePath, "utf8")) as RegistryState;
  const registered = new Set(registry.registered);
  const unregistered = new Set(registry.unregistered);
  assert.deepEqual(
    [...new Set([...registered, ...unregistered])].toSorted(),
    [...closure.canonicalPublishOrder].toSorted(),
    "registry measurement must cover the computed publish train",
  );
  const rows = closure.canonicalPublishOrder.map((crate): SurfaceRegisterRow => {
    const pkg = packageByName.get(crate);
    assert(pkg, `missing cargo package for ${crate}`);
    const snapshot = path.relative(
      repoRoot,
      path.join(path.dirname(pkg.manifest_path), "tests/snapshots/public-api.txt"),
    );
    const snapshotExists = existsSync(path.join(repoRoot, snapshot));
    const firstPublish = unregistered.has(crate);
    const snapshotGated = snapshotExists || requiredSnapshotCrates.has(crate) || firstPublish;
    return {
      crate,
      disposition: snapshotGated ? "snapshotGated" : "trainSemverOnly",
      registryBaselineAtRegistration: firstPublish ? "firstPublish" : "present",
      ...(snapshotGated ? { snapshot } : {}),
    };
  });
  const output: SurfaceRegister = {
    schemaVersion: "0",
    product: "omena-published-crate-surface-register",
    domainAuthority: "rust.publish-train-closure.canonicalPublishOrder",
    dispositions,
    rows,
  };
  writeFileSync(registerPath, `${JSON.stringify(output, null, 2)}\n`);
  execFileSync("pnpm", ["exec", "oxfmt", registerPath], {
    cwd: repoRoot,
    stdio: "ignore",
  });
}

function readPublishTrainClosure(): PublishTrainClosure {
  return JSON.parse(
    execFileSync(
      process.execPath,
      ["--import", "tsx", "./scripts/check-rust-publish-train-closure.ts"],
      { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    ),
  ) as PublishTrainClosure;
}

function readCargoPackages(): readonly CargoPackage[] {
  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
      { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    ),
  ) as { readonly packages: readonly CargoPackage[] };
  return metadata.packages;
}

function renderPublicApi(crate: string): string {
  const output = execFileSync(
    "cargo",
    ["public-api", "--manifest-path", "rust/Cargo.toml", "-p", crate, "-sss", "--color", "never"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  const normalized = normalizeOutput(output);
  const lines = normalized.split("\n").filter((line) => line.trim().length > 0);
  assert(lines.length > 2, `${crate} public API snapshot is unexpectedly small`);
  return normalized;
}

function normalizeOutput(output: string): string {
  const lines = output
    .replace(/\r\n/g, "\n")
    // Rustdoc changed the canonical print paths for these re-exported I/O types.
    .replace(/\b(?:alloc|core)::io::read::Read\b/gu, "std::io::Read")
    .replace(/\b(?:alloc|core)::io::write::Write\b/gu, "std::io::Write")
    .replace(/\b(?:alloc|core)::io::/gu, "std::io::")
    .split("\n")
    .map((line) => line.trimEnd())
    .filter((line) => line.length > 0)
    .toSorted();
  return `${lines.join("\n")}\n`;
}

function firstDifferingLine(expected: string, actual: string): string {
  const expectedLines = expected.split("\n");
  const actualLines = actual.split("\n");
  const limit = Math.max(expectedLines.length, actualLines.length);
  for (let index = 0; index < limit; index += 1) {
    if (expectedLines[index] !== actualLines[index]) {
      return `${index + 1}: expected ${JSON.stringify(expectedLines[index] ?? "")}, got ${JSON.stringify(actualLines[index] ?? "")}`;
    }
  }
  return "unknown";
}

function readArg(name: string): string | undefined {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

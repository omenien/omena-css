import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runDeclaredRustSemverCheck } from "./lib/rust-semver-intent.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const allFeaturesSnapshot = process.argv.includes("--all-features");
const snapshotPath = path.join(
  repoRoot,
  allFeaturesSnapshot
    ? "rust/crates/omena-query/tests/snapshots/public-api-all-features.txt"
    : "rust/crates/omena-query/tests/snapshots/public-api.txt",
);
const wildcardBaselinePath = path.join(
  repoRoot,
  "rust/crates/omena-query/tests/snapshots/wildcard-reexport-baseline.json",
);
const writeSnapshot = process.argv.includes("--write");
const workspaceVersion = readWorkspaceVersion();
const semverChecksRequired =
  process.env.OMENA_QUERY_PUBLIC_SURFACE_BASELINE_REV !== undefined ||
  requiresSteadyStateSemver(workspaceVersion);

ensureCargoSubcommand({
  subcommand: "public-api",
  crate: "cargo-public-api",
  version: "0.52.0",
  versionArgs: ["-V"],
});
if (semverChecksRequired) {
  ensureCargoSubcommand({
    subcommand: "semver-checks",
    crate: "cargo-semver-checks",
    version: "0.48.0",
    versionArgs: ["--version"],
  });
}
ensureRustupToolchain({
  toolchain: "nightly",
  reason: "cargo-public-api requires nightly rustdoc JSON support",
});

const publicApi = normalizeOutput(
  execFileSync(
    "cargo",
    [
      "public-api",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "-sss",
      "--color",
      "never",
      ...(allFeaturesSnapshot ? ["--all-features"] : []),
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
);
assertPublicApiSnapshotIsNonVacuous(publicApi);

if (writeSnapshot) {
  mkdirSync(path.dirname(snapshotPath), { recursive: true });
  writeFileSync(snapshotPath, publicApi);
} else {
  if (!existsSync(snapshotPath)) {
    throw new Error(
      `${path.relative(repoRoot, snapshotPath)} is missing. Run ` +
        `\`${snapshotUpdater()}\` to create it.`,
    );
  }
  const expected = normalizeOutput(readFileSync(snapshotPath, "utf8"));
  if (publicApi !== expected) {
    const firstMismatch = firstDifferingLine(expected, publicApi);
    throw new Error(
      "omena-query public API changed without updating the snapshot.\n" +
        `First differing line: ${firstMismatch}\n` +
        "If this surface change is intentional, run " +
        `\`${snapshotUpdater()}\` and review the diff.`,
    );
  }
}

const wildcardReexports = scanWildcardReexports();
assertWildcardReexportScannerIsSensitive();
if (writeSnapshot) {
  mkdirSync(path.dirname(wildcardBaselinePath), { recursive: true });
  writeFileSync(
    wildcardBaselinePath,
    `${JSON.stringify(wildcardReexportBaseline(wildcardReexports), null, 2)}\n`,
  );
} else {
  if (!existsSync(wildcardBaselinePath)) {
    throw new Error(
      `${path.relative(repoRoot, wildcardBaselinePath)} is missing. Run ` +
        "`pnpm run update:rust-omena-query-public-surface` to create it.",
    );
  }
  const expected = JSON.parse(readFileSync(wildcardBaselinePath, "utf8")) as unknown;
  const observed = wildcardReexportBaseline(wildcardReexports);
  if (JSON.stringify(observed) !== JSON.stringify(expected)) {
    throw new Error(
      "omena-query wildcard re-export baseline changed.\n" +
        `Expected ${JSON.stringify(expected)}, got ${JSON.stringify(observed)}.\n` +
        "If this change is intentional, run " +
        "`pnpm run update:rust-omena-query-public-surface` and review the baseline diff.",
    );
  }
}

const baseline = semverChecksRequired ? resolveSemverBaseline() : null;
if (baseline?.kind === "revision") {
  ensureGitRevision(baseline);
}
let semverPolicy = semverChecksRequired ? "steady-state" : "genesis-snapshot-only";
let declaredFailureCount = 0;
let declaredReleaseVersion: string | null = null;
if (baseline) {
  const result = runDeclaredRustSemverCheck({
    repoRoot,
    crate: "omena-query",
    workspaceVersion,
    baselineArgs: semverBaselineArgs(baseline),
    allFeatures: allFeaturesSnapshot,
  });
  semverPolicy = result.policy;
  declaredFailureCount = result.declaredFailureCount;
  declaredReleaseVersion = result.declaredReleaseVersion;
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: allFeaturesSnapshot
        ? "rust.omena-query.public-surface-all-features"
        : "rust.omena-query.public-surface",
      snapshot: path.relative(repoRoot, snapshotPath),
      allFeaturesSnapshot,
      workspaceVersion,
      semverPolicy,
      declaredFailureCount,
      declaredReleaseVersion,
      baselineKind: baseline?.kind ?? null,
      baselineRev: baseline?.kind === "revision" ? baseline.rev : null,
      baselineVersion: null,
      cargoPublicApiVersion: "0.52.0",
      cargoSemverChecksVersion: "0.48.0",
      wildcardReexportBaseline: path.relative(repoRoot, wildcardBaselinePath),
      wildcardReexportCount: wildcardReexports.total,
    },
    null,
    2,
  )}\n`,
);

function ensureCargoSubcommand(tool: {
  readonly subcommand: string;
  readonly crate: string;
  readonly version: string;
  readonly versionArgs: readonly string[];
}): void {
  const probe = spawnSync("cargo", [tool.subcommand, ...tool.versionArgs], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (probe.status === 0) {
    return;
  }

  execFileSync("cargo", ["install", tool.crate, "--version", tool.version, "--locked"], {
    cwd: repoRoot,
    stdio: "inherit",
  });

  const afterInstall = spawnSync("cargo", [tool.subcommand, ...tool.versionArgs], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (afterInstall.status !== 0) {
    throw new Error(
      `cargo ${tool.subcommand} was not available after installing ${tool.crate}@${tool.version}`,
    );
  }
}

function ensureRustupToolchain(tool: {
  readonly toolchain: string;
  readonly reason: string;
}): void {
  const probe = spawnSync("rustup", ["run", tool.toolchain, "rustc", "--version"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (probe.status === 0) {
    return;
  }

  process.stderr.write(
    `Installing Rust ${tool.toolchain} toolchain for ${tool.reason}.\n` +
      `rustup probe stderr: ${probe.stderr.trim() || "<empty>"}\n`,
  );
  execFileSync("rustup", ["toolchain", "install", tool.toolchain, "--profile", "minimal"], {
    cwd: repoRoot,
    stdio: "inherit",
  });

  const afterInstall = spawnSync("rustup", ["run", tool.toolchain, "rustc", "--version"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (afterInstall.status !== 0) {
    throw new Error(
      `Rust ${tool.toolchain} toolchain was not available after installation: ${
        afterInstall.stderr.trim() || "<empty stderr>"
      }`,
    );
  }
}

function assertPublicApiSnapshotIsNonVacuous(publicApi: string): void {
  const lines = publicApi.split("\n").filter((line) => line.trim().length > 0);
  if (lines.length <= 100) {
    throw new Error(`omena-query public API snapshot is unexpectedly small: ${lines.length} lines`);
  }
  if (!lines.some((line) => line.includes("OmenaQuery"))) {
    throw new Error(
      "omena-query public API snapshot does not contain any OmenaQuery-prefixed item",
    );
  }
}

function snapshotUpdater(): string {
  return allFeaturesSnapshot
    ? "node --import tsx ./scripts/check-rust-omena-query-public-surface.ts --all-features --write"
    : "pnpm run update:rust-omena-query-public-surface";
}

function scanWildcardReexports(): {
  readonly total: number;
  readonly files: readonly { readonly path: string; readonly count: number }[];
} {
  const srcRoot = path.join(repoRoot, "rust/crates/omena-query/src");
  const files = listRustSourceFiles(srcRoot)
    .map((filePath) => path.relative(srcRoot, filePath).replaceAll(path.sep, "/"))
    .filter((relativePath) => !relativePath.startsWith("bin/"))
    .filter((relativePath) => relativePath !== "tests.rs")
    .filter((relativePath) => !relativePath.startsWith("tests/"))
    .sort();
  const countedFiles = files
    .map((relativePath) => {
      const absolutePath = path.join(srcRoot, relativePath);
      const count = countWildcardReexports(readFileSync(absolutePath, "utf8"));
      return { path: `rust/crates/omena-query/src/${relativePath}`, count };
    })
    .filter((entry) => entry.count > 0);
  return {
    total: countedFiles.reduce((sum, entry) => sum + entry.count, 0),
    files: countedFiles,
  };
}

function listRustSourceFiles(root: string): readonly string[] {
  const entries = readdirSync(root);
  return entries.flatMap((entry) => {
    const fullPath = path.join(root, entry);
    const stats = statSync(fullPath);
    if (stats.isDirectory()) {
      return listRustSourceFiles(fullPath);
    }
    return stats.isFile() && fullPath.endsWith(".rs") ? [fullPath] : [];
  });
}

function countWildcardReexports(source: string): number {
  return Array.from(source.matchAll(/^\s*pub\s+use\s+[A-Za-z_][A-Za-z0-9_:]*::\*\s*;/gmu)).length;
}

function assertWildcardReexportScannerIsSensitive(): void {
  const fixturePath = path.join(repoRoot, "scripts/fixtures/rust-wildcard-reexports.rs");
  const observed = countWildcardReexports(readFileSync(fixturePath, "utf8"));
  const expected = 2;
  if (observed !== expected) {
    throw new Error(
      `omena-query wildcard re-export scanner selftest expected ${expected}, got ${observed}`,
    );
  }
  const productScan = scanWildcardReexports();
  if (productScan.total !== 0) {
    throw new Error(
      `omena-query production sources must use named re-exports; found ${productScan.total} wildcard re-exports`,
    );
  }
}

function wildcardReexportBaseline(wildcardReexports: {
  readonly total: number;
  readonly files: readonly { readonly path: string; readonly count: number }[];
}): {
  readonly schemaVersion: "0";
  readonly product: "rust.omena-query.wildcard-reexport-baseline";
  readonly wildcardReexportCount: number;
  readonly files: readonly { readonly path: string; readonly count: number }[];
  readonly excluded: readonly string[];
} {
  return {
    schemaVersion: "0",
    product: "rust.omena-query.wildcard-reexport-baseline",
    wildcardReexportCount: wildcardReexports.total,
    files: wildcardReexports.files,
    excluded: [
      "rust/crates/omena-query/src/bin/**",
      "rust/crates/omena-query/src/tests.rs",
      "rust/crates/omena-query/src/tests/**",
    ],
  };
}

function normalizeOutput(output: string): string {
  const lines = output
    .replace(/\r\n/g, "\n")
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
      return `${index + 1}: expected ${JSON.stringify(expectedLines[index] ?? "")}, got ${JSON.stringify(
        actualLines[index] ?? "",
      )}`;
    }
  }
  return "unknown";
}

function readWorkspaceVersion(): string {
  const manifest = readFileSync(path.join(repoRoot, "rust/Cargo.toml"), "utf8");
  const workspacePackage = manifest.match(/\[workspace\.package\]([\s\S]*?)(?:\n\[|$)/u)?.[1];
  const version = workspacePackage?.match(/^version\s*=\s*"([^"]+)"/mu)?.[1];
  if (!version) {
    throw new Error("Unable to resolve workspace.package.version from rust/Cargo.toml");
  }
  return version;
}

function requiresSteadyStateSemver(version: string): boolean {
  const [major, minor] = version.split(".").map((part) => Number.parseInt(part, 10));
  if (!Number.isInteger(major) || !Number.isInteger(minor)) {
    throw new Error(`Unsupported workspace version ${version}`);
  }
  return major > 0 || minor >= 3;
}

type SemverBaseline =
  | {
      readonly kind: "revision";
      readonly rev: string;
      readonly fetch?: readonly string[];
    }
  | {
      readonly kind: "publishedRegistry";
    };

function resolveSemverBaseline(): SemverBaseline {
  const explicit = process.env.OMENA_QUERY_PUBLIC_SURFACE_BASELINE_REV;
  if (explicit) {
    return {
      kind: "revision",
      rev: explicit,
      fetch: ["fetch", "--no-tags", "--depth=1", "origin", explicit],
    };
  }

  // The registry baseline remains stable across failed pushes and pre-publish version bumps.
  return {
    kind: "publishedRegistry",
  };
}

function semverBaselineArgs(baseline: SemverBaseline): readonly string[] {
  if (baseline.kind === "revision") {
    return ["--baseline-rev", baseline.rev];
  }
  return [];
}

function ensureGitRevision(baseline: {
  readonly rev: string;
  readonly fetch?: readonly string[];
}): void {
  if (gitRevisionExists(baseline.rev)) {
    return;
  }
  if (baseline.fetch) {
    execFileSync("git", baseline.fetch, { cwd: repoRoot, stdio: "inherit" });
  }
  if (!gitRevisionExists(baseline.rev)) {
    throw new Error(`Unable to resolve semver baseline revision ${baseline.rev}`);
  }
}

function gitRevisionExists(rev: string): boolean {
  const result = spawnSync("git", ["rev-parse", "--verify", `${rev}^{commit}`], {
    cwd: repoRoot,
    stdio: "ignore",
  });
  return result.status === 0;
}

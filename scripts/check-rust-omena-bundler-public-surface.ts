import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runDeclaredRustSemverCheck } from "./lib/rust-semver-intent.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const snapshotPath = path.join(
  repoRoot,
  "rust/crates/omena-bundler/tests/snapshots/public-api.txt",
);
const writeSnapshot = process.argv.includes("--write");
const workspaceVersion = readWorkspaceVersion();
const semverChecksRequired =
  process.env.OMENA_BUNDLER_PUBLIC_SURFACE_BASELINE_REV !== undefined ||
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
      "omena-bundler",
      "-sss",
      "--color",
      "never",
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  ),
);

if (writeSnapshot) {
  mkdirSync(path.dirname(snapshotPath), { recursive: true });
  writeFileSync(snapshotPath, publicApi);
} else {
  if (!existsSync(snapshotPath)) {
    throw new Error(
      `${path.relative(repoRoot, snapshotPath)} is missing. Run ` +
        "`pnpm run update:rust-omena-bundler-public-surface` to create it.",
    );
  }
  const expected = normalizeOutput(readFileSync(snapshotPath, "utf8"));
  if (publicApi !== expected) {
    const firstMismatch = firstDifferingLine(expected, publicApi);
    throw new Error(
      "omena-bundler public API changed without updating the snapshot.\n" +
        `First differing line: ${firstMismatch}\n` +
        "If this surface change is intentional, run " +
        "`pnpm run update:rust-omena-bundler-public-surface` and review the diff.",
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
    crate: "omena-bundler",
    workspaceVersion,
    baselineArgs: semverBaselineArgs(baseline),
    allFeatures: true,
  });
  semverPolicy = result.policy;
  declaredFailureCount = result.declaredFailureCount;
  declaredReleaseVersion = result.declaredReleaseVersion;
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-bundler.public-surface",
      snapshot: path.relative(repoRoot, snapshotPath),
      workspaceVersion,
      semverPolicy,
      declaredFailureCount,
      declaredReleaseVersion,
      baselineKind: baseline?.kind ?? null,
      baselineRev: baseline?.kind === "revision" ? baseline.rev : null,
      cargoPublicApiVersion: "0.52.0",
      cargoSemverChecksVersion: "0.48.0",
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

function normalizeOutput(output: string): string {
  return output.replace(/\r\n/g, "\n").replace(/\s+$/u, "") + "\n";
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
  const explicit = process.env.OMENA_BUNDLER_PUBLIC_SURFACE_BASELINE_REV;
  if (explicit) {
    return {
      kind: "revision",
      rev: explicit,
      fetch: ["fetch", "--no-tags", "--depth=1", "origin", explicit],
    };
  }

  // A published baseline is stable across failed pushes and intermediate commits.
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
    encoding: "utf8",
  });
  return result.status === 0;
}

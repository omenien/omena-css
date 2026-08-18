import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { runDeclaredRustSemverCheck } from "./lib/rust-semver-intent.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const snapshotPath = path.join(
  repoRoot,
  "rust/crates/omena-reactive/tests/snapshots/public-api.txt",
);
const writeSnapshot = process.argv.includes("--write");
const workspaceVersion = readWorkspaceVersion();
const baselineVersion = "0.4.0";

ensureCargoSubcommand({
  subcommand: "public-api",
  crate: "cargo-public-api",
  version: "0.52.0",
  versionArgs: ["-V"],
});
ensureCargoSubcommand({
  subcommand: "semver-checks",
  crate: "cargo-semver-checks",
  version: "0.48.0",
  versionArgs: ["--version"],
});
ensureRustupToolchain({
  toolchain: "nightly",
  reason: "cargo-public-api requires nightly rustdoc JSON support",
});
const publicApi = renderPublicApi();
if (writeSnapshot) {
  mkdirSync(path.dirname(snapshotPath), { recursive: true });
  writeFileSync(snapshotPath, publicApi);
} else {
  if (!existsSync(snapshotPath)) {
    throw new Error(
      `${path.relative(repoRoot, snapshotPath)} is missing. Run ` +
        "`pnpm run update:rust-omena-reactive-public-surface` to create it.",
    );
  }
  const expected = normalizeOutput(readFileSync(snapshotPath, "utf8"));
  if (publicApi !== expected) {
    throw new Error(
      "omena-reactive public API changed without updating the snapshot.\n" +
        `First differing line: ${firstDifferingLine(expected, publicApi)}\n` +
        "If this surface change is intentional, run " +
        "`pnpm run update:rust-omena-reactive-public-surface` and review the diff.",
    );
  }
}

const semverResult = runDeclaredRustSemverCheck({
  repoRoot,
  crate: "omena-reactive",
  workspaceVersion,
  baselineArgs: ["--baseline-version", baselineVersion],
  allFeatures: true,
});
if (semverResult.policy !== "steady-state-patch") {
  throw new Error(
    `omena-reactive public surface requires steady-state patch checking, got ${semverResult.policy}`,
  );
}
const semverPolicy = semverResult.policy.replace(/-patch$/u, "");

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-reactive.public-surface",
      snapshot: path.relative(repoRoot, snapshotPath),
      workspaceVersion,
      semverPolicy,
      semverCheckPolicy: semverResult.policy,
      baselineKind: "publishedRegistry",
      baselineVersion,
      baselineSemantics: "published-crates-io-release",
      declaredFailureCount: semverResult.declaredFailureCount,
      declaredReleaseVersion: semverResult.declaredReleaseVersion,
      cargoPublicApiVersion: "0.52.0",
      cargoSemverChecksVersion: "0.48.0",
    },
    null,
    2,
  )}\n`,
);

function renderPublicApi(): string {
  const output = execFileSync(
    "cargo",
    [
      "public-api",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-reactive",
      "-sss",
      "--color",
      "never",
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  const normalized = normalizeOutput(output);
  if (normalized.split("\n").filter(Boolean).length <= 2) {
    throw new Error("omena-reactive public API snapshot is unexpectedly small");
  }
  return normalized;
}

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
      return `${index + 1}: expected ${JSON.stringify(expectedLines[index] ?? "")}, got ${JSON.stringify(actualLines[index] ?? "")}`;
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

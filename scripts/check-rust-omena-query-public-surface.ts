import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const snapshotPath = path.join(repoRoot, "rust/crates/omena-query/tests/snapshots/public-api.txt");
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
        "`pnpm run update:rust-omena-query-public-surface` to create it.",
    );
  }
  const expected = normalizeOutput(readFileSync(snapshotPath, "utf8"));
  if (publicApi !== expected) {
    const firstMismatch = firstDifferingLine(expected, publicApi);
    throw new Error(
      "omena-query public API changed without updating the snapshot.\n" +
        `First differing line: ${firstMismatch}\n` +
        "If this surface change is intentional, run " +
        "`pnpm run update:rust-omena-query-public-surface` and review the diff.",
    );
  }
}

const wildcardReexports = scanWildcardReexports();
assertWildcardReexportScanIsNonVacuous(wildcardReexports);
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
  const expected = JSON.parse(readFileSync(wildcardBaselinePath, "utf8")) as {
    readonly wildcardReexportCount?: unknown;
  };
  if (typeof expected.wildcardReexportCount !== "number") {
    throw new Error(
      `${path.relative(repoRoot, wildcardBaselinePath)} does not contain numeric wildcardReexportCount`,
    );
  }
  if (wildcardReexports.total !== expected.wildcardReexportCount) {
    throw new Error(
      "omena-query wildcard re-export count changed without updating the baseline.\n" +
        `Expected ${expected.wildcardReexportCount}, got ${wildcardReexports.total}.\n` +
        "If this decrease is intentional, run " +
        "`pnpm run update:rust-omena-query-public-surface` in the same change that removes the wildcard.",
    );
  }
}

const baseline = semverChecksRequired ? resolveBaselineRev() : null;
if (baseline) {
  ensureGitRevision(baseline);
  execFileSync(
    "cargo",
    [
      "semver-checks",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "--baseline-rev",
      baseline.rev,
      "--all-features",
      "--release-type",
      "patch",
      "--color",
      "never",
    ],
    { cwd: repoRoot, stdio: "inherit" },
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-query.public-surface",
      snapshot: path.relative(repoRoot, snapshotPath),
      workspaceVersion,
      semverPolicy: semverChecksRequired ? "steady-state" : "genesis-snapshot-only",
      baselineRev: baseline?.rev ?? null,
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
  const wildcardReexportPattern = /^\s*pub\s+use\s+[A-Za-z_][A-Za-z0-9_:]*::\*\s*;/gmu;
  const countedFiles = files
    .map((relativePath) => {
      const absolutePath = path.join(srcRoot, relativePath);
      const count = Array.from(
        readFileSync(absolutePath, "utf8").matchAll(wildcardReexportPattern),
      ).length;
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

function assertWildcardReexportScanIsNonVacuous(wildcardReexports: {
  readonly total: number;
  readonly files: readonly { readonly path: string; readonly count: number }[];
}): void {
  if (wildcardReexports.total === 0) {
    throw new Error("omena-query wildcard re-export scan found no occurrences");
  }
  if (
    !wildcardReexports.files.some((entry) => entry.path === "rust/crates/omena-query/src/style.rs")
  ) {
    throw new Error(
      "omena-query wildcard re-export scan did not find the expected style.rs occurrences",
    );
  }
}

function wildcardReexportBaseline(wildcardReexports: {
  readonly total: number;
  readonly files: readonly { readonly path: string; readonly count: number }[];
}): {
  readonly schemaVersion: "0";
  readonly product: "rust.omena-query.wildcard-reexport-ratchet";
  readonly wildcardReexportCount: number;
  readonly files: readonly { readonly path: string; readonly count: number }[];
  readonly excluded: readonly string[];
} {
  return {
    schemaVersion: "0",
    product: "rust.omena-query.wildcard-reexport-ratchet",
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

function resolveBaselineRev(): { readonly rev: string; readonly fetch?: readonly string[] } {
  const explicit = process.env.OMENA_QUERY_PUBLIC_SURFACE_BASELINE_REV;
  if (explicit) {
    return { rev: explicit };
  }

  const baseRef = process.env.GITHUB_BASE_REF;
  if (baseRef) {
    return {
      rev: `origin/${baseRef}`,
      fetch: [
        "fetch",
        "--no-tags",
        "--depth=1",
        "origin",
        `${baseRef}:refs/remotes/origin/${baseRef}`,
      ],
    };
  }

  const before = githubEventBeforeSha();
  if (before && !/^0+$/u.test(before)) {
    return {
      rev: before,
      fetch: ["fetch", "--no-tags", "--depth=1", "origin", before],
    };
  }

  return {
    rev: "HEAD~1",
    fetch: fallbackHeadParentFetchCommand(),
  };
}

function githubEventBeforeSha(): string | null {
  const eventPath = process.env.GITHUB_EVENT_PATH;
  if (!eventPath || !existsSync(eventPath)) {
    return null;
  }
  const event = JSON.parse(readFileSync(eventPath, "utf8")) as { readonly before?: unknown };
  return typeof event.before === "string" ? event.before : null;
}

function fallbackHeadParentFetchCommand(): readonly string[] {
  const githubRef = process.env.GITHUB_REF;
  if (githubRef?.startsWith("refs/heads/")) {
    const branch = githubRef.slice("refs/heads/".length);
    return [
      "fetch",
      "--no-tags",
      "--depth=2",
      "origin",
      `${githubRef}:refs/remotes/origin/${branch}`,
    ];
  }

  return ["fetch", "--no-tags", "--depth=2", "origin", "HEAD"];
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

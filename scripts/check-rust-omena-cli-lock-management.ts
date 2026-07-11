import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

interface LockfileJson {
  readonly omenaMinVersion?: string;
  readonly entries: ReadonlyArray<{ readonly canonicalUrl: string; readonly sifPath: string }>;
}

interface LockStatusJson {
  readonly product: string;
  readonly present: boolean;
  readonly entryCount: number;
  readonly omenaMinVersion?: string;
}

const workspace = mkdtempSync(join(tmpdir(), "omena-cli-lock-management-"));

try {
  const designSourcePath = join(workspace, "tokens.scss");
  const paletteSourcePath = join(workspace, "palette.scss");
  const designSifPath = join(workspace, "tokens.sif.json");
  const paletteSifPath = join(workspace, "palette.sif.json");
  const updatedDesignSifPath = join(workspace, "tokens-updated.sif.json");
  const lockfilePath = join(workspace, "omena.lock");

  writeFileSync(designSourcePath, "$brand: red !default;");
  writeFileSync(paletteSourcePath, "$accent: blue !default;");

  runOmena([
    "sif",
    "generate",
    designSourcePath,
    "--canonical-url",
    "pkg:design-system/_tokens.scss",
    "--output",
    designSifPath,
  ]);
  runOmena([
    "sif",
    "generate",
    paletteSourcePath,
    "--canonical-url",
    "pkg:palette/_colors.scss",
    "--output",
    paletteSifPath,
  ]);

  runOmena([
    "lock",
    "add",
    "design-system",
    "--lockfile",
    lockfilePath,
    "--sif",
    designSifPath,
    "--json",
  ]);
  let lock = readLock(runOmena(["lock", "--lockfile", lockfilePath, "--json"]).stdout);
  assert.equal(lock.product, "omena-cli.lock-status");
  assert.equal(lock.present, true);
  assert.equal(lock.entryCount, 1);
  assert.equal(typeof lock.omenaMinVersion, "string");

  runOmena([
    "lock",
    "add",
    "palette",
    "--lockfile",
    lockfilePath,
    "--sif",
    paletteSifPath,
    "--json",
  ]);

  writeFileSync(designSourcePath, "$brand: green !default;");
  runOmena([
    "sif",
    "generate",
    designSourcePath,
    "--canonical-url",
    "pkg:design-system/_tokens.scss",
    "--output",
    updatedDesignSifPath,
  ]);
  runOmena([
    "lock",
    "update",
    "design-system",
    "--lockfile",
    lockfilePath,
    "--sif",
    updatedDesignSifPath,
    "--json",
  ]);

  const lockfile = JSON.parse(readFileSync(lockfilePath, "utf8")) as LockfileJson;
  assert.equal(typeof lockfile.omenaMinVersion, "string");
  assert.equal(lockfile.entries.length, 2);
  assert.ok(lockfile.entries.some((entry) => entry.canonicalUrl === "pkg:palette/_colors.scss"));
  assert.ok(
    lockfile.entries.some(
      (entry) =>
        entry.canonicalUrl === "pkg:design-system/_tokens.scss" &&
        entry.sifPath.endsWith("tokens-updated.sif.json"),
    ),
  );

  runOmena(["lock", "verify", "--lockfile", lockfilePath, "--frozen", "--json"]);

  writeFileSync(lockfilePath, JSON.stringify({ ...lockfile, omenaMinVersion: "999.0.0" }));
  const future = runOmena(["lock", "verify", "--lockfile", lockfilePath, "--frozen", "--json"], 1);
  assert.match(future.stdout, /omenaMinVersionUnsupported/);

  console.log("validated omena-cli lock management: status add update-package min-version");
} finally {
  rmSync(workspace, { force: true, recursive: true });
}

function readLock(stdout: string): LockStatusJson {
  return JSON.parse(stdout) as LockStatusJson;
}

function runOmena(args: readonly string[], expectedStatus = 0): { readonly stdout: string } {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cli",
      "--bin",
      "omena",
      "--",
      ...args,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 64,
    },
  );

  if (result.error) {
    throw result.error;
  }
  assert.equal(
    result.status,
    expectedStatus,
    `omena-cli ${args.join(" ")} exited ${result.status}, expected ${expectedStatus}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}

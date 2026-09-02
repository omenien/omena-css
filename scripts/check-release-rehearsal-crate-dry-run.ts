import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";

import { renderCrateResumeExcludeArgs } from "./crate-registry-state.ts";

interface ClosureReport {
  readonly canonicalPublishOrder: readonly string[];
  readonly trainCrateCount: number;
  readonly closureViolations: number;
}

interface RegistryReport {
  readonly publishable: readonly string[];
  readonly registered: readonly string[];
  readonly unregistered: readonly string[];
  readonly alreadyPublished: readonly string[];
  readonly remaining: readonly string[];
  readonly semverEligible: readonly string[];
  readonly semverNoCheckableLibraryBaseline: readonly string[];
  readonly semverAlreadyPublished: readonly string[];
  readonly publishMode: { readonly authenticationRequired: boolean };
}

const repoRoot = process.cwd();
for (const credential of [
  "CRATES_IO_TOKEN",
  "CARGO_REGISTRY_TOKEN",
  "NPM_TOKEN",
  "NODE_AUTH_TOKEN",
]) {
  assert.equal(
    process.env[credential],
    undefined,
    `${credential} must not enter release rehearsal`,
  );
}

const tempDir = mkdtempSync(path.join(os.tmpdir(), "omena-release-rehearsal-"));
try {
  const closure = JSON.parse(
    execFileSync(
      process.execPath,
      ["--import", "tsx", "./scripts/check-rust-publish-train-closure.ts"],
      {
        cwd: repoRoot,
        encoding: "utf8",
        maxBuffer: 64 * 1024 * 1024,
      },
    ),
  ) as ClosureReport;
  assert.equal(closure.closureViolations, 0);
  assert.equal(closure.canonicalPublishOrder.length, closure.trainCrateCount);
  const registryPath = path.join(tempDir, "crate-registry-state.json");
  const cargoConfigPath = path.join(tempDir, "cargo-publish-workspace.toml");

  execFileSync(
    process.execPath,
    [
      "--import",
      "tsx",
      "./scripts/crate-registry-state.ts",
      "--manifest-path",
      "rust/Cargo.toml",
      "--output-file",
      registryPath,
    ],
    {
      cwd: repoRoot,
      stdio: "inherit",
      env: { ...process.env, DRY_RUN: "true", PUBLISH_MODE: "auto" },
    },
  );
  const registry = JSON.parse(readFileSync(registryPath, "utf8")) as RegistryReport;
  assert.deepEqual(
    [...registry.publishable].toSorted(),
    [...closure.canonicalPublishOrder].toSorted(),
  );
  assert.equal(registry.registered.length + registry.unregistered.length, closure.trainCrateCount);
  assert.equal(
    registry.remaining.length + registry.alreadyPublished.length,
    closure.trainCrateCount,
  );
  assert.equal(
    registry.semverEligible.length +
      registry.semverNoCheckableLibraryBaseline.length +
      registry.semverAlreadyPublished.length +
      registry.unregistered.length,
    closure.trainCrateCount,
    "every crate must have one live semver/readiness disposition",
  );
  assert.equal(registry.publishMode.authenticationRequired, false);
  const resumeExcludeArgs = renderCrateResumeExcludeArgs(registry.alreadyPublished);
  assert.deepEqual(
    resumeExcludeArgs.match(/--exclude ([^ ]+)/gu)?.map((arg) => arg.slice("--exclude ".length)) ??
      [],
    registry.alreadyPublished,
    "resume excludes must cover every crate already live at the workspace version",
  );

  execFileSync("pnpm", ["omena-check", "run", "rust/release-semver-intent-contract"], {
    cwd: repoRoot,
    stdio: "inherit",
  });
  execFileSync("pnpm", ["omena-check", "run", "rust/inter-crate-pin"], {
    cwd: repoRoot,
    stdio: "inherit",
  });
  execFileSync(
    process.execPath,
    [
      "--import",
      "tsx",
      "./scripts/generate-cargo-publish-workspace-config.ts",
      "--manifest-path",
      "rust/Cargo.toml",
      "--output-file",
      cargoConfigPath,
    ],
    { cwd: repoRoot, stdio: "inherit" },
  );
  execFileSync(
    "cargo",
    [
      "publish",
      "--workspace",
      "--dry-run",
      "--manifest-path",
      "rust/Cargo.toml",
      "--locked",
      "--config",
      cargoConfigPath,
    ],
    { cwd: repoRoot, stdio: "inherit" },
  );

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "release.rehearsal.crate-dry-run",
        trainCrateCount: closure.trainCrateCount,
        registryRegisteredCount: registry.registered.length,
        registryUnregisteredCount: registry.unregistered.length,
        registryAlreadyPublishedCount: registry.alreadyPublished.length,
        resumeExcludeCount: registry.alreadyPublished.length,
        uploadAttempted: false,
      },
      null,
      2,
    )}\n`,
  );
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}

#!/usr/bin/env node
import { chmodSync, existsSync, readdirSync, statSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const binRoot = path.join(repoRoot, "dist", "bin");
const currentPlatformDir = `${process.platform}-${process.arch}`;
const currentBinaryName =
  process.platform === "win32" ? "engine-shadow-runner.exe" : "engine-shadow-runner";
const minimumTargets = Number.parseInt(process.env.OMENA_PACKAGED_RUNNER_MIN_TARGETS ?? "1", 10);
const requiredPlatforms = (process.env.OMENA_PACKAGED_RUNNER_REQUIRED_PLATFORMS ?? "")
  .split(",")
  .map((value) => value.trim())
  .filter(Boolean);

if (!Number.isInteger(minimumTargets) || minimumTargets < 1) {
  throw new Error(
    `OMENA_PACKAGED_RUNNER_MIN_TARGETS must be a positive integer, got ${process.env.OMENA_PACKAGED_RUNNER_MIN_TARGETS}`,
  );
}

if (!existsSync(binRoot)) {
  throw new Error(`Missing packaged runner directory: ${binRoot}`);
}

const targetDirs = readdirSync(binRoot, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .toSorted();

if (targetDirs.length < minimumTargets) {
  throw new Error(
    `Expected at least ${minimumTargets} packaged runner target(s), found ${targetDirs.length}: ${targetDirs.join(", ")}`,
  );
}

for (const platform of requiredPlatforms) {
  if (!targetDirs.some((targetDir) => targetDir.startsWith(`${platform}-`))) {
    throw new Error(
      `Missing packaged runner for required platform ${platform}; found ${targetDirs.join(", ")}`,
    );
  }
}

for (const targetDir of targetDirs) {
  const binaryName = targetDir.startsWith("win32-")
    ? "engine-shadow-runner.exe"
    : "engine-shadow-runner";
  const runnerPath = path.join(binRoot, targetDir, binaryName);
  if (!existsSync(runnerPath)) {
    throw new Error(`Missing packaged engine-shadow-runner at ${runnerPath}`);
  }
  const size = statSync(runnerPath).size;
  if (size <= 0) {
    throw new Error(`Packaged engine-shadow-runner is empty: ${runnerPath}`);
  }
}

const currentRunnerPath = path.join(binRoot, currentPlatformDir, currentBinaryName);
if (!existsSync(currentRunnerPath)) {
  throw new Error(`Missing current-platform packaged runner at ${currentRunnerPath}`);
}

if (process.platform !== "win32") {
  chmodSync(currentRunnerPath, 0o755);
}

const mode = "__packaged-runner-smoke__";
const child = spawnSync(currentRunnerPath, [mode], {
  cwd: repoRoot,
  input: "{}",
  encoding: "utf8",
});

const stderr = child.stderr ?? "";
if (child.status === 0 || !stderr.includes(`unsupported engine-shadow-runner mode: ${mode}`)) {
  throw new Error(
    [
      "Current-platform packaged engine-shadow-runner smoke failed",
      `status=${child.status ?? "unknown"}`,
      child.error ? `error=${child.error.message}` : null,
      stderr.trim() ? `stderr=${stderr.trim()}` : null,
    ]
      .filter(Boolean)
      .join("\n"),
  );
}

console.log(`Packaged engine-shadow-runner matrix ok: ${targetDirs.join(", ")}`);

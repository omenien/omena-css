#!/usr/bin/env node
import { chmodSync, existsSync, readdirSync, statSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const binRoot = path.join(repoRoot, "dist", "bin");
const currentPlatformDir = `${process.platform}-${process.arch}`;
const currentBinaryName = process.platform === "win32" ? "tsgo.exe" : "tsgo";
const minimumTargets = Number.parseInt(process.env.OMENA_PACKAGED_TSGO_MIN_TARGETS ?? "1", 10);
const requiredPlatforms = (process.env.OMENA_PACKAGED_TSGO_REQUIRED_PLATFORMS ?? "")
  .split(",")
  .map((value) => value.trim())
  .filter(Boolean);

if (!Number.isInteger(minimumTargets) || minimumTargets < 1) {
  throw new Error(
    `OMENA_PACKAGED_TSGO_MIN_TARGETS must be a positive integer, got ${process.env.OMENA_PACKAGED_TSGO_MIN_TARGETS}`,
  );
}

if (!existsSync(binRoot)) {
  throw new Error(`Missing packaged binary directory: ${binRoot}`);
}

const targetDirs = readdirSync(binRoot, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .toSorted();

if (targetDirs.length < minimumTargets) {
  throw new Error(
    `Expected at least ${minimumTargets} packaged tsgo target(s), found ${targetDirs.length}: ${targetDirs.join(", ")}`,
  );
}

for (const platform of requiredPlatforms) {
  if (!targetDirs.some((targetDir) => targetDir.startsWith(`${platform}-`))) {
    throw new Error(
      `Missing packaged tsgo for required platform ${platform}; found ${targetDirs.join(", ")}`,
    );
  }
}

for (const targetDir of targetDirs) {
  const binaryName = targetDir.startsWith("win32-") ? "tsgo.exe" : "tsgo";
  const tsgoPath = path.join(binRoot, targetDir, binaryName);
  const libPath = path.join(binRoot, targetDir, "lib.d.ts");
  if (!existsSync(tsgoPath)) {
    throw new Error(`Missing packaged tsgo at ${tsgoPath}`);
  }
  if (!existsSync(libPath)) {
    throw new Error(`Missing packaged tsgo lib.d.ts next to ${tsgoPath}`);
  }
  if (statSync(tsgoPath).size <= 0) {
    throw new Error(`Packaged tsgo is empty: ${tsgoPath}`);
  }
}

const currentTsgoPath = path.join(binRoot, currentPlatformDir, currentBinaryName);
if (!existsSync(currentTsgoPath)) {
  throw new Error(`Missing current-platform packaged tsgo at ${currentTsgoPath}`);
}

if (process.platform !== "win32") {
  chmodSync(currentTsgoPath, 0o755);
}

const child = spawnSync(currentTsgoPath, ["--version"], {
  cwd: repoRoot,
  encoding: "utf8",
});

if (child.status !== 0 || !child.stdout.includes("Version ")) {
  throw new Error(
    [
      "Current-platform packaged tsgo smoke failed",
      `status=${child.status ?? "unknown"}`,
      child.error ? `error=${child.error.message}` : null,
      child.stdout.trim() ? `stdout=${child.stdout.trim()}` : null,
      child.stderr.trim() ? `stderr=${child.stderr.trim()}` : null,
    ]
      .filter(Boolean)
      .join("\n"),
  );
}

console.log(`Packaged tsgo matrix ok: ${targetDirs.join(", ")}`);

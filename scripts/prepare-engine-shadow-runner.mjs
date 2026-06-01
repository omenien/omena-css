#!/usr/bin/env node
import { chmodSync, copyFileSync, existsSync, mkdirSync, statSync } from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const platform = process.env.OMENA_ENGINE_SHADOW_RUNNER_PLATFORM || process.platform;
const arch = process.env.OMENA_ENGINE_SHADOW_RUNNER_ARCH || process.arch;
const binaryName = platform === "win32" ? "engine-shadow-runner.exe" : "engine-shadow-runner";
const sourcePath = process.env.OMENA_ENGINE_SHADOW_RUNNER_SOURCE
  ? path.resolve(repoRoot, process.env.OMENA_ENGINE_SHADOW_RUNNER_SOURCE)
  : path.join(repoRoot, "rust", "target", "release", binaryName);
const outputDir = path.join(repoRoot, "dist", "bin", `${platform}-${arch}`);
const outputPath = path.join(outputDir, binaryName);

if (!existsSync(sourcePath)) {
  throw new Error(
    `Missing ${sourcePath}; run cargo build --manifest-path rust/Cargo.toml -p engine-shadow-runner --release first`,
  );
}

mkdirSync(outputDir, { recursive: true });
copyFileSync(sourcePath, outputPath);

if (platform !== "win32") {
  chmodSync(outputPath, 0o755);
}

const size = statSync(outputPath).size;
console.log(`Prepared ${path.relative(repoRoot, outputPath)} (${size} bytes)`);

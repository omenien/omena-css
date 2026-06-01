#!/usr/bin/env node
import { chmodSync, cpSync, existsSync, mkdirSync, readdirSync, statSync } from "node:fs";
import module from "node:module";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const requireFromRepo = module.createRequire(path.join(repoRoot, "package.json"));
const platform = process.env.OMENA_TSGO_PLATFORM || process.platform;
const arch = process.env.OMENA_TSGO_ARCH || process.arch;
const packageName = `@typescript/native-preview-${platform}-${arch}`;
const binaryName = platform === "win32" ? "tsgo.exe" : "tsgo";
const outputDir = path.join(repoRoot, "dist", "bin", `${platform}-${arch}`);

let packageJsonPath;
try {
  const nativePreviewPackageJson = requireFromRepo.resolve(
    "@typescript/native-preview/package.json",
  );
  const requireFromNativePreview = module.createRequire(nativePreviewPackageJson);
  packageJsonPath = requireFromNativePreview.resolve(`${packageName}/package.json`);
} catch (err) {
  throw new Error(
    [
      `Unable to resolve ${packageName}.`,
      "Install the repo-pinned @typescript/native-preview package before preparing tsgo.",
    ].join("\n"),
    { cause: err },
  );
}

const sourceLibDir = path.join(path.dirname(packageJsonPath), "lib");
const sourceBinaryPath = path.join(sourceLibDir, binaryName);

if (!existsSync(sourceBinaryPath)) {
  throw new Error(`Missing ${sourceBinaryPath}; cannot prepare packaged tsgo binary`);
}

mkdirSync(outputDir, { recursive: true });
for (const entry of readdirSync(sourceLibDir, { withFileTypes: true })) {
  cpSync(path.join(sourceLibDir, entry.name), path.join(outputDir, entry.name), {
    recursive: true,
  });
}

const outputBinaryPath = path.join(outputDir, binaryName);
if (platform !== "win32") {
  chmodSync(outputBinaryPath, 0o755);
}

const size = statSync(outputBinaryPath).size;
console.log(`Prepared ${path.relative(repoRoot, outputBinaryPath)} (${size} bytes)`);

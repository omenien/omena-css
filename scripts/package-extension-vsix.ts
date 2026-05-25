import { spawnSync } from "node:child_process";
import { copyFileSync, cpSync, existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";

type PackageJson = {
  readonly name: string;
  readonly version: string;
};

const repoRoot = process.cwd();
const packageArgs = process.argv.slice(2);
const packageJson = JSON.parse(readFile("package.json")) as PackageJson;
const outputPath = path.join(repoRoot, `${packageJson.name}-${packageJson.version}.vsix`);
const stageRoot = mkdtempSync(path.join(tmpdir(), "cme-vsix-stage-"));
const require = createRequire(import.meta.url);
const vsceCli = require.resolve("@vscode/vsce/vsce");

try {
  for (const fileName of ["package.json", "README.md", "CHANGELOG.md", "LICENSE"] as const) {
    copyRequiredFile(fileName);
  }
  copyRequiredFile(".vscodeignore");
  copyRequiredDirectory("dist");

  // VSCE walks the full cwd before applying .vscodeignore. Package from a
  // runtime-only staging directory so local Rust caches cannot dominate CI.

  rmSync(outputPath, { force: true });
  const result = spawnSync(
    process.execPath,
    [vsceCli, "package", ...packageArgs, "--out", outputPath],
    {
      cwd: stageRoot,
      stdio: "inherit",
      env: process.env,
    },
  );

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`vsce package failed with status ${result.status ?? "unknown"}`);
  }

  process.stdout.write(
    `packaged staged VSIX: ${path.relative(repoRoot, outputPath)} from ${path.relative(
      repoRoot,
      stageRoot,
    )}\n`,
  );
} finally {
  if (process.env.CME_KEEP_VSIX_STAGE !== "1") {
    rmSync(stageRoot, { recursive: true, force: true });
  }
}

function readFile(fileName: string): string {
  return readFileSync(path.join(repoRoot, fileName), "utf8");
}

function copyRequiredFile(fileName: string): void {
  const source = path.join(repoRoot, fileName);
  if (!existsSync(source)) {
    throw new Error(`Missing required VSIX file: ${fileName}`);
  }
  copyFileSync(source, path.join(stageRoot, fileName));
}

function copyRequiredDirectory(dirName: string): void {
  const source = path.join(repoRoot, dirName);
  if (!existsSync(source)) {
    throw new Error(`Missing required VSIX directory: ${dirName}`);
  }
  cpSync(source, path.join(stageRoot, dirName), {
    recursive: true,
    verbatimSymlinks: true,
  });
}

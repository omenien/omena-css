import { execFileSync, spawnSync } from "node:child_process";
import { chmodSync, existsSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const repoRoot = process.cwd();
const vsixFiles = readdirSync(repoRoot).filter((file) => file.endsWith(".vsix"));
if (vsixFiles.length !== 1) {
  throw new Error(`Expected exactly one VSIX in ${repoRoot}, found ${vsixFiles.length}`);
}

const vsixFile = vsixFiles[0]!;
const vsixPath = path.join(repoRoot, vsixFile);
const platformDir = `${process.platform}-${process.arch}`;
const serverBinaryName = process.platform === "win32" ? "omena-lsp-server.exe" : "omena-lsp-server";
const tsgoBinaryName = process.platform === "win32" ? "tsgo.exe" : "tsgo";
const extractionRoot = mkdtempSync(path.join(tmpdir(), "cme-packaged-omena-lsp-"));

try {
  execFileSync("unzip", ["-q", vsixPath, "-d", extractionRoot], {
    cwd: repoRoot,
    stdio: "pipe",
  });

  const extensionRoot = path.join(extractionRoot, "extension");
  const packagedBinDir = path.join(extensionRoot, "dist", "bin", platformDir);
  const serverPath = path.join(packagedBinDir, serverBinaryName);
  const tsgoPath = path.join(packagedBinDir, tsgoBinaryName);

  assertExists(serverPath, "packaged omena-lsp-server");
  assertExists(tsgoPath, "packaged tsgo");
  if (process.platform !== "win32") {
    chmodSync(serverPath, 0o755);
    chmodSync(tsgoPath, 0o755);
  }

  const result = spawnSync(
    process.execPath,
    ["--import", "tsx", "./scripts/check-rust-omena-lsp-server-type-fact-protocol.ts"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        CME_OMENA_LSP_SERVER_PATH: serverPath,
        CME_OMENA_LSP_SERVER_CWD: extensionRoot,
        CME_PROJECT_ROOT: extensionRoot,
        CME_TSGO_PATH: tsgoPath,
      },
    },
  );

  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`Packaged omena-lsp-server type-fact protocol failed with ${result.status}`);
  }

  process.stdout.write(
    `validated packaged omena-lsp-server type-fact protocol: vsix=${vsixFile} server=${path.relative(
      extensionRoot,
      serverPath,
    )}\n`,
  );
} finally {
  rmSync(extractionRoot, { recursive: true, force: true });
}

function assertExists(filePath: string, label: string): void {
  if (!existsSync(filePath)) {
    throw new Error(`Missing ${label} at ${filePath}`);
  }
}

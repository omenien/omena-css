const path = require("node:path");
const { runTests } = require("@vscode/test-electron");
const packageJson = require("../../package.json");

function resolveVsCodeTestVersion() {
  const configured = process.env.CME_EXTENSION_HOST_VSCODE_VERSION;
  if (configured) {
    return configured;
  }
  return packageJson.engines.vscode.replace(/^[^\d]*/, "");
}

function resolveVsCodeRequestTimeoutMs() {
  const configured = Number(process.env.CME_EXTENSION_HOST_VSCODE_REQUEST_TIMEOUT_MS);
  return Number.isFinite(configured) && configured > 0 ? configured : 60_000;
}

async function main() {
  const repoRoot = path.resolve(__dirname, "..", "..");
  const extensionTestsPath = path.resolve(__dirname, "suite", "index.cjs");
  const workspacePath = path.resolve(__dirname, "fixtures", "basic");

  await runTests({
    version: resolveVsCodeTestVersion(),
    timeout: resolveVsCodeRequestTimeoutMs(),
    extensionDevelopmentPath: repoRoot,
    extensionTestsPath,
    launchArgs: [workspacePath, "--disable-extensions"],
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});

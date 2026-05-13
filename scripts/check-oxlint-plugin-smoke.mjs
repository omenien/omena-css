import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";

const repoRoot = process.cwd();
const workspaceRoot = path.join(repoRoot, "test/_fixtures/eslint-plugin-smoke");
const sourcePath = path.join(workspaceRoot, "src/App.jsx");
const pluginPath = path.join(repoRoot, "packages/oxlint-plugin/index.cjs");
const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "omena-oxlint-plugin-"));
const configPath = path.join(tempRoot, ".oxlintrc.json");

try {
  fs.writeFileSync(
    configPath,
    JSON.stringify(
      {
        env: {
          browser: true,
          es2022: true,
        },
        jsPlugins: [
          {
            name: "omena",
            specifier: pluginPath,
          },
        ],
        rules: {
          "omena/missing-static-class": [
            "error",
            {
              workspaceRoot,
            },
          ],
        },
      },
      null,
      2,
    ),
    "utf8",
  );

  let stdout = "";
  try {
    stdout = execFileSync("pnpm", ["exec", "oxlint", "-c", configPath, sourcePath, "-f", "json"], {
      cwd: repoRoot,
      encoding: "utf8",
      env: process.env,
    });
  } catch (error) {
    stdout = error.stdout?.toString() ?? "";
    if (!stdout) throw error;
  }

  const report = JSON.parse(stdout);
  const diagnostics = report.diagnostics ?? [];
  const missingStaticClass = diagnostics.find(
    (diagnostic) =>
      diagnostic.code === "omena(missing-static-class)" &&
      typeof diagnostic.message === "string" &&
      diagnostic.message.includes("Class '.ghost' not found"),
  );

  if (!missingStaticClass) {
    throw new Error(`Expected omena/missing-static-class diagnostic, got: ${stdout}`);
  }
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}

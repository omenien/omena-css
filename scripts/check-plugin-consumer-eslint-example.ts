import path from "node:path";
import fs from "node:fs";
import { createRequire } from "node:module";
import { ESLint } from "eslint";

const require = createRequire(import.meta.url);
const plugin = require("@omena/eslint-plugin");
const REPO_ROOT = process.cwd();
const EXAMPLE_ROOT = path.join(REPO_ROOT, "examples/plugin-consumers");
const ESLINT_CONFIG_PATH = path.join(EXAMPLE_ROOT, "eslint.config.mjs");
const SOURCE_FILE_PATH = path.join(EXAMPLE_ROOT, "src/App.jsx");

async function main(): Promise<void> {
  assertConfigFileShape();
  const eslint = new ESLint({
    cwd: EXAMPLE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx,ts,tsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: {
              jsx: true,
            },
          },
        },
      },
      ...plugin.configs.recommended,
    ],
  });

  const [result] = await eslint.lintFiles([SOURCE_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint example returned no results.");
  }
  if (result.messages.length !== 0) {
    throw new Error(
      `Expected clean ESLint example, got ${result.messages.length} messages: ${result.messages.map((message) => message.message).join(" | ")}`,
    );
  }
}

function assertConfigFileShape(): void {
  const configText = fs.readFileSync(ESLINT_CONFIG_PATH, "utf8");
  if (!configText.includes("createRequire")) {
    throw new Error("Expected example ESLint config to use createRequire-based plugin loading.");
  }
  if (!configText.includes("configs.recommended")) {
    throw new Error("Expected example ESLint config to use configs.recommended.");
  }
}

void main().catch((error) => {
  console.error(error);
  process.exit(1);
});

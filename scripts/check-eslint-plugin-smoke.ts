import path from "node:path";
import { ESLint } from "eslint";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const plugin = require("@omena/eslint-plugin");

const REPO_ROOT = process.cwd();
const WORKSPACE_ROOT = path.join(REPO_ROOT, "test/_fixtures/eslint-plugin-smoke");
const INVALID_CLASS_FILE_PATH = path.join(WORKSPACE_ROOT, "src/App.jsx");
const DYNAMIC_CLASS_FILE_PATH = path.join(WORKSPACE_ROOT, "src/Dynamic.jsx");
const DYNAMIC_DOMAIN_FILE_PATH = path.join(WORKSPACE_ROOT, "src/DynamicDomain.jsx");
const MISSING_MODULE_FILE_PATH = path.join(WORKSPACE_ROOT, "src/MissingModule.jsx");
const TEMPLATE_PREFIX_FILE_PATH = path.join(WORKSPACE_ROOT, "src/TemplatePrefix.jsx");

async function main(): Promise<void> {
  assertNoLegacyDiagnosticFallback();
  await assertInvalidClassReferenceRule();
  await assertMissingStaticClassRule();
  await assertMissingTemplatePrefixRule();
  await assertMissingResolvedClassValuesRule();
  await assertMissingResolvedClassDomainRule();
  await assertNoUnknownDynamicClassRule();
  await assertNoImpossibleSelectorRule();
  await assertNoImpreciseValueRule();
  await assertMTierConfig();
  await assertMissingModuleRule();
  await assertOmenaCliBackend();
}

function assertNoLegacyDiagnosticFallback(): void {
  const sharedSource = require("node:fs").readFileSync(
    path.join(REPO_ROOT, "packages/eslint-plugin/lib/_shared.cjs"),
    "utf8",
  );
  for (const forbidden of [
    ["check", "SourceDocument"].join(""),
    ["format", "LegacyCheckerFinding"].join(""),
    ["create", "WorkspaceAnalysisHost"].join(""),
    ["create", "WorkspaceStyleHost"].join(""),
    ["OMENA", "ESLINT", "QUERY", "BACKEND"].join("_"),
  ]) {
    if (sharedSource.includes(forbidden)) {
      throw new Error(`ESLint plugin must not retain legacy diagnostic fallback: ${forbidden}`);
    }
  }
}

async function assertInvalidClassReferenceRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
      },
      ...plugin.configs.recommended,
    ],
  });

  const [result] = await eslint.lintFiles([INVALID_CLASS_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(`Expected 1 message, got ${result.messages.length}.`);
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("Class '.ghost' not found")) {
    throw new Error(`Unexpected ESLint message: ${message?.message ?? "<missing>"}`);
  }
}

async function assertMissingStaticClassRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
        plugins: {
          omena: plugin,
        },
        rules: {
          "omena/missing-static-class": "error",
        },
      },
    ],
  });

  const [result] = await eslint.lintFiles([INVALID_CLASS_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(`Expected 1 missing-static-class message, got ${result.messages.length}.`);
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("Class '.ghost' not found")) {
    throw new Error(`Unexpected missing-static-class message: ${message?.message ?? "<missing>"}`);
  }
}

async function assertMissingTemplatePrefixRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
        plugins: {
          omena: plugin,
        },
        rules: {
          "omena/missing-template-prefix": "error",
        },
      },
    ],
  });

  const [result] = await eslint.lintFiles([TEMPLATE_PREFIX_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(`Expected 1 missing-template-prefix message, got ${result.messages.length}.`);
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("No class starting with 'ghost-' found")) {
    throw new Error(
      `Unexpected missing-template-prefix message: ${message?.message ?? "<missing>"}`,
    );
  }
}

async function assertNoUnknownDynamicClassRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
        plugins: {
          omena: plugin,
        },
        rules: {
          "omena/no-unknown-dynamic-class": "error",
        },
      },
    ],
  });

  const [result] = await eslint.lintFiles([DYNAMIC_CLASS_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(`Expected 1 no-unknown-dynamic-class message, got ${result.messages.length}.`);
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("Missing class for possible value: 'ghost'")) {
    throw new Error(
      `Unexpected no-unknown-dynamic-class message: ${message?.message ?? "<missing>"}`,
    );
  }
}

async function assertMissingResolvedClassValuesRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
        plugins: {
          omena: plugin,
        },
        rules: {
          "omena/missing-resolved-class-values": "error",
        },
      },
    ],
  });

  const [result] = await eslint.lintFiles([DYNAMIC_CLASS_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(
      `Expected 1 missing-resolved-class-values message, got ${result.messages.length}.`,
    );
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("Missing class for possible value: 'ghost'")) {
    throw new Error(
      `Unexpected missing-resolved-class-values message: ${message?.message ?? "<missing>"}`,
    );
  }
}

async function assertMissingResolvedClassDomainRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
        plugins: {
          omena: plugin,
        },
        rules: {
          "omena/missing-resolved-class-domain": "error",
        },
      },
    ],
  });

  const [result] = await eslint.lintFiles([DYNAMIC_DOMAIN_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(
      `Expected 1 missing-resolved-class-domain message, got ${result.messages.length}.`,
    );
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("No class matched resolved prefix 'ghost-'")) {
    throw new Error(
      `Unexpected missing-resolved-class-domain message: ${message?.message ?? "<missing>"}`,
    );
  }
}

async function assertNoImpossibleSelectorRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
        plugins: {
          omena: plugin,
        },
        rules: {
          "omena/no-impossible-selector": "error",
        },
      },
    ],
  });

  const [result] = await eslint.lintFiles([DYNAMIC_CLASS_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(`Expected 1 no-impossible-selector message, got ${result.messages.length}.`);
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("Missing class for possible value: 'ghost'")) {
    throw new Error(
      `Unexpected no-impossible-selector message: ${message?.message ?? "<missing>"}`,
    );
  }
}

async function assertNoImpreciseValueRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
        plugins: {
          omena: plugin,
        },
        rules: {
          "omena/no-imprecise-value": "error",
        },
      },
    ],
  });

  const [result] = await eslint.lintFiles([DYNAMIC_DOMAIN_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(`Expected 1 no-imprecise-value message, got ${result.messages.length}.`);
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("No class matched resolved prefix 'ghost-'")) {
    throw new Error(`Unexpected no-imprecise-value message: ${message?.message ?? "<missing>"}`);
  }
}

async function assertMTierConfig(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
      },
      ...plugin.configs.mTier,
    ],
  });

  const results = await eslint.lintFiles([DYNAMIC_CLASS_FILE_PATH, DYNAMIC_DOMAIN_FILE_PATH]);
  const messages = results.flatMap((result) => result.messages);
  if (messages.length !== 2) {
    throw new Error(`Expected 2 mTier config messages, got ${messages.length}.`);
  }
  if (!messages.some((message) => message.ruleId === "omena/no-impossible-selector")) {
    throw new Error("Expected mTier config to enable no-impossible-selector.");
  }
  if (!messages.some((message) => message.ruleId === "omena/no-imprecise-value")) {
    throw new Error("Expected mTier config to enable no-imprecise-value.");
  }
}

async function assertMissingModuleRule(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
      },
      ...plugin.configs.recommended,
    ],
  });

  const [result] = await eslint.lintFiles([MISSING_MODULE_FILE_PATH]);
  if (!result) {
    throw new Error("ESLint returned no results.");
  }
  if (result.messages.length !== 1) {
    throw new Error(`Expected 1 missing-module message, got ${result.messages.length}.`);
  }
  const [message] = result.messages;
  if (!message || !message.message.includes("Cannot resolve CSS Module './Missing.module.scss'")) {
    throw new Error(`Unexpected missing-module message: ${message?.message ?? "<missing>"}`);
  }
}

async function assertOmenaCliBackend(): Promise<void> {
  const eslint = new ESLint({
    cwd: WORKSPACE_ROOT,
    ignore: false,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.{js,jsx}"],
        languageOptions: {
          ecmaVersion: "latest",
          sourceType: "module",
          parserOptions: {
            ecmaFeatures: { jsx: true },
          },
        },
      },
      ...plugin.configs.focused,
    ],
  });

  const results = await eslint.lintFiles([
    INVALID_CLASS_FILE_PATH,
    TEMPLATE_PREFIX_FILE_PATH,
    DYNAMIC_CLASS_FILE_PATH,
    DYNAMIC_DOMAIN_FILE_PATH,
    MISSING_MODULE_FILE_PATH,
  ]);
  const messages = results.flatMap((result) => result.messages);
  const expectedRuleIds = [
    "omena/missing-static-class",
    "omena/missing-template-prefix",
    "omena/missing-resolved-class-values",
    "omena/missing-resolved-class-domain",
    "omena/missing-module",
  ];
  for (const ruleId of expectedRuleIds) {
    if (!messages.some((message) => message.ruleId === ruleId)) {
      throw new Error(`Expected omena-cli diagnostic backend message for ${ruleId}.`);
    }
  }
}

void main().catch((error) => {
  console.error(error);
  process.exit(1);
});

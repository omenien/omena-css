import fs from "node:fs";
import path from "node:path";
import stylelint from "stylelint";

const REPO_ROOT = process.cwd();
const WORKSPACE_ROOT = path.join(REPO_ROOT, "test/_fixtures/stylelint-plugin-smoke");
const STYLE_FILE_PATHS = [
  path.join(WORKSPACE_ROOT, "src/App.module.css"),
  path.join(WORKSPACE_ROOT, "src/ComposesMissingModule.module.css"),
  path.join(WORKSPACE_ROOT, "src/ComposesMissingSelector.module.css"),
  path.join(WORKSPACE_ROOT, "src/ValueMissingModule.module.css"),
  path.join(WORKSPACE_ROOT, "src/ValueMissingImported.module.css"),
  path.join(WORKSPACE_ROOT, "src/KeyframesMissing.module.css"),
  path.join(WORKSPACE_ROOT, "src/CustomPropertyMissing.module.css"),
  path.join(WORKSPACE_ROOT, "src/SassSymbolMissing.module.scss"),
];
const PLUGIN_NAME = "@omena/stylelint-plugin";

async function main() {
  assertNoLegacyDiagnosticFallback();
  await assertRecommendedStylelintBridge();
  await assertOmenaQueryStyleDiagnosticsAdapter();
}

function assertNoLegacyDiagnosticFallback() {
  const sharedSource = fs.readFileSync(
    path.join(REPO_ROOT, "packages/stylelint-plugin/lib/_shared.cjs"),
    "utf8",
  );
  for (const forbidden of [
    ["read", "StyleCheckReport"].join(""),
    ["STYLE", "CHECK", "REPORT", "CACHE"].join("_"),
    ["check", "workspace"].join(":"),
    ["OMENA", "STYLELINT", "QUERY", "BACKEND"].join("_"),
  ]) {
    if (sharedSource.includes(forbidden)) {
      throw new Error(`Stylelint plugin must not retain legacy diagnostic fallback: ${forbidden}`);
    }
  }
}

async function assertRecommendedStylelintBridge() {
  const result = await stylelint.lint({
    files: STYLE_FILE_PATHS,
    configBasedir: REPO_ROOT,
    config: {
      extends: [`${PLUGIN_NAME}/recommended`],
      customSyntax: "postcss-scss",
      rules: {
        "omena/unused-selector": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-composed-module": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-composed-selector": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-value-module": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-imported-value": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-keyframes": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-custom-property": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-sass-symbol": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
      },
    },
  });

  const warningsByFile = new Map(
    result.results.map((fileResult) => [
      path.basename(fileResult.source ?? ""),
      fileResult.warnings,
    ]),
  );

  assertSingleWarning(
    warningsByFile.get("App.module.css"),
    "Selector '.ghost' is declared but never used.",
  );
  assertSingleWarning(
    warningsByFile.get("ComposesMissingModule.module.css"),
    "Cannot resolve composed CSS Module './Missing.module.css'.",
  );
  assertSingleWarning(
    warningsByFile.get("ComposesMissingSelector.module.css"),
    "Selector '.base' not found in composed module './Base.module.css'.",
  );
  assertSingleWarning(
    warningsByFile.get("ValueMissingModule.module.css"),
    "Cannot resolve imported @value module './MissingTokens.module.css'.",
  );
  assertSingleWarning(
    warningsByFile.get("ValueMissingImported.module.css"),
    "@value 'primary' not found in './Tokens.module.css'.",
  );
  assertSingleWarning(
    warningsByFile.get("KeyframesMissing.module.css"),
    "@keyframes 'fade' not found in this file.",
  );
  assertSingleWarning(
    warningsByFile.get("CustomPropertyMissing.module.css"),
    "CSS custom property '--missing' not found in indexed style tokens.",
  );
  assertSingleWarning(
    warningsByFile.get("SassSymbolMissing.module.scss"),
    "Sass variable '$missing' not found in the visible Sass module graph.",
  );
  assertSingleWarning(
    warningsByFile.get("SassSymbolMissing.module.scss"),
    "Sass mixin '@mixin absent' not found in the visible Sass module graph.",
  );
}

async function assertOmenaQueryStyleDiagnosticsAdapter() {
  const result = await stylelint.lint({
    files: [
      path.join(WORKSPACE_ROOT, "src/App.module.css"),
      path.join(WORKSPACE_ROOT, "src/ComposesMissingModule.module.css"),
      path.join(WORKSPACE_ROOT, "src/ComposesMissingSelector.module.css"),
      path.join(WORKSPACE_ROOT, "src/ValueMissingModule.module.css"),
      path.join(WORKSPACE_ROOT, "src/ValueMissingImported.module.css"),
      path.join(WORKSPACE_ROOT, "src/CustomPropertyMissing.module.css"),
      path.join(WORKSPACE_ROOT, "src/KeyframesMissing.module.css"),
      path.join(WORKSPACE_ROOT, "src/SassSymbolMissing.module.scss"),
    ],
    configBasedir: REPO_ROOT,
    config: {
      customSyntax: "postcss-scss",
      plugins: [PLUGIN_NAME],
      rules: {
        "omena/unused-selector": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-composed-module": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-composed-selector": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-value-module": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-imported-value": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-custom-property": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-keyframes": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "omena/missing-sass-symbol": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
      },
    },
  });
  const warningsByFile = new Map(
    result.results.map((fileResult) => [
      path.basename(fileResult.source ?? ""),
      fileResult.warnings,
    ]),
  );
  assertSingleWarning(
    warningsByFile.get("App.module.css"),
    "Selector '.ghost' is declared but never used.",
  );
  assertSingleWarning(
    warningsByFile.get("ComposesMissingModule.module.css"),
    "Cannot resolve composed CSS Module './Missing.module.css'.",
  );
  assertSingleWarning(
    warningsByFile.get("ComposesMissingSelector.module.css"),
    "Selector '.base' not found in composed module './Base.module.css'.",
  );
  assertSingleWarning(
    warningsByFile.get("ValueMissingModule.module.css"),
    "Cannot resolve imported @value module './MissingTokens.module.css'.",
  );
  assertSingleWarning(
    warningsByFile.get("ValueMissingImported.module.css"),
    "@value 'primary' not found in './Tokens.module.css'.",
  );
  assertSingleWarning(
    warningsByFile.get("CustomPropertyMissing.module.css"),
    "CSS custom property '--missing' not found in indexed style tokens.",
  );
  assertSingleWarning(
    warningsByFile.get("KeyframesMissing.module.css"),
    "@keyframes 'fade' not found in this file.",
  );
  assertSingleWarning(
    warningsByFile.get("SassSymbolMissing.module.scss"),
    "Sass variable '$missing' not found",
  );
  assertSingleWarning(
    warningsByFile.get("SassSymbolMissing.module.scss"),
    "Sass mixin '@mixin absent' not found",
  );
}

function assertSingleWarning(warnings, expectedText) {
  if (!warnings) {
    throw new Error(`Missing stylelint result for expected warning '${expectedText}'.`);
  }
  if (!warnings.some((warning) => warning.text.includes(expectedText))) {
    throw new Error(
      `Expected warning '${expectedText}', got ${warnings.map((warning) => warning.text).join(" | ")}`,
    );
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});

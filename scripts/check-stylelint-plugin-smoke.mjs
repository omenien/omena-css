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
const PLUGIN_NAME = "stylelint-plugin-css-module-explainer";

async function main() {
  await assertRecommendedStylelintBridge();
  await assertOmenaQueryStyleDiagnosticsAdapter();
}

async function assertRecommendedStylelintBridge() {
  const result = await stylelint.lint({
    files: STYLE_FILE_PATHS,
    configBasedir: REPO_ROOT,
    config: {
      extends: [`${PLUGIN_NAME}/recommended`],
      customSyntax: "postcss-scss",
      rules: {
        "css-module-explainer/unused-selector": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "css-module-explainer/missing-composed-module": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "css-module-explainer/missing-composed-selector": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "css-module-explainer/missing-value-module": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "css-module-explainer/missing-imported-value": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "css-module-explainer/missing-keyframes": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "css-module-explainer/missing-custom-property": [
          true,
          {
            workspaceRoot: WORKSPACE_ROOT,
          },
        ],
        "css-module-explainer/missing-sass-symbol": [
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
    "Sass variable '$missing' not found in this file.",
  );
  assertSingleWarning(
    warningsByFile.get("SassSymbolMissing.module.scss"),
    "Sass mixin '@mixin absent' not found in this file.",
  );
}

async function assertOmenaQueryStyleDiagnosticsAdapter() {
  const previousBackend = process.env.CME_STYLELINT_QUERY_BACKEND;
  process.env.CME_STYLELINT_QUERY_BACKEND = "omena-cli";
  try {
    const result = await stylelint.lint({
      files: [
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
          "css-module-explainer/missing-composed-module": [
            true,
            {
              workspaceRoot: WORKSPACE_ROOT,
            },
          ],
          "css-module-explainer/missing-composed-selector": [
            true,
            {
              workspaceRoot: WORKSPACE_ROOT,
            },
          ],
          "css-module-explainer/missing-value-module": [
            true,
            {
              workspaceRoot: WORKSPACE_ROOT,
            },
          ],
          "css-module-explainer/missing-imported-value": [
            true,
            {
              workspaceRoot: WORKSPACE_ROOT,
            },
          ],
          "css-module-explainer/missing-custom-property": [
            true,
            {
              workspaceRoot: WORKSPACE_ROOT,
            },
          ],
          "css-module-explainer/missing-keyframes": [
            true,
            {
              workspaceRoot: WORKSPACE_ROOT,
            },
          ],
          "css-module-explainer/missing-sass-symbol": [
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
      "Sass variable '$missing' not found in this file.",
    );
    assertSingleWarning(
      warningsByFile.get("SassSymbolMissing.module.scss"),
      "Sass mixin '@mixin absent' not found in this file.",
    );
  } finally {
    if (previousBackend === undefined) {
      delete process.env.CME_STYLELINT_QUERY_BACKEND;
    } else {
      process.env.CME_STYLELINT_QUERY_BACKEND = previousBackend;
    }
  }
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

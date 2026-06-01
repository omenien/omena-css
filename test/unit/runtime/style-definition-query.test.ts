import { describe, expect, it } from "vitest";
import { AliasResolver } from "../../../server/engine-core-ts/src/core/cx/alias-resolver";
import type { StyleDocumentHIR } from "../../../server/engine-core-ts/src/core/hir/style-types";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { WorkspaceStyleDependencyGraph } from "../../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import {
  resolveStyleDefinitionTargets,
  resolveStyleDefinitionTargetsAsync,
} from "../../../server/engine-host-node/src/style-definition-query";
import { targetFixture, workspace, type CmeWorkspace } from "../../../packages/vitest-cme/src";
import { makeBaseDeps } from "../../_fixtures/test-helpers";
import { makeDesignTokenDefinitionGraph } from "../../_fixtures/style-semantic-graph";

const BUTTON_PATH = "/fake/workspace/src/Button.module.scss";
const BASE_PATH = "/fake/workspace/src/Base.module.scss";
const THEME_PATH = "/fake/workspace/src/theme.module.scss";
const TOKENS_PATH = "/fake/workspace/src/tokens.module.scss";
const TOKENS_CSS_PATH = "/fake/workspace/src/tokens.module.css";
const TOKENS_PARTIAL_PATH = "/fake/workspace/src/_tokens.module.scss";
const UTILS_PATH = "/fake/workspace/src/_utils.scss";
const PACKAGE_TOKENS_ROOT = "/fake/workspace/node_modules/@design/tokens";
const PACKAGE_TOKENS_JSON_PATH = `${PACKAGE_TOKENS_ROOT}/package.json`;
const PACKAGE_TOKENS_INDEX_PATH = `${PACKAGE_TOKENS_ROOT}/src/index.scss`;
const PACKAGE_TOKENS_COLORS_ENTRY_PATH = `${PACKAGE_TOKENS_ROOT}/src/colors.scss`;
const PACKAGE_TOKENS_TYPOGRAPHY_ENTRY_PATH = `${PACKAGE_TOKENS_ROOT}/src/typography.scss`;
const PACKAGE_TOKENS_SRC_COLORS_PARTIAL_PATH = `${PACKAGE_TOKENS_ROOT}/src/_colors.scss`;
const PACKAGE_TOKENS_SRC_TYPOGRAPHY_PARTIAL_PATH = `${PACKAGE_TOKENS_ROOT}/src/_typography.scss`;
const PACKAGE_COLORS_PATH = "/fake/workspace/node_modules/@design/tokens/_colors.scss";
const PACKAGE_VARIABLES_CSS_PATH = "/fake/workspace/node_modules/@design/tokens/variables.css";
const PACKAGE_TYPOGRAPHY_PATH = "/fake/workspace/node_modules/@design/tokens/_typography.scss";
const PACKAGE_UTILS_PATH = "/fake/workspace/node_modules/@design/foundation/scss/_utils.scss";
const EMPTY_ALIAS_RESOLVER = new AliasResolver("/fake/workspace", {});

describe("resolveStyleDefinitionTargets", () => {
  it("resolves cross-file composes tokens to target selectors", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `
.button {
  composes: b/*|*/ase from './Base.module.scss';
  color: red;
}
`,
      [BASE_PATH]: `
.base {
  color: blue;
}
`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: BASE_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 1 },
        end: { line: 1, character: 5 },
      },
    });
  });

  it("resolves animation-name tokens to same-file keyframes", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@keyframes fade {
  from { opacity: 0; }
  to { opacity: 1; }
}

.box {
  animation: f/*|*/ade 1s linear;
}
`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: BUTTON_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 11 },
        end: { line: 0, character: 15 },
      },
    });
  });

  it("resolves imported value references to source value declarations", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@value primary from "./tokens.module.scss";

.button {
  color: p/*|*/rimary;
}
`,
      [TOKENS_PATH]: `@value primary: #ff3355;`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: TOKENS_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 7 },
        end: { line: 0, character: 14 },
      },
    });
  });

  it("resolves workspace-indexed CSS custom property references to source declarations", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `.button {
  color: var(--color/*|*/-gray-700);
}
`,
      [TOKENS_CSS_PATH]: `:root { --color-gray-700: #767678; }`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: TOKENS_CSS_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 8 },
        end: { line: 0, character: 24 },
      },
    });
  });

  it("does not resolve unmatched workspace CSS custom property declarations", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `.button {
  color: var(--br/*|*/and);
}
`,
      [THEME_PATH]: `.theme { --brand: #222; }`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toEqual([]);
  });

  it("resolves CSS custom property references to imported package CSS declarations", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens/variables.css";

.button {
  color: var(--color/*|*/-gray-700);
}
`,
      [PACKAGE_VARIABLES_CSS_PATH]: `:root { --color-gray-700: #767678; }`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: PACKAGE_VARIABLES_CSS_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 8 },
        end: { line: 0, character: 24 },
      },
    });
  });

  it("resolves CSS custom property references through package.json style entries", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens";

.button {
  color: var(--color/*|*/-gray-700);
}
`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"style":"variables.css"}`,
      [PACKAGE_VARIABLES_CSS_PATH]: `:root { --color-gray-700: #767678; }`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: PACKAGE_VARIABLES_CSS_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 8 },
        end: { line: 0, character: 24 },
      },
    });
  });

  it("resolves CSS custom property references forwarded from a package root through local utilities", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "utils" as *;

.button {
  color: var(--color/*|*/-gray-700);
}
`,
      [UTILS_PATH]: `@forward "@design/tokens" as ds_*;`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"style":"variables.css"}`,
      [PACKAGE_VARIABLES_CSS_PATH]: `:root { --color-gray-700: #767678; }`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: PACKAGE_VARIABLES_CSS_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 8 },
        end: { line: 0, character: 24 },
      },
    });
  });

  it("resolves CSS custom property references to the last matching same-file declaration", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `:root {
  --brand: #111;
}
:root { --brand: #222; }
.button {
  color: var(--br/*|*/and);
}
`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: BUTTON_PATH,
      targetSelectionRange: {
        start: { line: 3, character: 8 },
        end: { line: 3, character: 15 },
      },
    });
  });

  it("prefers matching CSS custom property theme context over unrelated later declarations", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `:root {
  --brand: #111;
}
.theme { --brand: #222; }
.theme .button {
  color: var(--br/*|*/and);
}
`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: BUTTON_PATH,
      targetSelectionRange: {
        start: { line: 3, character: 9 },
        end: { line: 3, character: 16 },
      },
    });
  });

  it("prefers matching data-theme CSS custom property context over root declarations", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `:root {
  --brand: #111;
}
:root[data-theme="dark"] {
  --brand: #222;
}
[data-theme="dark"] .button {
  color: var(--br/*|*/and);
}
`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: BUTTON_PATH,
      targetSelectionRange: {
        start: { line: 4, character: 2 },
        end: { line: 4, character: 9 },
      },
    });
  });

  it("falls back to root custom properties when a later theme declaration does not match", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `:root {
  --brand: #111;
}
.theme { --brand: #222; }
.button {
  color: var(--br/*|*/and);
}
`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: BUTTON_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 2 },
        end: { line: 1, character: 9 },
      },
    });
  });

  it("resolves CSS custom property references to the last imported module declaration", () => {
    const baseCssPath = `${PACKAGE_TOKENS_ROOT}/base.css`;
    const themeCssPath = `${PACKAGE_TOKENS_ROOT}/theme.css`;
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens/base.css";
@use "@design/tokens/theme.css";

.button {
  color: var(--br/*|*/and);
}
`,
      [baseCssPath]: `:root { --brand: #111; }`,
      [themeCssPath]: `:root { --brand: #222; }`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: themeCssPath,
      targetSelectionRange: {
        start: { line: 0, character: 8 },
        end: { line: 0, character: 15 },
      },
    });
  });

  it("uses Rust design-token winner ranges for external CSS custom property definitions", () => {
    const generatedTokensPath = "/fake/workspace/src/generated.tokens.css";
    const winnerRange = {
      start: { line: 4, character: 8 },
      end: { line: 4, character: 15 },
    };
    const ws = styleWorkspace({
      [BUTTON_PATH]: `.button {
  color: var(--br/*|*/and);
}
`,
    });

    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws), {
      env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
      readRustStyleSemanticGraphForWorkspaceTarget: () =>
        makeDesignTokenDefinitionGraph({
          referenceName: "--brand",
          winnerDeclarationFilePath: generatedTokensPath,
          winnerDeclarationRange: winnerRange,
        }),
    });

    expect(targets).toEqual([
      {
        originRange: {
          start: { line: 1, character: 13 },
          end: { line: 1, character: 20 },
        },
        targetFilePath: generatedTokensPath,
        targetRange: winnerRange,
        targetSelectionRange: winnerRange,
      },
    ]);
  });

  it("uses async Rust design-token winner ranges for external CSS custom property definitions", async () => {
    const generatedTokensPath = "/fake/workspace/src/generated.tokens.css";
    const winnerRange = {
      start: { line: 4, character: 8 },
      end: { line: 4, character: 15 },
    };
    const ws = styleWorkspace({
      [BUTTON_PATH]: `.button {
  color: var(--br/*|*/and);
}
`,
    });

    const targets = await resolveStyleDefinitionTargetsAsync(styleTarget(ws), styleDeps(ws), {
      env: { OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query" } as NodeJS.ProcessEnv,
      readRustStyleSemanticGraphForWorkspaceTargetAsync: async () =>
        makeDesignTokenDefinitionGraph({
          referenceName: "--brand",
          winnerDeclarationFilePath: generatedTokensPath,
          winnerDeclarationRange: winnerRange,
        }),
    });

    expect(targets).toEqual([
      {
        originRange: {
          start: { line: 1, character: 13 },
          end: { line: 1, character: 20 },
        },
        targetFilePath: generatedTokensPath,
        targetRange: winnerRange,
        targetSelectionRange: winnerRange,
      },
    ]);
  });

  it("resolves Sass @use source tokens to module files with partial candidates", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "./t/*|*/okens.module";

.button {
  color: tokens.$gap;
}
`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      originRange: {
        start: { line: 0, character: 6 },
        end: { line: 0, character: 21 },
      },
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 0 },
      },
    });
  });

  it("resolves Sass @use source tokens through path aliases", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@s/*|*/tyles/tokens.module" as tokens;

.button {
  color: tokens.$gap;
}
`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;`,
    });
    const targets = resolveStyleDefinitionTargets(
      styleTarget(ws),
      styleDeps(ws, {
        aliasResolver: new AliasResolver("/fake/workspace", {
          "@styles": "src",
        }),
      }),
    );

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      originRange: {
        start: { line: 0, character: 6 },
        end: { line: 0, character: 27 },
      },
      targetFilePath: TOKENS_PARTIAL_PATH,
    });
  });

  it("resolves Sass @use source tokens through the first existing tsconfig path target", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@s/*|*/tyles/tokens.module" as tokens;

.button {
  color: tokens.$gap;
}
`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;`,
    });
    const targets = resolveStyleDefinitionTargets(
      styleTarget(ws),
      styleDeps(ws, {
        aliasResolver: new AliasResolver(
          "/fake/workspace",
          {},
          {
            basePath: "/fake/workspace",
            paths: {
              "@styles/*": ["missing/*", "src/*"],
            },
          },
        ),
      }),
    );

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
    });
  });

  it("does not resolve built-in Sass modules as workspace files", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "s/*|*/ass:color";
`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toEqual([]);
  });

  it("resolves namespace-qualified Sass members to declarations in @use targets", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "./tokens.module" as tokens;

.button {
  color: tokens.$g/*at:variable*/ap;
  @include tokens.r/*at:mixin*/aised;
  border-color: tokens.t/*at:function*/one(tokens.$gap);
}
`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 7 },
        end: { line: 1, character: 13 },
      },
    });

    const functionTargets = resolveStyleDefinitionTargets(styleTarget(ws, "function"), deps);
    expect(functionTargets).toHaveLength(1);
    expect(functionTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 2, character: 10 },
        end: { line: 2, character: 14 },
      },
    });
  });

  it("resolves wildcard Sass module members to declarations in @use targets", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "./tokens.module" as *;

.button {
  color: $g/*at:variable*/ap;
  @include r/*at:mixin*/aised;
  border-color: t/*at:function*/one($gap);
}
`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 7 },
        end: { line: 1, character: 13 },
      },
    });

    const functionTargets = resolveStyleDefinitionTargets(styleTarget(ws, "function"), deps);
    expect(functionTargets).toHaveLength(1);
    expect(functionTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 2, character: 10 },
        end: { line: 2, character: 14 },
      },
    });
  });

  it("resolves legacy @import Sass symbols as wildcard module members", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@import "./tokens.module";

.button {
  color: $g/*|*/ap;
}
`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;`,
    });

    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });
  });

  it("resolves package Sass imports from node_modules", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/foundation/scss/utils" as *;

.button {
  color: $g/*at:variable*/ray-900;
  @include t/*at:mixin*/ypo-18;
}
`,
      [PACKAGE_UTILS_PATH]: `$gray-900: #111;
@mixin typo-18 {}
`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_UTILS_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 9 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_UTILS_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 7 },
        end: { line: 1, character: 14 },
      },
    });
  });

  it("resolves package root Sass imports through package.json sass entries", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens" as *;

.button {
  color: $g/*at:variable*/ray700;
  @include t/*at:mixin*/ypography16;
}
`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"sass":"src/index.scss"}`,
      [PACKAGE_TOKENS_INDEX_PATH]: `$gray700: #767678;
@mixin typography16 {}
`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_INDEX_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_INDEX_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 7 },
        end: { line: 1, character: 19 },
      },
    });
  });

  it("resolves package root Sass imports through index fallback when manifest has no style entry", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens" as *;

.button {
  color: $g/*at:variable*/ray700;
}
`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"name":"@design/tokens"}`,
      [PACKAGE_TOKENS_INDEX_PATH]: `$gray700: #767678;`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_INDEX_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });
  });

  it("resolves package Sass subpath imports through package.json exports entries", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens/colors" as *;

.button {
  color: $g/*at:variable*/ray700;
}
`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"exports":{"./colors":{"sass":"./src/colors.scss"}}}`,
      [PACKAGE_TOKENS_COLORS_ENTRY_PATH]: `$gray700: #767678;`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_COLORS_ENTRY_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });
  });

  it("resolves package Sass subpath imports through package.json exports patterns", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens/colors" as *;

.button {
  color: $g/*at:variable*/ray700;
}
`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"exports":{"./*":{"sass":"./src/*.scss"}}}`,
      [PACKAGE_TOKENS_COLORS_ENTRY_PATH]: `$gray700: #767678;`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_COLORS_ENTRY_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });
  });

  it("resolves extensionless package Sass subpath exports through Sass candidates", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens/colors" as *;

.button {
  color: $g/*at:variable*/ray700;
}
`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"exports":{"./colors":{"sass":"./src/colors"}}}`,
      [PACKAGE_TOKENS_COLORS_ENTRY_PATH]: `$gray700: #767678;`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_COLORS_ENTRY_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });
  });

  it("resolves package Sass subpath exports from fallback arrays", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "@design/tokens/colors" as *;

.button {
  color: $g/*at:variable*/ray700;
}
`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"exports":{"./colors":[{"sass":"./src/colors.scss"},"./fallback.scss"]}}`,
      [PACKAGE_TOKENS_COLORS_ENTRY_PATH]: `$gray700: #767678;`,
    });
    const targets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_COLORS_ENTRY_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });
  });

  it("resolves Sass members forwarded through @use targets", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "./theme.module" as *;

.button {
  color: $g/*at:variable*/ap;
  border-color: t/*at:function*/one($gap);
}
`,
      [THEME_PATH]: `@forward "./tokens.module";`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;
@function tone($value) { @return $value; }
`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });

    const functionTargets = resolveStyleDefinitionTargets(styleTarget(ws, "function"), deps);
    expect(functionTargets).toHaveLength(1);
    expect(functionTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 10 },
        end: { line: 1, character: 14 },
      },
    });
  });

  it("resolves namespace-qualified Sass members forwarded through @use targets", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "./theme.module" as theme;

.button {
  color: theme.$g/*|*/ap;
}
`,
      [THEME_PATH]: `@forward "./tokens.module";`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;`,
    });

    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });
  });

  it("resolves prefixed and filtered Sass members forwarded through @use targets", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "./theme.module" as *;

.button {
  color: $th/*at:variable*/eme-gap;
  border-color: theme-t/*at:function*/one($theme-gap);
  margin: $theme-s/*at:hidden*/ecret;
}
`,
      [THEME_PATH]: `@forward "./tokens.module" as theme-* show $gap, tone;`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;
$secret: 2rem;
@function tone($value) { @return $value; }
`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });

    const functionTargets = resolveStyleDefinitionTargets(styleTarget(ws, "function"), deps);
    expect(functionTargets).toHaveLength(1);
    expect(functionTargets[0]).toMatchObject({
      targetFilePath: TOKENS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 2, character: 10 },
        end: { line: 2, character: 14 },
      },
    });

    expect(resolveStyleDefinitionTargets(styleTarget(ws, "hidden"), deps)).toEqual([]);
  });

  it("resolves prefixed Sass members forwarded from package targets through a local utility module", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "utils" as *;

.title {
  color: $ds_g/*at:variable*/ray700;
  @include ds_t/*at:mixin*/ypography16;
}
`,
      [UTILS_PATH]: `@forward "@design/tokens/colors" as ds_*;
@forward "@design/tokens/typography" as ds_*;
`,
      [PACKAGE_COLORS_PATH]: `$gray700: #767678;`,
      [PACKAGE_TYPOGRAPHY_PATH]: `@mixin typography16 {}`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_COLORS_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TYPOGRAPHY_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 7 },
        end: { line: 0, character: 19 },
      },
    });
  });

  it("resolves prefixed Sass members forwarded from package export patterns through a local utility module", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "utils" as *;

.title {
  color: $ds_g/*at:variable*/ray700;
  @include ds_t/*at:mixin*/ypography16;
}
`,
      [UTILS_PATH]: `@forward "@design/tokens/colors" as ds_*;
@forward "@design/tokens/typography" as ds_*;
`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"exports":{"./*":{"sass":"./src/*.scss"}}}`,
      [PACKAGE_TOKENS_COLORS_ENTRY_PATH]: `$gray700: #767678;`,
      [PACKAGE_TOKENS_TYPOGRAPHY_ENTRY_PATH]: `@mixin typography16 {}`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_COLORS_ENTRY_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_TYPOGRAPHY_ENTRY_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 7 },
        end: { line: 0, character: 19 },
      },
    });
  });

  it("resolves prefixed Sass members forwarded from a package root through a local utility module", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "utils" as *;

.title {
  color: $ds_g/*at:variable*/ray700;
  @include ds_t/*at:mixin*/ypography16;
}
`,
      [UTILS_PATH]: `@forward "@design/tokens" as ds_*;`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"sass":"src/index.scss"}`,
      [PACKAGE_TOKENS_INDEX_PATH]: `$gray700: #767678;
@mixin typography16 {}
`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_INDEX_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_INDEX_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 7 },
        end: { line: 1, character: 19 },
      },
    });
  });

  it("resolves prefixed Sass members through package-root internal forward chains", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "utils" as *;

.title {
  color: $ds_g/*at:variable*/ray700;
  @include ds_t/*at:mixin*/ypography16;
}
`,
      [UTILS_PATH]: `@forward "@design/tokens" as ds_*;`,
      [PACKAGE_TOKENS_JSON_PATH]: `{"sass":"src/index.scss"}`,
      [PACKAGE_TOKENS_INDEX_PATH]: `@forward "./colors";
@forward "./typography";
`,
      [PACKAGE_TOKENS_SRC_COLORS_PARTIAL_PATH]: `$gray700: #767678;`,
      [PACKAGE_TOKENS_SRC_TYPOGRAPHY_PARTIAL_PATH]: `@mixin typography16 {}`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_SRC_COLORS_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 8 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: PACKAGE_TOKENS_SRC_TYPOGRAPHY_PARTIAL_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 7 },
        end: { line: 0, character: 19 },
      },
    });
  });

  it("resolves same-file Sass symbol references to declarations", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `$gap: 1rem;
@mixin raised() {}
.button {
  color: $g/*at:variable*/ap;
  @include r/*at:mixin*/aised();
}
`,
    });
    const deps = styleDeps(ws);

    const variableTargets = resolveStyleDefinitionTargets(styleTarget(ws, "variable"), deps);
    expect(variableTargets).toHaveLength(1);
    expect(variableTargets[0]).toMatchObject({
      targetFilePath: BUTTON_PATH,
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });

    const mixinTargets = resolveStyleDefinitionTargets(styleTarget(ws, "mixin"), deps);
    expect(mixinTargets).toHaveLength(1);
    expect(mixinTargets[0]).toMatchObject({
      targetFilePath: BUTTON_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 7 },
        end: { line: 1, character: 13 },
      },
    });
  });

  it("resolves Less variable references to the nearest scoped declaration", () => {
    const lessPath = BUTTON_PATH.replace(".scss", ".less");
    const ws = styleWorkspace({
      [lessPath]: `@gap: 1rem;
.card {
  @gap: 2rem;
  color: @g/*at:local*/ap;
}
.other {
  color: @g/*at:file*/ap;
}
`,
    });
    const deps = styleDeps(ws);

    const localTargets = resolveStyleDefinitionTargets(styleTarget(ws, "local"), deps);
    expect(localTargets).toHaveLength(1);
    expect(localTargets[0]).toMatchObject({
      targetSelectionRange: {
        start: { line: 2, character: 2 },
        end: { line: 2, character: 6 },
      },
    });

    const fileScopeTargets = resolveStyleDefinitionTargets(styleTarget(ws, "file"), deps);
    expect(fileScopeTargets).toHaveLength(1);
    expect(fileScopeTargets[0]).toMatchObject({
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });
  });

  it("prefers local Sass variable declarations over same-name file-scope declarations", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `$gap: 1rem;
.one {
  $gap: 2rem;
  color: $g/*at:local*/ap;
}
.two {
  color: $g/*at:file*/ap;
}
`,
    });
    const deps = styleDeps(ws);

    const localTargets = resolveStyleDefinitionTargets(styleTarget(ws, "local"), deps);
    expect(localTargets).toHaveLength(1);
    expect(localTargets[0]).toMatchObject({
      targetSelectionRange: {
        start: { line: 2, character: 2 },
        end: { line: 2, character: 6 },
      },
    });

    const fileScopeTargets = resolveStyleDefinitionTargets(styleTarget(ws, "file"), deps);
    expect(fileScopeTargets).toHaveLength(1);
    expect(fileScopeTargets[0]).toMatchObject({
      targetSelectionRange: {
        start: { line: 0, character: 0 },
        end: { line: 0, character: 4 },
      },
    });
  });

  it("prefers local Sass declarations over wildcard module members", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `@use "./tokens.module" as *;
$gap: 2rem;

.button {
  color: $g/*|*/ap;
}
`,
      [TOKENS_PARTIAL_PATH]: `$gap: 1rem;`,
    });

    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toHaveLength(1);
    expect(targets[0]).toMatchObject({
      targetFilePath: BUTTON_PATH,
      targetSelectionRange: {
        start: { line: 1, character: 0 },
        end: { line: 1, character: 4 },
      },
    });
  });

  it("does not resolve Sass variables to declarations from another local scope", () => {
    const ws = styleWorkspace({
      [BUTTON_PATH]: `.one {
  $gap: 1rem;
}
.two {
  color: $g/*|*/ap;
}
`,
    });

    const targets = resolveStyleDefinitionTargets(styleTarget(ws), styleDeps(ws));

    expect(targets).toEqual([]);
  });
});

function styleWorkspace(files: Record<string, string>): CmeWorkspace {
  return workspace(files);
}

function styleTarget(ws: CmeWorkspace, markerName = "cursor", filePath?: string) {
  return targetFixture({ workspace: ws, markerName, filePath });
}

function styleDeps(ws: CmeWorkspace, options: { readonly aliasResolver?: AliasResolver } = {}) {
  return depsForDocuments(
    ws.filePaths.map((filePath) => parseStyleDocument(ws.file(filePath).content, filePath)),
    {
      ...options,
      readStyleFile: (filePath) =>
        ws.filePaths.includes(filePath) ? ws.file(filePath).content : null,
    },
  );
}

function depsForDocuments(
  documents: readonly StyleDocumentHIR[],
  options: {
    readonly aliasResolver?: AliasResolver;
    readonly readStyleFile?: (filePath: string) => string | null;
  } = {},
) {
  const byPath = new Map(documents.map((document) => [document.filePath, document]));
  const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
  for (const document of documents) {
    styleDependencyGraph.record(document.filePath, document);
  }
  return makeBaseDeps({
    aliasResolver: options.aliasResolver ?? EMPTY_ALIAS_RESOLVER,
    styleDocumentForPath: (filePath: string) => byPath.get(filePath) ?? null,
    styleDependencyGraph,
    workspaceRoot: "/fake/workspace",
    readStyleFile: options.readStyleFile ?? (() => null),
  });
}

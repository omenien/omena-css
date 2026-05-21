import { describe, expect, it } from "vitest";
import { WorkspaceStyleDependencyGraph } from "../../../server/engine-core-ts/src/core/semantic/style-dependency-graph";
import { parseStyleDocument } from "../../../server/engine-core-ts/src/core/scss/scss-parser";
import { resolveStyleCompletionItems } from "../../../server/engine-host-node/src/style-completion-query";

const SCSS_PATH = "/fake/src/Button.module.scss";
const THEME_PATH = "/fake/src/theme.module.scss";
const TOKENS_PATH = "/fake/src/tokens.module.scss";
const PACKAGE_TOKENS_ROOT = "/fake/node_modules/@design/tokens";
const PACKAGE_TOKENS_JSON_PATH = `${PACKAGE_TOKENS_ROOT}/package.json`;
const PACKAGE_TOKENS_INDEX_PATH = `${PACKAGE_TOKENS_ROOT}/src/index.scss`;
const PACKAGE_COLORS_SCSS_PATH = `${PACKAGE_TOKENS_ROOT}/src/colors.scss`;
const PACKAGE_COLORS_CSS_PATH = `${PACKAGE_TOKENS_ROOT}/src/colors.css`;
const PACKAGE_TYPOGRAPHY_SCSS_PATH = `${PACKAGE_TOKENS_ROOT}/src/typography.scss`;
const PACKAGE_VARIABLES_CSS_PATH = "/fake/node_modules/@design/tokens/variables.css";

describe("resolveStyleCompletionItems", () => {
  it("returns same-file CSS custom property completions inside `var()`", () => {
    const scss = `:root { --brand: #0af; --surface: white; }
.button {
  color: var(--br)
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 2,
      character: 17,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]).toMatchObject({
      detail: "CSS custom property",
      insertText: "--brand",
      replacementRange: {
        start: { line: 2, character: 13 },
        end: { line: 2, character: 17 },
      },
      symbolKind: "customProperty",
    });
  });

  it("returns workspace CSS custom property completions inside `var()`", () => {
    const scss = `.button {
  color: var(--)
}
`;
    const tokensScss = `:root { --brand: #0af; --space: 1rem; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(TOKENS_PATH, tokensDocument);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 1,
      character: 15,
      styleDocument,
      styleDependencyGraph,
    });

    expect(result.map((item) => item.label)).toEqual(["--brand", "--space"]);
  });

  it("uses matching workspace theme context for duplicate CSS custom property completions", () => {
    const scss = `.theme .button {
  color: var(--br)
}
`;
    const baseTokensScss = `:root { --brand: #111; }`;
    const themeTokensScss = `.theme { --brand: #222; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const baseTokensDocument = parseStyleDocument(baseTokensScss, TOKENS_PATH);
    const themeTokensDocument = parseStyleDocument(themeTokensScss, THEME_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(TOKENS_PATH, baseTokensDocument);
    styleDependencyGraph.record(THEME_PATH, themeTokensDocument);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 1,
      character: 17,
      styleDocument,
      styleDependencyGraph,
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceFilePath).toBe(THEME_PATH);
  });

  it("uses matching theme context for duplicate same-file CSS custom property completions", () => {
    const scss = `:root {
  --brand: #111;
}
.theme { --brand: #222; }
.theme .button {
  color: var(--br)
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 5,
      character: 17,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceRange).toMatchObject({
      start: { line: 3, character: 9 },
      end: { line: 3, character: 16 },
    });
  });

  it("uses matching data-theme context for duplicate CSS custom property completions", () => {
    const scss = `:root {
  --brand: #111;
}
:root[data-theme="dark"] {
  --brand: #222;
}
[data-theme="dark"] .button {
  color: var(--br)
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 7,
      character: 17,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceRange).toMatchObject({
      start: { line: 4, character: 2 },
      end: { line: 4, character: 9 },
    });
  });

  it("keeps root CSS custom property completions ahead of unrelated theme overrides", () => {
    const scss = `:root {
  --brand: #111;
}
.theme { --brand: #222; }
.button {
  color: var(--br)
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 5,
      character: 17,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceRange).toMatchObject({
      start: { line: 1, character: 2 },
      end: { line: 1, character: 9 },
    });
  });

  it("keeps root CSS custom property completions ahead of unmatched media overrides", () => {
    const scss = `:root {
  --brand: #111;
}
@media (min-width: 600px) {
  :root { --brand: #222; }
}
.button {
  color: var(--br)
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 7,
      character: 17,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceRange).toMatchObject({
      start: { line: 1, character: 2 },
      end: { line: 1, character: 9 },
    });
  });

  it("prefers local CSS custom property completions over workspace duplicates", () => {
    const scss = `:root {
  --brand: #111;
}
.button {
  color: var(--br)
}
`;
    const tokensScss = `:root { --brand: #222; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(TOKENS_PATH, tokensDocument);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 4,
      character: 17,
      styleDocument,
      styleDependencyGraph,
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceFilePath).toBe(SCSS_PATH);
  });

  it("prefers imported CSS custom property completions over workspace duplicates", () => {
    const scss = `@use "@design/tokens/variables.css";

.button {
  color: var(--br)
}
`;
    const importedTokensCss = `:root { --brand: #111; }`;
    const workspaceTokensCss = `:root { --brand: #222; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const importedTokensDocument = parseStyleDocument(
      importedTokensCss,
      PACKAGE_VARIABLES_CSS_PATH,
    );
    const workspaceTokensDocument = parseStyleDocument(workspaceTokensCss, TOKENS_PATH);
    const styleDependencyGraph = new WorkspaceStyleDependencyGraph();
    styleDependencyGraph.record(TOKENS_PATH, workspaceTokensDocument);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 17,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, importedTokensDocument]),
      styleDependencyGraph,
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceFilePath).toBe(PACKAGE_VARIABLES_CSS_PATH);
  });

  it("uses Rust design-token declaration candidates for CSS custom property completions", () => {
    const scss = `.button {
  color: var(--br)
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 1,
      character: 17,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
      designTokenDeclarationCandidates: [
        {
          name: "--brand",
          sourceOrder: 0,
          filePath: PACKAGE_VARIABLES_CSS_PATH,
          range: {
            start: { line: 0, character: 8 },
            end: { line: 0, character: 15 },
          },
          selectorContexts: [":root"],
          underMedia: false,
          underSupports: false,
          underLayer: false,
          candidateScope: "cross-file-import-candidate",
          importGraphDistance: 1,
          importGraphOrder: 0,
        },
      ],
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]).toMatchObject({
      sourceFilePath: PACKAGE_VARIABLES_CSS_PATH,
      sourceRange: {
        start: { line: 0, character: 8 },
        end: { line: 0, character: 15 },
      },
    });
  });

  it("uses matching wrapper context for incomplete CSS custom property completions", () => {
    const scss = `:root { --brand: #222; }
@media (min-width: 600px) {
  :root { --brand: #111; }
}
@media (min-width: 600px) {
  .button {
    color: var(--);
  }
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 6,
      character: 17,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceRange).toMatchObject({
      start: { line: 2, character: 10 },
      end: { line: 2, character: 17 },
    });
  });

  it("falls back to local CSS custom properties when the current document is incomplete", () => {
    const scss = `:root { --brand: #0af; }
.button {
  color: var(--`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 2,
      character: 15,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceRange).toMatchObject({
      start: { line: 0, character: 8 },
      end: { line: 0, character: 15 },
    });
  });

  it("returns imported package CSS custom property completions inside `var()`", () => {
    const scss = `@use "@design/tokens/variables.css";

.button {
  color: var(--)
}
`;
    const tokensCss = `:root { --color-gray-700: #767678; --spacing-md: 16px; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument]),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700", "--spacing-md"]);
  });

  it("uses matching package theme context for duplicate CSS custom property completions", () => {
    const baseCssPath = `${PACKAGE_TOKENS_ROOT}/base.css`;
    const themeCssPath = `${PACKAGE_TOKENS_ROOT}/theme.css`;
    const scss = `@use "@design/tokens/base.css";
@use "@design/tokens/theme.css";

.theme .button {
  color: var(--br)
}
`;
    const baseTokensCss = `:root { --brand: #111; }`;
    const themeTokensCss = `.theme { --brand: #222; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const baseTokensDocument = parseStyleDocument(baseTokensCss, baseCssPath);
    const themeTokensDocument = parseStyleDocument(themeTokensCss, themeCssPath);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 4,
      character: 17,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([
        styleDocument,
        baseTokensDocument,
        themeTokensDocument,
      ]),
    });

    expect(result.map((item) => item.label)).toEqual(["--brand"]);
    expect(result[0]?.sourceFilePath).toBe(themeCssPath);
  });

  it("returns package-root CSS custom property completions through package.json exports", () => {
    const scss = `@use "@design/tokens";

.button {
  color: var(--)
}
`;
    const packageJson = `{"exports":{".":{"style":"./variables.css"}}}`;
    const tokensCss = `:root { --color-gray-700: #767678; --spacing-md: 16px; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700", "--spacing-md"]);
  });

  it("respects package export condition object order for package-root completions", () => {
    const scss = `@use "@design/tokens";

.button {
  color: var(--)
}
`;
    const packageJson = `{"exports":{".":{"style":"./variables.css","sass":"./src/index.scss"}}}`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const tokensScss = `$gray700: #767678;`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensCssDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);
    const tokensScssDocument = parseStyleDocument(tokensScss, PACKAGE_TOKENS_INDEX_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([
        styleDocument,
        tokensCssDocument,
        tokensScssDocument,
      ]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700"]);
    expect(result[0]?.sourceFilePath).toBe(PACKAGE_VARIABLES_CSS_PATH);
  });

  it("prefers package exports over legacy top-level package style fields", () => {
    const scss = `@use "@design/tokens";

.button {
  color: var(--)
}
`;
    const packageJson = `{"style":"legacy.css","exports":{".":{"style":"./variables.css"}}}`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const legacyCss = `:root { --legacy-only: #000; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);
    const legacyDocument = parseStyleDocument(legacyCss, `${PACKAGE_TOKENS_ROOT}/legacy.css`);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument, legacyDocument]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700"]);
    expect(result[0]?.sourceFilePath).toBe(PACKAGE_VARIABLES_CSS_PATH);
  });

  it("returns package-root CSS custom property completions through Sass pkg URLs", () => {
    const scss = `@use "pkg:@design/tokens";

.button {
  color: var(--)
}
`;
    const packageJson = `{"exports":{".":{"style":"./variables.css"}}}`;
    const tokensCss = `:root { --color-gray-700: #767678; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700"]);
    expect(result[0]?.sourceFilePath).toBe(PACKAGE_VARIABLES_CSS_PATH);
  });

  it("returns CSS custom property completions forwarded from a package root through a local utility module", () => {
    const scss = `@use "utils" as *;

.button {
  color: var(--)
}
`;
    const utilsScss = `@forward "@design/tokens" as ds_*;`;
    const packageJson = `{"style":"variables.css"}`;
    const tokensCss = `:root { --color-gray-700: #767678; --spacing-md: 16px; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const utilsDocument = parseStyleDocument(utilsScss, "/fake/src/_utils.scss");
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_VARIABLES_CSS_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, utilsDocument, tokensDocument]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700", "--spacing-md"]);
  });

  it("returns CSS custom property completions through package.json subpath exports", () => {
    const scss = `@use "@design/tokens/colors";

.button {
  color: var(--)
}
`;
    const packageJson = `{"exports":{"./colors":{"style":"./src/colors.css"}}}`;
    const tokensCss = `:root { --color-gray-700: #767678; --spacing-md: 16px; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_COLORS_CSS_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700", "--spacing-md"]);
  });

  it("returns CSS custom property completions through package.json subpath export patterns", () => {
    const scss = `@use "@design/tokens/colors";

.button {
  color: var(--)
}
`;
    const packageJson = `{"exports":{"./*":{"style":"./src/*.css"}}}`;
    const tokensCss = `:root { --color-gray-700: #767678; --spacing-md: 16px; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_COLORS_CSS_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700", "--spacing-md"]);
  });

  it("returns CSS custom property completions through extensionless package subpath exports", () => {
    const scss = `@use "@design/tokens/colors";

.button {
  color: var(--)
}
`;
    const packageJson = `{"exports":{"./colors":{"style":"./src/colors"}}}`;
    const tokensCss = `:root { --color-gray-700: #767678; --spacing-md: 16px; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_COLORS_CSS_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700", "--spacing-md"]);
  });

  it("returns CSS custom property completions through package export fallback arrays", () => {
    const scss = `@use "@design/tokens/colors";

.button {
  color: var(--)
}
`;
    const packageJson = `{"exports":{"./colors":[{"style":"./src/colors.css"},"./fallback.css"]}}`;
    const tokensCss = `:root { --color-gray-700: #767678; --spacing-md: 16px; }`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensCss, PACKAGE_COLORS_CSS_PATH);

    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 15,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument]),
      readFile: (filePath) => (filePath === PACKAGE_TOKENS_JSON_PATH ? packageJson : null),
    });

    expect(result.map((item) => item.label)).toEqual(["--color-gray-700", "--spacing-md"]);
  });

  it("returns same-file Sass variable completions after `$`", () => {
    const scss = `$gap: 1rem;
@mixin raised($depth) {
  box-shadow: 0 0 $depth black;
}
.button {
  color: $
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 5,
      character: 10,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["$gap"]);
    expect(result[0]).toMatchObject({
      insertText: "$gap",
      replacementRange: {
        start: { line: 5, character: 9 },
        end: { line: 5, character: 10 },
      },
    });
  });

  it("returns same-file Less variable completions after `@` in values", () => {
    const less = `@gap: 1rem;
@tone: #fff;
.button {
  color: @
}
`;
    const result = resolveStyleCompletionItems({
      content: less,
      line: 3,
      character: 10,
      styleDocument: parseStyleDocument(less, SCSS_PATH.replace(".scss", ".less")),
    });

    expect(result.map((item) => item.label)).toEqual(["@gap", "@tone"]);
    expect(result[0]).toMatchObject({
      detail: "Less variable",
      insertText: "@gap",
      replacementRange: {
        start: { line: 3, character: 9 },
        end: { line: 3, character: 10 },
      },
    });
  });

  it("returns same-file Sass mixin completions after `@include`", () => {
    const scss = `@mixin raised() {}
.button {
  @include ra
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 2,
      character: 13,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["raised"]);
    expect(result[0]).toMatchObject({
      insertText: "raised",
      replacementRange: {
        start: { line: 2, character: 11 },
        end: { line: 2, character: 13 },
      },
    });
  });

  it("returns same-file Sass function completions in declaration values", () => {
    const scss = `@function tone($value) { @return $value; }
.button {
  color: to
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 2,
      character: 11,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["tone"]);
    expect(result[0]).toMatchObject({
      insertText: "tone",
      replacementRange: {
        start: { line: 2, character: 9 },
        end: { line: 2, character: 11 },
      },
    });
  });

  it("returns wildcard Sass module completions from @use targets", () => {
    const scss = `@use "./tokens.module" as *;

.button {
  color: $;
  @include ra;
  border-color: to;
}
`;
    const tokensScss = `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const targetDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const styleDocumentForPath = styleDocumentMap([styleDocument, targetDocument]);

    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 3,
        character: 10,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["$gap"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 4,
        character: 13,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["raised"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 5,
        character: 18,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["tone"]);
  });

  it("returns completions forwarded through wildcard @use targets", () => {
    const scss = `@use "./theme.module" as *;

.button {
  color: $;
  @include ra;
  border-color: to;
}
`;
    const themeScss = `@forward "./tokens.module";`;
    const tokensScss = `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const themeDocument = parseStyleDocument(themeScss, THEME_PATH);
    const targetDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const styleDocumentForPath = styleDocumentMap([styleDocument, themeDocument, targetDocument]);

    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 3,
        character: 10,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["$gap"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 4,
        character: 13,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["raised"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 5,
        character: 18,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["tone"]);
  });

  it("returns prefixed completions filtered by forwarded wildcard @use targets", () => {
    const scss = `@use "./theme.module" as *;

.button {
  color: $theme;
  @include theme-ra;
  border-color: theme-to;
}
`;
    const themeScss = `@forward "./tokens.module" as theme-* show $gap, raised, tone;`;
    const tokensScss = `$gap: 1rem;
$secret: 2rem;
@mixin raised() {}
@function tone($value) { @return $value; }
@function hidden($value) { @return $value; }
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const themeDocument = parseStyleDocument(themeScss, THEME_PATH);
    const targetDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const styleDocumentForPath = styleDocumentMap([styleDocument, themeDocument, targetDocument]);

    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 3,
        character: 15,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["$theme-gap"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 4,
        character: 19,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["theme-raised"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 5,
        character: 24,
        styleDocument,
        styleDocumentForPath,
      }).map((item) => item.label),
    ).toEqual(["theme-tone"]);
  });

  it("returns prefixed completions forwarded from a package root through a local utility module", () => {
    const scss = `@use "utils" as *;

.title {
  color: $ds_;
  @include ds_typo;
}
`;
    const utilsScss = `@forward "@design/tokens" as ds_*;`;
    const tokensScss = `$gray700: #767678;
@mixin typography16 {}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const utilsDocument = parseStyleDocument(utilsScss, "/fake/src/_utils.scss");
    const tokensDocument = parseStyleDocument(tokensScss, PACKAGE_TOKENS_INDEX_PATH);
    const styleDocumentForPath = styleDocumentMap([styleDocument, utilsDocument, tokensDocument]);

    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 3,
        character: 13,
        styleDocument,
        styleDocumentForPath,
        readFile: (filePath) =>
          filePath === PACKAGE_TOKENS_JSON_PATH ? `{"sass":"src/index.scss"}` : null,
      }).map((item) => item.label),
    ).toEqual(["$ds_gray700"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 4,
        character: 18,
        styleDocument,
        styleDocumentForPath,
        readFile: (filePath) =>
          filePath === PACKAGE_TOKENS_JSON_PATH ? `{"sass":"src/index.scss"}` : null,
      }).map((item) => item.label),
    ).toEqual(["ds_typography16"]);
  });

  it("returns prefixed completions forwarded from package export patterns", () => {
    const scss = `@use "utils" as *;

.title {
  color: $ds_;
  @include ds_typo;
}
`;
    const utilsScss = `@forward "@design/tokens/colors" as ds_*;
@forward "@design/tokens/typography" as ds_*;
`;
    const colorsScss = `$gray700: #767678;`;
    const typographyScss = `@mixin typography16 {}`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const utilsDocument = parseStyleDocument(utilsScss, "/fake/src/_utils.scss");
    const colorsDocument = parseStyleDocument(colorsScss, PACKAGE_COLORS_SCSS_PATH);
    const typographyDocument = parseStyleDocument(typographyScss, PACKAGE_TYPOGRAPHY_SCSS_PATH);
    const styleDocumentForPath = styleDocumentMap([
      styleDocument,
      utilsDocument,
      colorsDocument,
      typographyDocument,
    ]);
    const readFile = (filePath: string) =>
      filePath === PACKAGE_TOKENS_JSON_PATH ? `{"exports":{"./*":{"sass":"./src/*.scss"}}}` : null;

    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 3,
        character: 13,
        styleDocument,
        styleDocumentForPath,
        readFile,
      }).map((item) => item.label),
    ).toEqual(["$ds_gray700"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 4,
        character: 18,
        styleDocument,
        styleDocumentForPath,
        readFile,
      }).map((item) => item.label),
    ).toEqual(["ds_typography16"]);
  });

  it("returns prefixed completions through package-root internal forward chains", () => {
    const scss = `@use "utils" as *;

.title {
  color: $ds_;
  @include ds_typo;
}
`;
    const utilsScss = `@forward "@design/tokens" as ds_*;`;
    const indexScss = `@forward "./colors";
@forward "./typography";
`;
    const colorsScss = `$gray700: #767678;`;
    const typographyScss = `@mixin typography16 {}`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const utilsDocument = parseStyleDocument(utilsScss, "/fake/src/_utils.scss");
    const indexDocument = parseStyleDocument(indexScss, PACKAGE_TOKENS_INDEX_PATH);
    const colorsDocument = parseStyleDocument(colorsScss, PACKAGE_COLORS_SCSS_PATH);
    const typographyDocument = parseStyleDocument(typographyScss, PACKAGE_TYPOGRAPHY_SCSS_PATH);
    const styleDocumentForPath = styleDocumentMap([
      styleDocument,
      utilsDocument,
      indexDocument,
      colorsDocument,
      typographyDocument,
    ]);
    const readFile = (filePath: string) =>
      filePath === PACKAGE_TOKENS_JSON_PATH ? `{"sass":"src/index.scss"}` : null;

    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 3,
        character: 13,
        styleDocument,
        styleDocumentForPath,
        readFile,
      }).map((item) => item.label),
    ).toEqual(["$ds_gray700"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 4,
        character: 18,
        styleDocument,
        styleDocumentForPath,
        readFile,
      }).map((item) => item.label),
    ).toEqual(["ds_typography16"]);
  });

  it("returns same-file CSS Modules @value completions in declaration values", () => {
    const scss = `@value primary: #ff3355;
@value spacing: 8px;
.button {
  color: pri;
  margin: spa;
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);

    const colorResult = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 12,
      styleDocument,
    });
    const marginResult = resolveStyleCompletionItems({
      content: scss,
      line: 4,
      character: 13,
      styleDocument,
    });

    expect(colorResult.map((item) => item.label)).toEqual(["primary"]);
    expect(colorResult[0]).toMatchObject({
      detail: "CSS Modules @value",
      insertText: "primary",
      replacementRange: {
        start: { line: 3, character: 9 },
        end: { line: 3, character: 12 },
      },
      symbolKind: "value",
    });
    expect(marginResult.map((item) => item.label)).toEqual(["spacing"]);
  });

  it("returns imported CSS Modules @value alias completions", () => {
    const scss = `@value secondary as accent from "./tokens.module.scss";
.button {
  color: acc;
}
`;
    const tokensScss = `@value secondary: #ff3355;`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const tokensDocument = parseStyleDocument(tokensScss, TOKENS_PATH);
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 2,
      character: 12,
      styleDocument,
      styleDocumentForPath: styleDocumentMap([styleDocument, tokensDocument]),
    });

    expect(result.map((item) => item.label)).toEqual(["accent"]);
    expect(result[0]).toMatchObject({
      detail: "CSS Modules @value import (secondary from ./tokens.module.scss)",
      insertText: "accent",
      sourceFilePath: TOKENS_PATH,
      symbolKind: "value",
    });
  });

  it("returns CSS Modules @value completions inside local @value declaration values", () => {
    const scss = `@value primary: #ff3355;
@value accent: pri;
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 1,
      character: 18,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["primary"]);
  });

  it("keeps Sass function completions alongside CSS Modules @value completions", () => {
    const scss = `@function tone($value) { @return $value; }
@value toneColor: #ff3355;
.button {
  color: to;
}
`;
    const result = resolveStyleCompletionItems({
      content: scss,
      line: 3,
      character: 11,
      styleDocument: parseStyleDocument(scss, SCSS_PATH),
    });

    expect(result.map((item) => item.label)).toEqual(["tone", "toneColor"]);
  });

  it("keeps Sass parameter variables local to their callable body", () => {
    const scss = `$gap: 1rem;
@mixin raised($depth) {
  box-shadow: $
}
.button {
  color: $
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);
    const insideMixin = resolveStyleCompletionItems({
      content: scss,
      line: 2,
      character: 16,
      styleDocument,
    });
    const outsideMixin = resolveStyleCompletionItems({
      content: scss,
      line: 5,
      character: 10,
      styleDocument,
    });

    expect(insideMixin.map((item) => item.label)).toEqual(["$depth", "$gap"]);
    expect(outsideMixin.map((item) => item.label)).toEqual(["$gap"]);
  });

  it("falls back to raw Sass declarations when the style document is mid-edit invalid", () => {
    const scss = `$gap: 1rem;
@mixin raised() {}
@function tone($value) { @return $value; }
.button {
  color: $
  @include ra
  border-color: to
}
`;
    const styleDocument = parseStyleDocument(scss, SCSS_PATH);

    expect(styleDocument.sassSymbolDecls).toEqual([]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 4,
        character: 10,
        styleDocument,
      }).map((item) => item.label),
    ).toEqual(["$gap"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 5,
        character: 13,
        styleDocument,
      }).map((item) => item.label),
    ).toEqual(["raised"]);
    expect(
      resolveStyleCompletionItems({
        content: scss,
        line: 6,
        character: 18,
        styleDocument,
      }).map((item) => item.label),
    ).toEqual(["tone"]);
  });
});

function styleDocumentMap(documents: readonly ReturnType<typeof parseStyleDocument>[]) {
  const byPath = new Map(documents.map((document) => [document.filePath, document]));
  return (filePath: string) => byPath.get(filePath) ?? null;
}

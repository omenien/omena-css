import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";
import { transform } from "lightningcss";

type OmenaParserDialect = "css" | "scss" | "sass" | "less";
type LegacyParserDialect = Exclude<OmenaParserDialect, "sass">;

interface LegacyParserIndexSummaryV0 {
  readonly language: LegacyParserDialect;
  readonly selectors: {
    readonly names: readonly string[];
  };
  readonly customProperties: {
    readonly declNames: readonly string[];
    readonly refNames: readonly string[];
  };
  readonly sass: {
    readonly variableDeclNames: readonly string[];
    readonly variableRefNames: readonly string[];
    readonly moduleUseSources: readonly string[];
    readonly moduleForwardSources: readonly string[];
  };
}

interface OmenaParserStyleFactsV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-query.omena-parser-style-facts";
  readonly dialect: OmenaParserDialect;
  readonly classSelectorNames: readonly string[];
  readonly idSelectorNames: readonly string[];
  readonly placeholderSelectorNames: readonly string[];
  readonly variableNames: readonly string[];
  readonly customPropertyNames: readonly string[];
  readonly keyframeNames: readonly string[];
  readonly animationReferenceNames: readonly string[];
  readonly cssModuleValueDefinitionNames: readonly string[];
  readonly cssModuleValueReferenceNames: readonly string[];
  readonly cssModuleValueImportSources: readonly string[];
  readonly cssModuleComposesTargetNames: readonly string[];
  readonly cssModuleComposesImportSources: readonly string[];
  readonly icssExportNames: readonly string[];
  readonly icssImportLocalNames: readonly string[];
  readonly icssImportRemoteNames: readonly string[];
  readonly icssImportSources: readonly string[];
  readonly atRuleNames: readonly string[];
  readonly parserErrorCount: number;
}

interface OmenaParserLexResultV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-parser.lex-result";
  readonly dialect: OmenaParserDialect;
  readonly tokens: readonly {
    readonly kind: string;
    readonly text: string;
    readonly start: number;
    readonly end: number;
  }[];
  readonly parserErrorCount: number;
}

interface ParserDifferentialSummary {
  readonly classSelectorNames: readonly string[];
  readonly idSelectorNames: readonly string[];
  readonly customPropertyNames: readonly string[];
  readonly atRuleNames: readonly string[];
}

const LEGACY_SUPPORTED_CORPUS = [
  {
    label: "css-custom-properties",
    filePath: "/f.module.css",
    dialect: "css",
    source: `:root { --brand: red; }\n.card { color: var(--brand); }`,
  },
  {
    label: "scss-nested-bem-and-sass-vars",
    filePath: "/f.module.scss",
    dialect: "scss",
    source: `@use "./tokens";\n@forward "./theme";\n$gap: 1rem;\n.card { &__icon { color: $gap; } }`,
  },
  {
    label: "less-variable-and-selector",
    filePath: "/f.module.less",
    dialect: "less",
    source: `@color: red;\n.card { color: @color; }`,
  },
] as const satisfies readonly {
  readonly label: string;
  readonly filePath: string;
  readonly dialect: LegacyParserDialect;
  readonly source: string;
}[];

const SASS_INDENTED_CORPUS = [
  {
    label: "sass-indented-nested-bem",
    dialect: "sass",
    source: `.card\n  color: red // comment\n  &__icon\n    color: $tone\n$gap: 1rem\n`,
    expected: {
      classSelectorNames: ["card", "card__icon"],
      placeholderSelectorNames: [],
      variableNames: ["$gap", "$tone"],
      customPropertyNames: [],
      atRuleNames: [],
    },
  },
  {
    label: "sass-indented-wrapper-at-rule",
    dialect: "sass",
    source: `@media (min-width: 1px)\n  .card\n    color: red\n`,
    expected: {
      classSelectorNames: ["card"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: ["@media"],
    },
  },
] as const satisfies readonly {
  readonly label: string;
  readonly dialect: "sass";
  readonly source: string;
  readonly expected: {
    readonly classSelectorNames: readonly string[];
    readonly placeholderSelectorNames: readonly string[];
    readonly variableNames: readonly string[];
    readonly customPropertyNames: readonly string[];
    readonly atRuleNames: readonly string[];
  };
}[];

const PARSER_ONLY_CORPUS = [
  {
    label: "css-page-margin-at-rules",
    dialect: "css",
    source: `@page :first { margin: 1cm; @top-left { content: "A"; } @bottom-center { content: counter(page); } }`,
    expected: {
      classSelectorNames: [],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: ["@bottom-center", "@page", "@top-left"],
    },
  },
  {
    label: "css-conditional-level-five-at-rules",
    dialect: "css",
    source: `@when media(width >= 1px) { .a { color: red; } } @else { .b { color: blue; } }`,
    expected: {
      classSelectorNames: ["a", "b"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: ["@else", "@when"],
    },
  },
  {
    label: "css-media-supports-prelude-validation",
    dialect: "css",
    source: `@media not screen and (color), (width >= 1px) { .a { color: red; } } @supports selector(:has(*)) { .b { color: blue; } }`,
    expected: {
      classSelectorNames: ["a", "b"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: ["@media", "@supports"],
    },
  },
  {
    label: "css-modern-declaration-at-rules",
    dialect: "css",
    source: `@counter-style thumbs { system: cyclic; symbols: "yes"; suffix: " "; } @font-palette-values --brand { font-family: Demo; base-palette: 1; } @color-profile --display-p3 { src: url(p3.icc); } @position-try --popover { inset-area: top; }`,
    expected: {
      classSelectorNames: [],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: ["@color-profile", "@counter-style", "@font-palette-values", "@position-try"],
    },
  },
  {
    label: "css-font-feature-values-and-view-transition",
    dialect: "css",
    source: `@font-feature-values Demo { @stylistic { nice: 1; } @styleset { alt: 2; } @character-variant { nice: 3 4; } @swash { fancy: 1; } @ornaments { leaf: 1; } @annotation { circled: 1; } @historical-forms { old: 1; } } @view-transition { navigation: auto; }`,
    expected: {
      classSelectorNames: [],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: [
        "@annotation",
        "@character-variant",
        "@font-feature-values",
        "@historical-forms",
        "@ornaments",
        "@styleset",
        "@stylistic",
        "@swash",
        "@view-transition",
      ],
    },
  },
  {
    label: "css-nesting-and-custom-media-at-rules",
    dialect: "css",
    source: `.card { @nest &__icon { color: red; &--active { color: blue; } } } @custom-media --narrow (width < 40rem);`,
    expected: {
      classSelectorNames: ["card", "card__icon", "card__icon--active"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: ["@custom-media", "@nest"],
    },
  },
  {
    label: "css-modules-icss-import-export",
    dialect: "css",
    source: `:export { primary: #fff; } :import("./tokens.css") { imported: primary; } .btn { composes: imported; color: primary; }`,
    expected: {
      classSelectorNames: ["btn"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: [],
    },
  },
  {
    label: "css-animation-name-facts",
    dialect: "css",
    source: `@keyframes fade { from { opacity: 0; } to { opacity: 1; } } @keyframes "slide" { to { opacity: 1; } } .card { animation-name: fade; animation: "slide" 2s linear both, none 1s, var(--anim) 1s; }`,
    expected: {
      classSelectorNames: ["card"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: ["--anim"],
      keyframeNames: ["fade", "slide"],
      animationReferenceNames: ["fade", "slide"],
      atRuleNames: ["@keyframes", "@keyframes"],
    },
  },
  {
    label: "css-modules-value-facts",
    dialect: "css",
    source: `@value primary: #fff; @value accent: primary; @value secondary as localSecondary from "./tokens.module.css"; .btn { color: accent; }`,
    expected: {
      classSelectorNames: ["btn"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      cssModuleValueDefinitionNames: ["primary", "accent", "localSecondary"],
      cssModuleValueReferenceNames: ["primary", "secondary"],
      cssModuleValueImportSources: ["./tokens.module.css"],
      atRuleNames: ["@value", "@value", "@value"],
    },
  },
  {
    label: "css-modules-composes-facts",
    dialect: "css",
    source: `.btn { composes: base utility from "./base.module.css"; } .global { composes: reset from global; }`,
    expected: {
      classSelectorNames: ["btn", "global"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      cssModuleComposesTargetNames: ["base", "utility", "reset"],
      cssModuleComposesImportSources: ["./base.module.css", "global"],
      atRuleNames: [],
    },
  },
  {
    label: "icss-import-export-facts",
    dialect: "css",
    source: `:export { primary: #fff; secondary: accent; } :import("./tokens.css") { imported: primary; tone: themeTone; }`,
    expected: {
      classSelectorNames: [],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      icssExportNames: ["primary", "secondary"],
      icssImportLocalNames: ["imported", "tone"],
      icssImportRemoteNames: ["primary", "themeTone"],
      icssImportSources: ["./tokens.css"],
      atRuleNames: [],
    },
  },
  {
    label: "css-color-function-micro-grammars",
    dialect: "css",
    source: `.paint { color: color-mix(in srgb, red, blue 30%); background: light-dark(white, black); border-color: contrast-color(red); accent-color: device-cmyk(0 1 1 0); }`,
    expected: {
      classSelectorNames: ["paint"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: [],
    },
  },
  {
    label: "scss-nested-property-blocks",
    dialect: "scss",
    source: `.card { font: { size: 1rem; weight: 700; } }`,
    expected: {
      classSelectorNames: ["card"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: [],
    },
  },
  {
    label: "scss-module-config-preludes",
    dialect: "scss",
    source: `@use "./tokens" as * with ($gap: 1rem, $tone: blue);\n@forward "./theme" as theme-* show $space, token with ($space: 2rem);\n.card { color: $gap; }`,
    expected: {
      classSelectorNames: ["card"],
      placeholderSelectorNames: [],
      variableNames: ["$gap", "$space", "$tone"],
      customPropertyNames: [],
      atRuleNames: ["@forward", "@use"],
    },
  },
  {
    label: "scss-variable-flags",
    dialect: "scss",
    source: `$gap: 1rem !default !global;\n.card { margin: $gap; }`,
    expected: {
      classSelectorNames: ["card"],
      placeholderSelectorNames: [],
      variableNames: ["$gap"],
      customPropertyNames: [],
      atRuleNames: [],
    },
  },
  {
    label: "scss-control-at-rules",
    dialect: "scss",
    source: `@if $enabled { .on { color: green; } } @for $i from 1 through 3 { .n { order: $i; } } @each $k, $v in $map { .e { color: $v; } } @while $enabled { .w { color: red; } }`,
    expected: {
      classSelectorNames: ["e", "n", "on", "w"],
      placeholderSelectorNames: [],
      variableNames: ["$enabled", "$i", "$k", "$map", "$v"],
      customPropertyNames: [],
      atRuleNames: ["@each", "@for", "@if", "@while"],
    },
  },
  {
    label: "scss-include-content-block",
    dialect: "scss",
    source: `.card { @include interactive($tone) using ($state) { &--active { color: red; } } }`,
    expected: {
      classSelectorNames: ["card", "card--active"],
      placeholderSelectorNames: [],
      variableNames: ["$state", "$tone"],
      customPropertyNames: [],
      atRuleNames: ["@include"],
    },
  },
  {
    label: "scss-placeholder-selector-and-extend",
    dialect: "scss",
    source: `%surface { color: red; }\n.card { @extend %surface; }`,
    expected: {
      classSelectorNames: ["card"],
      placeholderSelectorNames: ["surface"],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: ["@extend"],
    },
  },
  {
    label: "less-import-options",
    dialect: "less",
    source: `@import (reference) "theme.less" screen and (min-width: 1px);\n.card { color: red; }`,
    expected: {
      classSelectorNames: ["card"],
      placeholderSelectorNames: [],
      variableNames: [],
      customPropertyNames: [],
      atRuleNames: ["@import"],
    },
  },
] as const satisfies readonly {
  readonly label: string;
  readonly dialect: OmenaParserDialect;
  readonly source: string;
  readonly expected: {
    readonly classSelectorNames: readonly string[];
    readonly placeholderSelectorNames: readonly string[];
    readonly variableNames: readonly string[];
    readonly customPropertyNames: readonly string[];
    readonly keyframeNames?: readonly string[];
    readonly animationReferenceNames?: readonly string[];
    readonly cssModuleValueDefinitionNames?: readonly string[];
    readonly cssModuleValueReferenceNames?: readonly string[];
    readonly cssModuleValueImportSources?: readonly string[];
    readonly cssModuleComposesTargetNames?: readonly string[];
    readonly cssModuleComposesImportSources?: readonly string[];
    readonly icssExportNames?: readonly string[];
    readonly icssImportLocalNames?: readonly string[];
    readonly icssImportRemoteNames?: readonly string[];
    readonly icssImportSources?: readonly string[];
    readonly atRuleNames: readonly string[];
  };
}[];

const TOKEN_TEXT_CORPUS = [
  {
    label: "css-selectors-l4-token-text",
    dialect: "css",
    source: `.card:has(> .icon, + [data-active]):nth-child(2n + 1 of .item):lang(en-US, "ko"):dir(rtl) { color: var(--brand); }`,
    expectedTokenTexts: [
      ".",
      "card",
      "has",
      ">",
      "icon",
      "nth-child",
      "of",
      "item",
      "lang",
      "en-US",
      '"ko"',
      "dir",
      "rtl",
      "--brand",
    ],
  },
  {
    label: "css-syntax-input-preprocessing-token-text",
    dialect: "css",
    source: `\u{feff}.a\0b { background: url(foo\0bar); }`,
    expectedTokenTexts: ["a\u{fffd}b", "url(foo\u{fffd}bar)"],
  },
  {
    label: "scss-dialect-token-text",
    dialect: "scss",
    source: `$gap: 1rem;\n.card-#{$variant} { color: $gap; }`,
    expectedTokenTexts: ["$gap", "1rem", "#{", "$variant", "$gap"],
  },
  {
    label: "less-dialect-token-text",
    dialect: "less",
    source: `@gap: 1rem;\n.card-@{variant} { width: @gap; }`,
    expectedTokenTexts: ["@gap", "1rem", "@{", "variant", "@gap"],
  },
] as const satisfies readonly {
  readonly label: string;
  readonly dialect: OmenaParserDialect;
  readonly source: string;
  readonly expectedTokenTexts: readonly string[];
}[];

const LIGHTNINGCSS_CSS_CORPUS = [
  {
    label: "lightningcss-custom-property-and-calc",
    source: `:root { --brand: red; } .card { color: var(--brand); width: calc(1px + 2px); }`,
  },
  {
    label: "lightningcss-selectors-level-four",
    source: `.card:has(> .icon, + [data-active]):nth-child(2n + 1 of .item) { color: red; }\n.panel:dir(rtl):lang(en-US) { color: blue; }`,
  },
  {
    label: "lightningcss-conditional-at-rules",
    source: `@layer theme; @media (width >= 1px) { .mediaCard { color: red; } } @supports (display: grid) { .gridCard { display: grid; } }`,
  },
  {
    label: "lightningcss-layer-prelude-forms",
    source: `@layer reset, app.ui; @layer components { .card { color: red; } } @layer { .anon { color: blue; } }`,
  },
  {
    label: "lightningcss-values-level-four-functions",
    source: `.paint { color: color-mix(in srgb, red, blue); background: linear-gradient(red, blue); transform: translateX(1rem) rotate(10deg); }`,
  },
  {
    label: "lightningcss-id-selectors-container-and-keyframes",
    source: `#app.theme > .card:has(> .icon, + [data-active]) { --brand: red; color: var(--brand); }\n@container card (width > 20rem) { #inside.panel { color: red; } }\n@keyframes fade { from { opacity: 0; } to { opacity: 1; } }`,
  },
  {
    label: "lightningcss-namespace-selectors-and-keyframes-list",
    source: `@namespace svg url("http://www.w3.org/2000/svg"); svg|a.icon { color: red; } @keyframes fade { from { opacity: 0; } 50%, 75% { opacity: .5; } to { opacity: 1; } } .box { animation: fade 1s; }`,
  },
] as const satisfies readonly {
  readonly label: string;
  readonly source: string;
}[];

async function runLegacyIndex(
  filePath: string,
  source: string,
): Promise<LegacyParserIndexSummaryV0> {
  return runJson<LegacyParserIndexSummaryV0>(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-style-parser",
      "--bin",
      "engine-style-parser-css-modules-intermediate",
      "--",
      filePath,
    ],
    source,
  );
}

async function runOmenaParserStyleFacts(
  dialect: OmenaParserDialect,
  source: string,
): Promise<OmenaParserStyleFactsV0> {
  return runJson<OmenaParserStyleFactsV0>(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--",
      "omena-parser-style-facts",
    ],
    JSON.stringify({ styleSource: source, dialect }),
  );
}

async function runOmenaParserLex(
  dialect: OmenaParserDialect,
  source: string,
): Promise<OmenaParserLexResultV0> {
  return runJson<OmenaParserLexResultV0>(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--",
      "omena-parser-lex",
    ],
    JSON.stringify({ styleSource: source, dialect }),
  );
}

function runJson<T>(command: string, args: readonly string[], stdin: string): Promise<T> {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: process.cwd(),
      stdio: ["pipe", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += String(chunk);
    });
    child.stderr.on("data", (chunk) => {
      stderr += String(chunk);
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) {
        reject(new Error(`${command} ${args.join(" ")} exited with ${code}\n${stderr}`));
        return;
      }
      resolve(JSON.parse(stdout) as T);
    });
    child.stdin.end(stdin);
  });
}

function normalizeVariableName(name: string): string {
  return name.replace(/^[$@]/, "");
}

function sortedUnique(values: readonly string[]): string[] {
  return [...new Set(values)].toSorted();
}

function normalizeCssSyntaxInputText(text: string): string {
  return text.replaceAll("\0", "\u{fffd}");
}

function sourceByteSlice(source: string, start: number, end: number): string {
  return Buffer.from(source, "utf8").subarray(start, end).toString("utf8");
}

function summarizeLightningCss(source: string): ParserDifferentialSummary {
  const classSelectorNames: string[] = [];
  const idSelectorNames: string[] = [];
  const customPropertyNames: string[] = [];
  const atRuleNames: string[] = [];

  transform({
    filename: "fixture.module.css",
    code: Buffer.from(source),
    visitor: {
      Selector(selector: unknown) {
        classSelectorNames.push(...topLevelSelectorClassNames(selector));
        idSelectorNames.push(...topLevelSelectorIdNames(selector));
      },
      Declaration(declaration: unknown) {
        const customPropertyName = lightningCustomPropertyName(declaration);
        if (customPropertyName !== undefined) {
          customPropertyNames.push(customPropertyName);
        }
      },
      Rule(rule: unknown) {
        const atRuleName = lightningAtRuleName(rule);
        if (atRuleName !== undefined) {
          atRuleNames.push(atRuleName);
        }
      },
    },
  });

  return {
    classSelectorNames,
    idSelectorNames,
    customPropertyNames,
    atRuleNames,
  };
}

function topLevelSelectorClassNames(selector: unknown): string[] {
  return topLevelSelectorNames(selector, "class");
}

function topLevelSelectorIdNames(selector: unknown): string[] {
  return topLevelSelectorNames(selector, "id");
}

function topLevelSelectorNames(selector: unknown, selectorType: "class" | "id"): string[] {
  if (!Array.isArray(selector)) {
    return [];
  }
  return selector.flatMap((component) => {
    if (!recordHasString(component, "type") || component.type !== selectorType) {
      return [];
    }
    return recordHasString(component, "name") ? [component.name] : [];
  });
}

function lightningCustomPropertyName(declaration: unknown): string | undefined {
  if (!recordHasString(declaration, "property") || declaration.property !== "custom") {
    return undefined;
  }
  const value = declaration.value;
  return recordHasString(value, "name") ? value.name : undefined;
}

function lightningAtRuleName(rule: unknown): string | undefined {
  if (!recordHasString(rule, "type")) {
    return undefined;
  }
  switch (rule.type) {
    case "media":
      return "@media";
    case "supports":
      return "@supports";
    case "container":
      return "@container";
    case "layer-statement":
    case "layer-block":
      return "@layer";
    case "keyframes":
      return "@keyframes";
    case "namespace":
      return "@namespace";
    default:
      return undefined;
  }
}

function recordHasString(value: unknown, key: string): value is Record<string, string> {
  return (
    typeof value === "object" &&
    value !== null &&
    key in value &&
    typeof (value as Record<string, unknown>)[key] === "string"
  );
}

function assertCommonFacts(actual: OmenaParserStyleFactsV0, label: string): void {
  assert.equal(actual.schemaVersion, "0");
  assert.equal(actual.product, "omena-query.omena-parser-style-facts");
  assert.equal(actual.parserErrorCount, 0, `${label} should parse without errors`);
}

void (async () => {
  for (const entry of LEGACY_SUPPORTED_CORPUS) {
    process.stdout.write(`== omena-parser-differential-corpus:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const legacy = await runLegacyIndex(entry.filePath, entry.source);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runOmenaParserStyleFacts(entry.dialect, entry.source);

    assertCommonFacts(actual, entry.label);
    assert.equal(actual.dialect, entry.dialect);
    assert.equal(legacy.language, entry.dialect);
    assert.deepEqual(
      sortedUnique(actual.classSelectorNames),
      sortedUnique(legacy.selectors.names),
      `${entry.label} class selector differential drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.customPropertyNames),
      sortedUnique([...legacy.customProperties.declNames, ...legacy.customProperties.refNames]),
      `${entry.label} custom property differential drift`,
    );
    if (entry.dialect === "scss") {
      assert.deepEqual(
        sortedUnique(actual.variableNames.map(normalizeVariableName)),
        sortedUnique([...legacy.sass.variableDeclNames, ...legacy.sass.variableRefNames]),
        `${entry.label} variable differential drift`,
      );
      assert.deepEqual(
        sortedUnique(actual.atRuleNames.filter((name) => name === "@use" || name === "@forward")),
        sortedUnique([
          ...legacy.sass.moduleUseSources.map(() => "@use"),
          ...legacy.sass.moduleForwardSources.map(() => "@forward"),
        ]),
        `${entry.label} Sass module at-rule differential drift`,
      );
    }

    process.stdout.write(
      `validated legacy differential: selectors=${actual.classSelectorNames.length} variables=${actual.variableNames.length}\n\n`,
    );
  }

  for (const entry of SASS_INDENTED_CORPUS) {
    process.stdout.write(`== omena-parser-differential-corpus:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runOmenaParserStyleFacts(entry.dialect, entry.source);

    assertCommonFacts(actual, entry.label);
    assert.equal(actual.dialect, entry.dialect);
    assert.deepEqual(
      sortedUnique(actual.classSelectorNames),
      sortedUnique(entry.expected.classSelectorNames),
      `${entry.label} Sass-indented selector drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.placeholderSelectorNames),
      sortedUnique(entry.expected.placeholderSelectorNames),
      `${entry.label} Sass-indented placeholder selector drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.variableNames),
      sortedUnique(entry.expected.variableNames),
      `${entry.label} Sass-indented variable drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.customPropertyNames),
      sortedUnique(entry.expected.customPropertyNames),
      `${entry.label} Sass-indented custom property drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.atRuleNames),
      sortedUnique(entry.expected.atRuleNames),
      `${entry.label} Sass-indented at-rule drift`,
    );

    process.stdout.write(
      `validated Sass-indented corpus: selectors=${actual.classSelectorNames.length} variables=${actual.variableNames.length}\n\n`,
    );
  }

  for (const entry of PARSER_ONLY_CORPUS) {
    process.stdout.write(`== omena-parser-differential-corpus:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runOmenaParserStyleFacts(entry.dialect, entry.source);

    assertCommonFacts(actual, entry.label);
    assert.equal(actual.dialect, entry.dialect);
    assert.deepEqual(
      sortedUnique(actual.classSelectorNames),
      sortedUnique(entry.expected.classSelectorNames),
      `${entry.label} parser-only class selector drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.placeholderSelectorNames),
      sortedUnique(entry.expected.placeholderSelectorNames),
      `${entry.label} parser-only placeholder selector drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.variableNames),
      sortedUnique(entry.expected.variableNames),
      `${entry.label} parser-only variable drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.customPropertyNames),
      sortedUnique(entry.expected.customPropertyNames),
      `${entry.label} parser-only custom property drift`,
    );
    if (entry.expected.keyframeNames) {
      assert.deepEqual(
        sortedUnique(actual.keyframeNames),
        sortedUnique(entry.expected.keyframeNames),
        `${entry.label} parser-only keyframe drift`,
      );
    }
    if (entry.expected.animationReferenceNames) {
      assert.deepEqual(
        sortedUnique(actual.animationReferenceNames),
        sortedUnique(entry.expected.animationReferenceNames),
        `${entry.label} parser-only animation reference drift`,
      );
    }
    if (entry.expected.cssModuleValueDefinitionNames) {
      assert.deepEqual(
        sortedUnique(actual.cssModuleValueDefinitionNames),
        sortedUnique(entry.expected.cssModuleValueDefinitionNames),
        `${entry.label} parser-only CSS Modules @value definition drift`,
      );
    }
    if (entry.expected.cssModuleValueReferenceNames) {
      assert.deepEqual(
        sortedUnique(actual.cssModuleValueReferenceNames),
        sortedUnique(entry.expected.cssModuleValueReferenceNames),
        `${entry.label} parser-only CSS Modules @value reference drift`,
      );
    }
    if (entry.expected.cssModuleValueImportSources) {
      assert.deepEqual(
        sortedUnique(actual.cssModuleValueImportSources),
        sortedUnique(entry.expected.cssModuleValueImportSources),
        `${entry.label} parser-only CSS Modules @value import source drift`,
      );
    }
    if (entry.expected.cssModuleComposesTargetNames) {
      assert.deepEqual(
        sortedUnique(actual.cssModuleComposesTargetNames),
        sortedUnique(entry.expected.cssModuleComposesTargetNames),
        `${entry.label} parser-only CSS Modules composes target drift`,
      );
    }
    if (entry.expected.cssModuleComposesImportSources) {
      assert.deepEqual(
        sortedUnique(actual.cssModuleComposesImportSources),
        sortedUnique(entry.expected.cssModuleComposesImportSources),
        `${entry.label} parser-only CSS Modules composes import source drift`,
      );
    }
    if (entry.expected.icssExportNames) {
      assert.deepEqual(
        sortedUnique(actual.icssExportNames),
        sortedUnique(entry.expected.icssExportNames),
        `${entry.label} parser-only ICSS export drift`,
      );
    }
    if (entry.expected.icssImportLocalNames) {
      assert.deepEqual(
        sortedUnique(actual.icssImportLocalNames),
        sortedUnique(entry.expected.icssImportLocalNames),
        `${entry.label} parser-only ICSS import local drift`,
      );
    }
    if (entry.expected.icssImportRemoteNames) {
      assert.deepEqual(
        sortedUnique(actual.icssImportRemoteNames),
        sortedUnique(entry.expected.icssImportRemoteNames),
        `${entry.label} parser-only ICSS import remote drift`,
      );
    }
    if (entry.expected.icssImportSources) {
      assert.deepEqual(
        sortedUnique(actual.icssImportSources),
        sortedUnique(entry.expected.icssImportSources),
        `${entry.label} parser-only ICSS import source drift`,
      );
    }
    assert.deepEqual(
      sortedUnique(actual.atRuleNames),
      sortedUnique(entry.expected.atRuleNames),
      `${entry.label} parser-only at-rule drift`,
    );

    process.stdout.write(
      `validated parser-only corpus: selectors=${actual.classSelectorNames.length} placeholders=${actual.placeholderSelectorNames.length}\n\n`,
    );
  }

  for (const entry of TOKEN_TEXT_CORPUS) {
    process.stdout.write(`== omena-parser-differential-corpus:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runOmenaParserLex(entry.dialect, entry.source);
    const tokenTexts = new Set(actual.tokens.map((token) => token.text));

    assert.equal(actual.schemaVersion, "0");
    assert.equal(actual.product, "omena-parser.lex-result");
    assert.equal(actual.dialect, entry.dialect);
    assert.equal(actual.parserErrorCount, 0, `${entry.label} should lex without errors`);
    for (const token of actual.tokens) {
      assert.equal(
        normalizeCssSyntaxInputText(sourceByteSlice(entry.source, token.start, token.end)),
        token.text,
        `${entry.label} token text must match source range for ${token.kind}`,
      );
      assert.ok(!token.text.includes("\0"), `${entry.label} token text must replace NULL`);
      assert.ok(!token.text.includes("\u{feff}"), `${entry.label} token text must skip BOM`);
    }
    for (const expectedText of entry.expectedTokenTexts) {
      assert.ok(
        tokenTexts.has(expectedText),
        `${entry.label} should expose token text ${expectedText}`,
      );
    }

    process.stdout.write(
      `validated token text corpus: tokens=${actual.tokens.length} required=${entry.expectedTokenTexts.length}\n\n`,
    );
  }

  for (const entry of LIGHTNINGCSS_CSS_CORPUS) {
    process.stdout.write(`== omena-parser-differential-corpus:${entry.label} ==\n`);

    const lightning = summarizeLightningCss(entry.source);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runOmenaParserStyleFacts("css", entry.source);

    assertCommonFacts(actual, entry.label);
    assert.deepEqual(
      sortedUnique(actual.classSelectorNames),
      sortedUnique(lightning.classSelectorNames),
      `${entry.label} lightningcss class selector differential drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.idSelectorNames),
      sortedUnique(lightning.idSelectorNames),
      `${entry.label} lightningcss id selector differential drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.customPropertyNames),
      sortedUnique(lightning.customPropertyNames),
      `${entry.label} lightningcss custom property differential drift`,
    );
    assert.deepEqual(
      sortedUnique(actual.atRuleNames),
      sortedUnique(lightning.atRuleNames),
      `${entry.label} lightningcss at-rule differential drift`,
    );

    process.stdout.write(
      `validated lightningcss differential: classes=${actual.classSelectorNames.length} ids=${actual.idSelectorNames.length} atRules=${actual.atRuleNames.length}\n\n`,
    );
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

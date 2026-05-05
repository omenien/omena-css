import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";

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
    source: `@use "./tokens" with ($gap: 1rem, $tone: blue);\n@forward "./theme" with ($space: 2rem);\n.card { color: $gap; }`,
    expected: {
      classSelectorNames: ["card"],
      placeholderSelectorNames: [],
      variableNames: ["$gap", "$space", "$tone"],
      customPropertyNames: [],
      atRuleNames: ["@forward", "@use"],
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
] as const satisfies readonly {
  readonly label: string;
  readonly dialect: OmenaParserDialect;
  readonly source: string;
  readonly expected: {
    readonly classSelectorNames: readonly string[];
    readonly placeholderSelectorNames: readonly string[];
    readonly variableNames: readonly string[];
    readonly customPropertyNames: readonly string[];
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
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

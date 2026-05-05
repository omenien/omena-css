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
  readonly dialect: "scss";
  readonly source: string;
  readonly expected: {
    readonly classSelectorNames: readonly string[];
    readonly placeholderSelectorNames: readonly string[];
    readonly variableNames: readonly string[];
    readonly customPropertyNames: readonly string[];
    readonly atRuleNames: readonly string[];
  };
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
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

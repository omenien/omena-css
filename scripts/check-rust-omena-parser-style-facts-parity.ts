import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";

interface LegacyParserIndexSummaryV0 {
  readonly language: "css" | "scss" | "less";
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
    readonly moduleImportSources: readonly string[];
  };
}

interface OmenaParserStyleFactsV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-query.omena-parser-style-facts";
  readonly dialect: "css" | "scss" | "sass" | "less";
  readonly classSelectorNames: readonly string[];
  readonly idSelectorNames: readonly string[];
  readonly variableNames: readonly string[];
  readonly customPropertyNames: readonly string[];
  readonly keyframeNames: readonly string[];
  readonly animationReferenceNames: readonly string[];
  readonly atRuleNames: readonly string[];
  readonly parserErrorCount: number;
}

const CORPUS = [
  {
    label: "css-custom-property-facts",
    filePath: "/f.module.css",
    dialect: "css",
    source: `:root { --color: red; }\n.btn { color: var(--color); }`,
  },
  {
    label: "scss-nested-bem-facts",
    filePath: "/f.module.scss",
    dialect: "scss",
    source: `.card { &__icon { &--small { color: red; } } --space: 1rem; color: var(--space); }`,
  },
  {
    label: "scss-grouped-nested-bem-facts",
    filePath: "/f.module.scss",
    dialect: "scss",
    source: `.a, .b { &__icon { color: red; } }`,
  },
  {
    label: "scss-combinator-all-defining-facts",
    filePath: "/f.module.scss",
    dialect: "scss",
    source: `.a > .b { color: red; }`,
    expectedClassSelectorNames: ["a", "b"],
  },
  {
    label: "scss-pseudo-function-facts",
    filePath: "/f.module.scss",
    dialect: "scss",
    source: `.btn:is(.active, .primary) { color: red; }\n:local(.localName) { color: blue; }`,
  },
  {
    label: "css-modules-global-scope-facts",
    filePath: "/f.module.css",
    dialect: "css",
    source: `:global { .reset { color: red; } } :global(.standalone) { color: red; } .card :global(.child) { color: red; } :local(.button) { color: blue; }`,
    expectedClassSelectorNames: ["button", "card"],
  },
  {
    label: "css-modules-local-id-scope-facts",
    filePath: "/f.module.css",
    dialect: "css",
    source: `:local(#panel) { color: red; } :global(#reset) { color: red; } .card :global(#child) { color: blue; }`,
    expectedClassSelectorNames: ["card"],
    expectedIdSelectorNames: ["panel"],
  },
  {
    label: "css-animation-name-facts",
    filePath: "/f.module.css",
    dialect: "css",
    source: `@keyframes fade { from { opacity: 0; } to { opacity: 1; } } @keyframes "slide" { to { opacity: 1; } } .card { animation-name: fade, "slide", none; }`,
    expectedClassSelectorNames: ["card"],
    expectedKeyframeNames: ["fade", "slide"],
    expectedAnimationReferenceNames: ["fade", "slide"],
  },
  {
    label: "scss-sass-symbol-facts",
    filePath: "/f.module.scss",
    dialect: "scss",
    source: `@use "./tokens";\n@forward "./theme";\n$gap: 1rem;\n.btn { margin: $gap; }`,
  },
  {
    label: "less-selector-facts",
    filePath: "/f.module.less",
    dialect: "less",
    source: `@color: red;\n.btn { color: @color; }`,
  },
] as const;

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
  dialect: string,
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

function normalizeScssVariable(name: string): string {
  return name.replace(/^[$@]/, "");
}

function sortedUnique(values: readonly string[]): string[] {
  return [...new Set(values)].toSorted();
}

void (async () => {
  for (const entry of CORPUS) {
    process.stdout.write(`== omena-parser-style-facts-parity:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const legacy = await runLegacyIndex(entry.filePath, entry.source);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runOmenaParserStyleFacts(entry.dialect, entry.source);

    assert.equal(actual.schemaVersion, "0");
    assert.equal(actual.product, "omena-query.omena-parser-style-facts");
    assert.equal(actual.dialect, entry.dialect);
    assert.equal(actual.parserErrorCount, 0, `${entry.label} should parse without errors`);

    assert.deepEqual(
      sortedUnique(actual.classSelectorNames),
      sortedUnique(entry.expectedClassSelectorNames ?? legacy.selectors.names),
      `${entry.label} class selector parity drift`,
    );
    if ("expectedIdSelectorNames" in entry) {
      assert.deepEqual(
        sortedUnique(actual.idSelectorNames),
        sortedUnique(entry.expectedIdSelectorNames),
        `${entry.label} id selector parity drift`,
      );
    }
    if ("expectedKeyframeNames" in entry) {
      assert.deepEqual(
        sortedUnique(actual.keyframeNames),
        sortedUnique(entry.expectedKeyframeNames),
        `${entry.label} keyframe name drift`,
      );
    }
    if ("expectedAnimationReferenceNames" in entry) {
      assert.deepEqual(
        sortedUnique(actual.animationReferenceNames),
        sortedUnique(entry.expectedAnimationReferenceNames),
        `${entry.label} animation reference drift`,
      );
    }

    const legacyCustomProperties = sortedUnique([
      ...legacy.customProperties.declNames,
      ...legacy.customProperties.refNames,
    ]);
    assert.deepEqual(
      sortedUnique(actual.customPropertyNames),
      legacyCustomProperties,
      `${entry.label} custom property parity drift`,
    );

    if (entry.dialect === "scss") {
      assert.deepEqual(
        sortedUnique(actual.variableNames.map(normalizeScssVariable)),
        sortedUnique([...legacy.sass.variableDeclNames, ...legacy.sass.variableRefNames]),
        `${entry.label} Sass variable parity drift`,
      );
      assert.deepEqual(
        sortedUnique(actual.atRuleNames.filter((name) => name === "@use" || name === "@forward")),
        sortedUnique([
          ...legacy.sass.moduleUseSources.map(() => "@use"),
          ...legacy.sass.moduleForwardSources.map(() => "@forward"),
        ]),
        `${entry.label} Sass module at-rule parity drift`,
      );
    }

    process.stdout.write(
      `validated omena-parser style facts parity: selectors=${actual.classSelectorNames.length} customProperties=${actual.customPropertyNames.length} variables=${actual.variableNames.length}\n\n`,
    );
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

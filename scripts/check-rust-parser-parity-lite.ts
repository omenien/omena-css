import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";

import type { AtRule, ChildNode, Comment, Root, Rule } from "postcss";
import { parse as postcssParse } from "postcss";
import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";
import {
  findLangForPath,
  getRuntimeSyntax,
} from "../server/engine-core-ts/src/core/scss/lang-registry";

interface ParserParityLiteSummaryV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly selectorNames: readonly string[];
  readonly keyframesNames: readonly string[];
  readonly valueDeclNames: readonly string[];
  readonly diagnosticCount: number;
  readonly ruleCount: number;
  readonly declarationCount: number;
  readonly groupedSelectorCount: number;
  readonly maxNestingDepth: number;
  readonly atRuleKindCounts: {
    readonly media: number;
    readonly supports: number;
    readonly layer: number;
    readonly keyframes: number;
    readonly value: number;
    readonly atRoot: number;
    readonly generic: number;
  };
  readonly declarationKindCounts: {
    readonly composes: number;
    readonly animation: number;
    readonly animationName: number;
    readonly generic: number;
  };
}

const CORPUS = [
  {
    label: "scss-leading-comment",
    filePath: "/f.module.scss",
    source: `// leading comment\n.btn { color: red; }`,
  },
  {
    label: "scss-media-wrapper",
    filePath: "/f.module.scss",
    source: `@media (min-width: 600px) {\n  .btn { font-size: 16px; }\n}`,
  },
  {
    label: "scss-value-and-keyframes",
    filePath: "/f.module.scss",
    source: `@value brand: red;\n@keyframes fade { from { opacity: 0; } }\n.btn { color: brand; }`,
  },
  {
    label: "scss-bem-nested",
    filePath: "/f.module.scss",
    source: `.card { &__icon { &--small { color: red; } } }`,
  },
  {
    label: "scss-grouped-bem-nested",
    filePath: "/f.module.scss",
    source: `.a, .b { &__icon { &--small { color: red; } } }`,
  },
  {
    label: "scss-grouped-active",
    filePath: "/f.module.scss",
    source: `.a, .b { &.active { color: red; } }`,
  },
  {
    label: "scss-pseudo",
    filePath: "/f.module.scss",
    source: `.btn:hover { color: red; }`,
  },
  {
    label: "scss-compound-class",
    filePath: "/f.module.scss",
    source: `.btn.active { color: red; }`,
  },
  {
    label: "scss-combinator-rightmost",
    filePath: "/f.module.scss",
    source: `.a > .b { color: red; }`,
  },
  {
    label: "scss-layer-wrapper",
    filePath: "/f.module.scss",
    source: `@layer ui { .btn:hover { color: red; } }`,
  },
  {
    label: "scss-descendant-rightmost",
    filePath: "/f.module.scss",
    source: `.a .b { color: red; }`,
  },
  {
    label: "scss-pseudo-function-is",
    filePath: "/f.module.scss",
    source: `.btn:is(.active, .primary) { color: red; }`,
  },
  {
    label: "scss-global-function",
    filePath: "/f.module.scss",
    source: `:global(.foo) { color: red; }`,
  },
  {
    label: "scss-local-function",
    filePath: "/f.module.scss",
    source: `:local(.foo) { color: red; }`,
  },
  {
    label: "scss-pseudo-function-not",
    filePath: "/f.module.scss",
    source: `.btn:not(.disabled) { color: red; }`,
  },
  {
    label: "scss-wrapper-mixed",
    filePath: "/f.module.scss",
    source: `@media (min-width: 1px) { @value brand: red; @keyframes fade { from { opacity: 0; } } .btn:hover { color: brand; } }`,
  },
  {
    label: "scss-supports-layer",
    filePath: "/f.module.scss",
    source: `@supports (display: grid) { @layer ui { .grid > .item { display: grid; } } }`,
  },
  {
    label: "scss-composes-decl",
    filePath: "/f.module.scss",
    source: `.btn { composes: base from "./base.module.scss"; }`,
  },
  {
    label: "scss-animation-decls",
    filePath: "/f.module.scss",
    source: `@keyframes fade { from { opacity: 0; } }\n.btn { animation: fade 1s linear; animation-name: fade; }`,
  },
  {
    label: "less-variable",
    filePath: "/f.module.less",
    source: `@color: red;\n.btn { color: @color; }`,
  },
  {
    label: "css-basic",
    filePath: "/f.module.css",
    source: `.btn { color: red; }\n.link { color: blue; }`,
  },
] as const;

function deriveTsSummary(filePath: string, source: string): ParserParityLiteSummaryV0 {
  const document = parseStyleDocument(source, filePath);
  const structural = deriveTsStructuralSummary(filePath, source);
  return {
    schemaVersion: "0",
    language: filePath.endsWith(".module.less")
      ? "less"
      : filePath.endsWith(".module.scss")
        ? "scss"
        : "css",
    selectorNames: [...document.selectors].map((selector) => selector.name).toSorted(),
    keyframesNames: [...document.keyframes].map((entry) => entry.name).toSorted(),
    valueDeclNames: [...document.valueDecls].map((entry) => entry.name).toSorted(),
    diagnosticCount: 0,
    ruleCount: structural.ruleCount,
    declarationCount: structural.declarationCount,
    groupedSelectorCount: structural.groupedSelectorCount,
    maxNestingDepth: structural.maxNestingDepth,
    atRuleKindCounts: structural.atRuleKindCounts,
    declarationKindCounts: structural.declarationKindCounts,
  };
}

function deriveTsStructuralSummary(filePath: string, source: string) {
  const lang = findLangForPath(filePath);
  const syntax = lang ? getRuntimeSyntax(lang) : null;
  const parse = typeof syntax?.parse === "function" ? syntax.parse.bind(syntax) : postcssParse;
  const root = parse(source, { from: filePath }) as Root;
  const summary = {
    ruleCount: 0,
    declarationCount: 0,
    groupedSelectorCount: 0,
    maxNestingDepth: 0,
    atRuleKindCounts: {
      media: 0,
      supports: 0,
      layer: 0,
      keyframes: 0,
      value: 0,
      atRoot: 0,
      generic: 0,
    },
    declarationKindCounts: {
      composes: 0,
      animation: 0,
      animationName: 0,
      generic: 0,
    },
  };

  walkStructuralNodes(root.nodes ?? [], summary, 0);
  return summary;
}

function walkStructuralNodes(
  nodes: readonly ChildNode[],
  summary: ReturnType<typeof deriveTsStructuralSummary>,
  depth: number,
): void {
  for (const node of nodes) {
    if (node.type === "rule") {
      summary.ruleCount += 1;
      const rule = node as Rule;
      const nextDepth = depth + 1;
      summary.maxNestingDepth = Math.max(summary.maxNestingDepth, nextDepth);
      const selectorGroups = countTsSelectorGroups(rule.selector);
      if (selectorGroups > 1) {
        summary.groupedSelectorCount += selectorGroups;
      }
      walkStructuralNodes(rule.nodes ?? [], summary, nextDepth);
      continue;
    }
    if (node.type === "atrule") {
      const atRule = node as AtRule;
      const nextDepth = depth + 1;
      summary.maxNestingDepth = Math.max(summary.maxNestingDepth, nextDepth);
      incrementTsAtRuleKindCount(summary.atRuleKindCounts, classifyTsAtRuleKind(atRule));
      walkStructuralNodes(atRule.nodes ?? [], summary, nextDepth);
      continue;
    }
    if (node.type === "decl") {
      incrementTsDeclarationKindCount(
        summary.declarationKindCounts,
        classifyTsDeclarationKind(node.prop),
      );
      summary.declarationCount += 1;
      continue;
    }
    if (node.type === "comment") {
      void (node as Comment);
    }
  }
}

function classifyTsAtRuleKind(node: AtRule): keyof ParserParityLiteSummaryV0["atRuleKindCounts"] {
  switch (node.name) {
    case "media":
      return "media";
    case "supports":
      return "supports";
    case "layer":
      return "layer";
    case "keyframes":
    case "-webkit-keyframes":
      return "keyframes";
    case "value":
      return "value";
    case "at-root":
      return "atRoot";
    default:
      return "generic";
  }
}

function incrementTsAtRuleKindCount(
  counts: ParserParityLiteSummaryV0["atRuleKindCounts"],
  kind: keyof ParserParityLiteSummaryV0["atRuleKindCounts"],
) {
  counts[kind] += 1;
}

function classifyTsDeclarationKind(
  property: string,
): keyof ParserParityLiteSummaryV0["declarationKindCounts"] {
  switch (property.trim().toLowerCase()) {
    case "composes":
      return "composes";
    case "animation":
      return "animation";
    case "animation-name":
      return "animationName";
    default:
      return "generic";
  }
}

function incrementTsDeclarationKindCount(
  counts: ParserParityLiteSummaryV0["declarationKindCounts"],
  kind: keyof ParserParityLiteSummaryV0["declarationKindCounts"],
) {
  counts[kind] += 1;
}

function countTsSelectorGroups(selector: string): number {
  let depthParen = 0;
  let depthBracket = 0;
  let start = 0;
  let count = 0;

  for (let index = 0; index < selector.length; index += 1) {
    const ch = selector[index];
    if (ch === "(") {
      depthParen += 1;
      continue;
    }
    if (ch === ")") {
      depthParen = Math.max(depthParen - 1, 0);
      continue;
    }
    if (ch === "[") {
      depthBracket += 1;
      continue;
    }
    if (ch === "]") {
      depthBracket = Math.max(depthBracket - 1, 0);
      continue;
    }
    if (ch === "," && depthParen === 0 && depthBracket === 0) {
      if (selector.slice(start, index).trim().length > 0) {
        count += 1;
      }
      start = index + 1;
    }
  }

  if (selector.slice(start).trim().length > 0) {
    count += 1;
  }
  return count;
}

async function runRustSummary(
  filePath: string,
  source: string,
): Promise<ParserParityLiteSummaryV0> {
  return new Promise((resolve, reject) => {
    const child = spawn(
      "cargo",
      [
        "run",
        "--quiet",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "omena-parser",
        "--bin",
        "omena-parser-summary",
        "--",
        filePath,
      ],
      {
        cwd: process.cwd(),
        stdio: ["pipe", "pipe", "pipe"],
      },
    );

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
        reject(new Error(`omena-parser-summary exited with ${code}\n${stderr}`));
        return;
      }
      resolve(JSON.parse(stdout) as ParserParityLiteSummaryV0);
    });

    child.stdin.end(source);
  });
}

void (async () => {
  for (const entry of CORPUS) {
    process.stdout.write(`== rust-parser-parity-lite:${entry.label} ==\n`);
    const expected = deriveTsSummary(entry.filePath, entry.source);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runRustSummary(entry.filePath, entry.source);

    assert.deepEqual(
      actual,
      expected,
      [
        `parser parity-lite mismatch for ${entry.label}`,
        `expected: ${JSON.stringify(expected, null, 2)}`,
        `actual: ${JSON.stringify(actual, null, 2)}`,
      ].join("\n"),
    );

    process.stdout.write(
      `matched parser summary: selectors=${actual.selectorNames.length} keyframes=${actual.keyframesNames.length} values=${actual.valueDeclNames.length}\n\n`,
    );
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

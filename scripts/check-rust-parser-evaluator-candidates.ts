import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";

import type { Range } from "@omena/shared";
import { parse as postcssParse, type AtRule, type ChildNode, type Root, type Rule } from "postcss";
import safeParser from "postcss-safe-parser";

import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";

interface ParserEvaluatorCandidateV0 {
  readonly kind: "selector-index-facts";
  readonly selectorName: string;
  readonly nestedSafetyKind: "flat" | "bemSuffixSafe" | "nestedUnsafe";
  readonly bemSuffixParentName?: string;
  readonly underMedia: boolean;
  readonly underSupports: boolean;
  readonly underLayer: boolean;
  readonly hasValueRefs: boolean;
  readonly hasLocalValueRefs: boolean;
  readonly hasImportedValueRefs: boolean;
  readonly hasCustomPropertyRefs: boolean;
  readonly hasAnimationRef: boolean;
  readonly hasAnimationNameRef: boolean;
  readonly hasComposes: boolean;
  readonly hasLocalComposes: boolean;
  readonly hasImportedComposes: boolean;
  readonly hasGlobalComposes: boolean;
}

interface ParserEvaluatorCandidatesV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly results: readonly ParserEvaluatorCandidateV0[];
}

const CORPUS = [
  {
    label: "css-basic-parser-evaluator-candidates",
    filePath: "/f.module.css",
    source: `.btn { color: red; }`,
  },
  {
    label: "scss-media-keyframes-parser-evaluator-candidates",
    filePath: "/f.module.scss",
    source: `@media (min-width: 1px) { @keyframes pulse { from { opacity: 0; } } .btn { animation: pulse 1s linear; animation-name: pulse; } }`,
  },
  {
    label: "scss-mixed-value-refs-parser-evaluator-candidates",
    filePath: "/f.module.scss",
    source: `@supports (display: grid) { @layer ui { @value brand from "./tokens.module.scss"; @value accent: red; .btn { color: brand; background: accent; } } }`,
  },
  {
    label: "css-custom-property-parser-evaluator-candidates",
    filePath: "/f.module.css",
    source: `:root { --color-gray-700: #767678; }\n@media (min-width: 1px) { .btn { color: var(--color-gray-700); } }\n.card { color: var(--missing); }`,
  },
  {
    label: "scss-mixed-composes-parser-evaluator-candidates",
    filePath: "/f.module.scss",
    source: `@supports (display: grid) { @layer ui { .card { composes: base utility; composes: shell from global; composes: tone from "./base.module.scss"; } } }`,
  },
  {
    label: "scss-grouped-bem-unsafe-parser-evaluator-candidates",
    filePath: "/f.module.scss",
    source: `.a, .b { &__icon { &--small { color: red; } } }`,
  },
] as const;

function comparePosition(
  left: { readonly line: number; readonly character: number },
  right: { readonly line: number; readonly character: number },
): number {
  if (left.line !== right.line) return left.line - right.line;
  return left.character - right.character;
}

function rangeContains(outer: Range, inner: Range): boolean {
  return (
    comparePosition(outer.start, inner.start) <= 0 && comparePosition(outer.end, inner.end) >= 0
  );
}

function findLangForPath(filePath: string): "scss" | "less" | "css" {
  if (filePath.endsWith(".module.scss")) return "scss";
  if (filePath.endsWith(".module.less")) return "less";
  return "css";
}

function getRuntimeSyntax(lang: "scss" | "less" | "css") {
  switch (lang) {
    case "scss":
      return safeParser;
    case "less":
      return safeParser;
    case "css":
      return null;
  }
}

function collectWrapperNamesForRanges(
  filePath: string,
  source: string,
  entries: readonly { readonly name: string; readonly ruleRange: Range }[],
) {
  const lang = findLangForPath(filePath);
  const syntax = getRuntimeSyntax(lang);
  const root =
    typeof syntax?.parse === "function"
      ? (syntax.parse(source, { from: filePath }) as Root)
      : (postcssParse(source, { from: filePath }) as Root);

  const media = new Set<string>();
  const supports = new Set<string>();
  const layer = new Set<string>();

  function walk(
    nodes: readonly ChildNode[],
    ctx: {
      readonly underMedia: boolean;
      readonly underSupports: boolean;
      readonly underLayer: boolean;
    },
  ): void {
    for (const node of nodes) {
      if (node.type === "rule") {
        const rule = node as Rule;
        const ruleRange: Range = {
          start: { line: rule.source!.start!.line - 1, character: rule.source!.start!.column - 1 },
          end: { line: rule.source!.end!.line - 1, character: rule.source!.end!.column - 1 },
        };
        for (const entry of entries) {
          if (!rangeContains(ruleRange, entry.ruleRange)) continue;
          if (ctx.underMedia) media.add(entry.name);
          if (ctx.underSupports) supports.add(entry.name);
          if (ctx.underLayer) layer.add(entry.name);
        }
        walk(rule.nodes ?? [], ctx);
        continue;
      }
      if (node.type === "atrule") {
        const atRule = node as AtRule;
        const atRuleRange: Range = {
          start: {
            line: atRule.source!.start!.line - 1,
            character: atRule.source!.start!.column - 1,
          },
          end: { line: atRule.source!.end!.line - 1, character: atRule.source!.end!.column - 1 },
        };
        for (const entry of entries) {
          if (!rangeContains(atRuleRange, entry.ruleRange)) continue;
          if (ctx.underMedia) media.add(entry.name);
          if (ctx.underSupports) supports.add(entry.name);
          if (ctx.underLayer) layer.add(entry.name);
        }
        walk(atRule.nodes ?? [], {
          underMedia: ctx.underMedia || atRule.name === "media",
          underSupports: ctx.underSupports || atRule.name === "supports",
          underLayer: ctx.underLayer || atRule.name === "layer",
        });
      }
    }
  }

  walk(root.nodes ?? [], { underMedia: false, underSupports: false, underLayer: false });
  return {
    media: [...media].toSorted(),
    supports: [...supports].toSorted(),
    layer: [...layer].toSorted(),
  };
}

function deriveTsSummary(filePath: string, source: string): ParserEvaluatorCandidatesV0 {
  const document = parseStyleDocument(source, filePath);
  const localValueNames = new Set(document.valueDecls.map((entry) => entry.name));
  const importedValueNames = new Set(document.valueImports.map((entry) => entry.name));
  const wrapperSelectorNames = collectWrapperNamesForRanges(
    filePath,
    source,
    document.selectors.map((selector) => ({ name: selector.name, ruleRange: selector.ruleRange })),
  );

  return {
    schemaVersion: "0",
    language: findLangForPath(filePath),
    results: [...document.selectors]
      .map((selector) => {
        const valueRefs = document.valueRefs.filter((entry) =>
          rangeContains(selector.ruleRange, entry.range),
        );
        const animationRefs = document.animationNameRefs.filter(
          (entry) =>
            entry.property === "animation" && rangeContains(selector.ruleRange, entry.range),
        );
        const animationNameRefs = document.animationNameRefs.filter(
          (entry) =>
            entry.property === "animation-name" && rangeContains(selector.ruleRange, entry.range),
        );
        const customPropertyRefs = document.customPropertyRefs.filter((entry) =>
          rangeContains(selector.ruleRange, entry.range),
        );
        const hasLocalComposes = selector.composes.some(
          (ref) => ref.from === undefined && ref.fromGlobal !== true,
        );
        const hasImportedComposes = selector.composes.some((ref) => ref.from !== undefined);
        const hasGlobalComposes = selector.composes.some((ref) => ref.fromGlobal === true);

        const candidate: ParserEvaluatorCandidateV0 = {
          kind: "selector-index-facts",
          selectorName: selector.name,
          nestedSafetyKind: selector.nestedSafety,
          underMedia: wrapperSelectorNames.media.includes(selector.name),
          underSupports: wrapperSelectorNames.supports.includes(selector.name),
          underLayer: wrapperSelectorNames.layer.includes(selector.name),
          hasValueRefs: valueRefs.length > 0,
          hasLocalValueRefs: valueRefs.some((entry) => localValueNames.has(entry.name)),
          hasImportedValueRefs: valueRefs.some((entry) => importedValueNames.has(entry.name)),
          hasCustomPropertyRefs: customPropertyRefs.length > 0,
          hasAnimationRef: animationRefs.length > 0,
          hasAnimationNameRef: animationNameRefs.length > 0,
          hasComposes: selector.composes.length > 0,
          hasLocalComposes,
          hasImportedComposes,
          hasGlobalComposes,
        };
        if (selector.bemSuffix) {
          candidate.bemSuffixParentName = selector.bemSuffix.parentResolvedName;
        }
        return candidate;
      })
      .toSorted((left, right) => left.selectorName.localeCompare(right.selectorName)),
  };
}

async function runRustSummary(
  filePath: string,
  source: string,
): Promise<ParserEvaluatorCandidatesV0> {
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
        "omena-parser-evaluator-candidates",
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
        reject(new Error(`omena-parser-evaluator-candidates exited with ${code}\n${stderr}`));
        return;
      }
      resolve(JSON.parse(stdout) as ParserEvaluatorCandidatesV0);
    });

    child.stdin.end(source);
  });
}

void (async () => {
  for (const entry of CORPUS) {
    process.stdout.write(`== rust-parser-evaluator-candidates:${entry.label} ==\n`);
    const expected = deriveTsSummary(entry.filePath, entry.source);
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runRustSummary(entry.filePath, entry.source);

    assert.deepEqual(
      actual,
      expected,
      [
        `parser evaluator-candidates mismatch for ${entry.label}`,
        `expected: ${JSON.stringify(expected, null, 2)}`,
        `actual: ${JSON.stringify(actual, null, 2)}`,
      ].join("\n"),
    );

    process.stdout.write(
      `validated parser evaluator candidates: selectors=${actual.results.length}\n\n`,
    );
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

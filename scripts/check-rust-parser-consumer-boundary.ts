import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";

import type { Range } from "@css-module-explainer/shared";

import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";
import { deriveSassSummary, type ParserSassSeedFactsV0 } from "./rust-parser-sass-facts";

interface ParserParityLiteSummaryV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
}

interface ParserIndexSummaryV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly selectors: {
    readonly names: readonly string[];
    readonly definitionFacts: readonly ParserSelectorDefinitionFactV0[];
    readonly bemSuffixParentNames: readonly string[];
    readonly bemSuffixSafeNames: readonly string[];
    readonly nestedUnsafeNames: readonly string[];
    readonly bemSuffixCount: number;
    readonly nestedSafetyCounts: {
      readonly flat: number;
      readonly bemSuffixSafe: number;
      readonly nestedUnsafe: number;
    };
  };
  readonly values: {
    readonly declNames: readonly string[];
    readonly declNamesWithImportedRefs: readonly string[];
    readonly importNames: readonly string[];
    readonly importSources: readonly string[];
  };
  readonly customProperties: {
    readonly declNames: readonly string[];
    readonly refNames: readonly string[];
    readonly selectorsWithRefsNames: readonly string[];
  };
  readonly keyframes: {
    readonly names: readonly string[];
    readonly animationRefNames: readonly string[];
    readonly animationNameRefNames: readonly string[];
  };
  readonly sass: ParserSassSeedFactsV0;
  readonly composes: {
    readonly importSources: readonly string[];
    readonly classNameCount: number;
    readonly importedClassNameCount: number;
  };
}

interface ParserSelectorDefinitionFactV0 {
  readonly name: string;
  readonly sourceOrder: number;
  readonly byteSpan?: {
    readonly start: number;
    readonly end: number;
  };
  readonly range: Range;
  readonly nestedSafetyKind: "flat" | "bemSuffixSafe" | "nestedUnsafe";
  readonly bemSuffixParentName?: string;
  readonly underMedia: boolean;
  readonly underSupports: boolean;
  readonly underLayer: boolean;
}

type ParserSelectorDefinitionSemanticFactV0 = Omit<
  ParserSelectorDefinitionFactV0,
  "byteSpan" | "range"
>;

interface ParserEvaluatorCandidateV0 {
  readonly selectorName: string;
  readonly hasLocalValueRefs: boolean;
  readonly hasImportedValueRefs: boolean;
  readonly hasCustomPropertyRefs: boolean;
  readonly hasLocalComposes: boolean;
  readonly hasImportedComposes: boolean;
  readonly hasGlobalComposes: boolean;
}

interface ParserEvaluatorCandidatesV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly results: readonly ParserEvaluatorCandidateV0[];
}

interface ParserCanonicalCandidateBundleV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly parityLite: ParserParityLiteSummaryV0;
  readonly cssModulesIntermediate: ParserIndexSummaryV0;
}

interface ParserCanonicalProducerSignalV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly canonicalCandidate: ParserCanonicalCandidateBundleV0;
  readonly evaluatorCandidates: ParserEvaluatorCandidatesV0;
}

interface ParserConsumerBoundarySummaryV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly selectorUsage: {
    readonly names: readonly string[];
    readonly definitionFacts: readonly ParserSelectorDefinitionSemanticFactV0[];
    readonly bemSuffixParentNames: readonly string[];
    readonly bemSuffixSafeNames: readonly string[];
    readonly nestedUnsafeNames: readonly string[];
    readonly bemSuffixCount: number;
    readonly nestedSafetyCounts: {
      readonly flat: number;
      readonly bemSuffixSafe: number;
      readonly nestedUnsafe: number;
    };
  };
  readonly composesResolution: {
    readonly selectorsNeedingResolutionNames: readonly string[];
    readonly localSelectorNames: readonly string[];
    readonly importedSelectorNames: readonly string[];
    readonly globalSelectorNames: readonly string[];
    readonly importSources: readonly string[];
    readonly totalClassNameCount: number;
    readonly importedClassNameCount: number;
  };
  readonly valueResolution: {
    readonly declNames: readonly string[];
    readonly importNames: readonly string[];
    readonly importSources: readonly string[];
    readonly selectorsWithLocalRefsNames: readonly string[];
    readonly selectorsWithImportedRefsNames: readonly string[];
    readonly declNamesWithImportedRefs: readonly string[];
  };
  readonly customPropertyResolution: {
    readonly declNames: readonly string[];
    readonly refNames: readonly string[];
    readonly selectorsWithRefsNames: readonly string[];
    readonly missingCandidateNames: readonly string[];
  };
  readonly keyframesResolution: {
    readonly declaredNames: readonly string[];
    readonly referencedNames: readonly string[];
    readonly missingCandidateNames: readonly string[];
  };
  readonly sassSymbolSeed: ParserSassSeedFactsV0;
  readonly sassSymbolResolution: ParserConsumerBoundarySassSymbolResolutionV0;
}

interface ParserConsumerBoundarySassSymbolResolutionV0 {
  readonly sameFileResolution: ParserSassSeedFactsV0["sameFileResolution"];
  readonly selectorsWithVariableRefsNames: readonly string[];
  readonly selectorsWithResolvedVariableRefsNames: readonly string[];
  readonly selectorsWithUnresolvedVariableRefsNames: readonly string[];
  readonly selectorsWithMixinIncludesNames: readonly string[];
  readonly selectorsWithResolvedMixinIncludesNames: readonly string[];
  readonly selectorsWithUnresolvedMixinIncludesNames: readonly string[];
  readonly selectorsWithFunctionCallsNames: readonly string[];
  readonly selectorSymbolFacts: ParserSassSeedFactsV0["selectorSymbolFacts"];
}

const CORPUS = [
  {
    label: "css-basic-parser-consumer-boundary",
    filePath: "/f.module.css",
    source: `.btn { color: red; }`,
  },
  {
    label: "scss-media-keyframes-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `@media (min-width: 1px) { @keyframes pulse { from { opacity: 0; } } .btn { animation: pulse 1s linear; animation-name: pulse; } }`,
  },
  {
    label: "scss-mixed-value-refs-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `@supports (display: grid) { @layer ui { @value brand from "./tokens.module.scss"; @value accent: red; .btn { color: brand; background: accent; } } }`,
  },
  {
    label: "css-custom-property-token-parser-consumer-boundary",
    filePath: "/f.module.css",
    source: `:root { --color-gray-700: #767678; }\n@media (min-width: 1px) { .btn { color: var(--color-gray-700); } }\n.card { color: var(--missing); }`,
  },
  {
    label: "scss-mixed-composes-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `@supports (display: grid) { @layer ui { .card { composes: base utility; composes: shell from global; composes: tone from "./base.module.scss"; } } }`,
  },
  {
    label: "scss-grouped-bem-unsafe-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `.a, .b { &__icon { &--small { color: red; } } }`,
  },
  {
    label: "scss-nested-bem-safety-matrix-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `.btn { &--primary {} &__icon {} &.active {} &-legacy {} &_legacy {} & + &--paired {} }`,
  },
  {
    label: "scss-non-bare-parent-bem-suffix-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `.card:hover { &--primary {} }`,
  },
  {
    label: "scss-descendant-parent-bem-suffix-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `.wrapper { .inner { &--mod {} } }`,
  },
  {
    label: "scss-unsafe-suffix-can-become-deep-bem-base-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `.btn { &-legacy { &--x {} } }`,
  },
  {
    label: "scss-sass-symbol-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `@use "./plain";\n@use "./reset" as *;\n@use "./tokens" as tokens;\n@use "sass:color";\n@forward "./theme";\n@import "./legacy";\n$gap: 1rem;\n@mixin raised($depth) { box-shadow: 0 0 $depth black; }\n@function tone($value) { @return $value; }\n.btn { color: $gap; @include raised($gap); border-color: tone($gap); }\n.ghost { color: $missing; @include absent($gap); }`,
  },
  {
    label: "scss-sass-symbol-unicode-range-parser-consumer-boundary",
    filePath: "/f.module.scss",
    source: `$gap: 1rem;\n.btn { content: "한🙂"; color: $gap; }`,
  },
] as const;

function findLangForPath(filePath: string): "scss" | "less" | "css" {
  if (filePath.endsWith(".module.scss")) return "scss";
  if (filePath.endsWith(".module.less")) return "less";
  return "css";
}

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

function assertSassSymbolProviderRangeInvariants(
  label: string,
  filePath: string,
  source: string,
  facts: ParserSassSeedFactsV0["selectorSymbolFacts"],
): void {
  const document = parseStyleDocument(source, filePath);
  for (const fact of facts) {
    const containingSelector = document.selectors.find(
      (selector) =>
        selector.name === fact.selectorName && rangeContains(selector.ruleRange, fact.range),
    );
    assert.ok(
      containingSelector,
      `${label}: Sass ${fact.symbolKind} ${fact.name} range must be contained by selector ${fact.selectorName}`,
    );
    assert.equal(
      sliceRange(source, fact.range),
      expectedSassSymbolToken(fact),
      `${label}: Sass ${fact.symbolKind} ${fact.name} range must point at the source token`,
    );
  }
}

function expectedSassSymbolToken(
  fact: ParserSassSeedFactsV0["selectorSymbolFacts"][number],
): string {
  return fact.symbolKind === "variable" ? `$${fact.name}` : fact.name;
}

function sliceRange(source: string, range: Range): string {
  return source.slice(
    codeUnitOffsetForPosition(source, range.start),
    codeUnitOffsetForPosition(source, range.end),
  );
}

function codeUnitOffsetForPosition(
  source: string,
  target: { readonly line: number; readonly character: number },
): number {
  let line = 0;
  let character = 0;
  let offset = 0;

  while (offset < source.length) {
    if (line === target.line && character === target.character) return offset;
    const codePoint = source.codePointAt(offset);
    if (codePoint === undefined) break;
    const codeUnitLength = codePoint > 0xffff ? 2 : 1;
    const token = source.slice(offset, offset + codeUnitLength);
    if (token === "\n") {
      line += 1;
      character = 0;
    } else {
      character += codeUnitLength;
    }
    offset += codeUnitLength;
  }

  if (line === target.line && character === target.character) return offset;
  throw new Error(`Could not map position ${target.line}:${target.character} to a source offset`);
}

function uniqueSorted(values: readonly string[]): string[] {
  return [...new Set(values)].toSorted((left, right) => left.localeCompare(right));
}

function deriveSassSymbolResolution(
  sass: ParserSassSeedFactsV0,
): ParserConsumerBoundarySassSymbolResolutionV0 {
  return {
    sameFileResolution: sass.sameFileResolution,
    selectorsWithVariableRefsNames: [...sass.selectorsWithVariableRefsNames],
    selectorsWithResolvedVariableRefsNames: [...sass.selectorsWithResolvedVariableRefsNames],
    selectorsWithUnresolvedVariableRefsNames: [...sass.selectorsWithUnresolvedVariableRefsNames],
    selectorsWithMixinIncludesNames: [...sass.selectorsWithMixinIncludesNames],
    selectorsWithResolvedMixinIncludesNames: [...sass.selectorsWithResolvedMixinIncludesNames],
    selectorsWithUnresolvedMixinIncludesNames: [...sass.selectorsWithUnresolvedMixinIncludesNames],
    selectorsWithFunctionCallsNames: [...sass.selectorsWithFunctionCallsNames],
    selectorSymbolFacts: [...sass.selectorSymbolFacts],
  };
}

function stripParserOnlySassSeedFacts(sass: ParserSassSeedFactsV0): ParserSassSeedFactsV0 {
  return {
    variableDeclNames: sass.variableDeclNames,
    variableParameterNames: sass.variableParameterNames,
    variableRefNames: sass.variableRefNames,
    selectorsWithVariableRefsNames: sass.selectorsWithVariableRefsNames,
    selectorsWithResolvedVariableRefsNames: sass.selectorsWithResolvedVariableRefsNames,
    selectorsWithUnresolvedVariableRefsNames: sass.selectorsWithUnresolvedVariableRefsNames,
    mixinDeclNames: sass.mixinDeclNames,
    mixinIncludeNames: sass.mixinIncludeNames,
    selectorsWithMixinIncludesNames: sass.selectorsWithMixinIncludesNames,
    selectorsWithResolvedMixinIncludesNames: sass.selectorsWithResolvedMixinIncludesNames,
    selectorsWithUnresolvedMixinIncludesNames: sass.selectorsWithUnresolvedMixinIncludesNames,
    functionDeclNames: sass.functionDeclNames,
    functionCallNames: sass.functionCallNames,
    selectorsWithFunctionCallsNames: sass.selectorsWithFunctionCallsNames,
    selectorSymbolFacts: sass.selectorSymbolFacts,
    moduleUseSources: sass.moduleUseSources,
    moduleUseEdges: sass.moduleUseEdges,
    moduleForwardSources: sass.moduleForwardSources,
    moduleImportSources: sass.moduleImportSources,
    sameFileResolution: sass.sameFileResolution,
  };
}

function deriveSummaryFromProducer(
  producer: ParserCanonicalProducerSignalV0,
): ParserConsumerBoundarySummaryV0 {
  const intermediate = producer.canonicalCandidate.cssModulesIntermediate;
  const evaluator = producer.evaluatorCandidates.results;
  const sassSummary = stripParserOnlySassSeedFacts(intermediate.sass);
  const referencedNames = uniqueSorted([
    ...intermediate.keyframes.animationRefNames,
    ...intermediate.keyframes.animationNameRefNames,
  ]);
  const declaredKeyframes = new Set(intermediate.keyframes.names);
  const declaredCustomProperties = new Set(intermediate.customProperties.declNames);

  return {
    schemaVersion: "0",
    language: producer.language,
    selectorUsage: {
      names: [...intermediate.selectors.names],
      definitionFacts: sortSelectorDefinitionFacts(
        intermediate.selectors.definitionFacts.map(toSelectorDefinitionSemanticFact),
      ),
      bemSuffixParentNames: [...intermediate.selectors.bemSuffixParentNames],
      bemSuffixSafeNames: [...intermediate.selectors.bemSuffixSafeNames],
      nestedUnsafeNames: [...intermediate.selectors.nestedUnsafeNames],
      bemSuffixCount: intermediate.selectors.bemSuffixCount,
      nestedSafetyCounts: intermediate.selectors.nestedSafetyCounts,
    },
    composesResolution: {
      selectorsNeedingResolutionNames: uniqueSorted(
        evaluator
          .filter((candidate) => candidate.hasLocalComposes || candidate.hasImportedComposes)
          .map((candidate) => candidate.selectorName),
      ),
      localSelectorNames: uniqueSorted(
        evaluator
          .filter((candidate) => candidate.hasLocalComposes)
          .map((candidate) => candidate.selectorName),
      ),
      importedSelectorNames: uniqueSorted(
        evaluator
          .filter((candidate) => candidate.hasImportedComposes)
          .map((candidate) => candidate.selectorName),
      ),
      globalSelectorNames: uniqueSorted(
        evaluator
          .filter((candidate) => candidate.hasGlobalComposes)
          .map((candidate) => candidate.selectorName),
      ),
      importSources: [...intermediate.composes.importSources],
      totalClassNameCount: intermediate.composes.classNameCount,
      importedClassNameCount: intermediate.composes.importedClassNameCount,
    },
    valueResolution: {
      declNames: [...intermediate.values.declNames],
      importNames: [...intermediate.values.importNames],
      importSources: [...intermediate.values.importSources],
      selectorsWithLocalRefsNames: uniqueSorted(
        evaluator
          .filter((candidate) => candidate.hasLocalValueRefs)
          .map((candidate) => candidate.selectorName),
      ),
      selectorsWithImportedRefsNames: uniqueSorted(
        evaluator
          .filter((candidate) => candidate.hasImportedValueRefs)
          .map((candidate) => candidate.selectorName),
      ),
      declNamesWithImportedRefs: [...intermediate.values.declNamesWithImportedRefs],
    },
    customPropertyResolution: {
      declNames: [...intermediate.customProperties.declNames],
      refNames: [...intermediate.customProperties.refNames],
      selectorsWithRefsNames: uniqueSorted(
        evaluator
          .filter((candidate) => candidate.hasCustomPropertyRefs)
          .map((candidate) => candidate.selectorName),
      ),
      missingCandidateNames: intermediate.customProperties.refNames.filter(
        (name) => !declaredCustomProperties.has(name),
      ),
    },
    keyframesResolution: {
      declaredNames: [...intermediate.keyframes.names],
      referencedNames,
      missingCandidateNames: referencedNames.filter((name) => !declaredKeyframes.has(name)),
    },
    sassSymbolSeed: sassSummary,
    sassSymbolResolution: deriveSassSymbolResolution(sassSummary),
  };
}

function deriveTsSummary(filePath: string, source: string): ParserConsumerBoundarySummaryV0 {
  const document = parseStyleDocument(source, filePath);
  const localValueNames = new Set(document.valueDecls.map((entry) => entry.name));
  const importedValueNames = new Set(document.valueImports.map((entry) => entry.name));
  const referencedNames = uniqueSorted(document.animationNameRefs.map((entry) => entry.name));
  const declaredKeyframes = new Set(document.keyframes.map((entry) => entry.name));
  const declaredCustomProperties = new Set(document.customPropertyDecls.map((entry) => entry.name));
  const sassSummary = deriveSassSummary(source);

  return {
    schemaVersion: "0",
    language: findLangForPath(filePath),
    selectorUsage: {
      names: document.selectors.map((selector) => selector.name).toSorted(),
      definitionFacts: sortSelectorDefinitionFacts(
        document.selectors.map((selector, sourceOrder) => ({
          name: selector.name,
          sourceOrder,
          nestedSafetyKind: selector.nestedSafety,
          ...(selector.bemSuffix?.parentResolvedName
            ? { bemSuffixParentName: selector.bemSuffix.parentResolvedName }
            : {}),
          underMedia:
            selector.context?.wrapperAtRules.some((atRule) => atRule.name === "media") ?? false,
          underSupports:
            selector.context?.wrapperAtRules.some((atRule) => atRule.name === "supports") ?? false,
          underLayer:
            selector.context?.wrapperAtRules.some((atRule) => atRule.name === "layer") ?? false,
        })),
      ),
      bemSuffixParentNames: document.selectors
        .map((selector) => selector.bemSuffix?.parentResolvedName)
        .filter((value): value is string => value !== undefined)
        .toSorted(),
      bemSuffixSafeNames: document.selectors
        .filter((selector) => selector.nestedSafety === "bemSuffixSafe")
        .map((selector) => selector.name)
        .toSorted(),
      nestedUnsafeNames: document.selectors
        .filter((selector) => selector.nestedSafety === "nestedUnsafe")
        .map((selector) => selector.name)
        .toSorted(),
      bemSuffixCount: document.selectors.filter((selector) => selector.bemSuffix).length,
      nestedSafetyCounts: {
        flat: document.selectors.filter((selector) => selector.nestedSafety === "flat").length,
        bemSuffixSafe: document.selectors.filter(
          (selector) => selector.nestedSafety === "bemSuffixSafe",
        ).length,
        nestedUnsafe: document.selectors.filter(
          (selector) => selector.nestedSafety === "nestedUnsafe",
        ).length,
      },
    },
    composesResolution: {
      selectorsNeedingResolutionNames: uniqueSorted(
        document.selectors
          .filter((selector) =>
            selector.composes.some((ref) => ref.from === undefined && ref.fromGlobal !== true),
          )
          .map((selector) => selector.name)
          .concat(
            document.selectors
              .filter((selector) => selector.composes.some((ref) => ref.from !== undefined))
              .map((selector) => selector.name),
          ),
      ),
      localSelectorNames: uniqueSorted(
        document.selectors
          .filter((selector) =>
            selector.composes.some((ref) => ref.from === undefined && ref.fromGlobal !== true),
          )
          .map((selector) => selector.name),
      ),
      importedSelectorNames: uniqueSorted(
        document.selectors
          .filter((selector) => selector.composes.some((ref) => ref.from !== undefined))
          .map((selector) => selector.name),
      ),
      globalSelectorNames: uniqueSorted(
        document.selectors
          .filter((selector) => selector.composes.some((ref) => ref.fromGlobal === true))
          .map((selector) => selector.name),
      ),
      importSources: uniqueSorted(
        document.selectors.flatMap((selector) =>
          selector.composes
            .map((ref) => ref.from)
            .filter((value): value is string => value !== undefined),
        ),
      ),
      totalClassNameCount: document.selectors.reduce(
        (sum, selector) =>
          sum + selector.composes.reduce((inner, ref) => inner + ref.classNames.length, 0),
        0,
      ),
      importedClassNameCount: document.selectors.reduce(
        (sum, selector) =>
          sum +
          selector.composes.reduce(
            (inner, ref) => inner + (ref.from !== undefined ? ref.classNames.length : 0),
            0,
          ),
        0,
      ),
    },
    valueResolution: {
      declNames: document.valueDecls.map((entry) => entry.name).toSorted(),
      importNames: document.valueImports.map((entry) => entry.name).toSorted(),
      importSources: uniqueSorted(document.valueImports.map((entry) => entry.from)),
      selectorsWithLocalRefsNames: uniqueSorted(
        document.selectors
          .filter((selector) =>
            document.valueRefs.some(
              (entry) =>
                localValueNames.has(entry.name) && rangeContains(selector.ruleRange, entry.range),
            ),
          )
          .map((selector) => selector.name),
      ),
      selectorsWithImportedRefsNames: uniqueSorted(
        document.selectors
          .filter((selector) =>
            document.valueRefs.some(
              (entry) =>
                importedValueNames.has(entry.name) &&
                rangeContains(selector.ruleRange, entry.range),
            ),
          )
          .map((selector) => selector.name),
      ),
      declNamesWithImportedRefs: uniqueSorted(
        document.valueDecls
          .filter((decl) =>
            document.valueRefs.some(
              (entry) =>
                importedValueNames.has(entry.name) && rangeContains(decl.ruleRange, entry.range),
            ),
          )
          .map((decl) => decl.name),
      ),
    },
    customPropertyResolution: {
      declNames: uniqueSorted(document.customPropertyDecls.map((entry) => entry.name)),
      refNames: uniqueSorted(document.customPropertyRefs.map((entry) => entry.name)),
      selectorsWithRefsNames: uniqueSorted(
        document.selectors
          .filter((selector) =>
            document.customPropertyRefs.some((entry) =>
              rangeContains(selector.ruleRange, entry.range),
            ),
          )
          .map((selector) => selector.name),
      ),
      missingCandidateNames: uniqueSorted(
        document.customPropertyRefs
          .map((entry) => entry.name)
          .filter((name) => !declaredCustomProperties.has(name)),
      ),
    },
    keyframesResolution: {
      declaredNames: document.keyframes.map((entry) => entry.name).toSorted(),
      referencedNames,
      missingCandidateNames: referencedNames.filter((name) => !declaredKeyframes.has(name)),
    },
    sassSymbolSeed: sassSummary,
    sassSymbolResolution: deriveSassSymbolResolution(sassSummary),
  };
}

function toSelectorDefinitionSemanticFact(
  fact: ParserSelectorDefinitionFactV0,
): ParserSelectorDefinitionSemanticFactV0 {
  const { byteSpan: _byteSpan, range: _range, ...rest } = fact;
  return rest;
}

function sortSelectorDefinitionFacts(
  facts: readonly ParserSelectorDefinitionSemanticFactV0[],
): ParserSelectorDefinitionSemanticFactV0[] {
  return [...facts].toSorted((left, right) => {
    const name = left.name.localeCompare(right.name);
    if (name !== 0) return name;
    return left.sourceOrder - right.sourceOrder;
  });
}

function assertSelectorDefinitionRangeInvariants(
  label: string,
  source: string,
  facts: readonly ParserSelectorDefinitionFactV0[],
): void {
  for (const fact of facts) {
    const token = sliceRange(source, fact.range);
    assert.ok(token.length > 0, `${label}: selector ${fact.name} must have a non-empty range`);
    assert.ok(
      token === fact.name || (/^[-_]/.test(token) && fact.name.endsWith(token)),
      `${label}: selector ${fact.name} range must point at its source token; got ${JSON.stringify(token)}`,
    );
  }
}

async function runRustJson<T>(bin: string, filePath: string, source: string): Promise<T> {
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
        bin,
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
        reject(new Error(`${bin} exited with ${code}\n${stderr}`));
        return;
      }
      resolve(JSON.parse(stdout) as T);
    });

    child.stdin.end(source);
  });
}

void (async () => {
  for (const entry of CORPUS) {
    process.stdout.write(`== rust-parser-consumer-boundary:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const producer = await runRustJson<ParserCanonicalProducerSignalV0>(
      "omena-parser-canonical-producer",
      entry.filePath,
      entry.source,
    );
    const actual = deriveSummaryFromProducer(producer);
    const expected = deriveTsSummary(entry.filePath, entry.source);
    assertSelectorDefinitionRangeInvariants(
      entry.label,
      entry.source,
      producer.canonicalCandidate.cssModulesIntermediate.selectors.definitionFacts,
    );

    assert.deepEqual(
      actual,
      expected,
      [
        `parser consumer-boundary mismatch for ${entry.label}`,
        `expected: ${JSON.stringify(expected, null, 2)}`,
        `actual: ${JSON.stringify(actual, null, 2)}`,
      ].join("\n"),
    );
    assertSassSymbolProviderRangeInvariants(
      entry.label,
      entry.filePath,
      entry.source,
      actual.sassSymbolResolution.selectorSymbolFacts,
    );

    process.stdout.write(
      `validated parser consumer-boundary: selectors=${actual.selectorUsage.names.length} composes=${actual.composesResolution.totalClassNameCount} valueImports=${actual.valueResolution.importNames.length} sassVars=${actual.sassSymbolSeed.variableDeclNames.length} sassSymbols=${actual.sassSymbolResolution.selectorSymbolFacts.length}\n\n`,
    );
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});

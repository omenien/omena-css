import type { BemSuffixInfo, ComposesRef, Range } from "@omena/shared";
import {
  makeStyleDocumentHIR,
  type AnimationNameRefHIR,
  type KeyframesDeclHIR,
  type NestedSelectorSafety,
  type SelectorDeclHIR,
  type StyleDocumentHIR,
  type ValueDeclHIR,
  type ValueImportHIR,
  type ValueRefHIR,
} from "../../server/engine-core-ts/src/core/hir/style-types";
import {
  expandStyleDocumentWithTransform,
  type ClassnameTransformMode,
} from "../../server/engine-core-ts/src/core/scss/classname-transform";
import { parseStyleDocument } from "../../server/engine-core-ts/src/core/scss/scss-parser";

interface SelectorOptions {
  readonly declarations?: string;
  readonly fullSelector?: string;
  readonly range?: Range;
  readonly ruleRange?: Range;
  readonly composes?: readonly ComposesRef[];
  readonly nestedSafety?: NestedSelectorSafety;
  readonly bemSuffix?: BemSuffixInfo;
  readonly originalName?: string;
  readonly canonicalName?: string;
  readonly viewKind?: "canonical" | "alias";
  readonly id?: string;
}

export function makeTestSelector(
  name: string,
  line: number,
  options: SelectorOptions = {},
): SelectorDeclHIR {
  const range = options.range ?? {
    start: { line, character: 2 },
    end: { line, character: 2 + name.length },
  };
  const ruleRange = options.ruleRange ?? {
    start: { line, character: 0 },
    end: { line: line + 3, character: 1 },
  };
  const canonicalName = options.canonicalName ?? options.originalName ?? name;
  const viewKind = options.viewKind ?? (options.originalName ? "alias" : "canonical");

  return {
    kind: "selector",
    id: options.id ?? `selector:${line}:${name}`,
    name,
    canonicalName,
    viewKind,
    range,
    fullSelector: options.fullSelector ?? `.${name}`,
    declarations: options.declarations ?? "color: red",
    ruleRange,
    composes: options.composes ?? [],
    nestedSafety: options.nestedSafety ?? "flat",
    ...(options.bemSuffix ? { bemSuffix: options.bemSuffix } : {}),
    ...(options.originalName ? { originalName: options.originalName } : {}),
  };
}

export function makeStyleDocumentFixture(
  filePath: string,
  selectors: readonly SelectorDeclHIR[],
  keyframes: readonly KeyframesDeclHIR[] = [],
  animationNameRefs: readonly AnimationNameRefHIR[] = [],
  valueDecls: readonly ValueDeclHIR[] = [],
  valueImports: readonly ValueImportHIR[] = [],
  valueRefs: readonly ValueRefHIR[] = [],
): StyleDocumentHIR {
  return makeStyleDocumentHIR(
    filePath,
    [...selectors].toSorted(compareByRangeAndName),
    [...keyframes].toSorted(compareByRangeAndName),
    [...animationNameRefs].toSorted(compareByRangeAndName),
    [...valueDecls].toSorted(compareByRangeAndName),
    [...valueImports].toSorted(compareByRangeAndName),
    [...valueRefs].toSorted(compareByRangeAndName),
  );
}

export function selectorMapFromDocument(
  document: StyleDocumentHIR,
): ReadonlyMap<string, SelectorDeclHIR> {
  return new Map(document.selectors.map((selector) => [selector.name, selector]));
}

export function buildStyleDocumentFromSelectorMap(
  filePath: string,
  selectors: ReadonlyMap<string, SelectorDeclHIR>,
): StyleDocumentHIR {
  return makeStyleDocumentFixture(filePath, Array.from(selectors.values()));
}

export function parseStyleSelectorMap(
  content: string,
  filePath: string,
): ReadonlyMap<string, SelectorDeclHIR> {
  return selectorMapFromDocument(parseStyleDocument(content, filePath));
}

export function expandSelectorMapWithTransform(
  selectors: ReadonlyMap<string, SelectorDeclHIR>,
  mode: ClassnameTransformMode,
): ReadonlyMap<string, SelectorDeclHIR> {
  if (mode === "asIs") return selectors;
  return selectorMapFromDocument(
    expandStyleDocumentWithTransform(
      buildStyleDocumentFromSelectorMap("/selectors.module.scss", selectors),
      mode,
    ),
  );
}

function compareByRangeAndName(
  a: { range: { start: { line: number; character: number } }; name: string },
  b: { range: { start: { line: number; character: number } }; name: string },
): number {
  const line = a.range.start.line - b.range.start.line;
  if (line !== 0) return line;
  const character = a.range.start.character - b.range.start.character;
  if (character !== 0) return character;
  return a.name.localeCompare(b.name);
}

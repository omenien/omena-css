import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import type { AbstractClassValue } from "./class-value-domain";

export interface ClassValueUniverseAxisValueV0 {
  readonly name: string;
  readonly classNames: readonly string[];
}

export interface ClassValueUniverseAxisV0 {
  readonly axisName: string;
  readonly values: readonly ClassValueUniverseAxisValueV0[];
  readonly defaultValue?: string;
  readonly role?: "variant" | "slot";
  readonly reserved?: true;
}

export interface ClassValueUniverseCompoundVariantV0 {
  readonly conditions: readonly ClassValueUniverseConditionV0[];
  readonly classNames: readonly string[];
}

export interface ClassValueUniverseConditionV0 {
  readonly axisName: string;
  readonly value: string;
}

export type ClassValueUniverseLookupResultV0 =
  | NoneClassValueUniverseV0
  | FiniteClassValueUniverseV0
  | ReducedProductClassValueUniverseV0;

export interface NoneClassValueUniverseV0 {
  readonly kind: "none";
}

export interface FiniteClassValueUniverseV0 {
  readonly kind: "finite";
  readonly classNames: readonly string[];
}

export interface ReducedProductClassValueUniverseV0 {
  readonly kind: "reduced-product";
  readonly baseClassNames: readonly string[];
  readonly axes: readonly ClassValueUniverseAxisV0[];
  readonly compoundVariants: readonly ClassValueUniverseCompoundVariantV0[];
}

export const NONE_CLASS_VALUE_UNIVERSE_V0: NoneClassValueUniverseV0 = { kind: "none" };

export function finiteClassValueUniverseV0(
  classNames: readonly string[],
): ClassValueUniverseLookupResultV0 {
  const normalized = uniqueSortedStrings(classNames);
  return normalized.length === 0
    ? NONE_CLASS_VALUE_UNIVERSE_V0
    : { kind: "finite", classNames: normalized };
}

export function reducedProductClassValueUniverseV0(input: {
  readonly baseClassNames?: readonly string[];
  readonly axes: readonly ClassValueUniverseAxisV0[];
  readonly compoundVariants?: readonly ClassValueUniverseCompoundVariantV0[];
}): ClassValueUniverseLookupResultV0 {
  const baseClassNames = uniqueSortedStrings(input.baseClassNames ?? []);
  const axes = input.axes.map((axis) => ({
    ...axis,
    values: axis.values.map((value) => ({
      ...value,
      classNames: uniqueSortedStrings(value.classNames),
    })),
  }));
  const compoundVariants = (input.compoundVariants ?? []).map((compound) => ({
    conditions: [...compound.conditions].toSorted(compareConditions),
    classNames: uniqueSortedStrings(compound.classNames),
  }));
  if (
    baseClassNames.length === 0 &&
    axes.every((axis) => axis.values.length === 0 && !axis.reserved) &&
    compoundVariants.every((compound) => compound.classNames.length === 0)
  ) {
    return NONE_CLASS_VALUE_UNIVERSE_V0;
  }
  return {
    kind: "reduced-product",
    baseClassNames,
    axes,
    compoundVariants,
  };
}

export function classValueUniverseFromStyleDocument(
  styleDocument: StyleDocumentHIR,
): ClassValueUniverseLookupResultV0 {
  return finiteClassValueUniverseV0(styleDocument.selectors.map((selector) => selector.name));
}

export function projectAbstractValueClassNames(
  value: AbstractClassValue,
  universe: ClassValueUniverseLookupResultV0,
): readonly string[] {
  const classNames = classNamesForUniverse(universe);
  switch (value.kind) {
    case "bottom":
      return [];
    case "exact":
      return classNames.filter((className) => className === value.value);
    case "finiteSet": {
      const candidates = new Set(value.values);
      return classNames.filter((className) => candidates.has(className));
    }
    case "prefix":
      return classNames.filter((className) => className.startsWith(value.prefix));
    case "suffix":
      return classNames.filter((className) => className.endsWith(value.suffix));
    case "prefixSuffix":
      return classNames.filter(
        (className) => className.startsWith(value.prefix) && className.endsWith(value.suffix),
      );
    case "charInclusion":
      return classNames.filter((className) =>
        satisfiesCharInclusion(
          className,
          value.mustChars,
          value.mayChars,
          Boolean(value.mayIncludeOtherChars),
        ),
      );
    case "composite":
      return classNames.filter(
        (className) =>
          (value.prefix === undefined || className.startsWith(value.prefix)) &&
          (value.suffix === undefined || className.endsWith(value.suffix)) &&
          (value.minLength === undefined || className.length >= value.minLength) &&
          satisfiesCharInclusion(
            className,
            value.mustChars,
            value.mayChars,
            Boolean(value.mayIncludeOtherChars),
          ),
      );
    case "top":
      return classNames;
    default:
      value satisfies never;
      return [];
  }
}

export function classNamesForUniverse(
  universe: ClassValueUniverseLookupResultV0,
): readonly string[] {
  switch (universe.kind) {
    case "none":
      return [];
    case "finite":
      return uniqueSortedStrings(universe.classNames);
    case "reduced-product":
      return uniqueSortedStrings([
        ...universe.baseClassNames,
        ...universe.axes.flatMap((axis) => axis.values.flatMap((value) => value.classNames)),
        ...universe.compoundVariants.flatMap((compound) => compound.classNames),
      ]);
    default:
      universe satisfies never;
      return [];
  }
}

export function selectorsForProjectedClassNames(
  styleDocument: StyleDocumentHIR,
  classNames: readonly string[],
): readonly SelectorDeclHIR[] {
  const candidates = new Set(classNames);
  return uniqueSelectorsById(
    styleDocument.selectors.flatMap((selector) =>
      candidates.has(selector.name) ? findCanonicalSelectors(styleDocument, selector.name) : [],
    ),
  );
}

function findCanonicalSelectors(
  styleDocument: StyleDocumentHIR,
  viewName: string,
): readonly SelectorDeclHIR[] {
  const matches = styleDocument.selectors.filter((selector) => selector.name === viewName);
  if (matches.length === 0) return [];
  const canonicalNames = new Set(matches.map((selector) => selector.canonicalName));
  const canonicalSelectors = styleDocument.selectors.filter(
    (selector) => selector.viewKind === "canonical" && canonicalNames.has(selector.canonicalName),
  );
  return canonicalSelectors.length > 0 ? canonicalSelectors : matches;
}

function uniqueSelectorsById(selectors: readonly SelectorDeclHIR[]): readonly SelectorDeclHIR[] {
  const emitted = new Set<string>();
  const result: SelectorDeclHIR[] = [];
  for (const selector of selectors) {
    if (emitted.has(selector.id)) continue;
    emitted.add(selector.id);
    result.push(selector);
  }
  return result;
}

function satisfiesCharInclusion(
  className: string,
  mustChars: string,
  mayChars: string,
  mayIncludeOtherChars: boolean,
): boolean {
  const charSet = new Set(Array.from(className));
  const mustSet = new Set(Array.from(mustChars));
  const maySet = new Set(Array.from(mayChars));
  if (Array.from(mustSet).some((char) => !charSet.has(char))) return false;
  return mayIncludeOtherChars || !Array.from(charSet).some((char) => !maySet.has(char));
}

function uniqueSortedStrings(values: readonly string[]): readonly string[] {
  return Array.from(new Set(values.filter((value) => value.length > 0))).toSorted();
}

function compareConditions(
  left: ClassValueUniverseConditionV0,
  right: ClassValueUniverseConditionV0,
): number {
  return `${left.axisName}:${left.value}`.localeCompare(`${right.axisName}:${right.value}`);
}

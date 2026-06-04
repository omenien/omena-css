import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import type { ClassValueUniverseEntryV0 } from "../binder/class-value-universe-provider";
import { deriveSelectorProjectionCertainty, type EdgeCertainty } from "../semantic/certainty";
import type { AbstractClassValue } from "./class-value-domain";
import {
  classNamesForUniverse,
  classValueUniverseFromStyleDocument,
  finiteClassValueUniverseV0,
  projectAbstractValueClassNames,
  selectorsForProjectedClassNames,
  type ClassValueUniverseLookupResultV0,
} from "./class-value-universe";

export interface AbstractSelectorProjection {
  readonly selectors: readonly SelectorDeclHIR[];
  readonly certainty: EdgeCertainty;
}

export interface AbstractSelectorProjectionOptions {
  readonly classValueUniverses?: readonly ClassValueUniverseEntryV0[];
  readonly universeOwnerName?: string;
}

export function projectAbstractValueSelectors(
  value: AbstractClassValue,
  styleDocument: StyleDocumentHIR,
  options: AbstractSelectorProjectionOptions = {},
): AbstractSelectorProjection {
  const selectors = resolveAbstractValueSelectors(value, styleDocument, options);
  return {
    selectors,
    certainty: deriveSelectorProjectionCertainty(
      value,
      selectors.length,
      countCanonicalSelectors(styleDocument),
    ),
  };
}

export function resolveAbstractValueSelectors(
  value: AbstractClassValue,
  styleDocument: StyleDocumentHIR,
  options: AbstractSelectorProjectionOptions = {},
): readonly SelectorDeclHIR[] {
  const universe = classValueUniverseForProjection(styleDocument, options);
  const classNames = resolveAbstractValueClassNames(value, universe);
  return selectorsForProjectedClassNames(styleDocument, classNames);
}

export function resolveAbstractValueClassNames(
  value: AbstractClassValue,
  universe: ClassValueUniverseLookupResultV0,
): readonly string[] {
  return projectAbstractValueClassNames(value, universe);
}

function countCanonicalSelectors(styleDocument: StyleDocumentHIR): number {
  return styleDocument.selectors.filter((selector) => selector.viewKind === "canonical").length;
}

function classValueUniverseForProjection(
  styleDocument: StyleDocumentHIR,
  options: AbstractSelectorProjectionOptions,
): ClassValueUniverseLookupResultV0 {
  const ownerName = options.universeOwnerName;
  if (!ownerName || !options.classValueUniverses || options.classValueUniverses.length === 0) {
    return classValueUniverseFromStyleDocument(styleDocument);
  }
  const matchedUniverses = options.classValueUniverses
    .filter((entry) => entry.ownerName === ownerName)
    .map((entry) => entry.universe);
  if (matchedUniverses.length === 0) {
    return classValueUniverseFromStyleDocument(styleDocument);
  }
  return finiteClassValueUniverseV0(matchedUniverses.flatMap(classNamesForUniverse));
}

import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import { deriveSelectorProjectionCertainty, type EdgeCertainty } from "../semantic/certainty";
import type { AbstractClassValue } from "./class-value-domain";
import {
  classValueUniverseFromStyleDocument,
  projectAbstractValueClassNames,
  selectorsForProjectedClassNames,
  type ClassValueUniverseLookupResultV0,
} from "./class-value-universe";

export interface AbstractSelectorProjection {
  readonly selectors: readonly SelectorDeclHIR[];
  readonly certainty: EdgeCertainty;
}

export function projectAbstractValueSelectors(
  value: AbstractClassValue,
  styleDocument: StyleDocumentHIR,
): AbstractSelectorProjection {
  const selectors = resolveAbstractValueSelectors(value, styleDocument);
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
): readonly SelectorDeclHIR[] {
  const universe = classValueUniverseFromStyleDocument(styleDocument);
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

import type { ClassExpressionHIR } from "../hir/source-types";
import type { SelectorDeclHIR, StyleDocumentHIR } from "../hir/style-types";
import type { FlowResolution } from "../flow/lattice";
import type { EdgeCertainty } from "../semantic/certainty";
import { enumerateFiniteClassValues } from "../abstract-value/class-value-domain";
import type {
  ReadSourceExpressionResolutionContext,
  ReadSourceExpressionResolutionEnv,
} from "./read-source-expression-resolution";
import { readSourceExpressionResolution } from "./read-source-expression-resolution";

export type ExpressionValueDomainKind =
  | "none"
  | "exact"
  | "finiteSet"
  | "prefix"
  | "constrained"
  | "top";

export interface ReducedClassValueDerivation {
  readonly schemaVersion: string;
  readonly product: string;
  readonly inputFactKind: string;
  readonly inputConstraintKind?: string;
  readonly inputValueCount: number;
  readonly reducedKind: string;
  readonly steps: readonly ReducedClassValueDerivationStep[];
}

export interface ReducedClassValueDerivationStep {
  readonly operation: string;
  readonly inputKind?: string;
  readonly refinementKind?: string;
  readonly resultKind: string;
  readonly resultProvenance?: string;
  readonly reason: string;
}

export interface ExpressionSemanticsSummary {
  readonly expression: ClassExpressionHIR;
  readonly styleDocument: StyleDocumentHIR | null;
  readonly selectors: readonly SelectorDeclHIR[];
  readonly selectorNames: readonly string[];
  readonly candidateNames: readonly string[];
  readonly finiteValues: readonly string[] | null;
  readonly valueDomainKind: ExpressionValueDomainKind;
  readonly abstractValue?: FlowResolution["abstractValue"];
  readonly valueDomainDerivation?: ReducedClassValueDerivation;
  readonly valueCertainty?: EdgeCertainty;
  readonly selectorCertainty: EdgeCertainty;
  readonly reason?: FlowResolution["reason"];
}

export function readExpressionSemantics(
  ctx: ReadSourceExpressionResolutionContext,
  env: ReadSourceExpressionResolutionEnv,
): ExpressionSemanticsSummary {
  const resolution = readSourceExpressionResolution(ctx, env);
  const selectorNames = resolution.selectors.map((selector) => selector.name);
  const valueDomainKind = classifyValueDomain(resolution.abstractValue);
  const valueDomainDerivation = resolution.abstractValue
    ? buildReducedClassValueDerivation(resolution.abstractValue, valueDomainKind)
    : null;
  return {
    expression: ctx.expression,
    styleDocument: resolution.styleDocument,
    selectors: resolution.selectors,
    selectorNames,
    candidateNames: candidateNamesForResolution(resolution),
    finiteValues: resolution.finiteValues,
    valueDomainKind,
    ...(resolution.abstractValue ? { abstractValue: resolution.abstractValue } : {}),
    ...(valueDomainDerivation ? { valueDomainDerivation } : {}),
    ...(resolution.valueCertainty ? { valueCertainty: resolution.valueCertainty } : {}),
    ...(resolution.reason ? { reason: resolution.reason } : {}),
    selectorCertainty: resolution.selectorCertainty,
  };
}

function candidateNamesForResolution(
  resolution: ReturnType<typeof readSourceExpressionResolution>,
): readonly string[] {
  if (resolution.finiteValues && resolution.finiteValues.length > 0) {
    return resolution.finiteValues;
  }
  return resolution.selectors.map((selector) => selector.name);
}

function classifyValueDomain(
  abstractValue?: FlowResolution["abstractValue"],
): ExpressionValueDomainKind {
  if (!abstractValue) return "none";
  switch (abstractValue.kind) {
    case "bottom":
      return "none";
    case "exact":
      return "exact";
    case "finiteSet":
      return "finiteSet";
    case "prefix":
      return "prefix";
    case "suffix":
    case "prefixSuffix":
    case "charInclusion":
    case "composite":
      return "constrained";
    case "top":
      return "top";
    default:
      abstractValue satisfies never;
      return "none";
  }
}

function buildReducedClassValueDerivation(
  abstractValue: FlowResolution["abstractValue"],
  reducedKind: ExpressionValueDomainKind,
): ReducedClassValueDerivation {
  const inputConstraintKind = constraintKindForAbstractValue(abstractValue);
  const resultProvenance = provenanceForAbstractValue(abstractValue);
  const resultKind = inputConstraintKind ?? reducedKind;
  return {
    schemaVersion: "0",
    product: "omena-abstract-value.reduced-class-value-derivation",
    inputFactKind: reducedKind,
    ...(inputConstraintKind ? { inputConstraintKind } : {}),
    inputValueCount: finiteValueCountForAbstractValue(abstractValue),
    reducedKind,
    steps: [
      {
        operation: "baseFromFacts",
        resultKind,
        ...(resultProvenance ? { resultProvenance } : {}),
        reason:
          reducedKind === "exact" || reducedKind === "finiteSet"
            ? "preserved finite string literal facts"
            : reducedKind === "constrained" || reducedKind === "prefix"
              ? "preserved reduced string constraint facts"
              : "mapped input facts to the base abstract value",
      },
    ],
  };
}

function constraintKindForAbstractValue(
  abstractValue: FlowResolution["abstractValue"],
): "prefix" | "suffix" | "prefixSuffix" | "charInclusion" | "composite" | undefined {
  switch (abstractValue.kind) {
    case "prefix":
    case "suffix":
    case "prefixSuffix":
    case "charInclusion":
    case "composite":
      return abstractValue.kind;
    case "bottom":
    case "exact":
    case "finiteSet":
    case "top":
      return undefined;
    default:
      abstractValue satisfies never;
      return undefined;
  }
}

function provenanceForAbstractValue(
  abstractValue: FlowResolution["abstractValue"],
): string | undefined {
  switch (abstractValue.kind) {
    case "prefix":
    case "suffix":
    case "prefixSuffix":
    case "charInclusion":
    case "composite":
      return abstractValue.provenance;
    case "bottom":
    case "exact":
    case "finiteSet":
    case "top":
      return undefined;
    default:
      abstractValue satisfies never;
      return undefined;
  }
}

function finiteValueCountForAbstractValue(abstractValue: FlowResolution["abstractValue"]): number {
  return enumerateFiniteClassValues(abstractValue)?.length ?? 0;
}

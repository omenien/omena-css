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

export interface ValueDomainProvenanceTree {
  readonly schemaVersion: string;
  readonly product: string;
  readonly valueKind: FlowResolution["abstractValue"]["kind"];
  readonly value: FlowResolution["abstractValue"];
  readonly valueProvenance?: string;
  readonly root: ValueDomainProvenanceNode;
}

export interface ValueDomainProvenanceNode {
  readonly operation: string;
  readonly resultKind: string;
  readonly resultProvenance?: string;
  readonly detail?: string;
  readonly reason: string;
  readonly children: readonly ValueDomainProvenanceNode[];
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
  readonly valueDomainProvenanceTree?: ValueDomainProvenanceTree;
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
  const valueDomainProvenanceTree = resolution.abstractValue
    ? buildValueDomainProvenanceTree(resolution.abstractValue)
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
    ...(valueDomainProvenanceTree ? { valueDomainProvenanceTree } : {}),
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

function buildValueDomainProvenanceTree(
  abstractValue: FlowResolution["abstractValue"],
): ValueDomainProvenanceTree {
  const provenance = provenanceForAbstractValue(abstractValue);
  const rootDetail = provenanceTreeRootDetail(abstractValue);
  return {
    schemaVersion: "0",
    product: "omena-abstract-value.provenance-tree",
    valueKind: abstractValue.kind,
    value: abstractValue,
    ...(provenance ? { valueProvenance: provenance } : {}),
    root: {
      operation: provenanceTreeRootOperation(abstractValue, provenance),
      resultKind: abstractValue.kind,
      ...(provenance ? { resultProvenance: provenance } : {}),
      ...(rootDetail ? { detail: rootDetail } : {}),
      reason: provenanceTreeRootReason(abstractValue, provenance),
      children: provenanceTreeConstraintChildren(abstractValue),
    },
  };
}

function provenanceTreeRootOperation(
  abstractValue: FlowResolution["abstractValue"],
  provenance: string | undefined,
): string {
  switch (provenance) {
    case "finiteSetWidening":
    case "finiteSetWideningChars":
    case "finiteSetWideningComposite":
      return "finiteSetWidening";
    case "prefixJoinLcp":
      return "prefixJoinLongestCommonPrefix";
    case "suffixJoinLcs":
      return "suffixJoinLongestCommonSuffix";
    case "prefixSuffixJoin":
    case "charInclusionJoin":
    case "compositeJoin":
      return "reducedProductJoin";
    case "concatKnownEdges":
    case "finiteSetConcatPrefixLcp":
    case "finiteSetConcatSuffixProduct":
    case "charInclusionConcat":
    case "compositeConcat":
      return "reducedProductConcat";
    case "concatUnknownLeft":
    case "concatUnknownRight":
      return "concatenationWidening";
    case "unconstrainedInput":
      return "unconstrainedInput";
    case "automatonStateLimit":
      return "automatonStateWidening";
    case "flowIterationLimit":
      return "flowIterationWidening";
    case "missingFlowPredecessor":
      return "missingFlowPredecessor";
    case "joinUnrepresentable":
      return "unrepresentableJoin";
    case "concatenationUnrepresentable":
      return "unrepresentableConcatenation";
    case "reducedProductUnconstrained":
      return "reducedProductUnconstrained";
    case undefined:
      switch (abstractValue.kind) {
        case "bottom":
          return "bottomDomain";
        case "exact":
          return "exactLiteral";
        case "finiteSet":
          return "finiteSetDomain";
        case "prefix":
        case "suffix":
        case "prefixSuffix":
        case "charInclusion":
        case "composite":
          return "constraintDomain";
        case "top":
          return "topDomain";
        default:
          abstractValue satisfies never;
          return "unknownDomain";
      }
    default:
      return "constraintDomain";
  }
}

function provenanceTreeRootReason(
  abstractValue: FlowResolution["abstractValue"],
  provenance: string | undefined,
): string {
  switch (provenance) {
    case "finiteSetWidening":
    case "finiteSetWideningChars":
      return "large finite set widened to character or edge constraints";
    case "finiteSetWideningComposite":
      return "large finite set widened to preserved edge and character constraints";
    case "prefixJoinLcp":
      return "branch merge retained the meaningful longest common prefix";
    case "suffixJoinLcs":
      return "branch merge retained the meaningful longest common suffix";
    case "prefixSuffixJoin":
    case "charInclusionJoin":
    case "compositeJoin":
      return "reduced product combined compatible constraints from multiple domains";
    case "concatKnownEdges":
    case "finiteSetConcatPrefixLcp":
    case "finiteSetConcatSuffixProduct":
    case "charInclusionConcat":
    case "compositeConcat":
      return "reduced product concatenated compatible constraints without widening to top";
    case "concatUnknownLeft":
    case "concatUnknownRight":
      return "known constraints were preserved while concatenating an unknown edge";
    case "unconstrainedInput":
      return "the producing input did not provide a finite class-value constraint";
    case "automatonStateLimit":
      return "the finite language exceeded the bounded automaton state limit";
    case "flowIterationLimit":
      return "the class-value flow did not converge within its iteration limit";
    case "missingFlowPredecessor":
      return "a referenced flow predecessor was unavailable";
    case "joinUnrepresentable":
      return "the joined class-value constraints had no sound bounded representation";
    case "concatenationUnrepresentable":
      return "the concatenated class-value constraints had no sound bounded representation";
    case "reducedProductUnconstrained":
      return "the reduced product retained no constraining axis";
    case undefined:
      switch (abstractValue.kind) {
        case "bottom":
          return "no class value can satisfy the current constraints";
        case "exact":
          return "the class value is known exactly";
        case "finiteSet":
          return "the class value is one of a bounded set";
        case "prefix":
        case "suffix":
        case "prefixSuffix":
        case "charInclusion":
        case "composite":
          return "the class value is represented by explicit domain constraints";
        case "top":
          return "the class value is unconstrained";
        default:
          abstractValue satisfies never;
          return "the class value provenance is unknown";
      }
    default:
      return "the class value is represented by explicit domain constraints";
  }
}

function provenanceTreeRootDetail(
  abstractValue: FlowResolution["abstractValue"],
): string | undefined {
  switch (abstractValue.kind) {
    case "exact":
      return `value=${abstractValue.value}`;
    case "finiteSet":
      return `valueCount=${abstractValue.values.length}`;
    case "bottom":
    case "prefix":
    case "suffix":
    case "prefixSuffix":
    case "charInclusion":
    case "composite":
    case "top":
      return undefined;
    default:
      abstractValue satisfies never;
      return undefined;
  }
}

function provenanceTreeConstraintChildren(
  abstractValue: FlowResolution["abstractValue"],
): readonly ValueDomainProvenanceNode[] {
  const children: ValueDomainProvenanceNode[] = [];
  switch (abstractValue.kind) {
    case "prefix":
      children.push(provenanceConstraintNode("prefixConstraint", "prefix", abstractValue.prefix));
      break;
    case "suffix":
      children.push(provenanceConstraintNode("suffixConstraint", "suffix", abstractValue.suffix));
      break;
    case "prefixSuffix":
      children.push(provenanceConstraintNode("prefixConstraint", "prefix", abstractValue.prefix));
      children.push(provenanceConstraintNode("suffixConstraint", "suffix", abstractValue.suffix));
      children.push(
        provenanceConstraintNode("lengthConstraint", "minLength", String(abstractValue.minLength)),
      );
      break;
    case "charInclusion":
      pushCharConstraintChildren(
        children,
        abstractValue.mustChars,
        abstractValue.mayChars,
        Boolean(abstractValue.mayIncludeOtherChars),
      );
      break;
    case "composite":
      if (abstractValue.prefix) {
        children.push(provenanceConstraintNode("prefixConstraint", "prefix", abstractValue.prefix));
      }
      if (abstractValue.suffix) {
        children.push(provenanceConstraintNode("suffixConstraint", "suffix", abstractValue.suffix));
      }
      if (abstractValue.minLength !== undefined) {
        children.push(
          provenanceConstraintNode(
            "lengthConstraint",
            "minLength",
            String(abstractValue.minLength),
          ),
        );
      }
      pushCharConstraintChildren(
        children,
        abstractValue.mustChars,
        abstractValue.mayChars,
        Boolean(abstractValue.mayIncludeOtherChars),
      );
      break;
    case "bottom":
    case "exact":
    case "finiteSet":
    case "top":
      break;
    default:
      abstractValue satisfies never;
  }
  return children;
}

function pushCharConstraintChildren(
  children: ValueDomainProvenanceNode[],
  mustChars: string,
  mayChars: string,
  mayIncludeOtherChars: boolean,
) {
  if (mustChars.length > 0) {
    children.push(provenanceConstraintNode("characterMustConstraint", "mustChars", mustChars));
  }
  if (!mayIncludeOtherChars) {
    children.push(provenanceConstraintNode("characterMayConstraint", "mayChars", mayChars));
  }
}

function provenanceConstraintNode(
  operation: string,
  label: string,
  value: string,
): ValueDomainProvenanceNode {
  return {
    operation,
    resultKind: "constraint",
    detail: `${label}=${value}`,
    reason: "constraint retained by the abstract value domain",
    children: [],
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

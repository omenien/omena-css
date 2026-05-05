import type { ClassExpressionHIR } from "../hir/source-types";
import type { FlowResolution } from "../flow/lattice";
import {
  deriveSelectorCertaintyProfile,
  deriveValueCertaintyProfile,
  type EdgeCertainty,
} from "../semantic/certainty";
import type { ExpressionSemanticsSummary } from "./read-expression-semantics";
import type { InvalidClassReferenceFinding } from "./find-invalid-class-references";

type PrefixProvenance = Extract<FlowResolution["abstractValue"], { kind: "prefix" }>["provenance"];
type SuffixProvenance = Extract<FlowResolution["abstractValue"], { kind: "suffix" }>["provenance"];
type PrefixSuffixProvenance = Extract<
  FlowResolution["abstractValue"],
  { kind: "prefixSuffix" }
>["provenance"];
type CharInclusionProvenance = Extract<
  FlowResolution["abstractValue"],
  { kind: "charInclusion" }
>["provenance"];
type CompositeProvenance = Extract<
  FlowResolution["abstractValue"],
  { kind: "composite" }
>["provenance"];

export interface DynamicExpressionExplanation {
  readonly kind: "symbolRef" | "template";
  readonly subject: string;
  readonly candidates: readonly string[];
  readonly valueDomainLabel?: string;
  readonly valueDomainReasonLabel?: string;
  readonly valueDomainDerivationLabel?: string;
  readonly valueDomainDerivationStepLabels?: readonly string[];
  readonly valueDomainProvenanceLabel?: string;
  readonly valueDomainProvenanceStepLabels?: readonly string[];
  readonly valueCertainty?: EdgeCertainty;
  readonly valueCertaintyShapeLabel?: string;
  readonly valueCertaintyReasonLabel?: string;
  readonly selectorCertainty?: EdgeCertainty;
  readonly selectorCertaintyShapeLabel?: string;
  readonly selectorCertaintyReasonLabel?: string;
  readonly reasonLabel?: string;
}

export interface InvalidClassAnalysisMetadata {
  readonly analysisReason?: string;
  readonly valueCertaintyShapeLabel?: string;
  readonly valueDomainDerivationLabel?: string;
  readonly valueDomainDerivationStepLabels?: readonly string[];
}

export function buildDynamicExpressionExplanation(
  expression: ClassExpressionHIR,
  semantics: ExpressionSemanticsSummary,
): DynamicExpressionExplanation | null {
  switch (expression.kind) {
    case "symbolRef": {
      if (!semantics.abstractValue || !semantics.reason) return null;
      const valueDomainLabel = describeAbstractValue(semantics.abstractValue);
      const valueDomainReasonLabel = describeAbstractValueReason(semantics.abstractValue);
      const valueDomainDerivation = describeValueDomainDerivation(semantics.valueDomainDerivation);
      const valueDomainProvenance = describeValueDomainProvenanceTree(
        semantics.valueDomainProvenanceTree,
      );
      const valueCertaintyProfile = deriveValueCertaintyProfile(
        semantics.abstractValue,
        semantics.valueCertainty,
      );
      const valueCertaintyReasonLabel = describeValueCertaintyReason(
        semantics.abstractValue,
        semantics.valueCertainty,
        semantics.reason,
      );
      const selectorCertaintyProfile = deriveSelectorCertaintyProfile(
        semantics.selectors.length,
        semantics.selectorCertainty,
        semantics.abstractValue,
      );
      const selectorCertaintyReasonLabel = describeSelectorCertaintyReason(
        semantics.abstractValue,
        semantics.selectorCertainty,
        semantics.selectors.length,
      );
      const reasonLabel = describeResolutionReason(semantics.reason);
      return {
        kind: "symbolRef",
        subject: expression.rawReference,
        candidates: semantics.candidateNames,
        ...(valueDomainLabel ? { valueDomainLabel } : {}),
        ...(valueDomainReasonLabel ? { valueDomainReasonLabel } : {}),
        ...(valueDomainDerivation
          ? {
              valueDomainDerivationLabel: valueDomainDerivation.label,
              valueDomainDerivationStepLabels: valueDomainDerivation.stepLabels,
            }
          : {}),
        ...(valueDomainProvenance
          ? {
              valueDomainProvenanceLabel: valueDomainProvenance.label,
              valueDomainProvenanceStepLabels: valueDomainProvenance.stepLabels,
            }
          : {}),
        ...(semantics.valueCertainty ? { valueCertainty: semantics.valueCertainty } : {}),
        ...(valueCertaintyProfile
          ? { valueCertaintyShapeLabel: valueCertaintyProfile.shapeLabel }
          : {}),
        ...(valueCertaintyReasonLabel ? { valueCertaintyReasonLabel } : {}),
        ...(semantics.selectorCertainty ? { selectorCertainty: semantics.selectorCertainty } : {}),
        ...(selectorCertaintyProfile
          ? { selectorCertaintyShapeLabel: selectorCertaintyProfile.shapeLabel }
          : {}),
        ...(selectorCertaintyReasonLabel ? { selectorCertaintyReasonLabel } : {}),
        ...(reasonLabel ? { reasonLabel } : {}),
      };
    }
    case "template": {
      if (semantics.selectors.length === 0) return null;
      const valueDomainLabel = describeAbstractValue(semantics.abstractValue);
      const valueDomainReasonLabel = describeAbstractValueReason(semantics.abstractValue);
      const valueDomainDerivation = describeValueDomainDerivation(semantics.valueDomainDerivation);
      const valueDomainProvenance = describeValueDomainProvenanceTree(
        semantics.valueDomainProvenanceTree,
      );
      const selectorCertaintyProfile = deriveSelectorCertaintyProfile(
        semantics.selectors.length,
        semantics.selectorCertainty,
        semantics.abstractValue,
      );
      const selectorCertaintyReasonLabel = describeSelectorCertaintyReason(
        semantics.abstractValue,
        semantics.selectorCertainty,
        semantics.selectors.length,
      );
      return {
        kind: "template",
        subject: expression.staticPrefix,
        candidates: semantics.candidateNames,
        ...(valueDomainLabel ? { valueDomainLabel } : {}),
        ...(valueDomainReasonLabel ? { valueDomainReasonLabel } : {}),
        ...(valueDomainDerivation
          ? {
              valueDomainDerivationLabel: valueDomainDerivation.label,
              valueDomainDerivationStepLabels: valueDomainDerivation.stepLabels,
            }
          : {}),
        ...(valueDomainProvenance
          ? {
              valueDomainProvenanceLabel: valueDomainProvenance.label,
              valueDomainProvenanceStepLabels: valueDomainProvenance.stepLabels,
            }
          : {}),
        ...(semantics.selectorCertainty ? { selectorCertainty: semantics.selectorCertainty } : {}),
        ...(selectorCertaintyProfile
          ? { selectorCertaintyShapeLabel: selectorCertaintyProfile.shapeLabel }
          : {}),
        ...(selectorCertaintyReasonLabel ? { selectorCertaintyReasonLabel } : {}),
      };
    }
    case "literal":
    case "styleAccess":
      return null;
    default:
      expression satisfies never;
      return null;
  }
}

export function messageForInvalidClassFinding(finding: InvalidClassReferenceFinding): string {
  switch (finding.kind) {
    case "missingStaticClass":
    case "missingTemplatePrefix":
      return "";
    case "missingResolvedClassValues":
      if (finding.reason === "typeUnion") {
        return withAnalysisReason(
          `Missing class for union member${finding.missingValues.length > 1 ? "s" : ""}: ${finding.missingValues.map((value) => `'${value}'`).join(", ")}.`,
          finding,
        );
      }
      if (finding.missingValues.length === 1 && finding.valueCertainty === "exact") {
        return withAnalysisReason(
          `Missing class for resolved value: '${finding.missingValues[0]}'.`,
          finding,
        );
      }
      return withAnalysisReason(
        `Missing class for possible value${finding.missingValues.length > 1 ? "s" : ""}: ${finding.missingValues.map((value) => `'${value}'`).join(", ")}.`,
        finding,
      );
    case "missingResolvedClassDomain":
      switch (finding.abstractValue.kind) {
        case "prefix":
          return withAnalysisReason(
            `No class matched resolved prefix '${finding.abstractValue.prefix}'.`,
            finding,
          );
        case "suffix":
          return withAnalysisReason(
            `No class matched resolved suffix '${finding.abstractValue.suffix}'.`,
            finding,
          );
        case "prefixSuffix":
          return withAnalysisReason(
            `No class matched resolved prefix '${finding.abstractValue.prefix}' and suffix '${finding.abstractValue.suffix}'.`,
            finding,
          );
        case "charInclusion":
          return withAnalysisReason(
            `No class matched resolved character constraints ${formatCharacterConstraints(finding.abstractValue)}.`,
            finding,
          );
        case "composite":
          return withAnalysisReason(
            `No class matched resolved composite constraints ${formatCompositeConstraints(finding.abstractValue)}.`,
            finding,
          );
        case "top":
          return withAnalysisReason(
            "Dynamic class value could not be matched to any known selector.",
            finding,
          );
        default:
          return withAnalysisReason(
            "Resolved dynamic class domain did not match any known selector.",
            finding,
          );
      }
    default:
      finding satisfies never;
      return "";
  }
}

function formatCharacterConstraints(
  value: Extract<FlowResolution["abstractValue"], { kind: "charInclusion" }>,
): string {
  const parts = [
    value.mustChars ? `must include '${value.mustChars}'` : null,
    value.mayIncludeOtherChars ? null : `may only include '${value.mayChars}'`,
  ].filter((part): part is string => Boolean(part));
  return parts.length > 0 ? `(${parts.join(", ")})` : "(unbounded)";
}

function formatCompositeConstraints(
  value: Extract<FlowResolution["abstractValue"], { kind: "composite" }>,
): string {
  const parts = [
    value.prefix ? `prefix '${value.prefix}'` : null,
    value.suffix ? `suffix '${value.suffix}'` : null,
    value.minLength !== undefined ? `min length ${value.minLength}` : null,
    value.mustChars ? `must include '${value.mustChars}'` : null,
    value.mayIncludeOtherChars ? null : `may only include '${value.mayChars}'`,
  ].filter((part): part is string => Boolean(part));
  return parts.length > 0 ? `(${parts.join(", ")})` : "(unbounded)";
}

function withAnalysisReason(
  message: string,
  finding: Extract<
    InvalidClassReferenceFinding,
    { kind: "missingResolvedClassValues" | "missingResolvedClassDomain" }
  >,
): string {
  const metadata = buildInvalidClassAnalysisMetadata(
    finding.abstractValue,
    finding.valueCertainty,
    finding.reason,
  );
  const parts = [message];
  if (metadata.analysisReason) {
    parts.push(`Analysis reason: ${metadata.analysisReason}.`);
  }
  if (metadata.valueCertaintyShapeLabel) {
    parts.push(`Analysis shape: ${metadata.valueCertaintyShapeLabel}.`);
  }
  return parts.join(" ");
}

export function buildInvalidClassAnalysisMetadata(
  abstractValue: FlowResolution["abstractValue"] | undefined,
  valueCertainty: EdgeCertainty | undefined,
  reason: FlowResolution["reason"] | undefined,
  valueDomainDerivation?: ExpressionSemanticsSummary["valueDomainDerivation"],
): InvalidClassAnalysisMetadata {
  const reasons = [
    describeValueCertaintyReason(abstractValue, valueCertainty, reason),
    describeAbstractValueReason(abstractValue),
  ].filter((candidate): candidate is string => Boolean(candidate));
  const uniqueReasons = Array.from(new Set(reasons));
  const valueCertaintyProfile = deriveValueCertaintyProfile(abstractValue, valueCertainty);
  const derivation = describeValueDomainDerivation(valueDomainDerivation);
  return {
    ...(uniqueReasons.length > 0 ? { analysisReason: uniqueReasons.join("; ") } : {}),
    ...(valueCertaintyProfile
      ? { valueCertaintyShapeLabel: valueCertaintyProfile.shapeLabel }
      : {}),
    ...(derivation
      ? {
          valueDomainDerivationLabel: derivation.label,
          valueDomainDerivationStepLabels: derivation.stepLabels,
        }
      : {}),
  };
}

export function describeAbstractValue(
  abstractValue?: FlowResolution["abstractValue"],
): string | null {
  if (!abstractValue) return null;

  switch (abstractValue.kind) {
    case "bottom":
      return "empty";
    case "exact":
      return `exact \`${abstractValue.value}\``;
    case "finiteSet":
      return abstractValue.values.length > 1 ? `finite set (${abstractValue.values.length})` : null;
    case "prefix":
      return isWidenedPrefix(abstractValue.provenance)
        ? `prefix \`${abstractValue.prefix}\` (widened)`
        : `prefix \`${abstractValue.prefix}\``;
    case "suffix":
      return `suffix \`${abstractValue.suffix}\``;
    case "prefixSuffix":
      return `prefix \`${abstractValue.prefix}\` + suffix \`${abstractValue.suffix}\``;
    case "charInclusion":
      return abstractValue.mayIncludeOtherChars
        ? `character inclusion (must: \`${abstractValue.mustChars || "none"}\`)`
        : `character inclusion (must: \`${abstractValue.mustChars || "none"}\`, may: \`${abstractValue.mayChars}\`)`;
    case "composite":
      return [
        abstractValue.prefix ? `prefix \`${abstractValue.prefix}\`` : null,
        abstractValue.suffix ? `suffix \`${abstractValue.suffix}\`` : null,
        `character inclusion (must: \`${abstractValue.mustChars || "none"}\`)`,
      ]
        .filter((part): part is string => Boolean(part))
        .join(" + ");
    case "top":
      return "unknown";
    default:
      abstractValue satisfies never;
      return null;
  }
}

export function describeAbstractValueReason(
  abstractValue?: FlowResolution["abstractValue"],
): string | null {
  if (!abstractValue) return null;

  switch (abstractValue.kind) {
    case "prefix": {
      switch (abstractValue.provenance) {
        case "concatUnknownRight":
          return "known prefix preserved while concatenating an unknown suffix";
        case "prefixJoinLcp":
          return "branched prefixes merged at their longest common prefix";
        case "finiteSetWidening":
          return "finite candidates widened to a shared prefix";
        case "finiteSetConcatPrefixLcp":
          return "finite candidates concatenated with a prefix and reduced to their shared prefix";
        case undefined:
          return null;
      }
      return null;
    }
    case "suffix":
      return describeSuffixReason(abstractValue.provenance);
    case "prefixSuffix":
      return describePrefixSuffixReason(abstractValue.provenance);
    case "charInclusion":
      return describeCharInclusionReason(abstractValue.provenance);
    case "composite":
      return describeCompositeReason(abstractValue.provenance);
    default:
      return null;
  }
}

function describeValueDomainDerivation(
  derivation: ExpressionSemanticsSummary["valueDomainDerivation"],
): { readonly label: string; readonly stepLabels: readonly string[] } | null {
  if (!derivation) return null;

  const inputKind = derivation.inputConstraintKind
    ? `${derivation.inputFactKind}/${derivation.inputConstraintKind}`
    : derivation.inputFactKind;
  const operationPath = derivation.steps.map((step) => step.operation).join(" -> ");
  return {
    label: `${inputKind} reduced to ${derivation.reducedKind}${operationPath ? ` via ${operationPath}` : ""}`,
    stepLabels: derivation.steps.map((step, index) => {
      const refinement = step.refinementKind ? ` + ${step.refinementKind}` : "";
      const input = step.inputKind ? `${step.inputKind}${refinement}` : "input facts";
      const result = step.resultProvenance
        ? `${step.resultKind}/${step.resultProvenance}`
        : step.resultKind;
      return `${index + 1}. ${step.operation}: ${input} -> ${result} (${step.reason})`;
    }),
  };
}

function describeValueDomainProvenanceTree(
  tree: ExpressionSemanticsSummary["valueDomainProvenanceTree"],
): { readonly label: string; readonly stepLabels: readonly string[] } | null {
  if (!tree) return null;

  const result = tree.root.resultProvenance
    ? `${tree.root.resultKind}/${tree.root.resultProvenance}`
    : tree.root.resultKind;
  const stepLabels = tree.root.children.map((child, index) => {
    const detail = child.detail ? `${child.operation}: ${child.detail}` : child.operation;
    return `${index + 1}. ${detail} (${child.reason})`;
  });
  return {
    label: `${tree.root.operation}: ${result} (${tree.root.reason})`,
    stepLabels,
  };
}

export function describeValueCertaintyReason(
  abstractValue: FlowResolution["abstractValue"] | undefined,
  valueCertainty: EdgeCertainty | undefined,
  reason: FlowResolution["reason"] | undefined,
): string | null {
  if (!abstractValue || !valueCertainty) return null;

  switch (valueCertainty) {
    case "exact":
      return null;
    case "inferred":
      switch (abstractValue.kind) {
        case "finiteSet":
          return reason === "typeUnion"
            ? "TypeScript exposed multiple string-literal candidates"
            : "analysis preserved multiple finite candidate values";
        case "prefix":
          return (
            describeAbstractValueReason(abstractValue) ??
            "analysis preserved only a constrained prefix of the runtime value"
          );
        case "suffix":
          return (
            describeAbstractValueReason(abstractValue) ??
            "analysis preserved only a constrained suffix of the runtime value"
          );
        case "prefixSuffix":
          return (
            describeAbstractValueReason(abstractValue) ??
            "analysis preserved both a constrained prefix and suffix of the runtime value"
          );
        case "charInclusion":
          return (
            describeAbstractValueReason(abstractValue) ??
            "analysis preserved character inclusion constraints of the runtime value"
          );
        case "composite":
          return (
            describeAbstractValueReason(abstractValue) ??
            "analysis preserved multiple orthogonal runtime string constraints"
          );
        case "exact":
        case "bottom":
        case "top":
          return null;
        default:
          abstractValue satisfies never;
          return null;
      }
    case "possible":
      return abstractValue.kind === "top"
        ? "analysis lost finite shape information for this value"
        : "analysis could not narrow this value to a finite selector set";
    default:
      valueCertainty satisfies never;
      return null;
  }
}

export function describeSelectorCertaintyReason(
  abstractValue: FlowResolution["abstractValue"] | undefined,
  selectorCertainty: EdgeCertainty | undefined,
  matchedSelectorCount: number,
): string | null {
  if (!selectorCertainty) return null;

  switch (selectorCertainty) {
    case "exact":
      return null;
    case "inferred":
      if (
        abstractValue?.kind === "prefix" ||
        abstractValue?.kind === "suffix" ||
        abstractValue?.kind === "prefixSuffix" ||
        abstractValue?.kind === "charInclusion" ||
        abstractValue?.kind === "composite"
      ) {
        return (
          describeAbstractValueReason(abstractValue) ??
          "constrained runtime shape matched a bounded selector set"
        );
      }
      return "finite candidate values matched a bounded selector set";
    case "possible":
      return matchedSelectorCount === 0
        ? "no selector could be proven for this value"
        : "analysis could not prove an exact selector set";
    default:
      selectorCertainty satisfies never;
      return null;
  }
}

function isWidenedPrefix(provenance: PrefixProvenance): boolean {
  return provenance === "finiteSetWidening" || provenance === "finiteSetConcatPrefixLcp";
}

function describeSuffixReason(provenance: SuffixProvenance): string | null {
  switch (provenance) {
    case "concatUnknownLeft":
      return "known suffix preserved while prepending an unknown prefix";
    case "suffixJoinLcs":
      return "branched suffixes merged at their longest common suffix";
    case undefined:
      return null;
    default:
      provenance satisfies never;
      return null;
  }
}

function describePrefixSuffixReason(provenance: PrefixSuffixProvenance): string | null {
  switch (provenance) {
    case "concatKnownEdges":
      return "known prefix and suffix were preserved across concatenation";
    case "prefixFiniteSetSharedSuffix":
      return "known prefix combined with finite candidates that shared a stable suffix";
    case "finiteSetConcatSuffixProduct":
      return "finite candidates preserved a shared prefix while concatenating a known suffix";
    case "prefixSuffixJoin":
      return "branched values merged to a shared prefix and suffix";
    case undefined:
      return null;
    default:
      provenance satisfies never;
      return null;
  }
}

function describeCharInclusionReason(provenance: CharInclusionProvenance): string | null {
  switch (provenance) {
    case "finiteSetWideningChars":
      return "finite candidates widened to shared character inclusion constraints";
    case "charInclusionJoin":
      return "branched values merged to shared character inclusion constraints";
    case "charInclusionConcat":
      return "concatenation preserved character inclusion constraints";
    case "concatUnknownLeft":
      return "known required characters were preserved while prepending an unknown prefix";
    case "concatUnknownRight":
      return "known required characters were preserved while concatenating an unknown suffix";
    case undefined:
      return null;
    default:
      provenance satisfies never;
      return null;
  }
}

function describeCompositeReason(provenance: CompositeProvenance): string | null {
  switch (provenance) {
    case "finiteSetWideningComposite":
      return "finite candidates widened to shared edge and character constraints";
    case "compositeJoin":
      return "branched values merged to shared multi-axis string constraints";
    case "compositeConcat":
      return "concatenation preserved edge and character constraints together";
    case undefined:
      return null;
    default:
      provenance satisfies never;
      return null;
  }
}

export function describeResolutionReason(reason?: FlowResolution["reason"]): string | null {
  if (!reason) return null;
  switch (reason) {
    case "flowLiteral":
      return "local flow analysis";
    case "flowBranch":
      return "branched local flow analysis";
    case "typeUnion":
      return "TypeScript string-literal union analysis";
    default:
      reason satisfies never;
      return null;
  }
}

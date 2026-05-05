import type { TextRewritePlan } from "../core/rewrite/text-rewrite-plan";
import type { EngineWorkspaceV1, SourceAnalysisInputV1, StyleAnalysisInputV1 } from "./engine-v1";
import type { CheckerReportV1 } from "./checker-v1";

export const ENGINE_CONTRACT_VERSION_V2 = "2" as const;

export type StringTypeFactKindV2 = "unknown" | "exact" | "finiteSet" | "constrained" | "top";

export type StringConstraintKindV2 =
  | "prefix"
  | "suffix"
  | "prefixSuffix"
  | "charInclusion"
  | "composite";

export interface StringTypeFactsV2 {
  readonly kind: StringTypeFactKindV2;
  readonly values?: readonly string[];
  readonly constraintKind?: StringConstraintKindV2;
  readonly prefix?: string;
  readonly suffix?: string;
  readonly minLen?: number;
  readonly maxLen?: number;
  readonly charMust?: string;
  readonly charMay?: string;
  readonly mayIncludeOtherChars?: boolean;
  readonly provenance?: string;
}

export interface TypeFactTableEntryV2 {
  readonly filePath: string;
  readonly expressionId: string;
  readonly facts: StringTypeFactsV2;
}

export type TypeFactTableV2 = readonly TypeFactTableEntryV2[];

export interface EngineInputV2 {
  readonly version: typeof ENGINE_CONTRACT_VERSION_V2;
  readonly workspace: EngineWorkspaceV1;
  readonly sources: readonly SourceAnalysisInputV1[];
  readonly styles: readonly StyleAnalysisInputV1[];
  readonly typeFacts: TypeFactTableV2;
}

export type ValueDomainKindV2 = "none" | "exact" | "finiteSet" | "constrained" | "top";
export type ValueCertaintyShapeKindV2 = "exact" | "boundedFinite" | "constrained" | "unknown";
export type SelectorCertaintyShapeKindV2 = "exact" | "boundedFinite" | "constrained" | "unknown";
export type QueryResultKindV2 =
  | "expression-semantics"
  | "selector-usage"
  | "source-expression-resolution";

export interface ValueDomainDerivationStepV2 {
  readonly operation: string;
  readonly inputKind?: string;
  readonly refinementKind?: string;
  readonly resultKind: string;
  readonly resultProvenance?: string;
  readonly reason: string;
}

export interface ValueDomainDerivationV2 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly inputFactKind: string;
  readonly inputConstraintKind?: string;
  readonly inputValueCount: number;
  readonly reducedKind: string;
  readonly steps: readonly ValueDomainDerivationStepV2[];
}

export interface ValueDomainProvenanceNodeV2 {
  readonly operation: string;
  readonly resultKind: string;
  readonly resultProvenance?: string;
  readonly detail?: string;
  readonly reason: string;
  readonly children: readonly ValueDomainProvenanceNodeV2[];
}

export interface ValueDomainProvenanceTreeV2 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly valueKind: string;
  readonly value: unknown;
  readonly valueProvenance?: string;
  readonly root: ValueDomainProvenanceNodeV2;
}

export interface ExpressionSemanticsQueryResultV2 {
  readonly kind: "expression-semantics";
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: {
    readonly expressionId: string;
    readonly expressionKind: string;
    readonly styleFilePath: string | null;
    readonly selectorNames: readonly string[];
    readonly candidateNames: readonly string[];
    readonly finiteValues: readonly string[] | null;
    readonly valueDomainKind: ValueDomainKindV2;
    readonly valueConstraintKind?: StringConstraintKindV2;
    readonly valuePrefix?: string;
    readonly valueSuffix?: string;
    readonly valueMinLen?: number;
    readonly valueMaxLen?: number;
    readonly valueCharMust?: string;
    readonly valueCharMay?: string;
    readonly valueMayIncludeOtherChars?: boolean;
    readonly valueDomainReason?: string;
    readonly valueDomainDerivation?: ValueDomainDerivationV2;
    readonly valueDomainProvenanceTree?: ValueDomainProvenanceTreeV2;
    readonly selectorCertainty: string;
    readonly selectorCertaintyShapeKind?: SelectorCertaintyShapeKindV2;
    readonly selectorConstraintKind?: StringConstraintKindV2;
    readonly selectorCertaintyShapeLabel?: string;
    readonly selectorCertaintyReason?: string;
    readonly valueCertainty?: string;
    readonly valueCertaintyShapeKind?: ValueCertaintyShapeKindV2;
    readonly valueCertaintyConstraintKind?: StringConstraintKindV2;
    readonly valueCertaintyShapeLabel?: string;
    readonly valueCertaintyReason?: string;
    readonly reason?: string;
  };
}

export interface SourceExpressionResolutionQueryResultV2 {
  readonly kind: "source-expression-resolution";
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: {
    readonly expressionId: string;
    readonly styleFilePath: string | null;
    readonly selectorNames: readonly string[];
    readonly finiteValues: readonly string[] | null;
    readonly selectorCertainty: string;
    readonly selectorCertaintyShapeKind?: SelectorCertaintyShapeKindV2;
    readonly selectorConstraintKind?: StringConstraintKindV2;
    readonly selectorCertaintyShapeLabel?: string;
    readonly selectorCertaintyReason?: string;
    readonly valueCertainty?: string;
    readonly valueCertaintyShapeKind?: ValueCertaintyShapeKindV2;
    readonly valueCertaintyConstraintKind?: StringConstraintKindV2;
    readonly valuePrefix?: string;
    readonly valueSuffix?: string;
    readonly valueMinLen?: number;
    readonly valueMaxLen?: number;
    readonly valueCharMust?: string;
    readonly valueCharMay?: string;
    readonly valueMayIncludeOtherChars?: boolean;
    readonly valueCertaintyShapeLabel?: string;
    readonly valueCertaintyReason?: string;
    readonly reason?: string;
  };
}

export interface SelectorUsageQueryResultV2 {
  readonly kind: "selector-usage";
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: {
    readonly canonicalName: string;
    readonly totalReferences: number;
    readonly directReferenceCount: number;
    readonly editableDirectReferenceCount: number;
    readonly exactReferenceCount: number;
    readonly inferredOrBetterReferenceCount: number;
    readonly hasExpandedReferences: boolean;
    readonly hasStyleDependencyReferences: boolean;
    readonly hasAnyReferences: boolean;
  };
}

export type QueryResultV2 =
  | ExpressionSemanticsQueryResultV2
  | SourceExpressionResolutionQueryResultV2
  | SelectorUsageQueryResultV2;

export interface EngineOutputV2 {
  readonly version: typeof ENGINE_CONTRACT_VERSION_V2;
  readonly queryResults: readonly QueryResultV2[];
  readonly rewritePlans: readonly TextRewritePlan<unknown>[];
  readonly checkerReport: CheckerReportV1;
}

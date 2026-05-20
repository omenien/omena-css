import { spawn } from "node:child_process";
import path from "node:path";
import type { EngineParitySnapshotV2 } from "../server/engine-host-node/src/engine-parity-v2";
import type { EngineInputV2, QueryResultV2 } from "../server/engine-core-ts/src/contracts";

const REPO_ROOT = process.cwd();
const RUST_MANIFEST = path.join(REPO_ROOT, "rust/Cargo.toml");

export interface ShadowSummaryV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly sourceCount: number;
  readonly styleCount: number;
  readonly typeFactCount: number;
  readonly distinctFactFiles: number;
  readonly byKind: Readonly<Record<string, number>>;
  readonly constrainedKinds: Readonly<Record<string, number>>;
  readonly finiteValueCount: number;
  readonly queryResultCount: number;
  readonly queryKindCounts: Readonly<Record<string, number>>;
  readonly expressionValueDomainKinds: Readonly<Record<string, number>>;
  readonly expressionValueConstraintKinds: Readonly<Record<string, number>>;
  readonly expressionConstraintDetailCounts: ConstraintDetailCounts;
  readonly expressionValueCertaintyShapes: Readonly<Record<string, number>>;
  readonly expressionSelectorCertaintyShapes: Readonly<Record<string, number>>;
  readonly resolutionValueConstraintKinds: Readonly<Record<string, number>>;
  readonly resolutionConstraintDetailCounts: ConstraintDetailCounts;
  readonly resolutionValueCertaintyShapes: Readonly<Record<string, number>>;
  readonly resolutionSelectorCertaintyShapes: Readonly<Record<string, number>>;
  readonly selectorUsageReferencedCount: number;
  readonly selectorUsageUnreferencedCount: number;
  readonly selectorUsageTotalReferences: number;
  readonly selectorUsageDirectReferences: number;
  readonly selectorUsageEditableDirectReferences: number;
  readonly selectorUsageExactReferences: number;
  readonly selectorUsageInferredOrBetterReferences: number;
  readonly selectorUsageExpandedCount: number;
  readonly selectorUsageStyleDependencyCount: number;
  readonly expectedExpressionSemanticsCount: number;
  readonly expectedSourceExpressionResolutionCount: number;
  readonly expectedSelectorUsageCount: number;
  readonly expectedTotalQueryCount: number;
  readonly matchedExpressionQueryPairs: number;
  readonly missingExpressionSemanticsCount: number;
  readonly missingSourceExpressionResolutionCount: number;
  readonly unexpectedExpressionSemanticsCount: number;
  readonly unexpectedSourceExpressionResolutionCount: number;
  readonly matchedSelectorUsageCount: number;
  readonly missingSelectorUsageCount: number;
  readonly unexpectedSelectorUsageCount: number;
  readonly rewritePlanCount: number;
  readonly checkerWarningCount: number;
  readonly checkerHintCount: number;
  readonly checkerTotalFindings: number;
}

export interface TypeFactInputSummaryV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly typeFactCount: number;
  readonly distinctFactFiles: number;
  readonly byKind: Readonly<Record<string, number>>;
  readonly constrainedKinds: Readonly<Record<string, number>>;
  readonly finiteValueCount: number;
}

export interface QueryPlanSummaryV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly expressionSemanticsIds: readonly string[];
  readonly sourceExpressionResolutionIds: readonly string[];
  readonly selectorUsageIds: readonly string[];
  readonly totalQueryCount: number;
}

export interface ExpressionDomainPlanSummaryV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly plannedExpressionIds: readonly string[];
  readonly valueDomainKinds: Readonly<Record<string, number>>;
  readonly valueConstraintKinds: Readonly<Record<string, number>>;
  readonly constraintDetailCounts: ConstraintDetailCounts;
  readonly finiteValueCount: number;
}

export interface ExpressionDomainFragmentV0 {
  readonly expressionId: string;
  readonly filePath: string;
  readonly valueDomainKind: string;
  readonly valueConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
  readonly finiteValueCount: number;
}

export interface ExpressionDomainFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly ExpressionDomainFragmentV0[];
}

export interface ExpressionDomainCandidateV0 {
  readonly expressionId: string;
  readonly filePath: string;
  readonly valueDomainKind: string;
  readonly valueConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
  readonly finiteValueCount: number;
}

export interface ExpressionDomainCandidatesV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly candidates: readonly ExpressionDomainCandidateV0[];
}

export interface ExpressionDomainCanonicalCandidateBundleV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly planSummary: ExpressionDomainPlanSummaryV0;
  readonly fragments: readonly ExpressionDomainFragmentV0[];
  readonly candidates: readonly ExpressionDomainCandidateV0[];
}

export interface ExpressionDomainEvaluatorCandidatePayloadV0 {
  readonly expressionId: string;
  readonly valueDomainKind: string;
  readonly valueConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
  readonly finiteValueCount: number;
  readonly valueDomainDerivation: ReducedClassValueDerivationV0;
  readonly valueDomainProvenanceTree: AbstractClassValueProvenanceTreeV0;
}

export interface ReducedClassValueDerivationV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly inputFactKind: string;
  readonly inputConstraintKind?: string;
  readonly inputValueCount: number;
  readonly reducedKind: string;
  readonly steps: readonly ReducedClassValueDerivationStepV0[];
}

export interface ReducedClassValueDerivationStepV0 {
  readonly operation: string;
  readonly inputKind?: string;
  readonly refinementKind?: string;
  readonly resultKind: string;
  readonly resultProvenance?: string;
  readonly reason: string;
}

export interface AbstractClassValueProvenanceTreeV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly valueKind: string;
  readonly value: unknown;
  readonly valueProvenance?: string;
  readonly root: AbstractClassValueProvenanceNodeV0;
}

export interface AbstractClassValueProvenanceNodeV0 {
  readonly operation: string;
  readonly resultKind: string;
  readonly resultProvenance?: string;
  readonly detail?: string;
  readonly reason: string;
  readonly children: readonly AbstractClassValueProvenanceNodeV0[];
}

export interface ExpressionDomainEvaluatorCandidateV0 {
  readonly kind: string;
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: ExpressionDomainEvaluatorCandidatePayloadV0;
}

export interface ExpressionDomainEvaluatorCandidatesV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly results: readonly ExpressionDomainEvaluatorCandidateV0[];
}

export interface ExpressionDomainCanonicalProducerSignalV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalBundle: ExpressionDomainCanonicalCandidateBundleV0;
  readonly evaluatorCandidates: ExpressionDomainEvaluatorCandidatesV0;
}

export interface ExpressionDomainFlowAnalysisV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly inputVersion: string;
  readonly analyses: readonly ExpressionDomainFlowAnalysisEntryV0[];
}

export interface ExpressionDomainFlowAnalysisEntryV0 {
  readonly graphId: string;
  readonly filePath: string;
  readonly analysis: ClassValueFlowAnalysisV0;
}

export interface ExpressionDomainCallSiteFlowAnalysisV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly inputVersion: string;
  readonly zeroCfa: KLimitedCallSiteFlowAnalysisV0;
  readonly oneCfa: KLimitedCallSiteFlowAnalysisV0;
}

export interface KLimitedCallSiteFlowAnalysisV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly contextSensitivity: string;
  readonly maxContextDepth: number;
  readonly callSiteCount: number;
  readonly calleeCount: number;
  readonly entries: readonly KLimitedCallSiteFlowEntryV0[];
}

export interface KLimitedCallSiteFlowEntryV0 {
  readonly calleeKey: string;
  readonly callSiteStack: readonly string[];
  readonly contextKey: string;
  readonly exitNodeId: string;
  readonly exitValueKind: string;
  readonly exitValue: AbstractClassValueV0;
  readonly analysis: ClassValueFlowAnalysisV0;
}

export interface ClassValueFlowAnalysisV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly contextSensitivity: string;
  readonly contextKey?: string;
  readonly converged: boolean;
  readonly iterationCount: number;
  readonly nodes: readonly ClassValueFlowNodeResultV0[];
}

export interface ClassValueFlowNodeResultV0 {
  readonly id: string;
  readonly predecessorIds: readonly string[];
  readonly transferKind: string;
  readonly valueKind: string;
  readonly value: AbstractClassValueV0;
}

export interface AbstractClassValueV0 {
  readonly kind: string;
  readonly [key: string]: unknown;
}

export interface SelectorUsagePlanSummaryV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalSelectorNames: readonly string[];
  readonly viewKindCounts: Readonly<Record<string, number>>;
  readonly nestedSafetyCounts: Readonly<Record<string, number>>;
  readonly composedSelectorCount: number;
  readonly totalComposesRefs: number;
}

export interface SelectorUsageFragmentV0 {
  readonly ordinal: number;
  readonly viewKind: string;
  readonly canonicalName?: string;
  readonly nestedSafety?: string;
  readonly composesCount: number;
}

export interface SelectorUsageFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly SelectorUsageFragmentV0[];
}

export interface SelectorUsageQueryFragmentV0 {
  readonly queryId: string;
  readonly canonicalName: string;
  readonly nestedSafety?: string;
  readonly composesCount: number;
}

export interface SelectorUsageQueryFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly SelectorUsageQueryFragmentV0[];
}

export interface SourceResolutionPlanSummaryV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly plannedExpressionIds: readonly string[];
  readonly expressionKindCounts: Readonly<Record<string, number>>;
  readonly distinctStyleFilePaths: readonly string[];
  readonly symbolRefWithBindingCount: number;
  readonly styleAccessCount: number;
  readonly styleAccessPathDepthSum: number;
}

export interface SourceResolutionQueryFragmentV0 {
  readonly queryId: string;
  readonly expressionId: string;
  readonly expressionKind: string;
  readonly styleFilePath: string;
}

export interface SourceResolutionQueryFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly SourceResolutionQueryFragmentV0[];
}

export interface SourceResolutionMatchFragmentV0 {
  readonly queryId: string;
  readonly expressionId: string;
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
  readonly finiteValues?: readonly string[];
}

export interface SourceResolutionMatchFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly SourceResolutionMatchFragmentV0[];
}

export interface SourceResolutionCandidateV0 {
  readonly queryId: string;
  readonly expressionId: string;
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
  readonly finiteValues?: readonly string[];
  readonly selectorCertainty: string;
  readonly valueCertainty?: string;
  readonly selectorCertaintyShapeKind: string;
  readonly selectorCertaintyShapeLabel: string;
  readonly selectorConstraintKind?: string;
  readonly valueCertaintyShapeKind: string;
  readonly valueCertaintyShapeLabel: string;
  readonly valueCertaintyConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
}

export interface SourceResolutionCandidatesV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly candidates: readonly SourceResolutionCandidateV0[];
}

export interface SourceResolutionCanonicalCandidateBundleV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly queryFragments: readonly SourceResolutionQueryFragmentV0[];
  readonly fragments: readonly SourceResolutionFragmentV0[];
  readonly matchFragments: readonly SourceResolutionMatchFragmentV0[];
  readonly candidates: readonly SourceResolutionCandidateV0[];
}

export interface SourceResolutionEvaluatorCandidatePayloadV0 {
  readonly expressionId: string;
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
  readonly finiteValues?: readonly string[];
  readonly selectorCertainty: string;
  readonly valueCertainty?: string;
  readonly selectorCertaintyShapeKind: string;
  readonly selectorCertaintyShapeLabel: string;
  readonly valueCertaintyShapeKind: string;
  readonly valueCertaintyShapeLabel: string;
  readonly selectorConstraintKind?: string;
  readonly valueCertaintyConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
}

export interface SourceResolutionEvaluatorCandidateV0 {
  readonly kind: "source-expression-resolution";
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: SourceResolutionEvaluatorCandidatePayloadV0;
}

export interface SourceResolutionEvaluatorCandidatesV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly results: readonly SourceResolutionEvaluatorCandidateV0[];
}

export interface SourceResolutionCanonicalProducerSignalV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalBundle: SourceResolutionCanonicalCandidateBundleV0;
  readonly evaluatorCandidates: SourceResolutionEvaluatorCandidatesV0;
}

export interface SourceSideCanonicalCandidateBundleV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly expressionSemantics: ExpressionSemanticsCanonicalCandidateBundleV0;
  readonly sourceResolution: SourceResolutionCanonicalCandidateBundleV0;
}

export interface SourceSideEvaluatorCandidatesV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly expressionSemantics: ExpressionSemanticsEvaluatorCandidatesV0;
  readonly sourceResolution: SourceResolutionEvaluatorCandidatesV0;
}

export interface SourceSideCanonicalProducerSignalV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalBundle: SourceSideCanonicalCandidateBundleV0;
  readonly evaluatorCandidates: SourceSideEvaluatorCandidatesV0;
}

export interface SemanticCanonicalCandidateBundleV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly sourceSide: SourceSideCanonicalCandidateBundleV0;
  readonly expressionDomain: ExpressionDomainCanonicalCandidateBundleV0;
}

export interface SemanticEvaluatorCandidatesV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly sourceSide: SourceSideEvaluatorCandidatesV0;
  readonly expressionDomain: ExpressionDomainEvaluatorCandidatesV0;
}

export interface SemanticCanonicalProducerSignalV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalBundle: SemanticCanonicalCandidateBundleV0;
  readonly evaluatorCandidates: SemanticEvaluatorCandidatesV0;
}

export interface ExpressionSemanticsFragmentV0 {
  readonly queryId: string;
  readonly expressionId: string;
  readonly expressionKind: string;
  readonly styleFilePath: string;
  readonly valueDomainKind: string;
  readonly valueConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
}

export interface ExpressionSemanticsFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly ExpressionSemanticsFragmentV0[];
}

export interface ExpressionSemanticsQueryFragmentV0 {
  readonly queryId: string;
  readonly expressionId: string;
  readonly expressionKind: string;
  readonly styleFilePath: string;
}

export interface ExpressionSemanticsQueryFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly ExpressionSemanticsQueryFragmentV0[];
}

export interface ExpressionSemanticsMatchFragmentV0 {
  readonly queryId: string;
  readonly expressionId: string;
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
  readonly candidateNames: readonly string[];
  readonly finiteValues?: readonly string[];
}

export interface ExpressionSemanticsMatchFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly ExpressionSemanticsMatchFragmentV0[];
}

export interface ExpressionSemanticsCandidateV0 {
  readonly queryId: string;
  readonly expressionId: string;
  readonly expressionKind: string;
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
  readonly candidateNames: readonly string[];
  readonly finiteValues?: readonly string[];
  readonly valueDomainKind: string;
  readonly selectorCertainty: string;
  readonly valueCertainty?: string;
  readonly selectorCertaintyShapeKind: string;
  readonly selectorCertaintyShapeLabel: string;
  readonly valueCertaintyShapeKind: string;
  readonly valueCertaintyShapeLabel: string;
  readonly selectorConstraintKind?: string;
  readonly valueCertaintyConstraintKind?: string;
  readonly valueConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
}

export interface ExpressionSemanticsCandidatesV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly candidates: readonly ExpressionSemanticsCandidateV0[];
}

export interface ExpressionSemanticsCanonicalCandidateBundleV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly queryFragments: readonly ExpressionSemanticsQueryFragmentV0[];
  readonly fragments: readonly ExpressionSemanticsFragmentV0[];
  readonly matchFragments: readonly ExpressionSemanticsMatchFragmentV0[];
  readonly candidates: readonly ExpressionSemanticsCandidateV0[];
}

export interface ExpressionSemanticsEvaluatorCandidatePayloadV0 {
  readonly expressionId: string;
  readonly expressionKind: string;
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
  readonly candidateNames: readonly string[];
  readonly finiteValues?: readonly string[];
  readonly valueDomainKind: string;
  readonly selectorCertainty: string;
  readonly valueCertainty?: string;
  readonly selectorCertaintyShapeKind: string;
  readonly selectorCertaintyShapeLabel: string;
  readonly valueCertaintyShapeKind: string;
  readonly valueCertaintyShapeLabel: string;
  readonly selectorConstraintKind?: string;
  readonly valueCertaintyConstraintKind?: string;
  readonly valueConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
  readonly valueDomainDerivation: ReducedClassValueDerivationV0;
  readonly valueDomainProvenanceTree: AbstractClassValueProvenanceTreeV0;
}

export interface ExpressionSemanticsEvaluatorCandidateV0 {
  readonly kind: "expression-semantics";
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: ExpressionSemanticsEvaluatorCandidatePayloadV0;
}

export interface ExpressionSemanticsEvaluatorCandidatesV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly results: readonly ExpressionSemanticsEvaluatorCandidateV0[];
}

export interface ExpressionSemanticsCanonicalProducerSignalV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalBundle: ExpressionSemanticsCanonicalCandidateBundleV0;
  readonly evaluatorCandidates: ExpressionSemanticsEvaluatorCandidatesV0;
}

export interface SourceResolutionFragmentV0 {
  readonly queryId: string;
  readonly expressionId: string;
  readonly styleFilePath: string;
  readonly valueCertaintyShapeKind: string;
  readonly valueCertaintyConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
}

export interface SourceResolutionFragmentsV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly fragments: readonly SourceResolutionFragmentV0[];
}

export interface CheckerStyleRecoveryFindingV0 {
  readonly filePath: string;
  readonly code: string;
  readonly severity: string;
  readonly range: {
    readonly start: {
      readonly line: number;
      readonly character: number;
    };
    readonly end: {
      readonly line: number;
      readonly character: number;
    };
  };
  readonly message: string;
  readonly analysisReason?: string;
  readonly valueCertaintyShapeLabel?: string;
}

export interface CheckerStyleRecoveryCanonicalCandidateBundleV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly reportVersion: string;
  readonly bundle: "style-recovery";
  readonly distinctFileCount: number;
  readonly codeCounts: Readonly<Record<string, number>>;
  readonly summary: {
    readonly warnings: number;
    readonly hints: number;
    readonly total: number;
  };
  readonly findings: readonly CheckerStyleRecoveryFindingV0[];
}

export interface CheckerStyleRecoveryCanonicalProducerSignalV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalCandidate: CheckerStyleRecoveryCanonicalCandidateBundleV0;
  readonly boundedCheckerGate: {
    readonly canonicalCandidateCommand: "pnpm check:rust-checker-style-recovery-canonical-candidate";
    readonly canonicalProducerCommand: "pnpm check:rust-checker-style-recovery-canonical-producer";
    readonly consumerBoundaryCommand: "pnpm check:rust-checker-style-recovery-consumer-boundary";
    readonly boundedCheckerLaneCommand: "pnpm check:rust-checker-bounded-lanes";
    readonly promotionReviewCommand: "pnpm check:rust-checker-promotion-review";
    readonly promotionEvidenceCommand: "pnpm check:rust-checker-promotion-evidence";
    readonly broaderRustLaneCommand: "pnpm check:rust-lane-bundle";
    readonly releaseGateReadinessCommand: "pnpm check:rust-checker-release-gate-readiness";
    readonly releaseGateShadowCommand: "pnpm check:rust-checker-release-gate-shadow";
    readonly releaseGateShadowReviewCommand: "pnpm check:rust-checker-release-gate-shadow-review";
    readonly releaseBundleCommand: "pnpm check:rust-release-bundle";
    readonly minimumBoundedLaneCountForRustLaneBundle: 3;
    readonly minimumBoundedLaneCountForRustReleaseBundle: 3;
    readonly minimumSuccessfulShadowRunsForRustReleaseBundle: 3;
    readonly checkerBundle: "style-recovery";
    readonly releaseGateStage: "enforced";
    readonly includedInRustLaneBundle: true;
    readonly includedInRustReleaseBundle: true;
  };
}

export interface CheckerSourceMissingFindingV0 {
  readonly filePath: string;
  readonly code: string;
  readonly severity: string;
  readonly range: {
    readonly start: {
      readonly line: number;
      readonly character: number;
    };
    readonly end: {
      readonly line: number;
      readonly character: number;
    };
  };
  readonly message: string;
  readonly analysisReason?: string;
  readonly valueCertaintyShapeLabel?: string;
  readonly valueDomainDerivation?: ReducedClassValueDerivationV0;
}

export interface CheckerSourceMissingCanonicalCandidateBundleV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly reportVersion: string;
  readonly bundle: "source-missing";
  readonly distinctFileCount: number;
  readonly codeCounts: Readonly<Record<string, number>>;
  readonly summary: {
    readonly warnings: number;
    readonly hints: number;
    readonly total: number;
  };
  readonly findings: readonly CheckerSourceMissingFindingV0[];
}

export interface CheckerSourceMissingCanonicalProducerSignalV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalCandidate: CheckerSourceMissingCanonicalCandidateBundleV0;
  readonly flowEvidence: {
    readonly schemaVersion: string;
    readonly product: string;
    readonly inputVersion: string;
    readonly graphCount: number;
    readonly nodeCount: number;
    readonly convergedGraphCount: number;
    readonly unconvergedGraphCount: number;
    readonly maxIterationCount: number;
  };
  readonly boundedCheckerGate: {
    readonly canonicalCandidateCommand: "pnpm check:rust-checker-source-missing-canonical-candidate";
    readonly canonicalProducerCommand: "pnpm check:rust-checker-source-missing-canonical-producer";
    readonly consumerBoundaryCommand: "pnpm check:rust-checker-source-missing-consumer-boundary";
    readonly boundedCheckerLaneCommand: "pnpm check:rust-checker-bounded-lanes";
    readonly promotionReviewCommand: "pnpm check:rust-checker-promotion-review";
    readonly promotionEvidenceCommand: "pnpm check:rust-checker-promotion-evidence";
    readonly broaderRustLaneCommand: "pnpm check:rust-lane-bundle";
    readonly releaseGateReadinessCommand: "pnpm check:rust-checker-release-gate-readiness";
    readonly releaseGateShadowCommand: "pnpm check:rust-checker-release-gate-shadow";
    readonly releaseGateShadowReviewCommand: "pnpm check:rust-checker-release-gate-shadow-review";
    readonly releaseBundleCommand: "pnpm check:rust-release-bundle";
    readonly minimumBoundedLaneCountForRustLaneBundle: 3;
    readonly minimumBoundedLaneCountForRustReleaseBundle: 3;
    readonly minimumSuccessfulShadowRunsForRustReleaseBundle: 3;
    readonly checkerBundle: "source-missing";
    readonly releaseGateStage: "enforced";
    readonly includedInRustLaneBundle: true;
    readonly includedInRustReleaseBundle: true;
  };
}

export interface CheckerStyleUnusedFindingV0 {
  readonly filePath: string;
  readonly code: string;
  readonly severity: string;
  readonly range: {
    readonly start: {
      readonly line: number;
      readonly character: number;
    };
    readonly end: {
      readonly line: number;
      readonly character: number;
    };
  };
  readonly message: string;
  readonly analysisReason?: string;
  readonly valueCertaintyShapeLabel?: string;
}

export interface CheckerStyleUnusedCanonicalCandidateBundleV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly reportVersion: string;
  readonly bundle: "style-unused";
  readonly distinctFileCount: number;
  readonly codeCounts: Readonly<Record<string, number>>;
  readonly summary: {
    readonly warnings: number;
    readonly hints: number;
    readonly total: number;
  };
  readonly findings: readonly CheckerStyleUnusedFindingV0[];
}

export interface CheckerStyleUnusedCanonicalProducerSignalV0 {
  readonly schemaVersion: string;
  readonly inputVersion: string;
  readonly canonicalCandidate: CheckerStyleUnusedCanonicalCandidateBundleV0;
  readonly boundedCheckerGate: {
    readonly canonicalCandidateCommand: "pnpm check:rust-checker-style-unused-canonical-candidate";
    readonly canonicalProducerCommand: "pnpm check:rust-checker-style-unused-canonical-producer";
    readonly consumerBoundaryCommand: "pnpm check:rust-checker-style-unused-consumer-boundary";
    readonly boundedCheckerLaneCommand: "pnpm check:rust-checker-bounded-lanes";
    readonly promotionReviewCommand: "pnpm check:rust-checker-promotion-review";
    readonly promotionEvidenceCommand: "pnpm check:rust-checker-promotion-evidence";
    readonly broaderRustLaneCommand: "pnpm check:rust-lane-bundle";
    readonly releaseGateReadinessCommand: "pnpm check:rust-checker-release-gate-readiness";
    readonly releaseGateShadowCommand: "pnpm check:rust-checker-release-gate-shadow";
    readonly releaseGateShadowReviewCommand: "pnpm check:rust-checker-release-gate-shadow-review";
    readonly releaseBundleCommand: "pnpm check:rust-release-bundle";
    readonly minimumBoundedLaneCountForRustLaneBundle: 3;
    readonly minimumBoundedLaneCountForRustReleaseBundle: 3;
    readonly minimumSuccessfulShadowRunsForRustReleaseBundle: 3;
    readonly checkerBundle: "style-unused";
    readonly releaseGateStage: "enforced";
    readonly includedInRustLaneBundle: true;
    readonly includedInRustReleaseBundle: true;
  };
}

export interface OmenaQuerySelectedQueryBackendCapabilityV0 {
  readonly backendKind: string;
  readonly sourceResolution: boolean;
  readonly expressionSemantics: boolean;
  readonly selectorUsage: boolean;
  readonly styleSemanticGraph: boolean;
}

export interface OmenaQuerySelectedQueryRunnerCommandV0 {
  readonly surface: string;
  readonly command: string;
  readonly inputContract: string;
  readonly outputProduct: string;
}

export interface OmenaQuerySchemaVersionPolicyV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly currentVersion: string;
  readonly currentVersionLabel: string;
  readonly acceptedVersions: readonly string[];
  readonly deprecatedVersions: readonly string[];
  readonly rejectedVersionPolicy: string;
  readonly missingVersionPolicy: string;
  readonly migrationPolicy: readonly string[];
  readonly compatibilityGate: string;
}

export interface OmenaQuerySchemaVersionCheckV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly requestedVersion: string | null;
  readonly currentVersion: string;
  readonly accepted: boolean;
  readonly status: string;
  readonly migrationAction: string;
  readonly reason: string;
}

export interface OmenaQuerySelectedQueryAdapterCapabilitiesV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly defaultCandidateBackend: string;
  readonly schemaVersionPolicy: OmenaQuerySchemaVersionPolicyV0;
  readonly schemaVersionChecks: readonly OmenaQuerySchemaVersionCheckV0[];
  readonly backendKinds: readonly OmenaQuerySelectedQueryBackendCapabilityV0[];
  readonly runnerCommands: readonly OmenaQuerySelectedQueryRunnerCommandV0[];
  readonly expressionSemanticsPayloadContracts: readonly string[];
  readonly requiredInputContracts: readonly string[];
  readonly adapterReadiness: readonly string[];
  readonly routingStatus: string;
}

export async function runShadow(snapshot: unknown): Promise<ShadowSummaryV0> {
  return runShadowJson<ShadowSummaryV0>([], snapshot);
}

export async function runShadowOmenaQuerySelectedQueryAdapterCapabilities(): Promise<OmenaQuerySelectedQueryAdapterCapabilitiesV0> {
  return runShadowJson<OmenaQuerySelectedQueryAdapterCapabilitiesV0>(
    ["omena-query-selected-query-adapter-capabilities"],
    null,
  );
}

export async function runShadowTypeFactInput(
  input: EngineInputV2,
): Promise<TypeFactInputSummaryV0> {
  return runShadowJson<TypeFactInputSummaryV0>(["input-type-facts"], input);
}

export async function runShadowQueryPlanInput(input: EngineInputV2): Promise<QueryPlanSummaryV0> {
  return runShadowJson<QueryPlanSummaryV0>(["input-query-plan"], input);
}

export async function runShadowExpressionDomainInput(
  input: EngineInputV2,
): Promise<ExpressionDomainPlanSummaryV0> {
  return runShadowJson<ExpressionDomainPlanSummaryV0>(["input-expression-domains"], input);
}

export async function runShadowExpressionDomainFragmentsInput(
  input: EngineInputV2,
): Promise<ExpressionDomainFragmentsV0> {
  return runShadowJson<ExpressionDomainFragmentsV0>(["input-expression-domain-fragments"], input);
}

export async function runShadowExpressionDomainCandidatesInput(
  input: EngineInputV2,
): Promise<ExpressionDomainCandidatesV0> {
  return runShadowJson<ExpressionDomainCandidatesV0>(["input-expression-domain-candidates"], input);
}

export async function runShadowExpressionDomainCanonicalCandidateInput(
  input: EngineInputV2,
): Promise<ExpressionDomainCanonicalCandidateBundleV0> {
  return runShadowJson<ExpressionDomainCanonicalCandidateBundleV0>(
    ["input-expression-domain-canonical-candidate"],
    input,
  );
}

export async function runShadowExpressionDomainEvaluatorCandidatesInput(
  input: EngineInputV2,
): Promise<ExpressionDomainEvaluatorCandidatesV0> {
  return runShadowJson<ExpressionDomainEvaluatorCandidatesV0>(
    ["input-expression-domain-evaluator-candidates"],
    input,
  );
}

export async function runShadowExpressionDomainFlowAnalysisInput(
  input: EngineInputV2,
): Promise<ExpressionDomainFlowAnalysisV0> {
  return runShadowJson<ExpressionDomainFlowAnalysisV0>(
    ["input-expression-domain-flow-analysis"],
    input,
  );
}

export async function runShadowExpressionDomainCallSiteFlowAnalysisInput(
  input: EngineInputV2,
): Promise<ExpressionDomainCallSiteFlowAnalysisV0> {
  return runShadowJson<ExpressionDomainCallSiteFlowAnalysisV0>(
    ["input-expression-domain-call-site-flow-analysis"],
    input,
  );
}

export async function runShadowExpressionDomainCanonicalProducerInput(
  input: EngineInputV2,
): Promise<ExpressionDomainCanonicalProducerSignalV0> {
  return runShadowJson<ExpressionDomainCanonicalProducerSignalV0>(
    ["input-expression-domain-canonical-producer"],
    input,
  );
}

export async function runShadowSelectorUsagePlanInput(
  input: EngineInputV2,
): Promise<SelectorUsagePlanSummaryV0> {
  return runShadowJson<SelectorUsagePlanSummaryV0>(["input-selector-usage-plan"], input);
}

export async function runShadowSelectorUsageFragmentsInput(
  input: EngineInputV2,
): Promise<SelectorUsageFragmentsV0> {
  return runShadowJson<SelectorUsageFragmentsV0>(["input-selector-usage-fragments"], input);
}

export async function runShadowSelectorUsageQueryFragmentsInput(
  input: EngineInputV2,
): Promise<SelectorUsageQueryFragmentsV0> {
  return runShadowJson<SelectorUsageQueryFragmentsV0>(
    ["input-selector-usage-query-fragments"],
    input,
  );
}

export async function runShadowSourceResolutionPlanInput(
  input: EngineInputV2,
): Promise<SourceResolutionPlanSummaryV0> {
  return runShadowJson<SourceResolutionPlanSummaryV0>(["input-source-resolution-plan"], input);
}

export async function runShadowSourceResolutionQueryFragmentsInput(
  input: EngineInputV2,
): Promise<SourceResolutionQueryFragmentsV0> {
  return runShadowJson<SourceResolutionQueryFragmentsV0>(
    ["input-source-resolution-query-fragments"],
    input,
  );
}

export async function runShadowSourceResolutionMatchFragmentsInput(
  input: EngineInputV2,
): Promise<SourceResolutionMatchFragmentsV0> {
  return runShadowJson<SourceResolutionMatchFragmentsV0>(
    ["input-source-resolution-match-fragments"],
    input,
  );
}

export async function runShadowSourceResolutionCandidatesInput(
  input: EngineInputV2,
): Promise<SourceResolutionCandidatesV0> {
  return runShadowJson<SourceResolutionCandidatesV0>(["input-source-resolution-candidates"], input);
}

export async function runShadowSourceResolutionEvaluatorCandidatesInput(
  input: EngineInputV2,
): Promise<SourceResolutionEvaluatorCandidatesV0> {
  return runShadowJson<SourceResolutionEvaluatorCandidatesV0>(
    ["input-source-resolution-evaluator-candidates"],
    input,
  );
}

export async function runShadowSourceResolutionCanonicalCandidateInput(
  input: EngineInputV2,
): Promise<SourceResolutionCanonicalCandidateBundleV0> {
  return runShadowJson<SourceResolutionCanonicalCandidateBundleV0>(
    ["input-source-resolution-canonical-candidate"],
    input,
  );
}

export async function runShadowSourceResolutionCanonicalProducerInput(
  input: EngineInputV2,
): Promise<SourceResolutionCanonicalProducerSignalV0> {
  return runShadowJson<SourceResolutionCanonicalProducerSignalV0>(
    ["input-source-resolution-canonical-producer"],
    input,
  );
}

export async function runShadowExpressionSemanticsFragmentsInput(
  input: EngineInputV2,
): Promise<ExpressionSemanticsFragmentsV0> {
  return runShadowJson<ExpressionSemanticsFragmentsV0>(
    ["input-expression-semantics-fragments"],
    input,
  );
}

export async function runShadowExpressionSemanticsQueryFragmentsInput(
  input: EngineInputV2,
): Promise<ExpressionSemanticsQueryFragmentsV0> {
  return runShadowJson<ExpressionSemanticsQueryFragmentsV0>(
    ["input-expression-semantics-query-fragments"],
    input,
  );
}

export async function runShadowExpressionSemanticsMatchFragmentsInput(
  input: EngineInputV2,
): Promise<ExpressionSemanticsMatchFragmentsV0> {
  return runShadowJson<ExpressionSemanticsMatchFragmentsV0>(
    ["input-expression-semantics-match-fragments"],
    input,
  );
}

export async function runShadowExpressionSemanticsCandidatesInput(
  input: EngineInputV2,
): Promise<ExpressionSemanticsCandidatesV0> {
  return runShadowJson<ExpressionSemanticsCandidatesV0>(
    ["input-expression-semantics-candidates"],
    input,
  );
}

export async function runShadowExpressionSemanticsCanonicalCandidateInput(
  input: EngineInputV2,
): Promise<ExpressionSemanticsCanonicalCandidateBundleV0> {
  return runShadowJson<ExpressionSemanticsCanonicalCandidateBundleV0>(
    ["input-expression-semantics-canonical-candidate"],
    input,
  );
}

export async function runShadowExpressionSemanticsEvaluatorCandidatesInput(
  input: EngineInputV2,
): Promise<ExpressionSemanticsEvaluatorCandidatesV0> {
  return runShadowJson<ExpressionSemanticsEvaluatorCandidatesV0>(
    ["input-expression-semantics-evaluator-candidates"],
    input,
  );
}

export async function runShadowExpressionSemanticsCanonicalProducerInput(
  input: EngineInputV2,
): Promise<ExpressionSemanticsCanonicalProducerSignalV0> {
  return runShadowJson<ExpressionSemanticsCanonicalProducerSignalV0>(
    ["input-expression-semantics-canonical-producer"],
    input,
  );
}

export async function runShadowSourceSideCanonicalProducerInput(
  input: EngineInputV2,
): Promise<SourceSideCanonicalProducerSignalV0> {
  return runShadowJson<SourceSideCanonicalProducerSignalV0>(
    ["input-source-side-canonical-producer"],
    input,
  );
}

export async function runShadowSourceSideCanonicalCandidateInput(
  input: EngineInputV2,
): Promise<SourceSideCanonicalCandidateBundleV0> {
  return runShadowJson<SourceSideCanonicalCandidateBundleV0>(
    ["input-source-side-canonical-candidate"],
    input,
  );
}

export async function runShadowSourceSideEvaluatorCandidatesInput(
  input: EngineInputV2,
): Promise<SourceSideEvaluatorCandidatesV0> {
  return runShadowJson<SourceSideEvaluatorCandidatesV0>(
    ["input-source-side-evaluator-candidates"],
    input,
  );
}

export async function runShadowSemanticCanonicalCandidateInput(
  input: EngineInputV2,
): Promise<SemanticCanonicalCandidateBundleV0> {
  return runShadowJson<SemanticCanonicalCandidateBundleV0>(
    ["input-semantic-canonical-candidate"],
    input,
  );
}

export async function runShadowSemanticEvaluatorCandidatesInput(
  input: EngineInputV2,
): Promise<SemanticEvaluatorCandidatesV0> {
  return runShadowJson<SemanticEvaluatorCandidatesV0>(
    ["input-semantic-evaluator-candidates"],
    input,
  );
}

export async function runShadowSemanticCanonicalProducerInput(
  input: EngineInputV2,
): Promise<SemanticCanonicalProducerSignalV0> {
  return runShadowJson<SemanticCanonicalProducerSignalV0>(
    ["input-semantic-canonical-producer"],
    input,
  );
}

export async function runShadowSourceResolutionFragmentsInput(
  input: EngineInputV2,
): Promise<SourceResolutionFragmentsV0> {
  return runShadowJson<SourceResolutionFragmentsV0>(["input-source-resolution-fragments"], input);
}

export async function runShadowCheckerStyleRecoveryCanonicalCandidate(
  snapshot: unknown,
): Promise<CheckerStyleRecoveryCanonicalCandidateBundleV0> {
  return runShadowJson<CheckerStyleRecoveryCanonicalCandidateBundleV0>(
    ["output-checker-style-recovery-canonical-candidate"],
    snapshot,
  );
}

export async function runShadowCheckerStyleRecoveryCanonicalProducer(
  snapshot: unknown,
): Promise<CheckerStyleRecoveryCanonicalProducerSignalV0> {
  return runShadowJson<CheckerStyleRecoveryCanonicalProducerSignalV0>(
    ["output-checker-style-recovery-canonical-producer"],
    snapshot,
  );
}

export async function runShadowCheckerStyleUnusedCanonicalCandidate(
  snapshot: unknown,
): Promise<CheckerStyleUnusedCanonicalCandidateBundleV0> {
  return runShadowJson<CheckerStyleUnusedCanonicalCandidateBundleV0>(
    ["output-checker-style-unused-canonical-candidate"],
    snapshot,
  );
}

export async function runShadowCheckerStyleUnusedCanonicalProducer(
  snapshot: unknown,
): Promise<CheckerStyleUnusedCanonicalProducerSignalV0> {
  return runShadowJson<CheckerStyleUnusedCanonicalProducerSignalV0>(
    ["output-checker-style-unused-canonical-producer"],
    snapshot,
  );
}

export async function runShadowCheckerSourceMissingCanonicalCandidate(
  snapshot: unknown,
): Promise<CheckerSourceMissingCanonicalCandidateBundleV0> {
  return runShadowJson<CheckerSourceMissingCanonicalCandidateBundleV0>(
    ["output-checker-source-missing-canonical-candidate"],
    snapshot,
  );
}

export async function runShadowCheckerSourceMissingCanonicalProducer(
  snapshot: unknown,
): Promise<CheckerSourceMissingCanonicalProducerSignalV0> {
  return runShadowJson<CheckerSourceMissingCanonicalProducerSignalV0>(
    ["output-checker-source-missing-canonical-producer"],
    snapshot,
  );
}

function runShadowJson<T>(args: string[], payload: unknown): Promise<T> {
  return new Promise((resolve, reject) => {
    const child = spawn(
      "cargo",
      [
        "run",
        "--manifest-path",
        RUST_MANIFEST,
        "-p",
        "engine-shadow-runner",
        "--quiet",
        "--",
        ...args,
      ],
      {
        cwd: REPO_ROOT,
        stdio: ["pipe", "pipe", "pipe"],
      },
    );

    const stdout: string[] = [];
    const stderr: string[] = [];
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => stdout.push(chunk));
    child.stderr.on("data", (chunk) => stderr.push(chunk));
    child.once("error", reject);
    child.once("close", (code) => {
      if (code !== 0) {
        reject(
          new Error(
            [`engine-shadow-runner exited with code ${code}`, stderr.join("").trim()]
              .filter(Boolean)
              .join("\n"),
          ),
        );
        return;
      }

      try {
        resolve(JSON.parse(stdout.join("")) as T);
      } catch (error) {
        reject(error);
      }
    });

    child.stdin.end(JSON.stringify(payload));
  });
}

export function deriveTsShadowSummary(snapshot: EngineParitySnapshotV2): ShadowSummaryV0 {
  const byKind: Record<string, number> = {};
  const constrainedKinds: Record<string, number> = {};
  const queryKindCounts: Record<string, number> = {};
  const expressionValueDomainKinds: Record<string, number> = {};
  const expressionValueConstraintKinds: Record<string, number> = {};
  const expressionConstraintDetailCounts = createConstraintDetailCounts();
  const expressionValueCertaintyShapes: Record<string, number> = {};
  const expressionSelectorCertaintyShapes: Record<string, number> = {};
  const resolutionValueConstraintKinds: Record<string, number> = {};
  const resolutionConstraintDetailCounts = createConstraintDetailCounts();
  const resolutionValueCertaintyShapes: Record<string, number> = {};
  const resolutionSelectorCertaintyShapes: Record<string, number> = {};
  const distinctFactFiles = new Set<string>();
  let finiteValueCount = 0;
  let selectorUsageReferencedCount = 0;
  let selectorUsageUnreferencedCount = 0;
  let selectorUsageTotalReferences = 0;
  let selectorUsageDirectReferences = 0;
  let selectorUsageEditableDirectReferences = 0;
  let selectorUsageExactReferences = 0;
  let selectorUsageInferredOrBetterReferences = 0;
  let selectorUsageExpandedCount = 0;
  let selectorUsageStyleDependencyCount = 0;
  let expectedExpressionSemanticsCount = 0;
  let expectedSourceExpressionResolutionCount = 0;
  let expectedSelectorUsageCount = 0;
  const expectedExpressionIds = new Set<string>();
  const expectedSelectorUsageIds = new Set<string>();
  const expressionSemanticsIds = new Set<string>();
  const resolutionIds = new Set<string>();
  const selectorUsageIds = new Set<string>();

  for (const source of snapshot.input.sources) {
    expectedExpressionSemanticsCount += source.document.classExpressions.length;
    for (const expression of source.document.classExpressions) {
      expectedExpressionIds.add(expression.id);
    }
  }
  expectedSourceExpressionResolutionCount = expectedExpressionSemanticsCount;
  for (const style of snapshot.input.styles) {
    expectedSelectorUsageCount += style.document.selectors.filter(
      (selector) => selector.viewKind === "canonical",
    ).length;
    for (const selector of style.document.selectors) {
      if (selector.viewKind === "canonical") {
        expectedSelectorUsageIds.add(selector.canonicalName);
      }
    }
  }

  for (const entry of snapshot.input.typeFacts) {
    distinctFactFiles.add(entry.filePath);
    byKind[entry.facts.kind] = (byKind[entry.facts.kind] ?? 0) + 1;

    if (entry.facts.values) {
      finiteValueCount += entry.facts.values.length;
    }

    if (entry.facts.kind === "constrained") {
      constrainedKinds[entry.facts.constraintKind] =
        (constrainedKinds[entry.facts.constraintKind] ?? 0) + 1;
    }
  }

  for (const query of snapshot.output.queryResults) {
    queryKindCounts[query.kind] = (queryKindCounts[query.kind] ?? 0) + 1;
    collectQueryPayloadSummary(
      query,
      expressionValueDomainKinds,
      expressionValueConstraintKinds,
      expressionConstraintDetailCounts,
      expressionValueCertaintyShapes,
      expressionSelectorCertaintyShapes,
      resolutionValueConstraintKinds,
      resolutionConstraintDetailCounts,
      resolutionValueCertaintyShapes,
      resolutionSelectorCertaintyShapes,
      expressionSemanticsIds,
      resolutionIds,
      selectorUsageIds,
      (payload) => {
        selectorUsageTotalReferences += payload.totalReferences;
        selectorUsageDirectReferences += payload.directReferenceCount;
        selectorUsageEditableDirectReferences += payload.editableDirectReferenceCount;
        selectorUsageExactReferences += payload.exactReferenceCount;
        selectorUsageInferredOrBetterReferences += payload.inferredOrBetterReferenceCount;
        if (payload.hasExpandedReferences) {
          selectorUsageExpandedCount += 1;
        }
        if (payload.hasStyleDependencyReferences) {
          selectorUsageStyleDependencyCount += 1;
        }
        const used = payload.hasAnyReferences;
        if (used) {
          selectorUsageReferencedCount += 1;
        } else {
          selectorUsageUnreferencedCount += 1;
        }
      },
    );
  }

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    sourceCount: snapshot.input.sources.length,
    styleCount: snapshot.input.styles.length,
    typeFactCount: snapshot.input.typeFacts.length,
    distinctFactFiles: distinctFactFiles.size,
    byKind,
    constrainedKinds,
    finiteValueCount,
    queryResultCount: snapshot.output.queryResults.length,
    queryKindCounts,
    expressionValueDomainKinds,
    expressionValueConstraintKinds,
    expressionConstraintDetailCounts,
    expressionValueCertaintyShapes,
    expressionSelectorCertaintyShapes,
    resolutionValueConstraintKinds,
    resolutionConstraintDetailCounts,
    resolutionValueCertaintyShapes,
    resolutionSelectorCertaintyShapes,
    selectorUsageReferencedCount,
    selectorUsageUnreferencedCount,
    selectorUsageTotalReferences,
    selectorUsageDirectReferences,
    selectorUsageEditableDirectReferences,
    selectorUsageExactReferences,
    selectorUsageInferredOrBetterReferences,
    selectorUsageExpandedCount,
    selectorUsageStyleDependencyCount,
    expectedExpressionSemanticsCount,
    expectedSourceExpressionResolutionCount,
    expectedSelectorUsageCount,
    expectedTotalQueryCount:
      expectedExpressionSemanticsCount +
      expectedSourceExpressionResolutionCount +
      expectedSelectorUsageCount,
    matchedExpressionQueryPairs: [...expectedExpressionIds].filter(
      (id) => expressionSemanticsIds.has(id) && resolutionIds.has(id),
    ).length,
    missingExpressionSemanticsCount: [...expectedExpressionIds].filter(
      (id) => !expressionSemanticsIds.has(id),
    ).length,
    missingSourceExpressionResolutionCount: [...expectedExpressionIds].filter(
      (id) => !resolutionIds.has(id),
    ).length,
    unexpectedExpressionSemanticsCount: [...expressionSemanticsIds].filter(
      (id) => !expectedExpressionIds.has(id),
    ).length,
    unexpectedSourceExpressionResolutionCount: [...resolutionIds].filter(
      (id) => !expectedExpressionIds.has(id),
    ).length,
    matchedSelectorUsageCount: [...expectedSelectorUsageIds].filter((id) =>
      selectorUsageIds.has(id),
    ).length,
    missingSelectorUsageCount: [...expectedSelectorUsageIds].filter(
      (id) => !selectorUsageIds.has(id),
    ).length,
    unexpectedSelectorUsageCount: [...selectorUsageIds].filter(
      (id) => !expectedSelectorUsageIds.has(id),
    ).length,
    rewritePlanCount: snapshot.output.rewritePlans.length,
    checkerWarningCount: snapshot.output.checkerReport.summary.warnings,
    checkerHintCount: snapshot.output.checkerReport.summary.hints,
    checkerTotalFindings: snapshot.output.checkerReport.summary.total,
  };
}

export function deriveTsTypeFactInputSummary(
  snapshot: EngineParitySnapshotV2,
): TypeFactInputSummaryV0 {
  const byKind: Record<string, number> = {};
  const constrainedKinds: Record<string, number> = {};
  const distinctFactFiles = new Set<string>();
  let finiteValueCount = 0;

  for (const entry of snapshot.input.typeFacts) {
    distinctFactFiles.add(entry.filePath);
    byKind[entry.facts.kind] = (byKind[entry.facts.kind] ?? 0) + 1;

    if (entry.facts.kind === "finiteSet") {
      finiteValueCount += entry.facts.values.length;
    }

    if (entry.facts.kind === "constrained") {
      constrainedKinds[entry.facts.constraintKind] =
        (constrainedKinds[entry.facts.constraintKind] ?? 0) + 1;
    }
  }

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    typeFactCount: snapshot.input.typeFacts.length,
    distinctFactFiles: distinctFactFiles.size,
    byKind,
    constrainedKinds,
    finiteValueCount,
  };
}

export function deriveTsQueryPlanSummary(snapshot: EngineParitySnapshotV2): QueryPlanSummaryV0 {
  const expressionIds = snapshot.input.sources.flatMap((source) =>
    source.document.classExpressions.map((expression) => expression.id),
  );
  const selectorUsageIds = snapshot.input.styles.flatMap((style) =>
    style.document.selectors
      .filter((selector) => selector.viewKind === "canonical")
      .map((selector) => selector.canonicalName),
  );

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    expressionSemanticsIds: expressionIds,
    sourceExpressionResolutionIds: expressionIds,
    selectorUsageIds,
    totalQueryCount: expressionIds.length * 2 + selectorUsageIds.length,
  };
}

export function deriveTsExpressionDomainPlanSummary(
  snapshot: EngineParitySnapshotV2,
): ExpressionDomainPlanSummaryV0 {
  const valueDomainKinds: Record<string, number> = {};
  const valueConstraintKinds: Record<string, number> = {};
  const constraintDetailCounts = createConstraintDetailCounts();
  const plannedExpressionIds: string[] = [];
  let finiteValueCount = 0;

  for (const entry of snapshot.input.typeFacts) {
    plannedExpressionIds.push(entry.expressionId);
    increment(valueDomainKinds, entry.facts.kind);
    if (entry.facts.kind === "finiteSet") {
      finiteValueCount += entry.facts.values.length;
    }
    if (entry.facts.kind === "constrained") {
      increment(valueConstraintKinds, entry.facts.constraintKind);
    }
    collectConstraintDetailCounts(
      constraintDetailCounts,
      entry.facts.prefix,
      entry.facts.suffix,
      entry.facts.minLen,
      entry.facts.maxLen,
      entry.facts.charMust,
      entry.facts.charMay,
      entry.facts.mayIncludeOtherChars === true,
    );
  }

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    plannedExpressionIds,
    valueDomainKinds,
    valueConstraintKinds,
    constraintDetailCounts,
    finiteValueCount,
  };
}

export function deriveTsExpressionDomainFragments(
  snapshot: EngineParitySnapshotV2,
): ExpressionDomainFragmentsV0 {
  const fragments = snapshot.input.typeFacts
    .map((entry) => {
      const fragment: ExpressionDomainFragmentV0 = {
        expressionId: entry.expressionId,
        filePath: entry.filePath,
        valueDomainKind: entry.facts.kind,
        finiteValueCount: entry.facts.values?.length ?? 0,
      };
      if (entry.facts.constraintKind) {
        fragment.valueConstraintKind = entry.facts.constraintKind;
      }
      if (entry.facts.prefix) {
        fragment.valuePrefix = entry.facts.prefix;
      }
      if (entry.facts.suffix) {
        fragment.valueSuffix = entry.facts.suffix;
      }
      if (entry.facts.minLen !== undefined) {
        fragment.valueMinLen = entry.facts.minLen;
      }
      if (entry.facts.maxLen !== undefined) {
        fragment.valueMaxLen = entry.facts.maxLen;
      }
      if (entry.facts.charMust) {
        fragment.valueCharMust = entry.facts.charMust;
      }
      if (entry.facts.charMay) {
        fragment.valueCharMay = entry.facts.charMay;
      }
      if (entry.facts.mayIncludeOtherChars) {
        fragment.valueMayIncludeOtherChars = true;
      }
      return fragment;
    })
    .toSorted((a, b) => a.expressionId.localeCompare(b.expressionId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments,
  };
}

export function deriveTsExpressionDomainCandidates(
  snapshot: EngineParitySnapshotV2,
): ExpressionDomainCandidatesV0 {
  const candidates = deriveTsExpressionDomainFragments(snapshot).fragments.map((fragment) => {
    const candidate: ExpressionDomainCandidateV0 = {
      expressionId: fragment.expressionId,
      filePath: fragment.filePath,
      valueDomainKind: fragment.valueDomainKind,
      finiteValueCount: fragment.finiteValueCount,
    };
    if (fragment.valueConstraintKind) {
      candidate.valueConstraintKind = fragment.valueConstraintKind;
    }
    if (fragment.valuePrefix) {
      candidate.valuePrefix = fragment.valuePrefix;
    }
    if (fragment.valueSuffix) {
      candidate.valueSuffix = fragment.valueSuffix;
    }
    if (fragment.valueMinLen !== undefined) {
      candidate.valueMinLen = fragment.valueMinLen;
    }
    if (fragment.valueMaxLen !== undefined) {
      candidate.valueMaxLen = fragment.valueMaxLen;
    }
    if (fragment.valueCharMust) {
      candidate.valueCharMust = fragment.valueCharMust;
    }
    if (fragment.valueCharMay) {
      candidate.valueCharMay = fragment.valueCharMay;
    }
    if (fragment.valueMayIncludeOtherChars) {
      candidate.valueMayIncludeOtherChars = true;
    }
    return candidate;
  });

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    candidates,
  };
}

export function deriveTsExpressionDomainCanonicalCandidateBundle(
  snapshot: EngineParitySnapshotV2,
): ExpressionDomainCanonicalCandidateBundleV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    planSummary: deriveTsExpressionDomainPlanSummary(snapshot),
    fragments: deriveTsExpressionDomainFragments(snapshot).fragments,
    candidates: deriveTsExpressionDomainCandidates(snapshot).candidates,
  };
}

const MAX_EXPECTED_FINITE_CLASS_VALUES = 8;

type ExpressionValueDomainPayloadLike = {
  readonly valueDomainKind: string;
  readonly valueConstraintKind?: string;
  readonly valuePrefix?: string;
  readonly valueSuffix?: string;
  readonly valueMinLen?: number;
  readonly valueMaxLen?: number;
  readonly valueCharMust?: string;
  readonly valueCharMay?: string;
  readonly valueMayIncludeOtherChars?: boolean;
  readonly valueProvenance?: string;
  readonly finiteValues?: readonly string[] | null;
};

function deriveReducedExpressionValueDomainKind(payload: ExpressionValueDomainPayloadLike): string {
  if (payload.valueDomainKind === "none" || payload.valueDomainKind === "unknown") {
    return "none";
  }

  if (payload.valueDomainKind === "top") {
    return "top";
  }

  if (payload.valueDomainKind === "exact") {
    if (!payload.finiteValues?.length) return "top";
    return hasExpressionValueConstraintDetails(payload)
      ? finiteExpressionValueDomainKind(
          payload.finiteValues.filter((value) => expressionValueMatchesConstraint(payload, value)),
        )
      : "exact";
  }

  if (payload.valueDomainKind === "finiteSet") {
    const values = payload.finiteValues ?? [];
    return hasExpressionValueConstraintDetails(payload)
      ? finiteExpressionValueDomainKind(
          values.filter((value) => expressionValueMatchesConstraint(payload, value)),
        )
      : finiteExpressionValueDomainKind(values);
  }

  if (payload.valueDomainKind === "constrained") {
    if (payload.finiteValues?.length) {
      return finiteExpressionValueDomainKind(
        payload.finiteValues.filter((value) => expressionValueMatchesConstraint(payload, value)),
      );
    }
    return payload.valueConstraintKind ?? "top";
  }

  return payload.valueDomainKind;
}

function deriveReducedExpressionValueDomainDerivation(
  payload: ExpressionValueDomainPayloadLike,
): ReducedClassValueDerivationV0 {
  const inputFactKind = payload.valueDomainKind === "none" ? "unknown" : payload.valueDomainKind;
  let currentKind = deriveBaseExpressionClassValueKind(payload);
  const steps: ReducedClassValueDerivationStepV0[] = [
    {
      operation: "baseFromFacts",
      resultKind: currentKind,
      reason: "mapped input facts to the base abstract value",
    },
  ];

  if (
    hasExpressionValueConstraintDetails(payload) &&
    (inputFactKind === "exact" || inputFactKind === "finiteSet")
  ) {
    const refinementKind = deriveConstraintExpressionClassValueKind(payload);
    const resultKind = deriveReducedExpressionValueDomainKind(payload);
    steps.push({
      operation: "intersectConstraint",
      inputKind: currentKind,
      refinementKind,
      resultKind,
      reason: "refined exact or finite facts with constraint details",
    });
    currentKind = resultKind;
  }

  if (
    inputFactKind !== "exact" &&
    inputFactKind !== "finiteSet" &&
    payload.finiteValues != null &&
    payload.finiteValues.length > 0
  ) {
    const refinementKind = finiteExpressionValueDomainKind(payload.finiteValues);
    const resultKind = deriveReducedExpressionValueDomainKind(payload);
    steps.push({
      operation: "intersectFiniteValues",
      inputKind: currentKind,
      refinementKind,
      resultKind,
      reason: "refined constrained facts with explicit finite values",
    });
    currentKind = resultKind;
  }

  return {
    schemaVersion: "0",
    product: "omena-abstract-value.reduced-class-value-derivation",
    inputFactKind,
    ...(payload.valueConstraintKind ? { inputConstraintKind: payload.valueConstraintKind } : {}),
    inputValueCount: payload.finiteValues?.length ?? 0,
    reducedKind:
      inputFactKind === "unknown" ? "none" : currentKind === "unknown" ? "none" : currentKind,
    steps,
  };
}

function deriveReducedExpressionValueDomainProvenanceTree(
  payload: ExpressionValueDomainPayloadLike,
): AbstractClassValueProvenanceTreeV0 {
  const value = deriveReducedExpressionAbstractValue(payload);
  const valueKind = value.kind;
  const valueProvenance = provenanceForExpressionAbstractValue(value);
  const rootDetail = provenanceTreeRootDetail(value);

  return {
    schemaVersion: "0",
    product: "omena-abstract-value.provenance-tree",
    valueKind,
    value,
    ...(valueProvenance ? { valueProvenance } : {}),
    root: {
      operation: provenanceTreeRootOperation(value, valueProvenance),
      resultKind: valueKind,
      ...(valueProvenance ? { resultProvenance: valueProvenance } : {}),
      ...(rootDetail ? { detail: rootDetail } : {}),
      reason: provenanceTreeRootReason(value, valueProvenance),
      children: provenanceTreeConstraintChildren(value),
    },
  };
}

function deriveReducedExpressionAbstractValue(
  payload: ExpressionValueDomainPayloadLike,
): AbstractClassValueV0 {
  const inputFactKind = payload.valueDomainKind === "none" ? "unknown" : payload.valueDomainKind;
  let value = baseExpressionAbstractValue(payload);

  if (
    hasExpressionValueConstraintDetails(payload) &&
    (inputFactKind === "exact" || inputFactKind === "finiteSet")
  ) {
    value = intersectExpressionAbstractValues(value, constraintExpressionAbstractValue(payload));
  }

  if (
    inputFactKind !== "exact" &&
    inputFactKind !== "finiteSet" &&
    payload.finiteValues != null &&
    payload.finiteValues.length > 0
  ) {
    value = intersectExpressionAbstractValues(
      value,
      finiteSetExpressionAbstractValue(payload.finiteValues),
    );
  }

  return value;
}

function baseExpressionAbstractValue(
  payload: ExpressionValueDomainPayloadLike,
): AbstractClassValueV0 {
  switch (payload.valueDomainKind) {
    case "exact":
      return payload.finiteValues?.length
        ? exactExpressionAbstractValue(payload.finiteValues[0] ?? "")
        : topExpressionAbstractValue();
    case "finiteSet":
      return finiteSetExpressionAbstractValue(payload.finiteValues ?? []);
    case "constrained":
      return constraintExpressionAbstractValue(payload);
    case "none":
    case "unknown":
    case "top":
    default:
      return topExpressionAbstractValue();
  }
}

function constraintExpressionAbstractValue(
  payload: ExpressionValueDomainPayloadLike,
): AbstractClassValueV0 {
  switch (payload.valueConstraintKind) {
    case "prefix":
      return prefixExpressionAbstractValue(payload.valuePrefix ?? "");
    case "suffix":
      return suffixExpressionAbstractValue(payload.valueSuffix ?? "");
    case "prefixSuffix":
      return prefixSuffixExpressionAbstractValue(
        payload.valuePrefix ?? "",
        payload.valueSuffix ?? "",
        payload.valueMinLen,
      );
    case "charInclusion":
      return charInclusionExpressionAbstractValue(
        payload.valueCharMust ?? "",
        payload.valueCharMay ?? "",
        payload.valueMayIncludeOtherChars ?? false,
      );
    case "composite":
      return compositeExpressionAbstractValue({
        ...(payload.valuePrefix !== undefined ? { prefix: payload.valuePrefix } : {}),
        ...(payload.valueSuffix !== undefined ? { suffix: payload.valueSuffix } : {}),
        ...(payload.valueMinLen !== undefined ? { minLength: payload.valueMinLen } : {}),
        mustChars: payload.valueCharMust ?? "",
        mayChars: payload.valueCharMay ?? "",
        mayIncludeOtherChars: payload.valueMayIncludeOtherChars ?? false,
      });
    default:
      return topExpressionAbstractValue();
  }
}

function intersectExpressionAbstractValues(
  left: AbstractClassValueV0,
  right: AbstractClassValueV0,
): AbstractClassValueV0 {
  if (left.kind === "bottom" || right.kind === "bottom") return bottomExpressionAbstractValue();
  if (left.kind === "top") return right;
  if (right.kind === "top") return left;

  const leftValues = enumerateFiniteExpressionAbstractValues(left);
  const rightValues = enumerateFiniteExpressionAbstractValues(right);

  if (leftValues && rightValues) {
    const rightValueSet = new Set(rightValues);
    return finiteSetExpressionAbstractValue(leftValues.filter((value) => rightValueSet.has(value)));
  }

  if (leftValues) {
    return finiteSetExpressionAbstractValue(
      leftValues.filter((value) => expressionAbstractValueMatchesString(right, value)),
    );
  }

  if (rightValues) {
    return finiteSetExpressionAbstractValue(
      rightValues.filter((value) => expressionAbstractValueMatchesString(left, value)),
    );
  }

  return bottomExpressionAbstractValue();
}

function bottomExpressionAbstractValue(): AbstractClassValueV0 {
  return { kind: "bottom" };
}

function topExpressionAbstractValue(): AbstractClassValueV0 {
  return { kind: "top" };
}

function exactExpressionAbstractValue(value: string): AbstractClassValueV0 {
  return { kind: "exact", value };
}

function finiteSetExpressionAbstractValue(values: readonly string[]): AbstractClassValueV0 {
  const normalizedValues = normalizeExpressionValues(values);
  if (normalizedValues.length === 0) return bottomExpressionAbstractValue();
  if (normalizedValues.length === 1) return exactExpressionAbstractValue(normalizedValues[0] ?? "");
  if (normalizedValues.length <= MAX_EXPECTED_FINITE_CLASS_VALUES) {
    return { kind: "finiteSet", values: normalizedValues };
  }

  const prefix = meaningfulExpressionLongestCommonPrefix(normalizedValues);
  const suffix = meaningfulExpressionLongestCommonSuffix(normalizedValues);
  const { mustChars, mayChars } = charInclusionFromExpressionFiniteValues(normalizedValues);

  if (prefix.length > 0 || suffix.length > 0) {
    return compositeExpressionAbstractValue({
      ...(prefix.length > 0 ? { prefix } : {}),
      ...(suffix.length > 0 ? { suffix } : {}),
      minLength: Math.min(...normalizedValues.map((value) => value.length)),
      mustChars,
      mayChars,
      provenance: "finiteSetWideningComposite",
    });
  }

  return charInclusionExpressionAbstractValue(mustChars, mayChars, false, "finiteSetWideningChars");
}

function prefixExpressionAbstractValue(prefix: string, provenance?: string): AbstractClassValueV0 {
  return provenance ? { kind: "prefix", prefix, provenance } : { kind: "prefix", prefix };
}

function suffixExpressionAbstractValue(suffix: string, provenance?: string): AbstractClassValueV0 {
  return provenance ? { kind: "suffix", suffix, provenance } : { kind: "suffix", suffix };
}

function prefixSuffixExpressionAbstractValue(
  prefix: string,
  suffix: string,
  minLength?: number,
  provenance?: string,
): AbstractClassValueV0 {
  if (prefix.length === 0 && suffix.length === 0) return topExpressionAbstractValue();
  if (prefix.length === 0) return suffixExpressionAbstractValue(suffix, provenance);
  if (suffix.length === 0) return prefixExpressionAbstractValue(prefix, provenance);

  const value = {
    kind: "prefixSuffix",
    prefix,
    suffix,
    minLength: Math.max(minLength ?? prefix.length + suffix.length, prefix.length + suffix.length),
  };
  return provenance ? { ...value, provenance } : value;
}

function charInclusionExpressionAbstractValue(
  mustChars: string,
  mayChars: string,
  mayIncludeOtherChars: boolean,
  provenance?: string,
): AbstractClassValueV0 {
  const normalizedMustChars = normalizeExpressionCharSet(mustChars);
  const normalizedMayChars = normalizeExpressionCharSet(`${mayChars}${normalizedMustChars}`);
  if (mayIncludeOtherChars && normalizedMustChars.length === 0) return topExpressionAbstractValue();
  if (!mayIncludeOtherChars && normalizedMayChars.length === 0) return topExpressionAbstractValue();

  const value = {
    kind: "charInclusion",
    mustChars: normalizedMustChars,
    mayChars: normalizedMayChars,
    ...(mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
  };
  return provenance ? { ...value, provenance } : value;
}

function compositeExpressionAbstractValue(input: {
  readonly prefix?: string;
  readonly suffix?: string;
  readonly minLength?: number;
  readonly mustChars: string;
  readonly mayChars: string;
  readonly mayIncludeOtherChars?: boolean;
  readonly provenance?: string;
}): AbstractClassValueV0 {
  const prefix = input.prefix ?? "";
  const suffix = input.suffix ?? "";
  const edgeChars = normalizeExpressionCharSet(`${prefix}${suffix}`);
  const mustChars = normalizeExpressionCharSet(`${input.mustChars}${edgeChars}`);
  const mayChars = normalizeExpressionCharSet(`${input.mayChars}${mustChars}`);
  const mayIncludeOtherChars = input.mayIncludeOtherChars ?? false;
  const hasCharInfo = mustChars.length > 0 || (!mayIncludeOtherChars && mayChars.length > 0);

  if (!hasCharInfo) {
    return prefixSuffixExpressionAbstractValue(prefix, suffix, input.minLength, input.provenance);
  }

  if (prefix.length === 0 && suffix.length === 0) {
    return charInclusionExpressionAbstractValue(
      mustChars,
      mayChars,
      mayIncludeOtherChars,
      input.provenance,
    );
  }

  const edgeMinLength = prefix.length + suffix.length;
  const minLength = Math.max(input.minLength ?? edgeMinLength, edgeMinLength, mustChars.length);
  const value = {
    kind: "composite",
    ...(prefix.length > 0 ? { prefix } : {}),
    ...(suffix.length > 0 ? { suffix } : {}),
    minLength,
    mustChars,
    mayChars,
    ...(mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
  };
  return input.provenance ? { ...value, provenance: input.provenance } : value;
}

function enumerateFiniteExpressionAbstractValues(
  value: AbstractClassValueV0,
): readonly string[] | null {
  if (value.kind === "bottom") return [];
  if (value.kind === "exact" && typeof value.value === "string") return [value.value];
  if (value.kind === "finiteSet" && Array.isArray(value.values)) {
    return value.values.filter((item): item is string => typeof item === "string");
  }
  return null;
}

function expressionAbstractValueMatchesString(
  value: AbstractClassValueV0,
  candidate: string,
): boolean {
  switch (value.kind) {
    case "bottom":
      return false;
    case "exact":
      return value.value === candidate;
    case "finiteSet":
      return Array.isArray(value.values) && value.values.includes(candidate);
    case "prefix":
      return typeof value.prefix === "string" && candidate.startsWith(value.prefix);
    case "suffix":
      return typeof value.suffix === "string" && candidate.endsWith(value.suffix);
    case "prefixSuffix":
      return (
        typeof value.prefix === "string" &&
        typeof value.suffix === "string" &&
        typeof value.minLength === "number" &&
        candidate.startsWith(value.prefix) &&
        candidate.endsWith(value.suffix) &&
        candidate.length >= value.minLength
      );
    case "charInclusion":
      return expressionValueMatchesCharConstraint(
        {
          valueDomainKind: "constrained",
          valueConstraintKind: "charInclusion",
          valueCharMust: typeof value.mustChars === "string" ? value.mustChars : "",
          valueCharMay: typeof value.mayChars === "string" ? value.mayChars : "",
          valueMayIncludeOtherChars: value.mayIncludeOtherChars === true,
        },
        candidate,
      );
    case "composite":
      return (
        (typeof value.prefix !== "string" || candidate.startsWith(value.prefix)) &&
        (typeof value.suffix !== "string" || candidate.endsWith(value.suffix)) &&
        (typeof value.minLength !== "number" || candidate.length >= value.minLength) &&
        expressionValueMatchesCharConstraint(
          {
            valueDomainKind: "constrained",
            valueConstraintKind: "charInclusion",
            valueCharMust: typeof value.mustChars === "string" ? value.mustChars : "",
            valueCharMay: typeof value.mayChars === "string" ? value.mayChars : "",
            valueMayIncludeOtherChars: value.mayIncludeOtherChars === true,
          },
          candidate,
        )
      );
    case "top":
      return true;
    default:
      return false;
  }
}

function normalizeExpressionValues(values: readonly string[]): readonly string[] {
  return [...new Set(values)].toSorted((a, b) => a.localeCompare(b));
}

function normalizeExpressionCharSet(value: string): string {
  return [...new Set(value)].toSorted((a, b) => a.localeCompare(b)).join("");
}

function intersectExpressionCharSets(left: string, right: string): string {
  const rightChars = new Set(right);
  return normalizeExpressionCharSet([...left].filter((char) => rightChars.has(char)).join(""));
}

function charInclusionFromExpressionFiniteValues(values: readonly string[]): {
  readonly mustChars: string;
  readonly mayChars: string;
} {
  const [first, ...rest] = values.map(normalizeExpressionCharSet);
  if (first === undefined) return { mustChars: "", mayChars: "" };

  return rest.reduce(
    (acc, value) => ({
      mustChars: intersectExpressionCharSets(acc.mustChars, value),
      mayChars: normalizeExpressionCharSet(`${acc.mayChars}${value}`),
    }),
    { mustChars: first, mayChars: first },
  );
}

function meaningfulExpressionLongestCommonPrefix(values: readonly string[]): string {
  const prefix = longestExpressionCommonPrefix(values);
  if (prefix.length === 0) return "";
  return isMeaningfulExpressionClassPrefix(prefix, values) ? prefix : "";
}

function meaningfulExpressionLongestCommonSuffix(values: readonly string[]): string {
  const suffix = longestExpressionCommonSuffix(values);
  if (suffix.length === 0) return "";
  return isMeaningfulExpressionClassSuffix(suffix, values) ? suffix : "";
}

function longestExpressionCommonPrefix(values: readonly string[]): string {
  const [first, ...rest] = values;
  if (first === undefined) return "";
  return rest.reduce((prefix, value) => {
    let matchLength = 0;
    while (
      matchLength < prefix.length &&
      matchLength < value.length &&
      prefix[matchLength] === value[matchLength]
    ) {
      matchLength++;
    }
    return prefix.slice(0, matchLength);
  }, first);
}

function longestExpressionCommonSuffix(values: readonly string[]): string {
  return [...longestExpressionCommonPrefix(values.map((value) => [...value].toReversed().join("")))]
    .toReversed()
    .join("");
}

function isMeaningfulExpressionClassPrefix(prefix: string, values: readonly string[]): boolean {
  if (prefix.length === 0) return false;
  if (isExpressionClassBoundaryChar(prefix[prefix.length - 1])) return true;
  return values.every(
    (value) =>
      value.length === prefix.length || isExpressionClassBoundaryChar(value[prefix.length]),
  );
}

function isMeaningfulExpressionClassSuffix(suffix: string, values: readonly string[]): boolean {
  if (suffix.length === 0) return false;
  if (isExpressionClassBoundaryChar(suffix[0])) return true;
  return values.every(
    (value) =>
      value.length === suffix.length ||
      isExpressionClassBoundaryChar(value[value.length - suffix.length - 1]),
  );
}

function isExpressionClassBoundaryChar(char: string | undefined): boolean {
  return char === "-" || char === "_" || char === ":";
}

function provenanceForExpressionAbstractValue(value: AbstractClassValueV0): string | undefined {
  return typeof value.provenance === "string" ? value.provenance : undefined;
}

function provenanceTreeRootOperation(
  value: AbstractClassValueV0,
  provenance: string | undefined,
): string {
  switch (provenance) {
    case "finiteSetWideningChars":
    case "finiteSetWideningComposite":
      return "finiteSetWidening";
    case "prefixJoinLcp":
      return "prefixJoinLongestCommonPrefix";
    case "suffixJoinLcs":
      return "suffixJoinLongestCommonSuffix";
    case "prefixSuffixJoin":
    case "compositeJoin":
      return "reducedProductJoin";
    case "compositeConcat":
      return "reducedProductConcat";
    case undefined:
      switch (value.kind) {
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
          return "unknownDomain";
      }
    default:
      return "constraintDomain";
  }
}

function provenanceTreeRootReason(
  value: AbstractClassValueV0,
  provenance: string | undefined,
): string {
  switch (provenance) {
    case "finiteSetWideningChars":
      return "large finite set widened to character constraints";
    case "finiteSetWideningComposite":
      return "large finite set widened to preserved edge and character constraints";
    case "prefixJoinLcp":
      return "branch merge retained the meaningful longest common prefix";
    case "suffixJoinLcs":
      return "branch merge retained the meaningful longest common suffix";
    case "prefixSuffixJoin":
    case "compositeJoin":
      return "reduced product combined compatible constraints from multiple domains";
    case "compositeConcat":
      return "reduced product concatenated compatible constraints without widening to top";
    case undefined:
      switch (value.kind) {
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
          return "the class value provenance is unknown";
      }
    default:
      return "the class value is represented by explicit domain constraints";
  }
}

function provenanceTreeRootDetail(value: AbstractClassValueV0): string | undefined {
  if (value.kind === "exact" && typeof value.value === "string") return `value=${value.value}`;
  if (value.kind === "finiteSet" && Array.isArray(value.values)) {
    return `valueCount=${value.values.length}`;
  }
  return undefined;
}

function provenanceTreeConstraintChildren(
  value: AbstractClassValueV0,
): readonly AbstractClassValueProvenanceNodeV0[] {
  const children: AbstractClassValueProvenanceNodeV0[] = [];

  if (value.kind === "prefix" && typeof value.prefix === "string") {
    children.push(provenanceConstraintNode("prefixConstraint", "prefix", value.prefix));
  }
  if (value.kind === "suffix" && typeof value.suffix === "string") {
    children.push(provenanceConstraintNode("suffixConstraint", "suffix", value.suffix));
  }
  if (value.kind === "prefixSuffix") {
    if (typeof value.prefix === "string") {
      children.push(provenanceConstraintNode("prefixConstraint", "prefix", value.prefix));
    }
    if (typeof value.suffix === "string") {
      children.push(provenanceConstraintNode("suffixConstraint", "suffix", value.suffix));
    }
    if (typeof value.minLength === "number") {
      children.push(
        provenanceConstraintNode("lengthConstraint", "minLength", String(value.minLength)),
      );
    }
  }
  if (value.kind === "charInclusion" || value.kind === "composite") {
    if (value.kind === "composite") {
      if (typeof value.prefix === "string") {
        children.push(provenanceConstraintNode("prefixConstraint", "prefix", value.prefix));
      }
      if (typeof value.suffix === "string") {
        children.push(provenanceConstraintNode("suffixConstraint", "suffix", value.suffix));
      }
      if (typeof value.minLength === "number") {
        children.push(
          provenanceConstraintNode("lengthConstraint", "minLength", String(value.minLength)),
        );
      }
    }
    pushProvenanceCharConstraintChildren(
      children,
      typeof value.mustChars === "string" ? value.mustChars : "",
      typeof value.mayChars === "string" ? value.mayChars : "",
      value.mayIncludeOtherChars === true,
    );
  }

  return children;
}

function pushProvenanceCharConstraintChildren(
  children: AbstractClassValueProvenanceNodeV0[],
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
): AbstractClassValueProvenanceNodeV0 {
  return {
    operation,
    resultKind: "constraint",
    detail: `${label}=${value}`,
    reason: "constraint retained by the abstract value domain",
    children: [],
  };
}

function deriveBaseExpressionClassValueKind(payload: ExpressionValueDomainPayloadLike): string {
  switch (payload.valueDomainKind) {
    case "none":
    case "unknown":
      return "top";
    case "exact":
      return payload.finiteValues?.length ? "exact" : "top";
    case "finiteSet":
      return finiteExpressionValueDomainKind(payload.finiteValues ?? []);
    case "constrained":
      return deriveConstraintExpressionClassValueKind(payload);
    default:
      return payload.valueDomainKind;
  }
}

function deriveConstraintExpressionClassValueKind(
  payload: ExpressionValueDomainPayloadLike,
): string {
  switch (payload.valueConstraintKind) {
    case "prefix":
      return "prefix";
    case "suffix":
      return "suffix";
    case "prefixSuffix":
      if (!payload.valuePrefix && !payload.valueSuffix) return "top";
      if (!payload.valuePrefix) return "suffix";
      if (!payload.valueSuffix) return "prefix";
      return "prefixSuffix";
    case "charInclusion":
      return deriveCharInclusionExpressionClassValueKind(payload);
    case "composite":
      return deriveCompositeExpressionClassValueKind(payload);
    default:
      return "top";
  }
}

function deriveCharInclusionExpressionClassValueKind(
  payload: ExpressionValueDomainPayloadLike,
): string {
  const hasMustChars = (payload.valueCharMust ?? "").length > 0;
  const hasMayChars = (payload.valueCharMay ?? "").length > 0 || hasMustChars;
  if (payload.valueMayIncludeOtherChars && !hasMustChars) return "top";
  if (!payload.valueMayIncludeOtherChars && !hasMayChars) return "top";
  return "charInclusion";
}

function deriveCompositeExpressionClassValueKind(
  payload: ExpressionValueDomainPayloadLike,
): string {
  const hasEdges = Boolean(payload.valuePrefix || payload.valueSuffix);
  const hasCharInfo =
    (payload.valueCharMust ?? "").length > 0 ||
    (!payload.valueMayIncludeOtherChars && (payload.valueCharMay ?? "").length > 0);

  if (!hasCharInfo)
    return deriveConstraintExpressionClassValueKind({
      ...payload,
      valueConstraintKind: "prefixSuffix",
    });
  if (!hasEdges) return deriveCharInclusionExpressionClassValueKind(payload);
  return "composite";
}

function hasExpressionValueConstraintDetails(payload: ExpressionValueDomainPayloadLike): boolean {
  return (
    payload.valueConstraintKind !== undefined ||
    payload.valuePrefix !== undefined ||
    payload.valueSuffix !== undefined ||
    payload.valueMinLen !== undefined ||
    payload.valueCharMust !== undefined ||
    payload.valueCharMay !== undefined ||
    payload.valueMayIncludeOtherChars !== undefined
  );
}

function finiteExpressionValueDomainKind(values: readonly string[]): string {
  const uniqueCount = new Set(values).size;
  if (uniqueCount === 0) return "bottom";
  if (uniqueCount === 1) return "exact";
  if (uniqueCount <= MAX_EXPECTED_FINITE_CLASS_VALUES) return "finiteSet";
  return "composite";
}

function expressionValueMatchesConstraint(
  payload: ExpressionValueDomainPayloadLike,
  value: string,
): boolean {
  switch (payload.valueConstraintKind) {
    case "prefix":
      return value.startsWith(payload.valuePrefix ?? "");
    case "suffix":
      return value.endsWith(payload.valueSuffix ?? "");
    case "prefixSuffix":
      return (
        value.startsWith(payload.valuePrefix ?? "") &&
        value.endsWith(payload.valueSuffix ?? "") &&
        value.length >= (payload.valueMinLen ?? 0)
      );
    case "charInclusion":
      return expressionValueMatchesCharConstraint(payload, value);
    case "composite":
      return (
        (!payload.valuePrefix || value.startsWith(payload.valuePrefix)) &&
        (!payload.valueSuffix || value.endsWith(payload.valueSuffix)) &&
        value.length >= (payload.valueMinLen ?? 0) &&
        expressionValueMatchesCharConstraint(payload, value)
      );
    default:
      return true;
  }
}

function expressionValueMatchesCharConstraint(
  payload: ExpressionValueDomainPayloadLike,
  value: string,
): boolean {
  const valueChars = new Set(value);
  for (const char of payload.valueCharMust ?? "") {
    if (!valueChars.has(char)) return false;
  }
  if (payload.valueMayIncludeOtherChars) return true;

  const mayChars = new Set([...(payload.valueCharMay ?? ""), ...(payload.valueCharMust ?? "")]);
  for (const char of valueChars) {
    if (!mayChars.has(char)) return false;
  }
  return true;
}

export function deriveTsExpressionDomainEvaluatorCandidates(
  snapshot: EngineParitySnapshotV2,
): ExpressionDomainEvaluatorCandidatesV0 {
  const results = snapshot.input.typeFacts
    .map((entry) => {
      const payload = expressionValueDomainPayloadFromInputFacts(entry.facts);
      return {
        kind: "expression-domain" as const,
        filePath: entry.filePath,
        queryId: entry.expressionId,
        payload: {
          expressionId: entry.expressionId,
          valueDomainKind: deriveReducedExpressionValueDomainKind(payload),
          ...(entry.facts.constraintKind !== undefined
            ? { valueConstraintKind: entry.facts.constraintKind }
            : {}),
          ...(entry.facts.prefix !== undefined ? { valuePrefix: entry.facts.prefix } : {}),
          ...(entry.facts.suffix !== undefined ? { valueSuffix: entry.facts.suffix } : {}),
          ...(entry.facts.minLen !== undefined ? { valueMinLen: entry.facts.minLen } : {}),
          ...(entry.facts.maxLen !== undefined ? { valueMaxLen: entry.facts.maxLen } : {}),
          ...(entry.facts.charMust !== undefined ? { valueCharMust: entry.facts.charMust } : {}),
          ...(entry.facts.charMay !== undefined ? { valueCharMay: entry.facts.charMay } : {}),
          ...(entry.facts.mayIncludeOtherChars !== undefined
            ? { valueMayIncludeOtherChars: entry.facts.mayIncludeOtherChars }
            : {}),
          finiteValueCount: entry.facts.values?.length ?? 0,
          valueDomainDerivation: deriveReducedExpressionValueDomainDerivation(payload),
          valueDomainProvenanceTree: deriveReducedExpressionValueDomainProvenanceTree(payload),
        },
      };
    })
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    results,
  };
}

export function deriveTsExpressionDomainCanonicalProducerSignal(
  snapshot: EngineParitySnapshotV2,
): ExpressionDomainCanonicalProducerSignalV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    canonicalBundle: deriveTsExpressionDomainCanonicalCandidateBundle(snapshot),
    evaluatorCandidates: deriveTsExpressionDomainEvaluatorCandidates(snapshot),
  };
}

export function deriveTsSelectorUsagePlanSummary(
  snapshot: EngineParitySnapshotV2,
): SelectorUsagePlanSummaryV0 {
  const canonicalSelectorNames: string[] = [];
  const viewKindCounts: Record<string, number> = {};
  const nestedSafetyCounts: Record<string, number> = {};
  let composedSelectorCount = 0;
  let totalComposesRefs = 0;

  for (const style of snapshot.input.styles) {
    for (const selector of style.document.selectors) {
      increment(viewKindCounts, selector.viewKind);
      increment(nestedSafetyCounts, selector.nestedSafety);
      if (selector.composes.length > 0) {
        composedSelectorCount += 1;
        totalComposesRefs += selector.composes.length;
      }
      if (selector.viewKind === "canonical") {
        canonicalSelectorNames.push(selector.canonicalName);
      }
    }
  }

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    canonicalSelectorNames,
    viewKindCounts,
    nestedSafetyCounts,
    composedSelectorCount,
    totalComposesRefs,
  };
}

export function deriveTsSelectorUsageFragments(
  snapshot: EngineParitySnapshotV2,
): SelectorUsageFragmentsV0 {
  const fragments: SelectorUsageFragmentV0[] = [];

  for (const style of snapshot.input.styles) {
    for (const [ordinal, selector] of style.document.selectors.entries()) {
      const fragment: SelectorUsageFragmentV0 = { ordinal, viewKind: selector.viewKind };
      if (selector.canonicalName) {
        fragment.canonicalName = selector.canonicalName;
      }
      if (selector.nestedSafety) {
        fragment.nestedSafety = selector.nestedSafety;
      }
      fragment.composesCount = selector.composes.length;
      fragments.push(fragment);
    }
  }

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments,
  };
}

export function deriveTsSelectorUsageQueryFragments(
  snapshot: EngineParitySnapshotV2,
): SelectorUsageQueryFragmentsV0 {
  const fragments: SelectorUsageQueryFragmentV0[] = [];

  for (const style of snapshot.input.styles) {
    for (const selector of style.document.selectors) {
      if (selector.viewKind !== "canonical") {
        continue;
      }
      const fragment: SelectorUsageQueryFragmentV0 = {
        queryId: selector.canonicalName,
        canonicalName: selector.canonicalName,
      };
      if (selector.nestedSafety) {
        fragment.nestedSafety = selector.nestedSafety;
      }
      fragment.composesCount = selector.composes.length;
      fragments.push(fragment);
    }
  }

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments: fragments.toSorted((a, b) => a.queryId.localeCompare(b.queryId)),
  };
}

export function deriveTsSourceResolutionPlanSummary(
  snapshot: EngineParitySnapshotV2,
): SourceResolutionPlanSummaryV0 {
  const plannedExpressionIds: string[] = [];
  const expressionKindCounts: Record<string, number> = {};
  const distinctStyleFilePaths = new Set<string>();
  let symbolRefWithBindingCount = 0;
  let styleAccessCount = 0;
  let styleAccessPathDepthSum = 0;

  for (const source of snapshot.input.sources) {
    for (const expression of source.document.classExpressions) {
      plannedExpressionIds.push(expression.id);
      increment(expressionKindCounts, expression.kind);
      distinctStyleFilePaths.add(expression.scssModulePath);
      if (expression.kind === "symbolRef" && expression.rootBindingDeclId) {
        symbolRefWithBindingCount += 1;
      }
      if (expression.kind === "styleAccess") {
        styleAccessCount += 1;
        styleAccessPathDepthSum += expression.accessPath.length;
      }
    }
  }

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    plannedExpressionIds,
    expressionKindCounts,
    distinctStyleFilePaths: [...distinctStyleFilePaths].toSorted((a, b) => a.localeCompare(b)),
    symbolRefWithBindingCount,
    styleAccessCount,
    styleAccessPathDepthSum,
  };
}

export function deriveTsSourceResolutionQueryFragments(
  snapshot: EngineParitySnapshotV2,
): SourceResolutionQueryFragmentsV0 {
  const fragments = snapshot.input.sources
    .flatMap((source) =>
      source.document.classExpressions.map((expression) => ({
        queryId: expression.id,
        expressionId: expression.id,
        expressionKind: expression.kind,
        styleFilePath: expression.scssModulePath,
      })),
    )
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments,
  };
}

type SourceClassExpressionInputV2 =
  EngineInputV2["sources"][number]["document"]["classExpressions"][number];
type StyleAnalysisInputV2 = EngineInputV2["styles"][number];
type StringTypeFactsInputV2 = EngineInputV2["typeFacts"][number]["facts"];
type TypeFactEntryInputV2 = EngineInputV2["typeFacts"][number];

interface SourceSideInputRows {
  readonly expression: SourceClassExpressionInputV2;
  readonly style: StyleAnalysisInputV2;
  readonly entry: TypeFactEntryInputV2;
  readonly selectorNames: readonly string[];
  readonly candidateNames: readonly string[];
  readonly finiteValues?: readonly string[];
  readonly selectorCertainty: string;
  readonly valueCertainty?: string;
  readonly selectorCertaintyShapeKind: string;
  readonly selectorCertaintyShapeLabel: string;
  readonly valueCertaintyShapeKind: string;
  readonly valueCertaintyShapeLabel: string;
}

function collectSourceSideInputRows(snapshot: EngineParitySnapshotV2): SourceSideInputRows[] {
  const expressionIndex = new Map<string, SourceClassExpressionInputV2>();
  const styleIndex = new Map<string, StyleAnalysisInputV2>();

  for (const source of snapshot.input.sources) {
    for (const expression of source.document.classExpressions) {
      expressionIndex.set(expression.id, expression);
    }
  }

  for (const style of snapshot.input.styles) {
    styleIndex.set(style.filePath, style);
  }

  const rows: SourceSideInputRows[] = [];
  for (const entry of snapshot.input.typeFacts) {
    const expression = expressionIndex.get(entry.expressionId);
    if (!expression) continue;

    const style = styleIndex.get(expression.scssModulePath);
    if (!style) continue;

    const selectorNames = resolveInputSelectorNames(style, entry.facts);
    const finiteValues = finiteValuesForInputFacts(entry.facts);
    const candidateNames = finiteValues ?? selectorNames;
    const selectorUniverseCount = canonicalInputSelectorCount(style);
    const selectorCertainty = mapInputSelectorCertainty(
      entry.facts,
      selectorNames.length,
      selectorUniverseCount,
    );

    const valueCertainty = mapInputValueCertainty(entry.facts);
    rows.push({
      expression,
      style,
      entry,
      selectorNames,
      candidateNames,
      ...(finiteValues !== undefined ? { finiteValues } : {}),
      selectorCertainty,
      ...(valueCertainty !== undefined ? { valueCertainty } : {}),
      selectorCertaintyShapeKind: mapInputSelectorCertaintyShapeKind(
        entry.facts,
        selectorNames.length,
        selectorUniverseCount,
      ),
      selectorCertaintyShapeLabel: mapInputSelectorCertaintyShapeLabel(
        entry.facts,
        selectorNames.length,
        selectorUniverseCount,
      ),
      valueCertaintyShapeKind: mapInputValueCertaintyShapeKind(entry.facts),
      valueCertaintyShapeLabel: mapInputValueCertaintyShapeLabel(entry.facts),
    });
  }

  return rows.toSorted((a, b) => a.entry.expressionId.localeCompare(b.entry.expressionId));
}

function expressionValueDomainPayloadFromInputFacts(
  facts: StringTypeFactsInputV2,
): ExpressionValueDomainPayloadLike {
  return {
    valueDomainKind: mapInputExpressionValueDomainKind(facts),
    ...(facts.constraintKind !== undefined ? { valueConstraintKind: facts.constraintKind } : {}),
    ...(facts.prefix !== undefined ? { valuePrefix: facts.prefix } : {}),
    ...(facts.suffix !== undefined ? { valueSuffix: facts.suffix } : {}),
    ...(facts.minLen !== undefined ? { valueMinLen: facts.minLen } : {}),
    ...(facts.maxLen !== undefined ? { valueMaxLen: facts.maxLen } : {}),
    ...(facts.charMust !== undefined ? { valueCharMust: facts.charMust } : {}),
    ...(facts.charMay !== undefined ? { valueCharMay: facts.charMay } : {}),
    ...(facts.mayIncludeOtherChars !== undefined
      ? { valueMayIncludeOtherChars: facts.mayIncludeOtherChars }
      : {}),
    finiteValues: facts.values ?? null,
  };
}

function mapInputExpressionValueDomainKind(facts: StringTypeFactsInputV2): string {
  return facts.kind === "unknown" ? "none" : facts.kind;
}

function mapInputValueCertainty(facts: StringTypeFactsInputV2): string | undefined {
  switch (facts.kind) {
    case "exact":
      return "exact";
    case "finiteSet":
    case "constrained":
      return "inferred";
    case "unknown":
    case "top":
      return "possible";
    default:
      return undefined;
  }
}

function mapInputValueCertaintyShapeKind(facts: StringTypeFactsInputV2): string {
  switch (facts.kind) {
    case "exact":
      return "exact";
    case "finiteSet":
      return "boundedFinite";
    case "constrained":
      return "constrained";
    default:
      return "unknown";
  }
}

function mapInputValueCertaintyShapeLabel(facts: StringTypeFactsInputV2): string {
  switch (mapInputValueCertainty(facts)) {
    case "exact":
      return "exact";
    case "inferred":
      if (facts.kind === "finiteSet") {
        return `bounded finite (${uniqueInputValueCount(facts.values)})`;
      }
      if (facts.kind === "constrained") {
        return constrainedInputValueShapeLabel(facts);
      }
      return "unknown";
    default:
      return "unknown";
  }
}

function mapInputSelectorCertainty(
  facts: StringTypeFactsInputV2,
  matchedSelectorCount: number,
  selectorUniverseCount: number,
): string {
  switch (facts.kind) {
    case "unknown":
      return "possible";
    case "exact":
      return matchedSelectorCount === 1 ? "exact" : "possible";
    case "finiteSet": {
      const finiteValueCount = uniqueInputValueCount(facts.values);
      if (finiteValueCount === 0 || matchedSelectorCount === 0) return "possible";
      return matchedSelectorCount === finiteValueCount ? "exact" : "inferred";
    }
    case "constrained":
    case "top":
      if (matchedSelectorCount === 0) return "possible";
      return matchedSelectorCount === selectorUniverseCount ? "exact" : "inferred";
    default:
      return "possible";
  }
}

function mapInputSelectorCertaintyShapeKind(
  facts: StringTypeFactsInputV2,
  matchedSelectorCount: number,
  selectorUniverseCount: number,
): string {
  switch (mapInputSelectorCertainty(facts, matchedSelectorCount, selectorUniverseCount)) {
    case "exact":
      return "exact";
    case "possible":
      return "unknown";
    case "inferred":
      return isConstrainedInputSelectorShape(facts) ? "constrained" : "boundedFinite";
    default:
      return "unknown";
  }
}

function mapInputSelectorCertaintyShapeLabel(
  facts: StringTypeFactsInputV2,
  matchedSelectorCount: number,
  selectorUniverseCount: number,
): string {
  switch (mapInputSelectorCertainty(facts, matchedSelectorCount, selectorUniverseCount)) {
    case "exact":
      return "exact";
    case "possible":
      return "unknown";
    case "inferred":
      switch (facts.constraintKind) {
        case "prefix":
          return `constrained prefix selector set (${matchedSelectorCount})`;
        case "suffix":
          return `constrained suffix selector set (${matchedSelectorCount})`;
        case "prefixSuffix":
          return `constrained edge selector set (${matchedSelectorCount})`;
        case "charInclusion":
          return `constrained character selector set (${matchedSelectorCount})`;
        case "composite":
          return `constrained composite selector set (${matchedSelectorCount})`;
        default:
          return `bounded selector set (${matchedSelectorCount})`;
      }
    default:
      return "unknown";
  }
}

function finiteValuesForInputFacts(facts: StringTypeFactsInputV2): readonly string[] | undefined {
  if (facts.kind !== "exact" && facts.kind !== "finiteSet") return undefined;
  return facts.values;
}

function resolveInputSelectorNames(
  style: StyleAnalysisInputV2,
  facts: StringTypeFactsInputV2,
): readonly string[] {
  switch (facts.kind) {
    case "unknown":
      return [];
    case "top":
      return canonicalInputSelectorNames(style);
    case "exact":
    case "finiteSet": {
      const names: string[] = [];
      for (const value of facts.values ?? []) {
        pushCanonicalInputMatch(style, value, names);
      }
      return names;
    }
    case "constrained":
      return resolveConstrainedInputSelectorNames(style, facts);
    default:
      return [];
  }
}

function resolveConstrainedInputSelectorNames(
  style: StyleAnalysisInputV2,
  facts: StringTypeFactsInputV2,
): readonly string[] {
  const names: string[] = [];
  for (const selector of style.document.selectors) {
    if (!matchesInputSelectorConstraints(selector.name, facts)) continue;
    const canonicalName = canonicalInputNameForSelector(style, selector.name);
    if (canonicalName && !names.includes(canonicalName)) {
      names.push(canonicalName);
    }
  }
  return names;
}

function matchesInputSelectorConstraints(
  selectorName: string,
  facts: StringTypeFactsInputV2,
): boolean {
  switch (facts.constraintKind) {
    case "prefix":
      return facts.prefix !== undefined && selectorName.startsWith(facts.prefix);
    case "suffix":
      return facts.suffix !== undefined && selectorName.endsWith(facts.suffix);
    case "prefixSuffix": {
      const prefixOk = facts.prefix === undefined || selectorName.startsWith(facts.prefix);
      const suffixOk = facts.suffix === undefined || selectorName.endsWith(facts.suffix);
      const minLenOk = facts.minLen === undefined || selectorName.length >= facts.minLen;
      const maxLenOk = facts.maxLen === undefined || selectorName.length <= facts.maxLen;
      return prefixOk && suffixOk && minLenOk && maxLenOk;
    }
    case "charInclusion":
      return matchesInputCharConstraints(
        selectorName,
        facts.charMust ?? "",
        facts.charMay ?? "",
        facts.mayIncludeOtherChars === true,
      );
    case "composite": {
      const prefixOk = facts.prefix === undefined || selectorName.startsWith(facts.prefix);
      const suffixOk = facts.suffix === undefined || selectorName.endsWith(facts.suffix);
      const minLenOk = facts.minLen === undefined || selectorName.length >= facts.minLen;
      const maxLenOk = facts.maxLen === undefined || selectorName.length <= facts.maxLen;
      return (
        prefixOk &&
        suffixOk &&
        minLenOk &&
        maxLenOk &&
        matchesInputCharConstraints(
          selectorName,
          facts.charMust ?? "",
          facts.charMay ?? "",
          facts.mayIncludeOtherChars === true,
        )
      );
    }
    default:
      return false;
  }
}

function matchesInputCharConstraints(
  value: string,
  mustChars: string,
  mayChars: string,
  mayIncludeOtherChars: boolean,
): boolean {
  const valueChars = new Set(Array.from(value));
  const mustSet = new Set(Array.from(mustChars));
  const maySet = new Set(Array.from(mayChars));

  for (const char of mustSet) {
    if (!valueChars.has(char)) return false;
  }

  if (!mayIncludeOtherChars) {
    for (const char of valueChars) {
      if (!maySet.has(char)) return false;
    }
  }

  return true;
}

function pushCanonicalInputMatch(
  style: StyleAnalysisInputV2,
  viewName: string,
  names: string[],
): void {
  const canonicalName = canonicalInputNameForSelector(style, viewName);
  if (canonicalName && !names.includes(canonicalName)) {
    names.push(canonicalName);
  }
}

function canonicalInputSelectorNames(style: StyleAnalysisInputV2): readonly string[] {
  const names: string[] = [];
  for (const selector of style.document.selectors) {
    if (
      selector.viewKind === "canonical" &&
      selector.canonicalName &&
      !names.includes(selector.canonicalName)
    ) {
      names.push(selector.canonicalName);
    }
  }
  return names;
}

function canonicalInputSelectorCount(style: StyleAnalysisInputV2): number {
  return canonicalInputSelectorNames(style).length;
}

function canonicalInputNameForSelector(
  style: StyleAnalysisInputV2,
  viewName: string,
): string | undefined {
  const matched = style.document.selectors.find((selector) => selector.name === viewName);
  if (!matched) return undefined;

  const canonical = style.document.selectors.find(
    (selector) =>
      selector.viewKind === "canonical" && selector.canonicalName === matched.canonicalName,
  );

  return canonical?.canonicalName ?? matched.canonicalName ?? matched.name;
}

function uniqueInputValueCount(values: readonly string[] | undefined): number {
  return new Set(values ?? []).size;
}

function constrainedInputValueShapeLabel(facts: StringTypeFactsInputV2): string {
  switch (facts.constraintKind) {
    case "prefix":
      return `constrained prefix \`${facts.prefix ?? ""}\``;
    case "suffix":
      return `constrained suffix \`${facts.suffix ?? ""}\``;
    case "prefixSuffix":
      return `constrained prefix \`${facts.prefix ?? ""}\` + suffix \`${facts.suffix ?? ""}\``;
    case "charInclusion":
      return `constrained character inclusion (${facts.charMust ?? "none"})`;
    case "composite":
      return "constrained composite";
    default:
      return "unknown";
  }
}

function isConstrainedInputSelectorShape(facts: StringTypeFactsInputV2): boolean {
  return (
    facts.constraintKind === "prefix" ||
    facts.constraintKind === "suffix" ||
    facts.constraintKind === "prefixSuffix" ||
    facts.constraintKind === "charInclusion" ||
    facts.constraintKind === "composite"
  );
}

export function deriveTsExpressionSemanticsFragments(
  snapshot: EngineParitySnapshotV2,
): ExpressionSemanticsFragmentsV0 {
  const fragments = collectSourceSideInputRows(snapshot)
    .map((row) => {
      const fragment: ExpressionSemanticsFragmentV0 = {
        queryId: row.entry.expressionId,
        expressionId: row.entry.expressionId,
        expressionKind: row.expression.kind,
        styleFilePath: row.expression.scssModulePath,
        valueDomainKind: mapInputExpressionValueDomainKind(row.entry.facts),
      };
      if (row.entry.facts.constraintKind !== undefined) {
        fragment.valueConstraintKind = row.entry.facts.constraintKind;
      }
      if (row.entry.facts.prefix !== undefined) {
        fragment.valuePrefix = row.entry.facts.prefix;
      }
      if (row.entry.facts.suffix !== undefined) {
        fragment.valueSuffix = row.entry.facts.suffix;
      }
      if (row.entry.facts.minLen !== undefined) {
        fragment.valueMinLen = row.entry.facts.minLen;
      }
      if (row.entry.facts.maxLen !== undefined) {
        fragment.valueMaxLen = row.entry.facts.maxLen;
      }
      if (row.entry.facts.charMust !== undefined) {
        fragment.valueCharMust = row.entry.facts.charMust;
      }
      if (row.entry.facts.charMay !== undefined) {
        fragment.valueCharMay = row.entry.facts.charMay;
      }
      if (row.entry.facts.mayIncludeOtherChars !== undefined) {
        fragment.valueMayIncludeOtherChars = row.entry.facts.mayIncludeOtherChars;
      }
      return fragment;
    })
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments,
  };
}

export function deriveTsExpressionSemanticsQueryFragments(
  snapshot: EngineParitySnapshotV2,
): ExpressionSemanticsQueryFragmentsV0 {
  const fragments = snapshot.input.sources
    .flatMap((source) =>
      source.document.classExpressions.map((expression) => ({
        queryId: expression.id,
        expressionId: expression.id,
        expressionKind: expression.kind,
        styleFilePath: expression.scssModulePath,
      })),
    )
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments,
  };
}

export function deriveTsExpressionSemanticsMatchFragments(
  snapshot: EngineParitySnapshotV2,
): ExpressionSemanticsMatchFragmentsV0 {
  const fragments = collectSourceSideInputRows(snapshot)
    .map((row) => {
      const fragment: ExpressionSemanticsMatchFragmentV0 = {
        queryId: row.entry.expressionId,
        expressionId: row.entry.expressionId,
        styleFilePath: row.expression.scssModulePath,
        selectorNames: row.selectorNames,
        candidateNames: row.candidateNames,
      };
      if (row.finiteValues !== undefined) {
        fragment.finiteValues = row.finiteValues;
      }
      return fragment;
    })
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments,
  };
}

export function deriveTsExpressionSemanticsCandidates(
  snapshot: EngineParitySnapshotV2,
): ExpressionSemanticsCandidatesV0 {
  const candidates = collectSourceSideInputRows(snapshot)
    .map((row) => {
      const candidate: ExpressionSemanticsCandidateV0 = {
        queryId: row.entry.expressionId,
        expressionId: row.entry.expressionId,
        expressionKind: row.expression.kind,
        styleFilePath: row.expression.scssModulePath,
        selectorNames: row.selectorNames,
        candidateNames: row.candidateNames,
        valueDomainKind: mapInputExpressionValueDomainKind(row.entry.facts),
        selectorCertainty: row.selectorCertainty,
        selectorCertaintyShapeKind: row.selectorCertaintyShapeKind,
        selectorCertaintyShapeLabel: row.selectorCertaintyShapeLabel,
        valueCertaintyShapeKind: row.valueCertaintyShapeKind,
        valueCertaintyShapeLabel: row.valueCertaintyShapeLabel,
      };
      if (row.finiteValues !== undefined) {
        candidate.finiteValues = row.finiteValues;
      }
      if (row.valueCertainty !== undefined) {
        candidate.valueCertainty = row.valueCertainty;
      }
      if (row.entry.facts.constraintKind !== undefined) {
        candidate.selectorConstraintKind = row.entry.facts.constraintKind;
        candidate.valueCertaintyConstraintKind = row.entry.facts.constraintKind;
        candidate.valueConstraintKind = row.entry.facts.constraintKind;
      }
      if (row.entry.facts.prefix !== undefined) {
        candidate.valuePrefix = row.entry.facts.prefix;
      }
      if (row.entry.facts.suffix !== undefined) {
        candidate.valueSuffix = row.entry.facts.suffix;
      }
      if (row.entry.facts.minLen !== undefined) {
        candidate.valueMinLen = row.entry.facts.minLen;
      }
      if (row.entry.facts.maxLen !== undefined) {
        candidate.valueMaxLen = row.entry.facts.maxLen;
      }
      if (row.entry.facts.charMust !== undefined) {
        candidate.valueCharMust = row.entry.facts.charMust;
      }
      if (row.entry.facts.charMay !== undefined) {
        candidate.valueCharMay = row.entry.facts.charMay;
      }
      if (row.entry.facts.mayIncludeOtherChars !== undefined) {
        candidate.valueMayIncludeOtherChars = row.entry.facts.mayIncludeOtherChars;
      }
      return candidate;
    })
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    candidates,
  };
}

export function deriveTsExpressionSemanticsCanonicalCandidateBundle(
  snapshot: EngineParitySnapshotV2,
): ExpressionSemanticsCanonicalCandidateBundleV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    queryFragments: deriveTsExpressionSemanticsQueryFragments(snapshot).fragments,
    fragments: deriveTsExpressionSemanticsFragments(snapshot).fragments,
    matchFragments: deriveTsExpressionSemanticsMatchFragments(snapshot).fragments,
    candidates: deriveTsExpressionSemanticsCandidates(snapshot).candidates,
  };
}

export function deriveTsExpressionSemanticsEvaluatorCandidates(
  snapshot: EngineParitySnapshotV2,
): ExpressionSemanticsEvaluatorCandidatesV0 {
  const results = collectSourceSideInputRows(snapshot)
    .map((row) => {
      const valueDomainPayload = expressionValueDomainPayloadFromInputFacts(row.entry.facts);
      return {
        kind: "expression-semantics" as const,
        filePath: row.entry.filePath,
        queryId: row.entry.expressionId,
        payload: {
          expressionId: row.entry.expressionId,
          expressionKind: row.expression.kind,
          styleFilePath: row.expression.scssModulePath,
          selectorNames: row.selectorNames,
          candidateNames: row.candidateNames,
          ...(row.finiteValues !== undefined ? { finiteValues: row.finiteValues } : {}),
          valueDomainKind: mapInputExpressionValueDomainKind(row.entry.facts),
          selectorCertainty: row.selectorCertainty,
          ...(row.valueCertainty !== undefined ? { valueCertainty: row.valueCertainty } : {}),
          selectorCertaintyShapeKind: row.selectorCertaintyShapeKind,
          selectorCertaintyShapeLabel: row.selectorCertaintyShapeLabel,
          valueCertaintyShapeKind: row.valueCertaintyShapeKind,
          valueCertaintyShapeLabel: row.valueCertaintyShapeLabel,
          ...(row.entry.facts.constraintKind !== undefined
            ? { selectorConstraintKind: row.entry.facts.constraintKind }
            : {}),
          ...(row.entry.facts.constraintKind !== undefined
            ? { valueCertaintyConstraintKind: row.entry.facts.constraintKind }
            : {}),
          ...(row.entry.facts.constraintKind !== undefined
            ? { valueConstraintKind: row.entry.facts.constraintKind }
            : {}),
          ...(row.entry.facts.prefix !== undefined ? { valuePrefix: row.entry.facts.prefix } : {}),
          ...(row.entry.facts.suffix !== undefined ? { valueSuffix: row.entry.facts.suffix } : {}),
          ...(row.entry.facts.minLen !== undefined ? { valueMinLen: row.entry.facts.minLen } : {}),
          ...(row.entry.facts.maxLen !== undefined ? { valueMaxLen: row.entry.facts.maxLen } : {}),
          ...(row.entry.facts.charMust !== undefined
            ? { valueCharMust: row.entry.facts.charMust }
            : {}),
          ...(row.entry.facts.charMay !== undefined
            ? { valueCharMay: row.entry.facts.charMay }
            : {}),
          ...(row.entry.facts.mayIncludeOtherChars !== undefined
            ? { valueMayIncludeOtherChars: row.entry.facts.mayIncludeOtherChars }
            : {}),
          valueDomainDerivation: deriveReducedExpressionValueDomainDerivation(valueDomainPayload),
          valueDomainProvenanceTree:
            deriveReducedExpressionValueDomainProvenanceTree(valueDomainPayload),
        },
      };
    })
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    results,
  };
}

export function deriveTsExpressionSemanticsCanonicalProducerSignal(
  snapshot: EngineParitySnapshotV2,
): ExpressionSemanticsCanonicalProducerSignalV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    canonicalBundle: deriveTsExpressionSemanticsCanonicalCandidateBundle(snapshot),
    evaluatorCandidates: deriveTsExpressionSemanticsEvaluatorCandidates(snapshot),
  };
}

export function deriveTsSourceResolutionFragments(
  snapshot: EngineParitySnapshotV2,
): SourceResolutionFragmentsV0 {
  const fragments = collectSourceSideInputRows(snapshot)
    .map((row) => {
      const fragment: SourceResolutionFragmentV0 = {
        queryId: row.entry.expressionId,
        expressionId: row.entry.expressionId,
        styleFilePath: row.expression.scssModulePath,
        valueCertaintyShapeKind: row.valueCertaintyShapeKind,
      };
      if (row.entry.facts.constraintKind !== undefined) {
        fragment.valueCertaintyConstraintKind = row.entry.facts.constraintKind;
      }
      if (row.entry.facts.prefix !== undefined) {
        fragment.valuePrefix = row.entry.facts.prefix;
      }
      if (row.entry.facts.suffix !== undefined) {
        fragment.valueSuffix = row.entry.facts.suffix;
      }
      if (row.entry.facts.minLen !== undefined) {
        fragment.valueMinLen = row.entry.facts.minLen;
      }
      if (row.entry.facts.maxLen !== undefined) {
        fragment.valueMaxLen = row.entry.facts.maxLen;
      }
      if (row.entry.facts.charMust !== undefined) {
        fragment.valueCharMust = row.entry.facts.charMust;
      }
      if (row.entry.facts.charMay !== undefined) {
        fragment.valueCharMay = row.entry.facts.charMay;
      }
      if (row.entry.facts.mayIncludeOtherChars !== undefined) {
        fragment.valueMayIncludeOtherChars = row.entry.facts.mayIncludeOtherChars;
      }
      return fragment;
    })
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments,
  };
}

export function deriveTsSourceResolutionMatchFragments(
  snapshot: EngineParitySnapshotV2,
): SourceResolutionMatchFragmentsV0 {
  const fragments = collectSourceSideInputRows(snapshot)
    .map((row) => {
      const fragment: SourceResolutionMatchFragmentV0 = {
        queryId: row.entry.expressionId,
        expressionId: row.entry.expressionId,
        styleFilePath: row.expression.scssModulePath,
        selectorNames: row.selectorNames,
      };
      if (row.finiteValues !== undefined) {
        fragment.finiteValues = row.finiteValues;
      }
      return fragment;
    })
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    fragments,
  };
}

export function deriveTsSourceResolutionCandidates(
  snapshot: EngineParitySnapshotV2,
): SourceResolutionCandidatesV0 {
  const candidates = collectSourceSideInputRows(snapshot)
    .map((row) => {
      const candidate: SourceResolutionCandidateV0 = {
        queryId: row.entry.expressionId,
        expressionId: row.entry.expressionId,
        styleFilePath: row.expression.scssModulePath,
        selectorNames: row.selectorNames,
        selectorCertainty: row.selectorCertainty,
        selectorCertaintyShapeKind: row.selectorCertaintyShapeKind,
        selectorCertaintyShapeLabel: row.selectorCertaintyShapeLabel,
        valueCertaintyShapeKind: row.valueCertaintyShapeKind,
        valueCertaintyShapeLabel: row.valueCertaintyShapeLabel,
      };
      if (row.finiteValues !== undefined) {
        candidate.finiteValues = row.finiteValues;
      }
      if (row.valueCertainty !== undefined) {
        candidate.valueCertainty = row.valueCertainty;
      }
      if (row.entry.facts.constraintKind !== undefined) {
        candidate.selectorConstraintKind = row.entry.facts.constraintKind;
        candidate.valueCertaintyConstraintKind = row.entry.facts.constraintKind;
      }
      if (row.entry.facts.prefix !== undefined) {
        candidate.valuePrefix = row.entry.facts.prefix;
      }
      if (row.entry.facts.suffix !== undefined) {
        candidate.valueSuffix = row.entry.facts.suffix;
      }
      if (row.entry.facts.minLen !== undefined) {
        candidate.valueMinLen = row.entry.facts.minLen;
      }
      if (row.entry.facts.maxLen !== undefined) {
        candidate.valueMaxLen = row.entry.facts.maxLen;
      }
      if (row.entry.facts.charMust !== undefined) {
        candidate.valueCharMust = row.entry.facts.charMust;
      }
      if (row.entry.facts.charMay !== undefined) {
        candidate.valueCharMay = row.entry.facts.charMay;
      }
      if (row.entry.facts.mayIncludeOtherChars !== undefined) {
        candidate.valueMayIncludeOtherChars = row.entry.facts.mayIncludeOtherChars;
      }
      return candidate;
    })
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    candidates,
  };
}

export function deriveTsSourceResolutionEvaluatorCandidates(
  snapshot: EngineParitySnapshotV2,
): SourceResolutionEvaluatorCandidatesV0 {
  const results = collectSourceSideInputRows(snapshot)
    .map((row) => ({
      kind: "source-expression-resolution" as const,
      filePath: row.entry.filePath,
      queryId: row.entry.expressionId,
      payload: {
        expressionId: row.entry.expressionId,
        styleFilePath: row.expression.scssModulePath,
        selectorNames: row.selectorNames,
        ...(row.finiteValues !== undefined ? { finiteValues: row.finiteValues } : {}),
        selectorCertainty: row.selectorCertainty,
        ...(row.valueCertainty !== undefined ? { valueCertainty: row.valueCertainty } : {}),
        selectorCertaintyShapeKind: row.selectorCertaintyShapeKind,
        selectorCertaintyShapeLabel: row.selectorCertaintyShapeLabel,
        valueCertaintyShapeKind: row.valueCertaintyShapeKind,
        valueCertaintyShapeLabel: row.valueCertaintyShapeLabel,
        ...(row.entry.facts.constraintKind !== undefined
          ? { selectorConstraintKind: row.entry.facts.constraintKind }
          : {}),
        ...(row.entry.facts.constraintKind !== undefined
          ? { valueCertaintyConstraintKind: row.entry.facts.constraintKind }
          : {}),
        ...(row.entry.facts.prefix !== undefined ? { valuePrefix: row.entry.facts.prefix } : {}),
        ...(row.entry.facts.suffix !== undefined ? { valueSuffix: row.entry.facts.suffix } : {}),
        ...(row.entry.facts.minLen !== undefined ? { valueMinLen: row.entry.facts.minLen } : {}),
        ...(row.entry.facts.maxLen !== undefined ? { valueMaxLen: row.entry.facts.maxLen } : {}),
        ...(row.entry.facts.charMust !== undefined
          ? { valueCharMust: row.entry.facts.charMust }
          : {}),
        ...(row.entry.facts.charMay !== undefined ? { valueCharMay: row.entry.facts.charMay } : {}),
        ...(row.entry.facts.mayIncludeOtherChars !== undefined
          ? { valueMayIncludeOtherChars: row.entry.facts.mayIncludeOtherChars }
          : {}),
      },
    }))
    .toSorted((a, b) => a.queryId.localeCompare(b.queryId));

  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    results,
  };
}

export function deriveTsSourceResolutionCanonicalCandidateBundle(
  snapshot: EngineParitySnapshotV2,
): SourceResolutionCanonicalCandidateBundleV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    queryFragments: deriveTsSourceResolutionQueryFragments(snapshot).fragments,
    fragments: deriveTsSourceResolutionFragments(snapshot).fragments,
    matchFragments: deriveTsSourceResolutionMatchFragments(snapshot).fragments,
    candidates: deriveTsSourceResolutionCandidates(snapshot).candidates,
  };
}

export function deriveTsSourceResolutionCanonicalProducerSignal(
  snapshot: EngineParitySnapshotV2,
): SourceResolutionCanonicalProducerSignalV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    canonicalBundle: deriveTsSourceResolutionCanonicalCandidateBundle(snapshot),
    evaluatorCandidates: deriveTsSourceResolutionEvaluatorCandidates(snapshot),
  };
}

export function deriveTsSourceSideCanonicalCandidateBundle(
  snapshot: EngineParitySnapshotV2,
): SourceSideCanonicalCandidateBundleV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    expressionSemantics: deriveTsExpressionSemanticsCanonicalCandidateBundle(snapshot),
    sourceResolution: deriveTsSourceResolutionCanonicalCandidateBundle(snapshot),
  };
}

export function deriveTsSourceSideEvaluatorCandidates(
  snapshot: EngineParitySnapshotV2,
): SourceSideEvaluatorCandidatesV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    expressionSemantics: deriveTsExpressionSemanticsEvaluatorCandidates(snapshot),
    sourceResolution: deriveTsSourceResolutionEvaluatorCandidates(snapshot),
  };
}

export function deriveTsSourceSideCanonicalProducerSignal(
  snapshot: EngineParitySnapshotV2,
): SourceSideCanonicalProducerSignalV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    canonicalBundle: deriveTsSourceSideCanonicalCandidateBundle(snapshot),
    evaluatorCandidates: deriveTsSourceSideEvaluatorCandidates(snapshot),
  };
}

export function deriveTsSemanticCanonicalCandidateBundle(
  snapshot: EngineParitySnapshotV2,
): SemanticCanonicalCandidateBundleV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    sourceSide: deriveTsSourceSideCanonicalCandidateBundle(snapshot),
    expressionDomain: deriveTsExpressionDomainCanonicalCandidateBundle(snapshot),
  };
}

export function deriveTsSemanticEvaluatorCandidates(
  snapshot: EngineParitySnapshotV2,
): SemanticEvaluatorCandidatesV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    sourceSide: deriveTsSourceSideEvaluatorCandidates(snapshot),
    expressionDomain: deriveTsExpressionDomainEvaluatorCandidates(snapshot),
  };
}

export function deriveTsSemanticCanonicalProducerSignal(
  snapshot: EngineParitySnapshotV2,
): SemanticCanonicalProducerSignalV0 {
  return {
    schemaVersion: "0",
    inputVersion: snapshot.input.version,
    canonicalBundle: deriveTsSemanticCanonicalCandidateBundle(snapshot),
    evaluatorCandidates: deriveTsSemanticEvaluatorCandidates(snapshot),
  };
}

export function assertShadowSummaryMatch(
  label: string,
  actual: ShadowSummaryV0,
  expected: ShadowSummaryV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertEqualField(label, "sourceCount", actual.sourceCount, expected.sourceCount);
  assertEqualField(label, "styleCount", actual.styleCount, expected.styleCount);
  assertEqualField(label, "typeFactCount", actual.typeFactCount, expected.typeFactCount);
  assertEqualField(
    label,
    "distinctFactFiles",
    actual.distinctFactFiles,
    expected.distinctFactFiles,
  );
  assertEqualField(label, "finiteValueCount", actual.finiteValueCount, expected.finiteValueCount);
  assertEqualField(label, "queryResultCount", actual.queryResultCount, expected.queryResultCount);
  assertEqualField(
    label,
    "selectorUsageReferencedCount",
    actual.selectorUsageReferencedCount,
    expected.selectorUsageReferencedCount,
  );
  assertEqualField(
    label,
    "selectorUsageUnreferencedCount",
    actual.selectorUsageUnreferencedCount,
    expected.selectorUsageUnreferencedCount,
  );
  assertEqualField(
    label,
    "selectorUsageTotalReferences",
    actual.selectorUsageTotalReferences,
    expected.selectorUsageTotalReferences,
  );
  assertEqualField(
    label,
    "selectorUsageDirectReferences",
    actual.selectorUsageDirectReferences,
    expected.selectorUsageDirectReferences,
  );
  assertEqualField(
    label,
    "selectorUsageEditableDirectReferences",
    actual.selectorUsageEditableDirectReferences,
    expected.selectorUsageEditableDirectReferences,
  );
  assertEqualField(
    label,
    "selectorUsageExactReferences",
    actual.selectorUsageExactReferences,
    expected.selectorUsageExactReferences,
  );
  assertEqualField(
    label,
    "selectorUsageInferredOrBetterReferences",
    actual.selectorUsageInferredOrBetterReferences,
    expected.selectorUsageInferredOrBetterReferences,
  );
  assertEqualField(
    label,
    "selectorUsageExpandedCount",
    actual.selectorUsageExpandedCount,
    expected.selectorUsageExpandedCount,
  );
  assertEqualField(
    label,
    "selectorUsageStyleDependencyCount",
    actual.selectorUsageStyleDependencyCount,
    expected.selectorUsageStyleDependencyCount,
  );
  assertEqualField(
    label,
    "expectedExpressionSemanticsCount",
    actual.expectedExpressionSemanticsCount,
    expected.expectedExpressionSemanticsCount,
  );
  assertEqualField(
    label,
    "expectedSourceExpressionResolutionCount",
    actual.expectedSourceExpressionResolutionCount,
    expected.expectedSourceExpressionResolutionCount,
  );
  assertEqualField(
    label,
    "expectedSelectorUsageCount",
    actual.expectedSelectorUsageCount,
    expected.expectedSelectorUsageCount,
  );
  assertEqualField(
    label,
    "expectedTotalQueryCount",
    actual.expectedTotalQueryCount,
    expected.expectedTotalQueryCount,
  );
  assertEqualField(
    label,
    "matchedExpressionQueryPairs",
    actual.matchedExpressionQueryPairs,
    expected.matchedExpressionQueryPairs,
  );
  assertEqualField(
    label,
    "missingExpressionSemanticsCount",
    actual.missingExpressionSemanticsCount,
    expected.missingExpressionSemanticsCount,
  );
  assertEqualField(
    label,
    "missingSourceExpressionResolutionCount",
    actual.missingSourceExpressionResolutionCount,
    expected.missingSourceExpressionResolutionCount,
  );
  assertEqualField(
    label,
    "unexpectedExpressionSemanticsCount",
    actual.unexpectedExpressionSemanticsCount,
    expected.unexpectedExpressionSemanticsCount,
  );
  assertEqualField(
    label,
    "unexpectedSourceExpressionResolutionCount",
    actual.unexpectedSourceExpressionResolutionCount,
    expected.unexpectedSourceExpressionResolutionCount,
  );
  assertEqualField(
    label,
    "matchedSelectorUsageCount",
    actual.matchedSelectorUsageCount,
    expected.matchedSelectorUsageCount,
  );
  assertEqualField(
    label,
    "missingSelectorUsageCount",
    actual.missingSelectorUsageCount,
    expected.missingSelectorUsageCount,
  );
  assertEqualField(
    label,
    "unexpectedSelectorUsageCount",
    actual.unexpectedSelectorUsageCount,
    expected.unexpectedSelectorUsageCount,
  );
  assertEqualField(label, "rewritePlanCount", actual.rewritePlanCount, expected.rewritePlanCount);
  assertEqualField(
    label,
    "checkerWarningCount",
    actual.checkerWarningCount,
    expected.checkerWarningCount,
  );
  assertEqualField(label, "checkerHintCount", actual.checkerHintCount, expected.checkerHintCount);
  assertEqualField(
    label,
    "checkerTotalFindings",
    actual.checkerTotalFindings,
    expected.checkerTotalFindings,
  );
  assertRecordEqual(label, "byKind", actual.byKind, expected.byKind);
  assertRecordEqual(label, "constrainedKinds", actual.constrainedKinds, expected.constrainedKinds);
  assertRecordEqual(label, "queryKindCounts", actual.queryKindCounts, expected.queryKindCounts);
  assertRecordEqual(
    label,
    "expressionValueDomainKinds",
    actual.expressionValueDomainKinds,
    expected.expressionValueDomainKinds,
  );
  assertRecordEqual(
    label,
    "expressionValueConstraintKinds",
    actual.expressionValueConstraintKinds,
    expected.expressionValueConstraintKinds,
  );
  assertConstraintDetailEqual(
    label,
    "expressionConstraintDetailCounts",
    actual.expressionConstraintDetailCounts,
    expected.expressionConstraintDetailCounts,
  );
  assertRecordEqual(
    label,
    "expressionValueCertaintyShapes",
    actual.expressionValueCertaintyShapes,
    expected.expressionValueCertaintyShapes,
  );
  assertRecordEqual(
    label,
    "expressionSelectorCertaintyShapes",
    actual.expressionSelectorCertaintyShapes,
    expected.expressionSelectorCertaintyShapes,
  );
  assertRecordEqual(
    label,
    "resolutionValueConstraintKinds",
    actual.resolutionValueConstraintKinds,
    expected.resolutionValueConstraintKinds,
  );
  assertConstraintDetailEqual(
    label,
    "resolutionConstraintDetailCounts",
    actual.resolutionConstraintDetailCounts,
    expected.resolutionConstraintDetailCounts,
  );
  assertRecordEqual(
    label,
    "resolutionValueCertaintyShapes",
    actual.resolutionValueCertaintyShapes,
    expected.resolutionValueCertaintyShapes,
  );
  assertRecordEqual(
    label,
    "resolutionSelectorCertaintyShapes",
    actual.resolutionSelectorCertaintyShapes,
    expected.resolutionSelectorCertaintyShapes,
  );
}

export function assertQueryPlanSummaryMatch(
  label: string,
  actual: QueryPlanSummaryV0,
  expected: QueryPlanSummaryV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertEqualField(label, "totalQueryCount", actual.totalQueryCount, expected.totalQueryCount);
  assertArrayEqual(
    label,
    "expressionSemanticsIds",
    actual.expressionSemanticsIds,
    expected.expressionSemanticsIds,
  );
  assertArrayEqual(
    label,
    "sourceExpressionResolutionIds",
    actual.sourceExpressionResolutionIds,
    expected.sourceExpressionResolutionIds,
  );
  assertArrayEqual(label, "selectorUsageIds", actual.selectorUsageIds, expected.selectorUsageIds);
}

export function assertExpressionDomainPlanSummaryMatch(
  label: string,
  actual: ExpressionDomainPlanSummaryV0,
  expected: ExpressionDomainPlanSummaryV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertEqualField(label, "finiteValueCount", actual.finiteValueCount, expected.finiteValueCount);
  assertArrayEqual(
    label,
    "plannedExpressionIds",
    actual.plannedExpressionIds,
    expected.plannedExpressionIds,
  );
  assertRecordEqual(label, "valueDomainKinds", actual.valueDomainKinds, expected.valueDomainKinds);
  assertRecordEqual(
    label,
    "valueConstraintKinds",
    actual.valueConstraintKinds,
    expected.valueConstraintKinds,
  );
  assertConstraintDetailEqual(
    label,
    "constraintDetailCounts",
    actual.constraintDetailCounts,
    expected.constraintDetailCounts,
  );
}

export function assertExpressionDomainFragmentsMatch(
  label: string,
  actual: ExpressionDomainFragmentsV0,
  expected: ExpressionDomainFragmentsV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = stableJson(actual.fragments);
  const expectedJson = stableJson(expected.fragments);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: expressionDomainFragments mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertExpressionDomainCandidatesMatch(
  label: string,
  actual: ExpressionDomainCandidatesV0,
  expected: ExpressionDomainCandidatesV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = stableJson(actual.candidates);
  const expectedJson = stableJson(expected.candidates);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: expressionDomainCandidates mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertExpressionDomainCanonicalCandidateBundleMatch(
  label: string,
  actual: ExpressionDomainCanonicalCandidateBundleV0,
  expected: ExpressionDomainCanonicalCandidateBundleV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertExpressionDomainPlanSummaryMatch(
    `${label}:planSummary`,
    actual.planSummary,
    expected.planSummary,
  );
  if (stableJson(actual.fragments) !== stableJson(expected.fragments)) {
    throw new Error(
      `${label}: expressionDomainCanonicalCandidate fragments mismatch\nactual=${stableJson(actual.fragments)}\nexpected=${stableJson(expected.fragments)}`,
    );
  }
  if (stableJson(actual.candidates) !== stableJson(expected.candidates)) {
    throw new Error(
      `${label}: expressionDomainCanonicalCandidate candidates mismatch\nactual=${stableJson(actual.candidates)}\nexpected=${stableJson(expected.candidates)}`,
    );
  }
}

export function assertExpressionDomainCanonicalProducerSignalMatch(
  label: string,
  actual: ExpressionDomainCanonicalProducerSignalV0,
  expected: ExpressionDomainCanonicalProducerSignalV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertExpressionDomainCanonicalCandidateBundleMatch(
    `${label}:canonicalBundle`,
    actual.canonicalBundle,
    expected.canonicalBundle,
  );
  if (stableJson(actual.evaluatorCandidates) !== stableJson(expected.evaluatorCandidates)) {
    throw new Error(
      `${label}: expressionDomainEvaluatorCandidates mismatch\nactual=${stableJson(actual.evaluatorCandidates)}\nexpected=${stableJson(expected.evaluatorCandidates)}`,
    );
  }
}

export function assertSelectorUsagePlanSummaryMatch(
  label: string,
  actual: SelectorUsagePlanSummaryV0,
  expected: SelectorUsagePlanSummaryV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertEqualField(
    label,
    "composedSelectorCount",
    actual.composedSelectorCount,
    expected.composedSelectorCount,
  );
  assertEqualField(
    label,
    "totalComposesRefs",
    actual.totalComposesRefs,
    expected.totalComposesRefs,
  );
  assertArrayEqual(
    label,
    "canonicalSelectorNames",
    actual.canonicalSelectorNames,
    expected.canonicalSelectorNames,
  );
  assertRecordEqual(label, "viewKindCounts", actual.viewKindCounts, expected.viewKindCounts);
  assertRecordEqual(
    label,
    "nestedSafetyCounts",
    actual.nestedSafetyCounts,
    expected.nestedSafetyCounts,
  );
}

export function assertSelectorUsageFragmentsMatch(
  label: string,
  actual: SelectorUsageFragmentsV0,
  expected: SelectorUsageFragmentsV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = stableJson(actual.fragments);
  const expectedJson = stableJson(expected.fragments);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: selectorUsageFragments mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertSelectorUsageQueryFragmentsMatch(
  label: string,
  actual: SelectorUsageQueryFragmentsV0,
  expected: SelectorUsageQueryFragmentsV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = stableJson(actual.fragments);
  const expectedJson = stableJson(expected.fragments);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: selectorUsageQueryFragments mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertSourceResolutionPlanSummaryMatch(
  label: string,
  actual: SourceResolutionPlanSummaryV0,
  expected: SourceResolutionPlanSummaryV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertEqualField(
    label,
    "symbolRefWithBindingCount",
    actual.symbolRefWithBindingCount,
    expected.symbolRefWithBindingCount,
  );
  assertEqualField(label, "styleAccessCount", actual.styleAccessCount, expected.styleAccessCount);
  assertEqualField(
    label,
    "styleAccessPathDepthSum",
    actual.styleAccessPathDepthSum,
    expected.styleAccessPathDepthSum,
  );
  assertArrayEqual(
    label,
    "plannedExpressionIds",
    actual.plannedExpressionIds,
    expected.plannedExpressionIds,
  );
  assertArrayEqual(
    label,
    "distinctStyleFilePaths",
    actual.distinctStyleFilePaths,
    expected.distinctStyleFilePaths,
  );
  assertRecordEqual(
    label,
    "expressionKindCounts",
    actual.expressionKindCounts,
    expected.expressionKindCounts,
  );
}

export function assertSourceResolutionQueryFragmentsMatch(
  label: string,
  actual: SourceResolutionQueryFragmentsV0,
  expected: SourceResolutionQueryFragmentsV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = stableJson(actual.fragments);
  const expectedJson = stableJson(expected.fragments);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: sourceResolutionQueryFragments mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertExpressionSemanticsFragmentsMatch(
  label: string,
  actual: ExpressionSemanticsFragmentsV0,
  expected: ExpressionSemanticsFragmentsV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = stableJson(actual.fragments);
  const expectedJson = stableJson(expected.fragments);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: expressionSemanticsFragments mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertExpressionSemanticsQueryFragmentsMatch(
  label: string,
  actual: ExpressionSemanticsQueryFragmentsV0,
  expected: ExpressionSemanticsQueryFragmentsV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = JSON.stringify(actual.fragments);
  const expectedJson = JSON.stringify(expected.fragments);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: expressionSemanticsQueryFragments mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertExpressionSemanticsMatchFragmentsMatch(
  label: string,
  actual: ExpressionSemanticsMatchFragmentsV0,
  expected: ExpressionSemanticsMatchFragmentsV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = JSON.stringify(actual.fragments);
  const expectedJson = JSON.stringify(expected.fragments);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: expressionSemanticsMatchFragments mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertExpressionSemanticsCandidatesMatch(
  label: string,
  actual: ExpressionSemanticsCandidatesV0,
  expected: ExpressionSemanticsCandidatesV0,
): void {
  if (actual.schemaVersion !== expected.schemaVersion) {
    throw new Error(
      `${label}: expression semantics candidates schema mismatch: ${actual.schemaVersion} !== ${expected.schemaVersion}`,
    );
  }
  if (actual.inputVersion !== expected.inputVersion) {
    throw new Error(
      `${label}: expression semantics candidates input version mismatch: ${actual.inputVersion} !== ${expected.inputVersion}`,
    );
  }
  if (stableJson(actual.candidates) !== stableJson(expected.candidates)) {
    throw new Error(
      `${label}: expression semantics candidates mismatch\nactual=${stableJson(actual.candidates)}\nexpected=${stableJson(expected.candidates)}`,
    );
  }
}

export function assertExpressionSemanticsCanonicalCandidateBundleMatch(
  label: string,
  actual: ExpressionSemanticsCanonicalCandidateBundleV0,
  expected: ExpressionSemanticsCanonicalCandidateBundleV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  if (stableJson(actual.queryFragments) !== stableJson(expected.queryFragments)) {
    throw new Error(
      `${label}: expression semantics canonical candidate queryFragments mismatch\nactual=${stableJson(actual.queryFragments)}\nexpected=${stableJson(expected.queryFragments)}`,
    );
  }
  if (stableJson(actual.fragments) !== stableJson(expected.fragments)) {
    throw new Error(
      `${label}: expression semantics canonical candidate fragments mismatch\nactual=${stableJson(actual.fragments)}\nexpected=${stableJson(expected.fragments)}`,
    );
  }
  if (stableJson(actual.matchFragments) !== stableJson(expected.matchFragments)) {
    throw new Error(
      `${label}: expression semantics canonical candidate matchFragments mismatch\nactual=${stableJson(actual.matchFragments)}\nexpected=${stableJson(expected.matchFragments)}`,
    );
  }
  if (stableJson(actual.candidates) !== stableJson(expected.candidates)) {
    throw new Error(
      `${label}: expression semantics canonical candidate candidates mismatch\nactual=${stableJson(actual.candidates)}\nexpected=${stableJson(expected.candidates)}`,
    );
  }
}

export function assertExpressionSemanticsEvaluatorCandidatesMatch(
  label: string,
  actual: ExpressionSemanticsEvaluatorCandidatesV0,
  expected: ExpressionSemanticsEvaluatorCandidatesV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  if (stableJson(actual.results) !== stableJson(expected.results)) {
    throw new Error(
      `${label}: expression semantics evaluator candidates mismatch\nactual=${stableJson(actual.results)}\nexpected=${stableJson(expected.results)}`,
    );
  }
}

export function assertExpressionSemanticsCanonicalProducerSignalMatch(
  label: string,
  actual: ExpressionSemanticsCanonicalProducerSignalV0,
  expected: ExpressionSemanticsCanonicalProducerSignalV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertExpressionSemanticsCanonicalCandidateBundleMatch(
    `${label}:canonicalBundle`,
    actual.canonicalBundle,
    expected.canonicalBundle,
  );
  assertExpressionSemanticsEvaluatorCandidatesMatch(
    `${label}:evaluatorCandidates`,
    actual.evaluatorCandidates,
    expected.evaluatorCandidates,
  );
}

export function assertSourceResolutionFragmentsMatch(
  label: string,
  actual: SourceResolutionFragmentsV0,
  expected: SourceResolutionFragmentsV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  const actualJson = stableJson(actual.fragments);
  const expectedJson = stableJson(expected.fragments);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: sourceResolutionFragments mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

export function assertSourceResolutionMatchFragmentsMatch(
  label: string,
  actual: SourceResolutionMatchFragmentsV0,
  expected: SourceResolutionMatchFragmentsV0,
): void {
  if (actual.schemaVersion !== expected.schemaVersion) {
    throw new Error(
      `${label}: source resolution match fragment schema mismatch: ${actual.schemaVersion} !== ${expected.schemaVersion}`,
    );
  }
  if (actual.inputVersion !== expected.inputVersion) {
    throw new Error(
      `${label}: source resolution match fragment input version mismatch: ${actual.inputVersion} !== ${expected.inputVersion}`,
    );
  }
  if (stableJson(actual.fragments) !== stableJson(expected.fragments)) {
    throw new Error(
      `${label}: source resolution match fragments mismatch\nactual=${stableJson(actual.fragments)}\nexpected=${stableJson(expected.fragments)}`,
    );
  }
}

export function assertSourceResolutionCandidatesMatch(
  label: string,
  actual: SourceResolutionCandidatesV0,
  expected: SourceResolutionCandidatesV0,
): void {
  if (actual.schemaVersion !== expected.schemaVersion) {
    throw new Error(
      `${label}: source resolution candidates schema mismatch: ${actual.schemaVersion} !== ${expected.schemaVersion}`,
    );
  }
  if (actual.inputVersion !== expected.inputVersion) {
    throw new Error(
      `${label}: source resolution candidates input version mismatch: ${actual.inputVersion} !== ${expected.inputVersion}`,
    );
  }
  if (stableJson(actual.candidates) !== stableJson(expected.candidates)) {
    throw new Error(
      `${label}: source resolution candidates mismatch\nactual=${stableJson(actual.candidates)}\nexpected=${stableJson(expected.candidates)}`,
    );
  }
}

export function assertSourceResolutionEvaluatorCandidatesMatch(
  label: string,
  actual: SourceResolutionEvaluatorCandidatesV0,
  expected: SourceResolutionEvaluatorCandidatesV0,
): void {
  if (actual.schemaVersion !== expected.schemaVersion) {
    throw new Error(
      `${label}: source resolution evaluator candidates schema mismatch: ${actual.schemaVersion} !== ${expected.schemaVersion}`,
    );
  }
  if (actual.inputVersion !== expected.inputVersion) {
    throw new Error(
      `${label}: source resolution evaluator candidates input version mismatch: ${actual.inputVersion} !== ${expected.inputVersion}`,
    );
  }
  if (stableJson(actual.results) !== stableJson(expected.results)) {
    throw new Error(
      `${label}: source resolution evaluator candidates mismatch\nactual=${stableJson(actual.results)}\nexpected=${stableJson(expected.results)}`,
    );
  }
}

export function assertSourceResolutionCanonicalCandidateBundleMatch(
  label: string,
  actual: SourceResolutionCanonicalCandidateBundleV0,
  expected: SourceResolutionCanonicalCandidateBundleV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  if (stableJson(actual.queryFragments) !== stableJson(expected.queryFragments)) {
    throw new Error(
      `${label}: source resolution canonical candidate queryFragments mismatch\nactual=${stableJson(actual.queryFragments)}\nexpected=${stableJson(expected.queryFragments)}`,
    );
  }
  if (stableJson(actual.fragments) !== stableJson(expected.fragments)) {
    throw new Error(
      `${label}: source resolution canonical candidate fragments mismatch\nactual=${stableJson(actual.fragments)}\nexpected=${stableJson(expected.fragments)}`,
    );
  }
  if (stableJson(actual.matchFragments) !== stableJson(expected.matchFragments)) {
    throw new Error(
      `${label}: source resolution canonical candidate matchFragments mismatch\nactual=${stableJson(actual.matchFragments)}\nexpected=${stableJson(expected.matchFragments)}`,
    );
  }
  if (stableJson(actual.candidates) !== stableJson(expected.candidates)) {
    throw new Error(
      `${label}: source resolution canonical candidate candidates mismatch\nactual=${stableJson(actual.candidates)}\nexpected=${stableJson(expected.candidates)}`,
    );
  }
}

export function assertSourceResolutionCanonicalProducerSignalMatch(
  label: string,
  actual: SourceResolutionCanonicalProducerSignalV0,
  expected: SourceResolutionCanonicalProducerSignalV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertSourceResolutionCanonicalCandidateBundleMatch(
    `${label}:canonicalBundle`,
    actual.canonicalBundle,
    expected.canonicalBundle,
  );
  assertSourceResolutionEvaluatorCandidatesMatch(
    `${label}:evaluatorCandidates`,
    actual.evaluatorCandidates,
    expected.evaluatorCandidates,
  );
}

export function assertSourceSideCanonicalCandidateBundleMatch(
  label: string,
  actual: SourceSideCanonicalCandidateBundleV0,
  expected: SourceSideCanonicalCandidateBundleV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertExpressionSemanticsCanonicalCandidateBundleMatch(
    `${label}:expressionSemantics`,
    actual.expressionSemantics,
    expected.expressionSemantics,
  );
  assertSourceResolutionCanonicalCandidateBundleMatch(
    `${label}:sourceResolution`,
    actual.sourceResolution,
    expected.sourceResolution,
  );
}

export function assertSourceSideEvaluatorCandidatesMatch(
  label: string,
  actual: SourceSideEvaluatorCandidatesV0,
  expected: SourceSideEvaluatorCandidatesV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertExpressionSemanticsEvaluatorCandidatesMatch(
    `${label}:expressionSemantics`,
    actual.expressionSemantics,
    expected.expressionSemantics,
  );
  assertSourceResolutionEvaluatorCandidatesMatch(
    `${label}:sourceResolution`,
    actual.sourceResolution,
    expected.sourceResolution,
  );
}

export function assertSourceSideCanonicalProducerSignalMatch(
  label: string,
  actual: SourceSideCanonicalProducerSignalV0,
  expected: SourceSideCanonicalProducerSignalV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertSourceSideCanonicalCandidateBundleMatch(
    `${label}:canonicalBundle`,
    actual.canonicalBundle,
    expected.canonicalBundle,
  );
  assertSourceSideEvaluatorCandidatesMatch(
    `${label}:evaluatorCandidates`,
    actual.evaluatorCandidates,
    expected.evaluatorCandidates,
  );
}

export function assertSemanticCanonicalCandidateBundleMatch(
  label: string,
  actual: SemanticCanonicalCandidateBundleV0,
  expected: SemanticCanonicalCandidateBundleV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertSourceSideCanonicalCandidateBundleMatch(
    `${label}:sourceSide`,
    actual.sourceSide,
    expected.sourceSide,
  );
  assertExpressionDomainCanonicalCandidateBundleMatch(
    `${label}:expressionDomain`,
    actual.expressionDomain,
    expected.expressionDomain,
  );
}

export function assertSemanticEvaluatorCandidatesMatch(
  label: string,
  actual: SemanticEvaluatorCandidatesV0,
  expected: SemanticEvaluatorCandidatesV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertSourceSideEvaluatorCandidatesMatch(
    `${label}:sourceSide`,
    actual.sourceSide,
    expected.sourceSide,
  );
  if (stableJson(actual.expressionDomain) !== stableJson(expected.expressionDomain)) {
    throw new Error(
      `${label}: semanticExpressionDomainEvaluatorCandidates mismatch\nactual=${stableJson(actual.expressionDomain)}\nexpected=${stableJson(expected.expressionDomain)}`,
    );
  }
}

export function assertSemanticCanonicalProducerSignalMatch(
  label: string,
  actual: SemanticCanonicalProducerSignalV0,
  expected: SemanticCanonicalProducerSignalV0,
): void {
  assertEqualField(label, "schemaVersion", actual.schemaVersion, expected.schemaVersion);
  assertEqualField(label, "inputVersion", actual.inputVersion, expected.inputVersion);
  assertSemanticCanonicalCandidateBundleMatch(
    `${label}:canonicalBundle`,
    actual.canonicalBundle,
    expected.canonicalBundle,
  );
  assertSemanticEvaluatorCandidatesMatch(
    `${label}:evaluatorCandidates`,
    actual.evaluatorCandidates,
    expected.evaluatorCandidates,
  );
}

function assertEqualField<T>(label: string, field: string, actual: T, expected: T) {
  if (actual !== expected) {
    throw new Error(
      `${label}: ${field} mismatch\nexpected: ${JSON.stringify(expected)}\nreceived: ${JSON.stringify(actual)}`,
    );
  }
}

function assertArrayEqual(
  label: string,
  field: string,
  actual: readonly string[],
  expected: readonly string[],
) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: ${field} mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

function assertRecordEqual(
  label: string,
  field: string,
  actual: Readonly<Record<string, number>>,
  expected: Readonly<Record<string, number>>,
) {
  const actualJson = JSON.stringify(sortRecord(actual));
  const expectedJson = JSON.stringify(sortRecord(expected));
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: ${field} mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

function sortRecord(record: Readonly<Record<string, number>>): Record<string, number> {
  return Object.fromEntries(Object.entries(record).toSorted(([a], [b]) => a.localeCompare(b)));
}

function stableJson(value: unknown): string {
  return JSON.stringify(sortJsonValue(value));
}

function sortJsonValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sortJsonValue);
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .toSorted(([a], [b]) => a.localeCompare(b))
        .map(([key, child]) => [key, sortJsonValue(child)]),
    );
  }
  return value;
}

function collectQueryPayloadSummary(
  query: QueryResultV2,
  expressionValueDomainKinds: Record<string, number>,
  expressionValueConstraintKinds: Record<string, number>,
  expressionConstraintDetailCounts: ConstraintDetailCounts,
  expressionValueCertaintyShapes: Record<string, number>,
  expressionSelectorCertaintyShapes: Record<string, number>,
  resolutionValueConstraintKinds: Record<string, number>,
  resolutionConstraintDetailCounts: ConstraintDetailCounts,
  resolutionValueCertaintyShapes: Record<string, number>,
  resolutionSelectorCertaintyShapes: Record<string, number>,
  expressionSemanticsIds: Set<string>,
  resolutionIds: Set<string>,
  selectorUsageIds: Set<string>,
  onSelectorUsage: (payload: SelectorUsagePayloadSummary) => void,
) {
  switch (query.kind) {
    case "expression-semantics":
      expressionSemanticsIds.add(query.queryId);
      increment(expressionValueDomainKinds, query.payload.valueDomainKind);
      if (query.payload.valueConstraintKind) {
        increment(expressionValueConstraintKinds, query.payload.valueConstraintKind);
      }
      collectConstraintDetailCounts(
        expressionConstraintDetailCounts,
        query.payload.valuePrefix,
        query.payload.valueSuffix,
        query.payload.valueMinLen,
        query.payload.valueMaxLen,
        query.payload.valueCharMust,
        query.payload.valueCharMay,
        query.payload.valueMayIncludeOtherChars === true,
      );
      if (query.payload.valueCertaintyShapeKind) {
        increment(expressionValueCertaintyShapes, query.payload.valueCertaintyShapeKind);
      }
      if (query.payload.selectorCertaintyShapeKind) {
        increment(expressionSelectorCertaintyShapes, query.payload.selectorCertaintyShapeKind);
      }
      break;
    case "source-expression-resolution":
      resolutionIds.add(query.queryId);
      if (query.payload.valueCertaintyConstraintKind) {
        increment(resolutionValueConstraintKinds, query.payload.valueCertaintyConstraintKind);
      }
      collectConstraintDetailCounts(
        resolutionConstraintDetailCounts,
        query.payload.valuePrefix,
        query.payload.valueSuffix,
        query.payload.valueMinLen,
        query.payload.valueMaxLen,
        query.payload.valueCharMust,
        query.payload.valueCharMay,
        query.payload.valueMayIncludeOtherChars === true,
      );
      if (query.payload.valueCertaintyShapeKind) {
        increment(resolutionValueCertaintyShapes, query.payload.valueCertaintyShapeKind);
      }
      if (query.payload.selectorCertaintyShapeKind) {
        increment(resolutionSelectorCertaintyShapes, query.payload.selectorCertaintyShapeKind);
      }
      break;
    case "selector-usage":
      selectorUsageIds.add(query.queryId);
      onSelectorUsage(query.payload);
      break;
  }
}

function increment(record: Record<string, number>, key: string) {
  record[key] = (record[key] ?? 0) + 1;
}

interface SelectorUsagePayloadSummary {
  readonly totalReferences: number;
  readonly directReferenceCount: number;
  readonly editableDirectReferenceCount: number;
  readonly exactReferenceCount: number;
  readonly inferredOrBetterReferenceCount: number;
  readonly hasExpandedReferences: boolean;
  readonly hasStyleDependencyReferences: boolean;
  readonly hasAnyReferences: boolean;
}

interface ConstraintDetailCounts {
  prefixCount: number;
  suffixCount: number;
  minLenCount: number;
  minLenSum: number;
  maxLenCount: number;
  maxLenSum: number;
  charMustCount: number;
  charMustLenSum: number;
  charMayCount: number;
  charMayLenSum: number;
  mayIncludeOtherCharsCount: number;
}

function createConstraintDetailCounts(): ConstraintDetailCounts {
  return {
    prefixCount: 0,
    suffixCount: 0,
    minLenCount: 0,
    minLenSum: 0,
    maxLenCount: 0,
    maxLenSum: 0,
    charMustCount: 0,
    charMustLenSum: 0,
    charMayCount: 0,
    charMayLenSum: 0,
    mayIncludeOtherCharsCount: 0,
  };
}

function collectConstraintDetailCounts(
  counts: ConstraintDetailCounts,
  prefix: string | undefined,
  suffix: string | undefined,
  minLen: number | undefined,
  maxLen: number | undefined,
  charMust: string | undefined,
  charMay: string | undefined,
  mayIncludeOtherChars: boolean,
) {
  if (prefix !== undefined) counts.prefixCount += 1;
  if (suffix !== undefined) counts.suffixCount += 1;
  if (minLen !== undefined) {
    counts.minLenCount += 1;
    counts.minLenSum += minLen;
  }
  if (maxLen !== undefined) {
    counts.maxLenCount += 1;
    counts.maxLenSum += maxLen;
  }
  if (charMust !== undefined) {
    counts.charMustCount += 1;
    counts.charMustLenSum += charMust.length;
  }
  if (charMay !== undefined) {
    counts.charMayCount += 1;
    counts.charMayLenSum += charMay.length;
  }
  if (mayIncludeOtherChars) {
    counts.mayIncludeOtherCharsCount += 1;
  }
}

function assertConstraintDetailEqual(
  label: string,
  field: string,
  actual: ConstraintDetailCounts,
  expected: ConstraintDetailCounts,
) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(
      `${label}: ${field} mismatch\nexpected: ${expectedJson}\nreceived: ${actualJson}`,
    );
  }
}

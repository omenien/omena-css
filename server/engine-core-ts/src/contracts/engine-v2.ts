import type { TextRewritePlan } from "../core/rewrite/text-rewrite-plan";
import type { EngineWorkspaceV1, SourceAnalysisInputV1, StyleAnalysisInputV1 } from "./engine-v1";
import type { CheckerReportV1 } from "./checker-v1";
import type {
  EngineInputV2Json,
  StringConstraintKindV2Json,
  StringTypeFactKindV2Json,
  StringTypeFactsV2Json,
  TypeFactEntryV2Json,
} from "./engine-v2-input-idl.generated";
import type {
  CertaintyShapeKindV2Json,
  EngineOutputV2Json,
  ExpressionSemanticsQueryResultV2Json,
  QueryResultV2Json,
  SelectorUsageQueryResultV2Json,
  SourceExpressionResolutionQueryResultV2Json,
  ValueDomainDerivationStepV2Json,
  ValueDomainDerivationV2Json,
  ValueDomainKindV2Json,
  ValueDomainProvenanceNodeV2Json,
  ValueDomainProvenanceTreeV2Json,
} from "./engine-v2-output-idl.generated";

export const ENGINE_CONTRACT_VERSION_V2 = "2" as const;

export type StringTypeFactKindV2 = StringTypeFactKindV2Json;
export type StringConstraintKindV2 = StringConstraintKindV2Json;
export type StringTypeFactsV2 = StringTypeFactsV2Json;
export type TypeFactTableEntryV2 = TypeFactEntryV2Json;

export type TypeFactTableV2 = readonly TypeFactTableEntryV2[];

export type EngineInputV2 = Omit<
  EngineInputV2Json,
  "version" | "workspace" | "sources" | "styles" | "typeFacts"
> & {
  readonly version: typeof ENGINE_CONTRACT_VERSION_V2;
  readonly workspace: EngineWorkspaceV1;
  readonly sources: readonly SourceAnalysisInputV1[];
  readonly styles: readonly StyleAnalysisInputV1[];
  readonly typeFacts: TypeFactTableV2;
};

export type ValueDomainKindV2 = ValueDomainKindV2Json;
export type ValueCertaintyShapeKindV2 = CertaintyShapeKindV2Json;
export type SelectorCertaintyShapeKindV2 = CertaintyShapeKindV2Json;
export type ValueDomainDerivationStepV2 = ValueDomainDerivationStepV2Json;
export type ValueDomainDerivationV2 = ValueDomainDerivationV2Json;
export type ValueDomainProvenanceNodeV2 = ValueDomainProvenanceNodeV2Json;
export type ValueDomainProvenanceTreeV2 = ValueDomainProvenanceTreeV2Json;
export type ExpressionSemanticsQueryResultV2 = ExpressionSemanticsQueryResultV2Json;
export type SourceExpressionResolutionQueryResultV2 = SourceExpressionResolutionQueryResultV2Json;
export type SelectorUsageQueryResultV2 = SelectorUsageQueryResultV2Json;
export type QueryResultV2 = QueryResultV2Json;
export type QueryResultKindV2 = QueryResultV2["kind"];

export type EngineOutputV2 = Omit<
  EngineOutputV2Json,
  "version" | "queryResults" | "rewritePlans" | "checkerReport"
> & {
  readonly version: typeof ENGINE_CONTRACT_VERSION_V2;
  readonly queryResults: readonly QueryResultV2[];
  readonly rewritePlans: readonly TextRewritePlan<unknown>[];
  readonly checkerReport: CheckerReportV1;
};

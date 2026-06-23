import "@typespec/json-schema";

using TypeSpec.JsonSchema;

@jsonSchema
namespace Omena.EngineV2;

alias EngineContractVersionV2 = "2";

enum ClassnameTransformMode {
  asIs: "asIs",
  camelCase: "camelCase",
  camelCaseOnly: "camelCaseOnly",
  dashes: "dashes",
  dashesOnly: "dashesOnly",
}

model EngineWorkspaceV1 {
  root: string;
  classnameTransform: ClassnameTransformMode;
  settingsKey: string;
}

model SourceAnalysisInputV2 {
  filePath: string;
  document: unknown;
  bindingGraph?: unknown;
}

model StyleAnalysisInputV2 {
  filePath: string;
  source?: string;
  document: unknown;
}

enum StringTypeFactKindV2 {
  Unknown: "unknown",
  Exact: "exact",
  FiniteSet: "finiteSet",
  Constrained: "constrained",
  Top: "top",
}

enum StringConstraintKindV2 {
  prefix: "prefix",
  suffix: "suffix",
  prefixSuffix: "prefixSuffix",
  charInclusion: "charInclusion",
  composite: "composite",
}

model StringTypeFactsV2 {
  kind: StringTypeFactKindV2;
  values?: string[];
  constraintKind?: StringConstraintKindV2;
  prefix?: string;
  suffix?: string;
  minLen?: int32;
  maxLen?: int32;
  charMust?: string;
  charMay?: string;
  mayIncludeOtherChars?: boolean;
  provenance?: string;
}

model TypeFactEntryV2 {
  filePath: string;
  expressionId: string;
  facts: StringTypeFactsV2;
}

model EngineInputV2 {
  version: EngineContractVersionV2;
  workspace: EngineWorkspaceV1;
  sources: SourceAnalysisInputV2[];
  styles: StyleAnalysisInputV2[];
  typeFacts: TypeFactEntryV2[];
}

enum ValueDomainKindV2 {
  none: "none",
  exact: "exact",
  finiteSet: "finiteSet",
  constrained: "constrained",
  top: "top",
}

enum CertaintyShapeKindV2 {
  Exact: "exact",
  BoundedFinite: "boundedFinite",
  Constrained: "constrained",
  Unknown: "unknown",
}

model ValueDomainDerivationStepV2 {
  operation: string;
  inputKind?: string;
  refinementKind?: string;
  resultKind: string;
  resultProvenance?: string;
  reason: string;
}

model ValueDomainDerivationV2 {
  schemaVersion: string;
  product: string;
  inputFactKind: string;
  inputConstraintKind?: string;
  inputValueCount: int32;
  reducedKind: string;
  steps: ValueDomainDerivationStepV2[];
}

model ValueDomainProvenanceNodeV2 {
  operation: string;
  resultKind: string;
  resultProvenance?: string;
  detail?: string;
  reason: string;
  children: ValueDomainProvenanceNodeV2[];
}

model ValueDomainProvenanceTreeV2 {
  schemaVersion: string;
  product: string;
  valueKind: string;
  value: unknown;
  valueProvenance?: string;
  root: ValueDomainProvenanceNodeV2;
}

model ExpressionSemanticsPayloadV2 {
  expressionId: string;
  expressionKind: string;
  styleFilePath: string | null;
  selectorNames: string[];
  candidateNames: string[];
  finiteValues: string[] | null;
  valueDomainKind: ValueDomainKindV2;
  valueConstraintKind?: StringConstraintKindV2;
  valuePrefix?: string;
  valueSuffix?: string;
  valueMinLen?: int32;
  valueMaxLen?: int32;
  valueCharMust?: string;
  valueCharMay?: string;
  valueMayIncludeOtherChars?: boolean;
  valueDomainReason?: string;
  valueDomainDerivation?: ValueDomainDerivationV2;
  valueDomainProvenanceTree?: ValueDomainProvenanceTreeV2;
  selectorCertainty: string;
  selectorCertaintyShapeKind?: CertaintyShapeKindV2;
  selectorConstraintKind?: StringConstraintKindV2;
  selectorCertaintyShapeLabel?: string;
  selectorCertaintyReason?: string;
  valueCertainty?: string;
  valueCertaintyShapeKind?: CertaintyShapeKindV2;
  valueCertaintyConstraintKind?: StringConstraintKindV2;
  valueCertaintyShapeLabel?: string;
  valueCertaintyReason?: string;
  reason?: string;
}

model SourceExpressionResolutionPayloadV2 {
  expressionId: string;
  styleFilePath: string | null;
  selectorNames: string[];
  finiteValues: string[] | null;
  selectorCertainty: string;
  selectorCertaintyShapeKind?: CertaintyShapeKindV2;
  selectorConstraintKind?: StringConstraintKindV2;
  selectorCertaintyShapeLabel?: string;
  selectorCertaintyReason?: string;
  valueCertainty?: string;
  valueCertaintyShapeKind?: CertaintyShapeKindV2;
  valueCertaintyConstraintKind?: StringConstraintKindV2;
  valuePrefix?: string;
  valueSuffix?: string;
  valueMinLen?: int32;
  valueMaxLen?: int32;
  valueCharMust?: string;
  valueCharMay?: string;
  valueMayIncludeOtherChars?: boolean;
  valueCertaintyShapeLabel?: string;
  valueCertaintyReason?: string;
  reason?: string;
}

model SelectorUsagePayloadV2 {
  canonicalName: string;
  totalReferences: int32;
  directReferenceCount: int32;
  editableDirectReferenceCount: int32;
  exactReferenceCount: int32;
  inferredOrBetterReferenceCount: int32;
  hasExpandedReferences: boolean;
  hasStyleDependencyReferences: boolean;
  hasAnyReferences: boolean;
}

model ExpressionSemanticsQueryResultV2 {
  kind: "expression-semantics";
  filePath: string;
  queryId: string;
  payload: ExpressionSemanticsPayloadV2;
}

model SourceExpressionResolutionQueryResultV2 {
  kind: "source-expression-resolution";
  filePath: string;
  queryId: string;
  payload: SourceExpressionResolutionPayloadV2;
}

model SelectorUsageQueryResultV2 {
  kind: "selector-usage";
  filePath: string;
  queryId: string;
  payload: SelectorUsagePayloadV2;
}

@oneOf
union QueryResultV2 {
  expressionSemantics: ExpressionSemanticsQueryResultV2,
  sourceExpressionResolution: SourceExpressionResolutionQueryResultV2,
  selectorUsage: SelectorUsageQueryResultV2,
}

model TextPositionJsonV2 {
  line: int32;
  character: int32;
}

model TextRangeJsonV2 {
  start: TextPositionJsonV2;
  end: TextPositionJsonV2;
}

model PlannedTextEditJsonV2 {
  uri: string;
  range: TextRangeJsonV2;
  newText: string;
}

model TextRewritePlanJsonV2 {
  target: unknown;
  edits: PlannedTextEditJsonV2[];
}

model CheckerReportJsonV1 {
  version: string;
  findings: unknown[];
  summary: unknown;
}

model EngineOutputV2 {
  version: EngineContractVersionV2;
  queryResults: QueryResultV2[];
  rewritePlans: TextRewritePlanJsonV2[];
  checkerReport: CheckerReportJsonV1;
}

model OmenaQueryWorkspaceTextEditV0 {
  uri: string;
  range: unknown;
  newText: string;
}

enum CodeActionPlanKind {
  quickfix: "quickfix",
  refactorExtract: "refactor.extract",
  refactorInline: "refactor.inline",
}

model OmenaQueryCodeActionV0 {
  title: string;
  kind: CodeActionPlanKind;
  edits: OmenaQueryWorkspaceTextEditV0[];
  source: string;
}

model OmenaQueryCodeActionPlanV0 {
  schemaVersion: string;
  product: "omena-query.code-actions";
  fileUri: string;
  fileKind: "style";
  actionCount: int32;
  actions: OmenaQueryCodeActionV0[];
  readySurfaces?: string[];
}

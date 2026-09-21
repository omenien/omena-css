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

// Document payloads preserve the fields emitted by the source/style HIR producers.
// Only fields historically required by the Rust input boundary are mandatory;
// omitted HIR metadata remains omitted rather than acquiring invented defaults.
@minValue(0)
scalar EngineTextPositionOffsetV2 extends safeint;
model EnginePositionV2 {
  line: EngineTextPositionOffsetV2;
  character: EngineTextPositionOffsetV2;
}
model EngineRangeV2 {
  start: EnginePositionV2;
  end: EnginePositionV2;
}
model EngineNodeV2 {
  id?: string;
  range?: EngineRangeV2;
}
model EngineStyleImportV2 {
  kind: string;
  absolutePath: string;
  specifier?: string;
  range?: EngineRangeV2;
}
model EngineStyleImportBindingV2 extends EngineNodeV2 {
  kind: string;
  localName: string;
  bindingDeclId: string;
  resolved: EngineStyleImportV2;
}
model EngineUtilityBindingV2 extends EngineNodeV2 {
  kind: string;
  localName: string;
  bindingDeclId: string;
  stylesLocalName?: string;
  scssModulePath?: string;
  classNamesImportName?: string;
}
model EngineClassExpressionV2 {
  id: string;
  kind: string;
  scssModulePath: string;
  range: EngineRangeV2;
  origin?: string;
  className?: string;
  rootBindingDeclId?: string;
  accessPath?: string[];
  rawTemplate?: string;
  staticPrefix?: string;
  rawReference?: string;
  rootName?: string;
  pathSegments?: string[];
  bindingDeclId?: string;
}
model EngineDomainClassReferenceV2 extends EngineNodeV2 {
  kind: string;
  pluginId: string;
  domain: string;
  origin: string;
  matchKind: string;
  className?: string;
  rawTemplate?: string;
  staticPrefix?: string;
}
model EngineSourceDocumentV2 {
  kind?: string;
  filePath?: string;
  language?: string;
  classExpressions: EngineClassExpressionV2[];
  styleImports?: EngineStyleImportBindingV2[];
  utilityBindings?: EngineUtilityBindingV2[];
  domainClassReferences?: EngineDomainClassReferenceV2[];
}
model EngineBindingDeclarationV2 {
  id: string;
  name: string;
  kind: string;
}
model EngineBindingResolutionV2 {
  expressionId: string;
  declarationId: string | null;
}
model EngineBindingGraphV2 {
  declarations: EngineBindingDeclarationV2[];
  resolutions: EngineBindingResolutionV2[];
}
model EngineComposesClassTokenV2 {
  className: string;
  range: EngineRangeV2;
}
model EngineComposesRefV2 {
  classNames: string[];
  range?: EngineRangeV2;
  classTokens?: EngineComposesClassTokenV2[];
  from?: string;
  fromGlobal?: boolean;
}
model EngineBemSuffixV2 {
  rawTokenRange: EngineRangeV2;
  rawToken?: string;
  parentResolvedName?: string;
}
model EngineAtRuleContextV2 {
  name: string;
  params: string;
  range: EngineRangeV2;
}
model EngineSelectorContextV2 {
  wrapperAtRules: EngineAtRuleContextV2[];
}
model EngineStyleSelectorV2 {
  name: string;
  viewKind: string;
  range: EngineRangeV2;
  id?: string;
  kind?: string;
  canonicalName?: string;
  nestedSafety?: string;
  composes?: EngineComposesRefV2[];
  bemSuffix?: EngineBemSuffixV2;
  fullSelector?: string;
  declarations?: string;
  ruleRange?: EngineRangeV2;
  originalName?: string;
  context?: EngineSelectorContextV2;
}
model EngineKeyframesDeclV2 extends EngineNodeV2 {
  kind: string;
  name: string;
  ruleRange: EngineRangeV2;
}
model EngineAnimationNameRefV2 extends EngineNodeV2 {
  kind: string;
  name: string;
  property: string;
}
model EngineValueDeclV2 extends EngineNodeV2 {
  kind: string;
  name: string;
  value: string;
  ruleRange: EngineRangeV2;
}
model EngineValueImportV2 extends EngineNodeV2 {
  kind: string;
  name: string;
  importedName: string;
  importedNameRange?: EngineRangeV2;
  from: string;
  ruleRange: EngineRangeV2;
}
model EngineValueRefV2 extends EngineNodeV2 {
  kind: string;
  name: string;
  source: string;
}
model EngineCustomPropertyContextV2 {
  containerKind: string;
  selectorText: string | null;
  atRuleName: string | null;
  atRuleParams: string | null;
  wrapperAtRules: EngineAtRuleContextV2[];
}
model EngineCustomPropertyDeclV2 extends EngineNodeV2 {
  kind: string;
  name: string;
  value: string;
  ruleRange: EngineRangeV2;
  context: EngineCustomPropertyContextV2;
}
model EngineCustomPropertyRefV2 extends EngineNodeV2 {
  kind: string;
  name: string;
  context: EngineCustomPropertyContextV2;
}
model EngineSassSymbolV2 extends EngineNodeV2 {
  kind: string;
  selectorName: string;
  syntax?: string;
  symbolKind: string;
  name: string;
  role: string;
  resolution: string;
  ruleRange: EngineRangeV2;
}
model EngineSassSymbolDeclV2 extends EngineNodeV2 {
  kind: string;
  syntax?: string;
  symbolKind: string;
  name: string;
  ruleRange: EngineRangeV2;
}
model EngineSassModuleUseV2 extends EngineNodeV2 {
  kind: string;
  source: string;
  namespaceKind: string;
  `namespace`: string | null;
  ruleRange: EngineRangeV2;
}
model EngineSassForwardMemberV2 {
  name: string;
  symbolKind: string | null;
}
model EngineSassModuleForwardV2 extends EngineNodeV2 {
  kind: string;
  source: string;
  prefix: string;
  visibilityKind: string;
  visibilityMembers: EngineSassForwardMemberV2[];
  ruleRange: EngineRangeV2;
}
model EngineSassModuleMemberRefV2 extends EngineNodeV2 {
  kind: string;
  selectorName: string;
  `namespace`: string;
  symbolKind: string;
  name: string;
  role: string;
  ruleRange: EngineRangeV2;
}
model EngineStyleDocumentV2 {
  kind?: string;
  filePath?: string;
  selectors: EngineStyleSelectorV2[];
  keyframes?: EngineKeyframesDeclV2[];
  animationNameRefs?: EngineAnimationNameRefV2[];
  valueDecls?: EngineValueDeclV2[];
  valueImports?: EngineValueImportV2[];
  valueRefs?: EngineValueRefV2[];
  customPropertyDecls?: EngineCustomPropertyDeclV2[];
  customPropertyRefs?: EngineCustomPropertyRefV2[];
  sassSymbols?: EngineSassSymbolV2[];
  sassSymbolDecls?: EngineSassSymbolDeclV2[];
  sassModuleUses?: EngineSassModuleUseV2[];
  sassModuleForwards?: EngineSassModuleForwardV2[];
  sassModuleMemberRefs?: EngineSassModuleMemberRefV2[];
}

model SourceAnalysisInputV2 {
  filePath: string;
  document: EngineSourceDocumentV2;
  bindingGraph?: EngineBindingGraphV2;
}

model StyleAnalysisInputV2 {
  filePath: string;
  source?: string;
  document: EngineStyleDocumentV2;
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

scalar Utf16CodeUnitLengthV2 extends int32;

model StringTypeFactsV2 {
  kind: StringTypeFactKindV2;
  values?: string[];
  constraintKind?: StringConstraintKindV2;
  prefix?: string;
  suffix?: string;
  minLen?: Utf16CodeUnitLengthV2;
  maxLen?: Utf16CodeUnitLengthV2;
  charMust?: string;
  charMay?: string;
  mayIncludeOtherChars?: boolean;
  provenance?: string;
}

enum TypeFactControlFlowBlockKindV2 {
  entry: "entry",
  assignment: "assignment",
  branch: "branch",
  join: "join",
  loopHeader: "loopHeader",
  loopBody: "loopBody",
  loopExit: "loopExit",
  break: "break",
  terminate: "terminate",
  logicalOperand: "logicalOperand",
  logicalRhs: "logicalRhs",
  logicalJoin: "logicalJoin",
  exit: "exit",
}

enum TypeFactControlFlowTransferKindV2 {
  entry: "entry",
  assignFacts: "assignFacts",
  branch: "branch",
  concatFacts: "concatFacts",
  join: "join",
  loop: "loop",
  break: "break",
  terminate: "terminate",
  exit: "exit",
}

enum TypeFactControlFlowExpressionKindV2 {
  logicalAnd: "logicalAnd",
  logicalOr: "logicalOr",
  nullishCoalesce: "nullishCoalesce",
}

enum ClassBoundaryEffectV2 {
  concatInsideToken: "concatInsideToken",
  concatAtTokenBoundary: "concatAtTokenBoundary",
  unknownBoundary: "unknownBoundary",
}

model TypeFactControlFlowBlockV2 {
  id: string;
  kind: TypeFactControlFlowBlockKindV2;
  transferKind: TypeFactControlFlowTransferKindV2;
  successorBlockIds: string[];
  symbolOrdinal?: int32;
  variableName?: string;
  expressionKind?: TypeFactControlFlowExpressionKindV2;
  boundaryEffect: ClassBoundaryEffectV2;
  facts?: StringTypeFactsV2;
}

model TypeFactControlFlowGraphV2 {
  entryBlockId: string;
  blocks: TypeFactControlFlowBlockV2[];
}

model TypeFactEntryV2 {
  filePath: string;
  expressionId: string;
  facts: StringTypeFactsV2;
  controlFlowGraph?: TypeFactControlFlowGraphV2;
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
  valueMinLen?: Utf16CodeUnitLengthV2;
  valueMaxLen?: Utf16CodeUnitLengthV2;
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
  valueMinLen?: Utf16CodeUnitLengthV2;
  valueMaxLen?: Utf16CodeUnitLengthV2;
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

enum OmenaQueryDiagnosticSeverityV0 {
  error: "error",
  warning: "warning",
  information: "information",
  hint: "hint",
}

model OmenaQueryCreateModuleFileActionV0 {
  uri: string;
}

model OmenaQueryCreateSelectorActionV0 {
  uri: string;
  range: TextRangeJsonV2;
  newText: string;
  selectorName: string;
}

model OmenaQueryCreateValueActionV0 {
  uri: string;
  range: TextRangeJsonV2;
  newText: string;
  valueName: string;
}

model OmenaQueryCreateKeyframesActionV0 {
  uri: string;
  range: TextRangeJsonV2;
  newText: string;
  animationName: string;
}

model OmenaQueryCreateCustomPropertyActionV0 {
  uri: string;
  range: TextRangeJsonV2;
  newText: string;
  propertyName: string;
}

model OmenaQueryCreateSassSymbolActionV0 {
  uri: string;
  range: TextRangeJsonV2;
  newText: string;
  symbolKind: string;
  symbolName: string;
  symbolSyntax: string;
}

enum OmenaQueryValueDomainPrecisionV1 {
  exactClassValue: "exactClassValue",
  closedClassValueSet: "closedClassValueSet",
  patternClassValue: "patternClassValue",
  cascadeAtPosition: "cascadeAtPosition",
  styleModuleResolution: "styleModuleResolution",
  classValueResolution: "classValueResolution",
  classValueUniverse: "classValueUniverse",
  classValueFlow: "classValueFlow",
  unknownValue: "unknown",
}

enum OmenaQueryFlowPrecisionV1 {
  representationBound: "representationBound",
  providerObservation: "providerObservation",
  incrementalDataflow: "incrementalDataflow",
  positionScopedCascade: "positionScopedCascade",
  sourceSelectorUsage: "sourceSelectorUsage",
  sourceControlFlow: "sourceControlFlow",
  sourceSyntaxIndex: "sourceSyntaxIndex",
  globalClassUniverse: "globalClassUniverse",
  sourceImportResolution: "sourceImportResolution",
  typeOracleProviderUnavailable: "typeOracleProviderUnavailable",
  sourceDomainReference: "sourceDomainReference",
  sourceSelectorReference: "sourceSelectorReference",
  kLimitedCallSiteFlow: "kLimitedCallSiteFlow",
  fixture: "fixture",
  unknownValue: "unknown",
}

enum OmenaQueryContextPrecisionV1 {
  valueLocal: "valueLocal",
  providerScoped: "providerScoped",
  perExpressionGraph: "perExpressionGraph",
  styleSemanticGraph: "styleSemanticGraph",
  perModuleExport: "perModuleExport",
  sameFile: "sameFile",
  perSourceReference: "perSourceReference",
  perImportSpecifier: "perImportSpecifier",
  perTypeFactTarget: "perTypeFactTarget",
  perDomainAxis: "perDomainAxis",
  resolvedClassValueDomain: "resolvedClassValueDomain",
  contextInsensitive: "0-cfa",
  shallowCallSite: "1-cfa",
  kLimitedCallSite: "k-cfa",
  fixture: "fixture",
  unknownValue: "unknown",
}

enum OmenaQueryProviderCompletenessV1 {
  complete: "complete",
  partial: "partial",
  unresolved: "unresolved",
  unknownValue: "unknown",
}

enum OmenaQueryWorldAssumptionV1 {
  closed: "closed",
  open: "open",
  unknownValue: "unknown",
}

enum OmenaQueryRevisionIdentityV1 {
  querySourceDiagnosticsInput: "OmenaQuerySourceDiagnosticsForFileV0.input",
  expressionDomainFlowRuntime: "OmenaQueryExpressionDomainFlowRuntimeV0.revision",
  evaluationRuntimeExpressionDomain: "OmenaQueryEvaluationRuntimeSummaryV0.expressionDomainRevision",
  workspaceSnapshot: "workspaceSnapshot",
  current: "current",
  staleTypeFact: "staleTypeFact",
  unknownValue: "unknown",
}

model OmenaQueryAnalysisPrecisionV0 {
  product: string;
  valueDomain: OmenaQueryValueDomainPrecisionV1;
  flowSensitivity: OmenaQueryFlowPrecisionV1;
  contextSensitivity: OmenaQueryContextPrecisionV1;
  providerCompleteness: OmenaQueryProviderCompletenessV1;
  worldAssumption: OmenaQueryWorldAssumptionV1;
  revisionAxis: OmenaQueryRevisionIdentityV1;
}

model OmenaQuerySourceDiagnosticV0 {
  code: string;
  severity: OmenaQueryDiagnosticSeverityV0;
  provenance: string[];
  range: TextRangeJsonV2;
  message: string;
  precision?: OmenaQueryAnalysisPrecisionV0;
  suggestion?: string;
  createSelector?: OmenaQueryCreateSelectorActionV0;
}

model OmenaQuerySourceDiagnosticsForFileV0 {
  schemaVersion: string;
  product: "omena-query.diagnostics-for-file";
  fileUri: string;
  fileKind: "source";
  diagnosticCount: int32;
  diagnostics: OmenaQuerySourceDiagnosticV0[];
  readySurfaces: string[];
}

model OmenaQueryStyleDiagnosticV0 {
  code: string;
  severity: OmenaQueryDiagnosticSeverityV0;
  provenance: string[];
  range: TextRangeJsonV2;
  message: string;
  tags?: int32[];
  createModuleFile?: OmenaQueryCreateModuleFileActionV0;
  createSelector?: OmenaQueryCreateSelectorActionV0;
  createValue?: OmenaQueryCreateValueActionV0;
  createKeyframes?: OmenaQueryCreateKeyframesActionV0;
  createCustomProperty?: OmenaQueryCreateCustomPropertyActionV0;
  createSassSymbol?: OmenaQueryCreateSassSymbolActionV0;
  cascadeNarrowing?: unknown;
  cascadeConfidence?: unknown;
  polynomialProvenance?: unknown;
  crossFileScc?: unknown;
}

model OmenaQueryStyleDiagnosticsForFileV0 {
  schemaVersion: string;
  product: "omena-query.diagnostics-for-file";
  fileUri: string;
  fileKind: "style";
  diagnosticCount: int32;
  diagnostics: OmenaQueryStyleDiagnosticV0[];
  readySurfaces: string[];
  suppressionSummary?: unknown;
}

@oneOf
union OmenaQueryDiagnosticsForFileV0 {
  source: OmenaQuerySourceDiagnosticsForFileV0,
  style: OmenaQueryStyleDiagnosticsForFileV0,
}

export type { SourceExpressionCursor, SourceExpressionQueryDeps } from "./contracts";
export {
  buildInvalidClassAnalysisMetadata,
  buildDynamicExpressionExplanation,
  describeAbstractValueReason,
  describeSelectorCertaintyReason,
  describeValueCertaintyReason,
  messageForInvalidClassFinding,
  type DynamicExpressionExplanation,
  type InvalidClassAnalysisMetadata,
} from "./explain-expression-semantics";
export {
  findAnimationNameRefAtCursor,
  findCanonicalSelector,
  findCanonicalSelectorsByName,
  findComposesTokenAtCursor,
  findCustomPropertyDeclAtCursor,
  findCustomPropertyDeclByName,
  findCustomPropertyRefAtCursor,
  findKeyframesAtCursor,
  findKeyframesByName,
  findSassSymbolAtCursor,
  findSassSymbolDeclAtCursor,
  findSassSymbolDeclByName,
  findSassSymbolDeclForSymbol,
  findSassModuleUseAtCursor,
  findSassModuleMemberRefAtCursor,
  findSelectorAtCursor,
  findValueDeclAtCursor,
  findValueDeclByName,
  findValueImportAtCursor,
  findValueImportByName,
  findValueRefAtCursor,
  listAnimationNameRefs,
  listCustomPropertyModuleUseDeclTargets,
  listCanonicalSelectors,
  listCustomPropertyRefs,
  listSassSymbols,
  listSassSymbolsForDecl,
  listSassWildcardSymbolsForTarget,
  listValueRefs,
  listSassModuleUseCandidatePaths,
  listSassModuleMemberRefsForMember,
  listSassModuleExportedSymbols,
  listSassModuleExportedSymbolTargets,
  resolveValueImportTarget,
  resolveValueTarget,
  resolveComposesTarget,
  resolveCustomPropertyDeclTarget,
  resolveSassModuleForwardTarget,
  resolveSassModuleUseTarget,
  resolveSassModuleUseTargetFilePath,
  resolveSassModuleMemberRefTarget,
  resolveSassWildcardSymbolTarget,
  type CustomPropertyDeclLookup,
  type ResolvedCustomPropertyDeclTarget,
  type ResolvedSassModuleExportedSymbolTarget,
  type ResolvedSassModuleForwardTarget,
  type ResolvedSassModuleMemberTarget,
  type ResolvedSassModuleUseTarget,
  type ResolvedSassWildcardSymbolTarget,
  type SassModulePathAliasResolver,
  type SassModuleResolutionOptions,
} from "./find-style-selector";
export { findInvalidClassReference } from "./find-invalid-class-references";
export { isInsideCall, readCompletionContext } from "./read-completion-context";
export {
  readExpressionSemantics,
  type ValueDomainProvenanceTree,
} from "./read-expression-semantics";
export {
  readDomainClassReferenceSummary,
  type DomainClassReferenceGroup,
  type DomainClassReferenceSummary,
} from "./read-domain-class-references";
export { readSelectorRewriteSafetySummary } from "./read-selector-rewrite-safety";
export {
  readSelectorStyleDependencySummary,
  type SelectorStyleDependencySummary,
} from "./read-selector-style-dependencies";
export { readSelectorUsageSummary, type SelectorUsageSummary } from "./read-selector-usage";
export {
  readSourceExpressionContextAtCursor,
  type SourceExpressionContext,
} from "./read-source-expression-context";
export {
  readSourceExpressionResolution,
  type SourceExpressionResolution,
} from "./read-source-expression-resolution";
export { readStyleModuleUsageSummary } from "./read-style-module-usage";
export { resolveRefDetails, type DynamicHoverExplanation } from "./resolve-ref";

import "@typespec/json-schema";

using TypeSpec.JsonSchema;

@jsonSchema
namespace Omena.EngineNapi;

enum EngineNapiBoundaryErrorKindV0 {
  parseError: "parse-error",
  serializeError: "serialize-error",
  unsupportedMode: "unsupported-mode",
}

model EngineNapiBoundaryErrorV0 {
  kind: EngineNapiBoundaryErrorKindV0;
  message: string;
  functionName?: string;
}

model EngineNapiVendorPrefixPolicyV0 {
  webkit: boolean;
  moz: boolean;
  ms: boolean;
}

model EngineNapiTransformImportInlineV0 {
  importSource: string;
  replacementCss: string;
}

model EngineNapiClassNameRewriteV0 {
  originalName: string;
  rewrittenName: string;
}

model EngineNapiCssModuleComposesResolutionV0 {
  localClassName: string;
  exportedClassNames: string[];
}

model EngineNapiCssModuleValueResolutionV0 {
  localName: string;
  resolvedValue: string;
}

model EngineNapiDesignTokenRouteV0 {
  tokenName: string;
  routedValue: string;
}

model EngineNapiTransformExecutionContextV0 {
  dropDarkModeMediaQueries?: boolean;
  supportsTargetCapability?: unknown;
  vendorPrefixPolicy?: EngineNapiVendorPrefixPolicyV0;
  reachableClassNames?: string[];
  reachableKeyframeNames?: string[];
  reachableValueNames?: string[];
  reachableCustomPropertyNames?: string[];
  scssModuleEvaluation?: unknown;
  lessModuleEvaluation?: unknown;
  importInlines?: EngineNapiTransformImportInlineV0[];
  classNameRewrites?: EngineNapiClassNameRewriteV0[];
  cssModuleComposesResolutions?: EngineNapiCssModuleComposesResolutionV0[];
  cssModuleValueResolutions?: EngineNapiCssModuleValueResolutionV0[];
  designTokenRoutes?: EngineNapiDesignTokenRouteV0[];
}

model EngineNapiTargetTransformOptionsV0 {
  allowLogicalToPhysical?: boolean;
  allowScopeFlatten?: boolean;
  allowLayerFlatten?: boolean;
  enableSupportsStaticEval?: boolean;
  enableMediaStaticEval?: boolean;
  enableContainerStaticEval?: boolean;
  dropDarkModeMediaQueries?: boolean;
}

model EngineNapiStyleSourceInputV0 {
  stylePath: string;
  styleSource: string;
}

model EngineNapiStylePackageManifestV0 {
  packageJsonPath: string;
  packageJsonSource: string;
}

model EngineNapiSourceDocumentInputV0 {
  sourcePath: string;
  sourceSource: string;
  hasUnresolvedStyleImport?: boolean;
}

model EngineNapiPositionV0 {
  line: int32;
  character: int32;
}

model EngineNapiRangeV0 {
  start: EngineNapiPositionV0;
  end: EngineNapiPositionV0;
}

model EngineNapiSourceMissingSelectorDiagnosticCandidateV0 {
  targetStyleUri: string;
  targetStyleSource: string;
  selectorName: string;
  sourceReferenceRange: EngineNapiRangeV0;
}

model EngineNapiSourceImportedStyleBindingInputV0 {
  binding: string;
  styleUri: string;
}

model EngineNapiEngineInputV2 {
  version: string;
  workspace: unknown;
  sources: unknown[];
  styles: unknown[];
  typeFacts: unknown[];
}

model EngineNapiConsumerBuildSummaryV0 {
  schemaVersion: string;
  product: string;
  stylePath: string;
  dialect: string;
  requestedPassIds: string[];
  targetQuery?: unknown;
  unknownPassIds: string[];
  execution: unknown;
  semanticRemovalCount: int32;
  bundle?: unknown;
  sourceMapV3?: unknown;
  openWorldSnapshot?: unknown;
  readySurfaces: string[];
}

model EngineNapiBoundarySurfaceV0 {
  context: EngineNapiTransformExecutionContextV0;
  targetOptions: EngineNapiTargetTransformOptionsV0;
  input: EngineNapiEngineInputV2;
  sources: EngineNapiStyleSourceInputV0[];
  packageManifests: EngineNapiStylePackageManifestV0[];
  sourceDocuments: EngineNapiSourceDocumentInputV0[];
  candidates: EngineNapiSourceMissingSelectorDiagnosticCandidateV0[];
  importedStyleBindings: EngineNapiSourceImportedStyleBindingInputV0[];
  classnamesBindBindings: string[];
  buildSummary: EngineNapiConsumerBuildSummaryV0;
  boundaryError: EngineNapiBoundaryErrorV0;
}

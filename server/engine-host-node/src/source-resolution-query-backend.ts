import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import {
  TOP_CLASS_VALUE,
  charInclusionClassValue,
  compositeClassValue,
  finiteSetClassValue,
  prefixClassValue,
  prefixSuffixClassValue,
  suffixClassValue,
} from "../../engine-core-ts/src/core/abstract-value/class-value-domain";
import type {
  SelectorDeclHIR,
  StyleDocumentHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import type { SourceExpressionResolution } from "../../engine-core-ts/src/core/query/read-source-expression-resolution";
import type { EdgeCertainty } from "../../engine-core-ts/src/core/semantic/certainty";
import {
  buildSelectedQueryBackendInput,
  buildSelectedQueryBackendInputAsync,
  isEngineShadowRunnerCancelledError,
  SELECTED_QUERY_RUNNER_COMMANDS,
  resolveSelectedQueryBackendKind,
  runRustSelectedQueryBackendJson,
  runRustSelectedQueryBackendJsonAsync,
  usesRustExpressionSemanticsBackend,
  usesRustSelectorUsageBackend,
  usesRustSourceResolutionBackend,
  type RustSelectedQueryBackendJsonRunnerAsync,
  type SelectedQueryBackendDocument,
} from "./selected-query-backend";

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

export interface SourceResolutionSelectorMatch {
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
}

interface SourceResolutionEvaluatorCandidateV0 {
  readonly kind: "source-expression-resolution";
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: SourceResolutionEvaluatorCandidatePayloadV0;
}

interface SourceResolutionCanonicalProducerSignalV0 {
  readonly evaluatorCandidates: {
    readonly results: readonly SourceResolutionEvaluatorCandidateV0[];
  };
}

export function resolveRustSourceResolutionPayloads(
  document: SelectedQueryBackendDocument,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
): readonly SourceResolutionEvaluatorCandidatePayloadV0[] {
  const input = buildSelectedQueryBackendInput(document, scssModulePath, deps);
  try {
    const signal = runRustSelectedQueryBackendJson<SourceResolutionCanonicalProducerSignalV0>(
      SELECTED_QUERY_RUNNER_COMMANDS.sourceResolutionCanonicalProducer,
      input,
    );
    return signal.evaluatorCandidates.results.map((candidate) => candidate.payload);
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) return [];
    throw err;
  }
}

export async function resolveRustSourceResolutionPayloadsAsync(
  document: SelectedQueryBackendDocument,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  runJson: RustSelectedQueryBackendJsonRunnerAsync = runRustSelectedQueryBackendJsonAsync,
): Promise<readonly SourceResolutionEvaluatorCandidatePayloadV0[]> {
  const input = await buildSelectedQueryBackendInputAsync(document, scssModulePath, deps);
  try {
    const signal = await runJson<SourceResolutionCanonicalProducerSignalV0>(
      SELECTED_QUERY_RUNNER_COMMANDS.sourceResolutionCanonicalProducer,
      input,
    );
    return signal.evaluatorCandidates.results.map((candidate) => candidate.payload);
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) return [];
    throw err;
  }
}

export function resolveRustSourceResolutionPayload(
  document: SelectedQueryBackendDocument,
  expressionId: string,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
): SourceResolutionEvaluatorCandidatePayloadV0 | null {
  const match = resolveRustSourceResolutionPayloads(document, scssModulePath, deps).find(
    (payload) => payload.expressionId === expressionId,
  );
  return match ?? null;
}

export async function resolveRustSourceResolutionPayloadAsync(
  document: SelectedQueryBackendDocument,
  expressionId: string,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<SourceResolutionEvaluatorCandidatePayloadV0 | null> {
  const match = (
    await resolveRustSourceResolutionPayloadsAsync(document, scssModulePath, deps, runJson)
  ).find((payload) => payload.expressionId === expressionId);
  return match ?? null;
}

export function resolveRustSourceResolutionSelectorMatch(
  document: SelectedQueryBackendDocument,
  expressionId: string,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
): SourceResolutionSelectorMatch | null {
  const payload = resolveRustSourceResolutionPayload(document, expressionId, scssModulePath, deps);
  if (!payload || !payload.styleFilePath) return null;

  return {
    styleFilePath: payload.styleFilePath,
    selectorNames: payload.selectorNames,
  };
}

export async function resolveRustSourceResolutionSelectorMatchAsync(
  document: SelectedQueryBackendDocument,
  expressionId: string,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<SourceResolutionSelectorMatch | null> {
  const payload = await resolveRustSourceResolutionPayloadAsync(
    document,
    expressionId,
    scssModulePath,
    deps,
    runJson,
  );
  if (!payload || !payload.styleFilePath) return null;

  return {
    styleFilePath: payload.styleFilePath,
    selectorNames: payload.selectorNames,
  };
}

export function buildSourceResolutionSummaryFromRustPayload(
  styleDocument: StyleDocumentHIR | null,
  selectors: readonly SelectorDeclHIR[],
  payload: SourceResolutionEvaluatorCandidatePayloadV0,
): SourceExpressionResolution {
  const abstractValue = buildRustSourceResolutionAbstractValue(payload);
  const reason = inferRustSourceResolutionReason(payload);

  return {
    styleDocument,
    selectors,
    finiteValues: payload.finiteValues ?? null,
    ...(abstractValue ? { abstractValue } : {}),
    ...(payload.valueCertainty ? { valueCertainty: payload.valueCertainty as EdgeCertainty } : {}),
    ...(reason ? { reason } : {}),
    selectorCertainty: payload.selectorCertainty as EdgeCertainty,
  };
}

export {
  resolveSelectedQueryBackendKind,
  usesRustExpressionSemanticsBackend,
  usesRustSelectorUsageBackend,
  usesRustSourceResolutionBackend,
};

function buildRustSourceResolutionAbstractValue(
  payload: SourceResolutionEvaluatorCandidatePayloadV0,
) {
  switch (payload.valueCertaintyShapeKind) {
    case "unknown":
      return undefined;
    case "exact": {
      const value = payload.finiteValues?.[0] ?? payload.selectorNames[0];
      return value ? finiteSetClassValue([value]) : undefined;
    }
    case "boundedFinite":
      return payload.finiteValues?.length
        ? finiteSetClassValue(payload.finiteValues)
        : payload.selectorNames.length > 0
          ? finiteSetClassValue(payload.selectorNames)
          : undefined;
    case "prefix":
      return payload.valuePrefix ? prefixClassValue(payload.valuePrefix) : undefined;
    case "constrained":
      return buildRustConstrainedAbstractValue(payload);
    case "top":
      return TOP_CLASS_VALUE;
    default:
      return undefined;
  }
}

function buildRustConstrainedAbstractValue(payload: SourceResolutionEvaluatorCandidatePayloadV0) {
  switch (payload.valueCertaintyConstraintKind) {
    case "prefix":
      return payload.valuePrefix ? prefixClassValue(payload.valuePrefix) : TOP_CLASS_VALUE;
    case "suffix":
      return payload.valueSuffix ? suffixClassValue(payload.valueSuffix) : TOP_CLASS_VALUE;
    case "prefixSuffix":
      return prefixSuffixClassValue(
        payload.valuePrefix ?? "",
        payload.valueSuffix ?? "",
        payload.valueMinLen,
      );
    case "charInclusion":
      return charInclusionClassValue(
        payload.valueCharMust ?? "",
        payload.valueCharMay ?? "",
        undefined,
        Boolean(payload.valueMayIncludeOtherChars),
      );
    case "composite":
      return compositeClassValue({
        ...(payload.valuePrefix ? { prefix: payload.valuePrefix } : {}),
        ...(payload.valueSuffix ? { suffix: payload.valueSuffix } : {}),
        ...(payload.valueMinLen !== undefined ? { minLength: payload.valueMinLen } : {}),
        mustChars: payload.valueCharMust ?? "",
        mayChars: payload.valueCharMay ?? "",
        ...(payload.valueMayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
      });
    default:
      return TOP_CLASS_VALUE;
  }
}

function inferRustSourceResolutionReason(
  payload: SourceResolutionEvaluatorCandidatePayloadV0,
): SourceExpressionResolution["reason"] | undefined {
  if (payload.valueCertainty === "exact" && payload.selectorCertainty === "exact") {
    return "flowLiteral";
  }
  if (
    payload.finiteValues !== undefined ||
    payload.selectorNames.length > 1 ||
    payload.valueCertaintyShapeKind === "constrained" ||
    payload.valueCertaintyShapeKind === "top"
  ) {
    return "flowBranch";
  }
  return undefined;
}

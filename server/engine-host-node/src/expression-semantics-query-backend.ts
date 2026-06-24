import {
  TOP_CLASS_VALUE,
  charInclusionClassValue,
  compositeClassValue,
  finiteSetClassValue,
  prefixClassValue,
  prefixSuffixClassValue,
  suffixClassValue,
} from "../../engine-core-ts/src/core/abstract-value/class-value-domain";
import type { ClassExpressionHIR } from "../../engine-core-ts/src/core/hir/source-types";
import type {
  SelectorDeclHIR,
  StyleDocumentHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import type { EdgeCertainty } from "../../engine-core-ts/src/core/semantic/certainty";
import type {
  ExpressionSemanticsSummary,
  ValueDomainProvenanceTree,
} from "../../engine-core-ts/src/core/query/read-expression-semantics";
import {
  buildSelectedQueryBackendInput,
  buildSelectedQueryBackendInputAsync,
  isEngineShadowRunnerCancelledError,
  SELECTED_QUERY_RUNNER_COMMANDS,
  runRustSelectedQueryBackendJson,
  runRustSelectedQueryBackendJsonAsync,
  type RustSelectedQueryBackendJsonRunnerAsync,
  type SelectedQueryBackendDocument,
} from "./selected-query-backend";

export interface ExpressionSemanticsEvaluatorCandidatePayloadV0 {
  readonly expressionId: string;
  readonly expressionKind: string;
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
  readonly candidateNames: readonly string[];
  readonly finiteValues?: readonly string[];
  readonly valueDomainKind: string;
  readonly selectorCertainty: EdgeCertainty;
  readonly valueCertainty?: EdgeCertainty;
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
  readonly valueDomainDerivation?: ReducedClassValueDerivationV0;
  readonly valueDomainProvenanceTree?: ValueDomainProvenanceTree;
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

interface ExpressionSemanticsEvaluatorCandidateV0 {
  readonly kind: "expression-semantics";
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: ExpressionSemanticsEvaluatorCandidatePayloadV0;
}

interface ExpressionSemanticsCanonicalProducerSignalV0 {
  readonly evaluatorCandidates: {
    readonly results: readonly ExpressionSemanticsEvaluatorCandidateV0[];
  };
}

export function resolveRustExpressionSemanticsPayloads(
  document: SelectedQueryBackendDocument,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
): readonly ExpressionSemanticsEvaluatorCandidatePayloadV0[] {
  const input = buildSelectedQueryBackendInput(document, scssModulePath, deps);
  try {
    const signal = runRustSelectedQueryBackendJson<ExpressionSemanticsCanonicalProducerSignalV0>(
      SELECTED_QUERY_RUNNER_COMMANDS.expressionSemanticsCanonicalProducer,
      input,
    );
    return signal.evaluatorCandidates.results.map((candidate) => candidate.payload);
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) return [];
    throw err;
  }
}

export async function resolveRustExpressionSemanticsPayloadsAsync(
  document: SelectedQueryBackendDocument,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  runJson: RustSelectedQueryBackendJsonRunnerAsync = runRustSelectedQueryBackendJsonAsync,
): Promise<readonly ExpressionSemanticsEvaluatorCandidatePayloadV0[]> {
  const input = await buildSelectedQueryBackendInputAsync(document, scssModulePath, deps);
  try {
    const signal = await runJson<ExpressionSemanticsCanonicalProducerSignalV0>(
      SELECTED_QUERY_RUNNER_COMMANDS.expressionSemanticsCanonicalProducer,
      input,
    );
    return signal.evaluatorCandidates.results.map((candidate) => candidate.payload);
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) return [];
    throw err;
  }
}

export function resolveRustExpressionSemanticsPayload(
  document: SelectedQueryBackendDocument,
  expressionId: string,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
): ExpressionSemanticsEvaluatorCandidatePayloadV0 | null {
  const match = resolveRustExpressionSemanticsPayloads(document, scssModulePath, deps).find(
    (payload) => payload.expressionId === expressionId,
  );
  return match ?? null;
}

export async function resolveRustExpressionSemanticsPayloadAsync(
  document: SelectedQueryBackendDocument,
  expressionId: string,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<ExpressionSemanticsEvaluatorCandidatePayloadV0 | null> {
  const match = (
    await resolveRustExpressionSemanticsPayloadsAsync(document, scssModulePath, deps, runJson)
  ).find((payload) => payload.expressionId === expressionId);
  return match ?? null;
}

export function buildExpressionSemanticsSummaryFromRustPayload(
  expression: ClassExpressionHIR,
  styleDocument: StyleDocumentHIR | null,
  selectors: readonly SelectorDeclHIR[],
  payload: ExpressionSemanticsEvaluatorCandidatePayloadV0,
): ExpressionSemanticsSummary {
  const abstractValue = buildRustExpressionAbstractValue(payload);
  const selectorNames = selectors.map((selector) => selector.name);
  const reason = inferRustExpressionReason(payload);

  return {
    expression,
    styleDocument,
    selectors,
    selectorNames: selectorNames.length > 0 ? selectorNames : payload.selectorNames,
    candidateNames: payload.candidateNames,
    finiteValues: payload.finiteValues ?? null,
    valueDomainKind: mapRustValueDomainKind(payload.valueDomainKind),
    ...(abstractValue ? { abstractValue } : {}),
    ...(payload.valueDomainDerivation
      ? { valueDomainDerivation: payload.valueDomainDerivation }
      : {}),
    ...(payload.valueDomainProvenanceTree
      ? { valueDomainProvenanceTree: payload.valueDomainProvenanceTree }
      : {}),
    ...(payload.valueCertainty ? { valueCertainty: payload.valueCertainty } : {}),
    selectorCertainty: payload.selectorCertainty,
    ...(reason !== undefined ? { reason } : {}),
  };
}

function mapRustValueDomainKind(
  valueDomainKind: string,
): ExpressionSemanticsSummary["valueDomainKind"] {
  switch (valueDomainKind) {
    case "none":
    case "exact":
    case "finiteSet":
    case "prefix":
    case "top":
      return valueDomainKind;
    case "constrained":
      return "constrained";
    default:
      return "top";
  }
}

function buildRustExpressionAbstractValue(payload: ExpressionSemanticsEvaluatorCandidatePayloadV0) {
  switch (payload.valueDomainKind) {
    case "none":
      return undefined;
    case "exact": {
      const value = payload.finiteValues?.[0] ?? payload.candidateNames[0];
      return value ? finiteSetClassValue([value]) : undefined;
    }
    case "finiteSet":
      return payload.finiteValues?.length
        ? finiteSetClassValue(payload.finiteValues)
        : payload.candidateNames.length > 0
          ? finiteSetClassValue(payload.candidateNames)
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

function buildRustConstrainedAbstractValue(
  payload: ExpressionSemanticsEvaluatorCandidatePayloadV0,
) {
  switch (payload.valueConstraintKind) {
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

function inferRustExpressionReason(
  payload: ExpressionSemanticsEvaluatorCandidatePayloadV0,
): "flowLiteral" | "flowBranch" | undefined {
  if (payload.valueCertainty === "exact" && payload.selectorCertainty === "exact") {
    return "flowLiteral";
  }
  if (
    payload.finiteValues !== undefined ||
    payload.candidateNames.length > 1 ||
    payload.valueDomainKind === "constrained" ||
    payload.valueDomainKind === "top"
  ) {
    return "flowBranch";
  }
  return undefined;
}

import type { ResolvedType } from "@omena/shared";
import {
  finiteSetClassValue,
  type AbstractClassValue,
} from "../core/abstract-value/class-value-domain";
import type {
  EngineOutputV1,
  ExpressionSemanticsQueryResultV1,
  QueryResultV1,
  SelectorUsageQueryResultV1,
  SourceExpressionResolutionQueryResultV1,
  StringTypeFactsV1,
  TypeFactTableEntryV1,
} from "./engine-v1";
import { ENGINE_CONTRACT_VERSION_V1 } from "./engine-v1";
import type {
  EngineOutputV2,
  ExpressionSemanticsQueryResultV2,
  QueryResultV2,
  SelectorUsageQueryResultV2,
  SourceExpressionResolutionQueryResultV2,
  StringTypeFactsV2,
  TypeFactControlFlowGraphV2,
  TypeFactTableEntryV2,
} from "./engine-v2";

export function upcastFactsV1ToV2(facts: StringTypeFactsV1): StringTypeFactsV2 {
  switch (facts.kind) {
    case "unknown":
      return { kind: "unknown" };
    case "exact":
      return { kind: "exact", ...(facts.values ? { values: facts.values } : {}) };
    case "finiteSet":
      return { kind: "finiteSet", ...(facts.values ? { values: facts.values } : {}) };
    case "top":
      return { kind: "top" };
    case "prefix":
      return {
        kind: "constrained",
        constraintKind: "prefix",
        ...(facts.prefix ? { prefix: facts.prefix } : {}),
      };
    default:
      return { kind: "unknown" };
  }
}

export function normalizeResolvedTypeToTypeFactsV2(resolvedType: ResolvedType): StringTypeFactsV2 {
  if (resolvedType.kind === "unresolvable") {
    return { kind: "unknown" };
  }

  return externalizeAbstractClassValueToTypeFactsV2(finiteSetClassValue(resolvedType.values));
}

export function downcastFactsV2ToV1(facts: StringTypeFactsV2): StringTypeFactsV1 {
  switch (facts.kind) {
    case "unknown":
      return { kind: "unknown" };
    case "exact":
      return { kind: "exact", ...(facts.values ? { values: facts.values } : {}) };
    case "finiteSet":
      return { kind: "finiteSet", ...(facts.values ? { values: facts.values } : {}) };
    case "top":
      return { kind: "top" };
    case "constrained":
      switch (facts.constraintKind) {
        case "prefix":
          return facts.prefix ? { kind: "prefix", prefix: facts.prefix } : { kind: "top" };
        case "prefixSuffix":
          return facts.prefix ? { kind: "prefix", prefix: facts.prefix } : { kind: "top" };
        case "charInclusion":
        case "suffix":
        case "composite":
        case undefined:
          return { kind: "top" };
        default:
          facts.constraintKind satisfies never;
          return { kind: "top" };
      }
    default:
      return { kind: "unknown" };
  }
}

export function upcastTypeFactTableEntryV1ToV2(entry: TypeFactTableEntryV1): TypeFactTableEntryV2 {
  return {
    filePath: entry.filePath,
    expressionId: entry.expressionId,
    facts: upcastFactsV1ToV2(entry.facts),
  };
}

export function downcastQueryResultV2ToV1(query: QueryResultV2): QueryResultV1 {
  switch (query.kind) {
    case "expression-semantics":
      return downcastExpressionSemanticsQueryResultV2ToV1(query);
    case "source-expression-resolution":
      return downcastSourceExpressionResolutionQueryResultV2ToV1(query);
    case "selector-usage":
      return downcastSelectorUsageQueryResultV2ToV1(query);
    default:
      query satisfies never;
      return query;
  }
}

export function downcastEngineOutputV2ToV1(output: EngineOutputV2): EngineOutputV1 {
  return {
    version: ENGINE_CONTRACT_VERSION_V1,
    queryResults: output.queryResults.map(downcastQueryResultV2ToV1),
    rewritePlans: output.rewritePlans,
    checkerReport: output.checkerReport,
  };
}

export function createTypeFactTableEntryV2(
  filePath: string,
  expressionId: string,
  resolvedType: ResolvedType,
  controlFlowGraph?: TypeFactControlFlowGraphV2 | null,
): TypeFactTableEntryV2 {
  return {
    filePath,
    expressionId,
    facts: normalizeResolvedTypeToTypeFactsV2(resolvedType),
    ...(controlFlowGraph ? { controlFlowGraph } : {}),
  };
}

function externalizeAbstractClassValueToTypeFactsV2(value: AbstractClassValue): StringTypeFactsV2 {
  switch (value.kind) {
    case "bottom":
      return { kind: "unknown" };
    case "exact":
      return { kind: "exact", values: [value.value] };
    case "finiteSet":
      return { kind: "finiteSet", values: value.values };
    case "prefix":
      return {
        kind: "constrained",
        constraintKind: "prefix",
        prefix: value.prefix,
        ...(value.provenance ? { provenance: value.provenance } : {}),
      };
    case "suffix":
      return {
        kind: "constrained",
        constraintKind: "suffix",
        suffix: value.suffix,
        ...(value.provenance ? { provenance: value.provenance } : {}),
      };
    case "prefixSuffix":
      return {
        kind: "constrained",
        constraintKind: "prefixSuffix",
        prefix: value.prefix,
        suffix: value.suffix,
        minLen: value.minLength,
        ...(value.provenance ? { provenance: value.provenance } : {}),
      };
    case "charInclusion":
      return {
        kind: "constrained",
        constraintKind: "charInclusion",
        charMust: value.mustChars,
        charMay: value.mayChars,
        ...(value.mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
        ...(value.provenance ? { provenance: value.provenance } : {}),
      };
    case "composite":
      return {
        kind: "constrained",
        constraintKind: "composite",
        ...(value.prefix ? { prefix: value.prefix } : {}),
        ...(value.suffix ? { suffix: value.suffix } : {}),
        ...(value.minLength !== undefined ? { minLen: value.minLength } : {}),
        charMust: value.mustChars,
        charMay: value.mayChars,
        ...(value.mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
        ...(value.provenance ? { provenance: value.provenance } : {}),
      };
    case "top":
      return { kind: "top" };
    default:
      value satisfies never;
      return { kind: "unknown" };
  }
}

function downcastExpressionSemanticsQueryResultV2ToV1(
  query: ExpressionSemanticsQueryResultV2,
): ExpressionSemanticsQueryResultV1 {
  return {
    kind: query.kind,
    filePath: query.filePath,
    queryId: query.queryId,
    payload: {
      expressionId: query.payload.expressionId,
      expressionKind: query.payload.expressionKind,
      styleFilePath: query.payload.styleFilePath,
      selectorNames: query.payload.selectorNames,
      candidateNames: query.payload.candidateNames,
      finiteValues: query.payload.finiteValues,
      valueDomainKind: query.payload.valueDomainKind,
      ...(query.payload.valueDomainReason
        ? { valueDomainReason: query.payload.valueDomainReason }
        : {}),
      selectorCertainty: query.payload.selectorCertainty,
      ...(query.payload.selectorCertaintyShapeLabel
        ? { selectorCertaintyShapeLabel: query.payload.selectorCertaintyShapeLabel }
        : {}),
      ...(query.payload.selectorCertaintyReason
        ? { selectorCertaintyReason: query.payload.selectorCertaintyReason }
        : {}),
      ...(query.payload.valueCertainty ? { valueCertainty: query.payload.valueCertainty } : {}),
      ...(query.payload.valueCertaintyShapeLabel
        ? { valueCertaintyShapeLabel: query.payload.valueCertaintyShapeLabel }
        : {}),
      ...(query.payload.valueCertaintyReason
        ? { valueCertaintyReason: query.payload.valueCertaintyReason }
        : {}),
      ...(query.payload.reason ? { reason: query.payload.reason } : {}),
    },
  };
}

function downcastSourceExpressionResolutionQueryResultV2ToV1(
  query: SourceExpressionResolutionQueryResultV2,
): SourceExpressionResolutionQueryResultV1 {
  return {
    kind: query.kind,
    filePath: query.filePath,
    queryId: query.queryId,
    payload: {
      expressionId: query.payload.expressionId,
      styleFilePath: query.payload.styleFilePath,
      selectorNames: query.payload.selectorNames,
      finiteValues: query.payload.finiteValues,
      selectorCertainty: query.payload.selectorCertainty,
      ...(query.payload.selectorCertaintyShapeLabel
        ? { selectorCertaintyShapeLabel: query.payload.selectorCertaintyShapeLabel }
        : {}),
      ...(query.payload.selectorCertaintyReason
        ? { selectorCertaintyReason: query.payload.selectorCertaintyReason }
        : {}),
      ...(query.payload.valueCertainty ? { valueCertainty: query.payload.valueCertainty } : {}),
      ...(query.payload.valueCertaintyShapeLabel
        ? { valueCertaintyShapeLabel: query.payload.valueCertaintyShapeLabel }
        : {}),
      ...(query.payload.valueCertaintyReason
        ? { valueCertaintyReason: query.payload.valueCertaintyReason }
        : {}),
      ...(query.payload.reason ? { reason: query.payload.reason } : {}),
    },
  };
}

function downcastSelectorUsageQueryResultV2ToV1(
  query: SelectorUsageQueryResultV2,
): SelectorUsageQueryResultV1 {
  return {
    kind: query.kind,
    filePath: query.filePath,
    queryId: query.queryId,
    payload: {
      canonicalName: query.payload.canonicalName,
      totalReferences: query.payload.totalReferences,
      directReferenceCount: query.payload.directReferenceCount,
      editableDirectReferenceCount: query.payload.editableDirectReferenceCount,
      exactReferenceCount: query.payload.exactReferenceCount,
      inferredOrBetterReferenceCount: query.payload.inferredOrBetterReferenceCount,
      hasExpandedReferences: query.payload.hasExpandedReferences,
      hasStyleDependencyReferences: query.payload.hasStyleDependencyReferences,
      hasAnyReferences: query.payload.hasAnyReferences,
    },
  };
}

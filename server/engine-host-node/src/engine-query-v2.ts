import {
  describeAbstractValueReason,
  describeSelectorCertaintyReason,
  describeValueCertaintyReason,
  listCanonicalSelectors,
  readExpressionSemantics,
  readSelectorUsageSummary,
  readSourceExpressionResolution,
} from "../../engine-core-ts/src/core/query";
import {
  deriveSelectorCertaintyProfileV2,
  deriveValueCertaintyProfileV2,
} from "../../engine-core-ts/src/core/semantic/certainty";
import type { DocumentAnalysisCache } from "../../engine-core-ts/src/core/indexing/document-analysis-cache";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import type { WorkspaceSemanticWorkspaceReferenceIndex } from "../../engine-core-ts/src/core/semantic/workspace-reference-index";
import type { WorkspaceStyleDependencyGraph } from "../../engine-core-ts/src/core/semantic/style-dependency-graph";
import type { TypeResolver } from "../../engine-core-ts/src/core/ts/type-resolver";
import type {
  ExpressionSemanticsQueryResultV2,
  QueryResultV2,
  SelectorUsageQueryResultV2,
  SourceExpressionResolutionQueryResultV2,
} from "../../engine-core-ts/src/contracts";
import type { SourceDocumentSnapshot } from "./checker-host/workspace-check-support";
import { classifyValueDomainV2 } from "./query-metadata-v2";
import type { ClassnameTransformMode } from "../../engine-core-ts/src/core/scss/classname-transform";
import { DEFAULT_SETTINGS } from "../../engine-core-ts/src/settings";
import {
  buildExpressionSemanticsSummaryFromRustPayload,
  resolveRustExpressionSemanticsPayloads,
  type ExpressionSemanticsEvaluatorCandidatePayloadV0,
  type resolveRustExpressionSemanticsPayload,
} from "./expression-semantics-query-backend";
import {
  buildSourceResolutionSummaryFromRustPayload,
  resolveRustSourceResolutionPayloads,
  type SourceResolutionEvaluatorCandidatePayloadV0,
  type resolveRustSourceResolutionPayload,
} from "./source-resolution-query-backend";
import {
  resolveSelectedQueryBackendKind,
  usesRustExpressionSemanticsBackend,
  usesRustSelectorUsageBackend,
  usesRustSourceResolutionBackend,
} from "./selected-query-backend";
import {
  resolveRustSelectorUsagePayloads,
  type SelectorUsageEvaluatorCandidateV0,
  type resolveRustSelectorUsagePayload,
} from "./selector-usage-query-backend";

export interface BuildSelectedQueryResultsV2Options {
  readonly workspaceRoot: string;
  readonly classnameTransform: ClassnameTransformMode;
  readonly pathAlias: Readonly<Record<string, string>>;
  readonly sourceDocuments: readonly SourceDocumentSnapshot[];
  readonly styleFiles: readonly string[];
  readonly analysisCache: DocumentAnalysisCache;
  readonly styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null;
  readonly typeResolver: TypeResolver;
  readonly semanticReferenceIndex: WorkspaceSemanticWorkspaceReferenceIndex;
  readonly styleDependencyGraph: WorkspaceStyleDependencyGraph;
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustSourceResolutionPayload?: typeof resolveRustSourceResolutionPayload;
  readonly readRustSourceResolutionPayloads?: typeof resolveRustSourceResolutionPayloads;
  readonly readRustExpressionSemanticsPayload?: typeof resolveRustExpressionSemanticsPayload;
  readonly readRustExpressionSemanticsPayloads?: typeof resolveRustExpressionSemanticsPayloads;
  readonly readRustSelectorUsagePayload?: typeof resolveRustSelectorUsagePayload;
  readonly readRustSelectorUsagePayloads?: typeof resolveRustSelectorUsagePayloads;
}

export function buildSelectedQueryResultsV2(
  options: BuildSelectedQueryResultsV2Options,
): readonly QueryResultV2[] {
  const results: QueryResultV2[] = [];
  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);

  for (const document of options.sourceDocuments) {
    const readRustSourceResolutionPayload = usesRustSourceResolutionBackend(selectedQueryBackend)
      ? createSourceResolutionPayloadReader(document, options)
      : null;
    const readRustExpressionSemanticsPayload = usesRustExpressionSemanticsBackend(
      selectedQueryBackend,
    )
      ? createExpressionSemanticsPayloadReader(document, options)
      : null;
    const analysis = options.analysisCache.get(
      document.uri,
      document.content,
      document.filePath,
      document.version,
    );

    for (const expression of analysis.sourceDocument.classExpressions) {
      const queryContext = {
        expression,
        sourceFile: analysis.sourceFile,
      } as const;
      const queryEnv = {
        styleDocumentForPath: options.styleDocumentForPath,
        typeResolver: options.typeResolver,
        filePath: document.filePath,
        workspaceRoot: options.workspaceRoot,
        sourceBinder: analysis.sourceBinder,
        sourceBindingGraph: analysis.sourceBindingGraph,
        classValueUniverses: analysis.classValueUniverses,
      } as const;
      const rustSourceResolutionPayload = readRustSourceResolutionPayload
        ? readRustSourceResolutionPayload(
            {
              uri: document.uri,
              content: document.content,
              filePath: document.filePath,
              version: document.version,
            },
            expression.id,
            expression.scssModulePath,
            {
              analysisCache: options.analysisCache,
              styleDocumentForPath: options.styleDocumentForPath,
              typeResolver: options.typeResolver,
              workspaceRoot: options.workspaceRoot,
              settings: {
                ...DEFAULT_SETTINGS,
                scss: {
                  ...DEFAULT_SETTINGS.scss,
                  classnameTransform: options.classnameTransform,
                },
                pathAlias: options.pathAlias,
              },
            },
          )
        : null;
      const resolution = rustSourceResolutionPayload
        ? null
        : readSourceExpressionResolution(queryContext, queryEnv);
      const rustExpressionSemanticsPayload = readRustExpressionSemanticsPayload
        ? readRustExpressionSemanticsPayload(
            {
              uri: document.uri,
              content: document.content,
              filePath: document.filePath,
              version: document.version,
            },
            expression.id,
            expression.scssModulePath,
            {
              analysisCache: options.analysisCache,
              styleDocumentForPath: options.styleDocumentForPath,
              typeResolver: options.typeResolver,
              workspaceRoot: options.workspaceRoot,
              settings: {
                ...DEFAULT_SETTINGS,
                scss: {
                  ...DEFAULT_SETTINGS.scss,
                  classnameTransform: options.classnameTransform,
                },
                pathAlias: options.pathAlias,
              },
            },
          )
        : null;
      const semantics = rustExpressionSemanticsPayload
        ? null
        : readExpressionSemantics(queryContext, queryEnv);

      results.push(
        rustExpressionSemanticsPayload
          ? expressionSemanticsResultV2FromRustPayload(
              document.filePath,
              expression,
              rustExpressionSemanticsPayload,
              options.styleDocumentForPath,
            )
          : expressionSemanticsResultV2(document.filePath, semantics!),
      );
      results.push(
        rustSourceResolutionPayload
          ? sourceExpressionResolutionResultV2FromRustPayload(
              document.filePath,
              expression.id,
              rustSourceResolutionPayload,
              options.styleDocumentForPath,
            )
          : sourceExpressionResolutionResultV2(document.filePath, expression.id, resolution!),
      );
    }
  }

  const readRustSelectorUsagePayload = usesRustSelectorUsageBackend(selectedQueryBackend)
    ? createSelectorUsagePayloadReader(options)
    : null;
  for (const styleFile of options.styleFiles) {
    const styleDocument = options.styleDocumentForPath(styleFile);
    if (!styleDocument) continue;
    for (const selector of listCanonicalSelectors(styleDocument)) {
      const rustSelectorUsagePayload = readRustSelectorUsagePayload
        ? readRustSelectorUsagePayload(options, styleFile, selector.canonicalName)
        : null;
      if (rustSelectorUsagePayload) {
        results.push(
          selectorUsageResultV2FromRustPayload(
            styleFile,
            selector.canonicalName,
            rustSelectorUsagePayload,
          ),
        );
        continue;
      }

      const usage = readSelectorUsageSummary(
        {
          semanticReferenceIndex: options.semanticReferenceIndex,
          styleDependencyGraph: options.styleDependencyGraph,
          styleDocumentForPath: options.styleDocumentForPath,
        },
        styleFile,
        selector.canonicalName,
      );
      results.push(selectorUsageResultV2(styleFile, selector.canonicalName, usage));
    }
  }

  return results.toSorted(
    (a, b) =>
      a.filePath.localeCompare(b.filePath) ||
      a.kind.localeCompare(b.kind) ||
      a.queryId.localeCompare(b.queryId),
  );
}

function createSourceResolutionPayloadReader(
  document: SourceDocumentSnapshot,
  options: BuildSelectedQueryResultsV2Options,
): typeof resolveRustSourceResolutionPayload {
  if (options.readRustSourceResolutionPayload) {
    return options.readRustSourceResolutionPayload;
  }

  const readPayloads =
    options.readRustSourceResolutionPayloads ?? resolveRustSourceResolutionPayloads;
  const payloadsByStylePath = new Map<
    string,
    ReadonlyMap<string, SourceResolutionEvaluatorCandidatePayloadV0>
  >();

  return (_document, expressionId, scssModulePath, deps) => {
    let payloadsByExpressionId = payloadsByStylePath.get(scssModulePath);
    if (!payloadsByExpressionId) {
      const payloads = readPayloads(document, scssModulePath, deps);
      payloadsByExpressionId = new Map(
        payloads.map((payload) => [payload.expressionId, payload] as const),
      );
      payloadsByStylePath.set(scssModulePath, payloadsByExpressionId);
    }

    return payloadsByExpressionId.get(expressionId) ?? null;
  };
}

function createExpressionSemanticsPayloadReader(
  document: SourceDocumentSnapshot,
  options: BuildSelectedQueryResultsV2Options,
): typeof resolveRustExpressionSemanticsPayload {
  if (options.readRustExpressionSemanticsPayload) {
    return options.readRustExpressionSemanticsPayload;
  }

  const readPayloads =
    options.readRustExpressionSemanticsPayloads ?? resolveRustExpressionSemanticsPayloads;
  const payloadsByStylePath = new Map<
    string,
    ReadonlyMap<string, ExpressionSemanticsEvaluatorCandidatePayloadV0>
  >();

  return (_document, expressionId, scssModulePath, deps) => {
    let payloadsByExpressionId = payloadsByStylePath.get(scssModulePath);
    if (!payloadsByExpressionId) {
      const payloads = readPayloads(document, scssModulePath, deps);
      payloadsByExpressionId = new Map(
        payloads.map((payload) => [payload.expressionId, payload] as const),
      );
      payloadsByStylePath.set(scssModulePath, payloadsByExpressionId);
    }

    return payloadsByExpressionId.get(expressionId) ?? null;
  };
}

function selectorUsageResultV2FromRustPayload(
  filePath: string,
  canonicalName: string,
  payload: ReturnType<typeof resolveRustSelectorUsagePayload> extends infer T
    ? NonNullable<T>
    : never,
): SelectorUsageQueryResultV2 {
  return {
    kind: "selector-usage",
    filePath,
    queryId: canonicalName,
    payload: {
      canonicalName: payload.canonicalName,
      totalReferences: payload.totalReferences,
      directReferenceCount: payload.directReferenceCount,
      editableDirectReferenceCount: payload.editableDirectReferenceCount,
      exactReferenceCount: payload.exactReferenceCount,
      inferredOrBetterReferenceCount: payload.inferredOrBetterReferenceCount,
      hasExpandedReferences: payload.hasExpandedReferences,
      hasStyleDependencyReferences: payload.hasStyleDependencyReferences,
      hasAnyReferences: payload.hasAnyReferences,
    },
  };
}

function createSelectorUsagePayloadReader(
  options: BuildSelectedQueryResultsV2Options,
): typeof resolveRustSelectorUsagePayload {
  if (options.readRustSelectorUsagePayload) {
    return options.readRustSelectorUsagePayload;
  }

  const readPayloads = options.readRustSelectorUsagePayloads ?? resolveRustSelectorUsagePayloads;
  let payloadsBySelector: ReadonlyMap<string, SelectorUsageEvaluatorCandidateV0> | null = null;

  return (_options, filePath, canonicalName) => {
    if (!payloadsBySelector) {
      payloadsBySelector = new Map(
        readPayloads(options).map((candidate) => [
          selectorUsagePayloadKey(candidate.filePath, candidate.queryId),
          candidate,
        ]),
      );
    }

    return (
      payloadsBySelector.get(selectorUsagePayloadKey(filePath, canonicalName))?.payload ?? null
    );
  };
}

function selectorUsagePayloadKey(filePath: string, canonicalName: string): string {
  return `${filePath}\u0000${canonicalName}`;
}

function expressionSemanticsResultV2FromRustPayload(
  filePath: string,
  expression: Parameters<typeof buildExpressionSemanticsSummaryFromRustPayload>[0],
  payload: Parameters<typeof buildExpressionSemanticsSummaryFromRustPayload>[3],
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
): ExpressionSemanticsQueryResultV2 {
  const styleDocument = payload.styleFilePath ? styleDocumentForPath(payload.styleFilePath) : null;
  const selectors =
    styleDocument?.selectors.filter((selector) =>
      payload.selectorNames.includes(selector.canonicalName),
    ) ?? [];
  const semantics = buildExpressionSemanticsSummaryFromRustPayload(
    expression,
    styleDocument,
    selectors,
    payload,
  );
  return expressionSemanticsResultV2(filePath, semantics);
}

function sourceExpressionResolutionResultV2FromRustPayload(
  filePath: string,
  expressionId: string,
  payload: Parameters<typeof buildSourceResolutionSummaryFromRustPayload>[2],
  styleDocumentForPath: (filePath: string) => StyleDocumentHIR | null,
): SourceExpressionResolutionQueryResultV2 {
  const styleDocument = payload.styleFilePath ? styleDocumentForPath(payload.styleFilePath) : null;
  const selectors =
    styleDocument?.selectors.filter((selector) =>
      payload.selectorNames.includes(selector.canonicalName),
    ) ?? [];
  const resolution = buildSourceResolutionSummaryFromRustPayload(styleDocument, selectors, payload);
  return sourceExpressionResolutionResultV2(filePath, expressionId, resolution);
}

function expressionSemanticsResultV2(
  filePath: string,
  semantics: ReturnType<typeof readExpressionSemantics>,
): ExpressionSemanticsQueryResultV2 {
  const valueCertaintyProfile = deriveValueCertaintyProfileV2(
    semantics.abstractValue,
    semantics.valueCertainty,
  );
  const selectorCertaintyProfile = deriveSelectorCertaintyProfileV2(
    semantics.selectorNames.length,
    semantics.selectorCertainty,
    semantics.abstractValue,
  );
  const valueDomain = classifyValueDomainV2(semantics.abstractValue);
  const valueDomainReason = describeAbstractValueReason(semantics.abstractValue);
  const valueCertaintyReason = describeValueCertaintyReason(
    semantics.abstractValue,
    semantics.valueCertainty,
    semantics.reason,
  );
  const selectorCertaintyReason = describeSelectorCertaintyReason(
    semantics.abstractValue,
    semantics.selectorCertainty,
    semantics.selectorNames.length,
  );
  return {
    kind: "expression-semantics",
    filePath,
    queryId: semantics.expression.id,
    payload: {
      expressionId: semantics.expression.id,
      expressionKind: semantics.expression.kind,
      styleFilePath: semantics.styleDocument?.filePath ?? null,
      selectorNames: semantics.selectorNames,
      candidateNames: semantics.candidateNames,
      finiteValues: semantics.finiteValues,
      valueDomainKind: valueDomain.kind,
      ...(valueDomain.constraintKind ? { valueConstraintKind: valueDomain.constraintKind } : {}),
      ...(valueDomain.prefix ? { valuePrefix: valueDomain.prefix } : {}),
      ...(valueDomain.suffix ? { valueSuffix: valueDomain.suffix } : {}),
      ...(valueDomain.minLen !== undefined ? { valueMinLen: valueDomain.minLen } : {}),
      ...(valueDomain.maxLen !== undefined ? { valueMaxLen: valueDomain.maxLen } : {}),
      ...(valueDomain.charMust ? { valueCharMust: valueDomain.charMust } : {}),
      ...(valueDomain.charMay ? { valueCharMay: valueDomain.charMay } : {}),
      ...(valueDomain.mayIncludeOtherChars ? { valueMayIncludeOtherChars: true } : {}),
      ...(valueDomainReason ? { valueDomainReason } : {}),
      ...(semantics.valueDomainDerivation
        ? { valueDomainDerivation: semantics.valueDomainDerivation }
        : {}),
      ...(semantics.valueDomainProvenanceTree
        ? { valueDomainProvenanceTree: semantics.valueDomainProvenanceTree }
        : {}),
      selectorCertainty: semantics.selectorCertainty,
      ...(selectorCertaintyProfile
        ? { selectorCertaintyShapeKind: selectorCertaintyProfile.shapeKind }
        : {}),
      ...(selectorCertaintyProfile?.selectorConstraintKind
        ? { selectorConstraintKind: selectorCertaintyProfile.selectorConstraintKind }
        : {}),
      ...(selectorCertaintyProfile
        ? { selectorCertaintyShapeLabel: selectorCertaintyProfile.shapeLabel }
        : {}),
      ...(selectorCertaintyReason ? { selectorCertaintyReason } : {}),
      ...(semantics.valueCertainty ? { valueCertainty: semantics.valueCertainty } : {}),
      ...(valueCertaintyProfile
        ? { valueCertaintyShapeKind: valueCertaintyProfile.shapeKind }
        : {}),
      ...(valueCertaintyProfile?.valueConstraintKind
        ? { valueCertaintyConstraintKind: valueCertaintyProfile.valueConstraintKind }
        : {}),
      ...(valueCertaintyProfile
        ? { valueCertaintyShapeLabel: valueCertaintyProfile.shapeLabel }
        : {}),
      ...(valueCertaintyReason ? { valueCertaintyReason } : {}),
      ...(semantics.reason ? { reason: semantics.reason } : {}),
    },
  };
}

function sourceExpressionResolutionResultV2(
  filePath: string,
  expressionId: string,
  resolution: ReturnType<typeof readSourceExpressionResolution>,
): SourceExpressionResolutionQueryResultV2 {
  const valueCertaintyProfile = deriveValueCertaintyProfileV2(
    resolution.abstractValue,
    resolution.valueCertainty,
  );
  const valueDomain = classifyValueDomainV2(resolution.abstractValue);
  const selectorCertaintyProfile = deriveSelectorCertaintyProfileV2(
    resolution.selectors.length,
    resolution.selectorCertainty,
    resolution.abstractValue,
  );
  const valueCertaintyReason = describeValueCertaintyReason(
    resolution.abstractValue,
    resolution.valueCertainty,
    resolution.reason,
  );
  const selectorCertaintyReason = describeSelectorCertaintyReason(
    resolution.abstractValue,
    resolution.selectorCertainty,
    resolution.selectors.length,
  );
  return {
    kind: "source-expression-resolution",
    filePath,
    queryId: expressionId,
    payload: {
      expressionId,
      styleFilePath: resolution.styleDocument?.filePath ?? null,
      selectorNames: resolution.selectors.map((selector) => selector.name),
      finiteValues: resolution.finiteValues,
      selectorCertainty: resolution.selectorCertainty,
      ...(selectorCertaintyProfile
        ? { selectorCertaintyShapeKind: selectorCertaintyProfile.shapeKind }
        : {}),
      ...(selectorCertaintyProfile?.selectorConstraintKind
        ? { selectorConstraintKind: selectorCertaintyProfile.selectorConstraintKind }
        : {}),
      ...(selectorCertaintyProfile
        ? { selectorCertaintyShapeLabel: selectorCertaintyProfile.shapeLabel }
        : {}),
      ...(selectorCertaintyReason ? { selectorCertaintyReason } : {}),
      ...(resolution.valueCertainty ? { valueCertainty: resolution.valueCertainty } : {}),
      ...(valueCertaintyProfile
        ? { valueCertaintyShapeKind: valueCertaintyProfile.shapeKind }
        : {}),
      ...(valueCertaintyProfile?.valueConstraintKind
        ? { valueCertaintyConstraintKind: valueCertaintyProfile.valueConstraintKind }
        : {}),
      ...(valueDomain.prefix ? { valuePrefix: valueDomain.prefix } : {}),
      ...(valueDomain.suffix ? { valueSuffix: valueDomain.suffix } : {}),
      ...(valueDomain.minLen !== undefined ? { valueMinLen: valueDomain.minLen } : {}),
      ...(valueDomain.maxLen !== undefined ? { valueMaxLen: valueDomain.maxLen } : {}),
      ...(valueDomain.charMust ? { valueCharMust: valueDomain.charMust } : {}),
      ...(valueDomain.charMay ? { valueCharMay: valueDomain.charMay } : {}),
      ...(valueDomain.mayIncludeOtherChars ? { valueMayIncludeOtherChars: true } : {}),
      ...(valueCertaintyProfile
        ? { valueCertaintyShapeLabel: valueCertaintyProfile.shapeLabel }
        : {}),
      ...(valueCertaintyReason ? { valueCertaintyReason } : {}),
      ...(resolution.reason ? { reason: resolution.reason } : {}),
    },
  };
}

function selectorUsageResultV2(
  filePath: string,
  canonicalName: string,
  usage: ReturnType<typeof readSelectorUsageSummary>,
): SelectorUsageQueryResultV2 {
  return {
    kind: "selector-usage",
    filePath,
    queryId: canonicalName,
    payload: {
      canonicalName,
      totalReferences: usage.totalReferences,
      directReferenceCount: usage.directReferenceCount,
      editableDirectReferenceCount: usage.editableDirectSites.length,
      exactReferenceCount: usage.exactSites.length,
      inferredOrBetterReferenceCount: usage.inferredOrBetterSites.length,
      hasExpandedReferences: usage.hasExpandedReferences,
      hasStyleDependencyReferences: usage.hasStyleDependencyReferences,
      hasAnyReferences: usage.hasAnyReferences,
    },
  };
}

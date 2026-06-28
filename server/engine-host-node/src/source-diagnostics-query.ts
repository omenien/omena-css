import {
  checkSourceDocument,
  type SourceCheckerFinding,
} from "../../engine-core-ts/src/core/checker";
import { enumerateFiniteClassValues } from "../../engine-core-ts/src/core/abstract-value/class-value-domain";
import { findInvalidClassReference } from "../../engine-core-ts/src/core/query";
import type { DocumentParams, ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import {
  buildExpressionSemanticsSummaryFromRustPayload,
  resolveRustExpressionSemanticsPayloadsAsync,
  resolveRustExpressionSemanticsPayloads,
  type ExpressionSemanticsEvaluatorCandidatePayloadV0,
  type resolveRustExpressionSemanticsPayloadAsync,
  type resolveRustExpressionSemanticsPayload,
} from "./expression-semantics-query-backend";
import {
  indexExpressionDomainSelectorProjectionsForStyle,
  resolveRustExpressionDomainSelectorProjections,
  resolveRustExpressionDomainSelectorProjectionsAsync,
  type ExpressionDomainSelectorProjectionEntryV0,
  withExpressionDomainSelectorProjection,
} from "./expression-domain-selector-projection-query-backend";
import {
  resolveSelectedQueryBackendKind,
  usesRustExpressionSemanticsBackend,
} from "./selected-query-backend";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-backend";

export interface SourceDiagnosticsQueryOptions {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustExpressionSemanticsPayload?: typeof resolveRustExpressionSemanticsPayload;
  readonly readRustExpressionSemanticsPayloads?: typeof resolveRustExpressionSemanticsPayloads;
  readonly readRustExpressionSemanticsPayloadAsync?: typeof resolveRustExpressionSemanticsPayloadAsync;
  readonly readRustExpressionSemanticsPayloadsAsync?: typeof resolveRustExpressionSemanticsPayloadsAsync;
  readonly readRustExpressionDomainSelectorProjections?: typeof resolveRustExpressionDomainSelectorProjections;
  readonly readRustExpressionDomainSelectorProjectionsAsync?: typeof resolveRustExpressionDomainSelectorProjectionsAsync;
  readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
}

export function resolveSourceDiagnosticFindings(
  params: DocumentParams,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "logError"
  >,
  options: SourceDiagnosticsQueryOptions = {},
): readonly SourceCheckerFinding[] {
  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustExpressionSemanticsBackend(selectedQueryBackend)) {
    const readRustSelectorProjection =
      selectedQueryBackend === "rust-selected-query"
        ? createRustExpressionDomainSelectorProjectionReader(params, deps, options)
        : NO_RUST_SELECTOR_PROJECTION_READER;
    return resolveSourceDiagnosticFindingsViaRustSemantics(
      params,
      deps,
      createRustExpressionSemanticsPayloadReader(params, deps, options),
      readRustSelectorProjection,
    );
  }

  return checkSourceDocument(
    params,
    {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      workspaceRoot: deps.workspaceRoot,
    },
    {
      includeMissingModule: deps.settings.diagnostics.missingModule,
      logError: deps.logError,
    },
  );
}

export async function resolveSourceDiagnosticFindingsAsync(
  params: DocumentParams,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "logError"
  >,
  options: SourceDiagnosticsQueryOptions = {},
): Promise<readonly SourceCheckerFinding[]> {
  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustExpressionSemanticsBackend(selectedQueryBackend)) {
    const readRustSelectorProjection =
      selectedQueryBackend === "rust-selected-query"
        ? createRustExpressionDomainSelectorProjectionReaderAsync(params, deps, options)
        : NO_RUST_SELECTOR_PROJECTION_READER_ASYNC;
    return resolveSourceDiagnosticFindingsViaRustSemanticsAsync(
      params,
      deps,
      createRustExpressionSemanticsPayloadReaderAsync(params, deps, options),
      readRustSelectorProjection,
    );
  }

  return checkSourceDocument(
    params,
    {
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      workspaceRoot: deps.workspaceRoot,
    },
    {
      includeMissingModule: deps.settings.diagnostics.missingModule,
      logError: deps.logError,
    },
  );
}

function resolveSourceDiagnosticFindingsViaRustSemantics(
  params: DocumentParams,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "logError"
  >,
  readRustSemanticsPayload: typeof resolveRustExpressionSemanticsPayload,
  readRustSelectorProjection: resolveRustExpressionDomainSelectorProjection,
): readonly SourceCheckerFinding[] {
  const entry = deps.analysisCache.get(
    params.documentUri,
    params.content,
    params.filePath,
    params.version,
  );
  const findings: SourceCheckerFinding[] = [];

  if (deps.settings.diagnostics.missingModule) {
    for (const imp of entry.stylesBindings.values()) {
      if (imp.kind !== "missing") continue;
      findings.push({
        category: "source",
        code: "missing-module",
        severity: "warning",
        range: imp.range,
        specifier: imp.specifier,
        absolutePath: imp.absolutePath,
      });
    }
  }

  for (const expression of entry.sourceDocument.classExpressions) {
    if (expression.origin !== "cxCall") continue;
    try {
      const styleDocument = deps.styleDocumentForPath(expression.scssModulePath);
      if (!styleDocument) continue;

      if (expression.kind !== "symbolRef") {
        const finding = findInvalidClassReference(expression, styleDocument, {
          typeResolver: deps.typeResolver,
          filePath: params.filePath,
          workspaceRoot: deps.workspaceRoot,
          sourceBinder: entry.sourceBinder,
          sourceBindingGraph: entry.sourceBindingGraph,
          classValueUniverses: entry.classValueUniverses,
        });
        if (!finding) continue;
        findings.push(mapInvalidClassFinding(finding, styleDocument.filePath));
        continue;
      }

      const fallbackFinding = createFallbackFindingReader({
        expression,
        styleDocument,
        sourceBinder: entry.sourceBinder,
        sourceBindingGraph: entry.sourceBindingGraph,
        classValueUniverses: entry.classValueUniverses,
        deps,
        filePath: params.filePath,
      });
      const rawPayload = readRustSemanticsPayload(
        {
          uri: params.documentUri,
          content: params.content,
          filePath: params.filePath,
          version: params.version,
        },
        expression.id,
        expression.scssModulePath,
        deps,
      );
      const projection =
        rawPayload && rawPayload.selectorNames.length === 0
          ? readRustSelectorProjection(
              {
                uri: params.documentUri,
                content: params.content,
                filePath: params.filePath,
                version: params.version,
              },
              expression.id,
              expression.scssModulePath,
              deps,
            )
          : null;
      const payload = rawPayload
        ? withExpressionDomainSelectorProjection(rawPayload, projection)
        : rawPayload;
      if (!payload || !payload.styleFilePath) {
        const fallback = fallbackFinding();
        if (fallback) {
          findings.push(mapInvalidClassFinding(fallback, styleDocument.filePath));
        }
        continue;
      }

      const payloadStyleDocument = deps.styleDocumentForPath(payload.styleFilePath);
      if (!payloadStyleDocument) {
        const fallback = fallbackFinding();
        if (fallback) {
          findings.push(mapInvalidClassFinding(fallback, styleDocument.filePath));
        }
        continue;
      }
      const selectors =
        payloadStyleDocument.selectors.filter((selector) =>
          payload.selectorNames.includes(selector.name),
        ) ?? [];
      const semantics = buildExpressionSemanticsSummaryFromRustPayload(
        expression,
        payloadStyleDocument,
        selectors,
        payload,
      );
      if (!semantics.abstractValue || !semantics.reason || !semantics.valueCertainty) {
        const fallback = fallbackFinding();
        if (fallback) {
          findings.push(mapInvalidClassFinding(fallback, styleDocument.filePath));
        }
        continue;
      }

      const finiteValues =
        semantics.finiteValues ?? enumerateFiniteClassValues(semantics.abstractValue);
      if (!finiteValues) {
        if (semantics.selectors.length > 0) continue;
        findings.push({
          category: "source",
          code: "missing-resolved-class-domain",
          severity: "warning",
          range: expression.range,
          scssModulePath: payloadStyleDocument.filePath,
          abstractValue: semantics.abstractValue,
          valueCertainty: semantics.valueCertainty,
          selectorCertainty: semantics.selectorCertainty,
          reason: semantics.reason,
          ...(semantics.valueDomainDerivation
            ? { valueDomainDerivation: semantics.valueDomainDerivation }
            : {}),
        });
        continue;
      }

      const missingValues = finiteValues.filter(
        (value) => !payloadStyleDocument.selectors.some((selector) => selector.name === value),
      );
      if (missingValues.length === 0) continue;
      findings.push({
        category: "source",
        code: "missing-resolved-class-values",
        severity: "warning",
        range: expression.range,
        scssModulePath: payloadStyleDocument.filePath,
        missingValues,
        abstractValue: semantics.abstractValue,
        valueCertainty: semantics.valueCertainty,
        selectorCertainty: semantics.selectorCertainty,
        reason: semantics.reason,
        ...(semantics.valueDomainDerivation
          ? { valueDomainDerivation: semantics.valueDomainDerivation }
          : {}),
      });
    } catch (err) {
      deps.logError("diagnostics per-call validation failed", err);
    }
  }

  return findings;
}

async function resolveSourceDiagnosticFindingsViaRustSemanticsAsync(
  params: DocumentParams,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "logError"
  >,
  readRustSemanticsPayload: (
    document: Parameters<typeof resolveRustExpressionSemanticsPayloadAsync>[0],
    expressionId: string,
    scssModulePath: string,
    deps: Parameters<typeof resolveRustExpressionSemanticsPayloadAsync>[3],
  ) => Promise<ExpressionSemanticsEvaluatorCandidatePayloadV0 | null>,
  readRustSelectorProjection: (
    document: Parameters<typeof resolveRustExpressionSemanticsPayloadAsync>[0],
    expressionId: string,
    scssModulePath: string,
    deps: Parameters<typeof resolveRustExpressionSemanticsPayloadAsync>[3],
  ) => Promise<ExpressionDomainSelectorProjectionEntryV0 | null>,
): Promise<readonly SourceCheckerFinding[]> {
  const entry = deps.analysisCache.get(
    params.documentUri,
    params.content,
    params.filePath,
    params.version,
  );
  const findings: SourceCheckerFinding[] = [];

  if (deps.settings.diagnostics.missingModule) {
    for (const imp of entry.stylesBindings.values()) {
      if (imp.kind !== "missing") continue;
      findings.push({
        category: "source",
        code: "missing-module",
        severity: "warning",
        range: imp.range,
        specifier: imp.specifier,
        absolutePath: imp.absolutePath,
      });
    }
  }

  const expressionFindings = await Promise.all(
    entry.sourceDocument.classExpressions.map(
      async (expression): Promise<SourceCheckerFinding[]> => {
        if (expression.origin !== "cxCall") return [];
        try {
          const styleDocument = deps.styleDocumentForPath(expression.scssModulePath);
          if (!styleDocument) return [];

          if (expression.kind !== "symbolRef") {
            const finding = findInvalidClassReference(expression, styleDocument, {
              typeResolver: deps.typeResolver,
              filePath: params.filePath,
              workspaceRoot: deps.workspaceRoot,
              sourceBinder: entry.sourceBinder,
              sourceBindingGraph: entry.sourceBindingGraph,
              classValueUniverses: entry.classValueUniverses,
            });
            if (!finding) return [];
            return [mapInvalidClassFinding(finding, styleDocument.filePath)];
          }

          const fallbackFinding = createFallbackFindingReader({
            expression,
            styleDocument,
            sourceBinder: entry.sourceBinder,
            sourceBindingGraph: entry.sourceBindingGraph,
            classValueUniverses: entry.classValueUniverses,
            deps,
            filePath: params.filePath,
          });
          const rawPayload = await readRustSemanticsPayload(
            {
              uri: params.documentUri,
              content: params.content,
              filePath: params.filePath,
              version: params.version,
            },
            expression.id,
            expression.scssModulePath,
            deps,
          );
          const projection =
            rawPayload && rawPayload.selectorNames.length === 0
              ? await readRustSelectorProjection(
                  {
                    uri: params.documentUri,
                    content: params.content,
                    filePath: params.filePath,
                    version: params.version,
                  },
                  expression.id,
                  expression.scssModulePath,
                  deps,
                )
              : null;
          const payload = rawPayload
            ? withExpressionDomainSelectorProjection(rawPayload, projection)
            : rawPayload;
          if (!payload || !payload.styleFilePath) {
            const fallback = fallbackFinding();
            return fallback ? [mapInvalidClassFinding(fallback, styleDocument.filePath)] : [];
          }

          const payloadStyleDocument = deps.styleDocumentForPath(payload.styleFilePath);
          if (!payloadStyleDocument) {
            const fallback = fallbackFinding();
            return fallback ? [mapInvalidClassFinding(fallback, styleDocument.filePath)] : [];
          }
          const selectors =
            payloadStyleDocument.selectors.filter((selector) =>
              payload.selectorNames.includes(selector.name),
            ) ?? [];
          const semantics = buildExpressionSemanticsSummaryFromRustPayload(
            expression,
            payloadStyleDocument,
            selectors,
            payload,
          );
          if (!semantics.abstractValue || !semantics.reason || !semantics.valueCertainty) {
            const fallback = fallbackFinding();
            return fallback ? [mapInvalidClassFinding(fallback, styleDocument.filePath)] : [];
          }

          const finiteValues =
            semantics.finiteValues ?? enumerateFiniteClassValues(semantics.abstractValue);
          if (!finiteValues) {
            if (semantics.selectors.length > 0) return [];
            return [
              {
                category: "source",
                code: "missing-resolved-class-domain",
                severity: "warning",
                range: expression.range,
                scssModulePath: payloadStyleDocument.filePath,
                abstractValue: semantics.abstractValue,
                valueCertainty: semantics.valueCertainty,
                selectorCertainty: semantics.selectorCertainty,
                reason: semantics.reason,
                ...(semantics.valueDomainDerivation
                  ? { valueDomainDerivation: semantics.valueDomainDerivation }
                  : {}),
              },
            ];
          }

          const missingValues = finiteValues.filter(
            (value) => !payloadStyleDocument.selectors.some((selector) => selector.name === value),
          );
          if (missingValues.length === 0) return [];
          return [
            {
              category: "source",
              code: "missing-resolved-class-values",
              severity: "warning",
              range: expression.range,
              scssModulePath: payloadStyleDocument.filePath,
              missingValues,
              abstractValue: semantics.abstractValue,
              valueCertainty: semantics.valueCertainty,
              selectorCertainty: semantics.selectorCertainty,
              reason: semantics.reason,
              ...(semantics.valueDomainDerivation
                ? { valueDomainDerivation: semantics.valueDomainDerivation }
                : {}),
            },
          ];
        } catch (err) {
          deps.logError("diagnostics per-call validation failed", err);
          return [];
        }
      },
    ),
  );
  findings.push(...expressionFindings.flat());

  return findings;
}

function createRustExpressionSemanticsPayloadReader(
  params: DocumentParams,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  options: SourceDiagnosticsQueryOptions,
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

  return (_document, expressionId, scssModulePath) => {
    let payloadsByExpressionId = payloadsByStylePath.get(scssModulePath);
    if (!payloadsByExpressionId) {
      const payloads = readPayloads(
        {
          uri: params.documentUri,
          content: params.content,
          filePath: params.filePath,
          version: params.version,
        },
        scssModulePath,
        deps,
      );
      payloadsByExpressionId = new Map(
        payloads.map((payload) => [payload.expressionId, payload] as const),
      );
      payloadsByStylePath.set(scssModulePath, payloadsByExpressionId);
    }

    return payloadsByExpressionId.get(expressionId) ?? null;
  };
}

type resolveRustExpressionDomainSelectorProjection = (
  document: Parameters<typeof resolveRustExpressionSemanticsPayload>[0],
  expressionId: string,
  scssModulePath: string,
  deps: Parameters<typeof resolveRustExpressionSemanticsPayload>[3],
) => ExpressionDomainSelectorProjectionEntryV0 | null;

const NO_RUST_SELECTOR_PROJECTION_READER: resolveRustExpressionDomainSelectorProjection = () =>
  null;

const NO_RUST_SELECTOR_PROJECTION_READER_ASYNC = async () => null;

function createRustExpressionDomainSelectorProjectionReader(
  params: DocumentParams,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  options: SourceDiagnosticsQueryOptions,
): resolveRustExpressionDomainSelectorProjection {
  const readProjections =
    options.readRustExpressionDomainSelectorProjections ??
    resolveRustExpressionDomainSelectorProjections;
  const projectionsByStylePath = new Map<
    string,
    ReadonlyMap<string, ExpressionDomainSelectorProjectionEntryV0>
  >();

  return (_document, expressionId, scssModulePath) => {
    let projectionsByNodeId = projectionsByStylePath.get(scssModulePath);
    if (!projectionsByNodeId) {
      const projections = readProjections(
        {
          uri: params.documentUri,
          content: params.content,
          filePath: params.filePath,
          version: params.version,
        },
        scssModulePath,
        deps,
      );
      projectionsByNodeId = indexExpressionDomainSelectorProjectionsForStyle(
        projections,
        scssModulePath,
      );
      projectionsByStylePath.set(scssModulePath, projectionsByNodeId);
    }

    return projectionsByNodeId.get(expressionId) ?? null;
  };
}

function createRustExpressionSemanticsPayloadReaderAsync(
  params: DocumentParams,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  options: SourceDiagnosticsQueryOptions,
): (
  document: Parameters<typeof resolveRustExpressionSemanticsPayloadAsync>[0],
  expressionId: string,
  scssModulePath: string,
  deps: Parameters<typeof resolveRustExpressionSemanticsPayloadAsync>[3],
) => Promise<ExpressionSemanticsEvaluatorCandidatePayloadV0 | null> {
  if (options.readRustExpressionSemanticsPayloadAsync) {
    return options.readRustExpressionSemanticsPayloadAsync;
  }

  const readPayloads =
    options.readRustExpressionSemanticsPayloadsAsync ?? resolveRustExpressionSemanticsPayloadsAsync;
  const payloadsByStylePath = new Map<
    string,
    Promise<ReadonlyMap<string, ExpressionSemanticsEvaluatorCandidatePayloadV0>>
  >();

  return async (_document, expressionId, scssModulePath) => {
    let payloadsByExpressionId = payloadsByStylePath.get(scssModulePath);
    if (!payloadsByExpressionId) {
      payloadsByExpressionId = readPayloads(
        {
          uri: params.documentUri,
          content: params.content,
          filePath: params.filePath,
          version: params.version,
        },
        scssModulePath,
        deps,
        options.runRustSelectedQueryBackendJsonAsync,
      ).then(
        (payloads) => new Map(payloads.map((payload) => [payload.expressionId, payload] as const)),
      );
      payloadsByStylePath.set(scssModulePath, payloadsByExpressionId);
    }

    return (await payloadsByExpressionId).get(expressionId) ?? null;
  };
}

function createRustExpressionDomainSelectorProjectionReaderAsync(
  params: DocumentParams,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  options: SourceDiagnosticsQueryOptions,
): (
  document: Parameters<typeof resolveRustExpressionSemanticsPayloadAsync>[0],
  expressionId: string,
  scssModulePath: string,
  deps: Parameters<typeof resolveRustExpressionSemanticsPayloadAsync>[3],
) => Promise<ExpressionDomainSelectorProjectionEntryV0 | null> {
  const readProjections =
    options.readRustExpressionDomainSelectorProjectionsAsync ??
    resolveRustExpressionDomainSelectorProjectionsAsync;
  const projectionsByStylePath = new Map<
    string,
    Promise<ReadonlyMap<string, ExpressionDomainSelectorProjectionEntryV0>>
  >();

  return async (_document, expressionId, scssModulePath) => {
    let projectionsByNodeId = projectionsByStylePath.get(scssModulePath);
    if (!projectionsByNodeId) {
      projectionsByNodeId = readProjections(
        {
          uri: params.documentUri,
          content: params.content,
          filePath: params.filePath,
          version: params.version,
        },
        scssModulePath,
        deps,
        options.runRustSelectedQueryBackendJsonAsync,
      ).then((projections) =>
        indexExpressionDomainSelectorProjectionsForStyle(projections, scssModulePath),
      );
      projectionsByStylePath.set(scssModulePath, projectionsByNodeId);
    }

    return (await projectionsByNodeId).get(expressionId) ?? null;
  };
}

function createFallbackFindingReader(args: {
  readonly expression: Parameters<typeof findInvalidClassReference>[0];
  readonly styleDocument: Parameters<typeof findInvalidClassReference>[1];
  readonly sourceBinder: Parameters<typeof findInvalidClassReference>[2]["sourceBinder"];
  readonly sourceBindingGraph: Parameters<
    typeof findInvalidClassReference
  >[2]["sourceBindingGraph"];
  readonly classValueUniverses: Parameters<
    typeof findInvalidClassReference
  >[2]["classValueUniverses"];
  readonly deps: Pick<ProviderDeps, "typeResolver" | "workspaceRoot">;
  readonly filePath: string;
}): () => ReturnType<typeof findInvalidClassReference> {
  let didRead = false;
  let fallback: ReturnType<typeof findInvalidClassReference> = null;
  return () => {
    if (!didRead) {
      didRead = true;
      fallback = findInvalidClassReference(args.expression, args.styleDocument, {
        typeResolver: args.deps.typeResolver,
        filePath: args.filePath,
        workspaceRoot: args.deps.workspaceRoot,
        ...(args.sourceBinder !== undefined ? { sourceBinder: args.sourceBinder } : {}),
        ...(args.sourceBindingGraph !== undefined
          ? { sourceBindingGraph: args.sourceBindingGraph }
          : {}),
        ...(args.classValueUniverses !== undefined
          ? { classValueUniverses: args.classValueUniverses }
          : {}),
      });
    }
    return fallback;
  };
}

function mapInvalidClassFinding(
  finding: NonNullable<ReturnType<typeof findInvalidClassReference>>,
  scssModulePath: string,
): SourceCheckerFinding {
  switch (finding.kind) {
    case "missingStaticClass":
      return {
        category: "source",
        code: "missing-static-class",
        severity: "warning",
        range: finding.range,
        scssModulePath,
        className: finding.expression.className,
        ...(finding.suggestion ? { suggestion: finding.suggestion } : {}),
      };
    case "missingTemplatePrefix":
      return {
        category: "source",
        code: "missing-template-prefix",
        severity: "warning",
        range: finding.range,
        scssModulePath,
        staticPrefix: finding.expression.staticPrefix,
      };
    case "missingResolvedClassValues":
      return {
        category: "source",
        code: "missing-resolved-class-values",
        severity: "warning",
        range: finding.range,
        scssModulePath,
        missingValues: finding.missingValues,
        abstractValue: finding.abstractValue,
        valueCertainty: finding.valueCertainty,
        selectorCertainty: finding.selectorCertainty,
        reason: finding.reason,
        ...(finding.valueDomainDerivation
          ? { valueDomainDerivation: finding.valueDomainDerivation }
          : {}),
      };
    case "missingResolvedClassDomain":
      return {
        category: "source",
        code: "missing-resolved-class-domain",
        severity: "warning",
        range: finding.range,
        scssModulePath,
        abstractValue: finding.abstractValue,
        valueCertainty: finding.valueCertainty,
        selectorCertainty: finding.selectorCertainty,
        reason: finding.reason,
        ...(finding.valueDomainDerivation
          ? { valueDomainDerivation: finding.valueDomainDerivation }
          : {}),
      };
    default:
      finding satisfies never;
      return finding;
  }
}

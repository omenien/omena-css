import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import type {
  SelectorCertaintyShapeKindV2,
  StringConstraintKindV2,
  ValueCertaintyShapeKindV2,
  ValueDomainKindV2,
} from "../../engine-core-ts/src/contracts";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import type {
  DynamicHoverExplanation,
  ValueDomainProvenanceTree,
} from "../../engine-core-ts/src/core/query";
import {
  buildDynamicExpressionExplanation,
  readExpressionSemantics,
  readSourceExpressionContextAtCursor,
  readSourceExpressionResolution,
  resolveRefDetails,
} from "../../engine-core-ts/src/core/query";
import {
  deriveSelectorCertaintyProfileV2,
  deriveValueCertaintyProfileV2,
} from "../../engine-core-ts/src/core/semantic/certainty";
import type { ClassnameTransformMode } from "../../engine-core-ts/src/core/scss/classname-transform";
import type { TypeFactBackendKind } from "./type-backend";
import {
  buildExpressionSemanticsSummaryFromRustPayload,
  resolveRustExpressionSemanticsPayload,
  type ReducedClassValueDerivationV0,
} from "./expression-semantics-query-backend";
import {
  indexExpressionDomainSelectorProjectionsForStyle,
  resolveRustExpressionDomainSelectorProjections,
  withExpressionDomainSelectorProjection,
} from "./expression-domain-selector-projection-query-backend";
import {
  createWorkspaceAnalysisHost,
  createWorkspaceStyleHost,
} from "./checker-host/workspace-check-support";
import { classifyValueDomainV2 } from "./query-metadata-v2";
import {
  resolveSelectedQueryBackendKind,
  usesRustExpressionSemanticsBackend,
} from "./selected-query-backend";

export interface ExplainExpressionOptions {
  readonly workspaceRoot: string;
  readonly filePath: string;
  readonly line: number;
  readonly character: number;
  readonly classnameTransform?: ClassnameTransformMode;
  readonly pathAlias?: Readonly<Record<string, string>>;
  readonly typeBackend?: TypeFactBackendKind;
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustExpressionSemanticsPayload?: typeof resolveRustExpressionSemanticsPayload;
  readonly readRustExpressionDomainSelectorProjections?: typeof resolveRustExpressionDomainSelectorProjections;
}

export interface ExplainExpressionResult {
  readonly filePath: string;
  readonly line: number;
  readonly character: number;
  readonly analysisSource: "omena-query" | "typescript-current";
  readonly expressionKind: string;
  readonly styleFilePath: string;
  readonly selectorNames: readonly string[];
  readonly dynamicExplanation: DynamicHoverExplanation | null;
  readonly analysisV2: {
    readonly valueDomainKind: ValueDomainKindV2;
    readonly valueConstraintKind?: StringConstraintKindV2;
    readonly valuePrefix?: string;
    readonly valueSuffix?: string;
    readonly valueMinLen?: number;
    readonly valueMaxLen?: number;
    readonly valueCharMust?: string;
    readonly valueCharMay?: string;
    readonly valueMayIncludeOtherChars?: boolean;
    readonly valueDomainDerivation?: ReducedClassValueDerivationV0;
    readonly valueDomainProvenanceTree?: ValueDomainProvenanceTree;
    readonly valueCertaintyShapeKind?: ValueCertaintyShapeKindV2;
    readonly valueCertaintyConstraintKind?: StringConstraintKindV2;
    readonly selectorCertaintyShapeKind?: SelectorCertaintyShapeKindV2;
    readonly selectorConstraintKind?: StringConstraintKindV2;
  };
}

export function explainExpressionAtLocation(
  options: ExplainExpressionOptions,
): ExplainExpressionResult | null {
  const styleHost = createWorkspaceStyleHost({
    styleFiles: [],
    classnameTransform: options.classnameTransform ?? "asIs",
  });
  const analysisHost = createWorkspaceAnalysisHost({
    workspaceRoot: options.workspaceRoot,
    classnameTransform: options.classnameTransform ?? "asIs",
    pathAlias: options.pathAlias ?? {},
    styleDocumentForPath: styleHost.styleDocumentForPath,
    ...(options.typeBackend ? { typeBackend: options.typeBackend } : {}),
    env: options.env ?? process.env,
  });
  const content = readFileSync(options.filePath, "utf8");
  const documentUri = pathToFileURL(options.filePath).href;
  const ctx = readSourceExpressionContextAtCursor(
    {
      documentUri,
      content,
      filePath: options.filePath,
      version: 1,
      line: options.line,
      character: options.character,
    },
    {
      analysisCache: analysisHost.analysisCache,
      styleDocumentForPath: styleHost.styleDocumentForPath,
    },
  );
  if (!ctx) return null;

  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env ?? process.env);
  if (usesRustExpressionSemanticsBackend(selectedQueryBackend)) {
    const rustResult = resolveExplainExpressionViaRustSemantics(
      options,
      ctx,
      {
        analysisHost,
        styleHost,
        content,
        documentUri,
      },
      options.readRustExpressionSemanticsPayload ?? resolveRustExpressionSemanticsPayload,
      selectedQueryBackend === "rust-selected-query"
        ? (options.readRustExpressionDomainSelectorProjections ??
            resolveRustExpressionDomainSelectorProjections)
        : null,
    );
    if (rustResult) return rustResult;
    return null;
  }

  const resolved = resolveRefDetails(ctx, {
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeResolver: analysisHost.typeResolver,
    filePath: options.filePath,
    workspaceRoot: options.workspaceRoot,
  });
  const expressionResolutionContext = {
    expression: ctx.expression,
    sourceFile: ctx.entry.sourceFile,
    styleDocument: ctx.styleDocument,
  };
  const expressionResolutionEnv = {
    styleDocumentForPath: styleHost.styleDocumentForPath,
    typeResolver: analysisHost.typeResolver,
    filePath: options.filePath,
    workspaceRoot: options.workspaceRoot,
    sourceBinder: ctx.entry.sourceBinder,
    sourceBindingGraph: ctx.entry.sourceBindingGraph,
    classValueUniverses: ctx.entry.classValueUniverses,
  };
  const resolution = readSourceExpressionResolution(
    expressionResolutionContext,
    expressionResolutionEnv,
  );
  const semantics = readExpressionSemantics(expressionResolutionContext, expressionResolutionEnv);
  const valueDomain = classifyValueDomainV2(resolution.abstractValue);
  const valueCertaintyProfile = deriveValueCertaintyProfileV2(
    resolution.abstractValue,
    resolution.valueCertainty,
  );
  const selectorCertaintyProfile = deriveSelectorCertaintyProfileV2(
    resolution.selectors.length,
    resolution.selectorCertainty,
    resolution.abstractValue,
  );

  return {
    filePath: options.filePath,
    line: options.line,
    character: options.character,
    analysisSource: "typescript-current",
    expressionKind: ctx.expression.kind,
    styleFilePath: ctx.expression.scssModulePath,
    selectorNames: resolved.selectors.map((selector) => selector.name),
    dynamicExplanation: resolved.dynamicExplanation,
    analysisV2: {
      valueDomainKind: valueDomain.kind,
      ...(valueDomain.constraintKind ? { valueConstraintKind: valueDomain.constraintKind } : {}),
      ...(valueDomain.prefix ? { valuePrefix: valueDomain.prefix } : {}),
      ...(valueDomain.suffix ? { valueSuffix: valueDomain.suffix } : {}),
      ...(valueDomain.minLen !== undefined ? { valueMinLen: valueDomain.minLen } : {}),
      ...(valueDomain.maxLen !== undefined ? { valueMaxLen: valueDomain.maxLen } : {}),
      ...(valueDomain.charMust ? { valueCharMust: valueDomain.charMust } : {}),
      ...(valueDomain.charMay ? { valueCharMay: valueDomain.charMay } : {}),
      ...(valueDomain.mayIncludeOtherChars ? { valueMayIncludeOtherChars: true } : {}),
      ...(semantics.valueDomainDerivation
        ? { valueDomainDerivation: semantics.valueDomainDerivation }
        : {}),
      ...(semantics.valueDomainProvenanceTree
        ? { valueDomainProvenanceTree: semantics.valueDomainProvenanceTree }
        : {}),
      ...(valueCertaintyProfile
        ? { valueCertaintyShapeKind: valueCertaintyProfile.shapeKind }
        : {}),
      ...(valueCertaintyProfile?.valueConstraintKind
        ? { valueCertaintyConstraintKind: valueCertaintyProfile.valueConstraintKind }
        : {}),
      ...(selectorCertaintyProfile
        ? { selectorCertaintyShapeKind: selectorCertaintyProfile.shapeKind }
        : {}),
      ...(selectorCertaintyProfile?.selectorConstraintKind
        ? { selectorConstraintKind: selectorCertaintyProfile.selectorConstraintKind }
        : {}),
    },
  };
}

function resolveExplainExpressionViaRustSemantics(
  options: ExplainExpressionOptions,
  ctx: NonNullable<ReturnType<typeof readSourceExpressionContextAtCursor>>,
  runtime: {
    readonly analysisHost: ReturnType<typeof createWorkspaceAnalysisHost>;
    readonly styleHost: ReturnType<typeof createWorkspaceStyleHost>;
    readonly content: string;
    readonly documentUri: string;
  },
  readRustSemanticsPayload: typeof resolveRustExpressionSemanticsPayload,
  readRustSelectorProjections: typeof resolveRustExpressionDomainSelectorProjections | null,
): ExplainExpressionResult | null {
  const document = {
    uri: runtime.documentUri,
    content: runtime.content,
    filePath: options.filePath,
    version: 1,
  };
  const deps = {
    analysisCache: runtime.analysisHost.analysisCache,
    styleDocumentForPath: runtime.styleHost.styleDocumentForPath,
    typeResolver: runtime.analysisHost.typeResolver,
    workspaceRoot: options.workspaceRoot,
    settings: {
      scss: {
        classnameTransform: options.classnameTransform ?? "asIs",
      },
      pathAlias: options.pathAlias ?? {},
    },
  } as Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >;
  const rawPayload = readRustSemanticsPayload(
    document,
    ctx.expression.id,
    ctx.expression.scssModulePath,
    deps,
  );
  const projection =
    rawPayload && rawPayload.selectorNames.length === 0 && readRustSelectorProjections
      ? (indexExpressionDomainSelectorProjectionsForStyle(
          readRustSelectorProjections(document, ctx.expression.scssModulePath, deps),
          ctx.expression.scssModulePath,
        ).get(ctx.expression.id) ?? null)
      : null;
  const payload = rawPayload
    ? withExpressionDomainSelectorProjection(rawPayload, projection)
    : rawPayload;
  if (!payload || !payload.styleFilePath) return null;
  if (!isInformativeRustExpressionPayload(payload)) return null;

  const styleDocument = runtime.styleHost.styleDocumentForPath(payload.styleFilePath);
  const selectors =
    styleDocument?.selectors.filter((selector) => payload.selectorNames.includes(selector.name)) ??
    [];
  const semantics = buildExpressionSemanticsSummaryFromRustPayload(
    ctx.expression,
    styleDocument,
    selectors,
    payload,
  );
  const dynamicExplanation = buildDynamicExpressionExplanation(ctx.expression, semantics);

  return {
    filePath: options.filePath,
    line: options.line,
    character: options.character,
    analysisSource: "omena-query",
    expressionKind: ctx.expression.kind,
    styleFilePath: payload.styleFilePath,
    selectorNames: payload.selectorNames,
    dynamicExplanation,
    analysisV2: {
      valueDomainKind: payload.valueDomainKind as ValueDomainKindV2,
      ...(payload.valueConstraintKind
        ? { valueConstraintKind: payload.valueConstraintKind as StringConstraintKindV2 }
        : {}),
      ...(payload.valuePrefix ? { valuePrefix: payload.valuePrefix } : {}),
      ...(payload.valueSuffix ? { valueSuffix: payload.valueSuffix } : {}),
      ...(payload.valueMinLen !== undefined ? { valueMinLen: payload.valueMinLen } : {}),
      ...(payload.valueMaxLen !== undefined ? { valueMaxLen: payload.valueMaxLen } : {}),
      ...(payload.valueCharMust ? { valueCharMust: payload.valueCharMust } : {}),
      ...(payload.valueCharMay ? { valueCharMay: payload.valueCharMay } : {}),
      ...(payload.valueMayIncludeOtherChars ? { valueMayIncludeOtherChars: true } : {}),
      ...(payload.valueDomainDerivation
        ? { valueDomainDerivation: payload.valueDomainDerivation }
        : {}),
      ...(payload.valueDomainProvenanceTree
        ? { valueDomainProvenanceTree: payload.valueDomainProvenanceTree }
        : {}),
      ...(payload.valueCertaintyShapeKind
        ? { valueCertaintyShapeKind: payload.valueCertaintyShapeKind as ValueCertaintyShapeKindV2 }
        : {}),
      ...(payload.valueCertaintyConstraintKind
        ? {
            valueCertaintyConstraintKind:
              payload.valueCertaintyConstraintKind as StringConstraintKindV2,
          }
        : {}),
      ...(payload.selectorCertaintyShapeKind
        ? {
            selectorCertaintyShapeKind:
              payload.selectorCertaintyShapeKind as SelectorCertaintyShapeKindV2,
          }
        : {}),
      ...(payload.selectorConstraintKind
        ? { selectorConstraintKind: payload.selectorConstraintKind as StringConstraintKindV2 }
        : {}),
    },
  };
}

function isInformativeRustExpressionPayload(
  payload: ReturnType<typeof resolveRustExpressionSemanticsPayload>,
): boolean {
  if (!payload) return false;
  if (payload.selectorNames.length > 0) return true;
  if (payload.candidateNames.length > 0) return true;
  if ((payload.finiteValues?.length ?? 0) > 0) return true;
  if (payload.valueDomainKind !== "none" && payload.valueCertaintyShapeKind !== "unknown") {
    return true;
  }
  return payload.selectorCertaintyShapeKind !== "unknown";
}

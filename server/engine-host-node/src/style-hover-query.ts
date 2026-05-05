import type { Range } from "@css-module-explainer/shared";
import {
  findAnimationNameRefAtCursor,
  findCanonicalSelector,
  findComposesTokenAtCursor,
  findCustomPropertyDeclAtCursor,
  findCustomPropertyRefAtCursor,
  findKeyframesAtCursor,
  findKeyframesByName,
  findSassModuleMemberRefAtCursor,
  findSassSymbolAtCursor,
  findSassSymbolDeclAtCursor,
  findSassSymbolDeclForSymbol,
  findSelectorAtCursor,
  findValueDeclAtCursor,
  findValueImportAtCursor,
  findValueRefAtCursor,
  listAnimationNameRefs,
  listCustomPropertyRefs,
  listSassModuleMemberRefsForMember,
  listSassSymbolsForDecl,
  listSassWildcardSymbolsForTarget,
  listValueRefs,
  readSelectorStyleDependencySummary,
  readSelectorUsageSummary,
  resolveComposesTarget,
  resolveCustomPropertyDeclTarget,
  resolveSassModuleMemberRefTarget,
  resolveSassWildcardSymbolTarget,
  resolveValueImportTarget,
  resolveValueTarget,
} from "../../engine-core-ts/src/core/query";
import type {
  CustomPropertyDeclHIR,
  KeyframesDeclHIR,
  SassSymbolDeclHIR,
  SelectorDeclHIR,
  StyleDocumentHIR,
  ValueDeclHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import {
  resolveSelectedQueryBackendKind,
  usesRustSelectorUsageBackend,
  usesRustStyleSemanticGraphBackend,
} from "./selected-query-backend";
import {
  buildSelectorUsageRenderSummaryFromRustPayload,
  resolveRustSelectorUsagePayloadForWorkspaceTarget,
  type SelectorUsageRenderSummary,
} from "./selector-usage-query-backend";
import { resolveRustStyleSelectorIdentityReadModelForWorkspaceTarget } from "./style-selector-identity-query";
import {
  buildSelectorReferenceRenderSummaryFromRustGraph,
  resolveRustStyleSelectorReferenceSummaryForWorkspaceTargetAsync,
  resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget,
  type StyleSelectorReferenceQueryOptions,
} from "./style-selector-reference-query";
import {
  buildStyleSemanticGraphSelectorIdentityReadModels,
  resolveRustStyleSemanticGraphForWorkspaceTargetAsync,
  type StyleSemanticGraphCache,
  type StyleSemanticGraphDesignTokenRankedReferenceReadModel,
  type StyleSemanticGraphSelectorIdentityReadModel,
} from "./style-semantic-graph-query-backend";
import {
  resolveStyleDesignTokenRankingForReference,
  resolveStyleDesignTokenRankingForReferenceAsync,
} from "./style-design-token-ranking-query";

interface CustomPropertyHoverTarget {
  readonly filePath: string;
  readonly decl: CustomPropertyDeclHIR;
}

export interface StyleSelectorHoverResult {
  readonly kind: "selector";
  readonly selector: SelectorDeclHIR;
  readonly range: Range;
  readonly scssModulePath: string;
  readonly usageSummary: SelectorUsageRenderSummary;
  readonly styleDependencies: ReturnType<typeof readSelectorStyleDependencySummary>;
  readonly selectorIdentity?: StyleSemanticGraphSelectorIdentityReadModel;
  readonly headingName?: string;
  readonly note?: string;
}

export interface StyleKeyframesHoverResult {
  readonly kind: "keyframes";
  readonly keyframes: KeyframesDeclHIR;
  readonly range: Range;
  readonly scssModulePath: string;
  readonly referenceCount: number;
  readonly headingName?: string;
  readonly note?: string;
}

export interface StyleValueHoverResult {
  readonly kind: "value";
  readonly valueDecl: ValueDeclHIR;
  readonly range: Range;
  readonly scssModulePath: string;
  readonly referenceCount: number;
  readonly headingName?: string;
  readonly note?: string;
}

export interface StyleSassSymbolHoverResult {
  readonly kind: "sassSymbol";
  readonly sassSymbolDecl: SassSymbolDeclHIR;
  readonly range: Range;
  readonly scssModulePath: string;
  readonly referenceCount: number;
  readonly headingName?: string;
  readonly note?: string;
}

export interface StyleCustomPropertyHoverResult {
  readonly kind: "customProperty";
  readonly customPropertyDecl: Pick<
    CustomPropertyDeclHIR,
    "name" | "value" | "range" | "ruleRange"
  >;
  readonly range: Range;
  readonly scssModulePath: string;
  readonly referenceCount: number;
  readonly headingName?: string;
  readonly note?: string;
  readonly designTokenRanking?: StyleSemanticGraphDesignTokenRankedReferenceReadModel;
}

export type StyleHoverResult =
  | StyleSelectorHoverResult
  | StyleKeyframesHoverResult
  | StyleValueHoverResult
  | StyleSassSymbolHoverResult
  | StyleCustomPropertyHoverResult;

export interface StyleHoverQueryOptions extends StyleSelectorReferenceQueryOptions {
  readonly readRustSelectorUsagePayloadForWorkspaceTarget?: typeof resolveRustSelectorUsagePayloadForWorkspaceTarget;
}

export function resolveStyleHoverResult(
  args: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
  },
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "workspaceRoot"
    | "settings"
    | "aliasResolver"
    | "readStyleFile"
    | "buildStyleDocument"
  >,
  options: StyleHoverQueryOptions = {},
): StyleHoverResult | null {
  const styleDocument = deps.styleDocumentForPath(args.filePath);
  if (!styleDocument) return null;
  const queryOptions = withDepsStyleSemanticGraphCache(deps, options);

  const selectorHover = resolveStyleSelectorHoverResultForDocument(
    styleDocument,
    args,
    deps,
    queryOptions,
  );
  if (selectorHover) return selectorHover;

  const keyframes = findKeyframesAtCursor(styleDocument, args.line, args.character);
  if (keyframes) {
    return {
      kind: "keyframes",
      keyframes,
      range: keyframes.range,
      scssModulePath: args.filePath,
      referenceCount: listAnimationNameRefs(styleDocument, keyframes.name).length,
    };
  }

  const valueDecl = findValueDeclAtCursor(styleDocument, args.line, args.character);
  if (valueDecl) {
    return {
      kind: "value",
      valueDecl,
      range: valueDecl.range,
      scssModulePath: args.filePath,
      referenceCount: listValueRefs(styleDocument, valueDecl.name).length,
    };
  }

  const valueImport = findValueImportAtCursor(styleDocument, args.line, args.character);
  if (valueImport) {
    const targetValue = resolveValueImportTarget(
      deps.styleDocumentForPath,
      styleDocument.filePath,
      valueImport,
    );
    if (!targetValue) return null;
    return {
      kind: "value",
      valueDecl: targetValue.valueDecl,
      range: valueImport.range,
      headingName: valueImport.name,
      note: `Imported from \`${valueImport.from}\` as \`${valueImport.importedName}\``,
      scssModulePath: targetValue.filePath,
      referenceCount: listValueRefs(styleDocument, valueImport.name).length,
    };
  }

  const customPropertyDecl = findCustomPropertyDeclAtCursor(
    styleDocument,
    args.line,
    args.character,
  );
  if (customPropertyDecl) {
    return {
      kind: "customProperty",
      customPropertyDecl,
      range: customPropertyDecl.range,
      scssModulePath: args.filePath,
      referenceCount: readCustomPropertyReferenceCount(
        deps.styleDependencyGraph,
        styleDocument,
        customPropertyDecl.name,
      ),
    };
  }

  const customPropertyRef = findCustomPropertyRefAtCursor(styleDocument, args.line, args.character);
  if (customPropertyRef) {
    const target = resolveCustomPropertyDeclTarget(
      deps.styleDocumentForPath,
      args.filePath,
      styleDocument,
      customPropertyRef,
      deps.styleDependencyGraph,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    const designTokenRanking = withDesignTokenRanking(
      deps,
      styleDocument,
      args.filePath,
      customPropertyRef,
      queryOptions,
    );
    const rankedTarget = resolveDesignTokenRankingDeclTarget(
      deps,
      designTokenRanking.designTokenRanking,
    );
    const hoverTarget = rankedTarget ?? target;
    if (!hoverTarget) return null;
    return {
      kind: "customProperty",
      customPropertyDecl: hoverTarget.decl,
      range: customPropertyRef.range,
      headingName: customPropertyRef.name,
      note: "Referenced via `var()`",
      scssModulePath: hoverTarget.filePath,
      referenceCount: readCustomPropertyReferenceCount(
        deps.styleDependencyGraph,
        styleDocument,
        customPropertyRef.name,
      ),
      ...designTokenRanking,
    };
  }

  const sassSymbolDecl = findSassSymbolDeclAtCursor(styleDocument, args.line, args.character);
  if (sassSymbolDecl) {
    const incomingModuleMemberRefs = deps.styleDependencyGraph.getIncomingSassModuleMemberRefs(
      args.filePath,
      sassSymbolDecl.symbolKind,
      sassSymbolDecl.name,
    );
    return {
      kind: "sassSymbol",
      sassSymbolDecl,
      range: sassSymbolDecl.range,
      scssModulePath: args.filePath,
      referenceCount:
        listSassSymbolsForDecl(styleDocument, sassSymbolDecl).length +
        incomingModuleMemberRefs.length,
    };
  }

  const animationRef = findAnimationNameRefAtCursor(styleDocument, args.line, args.character);
  if (animationRef) {
    const targetKeyframes = findKeyframesByName(styleDocument, animationRef.name);
    if (!targetKeyframes) return null;
    return {
      kind: "keyframes",
      keyframes: targetKeyframes,
      range: animationRef.range,
      headingName: animationRef.name,
      note: `Referenced via \`${animationRef.property}\``,
      scssModulePath: args.filePath,
      referenceCount: listAnimationNameRefs(styleDocument, targetKeyframes.name).length,
    };
  }

  const sassSymbol = findSassSymbolAtCursor(styleDocument, args.line, args.character);
  if (sassSymbol) {
    const target = findSassSymbolDeclForSymbol(styleDocument, sassSymbol);
    if (!target) {
      const wildcardTarget = resolveSassWildcardSymbolTarget(
        deps.styleDocumentForPath,
        styleDocument.filePath,
        styleDocument,
        sassSymbol,
        deps.aliasResolver,
        { readFile: deps.readStyleFile },
      );
      if (!wildcardTarget) return null;
      return {
        kind: "sassSymbol",
        sassSymbolDecl: wildcardTarget.decl,
        range: sassSymbol.range,
        headingName: sassSymbol.name,
        note: `Referenced via Sass wildcard ${sassSymbol.role}`,
        scssModulePath: wildcardTarget.filePath,
        referenceCount: listSassWildcardSymbolsForTarget(styleDocument, wildcardTarget).length,
      };
    }
    return {
      kind: "sassSymbol",
      sassSymbolDecl: target,
      range: sassSymbol.range,
      headingName: sassSymbol.name,
      note: `Referenced via ${styleSymbolLanguageName(target)} ${sassSymbol.role}`,
      scssModulePath: args.filePath,
      referenceCount: listSassSymbolsForDecl(styleDocument, target).length,
    };
  }

  const sassModuleMemberRef = findSassModuleMemberRefAtCursor(
    styleDocument,
    args.line,
    args.character,
  );
  if (sassModuleMemberRef) {
    const target = resolveSassModuleMemberRefTarget(
      deps.styleDocumentForPath,
      styleDocument.filePath,
      styleDocument,
      sassModuleMemberRef,
      deps.aliasResolver,
      { readFile: deps.readStyleFile },
    );
    if (!target) return null;
    return {
      kind: "sassSymbol",
      sassSymbolDecl: target.decl,
      range: sassModuleMemberRef.range,
      headingName: `${sassModuleMemberRef.namespace}.${sassModuleMemberRef.name}`,
      note: `Referenced via Sass module ${sassModuleMemberRef.role}`,
      scssModulePath: target.filePath,
      referenceCount:
        deps.styleDependencyGraph.getIncomingSassModuleMemberRefs(
          target.filePath,
          target.decl.symbolKind,
          target.decl.name,
        ).length || listSassModuleMemberRefsForMember(styleDocument, sassModuleMemberRef).length,
    };
  }

  const valueRef = findValueRefAtCursor(styleDocument, args.line, args.character);
  if (!valueRef) return null;
  const targetValue = resolveValueTarget(
    deps.styleDocumentForPath,
    styleDocument.filePath,
    styleDocument,
    valueRef.name,
  );
  if (!targetValue) return null;
  return {
    kind: "value",
    valueDecl: targetValue.valueDecl,
    range: valueRef.range,
    headingName: valueRef.name,
    note:
      targetValue.bindingKind === "imported"
        ? `Referenced via \`${valueRef.source === "declaration" ? "declaration value" : "@value"}\`; imported from \`${targetValue.valueImport!.from}\` as \`${targetValue.valueImport!.importedName}\``
        : `Referenced via \`${valueRef.source === "declaration" ? "declaration value" : "@value"}\``,
    scssModulePath: targetValue.filePath,
    referenceCount: listValueRefs(styleDocument, valueRef.name).length,
  };
}

export async function resolveStyleHoverResultAsync(
  args: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
  },
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "workspaceRoot"
    | "settings"
    | "aliasResolver"
    | "readStyleFile"
    | "buildStyleDocument"
  >,
  options: StyleHoverQueryOptions = {},
): Promise<StyleHoverResult | null> {
  const styleDocument = deps.styleDocumentForPath(args.filePath);
  if (!styleDocument) return null;
  const queryOptions = withDepsStyleSemanticGraphCache(deps, options);

  const selectorHover = await resolveStyleSelectorHoverResultForDocumentAsync(
    styleDocument,
    args,
    deps,
    queryOptions,
  );
  if (selectorHover) return selectorHover;

  const customPropertyRef = findCustomPropertyRefAtCursor(styleDocument, args.line, args.character);
  if (!customPropertyRef) return resolveStyleHoverResult(args, deps, queryOptions);

  const target = resolveCustomPropertyDeclTarget(
    deps.styleDocumentForPath,
    args.filePath,
    styleDocument,
    customPropertyRef,
    deps.styleDependencyGraph,
    deps.aliasResolver,
    { readFile: deps.readStyleFile },
  );
  const designTokenRanking = await withDesignTokenRankingAsync(
    deps,
    styleDocument,
    args.filePath,
    customPropertyRef,
    queryOptions,
  );
  const rankedTarget = resolveDesignTokenRankingDeclTarget(
    deps,
    designTokenRanking.designTokenRanking,
  );
  const hoverTarget = rankedTarget ?? target;
  if (!hoverTarget) return null;

  return {
    kind: "customProperty",
    customPropertyDecl: hoverTarget.decl,
    range: customPropertyRef.range,
    headingName: customPropertyRef.name,
    note: "Referenced via `var()`",
    scssModulePath: hoverTarget.filePath,
    referenceCount: readCustomPropertyReferenceCount(
      deps.styleDependencyGraph,
      styleDocument,
      customPropertyRef.name,
    ),
    ...designTokenRanking,
  };
}

export function resolveStyleSelectorHoverResult(
  args: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
  },
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  options: StyleHoverQueryOptions = {},
): StyleSelectorHoverResult | null {
  const styleDocument = deps.styleDocumentForPath(args.filePath);
  if (!styleDocument) return null;

  return resolveStyleSelectorHoverResultForDocument(
    styleDocument,
    args,
    deps,
    withDepsStyleSemanticGraphCache(deps, options),
  );
}

export async function resolveStyleSelectorHoverResultAsync(
  args: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
  },
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  options: StyleHoverQueryOptions = {},
): Promise<StyleSelectorHoverResult | null> {
  const styleDocument = deps.styleDocumentForPath(args.filePath);
  if (!styleDocument) return null;

  return resolveStyleSelectorHoverResultForDocumentAsync(
    styleDocument,
    args,
    deps,
    withDepsStyleSemanticGraphCache(deps, options),
  );
}

function resolveStyleSelectorHoverResultForDocument(
  styleDocument: StyleDocumentHIR,
  args: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
  },
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  options: StyleHoverQueryOptions,
): StyleSelectorHoverResult | null {
  const selector = findSelectorAtCursor(styleDocument, args.line, args.character);
  if (selector) {
    const canonicalSelector = findCanonicalSelector(styleDocument, selector);
    return {
      kind: "selector",
      selector: canonicalSelector,
      range: selector.bemSuffix?.rawTokenRange ?? selector.range,
      scssModulePath: args.filePath,
      usageSummary: resolveStyleSelectorUsageSummary(
        deps,
        args.filePath,
        canonicalSelector.canonicalName,
        options,
      ),
      styleDependencies: readSelectorStyleDependencySummary(
        deps.styleDependencyGraph,
        args.filePath,
        canonicalSelector.canonicalName,
      ),
      ...withStyleSelectorIdentity(
        deps,
        styleDocument,
        args.filePath,
        canonicalSelector.canonicalName,
        options,
      ),
    };
  }

  const composesHit = findComposesTokenAtCursor(styleDocument, args.line, args.character);
  const target = resolveComposesTarget(
    deps.styleDocumentForPath,
    styleDocument.filePath,
    composesHit,
  );
  if (!composesHit || !target) return null;

  return {
    kind: "selector",
    selector: target.selector,
    range: composesHit.token.range,
    scssModulePath: target.filePath,
    usageSummary: resolveStyleSelectorUsageSummary(
      deps,
      target.filePath,
      target.selector.canonicalName,
      options,
    ),
    styleDependencies: readSelectorStyleDependencySummary(
      deps.styleDependencyGraph,
      target.filePath,
      target.selector.canonicalName,
    ),
    ...withStyleSelectorIdentity(
      deps,
      target.styleDocument,
      target.filePath,
      target.selector.canonicalName,
      options,
    ),
    headingName: composesHit.token.className,
    note: `Referenced via \`composes\` from \`.${composesHit.selector.name}\``,
  };
}

async function resolveStyleSelectorHoverResultForDocumentAsync(
  styleDocument: StyleDocumentHIR,
  args: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
  },
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  options: StyleHoverQueryOptions,
): Promise<StyleSelectorHoverResult | null> {
  const selector = findSelectorAtCursor(styleDocument, args.line, args.character);
  if (selector) {
    const canonicalSelector = findCanonicalSelector(styleDocument, selector);
    return {
      kind: "selector",
      selector: canonicalSelector,
      range: selector.bemSuffix?.rawTokenRange ?? selector.range,
      scssModulePath: args.filePath,
      usageSummary: await resolveStyleSelectorUsageSummaryAsync(
        deps,
        args.filePath,
        canonicalSelector.canonicalName,
        options,
      ),
      styleDependencies: readSelectorStyleDependencySummary(
        deps.styleDependencyGraph,
        args.filePath,
        canonicalSelector.canonicalName,
      ),
      ...(await withStyleSelectorIdentityAsync(
        deps,
        styleDocument,
        args.filePath,
        canonicalSelector.canonicalName,
        options,
      )),
    };
  }

  const composesHit = findComposesTokenAtCursor(styleDocument, args.line, args.character);
  const target = resolveComposesTarget(
    deps.styleDocumentForPath,
    styleDocument.filePath,
    composesHit,
  );
  if (!composesHit || !target) return null;

  return {
    kind: "selector",
    selector: target.selector,
    range: composesHit.token.range,
    scssModulePath: target.filePath,
    usageSummary: await resolveStyleSelectorUsageSummaryAsync(
      deps,
      target.filePath,
      target.selector.canonicalName,
      options,
    ),
    styleDependencies: readSelectorStyleDependencySummary(
      deps.styleDependencyGraph,
      target.filePath,
      target.selector.canonicalName,
    ),
    ...(await withStyleSelectorIdentityAsync(
      deps,
      target.styleDocument,
      target.filePath,
      target.selector.canonicalName,
      options,
    )),
    headingName: composesHit.token.className,
    note: `Referenced via \`composes\` from \`.${composesHit.selector.name}\``,
  };
}

function resolveStyleSelectorUsageSummary(
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  filePath: string,
  canonicalName: string,
  options: StyleHoverQueryOptions,
): SelectorUsageRenderSummary {
  const backend = resolveSelectedQueryBackendKind(options.env);
  const graphReferences = resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget(
    { filePath, canonicalName },
    deps,
    options,
  );
  if (graphReferences) {
    return buildSelectorReferenceRenderSummaryFromRustGraph(graphReferences);
  }

  if (usesRustSelectorUsageBackend(backend)) {
    const payload = (
      options.readRustSelectorUsagePayloadForWorkspaceTarget ??
      resolveRustSelectorUsagePayloadForWorkspaceTarget
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      deps,
      filePath,
      canonicalName,
    );
    if (payload) {
      return buildSelectorUsageRenderSummaryFromRustPayload(payload);
    }
  }

  const usage = readSelectorUsageSummary(
    {
      semanticReferenceIndex: deps.semanticReferenceIndex,
      styleDependencyGraph: deps.styleDependencyGraph,
      styleDocumentForPath: deps.styleDocumentForPath,
    },
    filePath,
    canonicalName,
  );
  return usage;
}

async function resolveStyleSelectorUsageSummaryAsync(
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  filePath: string,
  canonicalName: string,
  options: StyleHoverQueryOptions,
): Promise<SelectorUsageRenderSummary> {
  const graphReferences = await resolveRustStyleSelectorReferenceSummaryForWorkspaceTargetAsync(
    { filePath, canonicalName },
    deps,
    options,
  );
  if (graphReferences) {
    return buildSelectorReferenceRenderSummaryFromRustGraph(graphReferences);
  }

  return resolveStyleSelectorUsageSummary(deps, filePath, canonicalName, options);
}

function withStyleSelectorIdentity(
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  styleDocument: StyleDocumentHIR,
  filePath: string,
  canonicalName: string,
  options: StyleHoverQueryOptions,
): { readonly selectorIdentity?: StyleSemanticGraphSelectorIdentityReadModel } {
  const queryOptions = withDepsStyleSemanticGraphCache(deps, options);
  const selectorIdentity = resolveRustStyleSelectorIdentityReadModelForWorkspaceTarget(
    {
      filePath,
      styleDocument,
      canonicalName,
    },
    deps,
    queryOptions,
  );
  return selectorIdentity ? { selectorIdentity } : {};
}

async function withStyleSelectorIdentityAsync(
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  styleDocument: StyleDocumentHIR,
  filePath: string,
  canonicalName: string,
  options: StyleHoverQueryOptions,
): Promise<{ readonly selectorIdentity?: StyleSemanticGraphSelectorIdentityReadModel }> {
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return {};
  }

  const queryOptions = withDepsStyleSemanticGraphCache(deps, options);
  try {
    const graph = await (
      queryOptions.readRustStyleSemanticGraphForWorkspaceTargetAsync ??
      resolveRustStyleSemanticGraphForWorkspaceTargetAsync
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      deps,
      filePath,
      queryOptions,
    );
    if (!graph) return {};
    const selectorIdentity = buildStyleSemanticGraphSelectorIdentityReadModels(
      graph,
      styleDocument,
    ).find((identity) => identity.canonicalName === canonicalName);
    return selectorIdentity ? { selectorIdentity } : {};
  } catch {
    return {};
  }
}

function withDesignTokenRanking(
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  styleDocument: StyleDocumentHIR,
  filePath: string,
  customPropertyRef: StyleDocumentHIR["customPropertyRefs"][number],
  options: StyleHoverQueryOptions,
): { readonly designTokenRanking?: StyleSemanticGraphDesignTokenRankedReferenceReadModel } {
  const ranking = resolveStyleDesignTokenRankingForReference(
    { filePath, styleDocument, customPropertyRef },
    deps,
    options,
  );
  return ranking ? { designTokenRanking: ranking } : {};
}

async function withDesignTokenRankingAsync(
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  styleDocument: StyleDocumentHIR,
  filePath: string,
  customPropertyRef: StyleDocumentHIR["customPropertyRefs"][number],
  options: StyleHoverQueryOptions,
): Promise<{
  readonly designTokenRanking?: StyleSemanticGraphDesignTokenRankedReferenceReadModel;
}> {
  const ranking = await resolveStyleDesignTokenRankingForReferenceAsync(
    { filePath, styleDocument, customPropertyRef },
    deps,
    options,
  );
  return ranking ? { designTokenRanking: ranking } : {};
}

function withDepsStyleSemanticGraphCache(
  deps: object,
  options: StyleHoverQueryOptions,
): StyleHoverQueryOptions {
  const cache = (deps as { readonly styleSemanticGraphCache?: StyleSemanticGraphCache })
    .styleSemanticGraphCache;
  if (options.styleSemanticGraphCache || !cache) return options;
  return { ...options, styleSemanticGraphCache: cache };
}

function resolveDesignTokenRankingDeclTarget(
  deps: Pick<ProviderDeps, "styleDocumentForPath" | "readStyleFile" | "buildStyleDocument">,
  ranking: StyleSemanticGraphDesignTokenRankedReferenceReadModel | undefined,
): CustomPropertyHoverTarget | null {
  if (!ranking?.winnerDeclarationFilePath) return null;

  const styleDocument = readDesignTokenRankingTargetStyleDocument(
    deps,
    ranking.winnerDeclarationFilePath,
  );
  if (!styleDocument) return null;

  const winnerDeclarationRange = ranking.winnerDeclarationRange;
  const rangeMatchedDecl = winnerDeclarationRange
    ? styleDocument.customPropertyDecls.find(
        (decl) =>
          decl.name === ranking.referenceName && rangesEqual(decl.range, winnerDeclarationRange),
      )
    : undefined;
  const sourceOrderDecl = styleDocument.customPropertyDecls[ranking.winnerDeclarationSourceOrder];
  const decl =
    rangeMatchedDecl ??
    (sourceOrderDecl?.name === ranking.referenceName ? sourceOrderDecl : undefined);
  return decl ? { filePath: ranking.winnerDeclarationFilePath, decl } : null;
}

function readDesignTokenRankingTargetStyleDocument(
  deps: Pick<ProviderDeps, "styleDocumentForPath" | "readStyleFile" | "buildStyleDocument">,
  filePath: string,
): StyleDocumentHIR | null {
  const indexedDocument = deps.styleDocumentForPath(filePath);
  if (indexedDocument) return indexedDocument;
  const content = deps.readStyleFile(filePath);
  return content === null ? null : deps.buildStyleDocument(filePath, content);
}

function rangesEqual(left: Range, right: Range): boolean {
  return (
    left.start.line === right.start.line &&
    left.start.character === right.start.character &&
    left.end.line === right.end.line &&
    left.end.character === right.end.character
  );
}

function readCustomPropertyReferenceCount(
  styleDependencyGraph: ProviderDeps["styleDependencyGraph"],
  styleDocument: StyleDocumentHIR,
  name: string,
): number {
  const workspaceRefs = styleDependencyGraph.getCustomPropertyRefs(name);
  return workspaceRefs.length > 0
    ? workspaceRefs.length
    : listCustomPropertyRefs(styleDocument, name).length;
}

function styleSymbolLanguageName(decl: SassSymbolDeclHIR): "Sass" | "Less" {
  return decl.syntax === "less" ? "Less" : "Sass";
}

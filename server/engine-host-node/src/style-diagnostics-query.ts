import {
  checkStyleDocument,
  type StyleCheckerFinding,
} from "../../engine-core-ts/src/core/checker";
import type { StyleDocumentCheckOptions } from "../../engine-core-ts/src/core/checker/check-style-document";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import {
  resolveSelectedQueryBackendKind,
  usesRustStyleSemanticGraphBackend,
  usesRustSelectorUsageBackend,
} from "./selected-query-backend";
import {
  resolveUnusedStyleSelectorsAsync,
  resolveUnusedStyleSelectors,
  type StyleModuleUsageQueryOptions,
} from "./style-module-usage-query";
import type { SelectorUsagePayloadCache } from "./selector-usage-query-backend";
import {
  buildStyleSemanticGraphDesignTokenRankedReferenceReadModels,
  resolveRustStyleSemanticGraphForWorkspaceTargetAsync,
  resolveRustStyleSemanticGraphForWorkspaceTarget,
  type StyleSemanticGraphCache,
  type StyleSemanticGraphSummaryV0,
} from "./style-semantic-graph-query-backend";

export interface StyleDiagnosticsQueryOptions extends StyleModuleUsageQueryOptions {
  readonly includeUnusedSelectors?: boolean;
  readonly includeComposesResolution?: boolean;
}

export function resolveStyleDiagnosticFindings(
  args: {
    readonly scssPath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: Pick<ProviderDeps, "semanticReferenceIndex"> & {
    readonly analysisCache?: ProviderDeps["analysisCache"];
    readonly readStyleFile?: ProviderDeps["readStyleFile"];
    readonly styleDependencyGraph?: ProviderDeps["styleDependencyGraph"];
    readonly styleDocumentForPath?: ProviderDeps["styleDocumentForPath"];
    readonly typeResolver?: ProviderDeps["typeResolver"];
    readonly workspaceRoot?: ProviderDeps["workspaceRoot"];
    readonly settings?: ProviderDeps["settings"];
    readonly aliasResolver?: ProviderDeps["aliasResolver"];
    readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
    readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
  },
  options: StyleDiagnosticsQueryOptions = {},
): readonly StyleCheckerFinding[] {
  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  const includeUnusedSelectors = options.includeUnusedSelectors ?? true;
  const useRustSelectorUsage =
    includeUnusedSelectors && usesRustSelectorUsageBackend(selectedQueryBackend);
  if (useRustSelectorUsage && hasRustStyleDiagnosticsDeps(deps)) {
    const rustDeps = {
      analysisCache: deps.analysisCache,
      semanticReferenceIndex: deps.semanticReferenceIndex,
      styleDependencyGraph: deps.styleDependencyGraph,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      workspaceRoot: deps.workspaceRoot,
      settings: deps.settings,
      ...(deps.readStyleFile ? { readStyleFile: deps.readStyleFile } : {}),
      ...(deps.aliasResolver ? { aliasResolver: deps.aliasResolver } : {}),
      ...(deps.styleSemanticGraphCache
        ? { styleSemanticGraphCache: deps.styleSemanticGraphCache }
        : {}),
      ...(deps.selectorUsagePayloadCache
        ? { selectorUsagePayloadCache: deps.selectorUsagePayloadCache }
        : {}),
    } satisfies Pick<
      ProviderDeps,
      | "analysisCache"
      | "semanticReferenceIndex"
      | "styleDependencyGraph"
      | "styleDocumentForPath"
      | "typeResolver"
      | "workspaceRoot"
      | "settings"
    > & {
      readonly aliasResolver?: ProviderDeps["aliasResolver"];
      readonly readStyleFile?: ProviderDeps["readStyleFile"];
      readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
      readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
    };
    const unusedSelectors = resolveUnusedStyleSelectors(args, rustDeps, options);
    const otherFindings = checkStyleDocument(
      args,
      {
        semanticReferenceIndex: rustDeps.semanticReferenceIndex,
        styleDependencyGraph: rustDeps.styleDependencyGraph,
        styleDocumentForPath: rustDeps.styleDocumentForPath,
        ...(rustDeps.aliasResolver ? { aliasResolver: rustDeps.aliasResolver } : {}),
        ...(rustDeps.readStyleFile ? { readFile: rustDeps.readStyleFile } : {}),
      },
      {
        includeUnusedSelectors: false,
        ...(options.includeComposesResolution !== undefined
          ? { includeComposesResolution: options.includeComposesResolution }
          : {}),
      },
    );
    const filteredOtherFindings = filterResolvedRustDesignTokenFindings(
      args,
      rustDeps,
      options,
      otherFindings,
    );
    return [
      ...unusedSelectors.map<StyleCheckerFinding>((selector) => ({
        category: "style",
        code: "unused-selector",
        severity: "hint",
        range: selector.range,
        selectorFilePath: args.scssPath,
        canonicalName: selector.canonicalName,
      })),
      ...filteredOtherFindings,
    ];
  }

  return checkCurrentStyleDocument(args, deps, {
    includeUnusedSelectors: includeUnusedSelectors && !useRustSelectorUsage,
    ...(options.includeComposesResolution !== undefined
      ? { includeComposesResolution: options.includeComposesResolution }
      : {}),
  });
}

export async function resolveStyleDiagnosticFindingsAsync(
  args: {
    readonly scssPath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: Pick<ProviderDeps, "semanticReferenceIndex"> & {
    readonly analysisCache?: ProviderDeps["analysisCache"];
    readonly readStyleFile?: ProviderDeps["readStyleFile"];
    readonly styleDependencyGraph?: ProviderDeps["styleDependencyGraph"];
    readonly styleDocumentForPath?: ProviderDeps["styleDocumentForPath"];
    readonly typeResolver?: ProviderDeps["typeResolver"];
    readonly workspaceRoot?: ProviderDeps["workspaceRoot"];
    readonly settings?: ProviderDeps["settings"];
    readonly aliasResolver?: ProviderDeps["aliasResolver"];
    readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
    readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
  },
  options: StyleDiagnosticsQueryOptions = {},
): Promise<readonly StyleCheckerFinding[]> {
  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  const includeUnusedSelectors = options.includeUnusedSelectors ?? true;
  const useRustSelectorUsage =
    includeUnusedSelectors && usesRustSelectorUsageBackend(selectedQueryBackend);
  if (useRustSelectorUsage && hasRustStyleDiagnosticsDeps(deps)) {
    const rustDeps = {
      analysisCache: deps.analysisCache,
      semanticReferenceIndex: deps.semanticReferenceIndex,
      styleDependencyGraph: deps.styleDependencyGraph,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      workspaceRoot: deps.workspaceRoot,
      settings: deps.settings,
      ...(deps.readStyleFile ? { readStyleFile: deps.readStyleFile } : {}),
      ...(deps.aliasResolver ? { aliasResolver: deps.aliasResolver } : {}),
      ...(deps.styleSemanticGraphCache
        ? { styleSemanticGraphCache: deps.styleSemanticGraphCache }
        : {}),
      ...(deps.selectorUsagePayloadCache
        ? { selectorUsagePayloadCache: deps.selectorUsagePayloadCache }
        : {}),
    } satisfies Pick<
      ProviderDeps,
      | "analysisCache"
      | "semanticReferenceIndex"
      | "styleDependencyGraph"
      | "styleDocumentForPath"
      | "typeResolver"
      | "workspaceRoot"
      | "settings"
    > & {
      readonly aliasResolver?: ProviderDeps["aliasResolver"];
      readonly readStyleFile?: ProviderDeps["readStyleFile"];
      readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
      readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
    };
    const unusedSelectors = await resolveUnusedStyleSelectorsAsync(args, rustDeps, options);
    const otherFindings = checkStyleDocument(
      args,
      {
        semanticReferenceIndex: rustDeps.semanticReferenceIndex,
        styleDependencyGraph: rustDeps.styleDependencyGraph,
        styleDocumentForPath: rustDeps.styleDocumentForPath,
        ...(rustDeps.aliasResolver ? { aliasResolver: rustDeps.aliasResolver } : {}),
        ...(rustDeps.readStyleFile ? { readFile: rustDeps.readStyleFile } : {}),
      },
      {
        includeUnusedSelectors: false,
        ...(options.includeComposesResolution !== undefined
          ? { includeComposesResolution: options.includeComposesResolution }
          : {}),
      },
    );
    const filteredOtherFindings = await filterResolvedRustDesignTokenFindingsAsync(
      args,
      rustDeps,
      options,
      otherFindings,
    );
    return [
      ...unusedSelectors.map<StyleCheckerFinding>((selector) => ({
        category: "style",
        code: "unused-selector",
        severity: "hint",
        range: selector.range,
        selectorFilePath: args.scssPath,
        canonicalName: selector.canonicalName,
      })),
      ...filteredOtherFindings,
    ];
  }

  return checkCurrentStyleDocument(args, deps, {
    includeUnusedSelectors: includeUnusedSelectors && !useRustSelectorUsage,
    ...(options.includeComposesResolution !== undefined
      ? { includeComposesResolution: options.includeComposesResolution }
      : {}),
  });
}

function hasRustStyleDiagnosticsDeps(
  deps: Pick<ProviderDeps, "semanticReferenceIndex"> & {
    readonly analysisCache?: ProviderDeps["analysisCache"];
    readonly readStyleFile?: ProviderDeps["readStyleFile"];
    readonly styleDependencyGraph?: ProviderDeps["styleDependencyGraph"];
    readonly styleDocumentForPath?: ProviderDeps["styleDocumentForPath"];
    readonly typeResolver?: ProviderDeps["typeResolver"];
    readonly workspaceRoot?: ProviderDeps["workspaceRoot"];
    readonly settings?: ProviderDeps["settings"];
    readonly aliasResolver?: ProviderDeps["aliasResolver"];
    readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
    readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
  },
): deps is Pick<
  ProviderDeps,
  | "analysisCache"
  | "semanticReferenceIndex"
  | "styleDependencyGraph"
  | "styleDocumentForPath"
  | "typeResolver"
  | "workspaceRoot"
  | "settings"
> & {
  readonly aliasResolver?: ProviderDeps["aliasResolver"];
  readonly readStyleFile?: ProviderDeps["readStyleFile"];
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
} {
  return Boolean(
    deps.analysisCache &&
    deps.styleDependencyGraph &&
    deps.styleDocumentForPath &&
    deps.typeResolver &&
    deps.workspaceRoot &&
    deps.settings,
  );
}

function filterResolvedRustDesignTokenFindings(
  args: {
    readonly scssPath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  > & {
    readonly readStyleFile?: ProviderDeps["readStyleFile"];
    readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  },
  options: StyleDiagnosticsQueryOptions,
  findings: readonly StyleCheckerFinding[],
): readonly StyleCheckerFinding[] {
  if (!hasMissingCustomPropertyFindings(findings)) return findings;
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return findings;
  }
  const readStyleFile = deps.readStyleFile;
  if (!readStyleFile) return findings;

  const graph = safeResolveRustStyleSemanticGraphForDiagnostics(
    args.scssPath,
    toRustStyleSemanticGraphDeps(deps, readStyleFile),
    options,
  );
  if (!graph) return findings;

  const resolvedKeys = collectResolvedRustDesignTokenReferenceKeys(graph, args.styleDocument);
  if (resolvedKeys.size === 0) return findings;
  return findings.filter(
    (finding) =>
      finding.code !== "missing-custom-property" ||
      !resolvedKeys.has(customPropertyReferenceKey(finding.propertyName, finding.range)),
  );
}

async function filterResolvedRustDesignTokenFindingsAsync(
  args: {
    readonly scssPath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  > & {
    readonly readStyleFile?: ProviderDeps["readStyleFile"];
    readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  },
  options: StyleDiagnosticsQueryOptions,
  findings: readonly StyleCheckerFinding[],
): Promise<readonly StyleCheckerFinding[]> {
  if (!hasMissingCustomPropertyFindings(findings)) return findings;
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return findings;
  }
  const readStyleFile = deps.readStyleFile;
  if (!readStyleFile) return findings;

  const graph = await safeResolveRustStyleSemanticGraphForDiagnosticsAsync(
    args.scssPath,
    toRustStyleSemanticGraphDeps(deps, readStyleFile),
    options,
  );
  if (!graph) return findings;

  const resolvedKeys = collectResolvedRustDesignTokenReferenceKeys(graph, args.styleDocument);
  if (resolvedKeys.size === 0) return findings;
  return findings.filter(
    (finding) =>
      finding.code !== "missing-custom-property" ||
      !resolvedKeys.has(customPropertyReferenceKey(finding.propertyName, finding.range)),
  );
}

function hasMissingCustomPropertyFindings(findings: readonly StyleCheckerFinding[]): boolean {
  return findings.some((finding) => finding.code === "missing-custom-property");
}

function toRustStyleSemanticGraphDeps(
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  > & {
    readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  },
  readStyleFile: ProviderDeps["readStyleFile"],
): Pick<
  ProviderDeps,
  | "analysisCache"
  | "styleDocumentForPath"
  | "typeResolver"
  | "workspaceRoot"
  | "settings"
  | "readStyleFile"
> & {
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
} {
  return {
    analysisCache: deps.analysisCache,
    styleDocumentForPath: deps.styleDocumentForPath,
    typeResolver: deps.typeResolver,
    workspaceRoot: deps.workspaceRoot,
    settings: deps.settings,
    readStyleFile,
    ...(deps.styleSemanticGraphCache
      ? { styleSemanticGraphCache: deps.styleSemanticGraphCache }
      : {}),
  };
}

function safeResolveRustStyleSemanticGraphForDiagnostics(
  scssPath: string,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  > & {
    readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  },
  options: StyleDiagnosticsQueryOptions,
): StyleSemanticGraphSummaryV0 | null {
  const queryOptions =
    options.styleSemanticGraphCache || !deps.styleSemanticGraphCache
      ? options
      : { ...options, styleSemanticGraphCache: deps.styleSemanticGraphCache };
  try {
    return (
      options.readRustStyleSemanticGraphForWorkspaceTarget ??
      resolveRustStyleSemanticGraphForWorkspaceTarget
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      scssPath,
      queryOptions,
    );
  } catch {
    return null;
  }
}

async function safeResolveRustStyleSemanticGraphForDiagnosticsAsync(
  scssPath: string,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  > & {
    readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  },
  options: StyleDiagnosticsQueryOptions,
): Promise<StyleSemanticGraphSummaryV0 | null> {
  const queryOptions =
    options.styleSemanticGraphCache || !deps.styleSemanticGraphCache
      ? options
      : { ...options, styleSemanticGraphCache: deps.styleSemanticGraphCache };
  try {
    return await (
      options.readRustStyleSemanticGraphForWorkspaceTargetAsync ??
      resolveRustStyleSemanticGraphForWorkspaceTargetAsync
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      {
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
        readStyleFile: deps.readStyleFile,
      },
      scssPath,
      queryOptions,
    );
  } catch {
    return null;
  }
}

function collectResolvedRustDesignTokenReferenceKeys(
  graph: StyleSemanticGraphSummaryV0,
  styleDocument: StyleDocumentHIR,
): ReadonlySet<string> {
  const keys = new Set<string>();
  for (const ranking of buildStyleSemanticGraphDesignTokenRankedReferenceReadModels(
    graph,
    styleDocument,
  )) {
    if (!ranking.reference) continue;
    if (!ranking.winnerDeclaration && !ranking.winnerDeclarationRange) continue;
    keys.add(customPropertyReferenceKey(ranking.reference.name, ranking.reference.range));
  }
  return keys;
}

function customPropertyReferenceKey(name: string, range: StyleCheckerFinding["range"]): string {
  return [name, range.start.line, range.start.character, range.end.line, range.end.character].join(
    ":",
  );
}

function checkCurrentStyleDocument(
  args: {
    readonly scssPath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: Pick<ProviderDeps, "semanticReferenceIndex"> & {
    readonly styleDependencyGraph?: ProviderDeps["styleDependencyGraph"];
    readonly styleDocumentForPath?: ProviderDeps["styleDocumentForPath"];
    readonly aliasResolver?: ProviderDeps["aliasResolver"];
    readonly readStyleFile?: ProviderDeps["readStyleFile"];
  },
  options: Pick<StyleDocumentCheckOptions, "includeUnusedSelectors" | "includeComposesResolution">,
): readonly StyleCheckerFinding[] {
  return checkStyleDocument(
    args,
    {
      semanticReferenceIndex: deps.semanticReferenceIndex,
      ...(deps.styleDependencyGraph ? { styleDependencyGraph: deps.styleDependencyGraph } : {}),
      ...(deps.styleDocumentForPath ? { styleDocumentForPath: deps.styleDocumentForPath } : {}),
      ...(deps.aliasResolver ? { aliasResolver: deps.aliasResolver } : {}),
      ...(deps.readStyleFile ? { readFile: deps.readStyleFile } : {}),
    },
    options,
  );
}

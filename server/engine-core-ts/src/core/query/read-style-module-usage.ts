import type { Range } from "@omena/shared";
import type { StyleDocumentHIR } from "../hir/style-types";
import type { SemanticWorkspaceReferenceIndex } from "../semantic/workspace-reference-index";
import type { StyleDependencyGraph } from "../semantic/style-dependency-graph";
import { listCanonicalSelectors } from "./find-style-selector";
import { readSelectorUsageSummary, type SelectorUsageSummary } from "./read-selector-usage";

export interface StyleModuleSelectorUsage {
  readonly canonicalName: string;
  readonly range: Range;
  readonly usage: SelectorUsageSummary;
  readonly hasComposedReachability: boolean;
}

export interface StyleModuleUsageSummary {
  readonly hasUnresolvedDynamicUsage: boolean;
  readonly selectors: readonly StyleModuleSelectorUsage[];
  readonly unusedSelectors: readonly StyleModuleSelectorUsage[];
}

export function readStyleModuleUsageSummary(
  scssPath: string,
  styleDocument: StyleDocumentHIR,
  semanticReferenceIndex: SemanticWorkspaceReferenceIndex,
  styleDependencyGraph?: StyleDependencyGraph,
): StyleModuleUsageSummary {
  styleDependencyGraph?.record(scssPath, styleDocument);

  const hasUnresolvedDynamicUsage = semanticReferenceIndex
    .findModuleUsages(scssPath)
    .some((usage) => usage.isDynamic && !usage.hasResolvedTargets);

  const selectorReachabilityCache = new Map<string, boolean>();
  const directUsageCache = new Map<string, boolean>();

  const selectors = listCanonicalSelectors(styleDocument).map((selector) => ({
    canonicalName: selector.canonicalName,
    range: selector.range,
    usage: readSelectorUsageSummary({ semanticReferenceIndex }, scssPath, selector.canonicalName),
    hasComposedReachability: styleDependencyGraph
      ? hasComposedReachability(
          styleDependencyGraph,
          semanticReferenceIndex,
          scssPath,
          selector.canonicalName,
          selectorReachabilityCache,
          directUsageCache,
        )
      : false,
  }));

  return {
    hasUnresolvedDynamicUsage,
    selectors,
    unusedSelectors: hasUnresolvedDynamicUsage
      ? []
      : selectors.filter(
          (selector) => !selector.usage.hasAnyReferences && !selector.hasComposedReachability,
        ),
  };
}

function hasComposedReachability(
  styleDependencyGraph: StyleDependencyGraph,
  semanticReferenceIndex: SemanticWorkspaceReferenceIndex,
  scssPath: string,
  canonicalName: string,
  reachabilityCache: Map<string, boolean>,
  directUsageCache: Map<string, boolean>,
): boolean {
  const key = selectorKey(scssPath, canonicalName);
  if (reachabilityCache.has(key)) return reachabilityCache.get(key)!;

  reachabilityCache.set(key, false);
  for (const incoming of styleDependencyGraph.getIncoming(scssPath, canonicalName)) {
    if (
      hasDirectUsage(
        semanticReferenceIndex,
        incoming.filePath,
        incoming.canonicalName,
        directUsageCache,
      )
    ) {
      reachabilityCache.set(key, true);
      return true;
    }
    if (
      hasComposedReachability(
        styleDependencyGraph,
        semanticReferenceIndex,
        incoming.filePath,
        incoming.canonicalName,
        reachabilityCache,
        directUsageCache,
      )
    ) {
      reachabilityCache.set(key, true);
      return true;
    }
  }
  return false;
}

function hasDirectUsage(
  semanticReferenceIndex: SemanticWorkspaceReferenceIndex,
  scssPath: string,
  canonicalName: string,
  cache: Map<string, boolean>,
): boolean {
  const key = selectorKey(scssPath, canonicalName);
  if (cache.has(key)) return cache.get(key)!;
  const value = readSelectorUsageSummary(
    { semanticReferenceIndex },
    scssPath,
    canonicalName,
  ).hasAnyReferences;
  cache.set(key, value);
  return value;
}

function selectorKey(filePath: string, canonicalName: string): string {
  return `${filePath}\u0000${canonicalName}`;
}

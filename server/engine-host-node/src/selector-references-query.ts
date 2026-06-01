import type { Range } from "@omena/shared";
import { readSelectorUsageSummary } from "../../engine-core-ts/src/core/query";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";

export interface SelectorReferenceTarget {
  readonly filePath: string;
  readonly canonicalName: string;
}

export interface SelectorReferenceLocation {
  readonly uri: string;
  readonly range: Range;
}

export function resolveSelectorReferenceLocations(
  deps: Pick<
    ProviderDeps,
    "semanticReferenceIndex" | "styleDependencyGraph" | "styleDocumentForPath"
  >,
  target: SelectorReferenceTarget,
): readonly SelectorReferenceLocation[] {
  const usage = readSelectorUsageSummary(deps, target.filePath, target.canonicalName);
  if (!usage.hasAnyReferences) return [];
  return usage.allSites.map((site) => ({
    uri: site.uri,
    range: site.range,
  }));
}

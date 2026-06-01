import type { Range } from "@omena/shared";
import type { EdgeCertainty } from "../semantic/certainty";
import {
  filterSelectorReferencePolicy,
  type SelectorReferencePolicy,
} from "../semantic/reference-policy";
import type { SemanticWorkspaceReferenceIndex } from "../semantic/workspace-reference-index";
import type { StyleDocumentHIR } from "../hir/style-types";
import type { StyleDependencyGraph } from "../semantic/style-dependency-graph";
import { pathToFileUrl } from "../util/text-utils";
import { listCanonicalSelectors } from "./find-style-selector";

export interface ResolvedReferenceSite {
  readonly uri: string;
  readonly range: Range;
  readonly className: string;
  readonly selectorCertainty: EdgeCertainty;
  readonly expansion: "direct" | "expanded";
  readonly referenceKind: "source" | "styleDependency";
}

export interface ReferenceQueryEnv {
  readonly semanticReferenceIndex: SemanticWorkspaceReferenceIndex;
  readonly styleDependencyGraph?: StyleDependencyGraph;
  readonly styleDocumentForPath?: (path: string) => StyleDocumentHIR | null;
}

export interface ReferenceSiteQueryOptions extends SelectorReferencePolicy {}

export function findSelectorReferenceSites(
  deps: ReferenceQueryEnv,
  scssPath: string,
  canonicalName: string,
  options?: ReferenceSiteQueryOptions,
): readonly ResolvedReferenceSite[] {
  const queryOptions = options?.minimumSelectorCertainty
    ? { minimumSelectorCertainty: options.minimumSelectorCertainty }
    : undefined;
  return filterSelectorReferencePolicy(
    [
      ...deps.semanticReferenceIndex
        .findSelectorReferences(scssPath, canonicalName, queryOptions)
        .map((site) => ({
          uri: site.uri,
          range: site.range,
          className: site.className,
          selectorCertainty: site.selectorCertainty,
          expansion: site.expansion,
          referenceKind: "source" as const,
        })),
      ...collectStyleDependencyReferenceSites(deps, scssPath, canonicalName),
    ],
    options,
  );
}

export function countSelectorReferenceSites(
  deps: ReferenceQueryEnv,
  scssPath: string,
  canonicalName: string,
  options?: ReferenceSiteQueryOptions,
): number {
  return findSelectorReferenceSites(deps, scssPath, canonicalName, options).length;
}

export function hasNonDirectSelectorReferenceSites(
  deps: ReferenceQueryEnv,
  scssPath: string,
  canonicalName: string,
): boolean {
  return findSelectorReferenceSites(deps, scssPath, canonicalName).some(
    (site) => site.expansion !== "direct",
  );
}

function collectStyleDependencyReferenceSites(
  deps: ReferenceQueryEnv,
  scssPath: string,
  canonicalName: string,
): readonly ResolvedReferenceSite[] {
  if (!deps.styleDependencyGraph || !deps.styleDocumentForPath) return [];

  const sites: ResolvedReferenceSite[] = [];
  const seenSelectors = new Set<string>();

  const visit = (filePath: string, selectorName: string): void => {
    const targetKey = selectorKey(filePath, selectorName);
    for (const incoming of deps.styleDependencyGraph!.getIncoming(filePath, selectorName)) {
      const incomingKey = selectorKey(incoming.filePath, incoming.canonicalName);
      if (seenSelectors.has(incomingKey)) continue;
      seenSelectors.add(incomingKey);

      const styleDocument = deps.styleDocumentForPath!(incoming.filePath);
      const selector = styleDocument
        ? (listCanonicalSelectors(styleDocument).find(
            (candidate) => candidate.canonicalName === incoming.canonicalName,
          ) ?? null)
        : null;

      if (selector) {
        sites.push({
          uri: pathToFileUrl(incoming.filePath),
          range: selector.bemSuffix?.rawTokenRange ?? selector.range,
          className: incoming.canonicalName,
          selectorCertainty: "exact",
          expansion: "direct",
          referenceKind: "styleDependency",
        });
      }

      if (incomingKey !== targetKey) visit(incoming.filePath, incoming.canonicalName);
    }
  };

  visit(scssPath, canonicalName);
  return sites;
}

function selectorKey(filePath: string, canonicalName: string): string {
  return `${filePath}\u0000${canonicalName}`;
}

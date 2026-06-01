import type { Range } from "@omena/shared";
import type { StyleDocumentHIR } from "../hir/style-types";
import type { SemanticWorkspaceReferenceIndex } from "../semantic/workspace-reference-index";
import type { StyleDependencyGraph } from "../semantic/style-dependency-graph";
import { readStyleModuleUsageSummary } from "./read-style-module-usage";

export interface UnusedSelectorFinding {
  readonly canonicalName: string;
  readonly range: Range;
}

export function findUnusedSelectors(
  scssPath: string,
  styleDocument: StyleDocumentHIR,
  semanticReferenceIndex: SemanticWorkspaceReferenceIndex,
  styleDependencyGraph?: StyleDependencyGraph,
): readonly UnusedSelectorFinding[] {
  return readStyleModuleUsageSummary(
    scssPath,
    styleDocument,
    semanticReferenceIndex,
    styleDependencyGraph,
  ).unusedSelectors.map((selector) => ({
    canonicalName: selector.canonicalName,
    range: selector.range,
  }));
}

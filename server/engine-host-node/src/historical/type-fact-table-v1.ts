import type { AnalysisEntry } from "../../../engine-core-ts/src/core/indexing/document-analysis-cache";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import type { TypeFactTableEntryV1, TypeFactTableV1 } from "../../../engine-core-ts/src/contracts";
import { createTypeFactTableEntryV1 } from "../../../engine-core-ts/src/contracts";
import type { SourceDocumentSnapshot } from "../checker-host/workspace-check-support";
import type { TypeFactControlFlowGraphProvider } from "../type-fact-control-flow-graph";

export interface TypeFactSourceEntry {
  readonly document: SourceDocumentSnapshot;
  readonly analysis: AnalysisEntry;
}

export interface CollectTypeFactTableV1Options {
  readonly workspaceRoot: string;
  readonly typeResolver: TypeResolver;
  readonly sourceEntries: readonly TypeFactSourceEntry[];
  readonly controlFlowGraphProvider?: TypeFactControlFlowGraphProvider;
}

export function collectTypeFactTableV1(options: CollectTypeFactTableV1Options): TypeFactTableV1 {
  const entries: TypeFactTableEntryV1[] = [];

  for (const { document, analysis } of options.sourceEntries) {
    for (const expression of analysis.sourceDocument.classExpressions) {
      if (expression.kind !== "symbolRef") continue;
      entries.push(
        createTypeFactTableEntryV1(
          document.filePath,
          expression.id,
          options.typeResolver.resolve(
            document.filePath,
            expression.rootName,
            options.workspaceRoot,
            expression.range,
            {
              sourceBinder: analysis.sourceBinder,
              sourceBindingGraph: analysis.sourceBindingGraph,
              rootBindingDeclId: expression.rootBindingDeclId ?? null,
            },
          ),
        ),
      );
    }
  }

  return entries.toSorted(
    (a, b) => a.filePath.localeCompare(b.filePath) || a.expressionId.localeCompare(b.expressionId),
  );
}

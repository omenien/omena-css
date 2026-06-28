import type { TypeFactTableV2 } from "../../engine-core-ts/src/contracts";
import { createTypeFactTableEntryV2 } from "../../engine-core-ts/src/contracts";
import type { CollectTypeFactTableV1Options } from "./historical/type-fact-table-v1";
import { tsTypeFactControlFlowGraphProvider } from "./type-fact-control-flow-graph";

export function collectTypeFactTableV2(options: CollectTypeFactTableV1Options): TypeFactTableV2 {
  const controlFlowGraphProvider =
    options.controlFlowGraphProvider ?? tsTypeFactControlFlowGraphProvider;
  return options.sourceEntries
    .flatMap(({ document, analysis }) =>
      analysis.sourceDocument.classExpressions.flatMap((expression) => {
        if (expression.kind !== "symbolRef") return [];
        return [
          createTypeFactTableEntryV2(
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
            controlFlowGraphProvider.controlFlowGraphForSymbolExpression(
              analysis.sourceFile,
              expression,
              document.filePath,
            ),
          ),
        ];
      }),
    )
    .toSorted(
      (a, b) =>
        a.filePath.localeCompare(b.filePath) || a.expressionId.localeCompare(b.expressionId),
    );
}

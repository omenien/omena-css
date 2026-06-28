import type { TypeFactTableV2 } from "../../engine-core-ts/src/contracts";
import { createTypeFactTableEntryV2 } from "../../engine-core-ts/src/contracts";
import type { CollectTypeFactTableV1Options } from "./historical/type-fact-table-v1";
import { createDefaultRustTypeFactControlFlowGraphProvider } from "./type-fact-control-flow-graph";

const DEFAULT_CONTROL_FLOW_GRAPH_PROVIDER = createDefaultRustTypeFactControlFlowGraphProvider();

export function collectTypeFactTableV2(options: CollectTypeFactTableV1Options): TypeFactTableV2 {
  const controlFlowGraphProvider =
    options.controlFlowGraphProvider ?? DEFAULT_CONTROL_FLOW_GRAPH_PROVIDER;
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
              document.content,
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

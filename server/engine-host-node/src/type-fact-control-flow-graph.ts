import type {
  TypeFactControlFlowBlockV2,
  TypeFactControlFlowGraphV2,
} from "../../engine-core-ts/src/contracts";
import { buildFlowBlockGraphSnapshot } from "../../engine-core-ts/src/core/flow/cfg";
import { buildFlowSlice } from "../../engine-core-ts/src/core/flow/flow-slice";
import type { SymbolRefClassExpressionHIR } from "../../engine-core-ts/src/core/hir/source-types";
import type ts from "../../engine-core-ts/src/ts-facade";

type MutableTypeFactControlFlowBlockV2 = {
  -readonly [Key in keyof TypeFactControlFlowBlockV2]: TypeFactControlFlowBlockV2[Key];
};

export function typeFactControlFlowGraphForSymbolExpression(
  sourceFile: ts.SourceFile,
  expression: SymbolRefClassExpressionHIR,
): TypeFactControlFlowGraphV2 | null {
  if (typeof sourceFile.getLineStarts !== "function") return null;

  const slice = buildFlowSlice(sourceFile, expression.range, expression.rootName);
  if (!slice) return null;

  const graph = buildFlowBlockGraphSnapshot(slice.nodes);
  return {
    entryBlockId: graph.entryBlockId,
    blocks: graph.blocks.map((block) => {
      const output: MutableTypeFactControlFlowBlockV2 = {
        id: block.id,
        kind: block.kind,
        transferKind: block.transferKind,
        successorBlockIds: block.successorBlockIds,
      };
      if (block.variableName) output.variableName = block.variableName;
      if (block.expressionKind) output.expressionKind = block.expressionKind;
      return output;
    }),
  };
}

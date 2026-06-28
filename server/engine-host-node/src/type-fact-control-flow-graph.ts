import type {
  TypeFactControlFlowBlockV2,
  TypeFactControlFlowGraphV2,
} from "../../engine-core-ts/src/contracts";
import { buildFlowBlockGraphSnapshot } from "../../engine-core-ts/src/core/flow/cfg";
import { buildFlowSlice } from "../../engine-core-ts/src/core/flow/flow-slice";
import type { SymbolRefClassExpressionHIR } from "../../engine-core-ts/src/core/hir/source-types";
import type ts from "../../engine-core-ts/src/ts-facade";
import { positionOfLineChar } from "../../engine-core-ts/src/ts-facade";

type MutableTypeFactControlFlowBlockV2 = {
  -readonly [Key in keyof TypeFactControlFlowBlockV2]: TypeFactControlFlowBlockV2[Key];
};

export interface TypeFactControlFlowGraphProvider {
  controlFlowGraphForSymbolExpression(
    sourceFile: ts.SourceFile,
    expression: SymbolRefClassExpressionHIR,
    sourcePath: string,
  ): TypeFactControlFlowGraphV2 | null;
}

export interface RustTypeFactControlFlowGraphInput {
  readonly sourcePath: string;
  readonly source: string;
  readonly sourceLanguage: string;
  readonly variableName: string;
  readonly referenceByteOffset: number;
}

export type RunRustTypeFactControlFlowGraph = (
  input: RustTypeFactControlFlowGraphInput,
) => string | null | undefined;

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

export const tsTypeFactControlFlowGraphProvider: TypeFactControlFlowGraphProvider = {
  controlFlowGraphForSymbolExpression(sourceFile, expression) {
    return typeFactControlFlowGraphForSymbolExpression(sourceFile, expression);
  },
};

export function rustTypeFactControlFlowGraphProvider(
  run: RunRustTypeFactControlFlowGraph,
): TypeFactControlFlowGraphProvider {
  return {
    controlFlowGraphForSymbolExpression(sourceFile, expression, sourcePath) {
      if (typeof sourceFile.getLineStarts !== "function") return null;
      const sourceLanguage = sourceLanguageForPath(sourcePath);
      if (!sourceLanguage) return null;

      const referencePosition = positionOfLineChar(sourceFile, expression.range.start);
      const raw = run({
        sourcePath,
        source: sourceFile.text,
        sourceLanguage,
        variableName: expression.rootName,
        referenceByteOffset: utf8ByteOffsetAtPosition(sourceFile.text, referencePosition),
      });
      if (!raw) return null;

      try {
        const graph = JSON.parse(raw) as unknown;
        return isTypeFactControlFlowGraphV2(graph) ? graph : null;
      } catch {
        return null;
      }
    },
  };
}

function sourceLanguageForPath(sourcePath: string): string | null {
  const normalized = sourcePath.toLowerCase();
  if (normalized.endsWith(".tsx")) return "typescriptreact";
  if (normalized.endsWith(".ts") || normalized.endsWith(".mts") || normalized.endsWith(".cts")) {
    return "typescript";
  }
  if (normalized.endsWith(".jsx")) return "javascriptreact";
  if (normalized.endsWith(".js") || normalized.endsWith(".mjs") || normalized.endsWith(".cjs")) {
    return "javascript";
  }
  if (normalized.endsWith(".vue")) return "vue";
  return null;
}

function utf8ByteOffsetAtPosition(text: string, position: number): number {
  return Buffer.byteLength(text.slice(0, position), "utf8");
}

function isTypeFactControlFlowGraphV2(value: unknown): value is TypeFactControlFlowGraphV2 {
  if (!value || typeof value !== "object") return false;
  const graph = value as Partial<TypeFactControlFlowGraphV2>;
  return typeof graph.entryBlockId === "string" && Array.isArray(graph.blocks);
}

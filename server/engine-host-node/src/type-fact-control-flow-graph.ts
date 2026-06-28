import type { TypeFactControlFlowGraphV2 } from "../../engine-core-ts/src/contracts";
import type { SymbolRefClassExpressionHIR } from "../../engine-core-ts/src/core/hir/source-types";
import {
  loadDefaultOmenaNapiSourceFrontendBinding,
  type OmenaNapiSourceFrontendBinding,
} from "./omena-napi-source-frontend-binding";

export interface TypeFactControlFlowGraphProvider {
  controlFlowGraphForSymbolExpression(
    source: string,
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

export interface DefaultRustTypeFactControlFlowGraphProviderOptions {
  readonly loadBinding?: () => OmenaNapiSourceFrontendBinding | null | undefined;
}

export function rustTypeFactControlFlowGraphProvider(
  run: RunRustTypeFactControlFlowGraph,
): TypeFactControlFlowGraphProvider {
  return {
    controlFlowGraphForSymbolExpression(source, expression, sourcePath) {
      const sourceLanguage = sourceLanguageForPath(sourcePath);
      if (!sourceLanguage) return null;

      const referencePosition = utf16OffsetAtPosition(source, expression.range.start);
      const raw = run({
        sourcePath,
        source,
        sourceLanguage,
        variableName: expression.rootName,
        referenceByteOffset: utf8ByteOffsetAtPosition(source, referencePosition),
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

export function createDefaultRustTypeFactControlFlowGraphProvider(
  options: DefaultRustTypeFactControlFlowGraphProviderOptions = {},
): TypeFactControlFlowGraphProvider {
  const loadBinding = options.loadBinding ?? loadDefaultOmenaNapiSourceFrontendBinding;
  return rustTypeFactControlFlowGraphProvider((input) => {
    const binding = loadBinding();
    const read = binding?.readSourceTypeFactControlFlowGraphJson;
    if (typeof read !== "function") return null;
    try {
      return read(
        input.sourcePath,
        input.source,
        input.sourceLanguage,
        input.variableName,
        input.referenceByteOffset,
      );
    } catch {
      return null;
    }
  });
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

function utf16OffsetAtPosition(
  source: string,
  position: { readonly line: number; readonly character: number },
): number {
  let line = 0;
  let lineStart = 0;
  for (let index = 0; index < source.length && line < position.line; index += 1) {
    const char = source.charCodeAt(index);
    if (char === 13 || char === 10) {
      if (char === 13 && source.charCodeAt(index + 1) === 10) index += 1;
      line += 1;
      lineStart = index + 1;
    }
  }
  return Math.min(lineStart + position.character, source.length);
}

function utf8ByteOffsetAtPosition(text: string, position: number): number {
  return Buffer.byteLength(text.slice(0, position), "utf8");
}

function isTypeFactControlFlowGraphV2(value: unknown): value is TypeFactControlFlowGraphV2 {
  if (!value || typeof value !== "object") return false;
  const graph = value as Partial<TypeFactControlFlowGraphV2>;
  return typeof graph.entryBlockId === "string" && Array.isArray(graph.blocks);
}

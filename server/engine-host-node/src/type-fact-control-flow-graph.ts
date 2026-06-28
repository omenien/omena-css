import { createRequire } from "node:module";
import path from "node:path";
import type { TypeFactControlFlowGraphV2 } from "../../engine-core-ts/src/contracts";
import type { SymbolRefClassExpressionHIR } from "../../engine-core-ts/src/core/hir/source-types";
import type ts from "../../engine-core-ts/src/ts-facade";
import { positionOfLineChar } from "../../engine-core-ts/src/ts-facade";

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

export interface OmenaNapiSourceFrontendBinding {
  readonly readSourceTypeFactControlFlowGraphJson?: (
    sourcePath: string,
    source: string,
    sourceLanguage: string,
    variableName: string,
    referenceByteOffset: number,
  ) => string | null | undefined;
}

export interface DefaultRustTypeFactControlFlowGraphProviderOptions {
  readonly loadBinding?: () => OmenaNapiSourceFrontendBinding | null | undefined;
}

const requireFromHostNode = createRequire(__filename);
const DEFAULT_OMENA_NAPI_BINDING_CANDIDATES = [
  "@omena/napi",
  path.resolve(process.cwd(), "rust/crates/omena-napi/pkg/index.js"),
  path.resolve(__dirname, "../../../rust/crates/omena-napi/pkg/index.js"),
] as const;
let cachedDefaultOmenaNapiBinding: OmenaNapiSourceFrontendBinding | null | undefined;

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

export function createDefaultRustTypeFactControlFlowGraphProvider(
  options: DefaultRustTypeFactControlFlowGraphProviderOptions = {},
): TypeFactControlFlowGraphProvider {
  const loadBinding = options.loadBinding ?? loadDefaultOmenaNapiBinding;
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

function loadDefaultOmenaNapiBinding(): OmenaNapiSourceFrontendBinding | null {
  if (cachedDefaultOmenaNapiBinding !== undefined) {
    return cachedDefaultOmenaNapiBinding;
  }

  for (const candidate of DEFAULT_OMENA_NAPI_BINDING_CANDIDATES) {
    try {
      const binding = bindingFromModule(requireFromHostNode(candidate) as unknown);
      if (binding) {
        cachedDefaultOmenaNapiBinding = binding;
        return binding;
      }
    } catch {
      // Optional local/package binding. Absence means no CFG, not a TS fallback.
    }
  }

  cachedDefaultOmenaNapiBinding = null;
  return cachedDefaultOmenaNapiBinding;
}

function bindingFromModule(value: unknown): OmenaNapiSourceFrontendBinding | null {
  if (isOmenaNapiSourceFrontendBinding(value)) return value;
  if (!value || typeof value !== "object") return null;
  const maybeDefault = (value as { readonly default?: unknown }).default;
  return isOmenaNapiSourceFrontendBinding(maybeDefault) ? maybeDefault : null;
}

function isOmenaNapiSourceFrontendBinding(value: unknown): value is OmenaNapiSourceFrontendBinding {
  return (
    !!value &&
    typeof value === "object" &&
    typeof (value as OmenaNapiSourceFrontendBinding).readSourceTypeFactControlFlowGraphJson ===
      "function"
  );
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

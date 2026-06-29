import {
  type StringTypeFactsV2,
  type TypeFactControlFlowBlockV2,
  type TypeFactControlFlowGraphV2,
} from "../../engine-core-ts/src/contracts";
import {
  TOP_CLASS_VALUE,
  charInclusionClassValue,
  compositeClassValue,
  exactClassValue,
  finiteSetClassValue,
  joinClassValues,
  prefixClassValue,
  prefixSuffixClassValue,
  suffixClassValue,
  type AbstractClassValue,
  type CharInclusionClassValue,
  type CompositeClassValue,
  type PrefixClassValue,
  type PrefixSuffixClassValue,
  type SuffixClassValue,
} from "../../engine-core-ts/src/core/abstract-value/class-value-domain";
import { toFlowResolution, type FlowResolution } from "../../engine-core-ts/src/core/flow/lattice";
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

interface RustControlFlowSymbolValueResolutionInput {
  readonly source: string;
  readonly sourcePath: string;
  readonly expression: SymbolRefClassExpressionHIR;
  readonly provider?: TypeFactControlFlowGraphProvider;
}

interface FlowState {
  readonly abstractValue: AbstractClassValue;
  readonly reason: "flowLiteral" | "flowBranch";
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

export function resolveSymbolValuesFromRustControlFlow(
  input: RustControlFlowSymbolValueResolutionInput,
): FlowResolution | null {
  const provider = input.provider ?? createDefaultRustTypeFactControlFlowGraphProvider();
  const graph = provider.controlFlowGraphForSymbolExpression(
    input.source,
    input.expression,
    input.sourcePath,
  );
  if (!graph) return null;
  return analyzeRustControlFlowGraph(graph);
}

function analyzeRustControlFlowGraph(graph: TypeFactControlFlowGraphV2): FlowResolution | null {
  const blocksById = new Map(graph.blocks.map((block) => [block.id, block] as const));
  const predecessorIdsByBlockId = rustControlFlowPredecessorIds(graph.blocks);
  const states = new Map<string, FlowState | null>(
    graph.blocks.map((block) => [block.id, null] as const),
  );

  for (let iteration = 0; iteration < Math.max(graph.blocks.length * 2, 1); iteration += 1) {
    let changed = false;
    for (const block of graph.blocks) {
      const incoming = incomingRustControlFlowState(block, predecessorIdsByBlockId, states);
      const next = applyRustControlFlowBlock(block, incoming);
      if (!sameFlowState(states.get(block.id) ?? null, next)) {
        states.set(block.id, next);
        changed = true;
      }
    }
    if (!changed) break;
  }

  const exit = blocksById.get("exit") ?? graph.blocks.at(-1);
  const state = exit ? (states.get(exit.id) ?? null) : null;
  return state ? toFlowResolution(state) : null;
}

function rustControlFlowPredecessorIds(
  blocks: readonly TypeFactControlFlowBlockV2[],
): ReadonlyMap<string, readonly string[]> {
  const predecessors = new Map<string, string[]>();
  for (const block of blocks) {
    for (const successor of block.successorBlockIds) {
      const ids = predecessors.get(successor) ?? [];
      ids.push(block.id);
      predecessors.set(successor, ids);
    }
  }
  return predecessors;
}

function incomingRustControlFlowState(
  block: TypeFactControlFlowBlockV2,
  predecessorIdsByBlockId: ReadonlyMap<string, readonly string[]>,
  states: ReadonlyMap<string, FlowState | null>,
): FlowState | null {
  const predecessorIds = predecessorIdsByBlockId.get(block.id) ?? [];
  if (predecessorIds.length === 0) return null;
  return predecessorIds
    .map((id) => states.get(id) ?? null)
    .reduce<FlowState | null>((merged, state) => joinFlowStates(merged, state), null);
}

function applyRustControlFlowBlock(
  block: TypeFactControlFlowBlockV2,
  incoming: FlowState | null,
): FlowState | null {
  if (
    block.facts &&
    (block.transferKind === "assignFacts" || block.transferKind === "concatFacts")
  ) {
    const abstractValue = abstractValueFromStringFacts(block.facts);
    return abstractValue ? { abstractValue, reason: reasonFromStringFacts(block.facts) } : null;
  }
  if (block.successorBlockIds.length > 1 && incoming) {
    return { ...incoming, reason: "flowBranch" };
  }
  return incoming;
}

function joinFlowStates(left: FlowState | null, right: FlowState | null): FlowState | null {
  if (!left) return right;
  if (!right) return left;
  const abstractValue = joinClassValues(left.abstractValue, right.abstractValue);
  return {
    abstractValue,
    reason:
      left.reason === "flowBranch" ||
      right.reason === "flowBranch" ||
      !sameAbstractValue(left.abstractValue, right.abstractValue)
        ? "flowBranch"
        : "flowLiteral",
  };
}

function reasonFromStringFacts(facts: StringTypeFactsV2): FlowState["reason"] {
  if (facts.kind === "finiteSet" && (facts.values?.length ?? 0) > 1) {
    return "flowBranch";
  }
  if (facts.kind === "constrained") {
    return "flowBranch";
  }
  return "flowLiteral";
}

function abstractValueFromStringFacts(facts: StringTypeFactsV2): AbstractClassValue | null {
  switch (facts.kind) {
    case "exact": {
      const value = facts.values?.[0];
      return value !== undefined ? exactClassValue(value) : TOP_CLASS_VALUE;
    }
    case "finiteSet":
      return finiteSetClassValue(facts.values ?? []);
    case "constrained":
      return constrainedAbstractValueFromStringFacts(facts);
    case "top":
      return TOP_CLASS_VALUE;
    case "unknown":
      return null;
    default:
      facts.kind satisfies never;
      return null;
  }
}

function constrainedAbstractValueFromStringFacts(facts: StringTypeFactsV2): AbstractClassValue {
  switch (facts.constraintKind) {
    case "prefix":
      return facts.prefix
        ? prefixClassValue(
            facts.prefix,
            provenance<PrefixClassValue["provenance"]>(facts.provenance),
          )
        : TOP_CLASS_VALUE;
    case "suffix":
      return facts.suffix
        ? suffixClassValue(
            facts.suffix,
            provenance<SuffixClassValue["provenance"]>(facts.provenance),
          )
        : TOP_CLASS_VALUE;
    case "prefixSuffix":
      return prefixSuffixClassValue(
        facts.prefix ?? "",
        facts.suffix ?? "",
        facts.minLen,
        provenance<PrefixSuffixClassValue["provenance"]>(facts.provenance),
      );
    case "charInclusion":
      return charInclusionClassValue(
        facts.charMust ?? "",
        facts.charMay ?? "",
        provenance<CharInclusionClassValue["provenance"]>(facts.provenance),
        Boolean(facts.mayIncludeOtherChars),
      );
    case "composite":
      return compositeClassValue({
        ...(facts.prefix ? { prefix: facts.prefix } : {}),
        ...(facts.suffix ? { suffix: facts.suffix } : {}),
        ...(facts.minLen !== undefined ? { minLength: facts.minLen } : {}),
        mustChars: facts.charMust ?? "",
        mayChars: facts.charMay ?? "",
        ...(facts.mayIncludeOtherChars ? { mayIncludeOtherChars: true } : {}),
        ...(facts.provenance
          ? { provenance: provenance<CompositeClassValue["provenance"]>(facts.provenance) }
          : {}),
      });
    case undefined:
      return TOP_CLASS_VALUE;
    default:
      facts.constraintKind satisfies never;
      return TOP_CLASS_VALUE;
  }
}

function provenance<T extends string | undefined>(value: string | undefined): T | undefined {
  return value as T | undefined;
}

function sameFlowState(left: FlowState | null, right: FlowState | null): boolean {
  if (!left || !right) return left === right;
  return left.reason === right.reason && sameAbstractValue(left.abstractValue, right.abstractValue);
}

function sameAbstractValue(left: AbstractClassValue, right: AbstractClassValue): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
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

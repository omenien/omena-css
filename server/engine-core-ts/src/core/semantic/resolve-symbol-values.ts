import type { SymbolRefClassExpressionHIR } from "../hir/source-types";
import { typeUnionResolution, type FlowResolution } from "../flow/lattice";
import type { TypeResolver } from "../ts/type-resolver";
import type { SourceBindingGraph } from "../binder/source-binding-graph";
import type { SourceBinderResult } from "../binder/scope-types";

interface SymbolValueResolutionEnv {
  readonly typeResolver: TypeResolver;
  readonly filePath: string;
  readonly workspaceRoot: string;
  readonly sourceBinder?: SourceBinderResult;
  readonly sourceBindingGraph?: SourceBindingGraph;
}

interface SymbolValueResolutionInput {
  readonly range: {
    readonly start: { readonly line: number; readonly character: number };
    readonly end: { readonly line: number; readonly character: number };
  };
  readonly rawReference: string;
  readonly rootName: string;
  readonly rootBindingDeclId?: string;
}

export function resolveSymbolClassValues(
  input: SymbolValueResolutionInput,
  env: SymbolValueResolutionEnv,
): FlowResolution | null {
  const resolved = env.typeResolver.resolve(
    env.filePath,
    input.rawReference,
    env.workspaceRoot,
    input.range,
    {
      ...(env.sourceBinder ? { sourceBinder: env.sourceBinder } : {}),
      ...(env.sourceBindingGraph ? { sourceBindingGraph: env.sourceBindingGraph } : {}),
      ...(input.rootBindingDeclId ? { rootBindingDeclId: input.rootBindingDeclId } : {}),
    },
  );
  return resolved.kind === "union" ? typeUnionResolution(resolved.values) : null;
}

export function resolveSymbolExpressionValues(
  ref: SymbolRefClassExpressionHIR,
  env: SymbolValueResolutionEnv,
): FlowResolution | null {
  return resolveSymbolClassValues(
    {
      range: ref.range,
      rawReference: ref.rawReference,
      rootName: ref.rootName,
      ...(ref.rootBindingDeclId ? { rootBindingDeclId: ref.rootBindingDeclId } : {}),
    },
    env,
  );
}

import type { Range, ResolvedType } from "@omena/shared";
import type { SourceBindingGraph } from "../binder/source-binding-graph";
import type { SourceBinderResult } from "../binder/scope-types";

/**
 * TypeResolver resolves a bare identifier like `cx(size)` to its
 * string-literal union members. Implementations are supplied by
 * the host/runtime layer; engine-core only owns the contract.
 */
export interface TypeResolver {
  /**
   * Given a file path, an identifier name visible at that file,
   * and the owning workspace root, return the identifier's
   * string-literal union type.
   *
   * The method must always return a ResolvedType — `unresolvable`
   * is a valid "the checker could not narrow this" result.
   */
  resolve(
    filePath: string,
    variableName: string,
    workspaceRoot: string,
    range: Range,
    options?: ResolveTypeOptions,
  ): ResolvedType;

  /** Drop the cached program for one workspace (e.g. on tsconfig change). */
  invalidate(workspaceRoot: string): void;

  /** Drop every cached program. */
  clear(): void;
}

export interface ResolveTypeOptions {
  readonly sourceBinder?: SourceBinderResult;
  readonly sourceBindingGraph?: SourceBindingGraph;
  readonly rootBindingDeclId?: string | null;
}

export const UNRESOLVABLE_TYPE: ResolvedType = { kind: "unresolvable", values: [] };

export class UnresolvableTypeResolver implements TypeResolver {
  resolve(): ResolvedType {
    return UNRESOLVABLE_TYPE;
  }

  invalidate(): void {}

  clear(): void {}
}

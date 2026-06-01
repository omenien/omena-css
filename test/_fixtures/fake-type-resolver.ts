import type { Range, ResolvedType } from "@omena/shared";
import type {
  ResolveTypeOptions,
  TypeResolver,
} from "../../server/engine-core-ts/src/core/ts/type-resolver";

/**
 * Shared test double for `TypeResolver`. Defaults to
 * `unresolvable`; pass an array of values to get a union.
 */
export class FakeTypeResolver implements TypeResolver {
  private readonly values: readonly string[];

  constructor(values: readonly string[] = []) {
    this.values = values;
  }

  resolve(
    _filePath?: string,
    _variableName?: string,
    _workspaceRoot?: string,
    _range?: Range,
    _options?: ResolveTypeOptions,
  ): ResolvedType {
    return this.values.length > 0
      ? { kind: "union", values: this.values }
      : { kind: "unresolvable", values: [] };
  }

  invalidate(): void {}
  clear(): void {}
}

import type { TypeResolver } from "../../engine-core-ts/src/core/ts/type-resolver";
import { TsgoProbeTypeResolver } from "./tsgo-probe-type-resolver";

export type TypeFactBackendKind = "typescript-current" | "tsgo";

export interface SelectTypeResolverOptions {
  readonly typeResolver?: TypeResolver;
  readonly typeBackend?: TypeFactBackendKind;
  readonly env?: NodeJS.ProcessEnv;
}

export interface TypeResolverSelection {
  readonly backend: TypeFactBackendKind;
  readonly typeResolver: TypeResolver;
}

export function resolveTypeFactBackendKind(
  env: NodeJS.ProcessEnv = process.env,
): TypeFactBackendKind {
  const value = env.OMENA_TYPE_FACT_BACKEND ?? "tsgo";
  if (value === "typescript-current" || value === "tsgo") {
    return value;
  }

  throw new Error(`Unknown type fact backend: ${value}`);
}

export function selectTypeResolver(options: SelectTypeResolverOptions): TypeResolverSelection {
  const backend = options.typeBackend ?? resolveTypeFactBackendKind(options.env);
  if (options.typeResolver) {
    return {
      backend,
      typeResolver: options.typeResolver,
    };
  }

  if (backend === "tsgo") {
    return {
      backend,
      typeResolver: new TsgoProbeTypeResolver(),
    };
  }

  throw new Error(
    "typescript-current requires an explicit TypeResolver; the default runtime no longer constructs the current TypeScript workspace resolver.",
  );
}

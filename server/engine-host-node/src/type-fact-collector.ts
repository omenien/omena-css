import type { TypeResolver } from "../../engine-core-ts/src/core/ts/type-resolver";
import {
  downcastFactsV2ToV1,
  type TypeFactTableV1,
  type TypeFactTableV2,
} from "../../engine-core-ts/src/contracts";
import {
  type CollectTypeFactTableV1Options,
  type TypeFactSourceEntry,
} from "./historical/type-fact-table-v1";
import { collectTypeFactTableV2 } from "./type-fact-table-v2";
import {
  selectTypeResolver,
  type SelectTypeResolverOptions,
  type TypeFactBackendKind,
} from "./type-backend";
import { TsgoProbeTypeResolver } from "./tsgo-probe-type-resolver";
import {
  collectTypeFactTableV2WithTsgo,
  type RunTsgoTypeFactWorker,
} from "./tsgo-type-fact-collector";
import type { TypeFactControlFlowGraphProvider } from "./type-fact-control-flow-graph";

export interface SelectTypeFactCollectorOptions extends SelectTypeResolverOptions {
  readonly findTsgoConfigFile?: (workspaceRoot: string) => string | null;
  readonly runTsgoTypeFactWorker?: RunTsgoTypeFactWorker;
  readonly controlFlowGraphProvider?: TypeFactControlFlowGraphProvider;
}

export interface TypeFactCollectorSelection {
  readonly backend: TypeFactBackendKind;
  readonly typeResolver: TypeResolver;
  collectV1(options: CollectTypeFactCollectorOptions): TypeFactTableV1;
  collectV2(options: CollectTypeFactCollectorOptions): TypeFactTableV2;
  collectV1Async(options: CollectTypeFactCollectorOptions): Promise<TypeFactTableV1>;
  collectV2Async(options: CollectTypeFactCollectorOptions): Promise<TypeFactTableV2>;
}

export interface CollectTypeFactCollectorOptions {
  readonly workspaceRoot: string;
  readonly sourceEntries: readonly TypeFactSourceEntry[];
}

export function selectTypeFactCollector(
  options: SelectTypeFactCollectorOptions,
): TypeFactCollectorSelection {
  const resolverSelection = selectTypeResolver(options);
  const findTsgoConfigFile = options.findTsgoConfigFile;
  const runTsgoTypeFactWorker = options.runTsgoTypeFactWorker;
  const controlFlowGraphProvider = options.controlFlowGraphProvider;
  const shouldUseTsgoCollector =
    resolverSelection.backend === "tsgo" &&
    (runTsgoTypeFactWorker || resolverSelection.typeResolver instanceof TsgoProbeTypeResolver);
  const collectV2 = (collectOptions: CollectTypeFactCollectorOptions): TypeFactTableV2 => {
    if (shouldUseTsgoCollector) {
      throw new Error("tsgo type fact collection requires the async collector path");
    }
    return collectTypeFactTableV2(
      withTypeResolver(collectOptions, resolverSelection.typeResolver, controlFlowGraphProvider),
    );
  };
  const collectV2Async = async (
    collectOptions: CollectTypeFactCollectorOptions,
  ): Promise<TypeFactTableV2> => {
    if (shouldUseTsgoCollector) {
      return collectTypeFactTableV2WithTsgo({
        ...withTypeResolver(
          collectOptions,
          resolverSelection.typeResolver,
          controlFlowGraphProvider,
        ),
        ...(findTsgoConfigFile ? { findConfigFile: findTsgoConfigFile } : {}),
        ...(runTsgoTypeFactWorker ? { runWorker: runTsgoTypeFactWorker } : {}),
      });
    }
    return collectTypeFactTableV2(
      withTypeResolver(collectOptions, resolverSelection.typeResolver, controlFlowGraphProvider),
    );
  };

  return {
    backend: resolverSelection.backend,
    typeResolver: resolverSelection.typeResolver,
    collectV1(collectOptions) {
      return collectV2(collectOptions).map((entry) => ({
        filePath: entry.filePath,
        expressionId: entry.expressionId,
        facts: downcastFactsV2ToV1(entry.facts),
      }));
    },
    collectV2,
    async collectV1Async(collectOptions) {
      return (await collectV2Async(collectOptions)).map((entry) => ({
        filePath: entry.filePath,
        expressionId: entry.expressionId,
        facts: downcastFactsV2ToV1(entry.facts),
      }));
    },
    collectV2Async,
  };
}

function withTypeResolver(
  options: CollectTypeFactCollectorOptions,
  typeResolver: TypeResolver,
  controlFlowGraphProvider?: TypeFactControlFlowGraphProvider,
): CollectTypeFactTableV1Options {
  return {
    workspaceRoot: options.workspaceRoot,
    sourceEntries: options.sourceEntries,
    typeResolver,
    ...(controlFlowGraphProvider ? { controlFlowGraphProvider } : {}),
  };
}

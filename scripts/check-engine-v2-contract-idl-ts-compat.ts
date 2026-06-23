import type {
  EngineInputV2,
  EngineOutputV2,
  QueryResultV2,
  StringTypeFactsV2,
  TypeFactTableEntryV2,
} from "../server/engine-core-ts/src/contracts";
import type {
  EngineInputV2Json,
  StringTypeFactsV2Json,
  TypeFactEntryV2Json,
} from "../server/engine-core-ts/src/contracts/engine-v2-input-idl.generated";
import type {
  EngineOutputV2Json,
  QueryResultV2Json,
} from "../server/engine-core-ts/src/contracts/engine-v2-output-idl.generated";
import type { HostEngineOutputV2Json } from "../server/engine-host-node/src/engine-output-v2-idl.generated";
import type { EngineQueryResultV2Json } from "../server/engine-host-node/src/engine-query-v2-idl.generated";
import type * as EngineOutputBuilder from "../server/engine-host-node/src/engine-output-v2";
import type * as EngineQueryBuilder from "../server/engine-host-node/src/engine-query-v2";

type AssertAssignable<Actual extends Expected, Expected> = [Actual] extends [Expected]
  ? true
  : never;

type EngineV2ContractIdlCompatibility = [
  AssertAssignable<StringTypeFactsV2, StringTypeFactsV2Json>,
  AssertAssignable<TypeFactTableEntryV2, TypeFactEntryV2Json>,
  AssertAssignable<EngineInputV2, EngineInputV2Json>,
  AssertAssignable<QueryResultV2, QueryResultV2Json>,
  AssertAssignable<EngineOutputV2, EngineOutputV2Json>,
  AssertAssignable<
    ReturnType<typeof EngineOutputBuilder.buildEngineOutputV2>,
    HostEngineOutputV2Json
  >,
  AssertAssignable<
    ReturnType<typeof EngineQueryBuilder.buildSelectedQueryResultsV2>,
    readonly EngineQueryResultV2Json[]
  >,
];

export type { EngineV2ContractIdlCompatibility };

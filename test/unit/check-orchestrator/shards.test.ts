import { describe, expect, it } from "vitest";
import { loadCheckManifest, resolveGateTarget } from "../../../packages/check-orchestrator/src";
import {
  bundleShardNames,
  resolveShardMembers,
} from "../../../packages/check-orchestrator/src/manifest/shards";

describe("check orchestrator bundle shards", () => {
  it(
    "isolates the long-running query API and query core paths",
    () => {
      const closureFast = resolveGateTarget(loadCheckManifest(), "rust/closure-fast");
      const deps = closureFast?.referencedTargets ?? [];

      expect(resolveShardMembers("rust/closure-fast", "query-api", deps)).toEqual(
        new Set(["rust/runtime-query-api-hardening"]),
      );
      expect(resolveShardMembers("rust/closure-fast", "query-core", deps)).toEqual(
        new Set(["rust/omena-query/core-contract", "rust/omena-query/runtime-contract"]),
      );
      expect(resolveShardMembers("rust/closure-fast", "rest", deps)).not.toContain(
        "rust/runtime-query-api-hardening",
      );
      expect(bundleShardNames("rust/closure-fast")).toContain("query-api");
      expect(bundleShardNames("rust/closure-fast").at(-1)).toBe("rest");
    },
    5_000,
  );
});

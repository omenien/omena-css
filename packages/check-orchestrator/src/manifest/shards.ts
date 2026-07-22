export interface BundleShardTable {
  readonly [shardName: string]: readonly string[];
}

const REST_SHARD_NAME = "rest";

export const BUNDLE_SHARDS: { readonly [bundleId: string]: BundleShardTable } = {
  "rust/closure-fast": {
    "heavy-query": [
      "rust/runtime-query-api-hardening",
      "rust/omena-query/boundary",
      "rust/omena-lsp-server/boundary",
      "rust/omena-cascade/boundary",
    ],
    "diff-test": [
      "rust/omena-diff-test-boundary",
      "rust/omena-bundler/linked-emission-byte-differential",
      "rust/streaming-ifds-relocation-gate",
      "rust/streaming-ifds-settle-soak",
      "rust/discharge-ledger",
      "rust/semantic/preservation-model-conformance",
      "rust/translation-validation-kill-rate",
      "rust/verification-plane-closure",
      "rust/oss-corpus-farm-determinism",
      "rust/oss-corpus-farm-regressions",
      "rust/lint-finding-census",
    ],
  },
};

export function bundleShardNames(bundleId: string): readonly string[] {
  const table = BUNDLE_SHARDS[bundleId];
  if (!table) {
    return [];
  }
  return [...Object.keys(table), REST_SHARD_NAME];
}

export function resolveShardMembers(
  bundleId: string,
  shardName: string,
  bundleDeps: readonly string[],
): ReadonlySet<string> {
  const table = BUNDLE_SHARDS[bundleId];
  if (!table) {
    throw new Error(`Bundle "${bundleId}" has no shard table.`);
  }
  if (bundleDeps.length === 0) {
    throw new Error(`Bundle "${bundleId}" has no resolvable deps for sharding.`);
  }
  const depSet = new Set(bundleDeps);
  const named = new Set<string>();
  for (const [name, members] of Object.entries(table)) {
    for (const member of members) {
      if (!depSet.has(member)) {
        throw new Error(
          `Shard "${name}" of bundle "${bundleId}" pins "${member}", which is not a bundle dep.`,
        );
      }
      if (named.has(member)) {
        throw new Error(
          `Gate "${member}" appears in more than one named shard of bundle "${bundleId}".`,
        );
      }
      named.add(member);
    }
  }
  if (shardName === REST_SHARD_NAME) {
    return new Set(bundleDeps.filter((dep) => !named.has(dep)));
  }
  const members = table[shardName];
  if (!members) {
    throw new Error(
      `Unknown shard "${shardName}" for bundle "${bundleId}". Known: ${bundleShardNames(bundleId).join(", ")}.`,
    );
  }
  return new Set(members);
}

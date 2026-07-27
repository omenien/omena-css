export interface RustProductTestShard {
  readonly id: string;
  readonly packages: readonly string[];
  readonly workspaceRemainder: boolean;
}

// The remainder shard adopts new workspace crates automatically; only measured long runners are isolated.
export const RUST_PRODUCT_TEST_SHARDS = [
  {
    id: "workspace",
    packages: [],
    workspaceRemainder: true,
  },
  {
    id: "differential",
    packages: ["omena-diff-test"],
    workspaceRemainder: false,
  },
  {
    id: "benchmarks",
    packages: ["omena-benchmarks"],
    workspaceRemainder: false,
  },
] as const satisfies readonly RustProductTestShard[];

export const RUST_PRODUCT_TEST_SHARD_IDS = RUST_PRODUCT_TEST_SHARDS.map(({ id }) => id);

export function resolveRustProductTestShard(id: string): RustProductTestShard {
  const shard = RUST_PRODUCT_TEST_SHARDS.find((candidate) => candidate.id === id);
  if (!shard) {
    throw new Error(
      `Unknown Rust product-test shard "${id}". Expected one of: ${RUST_PRODUCT_TEST_SHARD_IDS.join(", ")}`,
    );
  }
  return shard;
}

export function rustProductTestPackagesByShard(
  workspacePackages: readonly string[],
): ReadonlyMap<string, readonly string[]> {
  const isolatedPackages = new Set(
    RUST_PRODUCT_TEST_SHARDS.flatMap((shard) =>
      shard.workspaceRemainder ? [] : [...shard.packages],
    ),
  );

  return new Map(
    RUST_PRODUCT_TEST_SHARDS.map((shard) => [
      shard.id,
      shard.workspaceRemainder
        ? workspacePackages.filter((packageName) => !isolatedPackages.has(packageName)).toSorted()
        : [...shard.packages].toSorted(),
    ]),
  );
}

export function rustProductTestCargoArgs(shard: RustProductTestShard): readonly string[] {
  const selection = shard.workspaceRemainder
    ? [
        "--workspace",
        ...RUST_PRODUCT_TEST_SHARDS.flatMap((candidate) =>
          candidate.workspaceRemainder
            ? []
            : candidate.packages.flatMap((packageName) => ["--exclude", packageName]),
        ),
      ]
    : shard.packages.flatMap((packageName) => ["-p", packageName]);

  return [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    ...selection,
    "--all-features",
    "--no-fail-fast",
  ];
}

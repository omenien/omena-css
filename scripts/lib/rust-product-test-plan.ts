export interface RustProductTestShard {
  readonly id: string;
  readonly packages: readonly string[];
  readonly workspaceRemainder: boolean;
}

export interface RustProductTestCargoInvocation {
  readonly id: string;
  readonly args: readonly string[];
}

// Keep verifier-free product roots out of a workspace-wide all-features build. Cargo unifies
// normal dependency features across selected workspace packages, so testing the CLI and LSP in
// one invocation would make the CLI-owned attestation feature appear in the LSP dependency graph.
export const RUST_PRODUCT_TEST_FEATURE_ISOLATION_PACKAGES = ["omena-lsp-server"] as const;

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

export function rustProductTestCargoInvocations(
  shard: RustProductTestShard,
): readonly RustProductTestCargoInvocation[] {
  if (!shard.workspaceRemainder) {
    return [
      cargoInvocation(
        shard.id,
        shard.packages.flatMap((packageName) => ["-p", packageName]),
      ),
    ];
  }

  const ordinaryExclusions = RUST_PRODUCT_TEST_SHARDS.flatMap((candidate) =>
    candidate.workspaceRemainder ? [] : candidate.packages,
  );
  return [
    cargoInvocation("workspace-remainder", [
      "--workspace",
      ...ordinaryExclusions.flatMap((packageName) => ["--exclude", packageName]),
      ...RUST_PRODUCT_TEST_FEATURE_ISOLATION_PACKAGES.flatMap((packageName) => [
        "--exclude",
        packageName,
      ]),
    ]),
    ...RUST_PRODUCT_TEST_FEATURE_ISOLATION_PACKAGES.map((packageName) =>
      cargoInvocation(`feature-isolated-${packageName}`, ["-p", packageName]),
    ),
  ];
}

function cargoInvocation(id: string, selection: readonly string[]): RustProductTestCargoInvocation {
  return {
    id,
    args: [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      ...selection,
      "--all-features",
      "--no-fail-fast",
    ],
  };
}

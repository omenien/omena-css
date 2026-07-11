import type { CiProbeProfileId } from "./probes";

export interface AffectedCheckReason {
  readonly path: string;
  readonly profiles: readonly CiProbeProfileId[];
  readonly reason: string;
  readonly requiresFullCi: boolean;
}

export interface AffectedCheckPlan {
  readonly changedPaths: readonly string[];
  readonly profiles: readonly CiProbeProfileId[];
  readonly requiresFullCi: boolean;
  readonly reasons: readonly AffectedCheckReason[];
}

const PROFILE_ORDER: readonly CiProbeProfileId[] = [
  "orchestrator",
  "rust-cli",
  "cross-platform-cli",
  "rust-workspace",
  "closure-diff",
  "linux-benchmark",
  "verify",
];

const CLI_SUPPORT_PATHS = [
  "scripts/check-rust-omena-cli-",
  ".github/workflows/release-cli.yml",
] as const;

const PERFORMANCE_PATH_MARKERS = [
  "rust/crates/omena-benchmarks/",
  "rust/crates/omena-streaming-ifds/",
  "scripts/check-rust-z5-perf-",
  "scripts/check-rust-streaming-ifds-relocation-",
  "scripts/check-rust-benchmark-",
  "benchmark-artifacts/",
] as const;

export function buildAffectedCheckPlan(changedPaths: readonly string[]): AffectedCheckPlan {
  const normalizedPaths = [...new Set(changedPaths.map(normalizePath).filter(Boolean))].toSorted();
  const reasons = normalizedPaths
    .filter((changedPath) => !changedPath.startsWith(".personal_docs/"))
    .map(classifyChangedPath);
  const profileSet = new Set(reasons.flatMap((entry) => entry.profiles));

  return {
    changedPaths: normalizedPaths,
    profiles: PROFILE_ORDER.filter((profile) => profileSet.has(profile)),
    requiresFullCi: reasons.some((entry) => entry.requiresFullCi),
    reasons,
  };
}

function classifyChangedPath(changedPath: string): AffectedCheckReason {
  if (
    changedPath.startsWith("packages/check-orchestrator/") ||
    changedPath.startsWith("test/unit/check-orchestrator/")
  ) {
    return makeAffectedReason(
      changedPath,
      ["orchestrator"],
      "check-orchestrator implementation changed",
    );
  }

  if (
    changedPath === "package.json" ||
    changedPath === "pnpm-lock.yaml" ||
    changedPath === "pnpm-workspace.yaml" ||
    changedPath.startsWith(".github/actions/") ||
    changedPath.startsWith(".github/workflows/")
  ) {
    return makeAffectedReason(
      changedPath,
      ["orchestrator"],
      "workspace or workflow topology changed",
      true,
    );
  }

  if (
    changedPath.startsWith("rust/crates/omena-cli/") ||
    CLI_SUPPORT_PATHS.some((prefix) => changedPath.startsWith(prefix))
  ) {
    return makeAffectedReason(changedPath, ["rust-cli"], "Rust CLI product path changed");
  }

  if (PERFORMANCE_PATH_MARKERS.some((marker) => changedPath.startsWith(marker))) {
    return makeAffectedReason(
      changedPath,
      ["rust-workspace", "linux-benchmark"],
      "performance-sensitive Rust path changed",
    );
  }

  if (changedPath.startsWith("rust/")) {
    return makeAffectedReason(changedPath, ["rust-workspace"], "Rust workspace path changed");
  }

  if (
    changedPath.startsWith("client/") ||
    changedPath.startsWith("server/") ||
    changedPath.startsWith("shared/") ||
    changedPath.startsWith("test/") ||
    changedPath.startsWith("examples/") ||
    changedPath.startsWith("scripts/") ||
    changedPath.endsWith(".ts") ||
    changedPath.endsWith(".tsx") ||
    changedPath.endsWith(".js") ||
    changedPath.endsWith(".mjs") ||
    changedPath.endsWith(".cjs")
  ) {
    return makeAffectedReason(
      changedPath,
      ["verify"],
      "TypeScript or product integration path changed",
    );
  }

  if (
    changedPath.startsWith("docs/") ||
    changedPath === "README.md" ||
    changedPath === "CHANGELOG.md" ||
    changedPath.endsWith(".md")
  ) {
    return makeAffectedReason(changedPath, ["orchestrator"], "public documentation changed");
  }

  return makeAffectedReason(
    changedPath,
    [],
    "unclassified path requires the complete CI graph",
    true,
  );
}

function makeAffectedReason(
  path: string,
  profiles: readonly CiProbeProfileId[],
  description: string,
  requiresFullCi = false,
): AffectedCheckReason {
  return { path, profiles, reason: description, requiresFullCi };
}

function normalizePath(value: string): string {
  return value.trim().replaceAll("\\", "/").replace(/^\.\//, "");
}

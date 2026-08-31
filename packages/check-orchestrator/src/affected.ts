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

export interface AffectedPathRuleMatcherV0 {
  readonly exactPaths?: readonly string[];
  readonly prefixes?: readonly string[];
  readonly suffixes?: readonly string[];
  readonly fallback?: true;
}

export interface AffectedPathRuleDeclarationV0 {
  readonly priority: number;
  readonly ruleId: string;
  readonly ownerModulePath: string;
  readonly profiles: readonly CiProbeProfileId[];
  readonly reason: string;
  readonly requiresFullCi: boolean;
  readonly matcher: AffectedPathRuleMatcherV0;
}

const PROFILE_ORDER: readonly CiProbeProfileId[] = [
  "orchestrator",
  "rust-cli",
  "cross-platform-cli",
  "rust-workspace",
  "linux-benchmark",
  "verify",
];

export const AFFECTED_PATH_RULE_MODULE_PATH = "packages/check-orchestrator/src/affected.ts";

export const AFFECTED_PATH_RULES = [
  {
    priority: 0,
    ruleId: "orchestrator-implementation",
    ownerModulePath: AFFECTED_PATH_RULE_MODULE_PATH,
    profiles: ["orchestrator"],
    reason: "check-orchestrator implementation changed",
    requiresFullCi: false,
    matcher: { prefixes: ["packages/check-orchestrator/", "test/unit/check-orchestrator/"] },
  },
  {
    priority: 1,
    ruleId: "workspace-workflow-topology",
    ownerModulePath: AFFECTED_PATH_RULE_MODULE_PATH,
    profiles: ["orchestrator"],
    reason: "workspace or workflow topology changed",
    requiresFullCi: true,
    matcher: {
      exactPaths: ["package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml"],
      prefixes: [".github/actions/", ".github/workflows/"],
    },
  },
  {
    priority: 2,
    ruleId: "rust-cli-product",
    ownerModulePath: AFFECTED_PATH_RULE_MODULE_PATH,
    profiles: ["rust-cli"],
    reason: "Rust CLI product path changed",
    requiresFullCi: false,
    matcher: {
      prefixes: [
        "rust/crates/omena-cli/",
        "scripts/check-rust-omena-cli-",
        ".github/workflows/release-cli.yml",
      ],
    },
  },
  {
    priority: 3,
    ruleId: "rust-performance",
    ownerModulePath: AFFECTED_PATH_RULE_MODULE_PATH,
    profiles: ["rust-workspace", "linux-benchmark"],
    reason: "performance-sensitive Rust path changed",
    requiresFullCi: false,
    matcher: {
      prefixes: [
        "rust/crates/omena-benchmarks/",
        "rust/crates/omena-streaming-ifds/",
        "scripts/check-rust-z5-perf-",
        "scripts/check-rust-demand-sliced-monotone-fact-propagation-",
        "scripts/check-rust-benchmark-",
        "benchmark-artifacts/",
      ],
    },
  },
  {
    priority: 4,
    ruleId: "rust-workspace",
    ownerModulePath: AFFECTED_PATH_RULE_MODULE_PATH,
    profiles: ["rust-workspace"],
    reason: "Rust workspace path changed",
    requiresFullCi: false,
    matcher: { prefixes: ["rust/"] },
  },
  {
    priority: 5,
    ruleId: "typescript-product-integration",
    ownerModulePath: AFFECTED_PATH_RULE_MODULE_PATH,
    profiles: ["verify"],
    reason: "TypeScript or product integration path changed",
    requiresFullCi: false,
    matcher: {
      prefixes: ["client/", "server/", "shared/", "test/", "examples/", "scripts/"],
      suffixes: [".ts", ".tsx", ".js", ".mjs", ".cjs"],
    },
  },
  {
    priority: 6,
    ruleId: "public-documentation",
    ownerModulePath: AFFECTED_PATH_RULE_MODULE_PATH,
    profiles: ["orchestrator"],
    reason: "public documentation changed",
    requiresFullCi: false,
    matcher: {
      exactPaths: ["README.md", "CHANGELOG.md"],
      prefixes: ["docs/", "apps/docs/"],
      suffixes: [".md"],
    },
  },
  {
    priority: 7,
    ruleId: "unclassified-full-ci",
    ownerModulePath: AFFECTED_PATH_RULE_MODULE_PATH,
    profiles: [],
    reason: "unclassified path requires the complete CI graph",
    requiresFullCi: true,
    matcher: { fallback: true },
  },
] as const satisfies readonly AffectedPathRuleDeclarationV0[];

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
  const rule = AFFECTED_PATH_RULES.find((candidate) => affectedRuleMatches(candidate, changedPath));
  if (!rule) throw new Error(`affected path has no owned classification: ${changedPath}`);
  return makeAffectedReason(changedPath, rule.profiles, rule.reason, rule.requiresFullCi);
}

function affectedRuleMatches(rule: AffectedPathRuleDeclarationV0, changedPath: string): boolean {
  const { matcher } = rule;
  if (matcher.exactPaths?.includes(changedPath)) return true;
  if (matcher.prefixes?.some((prefix) => changedPath.startsWith(prefix))) return true;
  if (matcher.suffixes?.some((suffix) => changedPath.endsWith(suffix))) return true;
  return matcher.fallback === true;
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

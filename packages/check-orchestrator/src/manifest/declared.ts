import type {
  CheckGate,
  CheckCiTier,
  CheckDiagnostic,
  CheckTargetRef,
  DeclaredCheckDepV0,
  DeclaredCheckGateV0,
} from "./types";

const VALID_CI_TIERS = new Set<CheckCiTier>([
  "verify",
  "closure-fast",
  "rust-workspace",
  "package",
  "protocol",
  "native",
  "plugin",
  "extension-host",
  "release",
  "scheduled",
  "manual",
  "none",
]);

export const DECLARED_CHECK_GATES = [
  {
    id: "tooling/ci-probe/orchestrator-tests",
    kind: "command",
    scope: "tooling",
    command: ["pnpm", "exec", "vitest", "run", "test/unit/check-orchestrator"],
    tags: ["ci-probe", "tooling"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "Focused check-orchestrator validation is dispatched explicitly during development.",
  },
  {
    id: "tooling/ci/probe",
    kind: "command",
    scope: "tooling",
    packageTarget: "tooling/ci/probe",
    tags: ["ci-probe"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "Remote probe dispatch is an explicit local development operation.",
  },
  {
    id: "tooling/ci-probe/orchestrator",
    kind: "bundle",
    scope: "tooling",
    deps: [
      "tooling/ci-probe/orchestrator-tests",
      "tooling/orchestrator-doctor",
      "tooling/orchestrator-inventory",
      "core/check",
    ],
    tags: ["ci-probe", "tooling"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "Focused check-orchestrator validation is dispatched explicitly during development.",
  },
  {
    id: "rust/ci-probe/omena-cli-tests",
    kind: "command",
    scope: "rust",
    command: ["cargo", "test", "--manifest-path", "rust/Cargo.toml", "-p", "omena-cli"],
    tags: ["ci-probe", "omena-cli"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "Focused CLI validation is dispatched explicitly during development.",
  },
  {
    id: "rust/ci-probe/omena-cli",
    kind: "bundle",
    scope: "rust",
    deps: [
      "rust/ci-probe/omena-cli-tests",
      "rust/omena-cli-engine-contract",
      "rust/omena-cli-trace",
      "rust/omena-cli-bundle-origin-chain",
      "rust/omena-cli-soundiness-report",
      "rust/omena-cli-resolution-policy",
      "rust/omena-cli-sass-module-conformance",
    ],
    tags: ["ci-probe", "omena-cli"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "Focused CLI validation is dispatched explicitly during development.",
  },
  {
    id: "rust/ci-probe/closure-diff",
    kind: "bundle",
    scope: "rust",
    deps: [{ target: "rust/closure-fast", args: ["--summary", "--shard=diff-test"] }],
    tags: ["ci-probe", "closure-fast", "diff-test"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "The expensive differential shard is dispatched only when its evidence is needed.",
  },
  {
    id: "rust/ci-probe/workspace",
    kind: "bundle",
    scope: "rust",
    deps: ["rust/workspace"],
    tags: ["ci-probe", "rust-workspace"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "Focused workspace validation is dispatched explicitly during development.",
  },
  {
    id: "rust/ci-probe/linux-benchmark",
    kind: "bundle",
    scope: "rust",
    deps: [
      "rust/benchmark/ci-reachability",
      "rust/benchmark/emitted-css-golden-gate",
      "rust/benchmark/headline-axis",
      "rust/benchmark/transform-relex-baseline",
      "rust/z5-parser-product-cutover",
      "rust/z5-perf-baseline",
      "rust/z5-perf-per-file-invariant",
      "rust/z5-perf-complexity-slope",
      "rust/streaming-ifds-relocation-gate-bound",
      "rust/z5-perf-warmup-wave-count",
      "rust/z5-perf-no-regression",
      "rust/bundler-productization-benchmark",
    ],
    tags: ["ci-probe", "benchmark", "linux"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "Linux performance evidence is dispatched explicitly before the final CI run.",
  },
  {
    id: "tooling/ci-probe/verify",
    kind: "bundle",
    scope: "tooling",
    deps: ["core/check", "test/test", "core/build"],
    tags: ["ci-probe", "verify"],
    ciTier: "manual",
    ciGroup: "ci-probe",
    ciReason: "Focused product verification is dispatched explicitly during development.",
  },
  {
    id: "release/sync-server-version",
    kind: "command",
    scope: "release",
    command: ["./scripts/release.sh"],
    tags: ["release"],
    ciTier: "manual",
    ciGroup: "release",
    ciReason: "Release metadata synchronization is a local/manual preparation step.",
  },
  {
    id: "release/release/verify",
    kind: "bundle",
    scope: "release",
    replacesPackageTarget: "release/release/verify",
    deps: [
      "release/sync-server-version",
      "release/check/release-m5-api-freeze-audit",
      "core/build",
      "core/check",
      "plugin/consumer-example",
      "plugin/consumers",
      "rust/release/bundle",
      "tsgo/release/bundle",
      "test/test",
      "release/package",
    ],
    tags: ["release"],
    ciTier: "manual",
    ciGroup: "release",
    ciReason: "Full release verification is intentionally invoked manually before publishing.",
  },
  {
    id: "rust/release/bundle",
    kind: "bundle",
    scope: "rust",
    replacesPackageTarget: "rust/release/bundle",
    deps: [
      "rust/workspace",
      "rust/omena-syntax/boundary",
      "rust/omena-interner/boundary",
      "rust/omena-parser/boundary",
      "rust/omena-testkit/boundary",
      "rust/omena-abstract-value/domain",
      "rust/omena-abstract-value/incremental-flow",
      "rust/omena-abstract-value/one-cfa",
      "rust/omena-incremental/boundary",
      "rust/omena-resolver/boundary",
      "rust/omena-sif/boundary",
      "rust/omena-sif/end-to-end",
      "rust/omena-query/boundary",
      "rust/omena-consumer-surfaces",
      "rust/omena-lsp-server/split-boundary",
      "rust/producer-boundary",
      "rust/parser/public-product",
      "rust/omena-bridge/boundary",
      "rust/omena-cascade/boundary",
      "rust/omena-bundler/boundary",
      "rust/omena-transform-cst/boundary",
      "rust/omena-transform-passes/boundary",
      "rust/omena-transform-bundle/boundary",
      "rust/omena-transform-target/boundary",
      "rust/omena-transform-print/boundary",
      "rust/omena-transform-egg/boundary",
      "rust/omena-css/fuzz-harness",
      "rust/omena-semantic-boundary",
      "rust/omena-semantic-publish-readiness",
      "rust/checker/entrance",
      "rust/theory-claim-levels",
      {
        target: "rust/gate/evidence",
        args: ["--variant", "tsgo", "--repeat", "1", "--json"],
      },
    ],
    tags: ["release"],
    ciTier: "manual",
    ciGroup: "release",
    ciReason: "Full Rust release bundle is covered by manual release verification.",
  },
  {
    id: "rust/lane/bundle",
    kind: "bundle",
    scope: "rust",
    replacesPackageTarget: "rust/lane/bundle",
    deps: [
      "rust/omena-syntax/boundary",
      "rust/omena-interner/boundary",
      "rust/omena-parser/boundary",
      "rust/omena-testkit/boundary",
      "rust/omena-abstract-value/domain",
      "rust/omena-abstract-value/incremental-flow",
      "rust/omena-abstract-value/one-cfa",
      "rust/omena-incremental/boundary",
      "rust/omena-resolver/boundary",
      "rust/omena-sif/boundary",
      "rust/omena-query/boundary",
      "rust/producer-boundary",
      "rust/parser/public-product",
      "rust/omena-bridge/boundary",
      "rust/omena-cascade/boundary",
      "rust/omena-bundler/boundary",
      "rust/omena-transform-cst/boundary",
      "rust/omena-transform-passes/boundary",
      "rust/omena-transform-bundle/boundary",
      "rust/omena-transform-target/boundary",
      "rust/omena-transform-print/boundary",
      "rust/omena-transform-egg/boundary",
      "rust/omena-css/fuzz-harness",
      "rust/omena-semantic-boundary",
      "rust/omena-semantic-publish-readiness",
      "rust/checker/entrance",
      "rust/theory-claim-levels",
    ],
    tags: ["rust", "lane"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Rust lane bundle is retained for targeted manual validation outside PR CI.",
  },
  {
    id: "rust/omena-css/h1-readiness",
    kind: "bundle",
    scope: "rust",
    replacesPackageTarget: "rust/omena-css/h1-readiness",
    deps: [
      "rust/omena-syntax/boundary",
      "rust/omena-parser/boundary",
      "rust/omena-diff-test-boundary",
      "rust/omena-testkit/boundary",
      "rust/omena-abstract-value/domain",
      "rust/omena-abstract-value/incremental-flow",
      "rust/omena-abstract-value/one-cfa",
      "rust/omena-incremental/boundary",
      "rust/omena-resolver/boundary",
      "rust/omena-bridge/boundary",
      "rust/omena-semantic-boundary",
      "rust/omena-cascade/boundary",
      "rust/omena-bundler/boundary",
      "rust/omena-transform-cst/boundary",
      "rust/omena-transform-passes/boundary",
      "rust/omena-transform-bundle/boundary",
      "rust/omena-transform-target/boundary",
      "rust/omena-transform-print/boundary",
      "rust/omena-transform-egg/boundary",
      "rust/omena-query/boundary",
      "rust/checker/entrance",
      "rust/omena-consumer-surfaces",
      "rust/omena-lsp-server/split-boundary",
      "rust/z5-performance-baseline-readiness",
      "rust/omena-css/fuzz-harness",
      "rust/omena-css/cargo-fuzz",
      "rust/omena-css/rustdoc-coverage",
    ],
    tags: ["rust", "omena-css", "readiness"],
    ciTier: "scheduled",
    ciGroup: "drift",
  },
  {
    id: "contract/engine-v2-contract-idl",
    kind: "bundle",
    scope: "contract",
    deps: [
      "contract/engine-v2-contract-idl-decisions",
      "contract/engine-v2-contract-idl-fixtures",
      "contract/engine-v2-contract-idl-generated",
      "contract/external-corpus-envelope-contract-idl-generated",
      "contract/engine-v2-contract-idl-rust-roundtrip",
      "contract/engine-v2-contract-idl-toolchain",
      "contract/engine-v2-contract-idl-ts-compat",
    ],
    tags: ["contract", "engine-v2"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "workspace/backend-typecheck-smoke",
    kind: "gate",
    scope: "workspace",
    packageTarget: "workspace/backend-typecheck-smoke",
    tags: ["workspace", "tsgo", "typecheck"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "contract/type-fact-backend-parity",
    kind: "gate",
    scope: "contract",
    packageTarget: "contract/type-fact-backend-parity",
    tags: ["contract", "tsgo", "type-fact"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "rust/streaming-ifds-relocation-gate-bound",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/streaming-ifds-relocation-gate-bound",
    tags: ["rust", "streaming-ifds", "benchmark"],
    ciTier: "verify",
    ciGroup: "benchmark-gates",
  },
  {
    id: "ts7/ts-api-surface-lock",
    kind: "gate",
    scope: "ts7",
    packageTarget: "ts7/ts-api-surface-lock",
    tags: ["ts7", "surface-lock"],
    ciTier: "package",
    ciGroup: "package",
  },
  {
    id: "rust/closure-fast",
    kind: "bundle",
    scope: "rust",
    deps: [
      "rust/runtime-query-api-hardening",
      "rust/product-facing-capability",
      "rust/theory-generalization",
      "rust/omena-query/boundary",
      "rust/omena-lsp-server/boundary",
      "rust/omena-cascade/boundary",
      "rust/omena-diff-test-boundary",
      "rust/publish-train-closure",
      "rust/inter-crate-pin",
      "rust/role-boundaries",
      "rust/layer-dependency-exceptions",
      "rust/product-path-matrix",
      "rust/core-layer-hygiene",
      "rust/omena-debt-clock",
      "rust/cst-typed-egress-closure",
      "rust/evidence-graph-single-authority",
      "rust/obligation-family-closure",
      "rust/precision-floor",
      "rust/transform-decision-census",
      "rust/transform-rollback",
      "rust/transform-source-map-integrity",
      "rust/source-precision-ratchet",
      "rust/source-frontend/cross-language",
      "rust/source-frontend/parity-ledger",
      "rust/feature-resolved-product-reachability",
      "rust/product-lab-severance",
      "rust/cross-file-reachability-diagnostic",
      "rust/streaming-ifds-solver-hygiene",
      "rust/streaming-ifds-relocation-gate",
      "rust/streaming-ifds-settle-soak",
      "rust/publish-flags",
      "rust/naming-consistency",
      "rust/no-split-repo-residue",
      "rust/two-tier-identity-contract",
      "rust/discharge-ledger",
      "rust/semantic/preservation-model-conformance",
      "rust/translation-validation-kill-rate",
      "rust/verification-plane-closure",
      "rust/oss-corpus-farm-determinism",
      "rust/oss-corpus-farm-regressions",
      "rust/omena-ffi-boundary-typing-census",
      "rust/omena-sdk-error-mapping-census",
      "rust/omena-cross-surface-parity",
      "rust/omena-response-surface-split-census",
      "rust/omena-cli-verb-census",
      "rust/omena-config-schema-census",
      "release/check/release-tag-grammar",
      "rust/closure-fast-aggregation-complete",
    ],
    tags: ["closure-fast", "ci-unreachable-allowed"],
    ciTier: "none",
    ciGroup: "closure-fast",
    ciReason:
      "Aggregator-only bundle: CI invokes its members directly and enforces them as a grouped job.",
  },
  {
    id: "rust/runtime-query-api-hardening",
    kind: "alias",
    scope: "rust",
    deps: ["rust/m1-runtime-query-api-hardening"],
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
    deprecatedAliases: [
      "rust/m1-runtime-query-api-hardening",
      "check:rust-m1-runtime-query-api-hardening",
    ],
  },
  {
    id: "rust/product-facing-capability",
    kind: "alias",
    scope: "rust",
    deps: ["rust/m2-product-facing-capability"],
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
    deprecatedAliases: [
      "rust/m2-product-facing-capability",
      "check:rust-m2-product-facing-capability",
    ],
  },
  {
    id: "rust/theory-generalization",
    kind: "alias",
    scope: "rust",
    deps: ["rust/m3-theoretical-moat-generalization"],
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
    deprecatedAliases: [
      "rust/m3-theoretical-moat-generalization",
      "check:rust-m3-theoretical-moat-generalization",
    ],
  },
  declaredClosurePackageGate("rust/omena-query/boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-lsp-server/boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-cascade/boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-diff-test-boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/publish-train-closure", "gate", "rust"),
  declaredClosurePackageGate("rust/inter-crate-pin", "gate", "rust"),
  declaredClosurePackageGate("rust/role-boundaries", "gate", "rust"),
  declaredClosurePackageGate("rust/layer-dependency-exceptions", "gate", "rust"),
  declaredClosurePackageGate("rust/product-path-matrix", "gate", "rust"),
  declaredClosurePackageGate("rust/core-layer-hygiene", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-debt-clock", "gate", "rust"),
  declaredClosurePackageGate("rust/cst-typed-egress-closure", "gate", "rust"),
  declaredClosurePackageGate("rust/evidence-graph-single-authority", "gate", "rust"),
  declaredClosurePackageGate("rust/obligation-family-closure", "gate", "rust"),
  declaredClosurePackageGate("rust/precision-floor", "gate", "rust"),
  declaredClosurePackageGate("rust/transform-decision-census", "gate", "rust"),
  declaredClosurePackageGate("rust/transform-rollback", "gate", "rust"),
  declaredClosurePackageGate("rust/transform-source-map-integrity", "gate", "rust"),
  declaredClosurePackageGate("rust/source-precision-ratchet", "gate", "rust"),
  declaredClosurePackageGate("rust/source-frontend/cross-language", "gate", "rust"),
  declaredClosurePackageGate("rust/source-frontend/parity-ledger", "gate", "rust"),
  declaredClosurePackageGate("rust/feature-resolved-product-reachability", "gate", "rust"),
  declaredClosurePackageGate("rust/product-lab-closure", "gate", "rust"),
  declaredClosurePackageGate("rust/product-lab-severance", "gate", "rust"),
  declaredClosurePackageGate("rust/cross-file-reachability-diagnostic", "gate", "rust"),
  declaredClosurePackageGate("rust/streaming-ifds-solver-hygiene", "gate", "rust"),
  declaredClosurePackageGate("rust/streaming-ifds-relocation-gate", "gate", "rust"),
  declaredClosurePackageGate("rust/streaming-ifds-settle-soak", "gate", "rust"),
  declaredClosurePackageGate("rust/publish-flags", "gate", "rust"),
  declaredClosurePackageGate("rust/naming-consistency", "gate", "rust"),
  declaredClosurePackageGate("rust/no-split-repo-residue", "gate", "rust"),
  declaredClosurePackageGate("rust/two-tier-identity-contract", "gate", "rust"),
  declaredClosurePackageGate("rust/discharge-ledger", "gate", "rust"),
  declaredClosurePackageGate("rust/semantic/preservation-model-conformance", "gate", "rust"),
  declaredClosurePackageGate("rust/translation-validation-kill-rate", "gate", "rust"),
  declaredClosurePackageGate("rust/verification-plane-closure", "gate", "rust"),
  declaredClosurePackageGate("rust/oss-corpus-farm-determinism", "gate", "rust"),
  declaredClosurePackageGate("rust/oss-corpus-farm-regressions", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-ffi-boundary-typing-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-sdk-error-mapping-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-cross-surface-parity", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-response-surface-split-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-cli-verb-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-config-schema-census", "gate", "rust"),
  declaredClosurePackageGate("release/check/release-tag-grammar", "gate", "release"),
  declaredClosurePackageGate("rust/closure-fast-aggregation-complete", "gate", "rust"),
  {
    id: "rust/omena-cross-surface-parity-full",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-cross-surface-parity-full",
    tags: ["rust", "cross-surface-parity", "full"],
    ciTier: "scheduled",
    ciGroup: "drift",
  },
  {
    id: "rust/oss-corpus-farm",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/oss-corpus-farm",
    tags: ["rust", "oss-corpus-farm"],
    ciTier: "scheduled",
    ciGroup: "drift",
  },
  {
    id: "rust/oss-corpus-farm:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/oss-corpus-farm:update",
    tags: ["rust", "oss-corpus-farm", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Baseline regeneration changes committed data and is invoked manually with review.",
  },
  {
    id: "rust/omena-diff-test-sass-spec-upstream-scale:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-diff-test-sass-spec-upstream-scale:update",
    tags: ["rust", "omena-diff-test", "sass-spec", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason:
      "Upstream corpus count refresh changes committed data and is invoked manually with review.",
  },
  // rfcs#60: the per-PR rust-workspace strict clippy/fmt job (the rfcs#56 gate) gets an
  // explicit ci tier so the reachability check fails loudly if the ci.yml job that runs
  // `pnpm omena-check run rust/workspace` is ever deleted or stops invoking it.
  {
    id: "rust/workspace",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/workspace",
    tags: ["rust-workspace"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-transform-target/boundary",
    kind: "bundle",
    scope: "rust",
    packageTarget: "rust/omena-transform-target/boundary",
    tags: ["rust-workspace", "transform-target"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/product-test-execution",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/product-test-execution",
    tags: ["rust-workspace", "test-execution"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-bundler/public-surface",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-bundler/public-surface",
    tags: ["rust-workspace", "public-api", "bundler"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-query/public-surface",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-query/public-surface",
    tags: ["closure-fast", "public-api", "query"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  },
  {
    id: "rust/omena-bundler/adapter-pass-authority",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-bundler/adapter-pass-authority",
    tags: ["rust-workspace", "bundler", "ffi"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-bundler/closed-world-authority",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-bundler/closed-world-authority",
    tags: ["rust-workspace", "bundler", "closed-world"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-transform-passes/structural-ir-shadow",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-transform-passes/structural-ir-shadow",
    tags: ["rust-workspace", "transform-passes", "structural-ir"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-cascade/property-metadata",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-cascade/property-metadata",
    tags: ["rust-workspace", "cascade", "property-metadata"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-transform-cst/observation-census",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-transform-cst/observation-census",
    tags: ["rust-workspace", "transform-cst", "observation-contract"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-bundler/public-surface:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-bundler/public-surface:update",
    tags: ["public-api", "bundler"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Snapshot refresh command is invoked deliberately when accepting public API drift.",
  },
  {
    id: "rust/omena-query/public-surface:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-query/public-surface:update",
    tags: ["public-api", "query"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Snapshot refresh command is invoked deliberately when accepting public API drift.",
  },
  {
    id: "rust/product-test-coverage-classguard",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/product-test-coverage-classguard",
    tags: ["rust-workspace", "test-execution"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
] satisfies readonly DeclaredCheckGateV0[];

const LEGACY_PACKAGE_SCRIPT_REPLACEMENTS = new Map(
  DECLARED_CHECK_GATES.flatMap((gate) =>
    (gate.deprecatedAliases ?? [])
      .filter((alias) => alias.startsWith("check:"))
      .map((alias) => [alias, gate.id] as const),
  ),
);

export function getDeprecatedPackageScriptReplacement(scriptName: string): string | undefined {
  return LEGACY_PACKAGE_SCRIPT_REPLACEMENTS.get(scriptName);
}

function declaredClosurePackageGate(
  id: string,
  kind: DeclaredCheckGateV0["kind"],
  scope: DeclaredCheckGateV0["scope"],
): DeclaredCheckGateV0 {
  return {
    id,
    kind,
    scope,
    packageTarget: id,
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  };
}

export function applyDeclaredPackageMetadata(
  packageGates: readonly CheckGate[],
  declarations: readonly DeclaredCheckGateV0[],
  diagnostics: CheckDiagnostic[],
): readonly CheckGate[] {
  const byScriptName = new Map(packageGates.map((gate) => [gate.scriptName, gate]));

  for (const declaration of declarations) {
    if (!declaration.packageTarget) {
      continue;
    }

    validateDeclaredShape(declaration, diagnostics);
    const packageGate = resolveDeclaredDependency(packageGates, declaration.packageTarget);
    if (!packageGate) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-target-unknown",
        message: `Declared package metadata "${declaration.id}" references unknown package target "${declaration.packageTarget}".`,
      });
      continue;
    }

    if (packageGate.id !== declaration.id) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-target-id-mismatch",
        message: `Declared package metadata "${declaration.id}" points to package gate "${packageGate.id}".`,
      });
      continue;
    }

    byScriptName.set(packageGate.scriptName, mergeDeclaredMetadata(packageGate, declaration));
  }

  return packageGates.map((gate) => byScriptName.get(gate.scriptName) ?? gate);
}

export function findDeclaredPackageReplacementIds(
  packageGates: readonly CheckGate[],
  declarations: readonly DeclaredCheckGateV0[],
): ReadonlySet<string> {
  const replacementIds = new Set<string>();
  for (const declaration of declarations) {
    if (!declaration.replacesPackageTarget) {
      continue;
    }
    const packageGate = resolveDeclaredDependency(packageGates, declaration.replacesPackageTarget);
    if (packageGate?.id === declaration.id) {
      replacementIds.add(packageGate.id);
    }
  }
  return replacementIds;
}

export function buildDeclaredGates(
  packageGates: readonly CheckGate[],
  declarations: readonly DeclaredCheckGateV0[],
  diagnostics: CheckDiagnostic[],
): readonly CheckGate[] {
  const duplicateDeclaredIds = findDuplicateValues(declarations.map((gate) => gate.id));
  for (const id of duplicateDeclaredIds) {
    diagnostics.push({
      severity: "error",
      code: "duplicate-declared-gate-id",
      message: `Declared gate id "${id}" is defined more than once.`,
    });
  }

  const packageGateIds = new Set(packageGates.map((gate) => gate.id));
  const replacementIds = findDeclaredPackageReplacementIds(packageGates, declarations);
  for (const declaration of declarations) {
    if (declaration.packageTarget) {
      continue;
    }
    if (packageGateIds.has(declaration.id) && !replacementIds.has(declaration.id)) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-gate-id-collision",
        message: `Declared gate id "${declaration.id}" collides with a package-derived gate id.`,
      });
    }
  }

  const executableDeclarations = declarations.filter((declaration) => !declaration.packageTarget);
  const declaredGates = executableDeclarations.map((declaration) =>
    buildDeclaredGate(declaration, packageGates, diagnostics),
  );
  const allGates = [...packageGates, ...declaredGates];

  diagnostics.push(
    ...findDeclaredPackageReplacementDiagnostics(executableDeclarations, packageGates),
  );
  diagnostics.push(...findDeclaredDependencyDiagnostics(executableDeclarations, allGates));
  diagnostics.push(...findDeclaredCycleDiagnostics(executableDeclarations));

  return declaredGates.map((gate) =>
    Object.assign({}, gate, {
      referencedScripts: (gate.referencedTargetSpecs ?? [])
        .map(({ target }) => resolveDeclaredDependency(allGates, target)?.scriptName)
        .filter((scriptName): scriptName is string => Boolean(scriptName)),
    }),
  );
}

function buildDeclaredGate(
  declaration: DeclaredCheckGateV0,
  packageGates: readonly CheckGate[],
  diagnostics: CheckDiagnostic[],
): CheckGate {
  validateDeclaredShape(declaration, diagnostics);
  const targetSpecs = normalizeDeclaredDeps(declaration.deps ?? []);
  const replacedPackageGate = declaration.replacesPackageTarget
    ? resolveDeclaredDependency(packageGates, declaration.replacesPackageTarget)
    : null;

  return {
    id: declaration.id,
    scriptName: replacedPackageGate?.scriptName ?? `@declared/${declaration.id}`,
    command:
      declaration.command?.join(" ") ??
      targetSpecs.map((targetSpec) => targetSpec.target).join(" && ") ??
      "",
    scope: declaration.scope,
    kind: declaration.kind,
    origin: "declared",
    referencedTargets: targetSpecs.map((targetSpec) => targetSpec.target),
    referencedTargetSpecs: targetSpecs,
    referencedScripts: [],
    ...(declaration.command ? { commandParts: declaration.command } : {}),
    ...(declaration.tags ? { tags: declaration.tags } : {}),
    ...(declaration.timeoutMinutes !== undefined
      ? { timeoutMinutes: declaration.timeoutMinutes }
      : {}),
    ...(declaration.ciTier ? { ciTier: declaration.ciTier } : {}),
    ...(declaration.ciGroup ? { ciGroup: declaration.ciGroup } : {}),
    ...(declaration.ciReason ? { ciReason: declaration.ciReason } : {}),
    ...(declaration.deprecatedAliases ? { deprecatedAliases: declaration.deprecatedAliases } : {}),
  };
}

function validateDeclaredShape(
  declaration: DeclaredCheckGateV0,
  diagnostics: CheckDiagnostic[],
): void {
  const hasCommand = (declaration.command?.length ?? 0) > 0;
  const depCount = declaration.deps?.length ?? 0;

  if (declaration.packageTarget && declaration.replacesPackageTarget) {
    diagnostics.push({
      severity: "error",
      code: "declared-package-target-conflict",
      message: `Declared gate "${declaration.id}" cannot set both packageTarget and replacesPackageTarget.`,
    });
  }

  if (declaration.packageTarget) {
    if (hasCommand || depCount > 0) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-metadata-has-executable",
        message: `Declared package metadata "${declaration.id}" must not define command or deps.`,
      });
    }
    if (declaration.ciTier && !VALID_CI_TIERS.has(declaration.ciTier)) {
      diagnostics.push({
        severity: "error",
        code: "declared-gate-unknown-ci-tier",
        message: `Declared gate "${declaration.id}" uses unknown ciTier "${declaration.ciTier}".`,
      });
    }
    return;
  }

  if ((declaration.kind === "command" || declaration.kind === "gate") && !hasCommand) {
    diagnostics.push({
      severity: "error",
      code: "declared-gate-missing-command",
      message: `Declared ${declaration.kind} "${declaration.id}" must define command parts.`,
    });
  }

  if (declaration.kind === "bundle" && depCount === 0) {
    diagnostics.push({
      severity: "error",
      code: "declared-bundle-missing-deps",
      message: `Declared bundle "${declaration.id}" must define deps.`,
    });
  }

  if (declaration.kind === "alias" && depCount !== 1) {
    diagnostics.push({
      severity: "error",
      code: "declared-alias-invalid-deps",
      message: `Declared alias "${declaration.id}" must point to exactly one dep.`,
    });
  }

  if (declaration.ciTier && !VALID_CI_TIERS.has(declaration.ciTier)) {
    diagnostics.push({
      severity: "error",
      code: "declared-gate-unknown-ci-tier",
      message: `Declared gate "${declaration.id}" uses unknown ciTier "${declaration.ciTier}".`,
    });
  }
}

function findDeclaredPackageReplacementDiagnostics(
  declarations: readonly DeclaredCheckGateV0[],
  packageGates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  for (const declaration of declarations) {
    if (!declaration.replacesPackageTarget) {
      continue;
    }

    const packageGate = resolveDeclaredDependency(packageGates, declaration.replacesPackageTarget);
    if (!packageGate) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-replacement-target-unknown",
        message: `Declared gate "${declaration.id}" replaces unknown package target "${declaration.replacesPackageTarget}".`,
      });
      continue;
    }

    if (packageGate.id !== declaration.id) {
      diagnostics.push({
        severity: "error",
        code: "declared-package-replacement-id-mismatch",
        message: `Declared gate "${declaration.id}" replaces package gate "${packageGate.id}".`,
      });
    }
  }
  return diagnostics;
}

function mergeDeclaredMetadata(gate: CheckGate, declaration: DeclaredCheckGateV0): CheckGate {
  return {
    ...gate,
    origin: "package+declared",
    ...(declaration.tags ? { tags: mergeUnique(gate.tags ?? [], declaration.tags) } : {}),
    ...(declaration.timeoutMinutes !== undefined
      ? { timeoutMinutes: declaration.timeoutMinutes }
      : {}),
    ...(declaration.ciTier ? { ciTier: declaration.ciTier } : {}),
    ...(declaration.ciGroup ? { ciGroup: declaration.ciGroup } : {}),
    ...(declaration.ciReason ? { ciReason: declaration.ciReason } : {}),
    ...(declaration.deprecatedAliases
      ? {
          deprecatedAliases: mergeUnique(
            gate.deprecatedAliases ?? [],
            declaration.deprecatedAliases,
          ),
        }
      : {}),
  };
}

function mergeUnique(left: readonly string[], right: readonly string[]): readonly string[] {
  return [...new Set([...left, ...right])];
}

function findDeclaredDependencyDiagnostics(
  declarations: readonly DeclaredCheckGateV0[],
  gates: readonly CheckGate[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  for (const declaration of declarations) {
    for (const dep of normalizeDeclaredDeps(declaration.deps ?? [])) {
      if (!resolveDeclaredDependency(gates, dep.target)) {
        diagnostics.push({
          severity: "error",
          code: "declared-gate-unknown-dep",
          message: `Declared gate "${declaration.id}" references unknown dep "${dep.target}".`,
        });
      }
    }
  }
  return diagnostics;
}

function findDeclaredCycleDiagnostics(
  declarations: readonly DeclaredCheckGateV0[],
): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const byId = new Map(declarations.map((gate) => [gate.id, gate]));
  const visiting = new Set<string>();
  const visited = new Set<string>();

  for (const declaration of declarations) {
    visit(declaration, []);
  }

  return diagnostics;

  function visit(declaration: DeclaredCheckGateV0, path: readonly string[]): void {
    if (visited.has(declaration.id)) return;
    if (visiting.has(declaration.id)) {
      diagnostics.push({
        severity: "error",
        code: "declared-gate-cycle",
        message: `Declared gate cycle detected: ${[...path, declaration.id].join(" -> ")}`,
      });
      return;
    }

    visiting.add(declaration.id);
    for (const dep of normalizeDeclaredDeps(declaration.deps ?? [])) {
      const depDeclaration = byId.get(dep.target);
      if (depDeclaration) {
        visit(depDeclaration, [...path, declaration.id]);
      }
    }
    visiting.delete(declaration.id);
    visited.add(declaration.id);
  }
}

function resolveDeclaredDependency(gates: readonly CheckGate[], target: string): CheckGate | null {
  return (
    gates.find((gate) => gate.id === target || gate.scriptName === target) ??
    gates.find((gate) => gate.deprecatedAliases?.includes(target)) ??
    gates.find((gate) => gate.id.endsWith(`/${target}`)) ??
    null
  );
}

function normalizeDeclaredDeps(deps: readonly DeclaredCheckDepV0[]): readonly CheckTargetRef[] {
  return deps.map((dep) => (typeof dep === "string" ? { target: dep } : dep));
}

function findDuplicateValues(values: readonly string[]): readonly string[] {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) {
      duplicates.add(value);
    }
    seen.add(value);
  }
  return [...duplicates].toSorted();
}

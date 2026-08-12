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
    id: "docs/site-contracts",
    kind: "gate",
    scope: "docs",
    packageTarget: "docs/site-contracts",
    tags: ["docs", "metadata", "links"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/site",
    kind: "gate",
    scope: "docs",
    packageTarget: "docs/site",
    tags: ["docs", "static-site", "typecheck"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/smoke",
    kind: "gate",
    scope: "docs",
    packageTarget: "docs/smoke",
    tags: ["docs", "static-site", "smoke"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/contracts",
    kind: "bundle",
    scope: "docs",
    deps: [
      "docs/site-contracts",
      "docs/reference-surface",
      "docs/readme-example",
      "docs/version-strings",
      "docs/version-governance",
      "docs/publication-material",
      "release/check/release-notes",
      "rust/crate-documentation",
    ],
    tags: ["docs", "contract"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/readme-example",
    kind: "gate",
    scope: "docs",
    packageTarget: "docs/readme-example",
    tags: ["docs", "product-example"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/reference-surface",
    kind: "gate",
    scope: "docs",
    packageTarget: "docs/reference-surface",
    tags: ["docs", "generated-reference"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/version-strings",
    kind: "gate",
    scope: "docs",
    packageTarget: "docs/version-strings",
    tags: ["docs", "version"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/version-governance",
    kind: "gate",
    scope: "docs",
    packageTarget: "docs/version-governance",
    tags: ["docs", "version", "release"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/publication-material",
    kind: "gate",
    scope: "docs",
    packageTarget: "docs/publication-material",
    tags: ["docs", "publication"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "release/check/release-notes",
    kind: "gate",
    scope: "release",
    packageTarget: "release/check/release-notes",
    tags: ["release", "docs", "github-release"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "release/release/notes",
    kind: "command",
    scope: "release",
    packageTarget: "release/release/notes",
    tags: ["release", "docs", "github-release"],
    ciTier: "release",
    ciGroup: "release",
  },
  {
    id: "rust/crate-documentation",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/crate-documentation",
    tags: ["rust", "docs", "crate"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "release/check/release-crate-registry-state",
    kind: "gate",
    scope: "release",
    packageTarget: "release/check/release-crate-registry-state",
    tags: ["release", "registry"],
    ciTier: "manual",
    ciGroup: "release",
    ciReason: "Live crates.io registry classification is an operator-time release preflight.",
  },
  {
    id: "rust/release-semver",
    kind: "command",
    scope: "release",
    command: ["node", "--import", "tsx", "./scripts/check-rust-release-semver.ts"],
    tags: ["release", "rust", "public-api", "publish"],
    ciTier: "release",
    ciGroup: "release",
  },
  {
    id: "rust/release-semver-intent-contract",
    kind: "gate",
    scope: "rust",
    command: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-rust-release-semver.ts",
      "--validate-intents-only",
    ],
    tags: ["verify", "rust", "public-api", "publish"],
    ciTier: "verify",
    ciGroup: "verify",
  },
  {
    id: "docs/reference-surface:update",
    kind: "command",
    scope: "docs",
    packageTarget: "docs/reference-surface:update",
    tags: ["docs", "generated-reference", "update"],
    ciTier: "manual",
    ciGroup: "docs",
    ciReason: "Reference regeneration changes committed documentation and requires review.",
  },
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
      "rust/omena-cli-migration",
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
      "rust/demand-sliced-monotone-fact-propagation-relocation-gate-bound",
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
      "release/check/release-notes",
      "docs/version-strings",
      "docs/version-governance",
      "rust/crate-documentation",
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
    id: "rust/omena-css/h1-core-semantics",
    kind: "bundle",
    scope: "rust",
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
    ],
    tags: ["rust", "omena-css", "readiness", "core"],
    ciTier: "scheduled",
    ciGroup: "drift",
  },
  {
    id: "rust/omena-css/h1-product-surfaces",
    kind: "bundle",
    scope: "rust",
    deps: [
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
    ],
    tags: ["rust", "omena-css", "readiness", "product"],
    ciTier: "scheduled",
    ciGroup: "drift",
  },
  {
    id: "rust/omena-css/h1-assurance",
    kind: "bundle",
    scope: "rust",
    deps: [
      "rust/z5-performance-baseline-readiness",
      "rust/omena-css/fuzz-harness",
      "rust/omena-css/cargo-fuzz",
      "rust/omena-css/rustdoc-coverage",
    ],
    tags: ["rust", "omena-css", "readiness", "assurance"],
    ciTier: "scheduled",
    ciGroup: "drift",
  },
  {
    id: "rust/omena-css/h1-readiness",
    kind: "bundle",
    scope: "rust",
    replacesPackageTarget: "rust/omena-css/h1-readiness",
    deps: [
      "rust/omena-css/h1-core-semantics",
      "rust/omena-css/h1-product-surfaces",
      "rust/omena-css/h1-assurance",
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
    id: "contract/parity-v2-golden",
    kind: "gate",
    scope: "contract",
    packageTarget: "contract/parity-v2-golden",
    tags: ["contract", "golden", "parity"],
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
    id: "rust/demand-sliced-monotone-fact-propagation-relocation-gate-bound",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/demand-sliced-monotone-fact-propagation-relocation-gate-bound",
    tags: ["rust", "demand-sliced-monotone-fact-propagation", "benchmark"],
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
      "rust/omena-query/core-contract",
      "rust/omena-query/transform-contract",
      "rust/omena-query/runtime-contract",
      "rust/omena-lsp-server/boundary",
      "rust/omena-cascade/boundary",
      "rust/omena-diff-test-core",
      "rust/omena-diff-test-wpt",
      "rust/omena-diff-test-sass-spec",
      "rust/omena-diff-test-extended",
      "rust/omena-bundler/linked-emission-byte-differential",
      "rust/omena-bundler/linked-emission-falsifier-disclosure",
      "rust/publish-train-closure",
      "rust/contract-shape-census",
      "rust/domain-claim-census",
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
      "rust/omena-plugin-consumption-law",
      "rust/omena-plugin-abi-stability",
      "rust/omena-tsgo-type-flags-abi",
      "rust/omena-source-type-fact-shape-census",
      "rust/omena-js-bundler-host-parity",
      "rust/omena-js-bundler-host-no-regex-classmap",
      "rust/omena-js-bundler-host-plugin-kind",
      "rust/transform-decision-census",
      "rust/transform-rollback",
      "rust/transform-source-map-integrity",
      "rust/source-precision-ratchet",
      "rust/source-frontend/cross-language",
      "rust/source-frontend/parity-ledger",
      "rust/feature-resolved-product-reachability",
      "rust/product-lab-severance",
      "rust/cross-file-reachability-diagnostic",
      "rust/demand-sliced-monotone-fact-propagation-solver-hygiene",
      "rust/demand-sliced-monotone-fact-propagation-relocation-gate",
      "rust/demand-sliced-monotone-fact-propagation-settle-soak",
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
      "rust/lint-finding-census",
      "rust/ranked-set-loss-census",
      "rust/omena-ffi-boundary-typing-census",
      "rust/omena-sdk-error-mapping-census",
      "rust/omena-cli-json-output-census",
      "rust/omena-sass-intelligence",
      "rust/omena-cross-surface-parity",
      "rust/omena-response-surface-split-census",
      "rust/omena-literal-evidence-census",
      "rust/omena-top-provenance-census",
      "rust/omena-precision-witness-census",
      "rust/omena-fact-precision-census",
      "rust/omena-cli-verb-census",
      "rust/omena-cli-persona-presets",
      "rust/product-surface-boundary-reviews",
      "rust/omena-syntax-authority-raw-scan-census",
      "rust/omena-keyword-case-authority-census",
      "rust/omena-style-resolution-authority",
      "rust/omena-alias-resolution-surfaces",
      "rust/omena-verification-targets",
      "rust/omena-cli-migration",
      "rust/omenad-substrate-inventory",
      "rust/omenad-protocol",
      "rust/omena-workspace-session-routing",
      "rust/omena-modules-surface",
      "rust/omena-lint-parity",
      "rust/omena-lint-tier-census",
      "rust/omena-stylelint-compat",
      "rust/omena-config-schema-census",
      "rust/omena-write-safety",
      "rust/omena-css/spec-boundary",
      "rust/omena-value-grammar-evidence",
      "rust/omena-value-grammar-differential",
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
    timeoutMinutes: 20,
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
    timeoutMinutes: 20,
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
    timeoutMinutes: 20,
    deprecatedAliases: [
      "rust/m3-theoretical-moat-generalization",
      "check:rust-m3-theoretical-moat-generalization",
    ],
  },
  declaredClosurePackageGate("rust/omena-query/core-contract", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-query/transform-contract", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-query/runtime-contract", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-lsp-server/boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-lsp-server/query-read-boundary", "gate", "rust"),
  {
    id: "rust/omena-lsp-server/tide-sole-authority",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-lsp-server/tide-sole-authority",
    tags: ["rust-workspace", "lsp", "tide", "reactive"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-lsp-server/reactive-shadow-parity",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-lsp-server/reactive-shadow-parity",
    tags: ["rust-workspace", "lsp", "reactive", "shadow", "parity"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  declaredClosurePackageGate("rust/omena-cascade/boundary", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-diff-test-core", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-diff-test-wpt", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-diff-test-sass-spec", "bundle", "rust"),
  declaredClosurePackageGate("rust/omena-diff-test-extended", "bundle", "rust"),
  declaredClosurePackageGate("rust/publish-train-closure", "gate", "rust"),
  declaredClosurePackageGate("rust/contract-shape-census", "gate", "rust"),
  declaredClosurePackageGate("rust/domain-claim-census", "gate", "rust"),
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
  declaredClosurePackageGate("rust/omena-plugin-consumption-law", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-plugin-abi-stability", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-tsgo-type-flags-abi", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-source-type-fact-shape-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-js-bundler-host-parity", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-js-bundler-host-no-regex-classmap", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-js-bundler-host-plugin-kind", "gate", "rust"),
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
  declaredClosurePackageGate(
    "rust/demand-sliced-monotone-fact-propagation-solver-hygiene",
    "gate",
    "rust",
  ),
  declaredClosurePackageGate(
    "rust/demand-sliced-monotone-fact-propagation-relocation-gate",
    "gate",
    "rust",
  ),
  declaredClosurePackageGate(
    "rust/demand-sliced-monotone-fact-propagation-settle-soak",
    "gate",
    "rust",
  ),
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
  declaredClosurePackageGate("rust/lint-finding-census", "gate", "rust"),
  declaredClosurePackageGate("rust/ranked-set-loss-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-ffi-boundary-typing-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-sdk-error-mapping-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-cli-json-output-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-sass-intelligence", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-cross-surface-parity", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-response-surface-split-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-literal-evidence-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-top-provenance-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-precision-witness-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-fact-precision-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-cli-verb-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-cli-persona-presets", "gate", "rust"),
  declaredClosurePackageGate("rust/product-surface-boundary-reviews", "gate", "rust"),
  {
    id: "rust/omena-syntax-authority-raw-scan-census",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-syntax-authority-raw-scan-census",
    tags: ["closure-fast"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
    ciReason:
      "Chains raw-syntax and identifier-authority censuses with the identifier firing selftest.",
    timeoutMinutes: 20,
  },
  declaredClosurePackageGate("rust/omena-keyword-case-authority-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-style-resolution-authority", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-alias-resolution-surfaces", "gate", "rust"),
  {
    id: "rust/omena-syntax-authority-raw-scan-census:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-syntax-authority-raw-scan-census:update",
    tags: ["rust", "omena-syntax", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Census regeneration changes committed governance data and requires review.",
  },
  {
    id: "rust/omena-keyword-case-authority-census:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-keyword-case-authority-census:update",
    tags: ["rust", "omena-syntax", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Census regeneration changes committed governance data and requires review.",
  },
  {
    id: "rust/omena-style-resolution-authority:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-style-resolution-authority:update",
    tags: ["rust", "omena-query", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Census regeneration changes committed governance data and requires review.",
  },
  declaredClosurePackageGate("rust/omena-verification-targets", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-cli-migration", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-modules-surface-authority", "gate", "rust"),
  {
    id: "rust/omena-modules-query-tests",
    kind: "command",
    scope: "rust",
    command: [
      "cargo",
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "css_modules_interface",
      "--",
      "--nocapture",
    ],
    tags: ["closure-fast", "css-modules"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  },
  {
    id: "rust/omena-modules-cli-tests",
    kind: "command",
    scope: "rust",
    command: [
      "cargo",
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cli",
      "css_modules_interface",
      "--",
      "--nocapture",
    ],
    tags: ["closure-fast", "css-modules", "omena-cli"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  },
  {
    id: "rust/omena-modules-surface",
    kind: "bundle",
    scope: "rust",
    deps: [
      "rust/omena-modules-surface-authority",
      "rust/omena-modules-query-tests",
      "rust/omena-modules-cli-tests",
    ],
    tags: ["closure-fast", "css-modules"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  },
  declaredClosurePackageGate("rust/omena-lint-parity", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-lint-tier-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-stylelint-compat", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-config-schema-census", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-write-safety", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-css/spec-boundary", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-value-grammar-evidence", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-value-grammar-differential", "gate", "rust"),
  {
    id: "rust/omena-value-grammar-corpus:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-value-grammar-corpus:update",
    tags: ["rust", "value-grammar", "corpus", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason:
      "Pinned real-declaration corpus regeneration changes committed data and requires review.",
  },
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
    id: "plugin/vite-plugin-hmr",
    kind: "gate",
    scope: "plugin",
    packageTarget: "plugin/vite-plugin-hmr",
    tags: ["plugin", "vite", "browser", "bundler-host"],
    ciTier: "scheduled",
    ciGroup: "nightly-soak",
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
    id: "rust/lint-finding-census:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/lint-finding-census:update",
    tags: ["rust", "oss-corpus-farm", "lint-census", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Finding adjudication and census report changes require reviewed regeneration.",
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
  // The per-PR strict clippy/fmt job gets an explicit CI tier so reachability
  // fails loudly if the workflow job that runs
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
    id: "rust/omena-reactive/contract",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-reactive/contract",
    tags: ["rust-workspace", "reactive", "shadow"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-reactive/public-surface",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-reactive/public-surface",
    tags: ["rust-workspace", "public-api", "reactive", "publish"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-reactive/performance",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-reactive/performance",
    tags: ["synthetic-smoke", "reactive", "shadow"],
    ciTier: "verify",
    ciGroup: "benchmark-gates",
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
    id: "rust/product-test-contracts",
    kind: "bundle",
    scope: "rust",
    deps: [
      "rust/omena-bundler/public-surface",
      "rust/omena-cascade/layer-rank-construction",
      "rust/published-crate-surface-register",
      "rust/omena-bundler/adapter-pass-authority",
      "rust/omena-bundler/closed-world-authority",
      "rust/omena-bundler/emission-order-contract",
      "rust/omena-bundler/emission-item-contract",
      "rust/omena-bundler/linked-emission-default-precondition",
      "rust/omena-query/linker-input-walk-authority",
      "rust/omena-query/linked-source-map-boundary",
      "rust/omena-query/bundle-execution-scope",
      "rust/omena-query/module-qualified-product-path",
      "rust/omena-query/css-module-token-integrity",
      "rust/omena-query/module-reachability-hoist",
      "rust/omena-transform-passes/lex-splice-equivalence",
      "rust/omena-transform-passes/structural-ir-shadow",
      "rust/omena-spec-audit-webref-grammar",
      "rust/omena-spec-audit-webref-drift",
      "rust/omena-coverage-gap-report",
    ],
    tags: ["rust-workspace", "test-execution", "contract"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/product-tests",
    kind: "bundle",
    scope: "rust",
    deps: [
      "rust/product-test-execution",
      "rust/product-test-contracts",
      "rust/product-test-coverage-classguard",
    ],
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
    id: "rust/published-crate-surface-register",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/published-crate-surface-register",
    tags: ["rust-workspace", "public-api", "publish"],
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
    id: "rust/omena-query/linker-input-walk-authority",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-query/linker-input-walk-authority",
    tags: ["rust-workspace", "query", "linker", "walk-authority"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  declaredClosurePackageGate("rust/omena-query/effective-pass-set", "gate", "rust"),
  declaredClosurePackageGate("rust/omena-query/strict-verification", "gate", "rust"),
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
    id: "rust/omena-bundler/emission-order-contract",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-bundler/emission-order-contract",
    tags: ["rust-workspace", "bundler", "emission-order"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-bundler/emission-item-contract",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-bundler/emission-item-contract",
    tags: ["rust-workspace", "bundler", "emission-order", "public-api"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omenad-substrate-inventory",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omenad-substrate-inventory",
    tags: ["closure-fast", "daemon", "workspace-session"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  },
  {
    id: "rust/omenad-protocol",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omenad-protocol",
    tags: ["closure-fast", "daemon", "workspace-session", "protocol"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  },
  {
    id: "rust/omena-workspace-session-routing",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-workspace-session-routing",
    tags: ["closure-fast", "daemon", "workspace-session", "routing"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
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
    id: "rust/transform-winner-equality-audit",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/transform-winner-equality-audit",
    tags: ["rust-workspace", "transform-passes", "cascade", "semantic-trust"],
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
    id: "rust/omena-cascade/layer-rank-construction",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-cascade/layer-rank-construction",
    tags: ["rust-workspace", "cascade", "public-api", "construction-census"],
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
    id: "rust/omena-transform-cst/minify-profile-manifest",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-transform-cst/minify-profile-manifest",
    tags: ["rust-workspace", "transform-cst", "minify-profile"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-cli-minify-backend",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-cli-minify-backend",
    tags: ["rust-workspace", "omena-cli", "minify-backend"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-cli-postcss-compat",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-cli-postcss-compat",
    tags: ["rust-workspace", "omena-cli", "postcss-compat", "transform-differential", "z5"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-query/transform-differential",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-query/transform-differential",
    tags: ["rust-workspace", "omena-query", "minify-differential"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-query/linked-source-map-boundary",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-query/linked-source-map-boundary",
    tags: ["rust-workspace", "omena-query", "source-map", "emission-order"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-query/bundle-execution-scope",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-query/bundle-execution-scope",
    tags: ["rust-workspace", "omena-query", "bundle", "source-map", "evidence"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-query/module-qualified-product-path",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-query/module-qualified-product-path",
    tags: ["rust-workspace", "omena-query", "closed-world", "tree-shake"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-query/css-module-token-integrity",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-query/css-module-token-integrity",
    tags: ["rust-workspace", "omena-query", "css-modules", "emitted-bytes"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-query/module-reachability-hoist",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-query/module-reachability-hoist",
    tags: ["rust-workspace", "omena-query", "closed-world", "tree-shake", "incremental"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-module-qualified-product-path-census:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-module-qualified-product-path-census:update",
    tags: ["rust", "omena-query", "closed-world", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Caller-census regeneration changes committed governance data and requires review.",
  },
  {
    id: "rust/omena-bundler/linked-emission-byte-differential",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-bundler/linked-emission-byte-differential",
    tags: ["closure-fast", "diff-test", "omena-bundler", "byte-differential"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  },
  {
    id: "rust/omena-bundler/linked-emission-falsifier-disclosure",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-bundler/linked-emission-falsifier-disclosure",
    tags: ["closure-fast", "diff-test", "omena-bundler", "governance"],
    ciTier: "closure-fast",
    ciGroup: "closure-fast",
  },
  {
    id: "rust/omena-bundler/linked-emission-default-precondition",
    kind: "gate",
    scope: "rust",
    packageTarget: "rust/omena-bundler/linked-emission-default-precondition",
    tags: ["rust-workspace", "omena-bundler", "emission-order", "release-policy"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
  },
  {
    id: "rust/omena-linked-emission-coverage-census:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-linked-emission-coverage-census:update",
    tags: ["rust", "diff-test", "linked-emission", "update"],
    ciTier: "manual",
    ciGroup: "rust",
    ciReason: "Coverage census regeneration changes committed governance data and requires review.",
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
    id: "rust/omena-reactive/public-surface:update",
    kind: "command",
    scope: "rust",
    packageTarget: "rust/omena-reactive/public-surface:update",
    tags: ["public-api", "reactive"],
    ciTier: "rust-workspace",
    ciGroup: "rust-workspace",
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
    timeoutMinutes: 20,
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
    executor: declaration.command ? "direct" : "dependencies",
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

  if (
    declaration.timeoutMinutes !== undefined &&
    (!Number.isFinite(declaration.timeoutMinutes) || declaration.timeoutMinutes <= 0)
  ) {
    diagnostics.push({
      severity: "error",
      code: "declared-gate-invalid-timeout",
      message: `Declared gate "${declaration.id}" timeoutMinutes must be greater than zero.`,
    });
  }

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

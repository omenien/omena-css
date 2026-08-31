/**
 * Commands whose output closure is not recoverable from one local static
 * write call alone. This is the reviewable authority for imported path
 * constants, computed repository roots, multi-output writers, and orchestrator
 * subcommands. The writer registry must cover every row and exact command.
 */
export const EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH =
  "packages/check-orchestrator/src/evidence/writer-command-authority.ts";

export interface EvidenceWriterCommandDeclaration {
  readonly commandId: string;
  readonly writeCommand: readonly string[];
  readonly writerScripts: readonly string[];
  readonly outputPaths: readonly string[];
  readonly inputPaths?: readonly string[];
}

export const EVIDENCE_WRITER_COMMAND_DECLARATIONS = [
  {
    commandId: "orchestrator-inventory",
    writeCommand: ["pnpm", "omena-check", "inventory", "--write"],
    writerScripts: ["packages/check-orchestrator/src/cli/main.ts"],
    outputPaths: ["packages/check-orchestrator/CHECKS.md"],
  },
  {
    commandId: "orchestrator-cost-ledger",
    writeCommand: ["pnpm", "omena-check", "cost-ledger", "--write"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/manifest/cost-ledger.ts",
    ],
    outputPaths: ["packages/check-orchestrator/ci-cost-ledger.json"],
  },
  {
    commandId: "orchestrator-ci-workflow",
    writeCommand: ["pnpm", "omena-check", "ci-workflow", "--write"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/manifest/ci-workflow.ts",
    ],
    outputPaths: [".github/workflows/ci.yml"],
  },
  {
    commandId: "orchestrator-ci-workflow-registry",
    writeCommand: ["pnpm", "omena-check", "ci-workflow", "--adopt"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/manifest/ci-workflow.ts",
    ],
    outputPaths: ["packages/check-orchestrator/ci-workflow.json"],
  },
  {
    commandId: "evidence-scan-surfaces",
    writeCommand: ["pnpm", "omena-check", "evidence-surfaces", "--write"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/evidence/scan-surface-registry.ts",
    ],
    outputPaths: ["rust/evidence-scan-surfaces.json"],
  },
  {
    commandId: "evidence-writer-registry",
    writeCommand: ["pnpm", "omena-check", "evidence-writers", "--write"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/evidence/writer-registry.ts",
    ],
    outputPaths: ["rust/evidence-writer-registry.json"],
  },
  {
    commandId: "spec-audit-webref-grammar",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/generate-rust-omena-spec-audit-webref-grammar.ts",
    ],
    writerScripts: [
      "scripts/generate-rust-omena-spec-audit-webref-grammar.ts",
      "scripts/webref-grammar-extract.ts",
    ],
    outputPaths: [
      "rust/crates/omena-spec-audit/data/webref-grammar.json",
      "rust/crates/omena-spec-audit/data/webref-registry-delta.json",
    ],
  },
  {
    commandId: "spec-audit-coverage-gap",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/generate-rust-omena-coverage-gap.ts",
      "--write",
    ],
    writerScripts: [
      "scripts/generate-rust-omena-coverage-gap.ts",
      "scripts/coverage-gap-report.ts",
    ],
    outputPaths: ["rust/crates/omena-spec-audit/data/omena-coverage-gap.json"],
  },
  {
    commandId: "syntax-line-index-authority",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-rust-line-index-authority.ts",
      "--write",
    ],
    writerScripts: ["scripts/check-rust-line-index-authority.ts"],
    outputPaths: [
      "rust/crates/omena-syntax/tests/snapshots/line-index-authority-baseline.json",
      "rust/crates/omena-syntax/tests/snapshots/public-api.txt",
    ],
  },
  {
    commandId: "query-public-surface",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-rust-omena-query-public-surface.ts",
      "--write",
    ],
    writerScripts: ["scripts/check-rust-omena-query-public-surface.ts"],
    outputPaths: [
      "rust/crates/omena-query/tests/snapshots/public-api.txt",
      "rust/crates/omena-query/tests/snapshots/wildcard-reexport-baseline.json",
    ],
  },
  {
    commandId: "query-public-surface-all-features",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-rust-omena-query-public-surface.ts",
      "--all-features",
      "--write",
    ],
    writerScripts: ["scripts/check-rust-omena-query-public-surface.ts"],
    outputPaths: ["rust/crates/omena-query/tests/snapshots/public-api-all-features.txt"],
  },
  {
    commandId: "reactive-public-surface",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-rust-omena-reactive-public-surface.ts",
      "--write",
    ],
    writerScripts: ["scripts/check-rust-omena-reactive-public-surface.ts"],
    outputPaths: ["rust/crates/omena-reactive/tests/snapshots/public-api.txt"],
  },
  {
    commandId: "published-crate-public-surface-snapshots",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-rust-published-crate-surface-register.ts",
      "--write-snapshots",
    ],
    writerScripts: ["scripts/check-rust-published-crate-surface-register.ts"],
    outputPaths: [
      "rust/crates/omena-reactive/tests/snapshots/public-api.txt",
      "rust/crates/omena-syntax/tests/snapshots/public-api.txt",
      "rust/crates/omena-cascade/tests/snapshots/public-api.txt",
      "rust/crates/omena-tsgo-client/tests/snapshots/public-api.txt",
      "rust/crates/omena-query-transform-runner/tests/snapshots/public-api.txt",
      "rust/crates/omena-wasm/tests/snapshots/public-api.txt",
    ],
  },
  {
    commandId: "documentation-reference-surface",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-docs-reference-surface.ts",
      "--write",
    ],
    writerScripts: ["scripts/check-docs-reference-surface.ts"],
    outputPaths: [
      "docs/reference/README.md",
      "docs/reference/cli.md",
      "docs/reference/personas.md",
      "docs/reference/configuration.md",
      "docs/reference/editor-settings.md",
      "docs/reference/lsp-capabilities.md",
      "rust/crates/omena-cli/README.md",
      "docs/vscode-extension.md",
      "docs/sdk.md",
    ],
  },
  {
    commandId: "external-corpus-baseline",
    writeCommand: ["node", "--import", "tsx", "./scripts/oss-corpus-farm.ts", "--write-baseline"],
    writerScripts: ["scripts/oss-corpus-farm.ts"],
    outputPaths: [
      "rust/crates/omena-diff-test/oss-corpus-farm/baselines.json",
      "rust/crates/omena-diff-test/oss-corpus-farm/report.json",
    ],
  },
  {
    commandId: "external-corpus-lint-census",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/oss-corpus-farm.ts",
      "--write-lint-census",
    ],
    writerScripts: ["scripts/oss-corpus-farm.ts"],
    outputPaths: [
      "rust/crates/omena-diff-test/oss-corpus-farm/report.json",
      "rust/crates/omena-diff-test/oss-corpus-farm/ranked-set-loss-census.json",
    ],
  },
  {
    commandId: "transform-target-compatibility",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/generate-rust-omena-transform-target-compat.ts",
      "--write",
    ],
    writerScripts: ["scripts/generate-rust-omena-transform-target-compat.ts"],
    outputPaths: [
      "rust/crates/omena-transform-target/data/browser-thresholds.toml",
      "rust/crates/omena-transform-target/data/pass-feature-bindings.toml",
      "rust/crates/omena-transform-target/data/native-stage2-coverage.json",
    ],
  },
  {
    commandId: "contract-parity-v1-goldens",
    writeCommand: ["node", "--import", "tsx", "./scripts/update-contract-parity-v1-golden.ts"],
    writerScripts: [
      "scripts/update-contract-parity-v1-golden.ts",
      "scripts/contract-parity-golden-corpus-v1.ts",
    ],
    outputPaths: [
      "test/_fixtures/contract-parity/type-fact-parity.json",
      "test/_fixtures/contract-parity/source-flow-parity.json",
      "test/_fixtures/contract-parity/source-prefix-suffix-parity.json",
      "test/_fixtures/contract-parity/style-composes-parity.json",
      "test/_fixtures/contract-parity/style-value-imports-parity.json",
      "test/_fixtures/contract-parity/style-keyframes-parity.json",
      "test/_fixtures/contract-parity/style-less-parity.json",
    ],
  },
  {
    commandId: "benchmark-emitted-css-golden",
    writeCommand: [
      "cargo",
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-benchmarks",
      "--bin",
      "emitted_css_golden_gate",
      "--quiet",
      "--",
      "--regen",
    ],
    writerScripts: [
      "rust/crates/omena-benchmarks/src/bin/emitted_css_golden_gate.rs",
      "rust/crates/omena-benchmarks/src/lib.rs",
    ],
    inputPaths: ["rust/Cargo.toml", "rust/Cargo.lock"],
    outputPaths: ["rust/crates/omena-benchmarks/fixtures/emitted-css-golden-v0.json"],
  },
  {
    commandId: "benchmark-transform-relex-baseline",
    writeCommand: [
      "cargo",
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-benchmarks",
      "--bin",
      "transform_relex_baseline_gate",
      "--quiet",
      "--",
      "--regen",
    ],
    writerScripts: [
      "rust/crates/omena-benchmarks/src/bin/transform_relex_baseline_gate.rs",
      "rust/crates/omena-benchmarks/src/lib.rs",
    ],
    inputPaths: ["rust/Cargo.toml", "rust/Cargo.lock"],
    outputPaths: ["rust/crates/omena-benchmarks/fixtures/transform-relex-baseline-v0.json"],
  },
  {
    commandId: "linked-stylesheet-byte-identity",
    writeCommand: [
      "env",
      "OMENA_UPDATE_LINKED_STYLESHEET_BYTE_IDENTITY=1",
      "cargo",
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-bundler",
      "--test",
      "linked_stylesheet_byte_identity",
      "linked_stylesheet_output_matches_committed_contract",
      "--",
      "--exact",
      "--nocapture",
    ],
    writerScripts: ["rust/crates/omena-bundler/tests/linked_stylesheet_byte_identity.rs"],
    inputPaths: ["rust/Cargo.toml", "rust/Cargo.lock"],
    outputPaths: ["rust/crates/omena-bundler/tests/snapshots/linked-stylesheet-byte-identity.json"],
  },
] as const satisfies readonly EvidenceWriterCommandDeclaration[];

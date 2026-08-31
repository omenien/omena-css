import { readFileSync } from "node:fs";
import path from "node:path";

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
  readonly manifestOutputPaths?: readonly EvidenceWriterManifestOutputDeclaration[];
  readonly inputPaths?: readonly string[];
}

export interface EvidenceWriterManifestOutputDeclaration {
  readonly manifestPath: string;
  readonly propertyPath: readonly string[];
  readonly baseDirectory: string;
}

/**
 * Resolve paths that are deliberately owned by committed manifest data rather
 * than by a TypeScript string literal. This authority can therefore name an
 * output even when the static write-call discovery cannot see it.
 */
export function resolveEvidenceWriterCommandOutputPaths(
  repoRoot: string,
  declaration: EvidenceWriterCommandDeclaration,
): readonly string[] {
  const outputPaths = new Set(declaration.outputPaths);
  for (const manifestOutput of declaration.manifestOutputPaths ?? []) {
    const manifest = JSON.parse(
      readFileSync(path.join(repoRoot, manifestOutput.manifestPath), "utf8"),
    ) as unknown;
    let value: unknown = manifest;
    for (const property of manifestOutput.propertyPath) {
      if (typeof value !== "object" || value === null || !(property in value)) {
        throw new Error(
          `evidence writer manifest output property is absent: ${declaration.commandId}:${manifestOutput.manifestPath}:${manifestOutput.propertyPath.join(".")}`,
        );
      }
      value = (value as Readonly<Record<string, unknown>>)[property];
    }
    if (typeof value !== "string" || value.length === 0) {
      throw new Error(
        `evidence writer manifest output property is not a path: ${declaration.commandId}:${manifestOutput.manifestPath}:${manifestOutput.propertyPath.join(".")}`,
      );
    }
    const outputPath = path.posix.normalize(path.posix.join(manifestOutput.baseDirectory, value));
    if (path.posix.isAbsolute(outputPath) || outputPath === ".." || outputPath.startsWith("../")) {
      throw new Error(
        `evidence writer manifest output escapes the repository: ${declaration.commandId}:${outputPath}`,
      );
    }
    outputPaths.add(outputPath);
  }
  return [...outputPaths].toSorted();
}

/**
 * Independent, hand-reviewed superset for every non-root repository path that a
 * tracked TypeScript/JavaScript write call can emit. The source scanner does not
 * produce this list. Both directions are checked: a newly discovered output must
 * be added here, and every path here must still be recovered from its write call.
 */
export const EVIDENCE_STATIC_WRITE_OUTPUT_AUTHORITY = [
  "docs/reference/crates.md",
  "packages/css-build-adapter/bundler-host-contract.generated.d.ts",
  "packages/css-build-adapter/interface-member-snapshot.json",
  "rust/crates/omena-abstract-value/data/closed-world-builtin-token-profiles.json",
  "rust/crates/omena-abstract-value/data/closed-world-keyword-closure-certificate.json",
  "rust/crates/omena-abstract-value/tests/fixtures/value-grammar-real-declarations.json",
  "rust/crates/omena-benchmarks/baselines/parser-edit-slope-baseline-v0.json",
  "rust/crates/omena-benchmarks/baselines/wpt-case-count-baseline-v0.json",
  "rust/crates/omena-benchmarks/baselines/z5-perf-gate-baseline-v0.json",
  "rust/crates/omena-bundler/tests/snapshots/public-api.txt",
  "rust/crates/omena-cascade-proof/discharge-ledger/ledger.v1.json",
  "rust/crates/omena-cascade/src/property_metadata_idl_generated.rs",
  "rust/crates/omena-cli/json-output-census.json",
  "rust/crates/omena-diff-test/oss-corpus-farm/baselines.json",
  "rust/crates/omena-diff-test/oss-corpus-farm/linked-emission-coverage-census.json",
  "rust/crates/omena-diff-test/oss-corpus-farm/ranked-set-loss-census.json",
  "rust/crates/omena-diff-test/oss-corpus-farm/report.json",
  "rust/crates/omena-diff-test/sass-spec-corpus/conformance-smoke-manifest.json",
  "rust/crates/omena-diff-test/sass-spec-corpus/conformance-smoke-oracle.json",
  "rust/crates/omena-diff-test/sass-spec-corpus/conformance-smoke.json",
  "rust/crates/omena-diff-test/sass-spec-corpus/imported-smoke-manifest.json",
  "rust/crates/omena-diff-test/sass-spec-corpus/imported-smoke-oracle.json",
  "rust/crates/omena-diff-test/sass-spec-corpus/imported-smoke.json",
  "rust/crates/omena-diff-test/sass-spec-corpus/upstream-scale.json",
  "rust/crates/omena-diff-test/src/external_corpus_envelope_idl_generated.rs",
  "rust/crates/omena-diff-test/wpt-corpus/extracted/tier-zero-coverage.json",
  "rust/crates/omena-diff-test/wpt-corpus/extracted/tier-zero-tuples.json",
  "rust/crates/omena-diff-test/wpt-corpus/manifest.json",
  "rust/crates/omena-engine-input-producers/src/engine_contract_v2_idl_generated.rs",
  "rust/crates/omena-napi/src/engine_napi_contract_idl_generated.rs",
  "rust/crates/omena-parser/src/parse_tree_contract_idl_generated.rs",
  "rust/crates/omena-query-transform-runner/plugin-abi-stability-contract.json",
  "rust/crates/omena-query-transform-runner/plugin-consumption-law-census.json",
  "rust/crates/omena-query/src/sdk_workflow_contract_idl_generated.rs",
  "rust/crates/omena-query/tests/snapshots/wildcard-reexport-baseline.json",
  "rust/crates/omena-spec-audit/data/omena-conformance-dashboard.json",
  "rust/crates/omena-spec-audit/data/omena-coverage-gap.json",
  "rust/crates/omena-spec-audit/data/value-grammar-differential.json",
  "rust/crates/omena-spec-audit/data/value-grammar-evidence.json",
  "rust/crates/omena-spec-audit/data/webref-grammar.json",
  "rust/crates/omena-spec-audit/data/webref-registry-delta.json",
  "rust/crates/omena-syntax/tests/snapshots/line-index-authority-baseline.json",
  "rust/crates/omena-syntax/tests/snapshots/public-api.txt",
  "server/engine-core-ts/src/contracts/engine-napi-boundary-idl.generated.ts",
  "server/engine-core-ts/src/contracts/engine-sdk-workflow-idl.generated.ts",
  "server/engine-core-ts/src/contracts/engine-v2-input-idl.generated.ts",
  "server/engine-core-ts/src/contracts/engine-v2-output-idl.generated.ts",
  "server/engine-core-ts/src/contracts/external-corpus-envelope-idl.generated.ts",
  "server/engine-core-ts/src/contracts/parse-tree-idl.generated.ts",
  "server/engine-core-ts/src/contracts/property-metadata-idl.generated.ts",
  "server/engine-host-node/src/code-action-query-idl.generated.ts",
  "server/engine-host-node/src/engine-output-v2-idl.generated.ts",
  "server/engine-host-node/src/engine-query-v2-idl.generated.ts",
  "server/engine-host-node/src/query-diagnostics-idl.generated.ts",
  "test/_fixtures/contract-parity-v2/source-char-inclusion-parity-v2.json",
  "test/_fixtures/contract-parity-v2/source-composite-parity-v2.json",
  "test/_fixtures/contract-parity-v2/source-prefix-suffix-overlap-parity-v2.json",
  "test/_fixtures/contract-parity-v2/source-prefix-suffix-parity-v2.json",
  "test/_fixtures/contract-parity-v2/source-unicode-length-parity-v2.json",
  "test/_fixtures/contract-parity-v2/type-fact-parity-v2.json",
  "test/_fixtures/contract-parity/source-flow-parity.json",
  "test/_fixtures/contract-parity/source-prefix-suffix-parity.json",
  "test/_fixtures/contract-parity/style-composes-parity.json",
  "test/_fixtures/contract-parity/style-keyframes-parity.json",
  "test/_fixtures/contract-parity/style-less-parity.json",
  "test/_fixtures/contract-parity/style-value-imports-parity.json",
  "test/_fixtures/contract-parity/type-fact-parity.json",
] as const;

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
    commandId: "engine-v2-contract-idl-generated",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/generate-engine-v2-contract-idl.ts",
      "--write",
    ],
    writerScripts: [
      "scripts/generate-engine-v2-contract-idl.ts",
      "scripts/engine-v2-contract-idl-files.ts",
    ],
    outputPaths: [
      "server/engine-core-ts/src/contracts/engine-v2-input-idl.generated.ts",
      "server/engine-core-ts/src/contracts/engine-v2-output-idl.generated.ts",
      "server/engine-core-ts/src/contracts/parse-tree-idl.generated.ts",
      "server/engine-core-ts/src/contracts/external-corpus-envelope-idl.generated.ts",
      "server/engine-core-ts/src/contracts/property-metadata-idl.generated.ts",
      "server/engine-core-ts/src/contracts/engine-napi-boundary-idl.generated.ts",
      "server/engine-core-ts/src/contracts/engine-sdk-workflow-idl.generated.ts",
      "packages/css-build-adapter/bundler-host-contract.generated.d.ts",
      "server/engine-host-node/src/engine-output-v2-idl.generated.ts",
      "server/engine-host-node/src/engine-query-v2-idl.generated.ts",
      "server/engine-host-node/src/code-action-query-idl.generated.ts",
      "server/engine-host-node/src/query-diagnostics-idl.generated.ts",
      "rust/crates/omena-engine-input-producers/src/engine_contract_v2_idl_generated.rs",
      "rust/crates/omena-parser/src/parse_tree_contract_idl_generated.rs",
      "rust/crates/omena-diff-test/src/external_corpus_envelope_idl_generated.rs",
      "rust/crates/omena-cascade/src/property_metadata_idl_generated.rs",
      "rust/crates/omena-napi/src/engine_napi_contract_idl_generated.rs",
      "rust/crates/omena-query/src/sdk_workflow_contract_idl_generated.rs",
    ],
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
    outputPaths: ["rust/crates/omena-diff-test/oss-corpus-farm/ranked-set-loss-census.json"],
    manifestOutputPaths: [
      {
        manifestPath: "rust/crates/omena-diff-test/oss-corpus-farm/manifest.json",
        propertyPath: ["lintCensus", "reportPath"],
        baseDirectory: "rust/crates/omena-diff-test/oss-corpus-farm",
      },
    ],
    inputPaths: ["rust/crates/omena-diff-test/oss-corpus-farm/manifest.json"],
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
    commandId: "contract-parity-v2-goldens",
    writeCommand: ["node", "--import", "tsx", "./scripts/update-contract-parity-v2-golden.ts"],
    writerScripts: [
      "scripts/update-contract-parity-v2-golden.ts",
      "scripts/contract-parity-golden-corpus-v2.ts",
      "scripts/contract-parity-corpus-v2.ts",
      "scripts/contract-parity-v2-fixture-selection.ts",
    ],
    outputPaths: [
      "test/_fixtures/contract-parity-v2/type-fact-parity-v2.json",
      "test/_fixtures/contract-parity-v2/source-prefix-suffix-parity-v2.json",
      "test/_fixtures/contract-parity-v2/source-char-inclusion-parity-v2.json",
      "test/_fixtures/contract-parity-v2/source-prefix-suffix-overlap-parity-v2.json",
      "test/_fixtures/contract-parity-v2/source-composite-parity-v2.json",
      "test/_fixtures/contract-parity-v2/source-unicode-length-parity-v2.json",
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

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
  readonly writeExpressions?: readonly string[];
  readonly outputWriteWitnesses?: readonly EvidenceWriterOutputWriteWitness[];
  readonly manifestOutputPaths?: readonly EvidenceWriterManifestOutputDeclaration[];
  readonly inputPaths?: readonly string[];
}

export interface EvidenceWriterOutputWriteWitness {
  readonly outputPaths: readonly string[];
  readonly writerScript: string;
  readonly writeExpression: string;
}

export interface EvidenceWriterUnresolvedWriteWitnessLedgerEntry {
  readonly commandId: string;
  readonly outputPath: string;
  readonly reason: string;
}

export interface EvidenceWriterManifestOutputDeclaration {
  readonly manifestPath: string;
  readonly baseDirectory: string;
  readonly writeExpression: string;
  readonly propertyPath?: readonly string[];
  readonly recordCollectionPath?: readonly string[];
  readonly recordOutputProperty?: string;
  readonly recordFilter?: {
    readonly property: string;
    readonly equals: string;
  };
}

export interface EvidenceWriterNonLiteralWriteRefusal {
  readonly writerScript: string;
  readonly writeExpression: string;
  readonly kind:
    | "ephemeral-fixture"
    | "operator-selected-external-output"
    | "failure-triggered-raw-capture";
  readonly reason: string;
}

export interface EvidenceWriterOutputJustification {
  readonly commandId?: string;
  readonly outputPath: string;
  readonly writerScript: string;
  readonly writeExpression: string;
  readonly kind: "non-typescript-writer" | "runtime-variable-output";
  readonly reason: string;
  readonly variableFields?: readonly {
    readonly fieldPath: string;
    readonly source: "timestamp" | "wallTime" | "sha" | "worktreeClean" | "host";
  }[];
}

/**
 * Outputs whose committed bytes intentionally record one measured execution.
 * This classification is independent of the justification text so removing a
 * justification cannot silently make a run-varying output reproducible.
 */
export const EVIDENCE_RUNTIME_VARIABLE_OUTPUT_PATHS = [
  "rust/crates/omena-benchmarks/baselines/wpt-case-count-baseline-v0.json",
] as const;

/**
 * Identity-keyed, decrease-only authority for declaration witnesses whose
 * write expression is AST-visible but not yet statically path-resolvable. A
 * new unresolved (commandId, outputPath) pair is a typed refusal; resolving an
 * existing pair requires deleting its row. The generated writer census echoes
 * this ledger so its official updater can reject growth independently of the
 * total count.
 */
export const EVIDENCE_UNRESOLVED_WRITE_WITNESS_LEDGER = [
  {
    commandId: "documentation-reference-surface",
    outputPath: "docs/reference/README.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "documentation-reference-surface",
    outputPath: "docs/reference/cli.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "documentation-reference-surface",
    outputPath: "docs/reference/configuration.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "documentation-reference-surface",
    outputPath: "docs/reference/editor-settings.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "documentation-reference-surface",
    outputPath: "docs/reference/lsp-capabilities.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "documentation-reference-surface",
    outputPath: "docs/reference/personas.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "documentation-reference-surface",
    outputPath: "docs/sdk.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "documentation-reference-surface",
    outputPath: "docs/vscode-extension.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "documentation-reference-surface",
    outputPath: "rust/crates/omena-cli/README.md",
    reason:
      "the documentation writer derives absolutePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "evidence-scan-surfaces",
    outputPath: "rust/evidence-scan-surfaces.json",
    reason:
      "the CLI receives manifestPath from the scan-surface writer result rather than a locally resolvable path expression",
  },
  {
    commandId: "evidence-writer-registry",
    outputPath: "rust/evidence-writer-nonliteral-write-census.json",
    reason:
      "the CLI writes output.path while iterating writer-result descriptors that the static path evaluator does not expand",
  },
  {
    commandId: "evidence-writer-registry",
    outputPath: "rust/evidence-writer-registry.json",
    reason:
      "the CLI writes output.path while iterating writer-result descriptors that the static path evaluator does not expand",
  },
  {
    commandId: "orchestrator-ci-workflow",
    outputPath: ".github/workflows/ci.yml",
    reason:
      "the workflow writer calls an imported root-relative path builder that the static path evaluator does not execute",
  },
  {
    commandId: "orchestrator-ci-workflow-registry",
    outputPath: "packages/check-orchestrator/ci-workflow.json",
    reason:
      "the workflow writer receives registryPath through a helper-owned result that the static path evaluator does not inline",
  },
  {
    commandId: "orchestrator-cost-ledger",
    outputPath: "packages/check-orchestrator/ci-cost-ledger.json",
    reason:
      "the CLI calls an imported root-relative cost-ledger path builder that the static path evaluator does not execute",
  },
  {
    commandId: "orchestrator-inventory",
    outputPath: "packages/check-orchestrator/CHECKS.md",
    reason:
      "the CLI derives inventoryPath from its command root through an alias the static path evaluator does not resolve",
  },
  {
    commandId: "transform-target-compatibility",
    outputPath: "rust/crates/omena-transform-target/data/browser-thresholds.toml",
    reason:
      "the compatibility writer derives relativePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "transform-target-compatibility",
    outputPath: "rust/crates/omena-transform-target/data/native-stage2-coverage.json",
    reason:
      "the compatibility writer derives relativePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
  {
    commandId: "transform-target-compatibility",
    outputPath: "rust/crates/omena-transform-target/data/pass-feature-bindings.toml",
    reason:
      "the compatibility writer derives relativePath from an Object.entries output-table loop that the static path evaluator does not resolve",
  },
] as const satisfies readonly EvidenceWriterUnresolvedWriteWitnessLedgerEntry[];

/**
 * A governed writer may use a computed write target only when it is either
 * resolved to a repository output, bound to committed data above, or refused
 * here as a non-repository side effect. Exact expression keys keep this list
 * shrink-only and make a newly shaped computed write fail the authority gate.
 */
export const EVIDENCE_WRITER_NON_LITERAL_WRITE_REFUSALS: readonly EvidenceWriterNonLiteralWriteRefusal[] =
  [
    {
      writerScript: "packages/check-orchestrator/src/cli/main.ts",
      writeExpression: "artifactPath",
      kind: "operator-selected-external-output",
      reason: "check summary artifacts are written only to the operator-selected summary directory",
    },
    {
      writerScript: "packages/check-orchestrator/src/evidence/scan-surface-registry.ts",
      writeExpression: 'path.join(temporaryRoot, "scan-surface.ts")',
      kind: "ephemeral-fixture",
      reason: "historical resolver replay is materialized under a fresh operating-system temp root",
    },
    {
      writerScript: "packages/check-orchestrator/src/evidence/scan-surface-registry.ts",
      writeExpression: "path.join(predicateDirectory, path.posix.basename(sourcePath))",
      kind: "ephemeral-fixture",
      reason: "historical predicate replay is materialized under the same fresh temp root",
    },
    {
      writerScript: "scripts/check-docs-reference-surface.ts",
      writeExpression: 'path.join(fixtureRoot, "src/input.css")',
      kind: "ephemeral-fixture",
      reason: "documentation examples are exercised in an operating-system temp fixture",
    },
    {
      writerScript: "scripts/check-docs-reference-surface.ts",
      writeExpression: 'path.join(fixtureRoot, "omena.toml")',
      kind: "ephemeral-fixture",
      reason: "documentation examples are exercised in an operating-system temp fixture",
    },
    {
      writerScript: "scripts/check-docs-reference-surface.ts",
      writeExpression: "destination",
      kind: "ephemeral-fixture",
      reason: "relative extends files are bounded to the documentation example temp root",
    },
    {
      writerScript: "scripts/generate-engine-v2-contract-idl.ts",
      writeExpression: "tempFile",
      kind: "ephemeral-fixture",
      reason: "formatter input is staged in a fresh temp file before committed outputs are written",
    },
    {
      writerScript: "scripts/oss-corpus-farm.ts",
      writeExpression: 'path.join(fixtureDir, "fixture.omena")',
      kind: "operator-selected-external-output",
      reason: "corpus capture fixtures are written below the explicitly selected capture root",
    },
    {
      writerScript: "scripts/oss-corpus-farm.ts",
      writeExpression: "manifestPathForCapture",
      kind: "failure-triggered-raw-capture",
      reason:
        "the manifest is a conditional failure-capture side effect whose destination is either an operator-selected root or the tracked raw-regression inbox, not a reproducible updater output",
    },
  ];

/**
 * A typed exception is narrower than a declaration: it binds one output to
 * one source-level write expression and explains why the TypeScript AST
 * authority cannot prove stable bytes for it. Runtime-variable artifacts name
 * every field that prevents a deterministic full-repository rewrite.
 */
export const EVIDENCE_WRITER_OUTPUT_JUSTIFICATIONS: readonly EvidenceWriterOutputJustification[] = [
  {
    commandId: "benchmark-emitted-css-golden",
    outputPath: "rust/crates/omena-benchmarks/fixtures/emitted-css-golden-v0.json",
    writerScript: "rust/crates/omena-benchmarks/src/bin/emitted_css_golden_gate.rs",
    writeExpression: "fs::write(&path, snapshot)",
    kind: "non-typescript-writer",
    reason: "the declared output is written by the Rust regeneration binary",
  },
  {
    commandId: "benchmark-transform-relex-baseline",
    outputPath: "rust/crates/omena-benchmarks/fixtures/transform-relex-baseline-v0.json",
    writerScript: "rust/crates/omena-benchmarks/src/bin/transform_relex_baseline_gate.rs",
    writeExpression: "fs::write(&path, snapshot)",
    kind: "non-typescript-writer",
    reason: "the declared output is written by the Rust regeneration binary",
  },
  {
    commandId: "linked-stylesheet-byte-identity",
    outputPath: "rust/crates/omena-bundler/tests/snapshots/linked-stylesheet-byte-identity.json",
    writerScript: "rust/crates/omena-bundler/tests/linked_stylesheet_byte_identity.rs",
    writeExpression: "std::fs::write(",
    kind: "non-typescript-writer",
    reason: "the declared output is written by the Rust contract test's explicit update mode",
  },
  {
    outputPath: "rust/crates/omena-benchmarks/baselines/wpt-case-count-baseline-v0.json",
    writerScript: "scripts/check-rust-omena-diff-test-wpt-perf.ts",
    writeExpression: "baselinePath",
    kind: "runtime-variable-output",
    reason:
      "the recorder deliberately captures a measured run and cannot be reproduced by an unqualified full-repository --write sweep",
    variableFields: [
      { fieldPath: "samples[].recordedAtUtc", source: "timestamp" },
      { fieldPath: "samples[].wallTimeMilliseconds", source: "wallTime" },
      { fieldPath: "samples[].microsecondsPerCase", source: "wallTime" },
      { fieldPath: "samples[].gitSha", source: "sha" },
      { fieldPath: "samples[].worktreeClean", source: "worktreeClean" },
      { fieldPath: "samples[].machine", source: "host" },
    ],
  },
];

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
    const scalarMode = manifestOutput.propertyPath !== undefined;
    const recordMode = manifestOutput.recordCollectionPath !== undefined;
    if (scalarMode === recordMode) {
      throw new Error(
        `evidence writer manifest output must select one data mode: ${declaration.commandId}:${manifestOutput.manifestPath}`,
      );
    }
    if (scalarMode) {
      const value = resolveManifestProperty(
        manifest,
        manifestOutput.propertyPath!,
        declaration.commandId,
        manifestOutput.manifestPath,
      );
      addManifestOutputPath(outputPaths, declaration.commandId, manifestOutput, value);
      continue;
    }
    if (!manifestOutput.recordOutputProperty) {
      throw new Error(
        `evidence writer record output selector is incomplete: ${declaration.commandId}:${manifestOutput.manifestPath}`,
      );
    }
    const records = resolveManifestProperty(
      manifest,
      manifestOutput.recordCollectionPath!,
      declaration.commandId,
      manifestOutput.manifestPath,
    );
    if (!Array.isArray(records)) {
      throw new Error(
        `evidence writer manifest record collection is not an array: ${declaration.commandId}:${manifestOutput.manifestPath}:${manifestOutput.recordCollectionPath!.join(".")}`,
      );
    }
    for (const [index, record] of records.entries()) {
      if (typeof record !== "object" || record === null) {
        throw new Error(
          `evidence writer manifest record is not an object: ${declaration.commandId}:${manifestOutput.manifestPath}:${index}`,
        );
      }
      const fields = record as Readonly<Record<string, unknown>>;
      if (
        manifestOutput.recordFilter &&
        fields[manifestOutput.recordFilter.property] !== manifestOutput.recordFilter.equals
      ) {
        continue;
      }
      if (!(manifestOutput.recordOutputProperty in fields)) {
        throw new Error(
          `evidence writer manifest record output is absent: ${declaration.commandId}:${manifestOutput.manifestPath}:${index}:${manifestOutput.recordOutputProperty}`,
        );
      }
      addManifestOutputPath(
        outputPaths,
        declaration.commandId,
        manifestOutput,
        fields[manifestOutput.recordOutputProperty],
      );
    }
  }
  return [...outputPaths].toSorted();
}

function resolveManifestProperty(
  manifest: unknown,
  propertyPath: readonly string[],
  commandId: string,
  manifestPath: string,
): unknown {
  let value = manifest;
  for (const property of propertyPath) {
    if (typeof value !== "object" || value === null || !(property in value)) {
      throw new Error(
        `evidence writer manifest output property is absent: ${commandId}:${manifestPath}:${propertyPath.join(".")}`,
      );
    }
    value = (value as Readonly<Record<string, unknown>>)[property];
  }
  return value;
}

function addManifestOutputPath(
  outputPaths: Set<string>,
  commandId: string,
  declaration: EvidenceWriterManifestOutputDeclaration,
  value: unknown,
): void {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(
      `evidence writer manifest output property is not a path: ${commandId}:${declaration.manifestPath}`,
    );
  }
  const outputPath = path.posix.normalize(path.posix.join(declaration.baseDirectory, value));
  if (path.posix.isAbsolute(outputPath) || outputPath === ".." || outputPath.startsWith("../")) {
    throw new Error(
      `evidence writer manifest output escapes the repository: ${commandId}:${outputPath}`,
    );
  }
  outputPaths.add(outputPath);
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
  "rust/crates/omena-reactive/tests/snapshots/public-api.txt",
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
    writeExpressions: ["inventoryPath"],
    outputWriteWitnesses: [
      {
        outputPaths: ["packages/check-orchestrator/CHECKS.md"],
        writerScript: "packages/check-orchestrator/src/cli/main.ts",
        writeExpression: "inventoryPath",
      },
    ],
  },
  {
    commandId: "orchestrator-cost-ledger",
    writeCommand: ["pnpm", "omena-check", "cost-ledger", "--write", "--", "--reuse-source-runs"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/manifest/cost-ledger.ts",
    ],
    outputPaths: ["packages/check-orchestrator/ci-cost-ledger.json"],
    writeExpressions: ["costLedgerPath(rootDir)"],
    outputWriteWitnesses: [
      {
        outputPaths: ["packages/check-orchestrator/ci-cost-ledger.json"],
        writerScript: "packages/check-orchestrator/src/cli/main.ts",
        writeExpression: "costLedgerPath(rootDir)",
      },
    ],
  },
  {
    commandId: "orchestrator-ci-workflow",
    writeCommand: ["pnpm", "omena-check", "ci-workflow", "--write"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/manifest/ci-workflow.ts",
    ],
    outputPaths: [".github/workflows/ci.yml"],
    writeExpressions: ["ciWorkflowPath(rootDir)"],
    outputWriteWitnesses: [
      {
        outputPaths: [".github/workflows/ci.yml"],
        writerScript: "packages/check-orchestrator/src/manifest/ci-workflow.ts",
        writeExpression: "ciWorkflowPath(rootDir)",
      },
    ],
  },
  {
    commandId: "orchestrator-ci-workflow-registry",
    writeCommand: ["pnpm", "omena-check", "ci-workflow", "--adopt"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/manifest/ci-workflow.ts",
    ],
    outputPaths: ["packages/check-orchestrator/ci-workflow.json"],
    writeExpressions: ["ciWorkflowRegistryPath(rootDir)", "registryPath"],
    outputWriteWitnesses: [
      {
        outputPaths: ["packages/check-orchestrator/ci-workflow.json"],
        writerScript: "packages/check-orchestrator/src/manifest/ci-workflow.ts",
        writeExpression: "registryPath",
      },
    ],
  },
  {
    commandId: "evidence-scan-surfaces",
    writeCommand: ["pnpm", "omena-check", "evidence-surfaces", "--write"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/evidence/scan-surface-registry.ts",
    ],
    outputPaths: ["rust/evidence-scan-surfaces.json"],
    writeExpressions: ["manifestPath"],
    outputWriteWitnesses: [
      {
        outputPaths: ["rust/evidence-scan-surfaces.json"],
        writerScript: "packages/check-orchestrator/src/cli/main.ts",
        writeExpression: "manifestPath",
      },
    ],
  },
  {
    commandId: "evidence-writer-registry",
    writeCommand: ["pnpm", "omena-check", "evidence-writers", "--write"],
    writerScripts: [
      "packages/check-orchestrator/src/cli/main.ts",
      "packages/check-orchestrator/src/evidence/writer-registry.ts",
    ],
    outputPaths: [
      "rust/evidence-writer-nonliteral-write-census.json",
      "rust/evidence-writer-registry.json",
    ],
    writeExpressions: ["output.path"],
    outputWriteWitnesses: [
      {
        outputPaths: [
          "rust/evidence-writer-nonliteral-write-census.json",
          "rust/evidence-writer-registry.json",
        ],
        writerScript: "packages/check-orchestrator/src/cli/main.ts",
        writeExpression: "output.path",
      },
    ],
  },
  {
    commandId: "diff-test-wpt-corpus",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/generate-rust-omena-diff-test-wpt-corpus.ts",
    ],
    writerScripts: ["scripts/generate-rust-omena-diff-test-wpt-corpus.ts"],
    outputPaths: ["rust/crates/omena-diff-test/wpt-corpus/manifest.json"],
    manifestOutputPaths: [
      {
        manifestPath: "rust/crates/omena-diff-test/wpt-corpus/selections.json",
        propertyPath: ["chunkPath"],
        baseDirectory: "rust/crates/omena-diff-test/wpt-corpus",
        writeExpression: "path.join(corpusRoot, chunk.chunkPath)",
      },
      {
        manifestPath: "rust/crates/omena-diff-test/wpt-corpus/selections.json",
        recordCollectionPath: ["advisoryChunks"],
        recordOutputProperty: "chunkPath",
        baseDirectory: "rust/crates/omena-diff-test/wpt-corpus",
        writeExpression: "path.join(corpusRoot, chunk.chunkPath)",
      },
    ],
    inputPaths: ["rust/crates/omena-diff-test/wpt-corpus/selections.json"],
  },
  {
    commandId: "identifier-authority-census",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-rust-omena-identifier-authority-census.ts",
      "--write",
    ],
    writerScripts: ["scripts/check-rust-omena-identifier-authority-census.ts"],
    outputPaths: ["rust/omena-identifier-authority-census.json"],
    outputWriteWitnesses: [
      {
        outputPaths: ["rust/omena-identifier-authority-census.json"],
        writerScript: "scripts/check-rust-omena-identifier-authority-census.ts",
        writeExpression: "censusPath",
      },
    ],
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
    writeExpressions: ["snapshotPath"],
    outputPaths: [
      "rust/crates/omena-query/tests/snapshots/public-api.txt",
      "rust/crates/omena-query/tests/snapshots/wildcard-reexport-baseline.json",
    ],
    outputWriteWitnesses: [
      {
        outputPaths: ["rust/crates/omena-query/tests/snapshots/public-api.txt"],
        writerScript: "scripts/check-rust-omena-query-public-surface.ts",
        writeExpression: "snapshotPath",
      },
      {
        outputPaths: ["rust/crates/omena-query/tests/snapshots/wildcard-reexport-baseline.json"],
        writerScript: "scripts/check-rust-omena-query-public-surface.ts",
        writeExpression: "wildcardBaselinePath",
      },
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
    outputWriteWitnesses: [
      {
        outputPaths: ["rust/crates/omena-query/tests/snapshots/public-api-all-features.txt"],
        writerScript: "scripts/check-rust-omena-query-public-surface.ts",
        writeExpression: "snapshotPath",
      },
    ],
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
    writeExpressions: ["writeTarget"],
    outputPaths: ["rust/crates/omena-reactive/tests/snapshots/public-api.txt"],
  },
  {
    commandId: "bundler-public-surface",
    writeCommand: [
      "node",
      "--import",
      "tsx",
      "./scripts/check-rust-omena-bundler-public-surface.ts",
      "--write",
    ],
    writerScripts: ["scripts/check-rust-omena-bundler-public-surface.ts"],
    outputPaths: ["rust/crates/omena-bundler/tests/snapshots/public-api.txt"],
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
    outputPaths: [],
    manifestOutputPaths: [
      {
        manifestPath: "rust/omena-published-crate-surface-register.json",
        recordCollectionPath: ["rows"],
        recordOutputProperty: "snapshot",
        recordFilter: { property: "disposition", equals: "snapshotGated" },
        baseDirectory: ".",
        writeExpression: "snapshotPath",
      },
    ],
    inputPaths: ["rust/omena-published-crate-surface-register.json"],
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
    writeExpressions: ["absolutePath"],
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
    outputWriteWitnesses: [
      {
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
        writerScript: "scripts/check-docs-reference-surface.ts",
        writeExpression: "absolutePath",
      },
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
    outputWriteWitnesses: [
      {
        outputPaths: ["rust/crates/omena-diff-test/oss-corpus-farm/baselines.json"],
        writerScript: "scripts/oss-corpus-farm.ts",
        writeExpression: "baselinePath",
      },
      {
        outputPaths: ["rust/crates/omena-diff-test/oss-corpus-farm/report.json"],
        writerScript: "scripts/oss-corpus-farm.ts",
        writeExpression: "reportPath",
      },
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
    outputWriteWitnesses: [
      {
        outputPaths: ["rust/crates/omena-diff-test/oss-corpus-farm/ranked-set-loss-census.json"],
        writerScript: "scripts/oss-corpus-farm.ts",
        writeExpression: "rankedSetLossCensusPath",
      },
    ],
    manifestOutputPaths: [
      {
        manifestPath: "rust/crates/omena-diff-test/oss-corpus-farm/manifest.json",
        propertyPath: ["lintCensus", "reportPath"],
        baseDirectory: "rust/crates/omena-diff-test/oss-corpus-farm",
        writeExpression: "reportPathForManifest",
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
    writeExpressions: ["path.join(repoRoot, relativePath)"],
    outputPaths: [
      "rust/crates/omena-transform-target/data/browser-thresholds.toml",
      "rust/crates/omena-transform-target/data/pass-feature-bindings.toml",
      "rust/crates/omena-transform-target/data/native-stage2-coverage.json",
    ],
    outputWriteWitnesses: [
      {
        outputPaths: [
          "rust/crates/omena-transform-target/data/browser-thresholds.toml",
          "rust/crates/omena-transform-target/data/pass-feature-bindings.toml",
          "rust/crates/omena-transform-target/data/native-stage2-coverage.json",
        ],
        writerScript: "scripts/generate-rust-omena-transform-target-compat.ts",
        writeExpression: "path.join(repoRoot, relativePath)",
      },
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

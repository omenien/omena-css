import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

const writeClassifications = [
  "artifact",
  "bookkeeping",
  "directory-preparation",
  "transaction-rollback",
  "transaction-staging",
] as const;
type WriteClassification = (typeof writeClassifications)[number];

interface WriteSite {
  readonly path: string;
  readonly function: string;
  readonly writeCount: number;
  readonly classification: WriteClassification;
  readonly owner: string;
}

interface WriteSafetyManifest {
  readonly schemaVersion: "0";
  readonly product: "omena-cli.write-safety-census";
  readonly sourceMutationGate: { readonly path: string; readonly function: string };
  readonly productSourceWriteCallers: number;
  readonly writeSites: readonly WriteSite[];
  readonly consumerContracts: readonly {
    readonly surface: string;
    readonly writeKind: string;
    readonly additionalRequirement: string;
    readonly defaultPosture: string;
  }[];
  readonly namedWaits: readonly {
    readonly surface: string;
    readonly condition: string;
    readonly owner: string;
  }[];
}

interface NonFilesystemWriteSink {
  readonly path: string;
  readonly function: string;
  readonly writeCount: number;
  readonly evidence: string;
}

interface FilesystemMutation {
  readonly api: string;
  readonly destination: string;
  readonly offset: number;
}

interface NamedFunction {
  readonly name: string;
  readonly shortName: string;
  readonly start: number;
  readonly end: number;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cliRoot = "rust/crates/omena-cli/src";
const manifestPath = "rust/crates/omena-cli/write-safety-census.json";
const transactionModulePath = "rust/crates/omena-cli/src/workspace_edit_transaction.rs";
const manifest = readWriteSafetyManifest(manifestPath);
const fixSafetySource = read("rust/crates/omena-checker/src/fix_safety.rs");
const writeGateSource = read(manifest.sourceMutationGate.path);
const queryRunnerSource = read("rust/crates/omena-query-transform-runner/src/lib.rs");
const queryFacadeSource = read("rust/crates/omena-query/src/lib.rs");
const directFilesystemMutation =
  /\b(?:std::|tokio::)?fs::(?:write|copy|rename|hard_link|remove_file|remove_dir|remove_dir_all|create_dir|create_dir_all|set_permissions)\s*\(|\bstd::io::copy\s*\(|\bstd::os::[a-z_]+::fs::symlink(?:_file|_dir)?\s*\(|\b(?:std::fs::)?File::(?:create|create_new|options)\s*\(|\b(?:std::fs::)?OpenOptions::new\s*\(|\.(?:write|write_all|set_len|truncate|create|create_new|append)\s*\(/gu;
const nonFilesystemWriteSinks: readonly NonFilesystemWriteSink[] = [
  {
    path: "rust/crates/omena-cli/src/daemon.rs",
    function: "emit_watch_result",
    writeCount: 1,
    evidence: "std::io::stdout()",
  },
  {
    path: "rust/crates/omena-cli/src/daemon.rs",
    function: "write_wire_bytes",
    writeCount: 2,
    evidence: "TcpStream",
  },
  {
    path: "rust/crates/omena-cli/src/migrate/mod.rs",
    function: "run_sass_migration_oracle",
    writeCount: 1,
    evidence: "stdin = child",
  },
  {
    path: "rust/crates/omena-cli/src/postcss_compat.rs",
    function: "run_node_bridge",
    writeCount: 1,
    evidence: "stdin = child",
  },
];

assert.deepEqual(
  enumDeclarations(`
const DECOY: &str = "pub enum StringLiteral { Safe, Conservative, ManualReview }";
const RAW_DECOY: &str = r#"pub enum RawLiteral { Safe, Conservative, ManualReview }"#;
const LEFT_BRACE: char = '{';
// pub enum LineComment { Safe, Conservative, ManualReview }
/* pub enum BlockComment { Safe, Conservative, ManualReview } */
enum SourceDeclaration {
  Safe,
  Conservative,
  ManualReview,
}
`),
  [{ name: "SourceDeclaration", variants: ["Safe", "Conservative", "ManualReview"] }],
  "write-safety census must inspect Rust declarations rather than literal or comment text",
);

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-cli.write-safety-census");
assert.ok(
  manifest.writeSites.every(({ function: functionName }) => functionName.startsWith("crate::")),
  "every write-site owner must use a full Rust identifier",
);
assert.deepEqual(extractEnumVariants(fixSafetySource, "FixSafetyV0"), [
  "Safe",
  "Conservative",
  "ManualReview",
]);
assert.doesNotMatch(
  fixSafetySource,
  /OmenaCheckerRuleCodeV0/u,
  "fix safety must be derived from evidence rather than a rule-code table",
);
for (const signal of [
  "syntax_preserving",
  "local_semantics_required",
  "local_semantics_ready",
  "closed_world_required",
  "closed_world_ready",
  "reference_precision_required",
  "reference_precision",
]) {
  assert.ok(fixSafetySource.includes(`pub ${signal}:`), `missing evidence signal ${signal}`);
}
for (const precision of ["Exact", "Conservative", "Heuristic", "Unknown"]) {
  assert.ok(
    fixSafetySource.includes(`FactPrecision::${precision}`),
    `FactPrecision::${precision} must affect classification`,
  );
}
assert.ok(fixSafetySource.includes('rationale.push("syntaxSafe")'));
assert.ok(fixSafetySource.includes('rationale.push("localSemanticSafe")'));
assert.ok(fixSafetySource.includes('rationale.push("workspaceClosedWorldSafe")'));

assert.ok(queryRunnerSource.includes("RollbackReceiptV0"));
assert.ok(queryRunnerSource.includes("TransformDecision"));
assert.ok(queryFacadeSource.includes("TransformDecision as OmenaQueryTransformDecisionV0"));
for (const variant of ["Applied", "NoChange", "Blocked", "Rejected"]) {
  assert.ok(
    writeGateSource.includes(`OmenaQueryTransformDecisionV0::${variant}`),
    `write gate must consume TransformDecision::${variant}`,
  );
}

const allRustFiles = rustSourceFiles("rust/crates");
const safetyAuthorities = allRustFiles.filter((file) => read(file).includes("enum FixSafetyV0"));
assert.deepEqual(safetyAuthorities, ["rust/crates/omena-checker/src/fix_safety.rs"]);
assertNoSemanticSafetyCopies(allRustFiles);
assertNoTypeScriptSafetyCopies();

const derivedWriteSites = deriveProductionWriteSites();
const shortIdentifierRegistryMutation = manifest.writeSites.map((site, index) =>
  index === 0
    ? { ...site, function: site.function.slice(site.function.lastIndexOf("::") + 2) }
    : site,
);
assert.throws(
  () => deriveProductionWriteSites(new Map(), shortIdentifierRegistryMutation),
  /unclassified production write/u,
  "a short-name registry row must not replace a full Rust owner identifier",
);
assertProductSourcePlaneZero(derivedWriteSites);
const productSourceBareWriteCount = derivedWriteSites.reduce(
  (count, site) =>
    count +
    (site.classification === "artifact" || site.classification === "bookkeeping"
      ? ((site as WriteSite & { readonly productSourceWriteCount?: number })
          .productSourceWriteCount ?? 0)
      : 0),
  0,
);
assert.equal(productSourceBareWriteCount, 0, "product/source-plane bare write count must be zero");
assert.deepEqual(
  manifest.writeSites.map(siteIdentity).toSorted(),
  derivedWriteSites.map(siteIdentity).toSorted(),
  "every production filesystem write must have an owned classification",
);
const transactionWriteSites = manifest.writeSites.filter(
  ({ path: sitePath }) => sitePath === transactionModulePath,
);
assert.deepEqual(
  transactionWriteSites,
  [
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::prepare_rollback_backup",
      writeCount: 2,
      classification: "bookkeeping",
      owner: "transaction rollback backup preparation",
    },
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::WorkspaceEditTransaction::rename_all",
      writeCount: 1,
      classification: "transaction-staging",
      owner: "transaction staged product publication",
    },
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::TransactionLockGuard::drop",
      writeCount: 1,
      classification: "bookkeeping",
      owner: "transaction concurrency lock cleanup",
    },
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::rollback_and_cleanup",
      writeCount: 2,
      classification: "transaction-rollback",
      owner: "transaction rollback restoration",
    },
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::cleanup_staged",
      writeCount: 2,
      classification: "bookkeeping",
      owner: "transaction staged sidecar cleanup",
    },
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::remove_if_exists",
      writeCount: 1,
      classification: "bookkeeping",
      owner: "transaction sidecar cleanup",
    },
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::write_staged_product_bytes",
      writeCount: 5,
      classification: "transaction-staging",
      owner: "transaction staged product bytes",
    },
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::write_transaction_journal_file",
      writeCount: 4,
      classification: "bookkeeping",
      owner: "transaction rollback journal sidecar",
    },
    {
      path: transactionModulePath,
      function: "crate::workspace_edit_transaction::write_transaction_lock_file",
      writeCount: 4,
      classification: "bookkeeping",
      owner: "transaction concurrency lock sidecar",
    },
  ],
  "transaction primitive owners must remain explicit and purpose-specific",
);
assert.equal(
  manifest.writeSites.filter(({ classification }) => classification === "transaction-staging")
    .length,
  2,
  "transaction staging and publication must have separate primitive owners",
);
assert.equal(
  manifest.writeSites.some(
    ({ path: sitePath, function: siteFunction }) =>
      sitePath === manifest.sourceMutationGate.path &&
      siteFunction === manifest.sourceMutationGate.function,
  ),
  false,
  "the source authorization gate must not own a direct filesystem primitive",
);

const productionGateSource = productionRustSource(writeGateSource);
assertSourceGateRoutesToTransaction(productionGateSource, manifest.sourceMutationGate.function);
const gateOccurrenceCount = [...productionGateSource.matchAll(/\bapply_write_with_safety\s*\(/gu)]
  .length;
assert.equal(gateOccurrenceCount, 1, "write gate must have one definition and no hidden self-call");
const cliProductionSources = rustSourceFiles(cliRoot).map((file) =>
  productionRustSource(read(file)),
);
const allGateOccurrences = cliProductionSources.reduce(
  (count, source) => count + [...source.matchAll(/\bapply_write_with_safety\s*\(/gu)].length,
  0,
);
assert.equal(
  allGateOccurrences - 1,
  manifest.productSourceWriteCallers,
  "routed source-write caller count must remain explicit",
);

const disconnectedGateSource = productionGateSource.replace(".commit()", ".disconnected_commit()");
assert.notEqual(
  disconnectedGateSource,
  productionGateSource,
  "gate disconnection mutation must alter the source",
);
assert.throws(
  () =>
    assertSourceGateRoutesToTransaction(
      disconnectedGateSource,
      manifest.sourceMutationGate.function,
    ),
  /must route to WorkspaceEditTransaction::new\(\.\.\.\)\.commit\(\)/u,
  "disconnecting the source gate from transaction commit must be RED",
);

const directBypassSource = writeGateSource.replace(
  "    report.wrote = true;",
  '    std::fs::write(output_path, content).expect("mutation control");\n    report.wrote = true;',
);
assert.notEqual(directBypassSource, writeGateSource, "direct-write mutation must alter the source");
assert.throws(
  () =>
    deriveProductionWriteSites(new Map([[manifest.sourceMutationGate.path, directBypassSource]])),
  /unclassified production write: .*#.*apply_write_with_safety/u,
  "reintroducing a direct product write must be RED",
);

const transactionSource = read(transactionModulePath);
const minifySource = read("rust/crates/omena-cli/src/minify.rs");
const buildSource = read("rust/crates/omena-cli/src/build.rs");
const modulesPath = "rust/crates/omena-cli/src/modules.rs";
const modulesSource = read(modulesPath);
assertDestinationKeyedPostconditions(transactionSource, minifySource, buildSource);
const inputKeyedMinifyMutation = minifySource.replace(
  "text_reparse_for_destination()",
  "text_reparse_for_path(input.as_path())",
);
assert.notEqual(
  inputKeyedMinifyMutation,
  minifySource,
  "minify path-key mutation must alter source",
);
assert.throws(
  () =>
    assertDestinationKeyedPostconditions(transactionSource, inputKeyedMinifyMutation, buildSource),
  /input-keyed postcondition APIs are forbidden/u,
  "keying the minify postcondition to the input path must be RED",
);
const inputKeyedBuildMutation = buildSource.replace(
  "WorkspaceEditPostconditionV0::style_reparse_for_admitted_output(\n                        summary.execution.output_css.as_bytes(),",
  "WorkspaceEditPostconditionV0::style_reparse_for_admitted_output(\n                        path.as_path(),\n                        summary.execution.output_css.as_bytes(),",
);
assert.notEqual(inputKeyedBuildMutation, buildSource, "build path-key mutation must alter source");
assert.throws(
  () =>
    assertDestinationKeyedPostconditions(transactionSource, minifySource, inputKeyedBuildMutation),
  /input-keyed admitted-output postconditions are forbidden/u,
  "keying either build postcondition to an input path must be RED",
);
const stagedPathKeyMutation = transactionSource.replace(
  "(postcondition.check)(edit.path.as_path(), staged_content.as_slice())",
  "(postcondition.check)(staged_edit.stage.as_path(), staged_content.as_slice())",
);
assert.notEqual(
  stagedPathKeyMutation,
  transactionSource,
  "destination dispatch mutation must alter source",
);
assert.throws(
  () => assertDestinationKeyedPostconditions(stagedPathKeyMutation, minifySource, buildSource),
  /postconditions must receive the actual edit destination/u,
  "dispatching postconditions with a staging or input path must remain RED",
);
const compositeModulesWriteMutation = modulesSource.replace(
  "    for plan in plans {",
  '    for plan in plans {\n        let _ = fs::write(plan.path.as_path(), b"mutation control");',
);
assert.notEqual(
  compositeModulesWriteMutation,
  modulesSource,
  "modules composite write mutation must alter source",
);
const modulesBookkeepingRegistryMutation = manifest.writeSites.map((site) =>
  site.path === modulesPath && site.function === "crate::modules::apply_or_check_module_artifacts"
    ? { ...site, classification: "bookkeeping" as const }
    : site,
);
const compositeModulesWriteSites = deriveProductionWriteSites(
  new Map([[modulesPath, compositeModulesWriteMutation]]),
  modulesBookkeepingRegistryMutation,
);
assert.throws(
  () => assertProductSourcePlaneZero(compositeModulesWriteSites),
  /product\/source-plane bare writes must be zero/u,
  "a modules product write cannot be laundered as bookkeeping",
);
const testModuleMarker = "\n#[cfg(test)]\nmod tests {";
assert.ok(
  transactionSource.includes(testModuleMarker),
  "transaction test module marker is missing",
);
const unregisteredOwnerSource = transactionSource.replace(
  testModuleMarker,
  '\nfn unregistered_transaction_write_authority(path: &std::path::Path) {\n    let _ = std::fs::write(path, b"mutation control");\n}\n\n#[cfg(test)]\nmod tests {',
);
assert.throws(
  () => deriveProductionWriteSites(new Map([[transactionModulePath, unregisteredOwnerSource]])),
  /unclassified production write: .*#.*unregistered_transaction_write_authority/u,
  "an unregistered primitive owner must be RED",
);

const registeredArtifactMutation = [
  ...manifest.writeSites,
  {
    path: transactionModulePath,
    function: "crate::workspace_edit_transaction::unregistered_transaction_write_authority",
    writeCount: 1,
    classification: "artifact" as const,
    owner: "mutation control product bytes",
  },
];
const adoptedArtifactMutationSites = deriveProductionWriteSites(
  new Map([[transactionModulePath, unregisteredOwnerSource]]),
  registeredArtifactMutation,
);
assert.throws(
  () => assertProductSourcePlaneZero(adoptedArtifactMutationSites),
  /product\/source-plane bare writes must be zero/u,
  "registering a new bare product writer as classification:artifact must remain RED",
);

const registeredBookkeepingMutation = registeredArtifactMutation.map((site) =>
  site.function === "crate::workspace_edit_transaction::unregistered_transaction_write_authority"
    ? { ...site, classification: "bookkeeping" as const }
    : site,
);
const adoptedBookkeepingMutationSites = deriveProductionWriteSites(
  new Map([[transactionModulePath, unregisteredOwnerSource]]),
  registeredBookkeepingMutation,
);
assert.throws(
  () => assertProductSourcePlaneZero(adoptedBookkeepingMutationSites),
  /product\/source-plane bare writes must be zero/u,
  "registering a bare product writer as bookkeeping must remain RED",
);

const registeredUnknownClassificationMutation = registeredArtifactMutation.map((site) =>
  site.function === "crate::workspace_edit_transaction::unregistered_transaction_write_authority"
    ? { ...site, classification: "productArtifact" as WriteClassification }
    : site,
);
assert.throws(
  () =>
    deriveProductionWriteSites(
      new Map([[transactionModulePath, unregisteredOwnerSource]]),
      registeredUnknownClassificationMutation,
    ),
  /unknown write classification/u,
  "a non-member write classification must remain RED",
);
const registeredEmptyClassificationMutation = registeredArtifactMutation.map((site) =>
  site.function === "crate::workspace_edit_transaction::unregistered_transaction_write_authority"
    ? { ...site, classification: "" as WriteClassification }
    : site,
);
assert.throws(
  () =>
    deriveProductionWriteSites(
      new Map([[transactionModulePath, unregisteredOwnerSource]]),
      registeredEmptyClassificationMutation,
    ),
  /unknown write classification: <empty>/u,
  "an empty write classification must remain RED",
);

const fsCopyBypassSource = transactionSource.replace(
  testModuleMarker,
  "\nfn copy_product_bytes(source: &std::path::Path, destination: &std::path::Path) {\n    let _ = std::fs::copy(source, destination);\n}\n\n#[cfg(test)]\nmod tests {",
);
assert.throws(
  () => deriveProductionWriteSites(new Map([[transactionModulePath, fsCopyBypassSource]])),
  /unclassified production write: .*#.*copy_product_bytes/u,
  "an fs::copy product-byte bypass must be RED",
);

const fileOptionsWriteBypassSource = transactionSource.replace(
  testModuleMarker,
  '\nfn options_write_product_bytes(path: &std::path::Path) {\n    use std::io::Write as _;\n    let mut file = std::fs::File::options().write(true).open(path).expect("mutation control");\n    let _ = file.write(b"mutation control");\n}\n\n#[cfg(test)]\nmod tests {',
);
assert.throws(
  () =>
    deriveProductionWriteSites(new Map([[transactionModulePath, fileOptionsWriteBypassSource]])),
  /unclassified production write: .*#.*options_write_product_bytes/u,
  "a File::options().write() product-byte bypass must be RED",
);

for (const [label, body, functionName] of [
  [
    "an imported filesystem write alias",
    '    use std::fs::write as emit_bytes;\n    let _ = emit_bytes(path, b"mutation control");',
    "aliased_write_product_bytes",
  ],
  [
    "a bare imported filesystem write",
    '    use std::fs::write;\n    let _ = write(path, b"mutation control");',
    "imported_write_product_bytes",
  ],
  [
    "an imported filesystem namespace alias",
    '    use std::fs as disk;\n    let _ = disk::write(path, b"mutation control");',
    "namespace_aliased_write_product_bytes",
  ],
  [
    "a grouped imported filesystem namespace alias",
    '    use std::fs::{self as disk};\n    let _ = disk::write(path, b"mutation control");',
    "grouped_namespace_aliased_write_product_bytes",
  ],
  [
    "an imported std::io::copy alias",
    "    use std::io::copy as transfer;\n    let _ = transfer(source, destination);",
    "aliased_io_copy_product_bytes",
  ],
  [
    "an imported std::io namespace alias",
    "    use std::io as stream;\n    let _ = stream::copy(source, destination);",
    "namespace_aliased_io_copy_product_bytes",
  ],
  [
    "a hard-link publication",
    "    let _ = std::fs::hard_link(source, destination);",
    "hard_link_product_bytes",
  ],
  [
    "an OS filesystem symlink publication",
    "    let _ = std::os::unix::fs::symlink(source, destination);",
    "symlink_product_bytes",
  ],
  [
    "an OS filesystem namespace alias publication",
    "    use std::os::unix::fs as unix_fs;\n    let _ = unix_fs::symlink(source, destination);",
    "namespace_aliased_symlink_product_bytes",
  ],
  [
    "a remove-then-hard-link replacement",
    "    let _ = std::fs::remove_file(destination);\n    let _ = std::fs::hard_link(source, destination);",
    "replace_with_hard_link_product_bytes",
  ],
] as const) {
  const parameters = body.includes("source")
    ? "source: &std::path::Path, destination: &std::path::Path"
    : "path: &std::path::Path";
  const mutation = transactionSource.replace(
    testModuleMarker,
    `\nfn ${functionName}(${parameters}) {\n${body}\n}\n\n#[cfg(test)]\nmod tests {`,
  );
  assert.throws(
    () => deriveProductionWriteSites(new Map([[transactionModulePath, mutation]])),
    new RegExp(`unclassified production write: .*#.*${functionName}`, "u"),
    `${label} must be RED`,
  );
}

const ungatedTestsPath = "rust/crates/omena-cli/src/tests/emit.rs";
assert.throws(
  () =>
    deriveProductionWriteSites(
      new Map([
        [
          ungatedTestsPath,
          'pub(crate) fn emit(path: &std::path::Path) {\n    let _ = std::fs::write(path, b"mutation control");\n}\n',
        ],
      ]),
    ),
  /unclassified production write: .*tests\/emit\.rs#.*emit/u,
  "an ungated production writer under a tests-named path must be RED",
);

const fullIdentifierProbePath = "rust/crates/omena-cli/src/full_identifier_probe.rs";
const fullIdentifierProbeSource =
  'struct First;\nimpl First {\n    fn emit(path: &std::path::Path) { let _ = std::fs::write(path, b"first"); }\n}\nstruct Second;\nimpl Second {\n    fn emit(path: &std::path::Path) { let _ = std::fs::write(path, b"second"); }\n}\n';
const fullIdentifierSites = deriveProductionWriteSites(
  new Map([[fullIdentifierProbePath, fullIdentifierProbeSource]]),
  [
    ...manifest.writeSites,
    {
      path: fullIdentifierProbePath,
      function: "crate::full_identifier_probe::First::emit",
      writeCount: 1,
      classification: "artifact",
      owner: "full identifier mutation control one",
    },
    {
      path: fullIdentifierProbePath,
      function: "crate::full_identifier_probe::Second::emit",
      writeCount: 1,
      classification: "artifact",
      owner: "full identifier mutation control two",
    },
  ],
).filter(({ path: sitePath }) => sitePath === fullIdentifierProbePath);
assert.deepEqual(
  fullIdentifierSites.map(({ function: functionName }) => functionName).toSorted(),
  ["crate::full_identifier_probe::First::emit", "crate::full_identifier_probe::Second::emit"],
  "write owners with the same short name must retain full Rust identifiers",
);

const productionAfterTestsSource = `${transactionSource}\nstruct PostTestProductionWriter;\nimpl PostTestProductionWriter {\n    fn write_product_bytes_after_tests(path: &std::path::Path) {\n        let _ = std::fs::write(path, b"mutation control");\n    }\n}\n`;
assert.throws(
  () => deriveProductionWriteSites(new Map([[transactionModulePath, productionAfterTestsSource]])),
  /unclassified production write: .*#.*write_product_bytes_after_tests/u,
  "a production impl method after cfg(test) must be RED",
);

const testOnlyWriteSource = `${transactionSource}\n#[cfg(test)]\nmod test_only_write_control {\n    fn write(path: &std::path::Path) {\n        let _ = std::fs::write(path, b"test-only control");\n    }\n}\n`;
assert.deepEqual(
  deriveProductionWriteSites(new Map([[transactionModulePath, testOnlyWriteSource]])).map(
    siteIdentity,
  ),
  derivedWriteSites.map(siteIdentity),
  "a test-only filesystem write must remain outside the production census",
);

const cfgTestFunctionPath = "rust/crates/omena-cli/src/cfg_test_function_probe.rs";
const cfgTestFunctionSource =
  '#[cfg(test)]\nfn test_only_write(path: &std::path::Path) {\n    let _ = std::fs::write(path, b"test-only control");\n}\n';
assert.deepEqual(
  deriveProductionWriteSites(new Map([[cfgTestFunctionPath, cfgTestFunctionSource]])).map(
    siteIdentity,
  ),
  derivedWriteSites.map(siteIdentity),
  "a cfg(test)-gated write function must remain outside the production census",
);

assert.deepEqual(
  manifest.consumerContracts.map(({ surface }) => surface),
  ["lint", "format", "minify", "migrate"],
);
assert.deepEqual(
  manifest.consumerContracts.map(({ writeKind }) => writeKind),
  ["lintFix", "formatting", "transform", "migrationPlan"],
);
assert.deepEqual(
  manifest.consumerContracts.map(({ additionalRequirement }) => additionalRequirement),
  [
    "sharedSafetyAssessment",
    "observedIdempotence",
    "appliedTransformDecisionWithoutBlockedOrRejected",
    "reviewedPlan",
  ],
);
assert.deepEqual(
  manifest.namedWaits.map(({ surface, condition }) => `${surface}:${condition}`),
  [
    "lint:routedSourceFix",
    "check:integratedCheckComposition",
    "source-edit:structuralSharingRevalidation",
  ],
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-write-safety",
      safetyAuthorityCount: safetyAuthorities.length,
      productionWriteSiteCount: derivedWriteSites.reduce(
        (count, { writeCount }) => count + writeCount,
        0,
      ),
      classifiedFunctionCount: derivedWriteSites.length,
      sourceMutationGateCount: 1,
      transactionWriteAuthorityCount: transactionWriteSites.length,
      sealedTransactionStagingFunctionCount: manifest.writeSites.filter(
        ({ classification }) => classification === "transaction-staging",
      ).length,
      productSourceBareWriteCount,
      productSourceWriteCallers: manifest.productSourceWriteCallers,
      consumerContractCount: manifest.consumerContracts.length,
      namedWaitCount: manifest.namedWaits.length,
    },
    null,
    2,
  )}\n`,
);

function deriveProductionWriteSites(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
  registeredWriteSites: readonly WriteSite[] = manifest.writeSites,
): WriteSite[] {
  validateWriteSites(registeredWriteSites);
  const derived = new Map<
    string,
    {
      path: string;
      function: string;
      shortFunction: string;
      writeCount: number;
      productSourceWriteCount: number;
      apis: Set<string>;
    }
  >();
  const observedNonFilesystemWrites = new Map<string, number>();
  const nonFilesystemByKey = new Map(
    nonFilesystemWriteSinks.map((sink) => [`${sink.path}#${sink.function}`, sink]),
  );
  const files = [...new Set([...rustSourceFiles(cliRoot), ...sourceOverrides.keys()])].toSorted();
  const cfgTestFiles = cfgDerivedTestFiles(files, sourceOverrides);

  for (const file of files) {
    if (cfgTestFiles.has(file)) continue;
    const source = productionRustSource(sourceOverrides.get(file) ?? read(file));
    const structuralSource = maskRustCommentsAndLiterals(source);
    const functions = namedFunctions(structuralSource, file);
    for (const mutation of filesystemMutations(source)) {
      const offset = mutation.offset;
      const owner = functions.findLast(({ start, end }) => start < offset && offset < end);
      assert.ok(owner, `${file} contains a production write primitive outside a named function`);
      const shortKey = `${file}#${owner.shortName}`;
      const key = `${file}#${owner.name}`;
      if (nonFilesystemByKey.has(shortKey)) {
        observedNonFilesystemWrites.set(
          shortKey,
          (observedNonFilesystemWrites.get(shortKey) ?? 0) + 1,
        );
        continue;
      }
      const current = derived.get(key) ?? {
        path: file,
        function: owner.name,
        shortFunction: owner.shortName,
        writeCount: 0,
        productSourceWriteCount: 0,
        apis: new Set<string>(),
      };
      current.writeCount += 1;
      current.apis.add(mutation.api);
      if (
        !isProvenNonProductDestination(shortKey, mutation, source.slice(owner.start, owner.end))
      ) {
        current.productSourceWriteCount += 1;
      }
      derived.set(key, current);
    }
  }

  for (const sink of nonFilesystemWriteSinks) {
    const key = `${sink.path}#${sink.function}`;
    assert.equal(
      observedNonFilesystemWrites.get(key),
      sink.writeCount,
      `non-filesystem write sink changed: ${key}`,
    );
    const source = productionRustSource(sourceOverrides.get(sink.path) ?? read(sink.path));
    const functions = namedFunctions(maskRustCommentsAndLiterals(source), sink.path);
    const index = functions.findIndex(({ shortName }) => shortName === sink.function);
    assert.ok(index >= 0, `non-filesystem write sink is missing: ${key}`);
    assert.ok(
      source.slice(functions[index]!.start, functions[index]!.end).includes(sink.evidence),
      `non-filesystem write sink lost its ${sink.evidence} evidence: ${key}`,
    );
  }

  return [...derived.values()].map((site) => {
    const registered = registeredSiteForFunction(registeredWriteSites, site.path, site.function);
    assert.ok(
      registered,
      `unclassified production write: ${site.path}#${site.function} [${[...site.apis].join(", ")}]`,
    );
    assertClassificationRole(registered, site.apis);
    return {
      path: site.path,
      function: registered.function,
      writeCount: site.writeCount,
      classification: registered.classification,
      owner: registered.owner,
      productSourceWriteCount: site.productSourceWriteCount,
    } as WriteSite & { readonly productSourceWriteCount: number };
  });
}

function registeredSiteForFunction(
  registeredWriteSites: readonly WriteSite[],
  file: string,
  rustIdentifier: string,
): WriteSite | undefined {
  const candidates = registeredWriteSites.filter(
    (site) => site.path === file && site.function === rustIdentifier,
  );
  assert.ok(
    candidates.length <= 1,
    `ambiguous write-site Rust identifier ${file}#${rustIdentifier}`,
  );
  return candidates[0];
}

function assertProductSourcePlaneZero(writeSites: readonly WriteSite[]): void {
  assert.equal(
    writeSites.filter(
      (site) =>
        ((site as WriteSite & { readonly productSourceWriteCount?: number })
          .productSourceWriteCount ?? 1) > 0 &&
        (site.classification === "artifact" || site.classification === "bookkeeping"),
    ).length,
    0,
    "product/source-plane bare writes must be zero",
  );
}

function nonProductDestinationAuthority(
  key: string,
): { readonly destinations: readonly RegExp[]; readonly evidence: readonly RegExp[] } | undefined {
  return new Map<
    string,
    { readonly destinations: readonly RegExp[]; readonly evidence: readonly RegExp[] }
  >([
    [
      "rust/crates/omena-cli/src/daemon.rs#write_endpoint",
      {
        destinations: [/^parent$/u, /^&?temporary$/u, /^path$/u],
        evidence: [/path\.with_extension\(/u, /fs::rename\(&temporary,\s*path\)/u],
      },
    ],
    [
      "rust/crates/omena-cli/src/daemon.rs#cleanup_endpoint",
      { destinations: [/^path$/u], evidence: [/fs::remove_file\(path\)/u] },
    ],
    ...[
      "lock_update",
      "lock_add",
      "lock_fetch_provenance",
      "lock_record_verification",
      "lock_verify_attestation",
    ].map(
      (functionName) =>
        [
          `rust/crates/omena-cli/src/lock.rs#${functionName}`,
          { destinations: [/^&?lockfile$/u], evidence: [/write_omena_lock_json_v1/u] },
        ] as const,
    ),
    [
      "rust/crates/omena-cli/src/lock.rs#write_recorded_shard_verdicts",
      {
        destinations: [/^verdict_dir\.as_path\(\)$/u],
        evidence: [/\.join\("\.cache"\)/u, /OMENA_SIF_SHARD_VERDICT_DIR_V1/u],
      },
    ],
    [
      "rust/crates/omena-cli/src/lock.rs#write_recorded_sigstore_bundle",
      {
        destinations: [/^bundle_dir\.as_path\(\)$/u],
        evidence: [/\.join\("\.cache"\)/u, /RECORDED_SIGSTORE_BUNDLE_DIR_V1/u],
      },
    ],
    [
      "rust/crates/omena-cli/src/lock.rs#publish_immutable_recorded_shard_verdict",
      {
        destinations: [/^<receiver-bound>$/u, /^temporary\.as_path\(\)$/u, /^path$/u],
        evidence: [/create_new\(true\)/u, /temporary/u],
      },
    ],
    [
      "rust/crates/omena-cli/src/workspace_edit_transaction.rs#prepare_rollback_backup",
      {
        destinations: [/^edit\.backup\.as_path\(\)$/u],
        evidence: [/rollback backup/u],
      },
    ],
    [
      "rust/crates/omena-cli/src/workspace_edit_transaction.rs#write_transaction_journal_file",
      {
        destinations: [/^<receiver-bound>$/u, /^path$/u],
        evidence: [/create_new\(true\)/u],
      },
    ],
    [
      "rust/crates/omena-cli/src/workspace_edit_transaction.rs#write_transaction_lock_file",
      {
        destinations: [/^<receiver-bound>$/u, /^lock_path$/u],
        evidence: [/create_new\(true\)/u],
      },
    ],
    [
      "rust/crates/omena-cli/src/workspace_edit_transaction.rs#drop",
      { destinations: [/^path$/u], evidence: [/self\.paths\.iter\(\)\.rev\(\)/u] },
    ],
    [
      "rust/crates/omena-cli/src/workspace_edit_transaction.rs#cleanup_staged",
      {
        destinations: [/^edit\.(?:stage|backup)\.as_path\(\)$/u],
        evidence: [/!edit\.backup_ready/u],
      },
    ],
    [
      "rust/crates/omena-cli/src/workspace_edit_transaction.rs#remove_if_exists",
      { destinations: [/^path$/u], evidence: [/remove transaction sidecar/u] },
    ],
  ]).get(key);
}

function isProvenNonProductDestination(
  key: string,
  mutation: FilesystemMutation,
  functionSource: string,
): boolean {
  const authority = nonProductDestinationAuthority(key);
  if (!authority) return false;
  assert.ok(
    authority.evidence.every((pattern) => pattern.test(functionSource)),
    `non-product destination evidence changed: ${key}`,
  );
  const destination = mutation.destination.replaceAll(/\s+/gu, "");
  return authority.destinations.some((pattern) => pattern.test(destination));
}

function assertClassificationRole(site: WriteSite, apis: ReadonlySet<string>): void {
  const key = `${site.path}#${site.function}`;
  if (site.classification === "directory-preparation") {
    assert.ok(
      [...apis].every((api) => api === "create_dir" || api === "create_dir_all"),
      `directory-preparation role cannot own byte or replacement APIs: ${key}`,
    );
  }
  if (site.classification === "transaction-staging") {
    assert.ok(
      new Set([
        `${transactionModulePath}#crate::workspace_edit_transaction::WorkspaceEditTransaction::rename_all`,
        `${transactionModulePath}#crate::workspace_edit_transaction::write_staged_product_bytes`,
      ]).has(key),
      `transaction-staging role is sealed: ${key}`,
    );
  }
  if (site.classification === "transaction-rollback") {
    assert.equal(
      key,
      `${transactionModulePath}#crate::workspace_edit_transaction::rollback_and_cleanup`,
      `transaction-rollback role is sealed: ${key}`,
    );
  }
}

function filesystemMutations(source: string): FilesystemMutation[] {
  const structural = maskRustCommentsAndLiterals(source);
  const mutations: FilesystemMutation[] = [];
  const seen = new Set<string>();
  const add = (api: string, offset: number, open: number): void => {
    const destinationIndex = new Set([
      "copy",
      "rename",
      "hard_link",
      "symlink",
      "symlink_file",
      "symlink_dir",
    ]).has(api)
      ? 1
      : 0;
    const destination =
      api === "options" || api === "new" || api.startsWith("method:")
        ? "<receiver-bound>"
        : callArgument(source, open, destinationIndex);
    const identity = `${offset}:${api}`;
    if (!seen.has(identity)) {
      seen.add(identity);
      mutations.push({ api: api.replace(/^method:/u, ""), destination, offset });
    }
  };

  for (const match of structural.matchAll(directFilesystemMutation)) {
    const text = match[0];
    const api =
      text.match(
        /(?:fs::|io::|\.)(write_all|write|copy|rename|hard_link|symlink(?:_file|_dir)?|remove_file|remove_dir_all|remove_dir|create_dir_all|create_dir|set_permissions|set_len|truncate|create_new|create|append)/u,
      )?.[1] ?? text.match(/(?:File|OpenOptions)::(create_new|create|options|new)/u)?.[1];
    assert.ok(api, `unclassified filesystem API spelling: ${text}`);
    const offset = match.index ?? -1;
    add(text.startsWith(".") ? `method:${api}` : api, offset, offset + text.lastIndexOf("("));
  }

  for (const [localName, api] of importedFilesystemMutationAliases(structural)) {
    const call = new RegExp(`(?<![:\\w])${escapeRegExp(localName)}\\s*\\(`, "gu");
    for (const match of structural.matchAll(call)) {
      const offset = match.index ?? -1;
      add(api, offset, offset + match[0].lastIndexOf("("));
    }
  }
  for (const [localName, kind] of importedFilesystemNamespaceAliases(structural)) {
    const methods =
      kind === "os-fs"
        ? "symlink|symlink_file|symlink_dir"
        : kind === "io"
          ? "copy"
          : "write|copy|rename|hard_link|remove_file|remove_dir|remove_dir_all|create_dir|create_dir_all|set_permissions";
    const call = new RegExp(`\\b${escapeRegExp(localName)}::(${methods})\\s*\\(`, "gu");
    for (const match of structural.matchAll(call)) {
      const offset = match.index ?? -1;
      add(match[1]!, offset, offset + match[0].lastIndexOf("("));
    }
  }
  for (const [localName, api] of importedFilesystemTypeAliases(structural)) {
    const call = new RegExp(`\\b${escapeRegExp(localName)}::(${api})\\s*\\(`, "gu");
    for (const match of structural.matchAll(call)) {
      const offset = match.index ?? -1;
      add(match[1]!, offset, offset + match[0].lastIndexOf("("));
    }
  }
  return mutations.toSorted((left, right) => left.offset - right.offset);
}

function importedFilesystemMutationAliases(source: string): Map<string, string> {
  const aliases = new Map<string, string>();
  const supported = new Set([
    "write",
    "copy",
    "rename",
    "hard_link",
    "remove_file",
    "remove_dir",
    "remove_dir_all",
    "create_dir",
    "create_dir_all",
    "set_permissions",
  ]);
  for (const match of source.matchAll(
    /\buse\s+std::fs::([a-z_]+)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/gu,
  )) {
    if (supported.has(match[1]!)) aliases.set(match[2] ?? match[1]!, match[1]!);
  }
  for (const match of source.matchAll(/\buse\s+std::fs::\{([^}]*)\}\s*;/gu)) {
    for (const entry of match[1]!.split(",")) {
      const parts = entry.trim().match(/^([a-z_]+)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/u);
      if (parts && supported.has(parts[1]!)) aliases.set(parts[2] ?? parts[1]!, parts[1]!);
    }
  }
  for (const match of source.matchAll(
    /\buse\s+std::io::copy(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/gu,
  )) {
    aliases.set(match[1] ?? "copy", "copy");
  }
  for (const match of source.matchAll(/\buse\s+std::io::\{([^}]*)\}\s*;/gu)) {
    for (const entry of match[1]!.split(",")) {
      const parts = entry.trim().match(/^copy(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/u);
      if (parts) aliases.set(parts[1] ?? "copy", "copy");
    }
  }
  for (const match of source.matchAll(
    /\buse\s+std::os::[a-z_]+::fs::(symlink|symlink_file|symlink_dir)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/gu,
  )) {
    aliases.set(match[2] ?? match[1]!, match[1]!);
  }
  for (const match of source.matchAll(/\buse\s+std::os::[a-z_]+::fs::\{([^}]*)\}\s*;/gu)) {
    for (const entry of match[1]!.split(",")) {
      const parts = entry
        .trim()
        .match(/^(symlink|symlink_file|symlink_dir)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/u);
      if (parts) aliases.set(parts[2] ?? parts[1]!, parts[1]!);
    }
  }
  return aliases;
}

function importedFilesystemNamespaceAliases(source: string): Map<string, "fs" | "io" | "os-fs"> {
  const aliases = new Map<string, "fs" | "io" | "os-fs">();
  for (const match of source.matchAll(
    /\buse\s+std::fs(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/gu,
  )) {
    aliases.set(match[1] ?? "fs", "fs");
  }
  for (const match of source.matchAll(
    /\buse\s+std::os::[a-z_]+::fs(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/gu,
  )) {
    aliases.set(match[1] ?? "fs", "os-fs");
  }
  for (const match of source.matchAll(
    /\buse\s+std::io(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/gu,
  )) {
    aliases.set(match[1] ?? "io", "io");
  }
  for (const match of source.matchAll(/\buse\s+std::fs::\{([^}]*)\}\s*;/gu)) {
    for (const entry of match[1]!.split(",")) {
      const parts = entry.trim().match(/^self(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/u);
      if (parts) aliases.set(parts[1] ?? "fs", "fs");
    }
  }
  for (const match of source.matchAll(/\buse\s+std::\{([^}]*)\}\s*;/gu)) {
    for (const entry of match[1]!.split(",")) {
      const parts = entry.trim().match(/^(fs|io)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/u);
      if (parts) aliases.set(parts[2] ?? parts[1]!, parts[1] as "fs" | "io");
    }
  }
  return aliases;
}

function importedFilesystemTypeAliases(source: string): Map<string, string> {
  const aliases = new Map<string, string>();
  for (const match of source.matchAll(
    /\buse\s+std::fs::(File|OpenOptions)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/gu,
  )) {
    aliases.set(match[2] ?? match[1]!, match[1] === "File" ? "create|create_new|options" : "new");
  }
  for (const match of source.matchAll(/\buse\s+std::fs::\{([^}]*)\}\s*;/gu)) {
    for (const entry of match[1]!.split(",")) {
      const parts = entry
        .trim()
        .match(/^(File|OpenOptions)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/u);
      if (parts) {
        aliases.set(
          parts[2] ?? parts[1]!,
          parts[1] === "File" ? "create|create_new|options" : "new",
        );
      }
    }
  }
  return aliases;
}

function callArgument(source: string, open: number, index: number): string {
  let depth = 0;
  let argumentStart = open + 1;
  let argumentIndex = 0;
  for (let cursor = open + 1; cursor < source.length; cursor += 1) {
    const current = source[cursor]!;
    if (current === "(" || current === "[" || current === "{") depth += 1;
    else if (current === ")" || current === "]" || current === "}") {
      if (current === ")" && depth === 0) {
        return argumentIndex === index ? source.slice(argumentStart, cursor).trim() : "<missing>";
      }
      depth -= 1;
    } else if (current === "," && depth === 0) {
      if (argumentIndex === index) return source.slice(argumentStart, cursor).trim();
      argumentIndex += 1;
      argumentStart = cursor + 1;
    }
  }
  return "<unterminated>";
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function assertDestinationKeyedPostconditions(
  transaction: string,
  minify: string,
  build: string,
): void {
  assert.match(
    transaction,
    /\(postcondition\.check\)\(edit\.path\.as_path\(\),\s*staged_content\.as_slice\(\)\)/u,
    "postconditions must receive the actual edit destination",
  );
  assert.doesNotMatch(
    `${transaction}\n${minify}\n${build}`,
    /\btext_reparse_for_path\s*\(/u,
    "input-keyed postcondition APIs are forbidden",
  );
  assert.doesNotMatch(
    build,
    /style_reparse_for_admitted_output\(\s*(?:input|path|output_path)(?:\.as_path\(\))?\s*,/u,
    "input-keyed admitted-output postconditions are forbidden",
  );
  assert.match(
    minify,
    /text_reparse_for_destination\(\)/u,
    "minify must use destination-bound reparsing",
  );
}

function assertNoSemanticSafetyCopies(files: readonly string[]): void {
  const auto = new Set(["safe", "automatic", "autoapply", "autowrite"]);
  const optIn = new Set(["conservative", "optin", "explicitapproval"]);
  const manual = new Set(["manualreview", "manual", "reviewonly"]);
  const copies: string[] = [];

  for (const file of files) {
    const source = read(file);
    for (const declaration of enumDeclarations(source)) {
      const normalized = declaration.variants.map((variant) => variant.toLowerCase());
      if (
        normalized.some((variant) => auto.has(variant)) &&
        normalized.some((variant) => optIn.has(variant)) &&
        normalized.some((variant) => manual.has(variant)) &&
        !(declaration.name === "FixSafetyV0" && file.endsWith("/omena-checker/src/fix_safety.rs"))
      ) {
        copies.push(`${file}:${declaration.name}`);
      }
    }
  }
  assert.deepEqual(
    copies,
    [],
    `semantic write-safety enum copies are forbidden: ${copies.join(", ")}`,
  );
}

function assertNoTypeScriptSafetyCopies(): void {
  const copies = sourceFiles(
    ["packages", "server", "client"],
    [".ts", ".tsx", ".js", ".cjs", ".mjs"],
  ).filter((file) => {
    const source = read(file);
    return (
      /["']safe["']/u.test(source) &&
      /["']conservative["']/u.test(source) &&
      /["']manualReview["']/u.test(source)
    );
  });
  assert.deepEqual(
    copies,
    [],
    `TypeScript write-safety copies are forbidden: ${copies.join(", ")}`,
  );
}

function enumDeclarations(source: string): { name: string; variants: string[] }[] {
  const code = maskRustCommentsAndLiterals(source);
  const declarations: { name: string; variants: string[] }[] = [];
  for (const match of code.matchAll(/\benum\s+([A-Z][A-Za-z0-9_]*)\s*\{/gu)) {
    const bodyStart = (match.index ?? 0) + match[0].length;
    const bodyEnd = matchingBrace(code, bodyStart - 1, `enum ${match[1]}`);
    const variants = code
      .slice(bodyStart, bodyEnd)
      .split("\n")
      .flatMap((line) => line.match(/^\s*([A-Z][A-Za-z0-9_]*)\s*(?:,|\{|\()/u)?.slice(1) ?? []);
    declarations.push({ name: match[1]!, variants });
  }
  return declarations;
}

function maskRustCommentsAndLiterals(source: string): string {
  const masked = source.split("");
  let index = 0;

  const blank = (start: number, end: number): void => {
    for (let cursor = start; cursor < end; cursor += 1) {
      if (masked[cursor] !== "\n" && masked[cursor] !== "\r") masked[cursor] = " ";
    }
  };

  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      const stop = end < 0 ? source.length : end;
      blank(index, stop);
      index = stop;
      continue;
    }
    if (source.startsWith("/*", index)) {
      const start = index;
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      blank(start, index);
      continue;
    }

    const rawPrefix = source.slice(index).match(/^(?:br|r)(#*)"/u);
    if (rawPrefix) {
      const start = index;
      const terminator = `"${rawPrefix[1] ?? ""}`;
      index += rawPrefix[0].length;
      const end = source.indexOf(terminator, index);
      index = end < 0 ? source.length : end + terminator.length;
      blank(start, index);
      continue;
    }

    if (source[index] === '"') {
      const start = index;
      index += 1;
      let escaped = false;
      while (index < source.length) {
        const current = source[index]!;
        index += 1;
        if (escaped) escaped = false;
        else if (current === "\\") escaped = true;
        else if (current === '"') break;
      }
      blank(start, index);
      continue;
    }

    const characterLiteral = source
      .slice(index)
      .match(/^'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]{1,6}\}|.)|[^'\\\r\n])'/u);
    if (characterLiteral) {
      const start = index;
      index += characterLiteral[0].length;
      blank(start, index);
      continue;
    }

    index += 1;
  }

  return masked.join("");
}

function extractEnumVariants(source: string, name: string): string[] {
  const declaration = enumDeclarations(source).find((candidate) => candidate.name === name);
  assert.ok(declaration, `missing enum ${name}`);
  return declaration.variants;
}

function productionRustSource(source: string): string {
  const structural = maskRustCommentsAndLiterals(source);
  const spans: { start: number; end: number }[] = [];
  const testAttribute = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/gu;
  for (const match of structural.matchAll(testAttribute)) {
    const start = match.index ?? 0;
    let cursor = start + match[0].length;
    while (true) {
      cursor = skipWhitespace(structural, cursor);
      if (structural[cursor] !== "#") break;
      const close = structural.indexOf("]", cursor + 1);
      if (close < 0) break;
      cursor = close + 1;
    }
    cursor = skipWhitespace(structural, cursor);
    const keyword = structural.slice(cursor).match(/^([A-Za-z_][A-Za-z0-9_]*)/u)?.[1] ?? "";
    const blockItems = new Set([
      "fn",
      "mod",
      "impl",
      "struct",
      "enum",
      "if",
      "match",
      "while",
      "for",
      "loop",
    ]);
    if (blockItems.has(keyword)) {
      const open = structural.indexOf("{", cursor);
      const semicolon = structural.indexOf(";", cursor);
      if (semicolon >= 0 && (open < 0 || semicolon < open)) {
        spans.push({ start, end: semicolon + 1 });
        continue;
      }
      assert.ok(open >= 0, `cfg(test) ${keyword} is missing its body`);
      spans.push({ start, end: matchingBrace(structural, open, `cfg(test) ${keyword}`) + 1 });
      continue;
    }
    const semicolon = structural.indexOf(";", cursor);
    const nextBrace = structural.indexOf("{", cursor);
    if (semicolon >= 0 && (nextBrace < 0 || semicolon < nextBrace)) {
      spans.push({ start, end: semicolon + 1 });
    } else {
      spans.push({ start, end: cursor });
    }
  }
  if (spans.length === 0) return source;
  const chars = source.split("");
  for (const { start, end } of spans) {
    for (let index = start; index < end; index += 1) {
      if (chars[index] !== "\n" && chars[index] !== "\r") chars[index] = " ";
    }
  }
  return chars.join("");
}

function skipWhitespace(source: string, start: number): number {
  let cursor = start;
  while (/\s/u.test(source[cursor] ?? "")) cursor += 1;
  return cursor;
}

function matchingBrace(source: string, open: number, label = "source"): number {
  let depth = 0;
  let quote: string | null = null;
  let escaped = false;
  let lineComment = false;
  let blockCommentDepth = 0;

  for (let index = open; index < source.length; index += 1) {
    const current = source[index]!;
    const next = source[index + 1] ?? "";
    if (lineComment) {
      if (current === "\n") lineComment = false;
      continue;
    }
    if (blockCommentDepth > 0) {
      if (current === "/" && next === "*") {
        blockCommentDepth += 1;
        index += 1;
      } else if (current === "*" && next === "/") {
        blockCommentDepth -= 1;
        index += 1;
      }
      continue;
    }
    if (quote) {
      if (escaped) escaped = false;
      else if (current === "\\") escaped = true;
      else if (current === quote) quote = null;
      continue;
    }
    if (current === "/" && next === "/") {
      lineComment = true;
      index += 1;
    } else if (current === "/" && next === "*") {
      blockCommentDepth = 1;
      index += 1;
    } else if (current === '"') {
      quote = current;
    } else if (current === "{") {
      depth += 1;
    } else if (current === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  assert.fail(`${label} has an unterminated brace-delimited block`);
}

function assertSourceGateRoutesToTransaction(source: string, functionName: string): void {
  const functionSource = topLevelFunctionSource(source, functionName);
  const constructorOffset = functionSource.indexOf("WorkspaceEditTransaction::new(");
  const commitOffset = functionSource.indexOf(".commit()", constructorOffset);
  assert.ok(
    constructorOffset >= 0 && commitOffset > constructorOffset,
    `${functionName} must route to WorkspaceEditTransaction::new(...).commit()`,
  );
  assert.equal(
    filesystemMutations(functionSource).length,
    0,
    `${functionName} must authorize transaction commit without a direct filesystem primitive`,
  );
}

function topLevelFunctionSource(source: string, functionName: string): string {
  const functions = namedFunctions(maskRustCommentsAndLiterals(source));
  const index = functions.findIndex(
    ({ name, shortName }) => name === functionName || shortName === functionName,
  );
  assert.ok(index >= 0, `missing top-level function ${functionName}`);
  return source.slice(functions[index]!.start, functions[index]!.end);
}

function namedFunctions(source: string, file = "crate.rs"): NamedFunction[] {
  const functions: NamedFunction[] = [];
  const implementations: { label: string; start: number; end: number }[] = [];
  for (const match of source.matchAll(/\bimpl\b([^;{]*)\{/gu)) {
    const start = match.index ?? 0;
    const open = start + match[0].lastIndexOf("{");
    const header = (match[1] ?? "").trim();
    const target = header.includes(" for ")
      ? header.slice(header.lastIndexOf(" for ") + " for ".length)
      : (header.replace(/^<[^>]*>\s*/u, "").split(/\s+where\s+|\s+/u)[0] ?? "impl");
    implementations.push({
      label: target.replace(/<.*$/u, "").replace(/^.*::/u, "") || "impl",
      start,
      end: matchingBrace(source, open, `impl ${target}`) + 1,
    });
  }
  const moduleIdentifier = rustModuleIdentifier(file);
  const declaration =
    /^[\t ]*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+([a-z][a-z0-9_]*)[^;{]*\{/gmu;
  for (const match of source.matchAll(declaration)) {
    const start = match.index ?? 0;
    const open = start + match[0].lastIndexOf("{");
    const end = matchingBrace(source, open, `function ${match[1]}`) + 1;
    const implementation = implementations.findLast(
      (candidate) => candidate.start < start && end < candidate.end,
    );
    const shortName = match[1]!;
    functions.push({
      name: `${moduleIdentifier}::${implementation ? `${implementation.label}::` : ""}${shortName}`,
      shortName,
      start,
      end,
    });
  }
  return functions;
}

function rustModuleIdentifier(file: string): string {
  const relative = file.startsWith(`${cliRoot}/`) ? file.slice(cliRoot.length + 1) : file;
  const withoutExtension = relative.replace(/\.rs$/u, "");
  const segments = withoutExtension.split("/");
  if (["lib", "main", "mod"].includes(segments.at(-1) ?? "")) segments.pop();
  return segments.length === 0 ? "crate" : `crate::${segments.join("::")}`;
}

function siteIdentity(site: WriteSite): string {
  return [site.path, site.function, site.writeCount, site.classification, site.owner].join("|");
}

function rustSourceFiles(root: string): string[] {
  return sourceFiles([root], [".rs"]);
}

function cfgDerivedTestFiles(
  files: readonly string[],
  sourceOverrides: ReadonlyMap<string, string>,
): ReadonlySet<string> {
  const available = new Set(files);
  const derived = new Set<string>();
  for (const file of files) {
    const source = sourceOverrides.get(file) ?? read(file);
    const structural = maskRustCommentsAndLiterals(source);
    for (const match of structural.matchAll(
      /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
    )) {
      const moduleName = match[1]!;
      const directory = path.posix.dirname(file);
      const stem = path.posix.basename(file, ".rs");
      const moduleRoot = new Set(
        stem === "lib" || stem === "main" || stem === "mod"
          ? [path.posix.join(directory, moduleName)]
          : [path.posix.join(directory, stem, moduleName)],
      );
      if (stem === "mod") moduleRoot.add(path.posix.join(directory, moduleName));
      for (const root of moduleRoot) {
        const direct = `${root}.rs`;
        const nested = `${root}/mod.rs`;
        if (available.has(direct)) derived.add(direct);
        if (available.has(nested)) derived.add(nested);
      }
    }
  }
  let changed = true;
  while (changed) {
    changed = false;
    for (const file of [...derived]) {
      const source = sourceOverrides.get(file) ?? read(file);
      const structural = maskRustCommentsAndLiterals(source);
      for (const match of structural.matchAll(/\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/gu)) {
        const moduleName = match[1]!;
        const directory = path.posix.dirname(file);
        const stem = path.posix.basename(file, ".rs");
        const root =
          stem === "mod"
            ? path.posix.join(directory, moduleName)
            : path.posix.join(directory, stem, moduleName);
        for (const candidate of [`${root}.rs`, `${root}/mod.rs`]) {
          if (available.has(candidate) && !derived.has(candidate)) {
            derived.add(candidate);
            changed = true;
          }
        }
      }
    }
  }
  return derived;
}

function sourceFiles(roots: readonly string[], extensions: readonly string[]): string[] {
  return roots
    .flatMap((root) => walk(root))
    .filter((file) => extensions.some((extension) => file.endsWith(extension)))
    .toSorted();
}

function walk(relativeRoot: string): string[] {
  const absoluteRoot = path.join(repoRoot, relativeRoot);
  return evidenceScanSurface.readdirSync(absoluteRoot, { withFileTypes: true }).flatMap((entry) => {
    const relativePath = path.posix.join(relativeRoot, entry.name);
    return entry.isDirectory() ? walk(relativePath) : [relativePath];
  });
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}

function readWriteSafetyManifest(relativePath: string): WriteSafetyManifest {
  const candidate = readJson<unknown>(relativePath);
  assert.ok(candidate && typeof candidate === "object", "write-safety manifest must be an object");
  const parsed = candidate as WriteSafetyManifest;
  assert.ok(Array.isArray(parsed.writeSites), "write-safety manifest writeSites must be an array");
  validateWriteSites(parsed.writeSites);
  return parsed;
}

function validateWriteSites(writeSites: readonly WriteSite[]): void {
  for (const site of writeSites) {
    assert.ok(
      writeClassifications.includes(site.classification),
      `unknown write classification: ${String(site.classification) || "<empty>"}`,
    );
    assert.ok(site.path.length > 0, "write-site path must not be empty");
    assert.ok(site.function.length > 0, "write-site Rust identifier must not be empty");
    assert.ok(
      Number.isInteger(site.writeCount) && site.writeCount > 0,
      "writeCount must be positive",
    );
    assert.ok(site.owner.length > 0, "write-site owner must not be empty");
  }
}

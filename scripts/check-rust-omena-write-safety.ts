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
const sanctionedProductDestinationCalls = [
  "crate::daemon::watch_endpoint_path",
  "crate::workspace_edit_transaction::sidecar_path",
  "crate::workspace_edit_transaction::journal_path",
  "crate::workspace_edit_transaction::transaction_lock_path",
] as const;
const pathPreservingMethods = new Set([
  "as_path",
  "as_ref",
  "clone",
  "iter",
  "join",
  "ok_or_else",
  "parent",
  "rev",
  "to_path_buf",
  "unwrap_or",
  "unwrap_or_else",
  "with_extension",
]);
let productDestinationGraphCache: ProductDestinationGraph | undefined;
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
const destinationFixtureOptions = {
  modulePath: "fixture::authority",
  sanctionedCalls: ["fixture::authority::sidecar_path"],
  trustedFields: new Set(["fixture::authority::TrustedSidecar.backup"]),
} as const;
assertDestinationFixtureAllowed(
  "a fully resolved sanctioned constructor",
  `fn emit(product: &Path) {
    let destination = fixture::authority::sidecar_path(product);
    let _ = std::fs::write(destination, b"control");
  }`,
  destinationFixtureOptions,
);
assertDestinationFixtureAllowed(
  "a type-resolved trusted field",
  `fn emit(sidecar: TrustedSidecar) {
    let _ = std::fs::write(sidecar.backup, b"control");
  }`,
  destinationFixtureOptions,
);
for (const [label, fixture] of [
  [
    "an arbitrary function parameter",
    `fn emit(path: &Path) {
      let _ = std::fs::write(path, b"control");
    }`,
  ],
  [
    "a local same-name constructor",
    `fn emit(product: &Path) {
      fn sidecar_path(_: &Path) -> PathBuf { PathBuf::from("decoy") }
      let destination = sidecar_path(product);
      let _ = std::fs::write(destination, b"control");
    }`,
  ],
  [
    "a foreign same-name field",
    `fn emit(sidecar: ForeignSidecar) {
      let _ = std::fs::write(sidecar.backup, b"control");
    }`,
  ],
  [
    "a mixed conditional destination",
    `fn emit(product: &Path, choose_product: bool) {
      let sidecar = fixture::authority::sidecar_path(product);
      let destination = if choose_product { product.to_path_buf() } else { sidecar };
      let _ = std::fs::write(destination, b"control");
    }`,
  ],
  [
    "a tuple binding",
    `fn emit(product: &Path) {
      let (destination, marker) = (fixture::authority::sidecar_path(product), 0);
      let _ = marker;
      let _ = std::fs::write(destination, b"control");
    }`,
  ],
  [
    "a let-else binding",
    `fn emit(product: &Path) {
      let Some(destination) = Some(fixture::authority::sidecar_path(product)) else { return; };
      let _ = std::fs::write(destination, b"control");
    }`,
  ],
  [
    "a destructuring assignment",
    `fn emit(product: &Path) {
      let mut destination = product.to_path_buf();
      let mut marker = 0;
      (destination, marker) = (fixture::authority::sidecar_path(product), 1);
      let _ = marker;
      let _ = std::fs::write(destination, b"control");
    }`,
  ],
  [
    "an inner-block binding leak",
    `fn emit(product: &Path) {
      let destination = product.to_path_buf();
      { let destination = fixture::authority::sidecar_path(product); let _ = destination; }
      let _ = std::fs::write(destination, b"control");
    }`,
  ],
] as const) {
  assertDestinationFixtureDenied(label, fixture, destinationFixtureOptions);
}
assertDestinationFixtureDenied(
  "a statement macro assignment",
  `fn emit(product: &Path) {
    macro_rules! assign_destination { ($target:ident, $value:expr) => { $target = $value; }; }
    let mut destination = product.to_path_buf();
    assign_destination!(destination, fixture::authority::sidecar_path(product));
    let _ = std::fs::write(destination, b"control");
  }`,
  destinationFixtureOptions,
  "statement-macro:assign_destination",
);
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
        !isProvenNonProductDestination(
          key,
          { ...mutation, offset: mutation.offset - owner.start },
          source.slice(owner.start, owner.end),
        )
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

interface DestinationTransition {
  readonly offset: number;
  readonly kind: "let" | "assign";
  readonly pattern: string;
  readonly expression: string;
  readonly letElse: boolean;
  readonly controlPattern?: "if-let" | "for";
}

interface DestinationDataflow {
  readonly derivedByMutation: ReadonlyMap<string, boolean>;
  readonly unparsedConstructs: readonly string[];
  readonly calls: readonly {
    readonly name: string;
    readonly argumentsDerived: readonly boolean[];
  }[];
}

interface DestinationCall {
  readonly offset: number;
  readonly name: string;
  readonly arguments: readonly string[];
}

function destinationMutationIdentity(mutation: FilesystemMutation): string {
  return `${mutation.offset}:${mutation.api}`;
}

function simpleBindingPattern(pattern: string): string | undefined {
  const normalized = pattern
    .trim()
    .replace(/^ref\s+mut\s+/u, "")
    .replace(/^ref\s+/u, "")
    .replace(/^mut\s+/u, "")
    .split(/\s*:\s*/u)[0]!
    .trim();
  return /^[A-Za-z_][A-Za-z0-9_]*$/u.test(normalized) ? normalized : undefined;
}

function topLevelAssignment(source: string): number {
  let round = 0;
  let square = 0;
  let brace = 0;
  for (let cursor = 0; cursor < source.length; cursor += 1) {
    const current = source[cursor]!;
    if (current === "(") round += 1;
    else if (current === ")") round -= 1;
    else if (current === "[") square += 1;
    else if (current === "]") square -= 1;
    else if (current === "{") brace += 1;
    else if (current === "}") brace -= 1;
    else if (
      current === "=" &&
      round === 0 &&
      square === 0 &&
      brace === 0 &&
      source[cursor - 1] !== "=" &&
      source[cursor - 1] !== "!" &&
      source[cursor - 1] !== "<" &&
      source[cursor - 1] !== ">" &&
      source[cursor + 1] !== "=" &&
      source[cursor + 1] !== ">"
    ) {
      return cursor;
    }
  }
  return -1;
}

function matchingRustDelimiter(
  source: string,
  open: number,
  opening: "(" | "[" | "{",
  closing: ")" | "]" | "}",
): number {
  assert.equal(source[open], opening, `expected ${opening} at Rust delimiter start`);
  let depth = 0;
  for (let cursor = open; cursor < source.length; cursor += 1) {
    if (source[cursor] === opening) depth += 1;
    else if (source[cursor] === closing) {
      depth -= 1;
      if (depth === 0) return cursor;
    }
  }
  throw new Error(`unterminated Rust delimiter ${opening}${closing}`);
}

function statementEnd(source: string, start: number): number {
  let round = 0;
  let square = 0;
  let brace = 0;
  for (let cursor = start; cursor < source.length; cursor += 1) {
    const current = source[cursor]!;
    if (current === "(") round += 1;
    else if (current === ")") round -= 1;
    else if (current === "[") square += 1;
    else if (current === "]") square -= 1;
    else if (current === "{") brace += 1;
    else if (current === "}") brace -= 1;
    else if (current === ";" && round === 0 && square === 0 && brace === 0) return cursor;
  }
  return source.length;
}

function destinationTransitions(source: string): DestinationTransition[] {
  const structural = maskRustCommentsAndLiterals(source);
  const rows: DestinationTransition[] = [];
  const occupied = new Array<boolean>(source.length).fill(false);
  for (const match of structural.matchAll(/\blet\b/gu)) {
    const start = match.index ?? 0;
    const prefix = structural.slice(Math.max(0, start - 12), start);
    if (/\b(?:if|while)\s*$/u.test(prefix)) continue;
    const end = statementEnd(structural, start);
    const declaration = structural.slice(start + 3, end);
    const equal = topLevelAssignment(declaration);
    if (equal < 0) continue;
    const pattern = declaration.slice(0, equal).trim();
    let expression = declaration.slice(equal + 1).trim();
    const letElseMatch = expression.match(/^([\s\S]*?)\s+else\s*\{/u);
    const letElse = Boolean(letElseMatch);
    if (letElseMatch) expression = letElseMatch[1]!.trim();
    rows.push({ offset: end, kind: "let", pattern, expression, letElse });
    for (let cursor = start; cursor <= Math.min(end, source.length - 1); cursor += 1) {
      occupied[cursor] = true;
    }
  }
  for (const match of structural.matchAll(/\bif\s+let\s+([^=;{}]+?)\s*=\s*([^;{}]+?)\s*\{/gu)) {
    const open = (match.index ?? 0) + match[0].lastIndexOf("{");
    rows.push({
      offset: open,
      kind: "let",
      pattern: match[1]!.trim(),
      expression: match[2]!.trim(),
      letElse: false,
      controlPattern: "if-let",
    });
  }
  for (const match of structural.matchAll(/\bfor\s+([^=;{}]+?)\s+in\s+([^;{}]+?)\s*\{/gu)) {
    const open = (match.index ?? 0) + match[0].lastIndexOf("{");
    rows.push({
      offset: open,
      kind: "let",
      pattern: match[1]!.trim(),
      expression: match[2]!.trim(),
      letElse: false,
      controlPattern: "for",
    });
  }
  for (const match of structural.matchAll(/(?:^|[;{}])\s*([^;{}=]+?)\s*(?<![=!<>])=(?!=|>)/gmu)) {
    const start = (match.index ?? 0) + match[0].lastIndexOf(match[1]!);
    if (occupied[start]) continue;
    const end = statementEnd(structural, start);
    const statement = structural.slice(start, end);
    const equal = topLevelAssignment(statement);
    if (equal < 0) continue;
    rows.push({
      offset: end,
      kind: "assign",
      pattern: statement.slice(0, equal).trim(),
      expression: statement.slice(equal + 1).trim(),
      letElse: false,
    });
  }
  return rows.toSorted((left, right) => left.offset - right.offset);
}

function patternBindings(pattern: string): string[] {
  return [...pattern.matchAll(/\b([a-z][A-Za-z0-9_]*)\b/gu)]
    .map((match) => match[1]!)
    .filter((name) => !new Set(["mut", "ref", "self", "crate", "super"]).has(name));
}

function functionParameterBindings(source: string): string[] {
  const structural = maskRustCommentsAndLiterals(source);
  const fn = structural.match(/\bfn\s+[a-z][a-z0-9_]*\s*(?:<[^;{]*?>\s*)?\(/u);
  if (fn?.index === undefined) return [];
  const open = fn.index + fn[0].lastIndexOf("(");
  let close: number;
  try {
    close = matchingRustDelimiter(structural, open, "(", ")");
  } catch {
    return [];
  }
  return splitTopLevelArguments(structural.slice(open + 1, close)).flatMap((parameter) => {
    const beforeType = parameter.split(":")[0]!.trim();
    if (/^(?:&\s*)?(?:mut\s+)?self$/u.test(beforeType)) return ["self"];
    const binding = simpleBindingPattern(beforeType);
    return binding ? [binding] : [];
  });
}

function resolvedRustTypeIdentity(
  typeSource: string,
  modulePath: string,
  ownerType?: string,
): string | undefined {
  const candidates = [
    ...maskRustCommentsAndLiterals(typeSource).matchAll(
      /(?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Z][A-Za-z0-9_]*/gu,
    ),
  ].map((match) => match[0]!);
  const candidate = candidates.at(-1);
  if (!candidate) return undefined;
  if (candidate === "Self") return ownerType;
  return candidate.includes("::") ? candidate : `${modulePath}::${candidate}`;
}

function functionOwnerType(functionIdentity: string, modulePath: string): string | undefined {
  const relative = functionIdentity.startsWith(`${modulePath}::`)
    ? functionIdentity.slice(modulePath.length + 2)
    : functionIdentity;
  const owner = relative.split("::").at(-2);
  return owner && /^[A-Z]/u.test(owner) ? `${modulePath}::${owner}` : undefined;
}

function functionParameterTypes(
  source: string,
  modulePath: string,
  ownerType?: string,
): ReadonlyMap<string, string> {
  const structural = maskRustCommentsAndLiterals(source);
  const fn = structural.match(/\bfn\s+[a-z][a-z0-9_]*\s*(?:<[^;{]*?>\s*)?\(/u);
  if (fn?.index === undefined) return new Map();
  const open = fn.index + fn[0].lastIndexOf("(");
  let close: number;
  try {
    close = matchingRustDelimiter(structural, open, "(", ")");
  } catch {
    return new Map();
  }
  const types = new Map<string, string>();
  for (const parameter of splitTopLevelArguments(structural.slice(open + 1, close))) {
    const trimmed = parameter.trim();
    if (/^(?:&\s*)?(?:mut\s+)?self$/u.test(trimmed)) {
      if (ownerType) types.set("self", ownerType);
      continue;
    }
    const colon = trimmed.indexOf(":");
    if (colon < 0) continue;
    const binding = simpleBindingPattern(trimmed.slice(0, colon));
    const type = resolvedRustTypeIdentity(trimmed.slice(colon + 1), modulePath, ownerType);
    if (binding && type) types.set(binding, type);
  }
  return types;
}

function functionCalls(source: string): DestinationCall[] {
  const structural = maskRustCommentsAndLiterals(source);
  const bodyStart = structural.indexOf("{");
  const calls: DestinationCall[] = [];
  for (const match of structural.matchAll(
    /(?<![A-Za-z0-9_.])((?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*)\s*\(/gu,
  )) {
    const offset = match.index ?? 0;
    if (offset <= bodyStart) continue;
    const open = offset + match[0].lastIndexOf("(");
    let close: number;
    try {
      close = matchingRustDelimiter(structural, open, "(", ")");
    } catch {
      continue;
    }
    calls.push({
      offset,
      name: match[1]!,
      arguments: splitTopLevelArguments(source.slice(open + 1, close)),
    });
  }
  for (const match of structural.matchAll(/\bself\.([a-z][A-Za-z0-9_]*)\s*\(/gu)) {
    const offset = match.index ?? 0;
    const open = offset + match[0].lastIndexOf("(");
    let close: number;
    try {
      close = matchingRustDelimiter(structural, open, "(", ")");
    } catch {
      continue;
    }
    calls.push({
      offset,
      name: `Self::${match[1]!}`,
      arguments: splitTopLevelArguments(source.slice(open + 1, close)),
    });
  }
  return calls.toSorted((left, right) => left.offset - right.offset);
}

function splitTopLevelArguments(source: string): string[] {
  const rows: string[] = [];
  let start = 0;
  let round = 0;
  let square = 0;
  let angle = 0;
  for (let cursor = 0; cursor < source.length; cursor += 1) {
    const current = source[cursor]!;
    if (current === "(") round += 1;
    else if (current === ")") round -= 1;
    else if (current === "[") square += 1;
    else if (current === "]") square -= 1;
    else if (current === "<") angle += 1;
    else if (current === ">") angle = Math.max(0, angle - 1);
    else if (current === "," && round === 0 && square === 0 && angle === 0) {
      rows.push(source.slice(start, cursor));
      start = cursor + 1;
    }
  }
  rows.push(source.slice(start));
  return rows.filter((row) => row.trim().length > 0);
}

function braceEvents(source: string): Array<{ offset: number; open: boolean }> {
  const structural = maskRustCommentsAndLiterals(source);
  return [...structural].flatMap((character, offset) =>
    character === "{"
      ? [{ offset, open: true }]
      : character === "}"
        ? [{ offset, open: false }]
        : [],
  );
}

function nearestBinding(scopes: readonly Map<string, boolean>[], name: string): boolean {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index]!.has(name)) return scopes[index]!.get(name) ?? false;
  }
  return false;
}

function nearestBindingType(
  scopes: readonly Map<string, string>[],
  name: string,
): string | undefined {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index]!.has(name)) return scopes[index]!.get(name);
  }
  return undefined;
}

function setNearestBindingType(
  scopes: readonly Map<string, string>[],
  name: string,
  value: string | undefined,
): void {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index]!.has(name)) {
      if (value) scopes[index]!.set(name, value);
      else scopes[index]!.delete(name);
      return;
    }
  }
  if (value) scopes.at(-1)!.set(name, value);
}

function setNearestBinding(
  scopes: readonly Map<string, boolean>[],
  name: string,
  value: boolean,
): void {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index]!.has(name)) {
      scopes[index]!.set(name, value);
      return;
    }
  }
  scopes.at(-1)!.set(name, value);
}

function pathBinding(expression: string): string | undefined {
  let normalized = maskRustCommentsAndLiterals(expression).trim();
  while (normalized.startsWith("&")) normalized = normalized.slice(1).trim();
  while (normalized.startsWith("(") && normalized.endsWith(")")) {
    normalized = normalized.slice(1, -1).trim();
  }
  normalized = normalized.replace(/\.(?:as_path|as_ref|clone)\s*\(\s*\)\s*$/u, "");
  return /^[A-Za-z_][A-Za-z0-9_]*$/u.test(normalized) ? normalized : undefined;
}

function stripOuterParens(expression: string): string {
  let current = expression.trim();
  while (current.startsWith("(") && current.endsWith(")")) {
    try {
      if (matchingRustDelimiter(current, 0, "(", ")") !== current.length - 1) break;
    } catch {
      break;
    }
    current = current.slice(1, -1).trim();
  }
  return current;
}

function hasTopLevelPathMixingOperator(expression: string): boolean {
  let round = 0;
  let square = 0;
  let brace = 0;
  for (let cursor = 0; cursor < expression.length; cursor += 1) {
    const current = expression[cursor]!;
    if (current === "(") round += 1;
    else if (current === ")") round -= 1;
    else if (current === "[") square += 1;
    else if (current === "]") square -= 1;
    else if (current === "{") brace += 1;
    else if (current === "}") brace -= 1;
    else if (round === 0 && square === 0 && brace === 0 && /[,+*/|&]/u.test(current)) {
      return true;
    }
  }
  return false;
}

function topLevelMethodNames(expression: string): string[] {
  const methods: string[] = [];
  let round = 0;
  let square = 0;
  let brace = 0;
  for (let cursor = 0; cursor < expression.length; cursor += 1) {
    const current = expression[cursor]!;
    if (current === "." && round === 0 && square === 0 && brace === 0) {
      const method = expression.slice(cursor).match(/^\.([A-Za-z_][A-Za-z0-9_]*)\s*\(/u)?.[1];
      if (method) methods.push(method);
    }
    if (current === "(") round += 1;
    else if (current === ")") round -= 1;
    else if (current === "[") square += 1;
    else if (current === "]") square -= 1;
    else if (current === "{") brace += 1;
    else if (current === "}") brace -= 1;
  }
  return methods;
}

function directCallPath(expression: string): string | undefined {
  let structural = stripOuterParens(maskRustCommentsAndLiterals(expression));
  while (structural.startsWith("&")) structural = structural.slice(1).trim();
  const match = structural.match(/^((?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*)\s*\(/u);
  if (!match) return undefined;
  const open = match[0].lastIndexOf("(");
  let close: number;
  try {
    close = matchingRustDelimiter(structural, open, "(", ")");
  } catch {
    return undefined;
  }
  const suffix = structural
    .slice(close + 1)
    .trim()
    .replace(/\?$/u, "");
  if (suffix && !/^(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^;]*\))*$/u.test(suffix)) {
    return undefined;
  }
  return match[1]!;
}

function resolvedCallIdentity(
  rawCall: string,
  source: string,
  options: DestinationDataflowOptions,
): string | undefined {
  const modulePath = options.modulePath ?? "crate";
  let identity: string;
  if (rawCall.startsWith("crate::") || rawCall.startsWith("std::")) {
    identity = rawCall;
  } else if (rawCall.startsWith("Self::")) {
    if (!options.ownerType) return undefined;
    identity = `${options.ownerType}::${rawCall.slice("Self::".length)}`;
  } else if (rawCall.includes("::")) {
    const first = rawCall.split("::", 1)[0]!;
    identity = /^[A-Z]/u.test(first) ? `${modulePath}::${rawCall}` : rawCall;
  } else {
    const structural = maskRustCommentsAndLiterals(options.lexicalSource ?? source);
    const body = structural.slice(Math.max(0, structural.indexOf("{") + 1));
    const shadow = new RegExp(`\\bfn\\s+${escapeRegExp(rawCall)}\\b`, "u");
    if (shadow.test(body)) return undefined;
    identity = `${modulePath}::${rawCall}`;
  }
  if (options.knownCalls && !options.knownCalls.has(identity)) return undefined;
  return identity;
}

function derivedOpenHandle(
  expression: string,
  scopes: readonly Map<string, boolean>[],
  options: DestinationDataflowOptions,
  typeScopes: readonly Map<string, string>[],
): boolean {
  const structural = maskRustCommentsAndLiterals(expression);
  const matches = [...structural.matchAll(/\.open\s*\(/gu)];
  if (matches.length !== 1) return false;
  const open = (matches[0]!.index ?? 0) + matches[0]![0].lastIndexOf("(");
  const argument = callArgument(expression, open, 0);
  return expressionDerived(argument, scopes, options, typeScopes);
}

interface DestinationDataflowOptions {
  readonly sanctionedCalls: readonly string[];
  readonly initialBindings?: readonly string[];
  readonly trustedFields?: ReadonlySet<string>;
  readonly knownCalls?: ReadonlySet<string>;
  readonly bindingTypes?: ReadonlyMap<string, string>;
  readonly modulePath?: string;
  readonly ownerType?: string;
  readonly lexicalSource?: string;
}

interface ProductDestinationFunction {
  readonly file: string;
  readonly key: string;
  readonly identity: string;
  readonly shortName: string;
  readonly source: string;
  readonly parameters: readonly string[];
  readonly bindingTypes: ReadonlyMap<string, string>;
  readonly modulePath: string;
  readonly ownerType?: string;
}

interface ProductDestinationGraph {
  readonly initialBindingsByFunction: ReadonlyMap<string, ReadonlySet<string>>;
  readonly trustedFields: ReadonlySet<string>;
  readonly bindingTypesByFunction: ReadonlyMap<string, ReadonlyMap<string, string>>;
  readonly modulePathByFunction: ReadonlyMap<string, string>;
  readonly ownerTypeByFunction: ReadonlyMap<string, string | undefined>;
}

function rustStructBody(source: string, name: string): string {
  const structural = maskRustCommentsAndLiterals(source);
  const declaration = new RegExp(
    `\\bstruct\\s+${escapeRegExp(name)}(?:\\s*<[^>{;]+>)?\\s*\\{`,
    "u",
  ).exec(structural);
  assert.ok(declaration?.index !== undefined, `missing Rust struct ${name}`);
  const open = declaration.index + declaration[0].lastIndexOf("{");
  const close = matchingRustDelimiter(structural, open, "{", "}");
  return structural.slice(open + 1, close);
}

function deriveProductDestinationGraph(): ProductDestinationGraph {
  if (productDestinationGraphCache) return productDestinationGraphCache;
  const files = [
    "rust/crates/omena-cli/src/daemon.rs",
    "rust/crates/omena-cli/src/lock.rs",
    transactionModulePath,
  ];
  const sources = new Map(files.map((file) => [file, productionRustSource(read(file))]));
  const functions: ProductDestinationFunction[] = files.flatMap((file) => {
    const source = sources.get(file)!;
    const modulePath = rustModuleIdentifier(file);
    return namedFunctions(maskRustCommentsAndLiterals(source), file).map((fn) => {
      const functionSource = source.slice(fn.start, fn.end);
      const ownerType = functionOwnerType(fn.name, modulePath);
      return {
        file,
        key: `${file}#${fn.name}`,
        identity: fn.name,
        shortName: fn.shortName,
        source: functionSource,
        parameters: functionParameterBindings(functionSource),
        bindingTypes: functionParameterTypes(functionSource, modulePath, ownerType),
        modulePath,
        ownerType,
      };
    });
  });
  const byFileAndShort = new Map<string, ProductDestinationFunction[]>();
  const byIdentity = new Map<string, ProductDestinationFunction[]>();
  for (const fn of functions) {
    const indexKey = `${fn.file}#${fn.shortName}`;
    const rows = byFileAndShort.get(indexKey) ?? [];
    rows.push(fn);
    byFileAndShort.set(indexKey, rows);
    const identities = byIdentity.get(fn.identity) ?? [];
    identities.push(fn);
    byIdentity.set(fn.identity, identities);
  }
  const trustedFields = new Set<string>();
  for (const [file, type, field, fieldType] of [
    [files[0]!, "OmenadArgs", "endpoint_file", "PathBuf"],
    [files[0]!, "OmenadProcessV0", "endpoint_file", "PathBuf"],
    [transactionModulePath, "StagedEditV0", "stage", "PathBuf"],
    [transactionModulePath, "StagedEditV0", "backup", "PathBuf"],
    [transactionModulePath, "TransactionLockGuard", "paths", "Vec<PathBuf>"],
  ] as const) {
    const body = rustStructBody(sources.get(file)!, type);
    assert.match(
      body,
      new RegExp(`(?:^|\\n)\\s*${field}\\s*:\\s*${escapeRegExp(fieldType)}\\s*,`, "u"),
      `trusted destination field declaration changed ${rustModuleIdentifier(file)}::${type}.${field}`,
    );
    trustedFields.add(`${rustModuleIdentifier(file)}::${type}.${field}`);
  }
  for (const identity of sanctionedProductDestinationCalls) {
    assert.equal(
      byIdentity.get(identity)?.length ?? 0,
      1,
      `sanctioned destination constructor not uniquely module-resolved ${identity}`,
    );
  }
  for (const [file, source] of sources) {
    assert.doesNotMatch(
      maskRustCommentsAndLiterals(source),
      /\buse\s+[^;]*::\s*\*\s*;/u,
      `destination authority does not admit glob imports ${file}`,
    );
  }
  const transaction = maskRustCommentsAndLiterals(sources.get(transactionModulePath)!);
  for (const field of ["stage", "backup"] as const) {
    assert.match(
      transaction,
      new RegExp(`let\\s+${field}_path\\s*=\\s*sidecar_path\\s*\\(`, "u"),
      `destination class not derived from a sanctioned-module call ${transactionModulePath}#${field}`,
    );
    assert.match(
      transaction,
      new RegExp(`${field}\\s*:\\s*${field}_path\\b`, "u"),
      `destination class not derived from a sanctioned-module call ${transactionModulePath}#${field}`,
    );
  }
  assert.match(
    transaction,
    /let\s+lock_path\s*=\s*transaction_lock_path\s*\(/u,
    `destination class not derived from a sanctioned-module call ${transactionModulePath}#lock_path`,
  );
  assert.match(
    transaction,
    /paths\.push\s*\(\s*lock_path\.clone\s*\(\s*\)\s*\)/u,
    `destination class not derived from a sanctioned-module call ${transactionModulePath}#paths`,
  );
  const daemon = maskRustCommentsAndLiterals(sources.get(files[0]!)!);
  assert.match(
    daemon,
    /let\s+endpoint_file\s*=\s*watch_endpoint_path\s*\(/u,
    `destination class not derived from a sanctioned-module call ${files[0]}#endpoint_file`,
  );
  assert.match(
    daemon,
    /OmenadProcessV0\s*\{[\s\S]*?\bendpoint_file\s*,[\s\S]*?\bdetached\s*:/u,
    `daemon process endpoint field lost its sanctioned constructor flow ${files[0]}#OmenadProcessV0.endpoint_file`,
  );
  const lock = maskRustCommentsAndLiterals(sources.get(files[1]!)!);
  assert.match(
    lock,
    /lock_verify_attestation\s*\(\s*LockVerifyAttestationInput\s*\{[\s\S]*?\blockfile\s*,/u,
    `destination class not derived from a sanctioned-module call ${files[1]}#lock_verify_attestation`,
  );

  const bindings = new Map<string, Set<string>>(functions.map((fn) => [fn.key, new Set()]));
  const root = (file: string, shortName: string, names: readonly string[]): void => {
    const candidates = byFileAndShort.get(`${file}#${shortName}`) ?? [];
    assert.equal(candidates.length, 1, `destination root item not resolvable ${file}#${shortName}`);
    for (const name of names) candidates[0] && bindings.get(candidates[0].key)!.add(name);
  };
  root(files[1]!, "lock_command", ["status_lockfile", "lockfile"]);
  root(files[1]!, "lock_verify_attestation", ["lockfile"]);

  const knownCalls = new Set(functions.map(({ identity }) => identity));
  let changed = true;
  while (changed) {
    changed = false;
    for (const fn of functions) {
      const analysis = deriveDestinationDataflow(fn.source, [], {
        sanctionedCalls: sanctionedProductDestinationCalls,
        initialBindings: [...bindings.get(fn.key)!],
        trustedFields,
        knownCalls,
        bindingTypes: fn.bindingTypes,
        modulePath: fn.modulePath,
        ownerType: fn.ownerType,
      });
      for (const call of analysis.calls) {
        for (const target of byIdentity.get(call.name) ?? []) {
          if (target.file !== fn.file) continue;
          const implicitSelf = target.parameters[0] === "self" ? 1 : 0;
          for (let index = 0; index < call.argumentsDerived.length; index += 1) {
            const parameter = target.parameters[index + implicitSelf];
            if (!parameter || !call.argumentsDerived[index]) continue;
            const targetBindings = bindings.get(target.key)!;
            if (!targetBindings.has(parameter)) {
              targetBindings.add(parameter);
              changed = true;
            }
          }
        }
      }
    }
  }
  for (const [identity, binding] of [
    ["crate::daemon::write_endpoint", "path"],
    ["crate::daemon::cleanup_endpoint", "path"],
    ["crate::lock::publish_immutable_recorded_shard_verdict", "path"],
    ["crate::workspace_edit_transaction::write_transaction_lock_file", "lock_path"],
    ["crate::workspace_edit_transaction::write_transaction_journal_file", "path"],
    ["crate::workspace_edit_transaction::remove_if_exists", "path"],
  ] as const) {
    const candidates = byIdentity.get(identity) ?? [];
    assert.equal(candidates.length, 1, `destination sink not uniquely resolved ${identity}`);
    assert.ok(
      bindings.get(candidates[0]!.key)?.has(binding),
      `destination sink lost full-call-graph authority ${identity}#${binding}`,
    );
  }
  productDestinationGraphCache = {
    initialBindingsByFunction: bindings,
    trustedFields,
    bindingTypesByFunction: new Map(functions.map((fn) => [fn.key, fn.bindingTypes])),
    modulePathByFunction: new Map(functions.map((fn) => [fn.key, fn.modulePath])),
    ownerTypeByFunction: new Map(functions.map((fn) => [fn.key, fn.ownerType])),
  };
  return productDestinationGraphCache;
}

function expressionDerived(
  expression: string,
  scopes: readonly Map<string, boolean>[],
  options: DestinationDataflowOptions,
  typeScopes: readonly Map<string, string>[] = [new Map(options.bindingTypes ?? [])],
): boolean {
  let structural = stripOuterParens(maskRustCommentsAndLiterals(expression));
  while (structural.startsWith("&")) structural = structural.slice(1).trim();
  const rawCall = directCallPath(structural);
  const call = rawCall ? resolvedCallIdentity(rawCall, expression, options) : undefined;
  if (call && options.sanctionedCalls.includes(call)) return true;
  if (/\.open\s*\(/u.test(structural)) {
    return derivedOpenHandle(expression, scopes, options, typeScopes);
  }
  if (/\b(?:if|match)\b/u.test(structural)) {
    return derivedOpenHandle(expression, scopes, options, typeScopes);
  }
  if (rawCall && !pathPreservingMethods.has(rawCall.split("::").at(-1)!)) return false;
  if (hasTopLevelPathMixingOperator(structural)) return false;
  const root = structural.match(/^([A-Za-z_][A-Za-z0-9_]*)\b/u)?.[1];
  if (!root) return false;
  const methods = topLevelMethodNames(structural);
  if (methods.some((method) => !pathPreservingMethods.has(method) && method !== "open")) {
    return false;
  }
  const field = structural.match(/^[A-Za-z_][A-Za-z0-9_]*\.([A-Za-z_][A-Za-z0-9_]*)\b/u)?.[1];
  const rootType = nearestBindingType(typeScopes, root);
  const fieldDerived = Boolean(
    field && rootType && options.trustedFields?.has(`${rootType}.${field}`),
  );
  return nearestBinding(scopes, root) || fieldDerived;
}

function receiverBinding(source: string, offset: number): string | undefined {
  const before = maskRustCommentsAndLiterals(source.slice(0, offset));
  return before.match(/([A-Za-z_][A-Za-z0-9_]*)\s*$/u)?.[1];
}

function statementContaining(source: string, offset: number): string {
  const structural = maskRustCommentsAndLiterals(source);
  let start = offset;
  let round = 0;
  let square = 0;
  while (start > 0) {
    const current = structural[start - 1]!;
    if (current === ")") round += 1;
    else if (current === "(") round -= 1;
    else if (current === "]") square += 1;
    else if (current === "[") square -= 1;
    if ((current === ";" || current === "{") && round <= 0 && square <= 0) break;
    start -= 1;
  }
  const end = statementEnd(structural, start);
  return source.slice(start, end);
}

function transitionBindingType(
  transition: DestinationTransition,
  typeScopes: readonly Map<string, string>[],
  options: DestinationDataflowOptions,
): string | undefined {
  const colon = transition.pattern.indexOf(":");
  if (colon >= 0) {
    const explicit = resolvedRustTypeIdentity(
      transition.pattern.slice(colon + 1),
      options.modulePath ?? "crate",
      options.ownerType,
    );
    if (explicit) return explicit;
  }
  const structural = stripOuterParens(maskRustCommentsAndLiterals(transition.expression));
  const root = structural.match(/^&?\s*([A-Za-z_][A-Za-z0-9_]*)\b/u)?.[1];
  if (root) {
    const inherited = nearestBindingType(typeScopes, root);
    if (inherited) return inherited;
  }
  return resolvedRustTypeIdentity(
    structural.match(/^([A-Z][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)\s*\{/u)?.[1] ?? "",
    options.modulePath ?? "crate",
    options.ownerType,
  );
}

function deriveDestinationDataflow(
  source: string,
  mutations: readonly FilesystemMutation[],
  options: DestinationDataflowOptions,
): DestinationDataflow {
  const effectiveOptions: DestinationDataflowOptions = { ...options, lexicalSource: source };
  const transitions = destinationTransitions(source);
  const braces = braceEvents(source);
  const calls = functionCalls(source)
    .flatMap((call) => {
      const name = resolvedCallIdentity(call.name, source, effectiveOptions);
      return name ? [{ ...call, name }] : [];
    })
    .filter(({ name }) => options.knownCalls?.has(name) ?? false);
  const scopes: Map<string, boolean>[] = [
    new Map((options.initialBindings ?? []).map((binding) => [binding, true])),
  ];
  const typeScopes: Map<string, string>[] = [new Map(options.bindingTypes ?? [])];
  const derivedByMutation = new Map<string, boolean>();
  const observedCalls: Array<{ name: string; argumentsDerived: readonly boolean[] }> = [];
  const unparsedConstructs: string[] = [];
  const structural = maskRustCommentsAndLiterals(source);
  const definedMacros = new Set(
    [...structural.matchAll(/\bmacro_rules!\s*([A-Za-z_][A-Za-z0-9_]*)/gu)].map(
      (match) => match[1]!,
    ),
  );
  for (const name of definedMacros) {
    if (new RegExp(`\\b${escapeRegExp(name)}!\\s*\\(`, "u").test(structural)) {
      unparsedConstructs.push(`statement-macro:${name}`);
    }
  }
  if (/\buse\s+[^;]*::\s*\*\s*;/u.test(structural)) {
    unparsedConstructs.push("glob-import");
  }
  const events = [
    ...braces.map((event) => ({ ...event, type: "brace" as const })),
    ...transitions.map((transition) => ({ ...transition, type: "transition" as const })),
    ...calls.map((call) => ({ ...call, type: "call" as const })),
    ...mutations.map((mutation) => ({ ...mutation, type: "mutation" as const })),
  ].toSorted((left, right) => left.offset - right.offset || (left.type === "brace" ? -1 : 1));
  for (const event of events) {
    if (event.type === "brace") {
      if (event.open) {
        scopes.push(new Map());
        typeScopes.push(new Map());
      } else if (scopes.length > 1) {
        scopes.pop();
        typeScopes.pop();
      }
      continue;
    }
    if (event.type === "call") {
      observedCalls.push({
        name: event.name,
        argumentsDerived: event.arguments.map((argument) =>
          expressionDerived(argument, scopes, effectiveOptions, typeScopes),
        ),
      });
      continue;
    }
    if (event.type === "transition") {
      const derived = expressionDerived(event.expression, scopes, effectiveOptions, typeScopes);
      const binding = simpleBindingPattern(event.pattern);
      const bindingType = transitionBindingType(event, typeScopes, effectiveOptions);
      if (event.letElse && derived) {
        unparsedConstructs.push("let-else-pattern");
      }
      if (!binding) {
        if (event.controlPattern) {
          for (const name of patternBindings(event.pattern)) {
            scopes.at(-1)!.set(name, derived);
            if (bindingType) typeScopes.at(-1)!.set(name, bindingType);
          }
        } else if (event.pattern.trim() !== "_" && derived) {
          unparsedConstructs.push(
            event.kind === "let" ? "non-identifier-pattern" : "non-identifier-lvalue",
          );
        }
        continue;
      }
      if (event.kind === "let") {
        scopes.at(-1)!.set(binding, derived);
        if (bindingType) typeScopes.at(-1)!.set(binding, bindingType);
      } else {
        setNearestBinding(scopes, binding, derived);
        setNearestBindingType(typeScopes, binding, bindingType);
      }
      continue;
    }
    const direct = expressionDerived(event.destination, scopes, effectiveOptions, typeScopes);
    const binding = pathBinding(event.destination);
    const receiver =
      event.destination === "<receiver-bound>" ? receiverBinding(source, event.offset) : undefined;
    const deferredBuilder =
      event.destination === "<receiver-bound>" &&
      expressionDerived(
        statementContaining(source, event.offset),
        scopes,
        effectiveOptions,
        typeScopes,
      );
    derivedByMutation.set(
      destinationMutationIdentity(event),
      direct ||
        Boolean(binding && nearestBinding(scopes, binding)) ||
        Boolean(receiver && nearestBinding(scopes, receiver)) ||
        deferredBuilder,
    );
  }
  return {
    derivedByMutation,
    unparsedConstructs: [...new Set(unparsedConstructs)],
    calls: observedCalls,
  };
}

function destinationFixtureAnalysis(
  source: string,
  options: Pick<DestinationDataflowOptions, "modulePath" | "sanctionedCalls" | "trustedFields">,
): { mutation: FilesystemMutation; analysis: DestinationDataflow } {
  const mutations = filesystemMutations(source);
  assert.equal(
    mutations.length,
    1,
    "destination dataflow fixture must contain one write primitive",
  );
  const modulePath = options.modulePath ?? "crate";
  return {
    mutation: mutations[0]!,
    analysis: deriveDestinationDataflow(source, mutations, {
      ...options,
      bindingTypes: functionParameterTypes(source, modulePath),
    }),
  };
}

function assertDestinationFixtureAllowed(
  label: string,
  source: string,
  options: Pick<DestinationDataflowOptions, "modulePath" | "sanctionedCalls" | "trustedFields">,
): void {
  const { mutation, analysis } = destinationFixtureAnalysis(source, options);
  assert.deepEqual(analysis.unparsedConstructs, [], `${label} must be fully parsed`);
  assert.equal(
    analysis.derivedByMutation.get(destinationMutationIdentity(mutation)),
    true,
    `${label} must carry non-product destination authority`,
  );
}

function assertDestinationFixtureDenied(
  label: string,
  source: string,
  options: Pick<DestinationDataflowOptions, "modulePath" | "sanctionedCalls" | "trustedFields">,
  expectedUnparsed?: string,
): void {
  const { mutation, analysis } = destinationFixtureAnalysis(source, options);
  assert.equal(
    analysis.derivedByMutation.get(destinationMutationIdentity(mutation)),
    false,
    `${label} must not carry non-product destination authority`,
  );
  if (expectedUnparsed) {
    assert.ok(
      analysis.unparsedConstructs.includes(expectedUnparsed),
      `${label} must fail closed as ${expectedUnparsed}`,
    );
  }
}

function isProvenNonProductDestination(
  key: string,
  mutation: FilesystemMutation,
  functionSource: string,
): boolean {
  const graph = deriveProductDestinationGraph();
  const initialBindings = [...(graph.initialBindingsByFunction.get(key) ?? [])];
  const analysis = deriveDestinationDataflow(functionSource, filesystemMutations(functionSource), {
    sanctionedCalls: sanctionedProductDestinationCalls,
    initialBindings,
    trustedFields: graph.trustedFields,
    bindingTypes: graph.bindingTypesByFunction.get(key),
    modulePath: graph.modulePathByFunction.get(key),
    ownerType: graph.ownerTypeByFunction.get(key),
  });
  assert.deepEqual(
    analysis.unparsedConstructs,
    [],
    `unparsed destination construct ${analysis.unparsedConstructs[0] ?? "unknown"} ${key}`,
  );
  const derived = analysis.derivedByMutation.get(destinationMutationIdentity(mutation)) ?? false;
  return derived;
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
    for (const file of Array.from(derived)) {
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

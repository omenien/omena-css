import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

type WriteClassification = "artifact" | "bookkeeping" | "transaction-staging";

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

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const cliRoot = "rust/crates/omena-cli/src";
const manifestPath = "rust/crates/omena-cli/write-safety-census.json";
const transactionModulePath = "rust/crates/omena-cli/src/workspace_edit_transaction.rs";
const manifest = readJson<WriteSafetyManifest>(manifestPath);
const fixSafetySource = read("rust/crates/omena-checker/src/fix_safety.rs");
const writeGateSource = read(manifest.sourceMutationGate.path);
const queryRunnerSource = read("rust/crates/omena-query-transform-runner/src/lib.rs");
const queryFacadeSource = read("rust/crates/omena-query/src/lib.rs");
const productionWritePrimitive =
  /\b(?:std::)?fs::(?:write|copy|rename)\s*\(|\b(?:std::fs::)?File::(?:create|options|create_new)\s*\(|\b(?:std::fs::)?OpenOptions::new\s*\(|\.write(?:_all)?\s*\(/gu;
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
assertProductSourcePlaneZero(derivedWriteSites);
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
      function: "prepare_rollback_backup",
      writeCount: 2,
      classification: "bookkeeping",
      owner: "transaction rollback backup preparation",
    },
    {
      path: transactionModulePath,
      function: "rename_all",
      writeCount: 1,
      classification: "transaction-staging",
      owner: "transaction staged product publication",
    },
    {
      path: transactionModulePath,
      function: "rollback_and_cleanup",
      writeCount: 1,
      classification: "bookkeeping",
      owner: "transaction rollback restoration",
    },
    {
      path: transactionModulePath,
      function: "write_staged_product_bytes",
      writeCount: 3,
      classification: "transaction-staging",
      owner: "transaction staged product bytes",
    },
    {
      path: transactionModulePath,
      function: "write_transaction_journal_file",
      writeCount: 3,
      classification: "bookkeeping",
      owner: "transaction rollback journal sidecar",
    },
    {
      path: transactionModulePath,
      function: "write_transaction_lock_file",
      writeCount: 2,
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
  /unclassified production write: .*#apply_write_with_safety/u,
  "reintroducing a direct product write must be RED",
);

const transactionSource = read(transactionModulePath);
const minifySource = read("rust/crates/omena-cli/src/minify.rs");
const buildSource = read("rust/crates/omena-cli/src/build.rs");
assertDestinationKeyedPostconditions(minifySource, buildSource);
const inputKeyedMinifyMutation = minifySource.replace(
  /(text_reparse_for_path\(\s*)output(\.as_path\(\)\s*,?\s*\))/u,
  "$1input$2",
);
assert.notEqual(
  inputKeyedMinifyMutation,
  minifySource,
  "minify path-key mutation must alter source",
);
assert.throws(
  () => assertDestinationKeyedPostconditions(inputKeyedMinifyMutation, buildSource),
  /minify postcondition must be keyed to its destination path/u,
  "keying the minify postcondition to the input path must be RED",
);
const inputKeyedBuildMutation = buildSource.replace(
  /(style_reparse_for_admitted_output\(\s*)output_path(\.as_path\(\)\s*,)/gu,
  "$1path$2",
);
assert.notEqual(inputKeyedBuildMutation, buildSource, "build path-key mutation must alter source");
assert.throws(
  () => assertDestinationKeyedPostconditions(minifySource, inputKeyedBuildMutation),
  /build postconditions must be keyed to their destination paths/u,
  "keying either build postcondition to an input path must be RED",
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
  /unclassified production write: .*#unregistered_transaction_write_authority/u,
  "an unregistered primitive owner must be RED",
);

const registeredArtifactMutation = [
  ...manifest.writeSites,
  {
    path: transactionModulePath,
    function: "unregistered_transaction_write_authority",
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
  /product\/source-plane bare artifact writes must be zero/u,
  "registering a new bare product writer as classification:artifact must remain RED",
);

const fsCopyBypassSource = transactionSource.replace(
  testModuleMarker,
  "\nfn copy_product_bytes(source: &std::path::Path, destination: &std::path::Path) {\n    let _ = std::fs::copy(source, destination);\n}\n\n#[cfg(test)]\nmod tests {",
);
assert.throws(
  () => deriveProductionWriteSites(new Map([[transactionModulePath, fsCopyBypassSource]])),
  /unclassified production write: .*#copy_product_bytes/u,
  "an fs::copy product-byte bypass must be RED",
);

const fileOptionsWriteBypassSource = transactionSource.replace(
  testModuleMarker,
  '\nfn options_write_product_bytes(path: &std::path::Path) {\n    use std::io::Write as _;\n    let mut file = std::fs::File::options().write(true).open(path).expect("mutation control");\n    let _ = file.write(b"mutation control");\n}\n\n#[cfg(test)]\nmod tests {',
);
assert.throws(
  () =>
    deriveProductionWriteSites(new Map([[transactionModulePath, fileOptionsWriteBypassSource]])),
  /unclassified production write: .*#options_write_product_bytes/u,
  "a File::options().write() product-byte bypass must be RED",
);

const productionAfterTestsSource = `${transactionSource}\nstruct PostTestProductionWriter;\nimpl PostTestProductionWriter {\n    fn write_product_bytes_after_tests(path: &std::path::Path) {\n        let _ = std::fs::write(path, b"mutation control");\n    }\n}\n`;
assert.throws(
  () => deriveProductionWriteSites(new Map([[transactionModulePath, productionAfterTestsSource]])),
  /unclassified production write: .*#write_product_bytes_after_tests/u,
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
  const manifestByKey = new Map(
    registeredWriteSites.map((site) => [`${site.path}#${site.function}`, site]),
  );
  const derived = new Map<string, { path: string; function: string; writeCount: number }>();
  const observedNonFilesystemWrites = new Map<string, number>();
  const nonFilesystemByKey = new Map(
    nonFilesystemWriteSinks.map((sink) => [`${sink.path}#${sink.function}`, sink]),
  );

  for (const file of rustSourceFiles(cliRoot)) {
    const source = productionRustSource(sourceOverrides.get(file) ?? read(file));
    const structuralSource = maskRustCommentsAndLiterals(source);
    const functions = namedFunctions(structuralSource);
    for (const match of structuralSource.matchAll(productionWritePrimitive)) {
      const offset = match.index ?? -1;
      const owner = functions.findLast(({ start, end }) => start < offset && offset < end);
      assert.ok(owner, `${file} contains a production write primitive outside a named function`);
      const key = `${file}#${owner.name}`;
      if (nonFilesystemByKey.has(key)) {
        observedNonFilesystemWrites.set(key, (observedNonFilesystemWrites.get(key) ?? 0) + 1);
        continue;
      }
      const current = derived.get(key) ?? { path: file, function: owner.name, writeCount: 0 };
      current.writeCount += 1;
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
    const functions = namedFunctions(maskRustCommentsAndLiterals(source));
    const index = functions.findIndex(({ name }) => name === sink.function);
    assert.ok(index >= 0, `non-filesystem write sink is missing: ${key}`);
    assert.ok(
      source.slice(functions[index]!.start, functions[index]!.end).includes(sink.evidence),
      `non-filesystem write sink lost its ${sink.evidence} evidence: ${key}`,
    );
  }

  return [...derived.values()].map((site) => {
    const registered = manifestByKey.get(`${site.path}#${site.function}`);
    assert.ok(registered, `unclassified production write: ${site.path}#${site.function}`);
    return { ...site, classification: registered.classification, owner: registered.owner };
  });
}

function assertProductSourcePlaneZero(writeSites: readonly WriteSite[]): void {
  assert.equal(
    writeSites.filter(({ classification }) => classification === "artifact").length,
    0,
    "product/source-plane bare artifact writes must be zero",
  );
}

function assertDestinationKeyedPostconditions(minify: string, build: string): void {
  assert.equal(
    [...minify.matchAll(/text_reparse_for_path\(\s*output\.as_path\(\)\s*,?\s*\)/gu)].length,
    1,
    "minify postcondition must be keyed to its destination path",
  );
  assert.equal(
    [...build.matchAll(/style_reparse_for_admitted_output\(\s*output_path\.as_path\(\)\s*,/gu)]
      .length,
    2,
    "build postconditions must be keyed to their destination paths",
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
  const testModule = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+[a-zA-Z0-9_]+\s*\{/gu;
  for (const match of structural.matchAll(testModule)) {
    const start = match.index ?? 0;
    const open = start + match[0].lastIndexOf("{");
    spans.push({ start, end: matchingBrace(structural, open, "cfg(test) module") + 1 });
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
    [...maskRustCommentsAndLiterals(functionSource).matchAll(productionWritePrimitive)].length,
    0,
    `${functionName} must authorize transaction commit without a direct filesystem primitive`,
  );
}

function topLevelFunctionSource(source: string, functionName: string): string {
  const functions = namedFunctions(maskRustCommentsAndLiterals(source));
  const index = functions.findIndex(({ name }) => name === functionName);
  assert.ok(index >= 0, `missing top-level function ${functionName}`);
  return source.slice(functions[index]!.start, functions[index]!.end);
}

function namedFunctions(source: string): { name: string; start: number; end: number }[] {
  const functions: { name: string; start: number; end: number }[] = [];
  const declaration =
    /^[\t ]*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+([a-z][a-z0-9_]*)[^;{]*\{/gmu;
  for (const match of source.matchAll(declaration)) {
    const start = match.index ?? 0;
    const open = start + match[0].lastIndexOf("{");
    const end = matchingBrace(source, open, `function ${match[1]}`) + 1;
    functions.push({ name: match[1]!, start, end });
  }
  return functions;
}

function siteIdentity(site: WriteSite): string {
  return [site.path, site.function, site.writeCount, site.classification, site.owner].join("|");
}

function rustSourceFiles(root: string): string[] {
  return sourceFiles([root], [".rs"]).filter(
    (file) => !file.endsWith("/tests.rs") && !file.includes("/tests/"),
  );
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

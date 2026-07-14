import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

interface SassUnsupportedRecordV0 {
  readonly file: string;
  readonly reason: string;
  readonly currentLine: number | null;
  readonly ledgerLineHint: number;
  readonly linkedFixtureIds: readonly string[];
  readonly gap: string | null;
}

interface SassUnsupportedLedgerV0 {
  readonly product: "omena-diff-test.sass-spec-bail-site-ledger";
  readonly semanticSiteCount: number;
  readonly ledgerSemanticSiteCount: number;
  readonly linkedSiteCount: number;
  readonly namedGapSiteCount: number;
  readonly allSemanticSitesMatchLedger: boolean;
  readonly allSitesLinkedOrNamedGap: boolean;
  readonly allBailSiteLedgerChecksHold: boolean;
  readonly records: SassUnsupportedRecordV0[];
}

const repoRoot = process.cwd();
const cliSassPath = "rust/crates/omena-cli/src/sass.rs";
const compilePath = "rust/crates/omena-cli/src/sass/compile.rs";
const sifPath = "rust/crates/omena-sif/src/lib.rs";
const sifDiffPath = "rust/crates/omena-sif/src/structural_diff.rs";
const bridgePath = "scripts/run-sass-compile-bridge.ts";
const queryPath = "rust/crates/omena-query/src/sass_unsupported.rs";
const diffTestPath = "rust/crates/omena-diff-test/src/lib.rs";
const ledgerPath = "rust/crates/omena-query/data/sass-unsupported-ledger.json";

let cliSassSource = read(cliSassPath);
let compileSource = read(compilePath);
const sifSource = read(sifPath);
let sifDiffSource = read(sifDiffPath);
const bridgeSource = read(bridgePath);
const querySource = read(queryPath);
const diffTestSource = read(diffTestPath);
const ledger = JSON.parse(read(ledgerPath)) as SassUnsupportedLedgerV0;

if (process.argv.includes("--inject-direct-parser")) {
  cliSassSource += '\nfn injected_parser_bypass() { omena_parser::parse_style(""); }\n';
}
if (process.argv.includes("--drop-visibility")) {
  cliSassSource = cliSassSource.replaceAll("visibility_filter", "omitted_filter");
}
if (process.argv.includes("--drop-export-kind")) {
  sifDiffSource = sifDiffSource.replaceAll("forwards: old_forwards,", "");
}
if (process.argv.includes("--invert-removal")) {
  const start = sifDiffSource.indexOf("(Some(old), None) =>");
  assert.ok(start >= 0, "missing removed-export classification arm");
  const end = sifDiffSource.indexOf("(None, Some(new)) =>", start);
  assert.ok(end > start, "missing added-export classification arm");
  sifDiffSource =
    sifDiffSource.slice(0, start) +
    sifDiffSource.slice(start, end).replace("::Removed", "::Added") +
    sifDiffSource.slice(end);
}
if (process.argv.includes("--drop-authority-label")) {
  compileSource = compileSource.replaceAll("compiled by dart-sass", "compiled externally");
}
if (process.argv.includes("--drop-external-witness")) {
  compileSource = compileSource.replace(
    "FamilyStampV0::external_tool(&witness)",
    "FamilyStampV0::default()",
  );
}
if (process.argv.includes("--drop-ledger-record")) {
  ledger.records.pop();
}

assert.doesNotMatch(
  cliSassSource,
  /\bomena_parser::|\bparse_omena_(?:style|stylesheet)|\bparse_style_document/u,
  "the Sass product surface must view query-owned facts instead of parsing sources directly",
);
for (const binding of [
  "summarize_omena_query_sass_module_cross_file_resolution_for_workspace",
  "unresolved_module_edge_count",
  "visibility_filter_count",
  "visibility_filter_kind",
  "visibility_filter_names",
  "namespace_show_hide_filter_ready",
  "edge.namespace",
]) {
  assert.ok(cliSassSource.includes(binding), `missing Sass graph-view binding: ${binding}`);
}

const exportFields = collectPublicFieldNames(
  extractRustBlock(sifSource, "pub struct OmenaSifExportsV1"),
);
assert.deepEqual(
  exportFields,
  ["variables", "mixins", "functions", "placeholders", "forwards"],
  "the structural diff gate must be updated when the SIF export universe changes",
);
const exportKindCount = Number(
  requiredCapture(
    sifDiffSource,
    /pub const OMENA_SIF_EXPORT_KIND_COUNT_V1: usize = (\d+);/u,
    "SIF export-kind count",
  ),
);
assert.equal(exportKindCount, exportFields.length, "SIF export-kind census must cover every field");
for (const field of exportFields) {
  assert.ok(
    sifDiffSource.includes(`${field}: old_${field}`) &&
      sifDiffSource.includes(`${field}: new_${field}`),
    `structural diff does not destructure both sides of SIF export field: ${field}`,
  );
}
assert.deepEqual(
  collectEnumVariants(extractRustBlock(sifDiffSource, "pub enum OmenaSifStructuralChangeKindV0")),
  ["Removed", "Changed", "VisibilityNarrowed", "Added"],
  "SIF structural compatibility must retain the four typed change classes",
);
const removedArm = sourceBetween(sifDiffSource, "(Some(old), None) =>", "(None, Some(new)) =>");
assert.ok(
  removedArm.includes("OmenaSifStructuralChangeKindV0::Removed") &&
    !removedArm.includes("OmenaSifStructuralChangeKindV0::Added"),
  "an export present only in the previous SIF must be classified as removed",
);
for (const binding of [
  "compute_omena_sif_interface_hash_v1",
  "stored_interface_hashes_valid",
  'fast_path: Some("verifiedInterfaceHash")',
  "removed_count + changed_count + visibility_narrowed_count + added_count",
  "forward_visibility_narrowed(old, new)",
]) {
  assert.ok(sifDiffSource.includes(binding), `missing structural-diff invariant: ${binding}`);
}

assert.ok(
  bridgeSource.includes("assertPinnedDartSassVersion") &&
    bridgeSource.includes("runPinnedDartSass"),
  "Sass compilation must reuse the shared pinned Dart Sass primitive",
);
assert.doesNotMatch(
  compileSource,
  /\b\d+\.\d+\.\d+\b/u,
  "the Rust bridge must not introduce a second Dart Sass version pin",
);
for (const binding of [
  "compiled by dart-sass",
  "ExternalToolRunWitnessV0",
  "FamilyStampV0::external_tool(&witness)",
  "summarize_omena_query_sass_module_cross_file_resolution_for_workspace",
]) {
  const source = binding.startsWith("summarize_") ? cliSassSource : compileSource;
  assert.ok(source.includes(binding), `missing delegated Sass compile binding: ${binding}`);
}

assert.equal(ledger.product, "omena-diff-test.sass-spec-bail-site-ledger");
assert.ok(ledger.records.length > 0, "the unsupported-site product view must be non-vacuous");
assert.equal(ledger.records.length, ledger.semanticSiteCount);
assert.equal(ledger.records.length, ledger.ledgerSemanticSiteCount);
assert.equal(
  ledger.records.filter((record) => record.linkedFixtureIds.length > 0).length,
  ledger.linkedSiteCount,
);
assert.equal(
  ledger.records.filter((record) => record.gap !== null).length,
  ledger.namedGapSiteCount,
);
assert.ok(ledger.allSemanticSitesMatchLedger);
assert.ok(ledger.allSitesLinkedOrNamedGap);
assert.ok(ledger.allBailSiteLedgerChecksHold);
assert.ok(
  ledger.records.every(
    (record) =>
      record.file.length > 0 &&
      record.reason.length > 0 &&
      (record.currentLine ?? record.ledgerLineHint) > 0 &&
      (record.linkedFixtureIds.length > 0 || (record.gap?.length ?? 0) > 0),
  ),
  "every unsupported site must retain a location, reason, and coverage disposition",
);
for (const binding of [
  'include_str!("../data/sass-unsupported-ledger.json")',
  "canonical.all_semantic_sites_match_ledger",
  "surface_record_count == canonical.ledger_semantic_site_count",
  "summary_view_ready",
]) {
  assert.ok(
    querySource.includes(binding),
    `missing unsupported-site projection binding: ${binding}`,
  );
}
assert.ok(
  diffTestSource.includes('include_str!("../../omena-query/data/sass-unsupported-ledger.json")') &&
    diffTestSource.includes("render_sass_spec_bail_site_product_view_json") &&
    diffTestSource.includes("summarize_sass_spec_bail_site_ledger()"),
  "the committed product projection must be regenerated by the independent parity authority",
);

process.stdout.write(
  `Omena Sass intelligence OK: graph=view exportKinds=${exportKindCount} changeKinds=4 unsupportedSites=${ledger.records.length} authority=dart-sass\n`,
);

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function requiredCapture(source: string, pattern: RegExp, label: string): string {
  const match = pattern.exec(source);
  assert.ok(match, `missing ${label}`);
  return match[1];
}

function extractRustBlock(source: string, marker: string): string {
  const markerIndex = source.indexOf(marker);
  assert.ok(markerIndex >= 0, `missing Rust block marker: ${marker}`);
  const openIndex = source.indexOf("{", markerIndex);
  assert.ok(openIndex >= 0, `missing opening brace for: ${marker}`);
  let depth = 0;
  for (let index = openIndex; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(markerIndex, index + 1);
  }
  throw new Error(`unterminated Rust block: ${marker}`);
}

function collectPublicFieldNames(structBlock: string): string[] {
  return [...structBlock.matchAll(/^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/gmu)].map(
    (match) => match[1],
  );
}

function collectEnumVariants(enumBlock: string): string[] {
  return enumBlock
    .slice(enumBlock.indexOf("{") + 1, enumBlock.lastIndexOf("}"))
    .split(",")
    .map((part) => part.trim())
    .filter((part) => /^[A-Z][A-Za-z0-9_]*$/u.test(part));
}

function sourceBetween(source: string, startMarker: string, endMarker: string): string {
  const start = source.indexOf(startMarker);
  const end = source.indexOf(endMarker, start + startMarker.length);
  assert.ok(start >= 0 && end > start, `missing source span: ${startMarker} .. ${endMarker}`);
  return source.slice(start, end);
}

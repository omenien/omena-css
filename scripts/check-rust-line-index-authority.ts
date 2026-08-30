import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface LineIndexAuthorityBaselineV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-syntax.line-index-authority-baseline";
  readonly workspaceMemberManifestPaths: readonly string[];
  readonly initialQueryWrapperSourcePin: string;
  readonly initialQueryWrapperCallSiteCount: 59;
  readonly initialQueryWrapperCallSites: readonly FreshIndexCallSite[];
  readonly migratedQueryWrapperCallSiteCount: number;
  readonly queryWrapperCallSites: readonly FreshIndexCallSite[];
  readonly queryConstructorCallSites: readonly FreshIndexCallSite[];
}

interface DefinitionSite {
  readonly id: string;
  readonly relativePath: string;
  readonly functionName: string;
  readonly direction: "forward" | "reverse";
}

interface ProductPassSite {
  readonly id: string;
  readonly relativePath: string;
  readonly functionName: string;
  readonly buildPattern: RegExp;
  readonly conversionPattern: RegExp;
  readonly expectedBuildCount?: number;
  readonly expectedLoopDepth?: number;
}

interface FreshIndexCallSite {
  readonly id: string;
  readonly relativePath: string;
  readonly functionName: string;
  readonly wrapperName: string;
  readonly occurrenceWithinFunction: number;
  readonly loopNested: boolean;
  readonly callerLoopNested: boolean;
  readonly loopNestedCallerCallSiteIds: readonly string[];
}

interface RustFunctionSpan {
  readonly name: string;
  readonly bodyStart: number;
  readonly bodyEnd: number;
}

interface RustSourceFile {
  readonly relativePath: string;
  readonly source: string;
}

interface RustFunctionCallSite {
  readonly id: string;
  readonly relativePath: string;
  readonly functionName: string;
  readonly calleeName: string;
  readonly loopNested: boolean;
}

interface TwoFrameLoopResidentWrapperSite {
  readonly wrapperCallSiteId: string;
  readonly paths: readonly string[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const baselinePath = path.join(
  repoRoot,
  "rust/crates/omena-syntax/tests/snapshots/line-index-authority-baseline.json",
);
const publicApiSnapshotPath = path.join(
  repoRoot,
  "rust/crates/omena-syntax/tests/snapshots/public-api.txt",
);
const writeBaseline = process.argv.includes("--write");
const injectedScanner = process.argv
  .find((argument) => argument.startsWith("--inject-scanner="))
  ?.slice("--inject-scanner=".length);
const injectWorkspaceMember = process.argv.includes("--inject-workspace-member");
const injectedProductRebuild = process.argv
  .find((argument) => argument.startsWith("--inject-product-rebuild="))
  ?.slice("--inject-product-rebuild=".length);
const injectedProductLoop = process.argv
  .find((argument) => argument.startsWith("--inject-product-loop="))
  ?.slice("--inject-product-loop=".length);
const injectedWrapperLoop = process.argv
  .find((argument) => argument.startsWith("--inject-wrapper-loop="))
  ?.slice("--inject-wrapper-loop=".length);
const injectedCallerWrapperLoop = process.argv
  .find((argument) => argument.startsWith("--inject-caller-wrapper-loop="))
  ?.slice("--inject-caller-wrapper-loop=".length);
const injectedTwoFrameWrapperLoop = process.argv
  .find((argument) => argument.startsWith("--inject-two-frame-wrapper-loop="))
  ?.slice("--inject-two-frame-wrapper-loop=".length);
const reportTwoFramePopulation = process.argv.includes("--report-two-frame-population");
const initialWrapperSourcePinArgument = process.argv
  .find((argument) => argument.startsWith("--initial-wrapper-source-pin="))
  ?.slice("--initial-wrapper-source-pin=".length);
const INITIAL_QUERY_WRAPPER_CALL_SITE_COUNT = 59 as const;
const QUERY_FRESH_INDEX_WRAPPERS = [
  "parser_range_for_byte_span",
  "parser_position_for_byte_offset",
  "byte_offset_for_parser_position",
] as const;
const QUERY_LINE_INDEX_CONSTRUCTORS = ["omena_query_line_index", "OmenaLineIndexV0::new"] as const;

const definitionSites: readonly DefinitionSite[] = [
  {
    id: "query-forward",
    relativePath: "rust/crates/omena-query/src/style.rs",
    functionName: "parser_position_for_byte_offset_with_line_index",
    direction: "forward",
  },
  {
    id: "query-reverse",
    relativePath: "rust/crates/omena-query/src/style.rs",
    functionName: "byte_offset_for_parser_position_with_line_index",
    direction: "reverse",
  },
  {
    id: "semantic-forward",
    relativePath: "rust/crates/omena-semantic/src/lib.rs",
    functionName: "parser_position_for_byte_offset_with_line_index",
    direction: "forward",
  },
  {
    id: "lsp-forward",
    relativePath: "rust/crates/omena-lsp-server/src/protocol.rs",
    functionName: "parser_position_for_byte_offset_with_line_index",
    direction: "forward",
  },
  {
    id: "lsp-reverse",
    relativePath: "rust/crates/omena-lsp-server/src/protocol.rs",
    functionName: "byte_offset_for_parser_position_with_line_index",
    direction: "reverse",
  },
  {
    id: "cli-forward",
    relativePath: "rust/crates/omena-cli/src/text_edit.rs",
    functionName: "position_for_byte_offset_with_line_index",
    direction: "forward",
  },
  {
    id: "cli-reverse",
    relativePath: "rust/crates/omena-cli/src/text_edit.rs",
    functionName: "byte_offset_for_position_with_line_index",
    direction: "reverse",
  },
  {
    id: "parser-forward",
    relativePath: "rust/crates/omena-parser/src/public_product.rs",
    functionName: "parser_range_for_byte_span",
    direction: "forward",
  },
];

const productPassSites: readonly ProductPassSite[] = [
  {
    id: "query-hover",
    relativePath: "rust/crates/omena-query/src/style.rs",
    functionName: "summarize_omena_query_style_hover_candidates",
    buildPattern: /omena_query_line_index\s*\(/g,
    conversionPattern: /collect_[a-z_]+_with_line_index|collect_[a-z_]+_from_omena_parser_facts/g,
  },
  {
    id: "query-custom-property-occurrences",
    relativePath: "rust/crates/omena-query/src/style.rs",
    functionName: "summarize_omena_query_custom_property_occurrence_index",
    buildPattern: /omena_query_line_index\s*\(/g,
    conversionPattern: /parser_range_for_byte_span_with_line_index\s*\(/g,
    expectedLoopDepth: 1,
  },
  {
    id: "semantic-boundary",
    relativePath: "rust/crates/omena-semantic/src/lib.rs",
    functionName: "summarize_omena_parser_style_semantic_boundary_with_facts_from_source",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /summarize_omena_parser_(?:contract|semantic)_facts\s*\(/g,
  },
  {
    id: "semantic-declaration-context",
    relativePath: "rust/crates/omena-semantic/src/lib.rs",
    functionName: "collect_parser_declaration_syntax_and_style_context_from_parse",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /summarize_style_context_index\s*\(/g,
  },
  {
    id: "semantic-layer-order",
    relativePath: "rust/crates/omena-semantic/src/lib.rs",
    functionName: "summarize_style_layer_order_from_source",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /summarize_style_context_index\s*\(/g,
  },
  {
    id: "lsp-source-candidates",
    relativePath: "rust/crates/omena-lsp-server/src/source_syntax_index.rs",
    functionName: "source_selector_candidates_from_index",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /source_reference_candidate\s*\(/g,
  },
  {
    id: "lsp-source-diagnostics",
    relativePath: "rust/crates/omena-lsp-server/src/source_diagnostics.rs",
    functionName: "gather_source_diagnostics_render_inputs",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /parser_range_for_byte_span_with_line_index\s*\(/g,
  },
  {
    id: "lsp-source-type-facts",
    relativePath: "rust/crates/omena-lsp-server/src/source_type_facts.rs",
    functionName: "query_engine_input_for_source_type_facts",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /parser_range_for_byte_span_with_line_index\s*\(/g,
  },
  {
    id: "lsp-domain-trace",
    relativePath: "rust/crates/omena-lsp-server/src/source_domain_hover.rs",
    functionName: "source_domain_reference_trace_at_position",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern:
      /(?:source_domain_reference_at_position|parser_range_for_byte_span_with_line_index)\s*\(/g,
  },
  {
    id: "lsp-domain-hover",
    relativePath: "rust/crates/omena-lsp-server/src/source_domain_hover.rs",
    functionName: "source_domain_reference_hover_at_position",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern:
      /(?:source_domain_reference_at_position|parser_range_for_byte_span_with_line_index)\s*\(/g,
  },
  {
    id: "lsp-source-completion",
    relativePath: "rust/crates/omena-lsp-server/src/source_completion.rs",
    functionName: "source_completion_context_at_position",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /byte_offset_for_parser_position_with_line_index\s*\(/g,
  },
  {
    id: "lsp-document-links",
    relativePath: "rust/crates/omena-lsp-server/src/document_links.rs",
    functionName: "resolve_lsp_document_links",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /quoted_occurrence_ranges\s*\(/g,
  },
  {
    id: "lsp-vue-embedded-hover",
    relativePath: "rust/crates/omena-lsp-server/src/query_reuse.rs",
    functionName: "collect_vue_embedded_module_style_indexes",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /embedded_range_to_document_range\s*\(/g,
    expectedBuildCount: 2,
  },
  {
    id: "cli-sass-migration",
    relativePath: "rust/crates/omena-cli/src/migrate/mod.rs",
    functionName: "build_sass_import_to_use_plan_with_oracle",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /range_for_byte_span_with_line_index\s*\(/g,
    expectedLoopDepth: 1,
  },
  {
    id: "cli-selector-migration",
    relativePath: "rust/crates/omena-cli/src/migrate/mod.rs",
    functionName: "build_css_modules_rename_plan",
    buildPattern: /OmenaLineIndexV0::new\s*\(/g,
    conversionPattern: /draft_from_(?:workspace_edit|review_location)\s*\(/g,
    expectedLoopDepth: 1,
  },
];

assert.equal(
  new Set(definitionSites.map((site) => site.id)).size,
  8,
  "the line-index census must enumerate all 5 forward and 3 reverse definition sites",
);
if (injectedScanner !== undefined) {
  assert.ok(
    definitionSites.some((site) => site.id === injectedScanner),
    `unknown scanner injection ${injectedScanner}`,
  );
}

const sourceByPath = new Map<string, string>();
for (const site of definitionSites) {
  if (!sourceByPath.has(site.relativePath)) {
    sourceByPath.set(
      site.relativePath,
      readFileSync(path.join(repoRoot, site.relativePath), "utf8"),
    );
  }
}

for (const site of definitionSites) {
  let body = extractFunctionBody(sourceByPath.get(site.relativePath) ?? "", site.functionName);
  if (site.id === injectedScanner) {
    body += "\nlet _pickup_regression = source.chars().count();";
  }
  assertNoIndependentSourceScan(site, body);
  const delegation =
    site.direction === "forward"
      ? /line_index\s*\.\s*position_for_byte_offset\s*\(/
      : /line_index\s*\.\s*byte_offset_for_position\s*\(/;
  assert.match(body, delegation, `${site.id} does not invoke the shared line-index authority`);
}

assert.ok(productPassSites.length >= 12, "the product pass census must cover at least 12 sites");
if (injectedProductRebuild !== undefined) {
  assert.ok(
    productPassSites.some((site) => site.id === injectedProductRebuild),
    `unknown product rebuild injection ${injectedProductRebuild}`,
  );
}
if (injectedProductLoop !== undefined) {
  assert.ok(
    productPassSites.some((site) => site.id === injectedProductLoop),
    `unknown product loop injection ${injectedProductLoop}`,
  );
}
for (const site of productPassSites) {
  const source =
    sourceByPath.get(site.relativePath) ??
    readFileSync(path.join(repoRoot, site.relativePath), "utf8");
  let body = extractFunctionBody(source, site.functionName);
  if (site.id === injectedProductRebuild) {
    body += site.buildPattern.source.includes("omena_query_line_index")
      ? "\nlet _line_index_regression = omena_query_line_index(source);"
      : "\nlet _line_index_regression = OmenaLineIndexV0::new(source);";
  }
  if (site.id === injectedProductLoop) {
    body = moveFirstMatchIntoSyntheticLoop(body, site.buildPattern);
  }
  const expectedBuildCount = site.expectedBuildCount ?? 1;
  assert.equal(
    countMatches(body, site.buildPattern),
    expectedBuildCount,
    `${site.id} must have exactly ${expectedBuildCount} line-index construction site(s) per source pass`,
  );
  for (const buildOffset of matchOffsets(body, site.buildPattern)) {
    assert.equal(
      loopNestingDepth(body, buildOffset),
      site.expectedLoopDepth ?? 0,
      `${site.id} moved a shared line-index construction across its source/item loop boundary`,
    );
  }
  assert.match(
    body,
    withoutGlobalFlag(site.conversionPattern),
    `${site.id} does not route conversions through its shared line index`,
  );
}

assertLoopNestingScannerSelfTest();
const queryProductionSources = trackedQueryProductionSources();
let queryWrapperCallSites = enumerateFreshIndexCallSites(
  queryProductionSources,
  QUERY_FRESH_INDEX_WRAPPERS,
);
const queryConstructorCallSites = enumerateFreshIndexCallSites(
  queryProductionSources,
  QUERY_LINE_INDEX_CONSTRUCTORS,
);
if (injectedWrapperLoop !== undefined) {
  queryWrapperCallSites = injectCallSiteIntoSyntheticLoop(
    queryProductionSources,
    queryWrapperCallSites,
    injectedWrapperLoop,
  );
}
if (injectedCallerWrapperLoop !== undefined) {
  queryWrapperCallSites = injectCallerCallIntoSyntheticLoop(
    queryProductionSources,
    queryWrapperCallSites,
    injectedCallerWrapperLoop,
  );
}
const intraproceduralLoopResidentQueryWrapperCallSites = queryWrapperCallSites.filter(
  (site) => site.loopNested,
);
const callerLoopResidentQueryWrapperCallSites = queryWrapperCallSites.filter(
  (site) => site.callerLoopNested,
);
const loopResidentQueryWrapperCallSites = queryWrapperCallSites.filter(
  (site) => site.loopNested || site.callerLoopNested,
);
let rustProductionSources = trackedRustProductionSources();
if (injectedTwoFrameWrapperLoop !== undefined) {
  rustProductionSources = injectTwoFrameCallerCallIntoSyntheticLoop(
    rustProductionSources,
    queryWrapperCallSites,
    injectedTwoFrameWrapperLoop,
  );
}
const twoFrameLoopResidentQueryWrapperCallSites = enumerateTwoFrameLoopResidentWrapperSites(
  rustProductionSources,
  queryWrapperCallSites,
);
if (reportTwoFramePopulation) {
  process.stderr.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "omena-syntax.line-index-authority-two-frame-population",
        predicate:
          "wrapper host is called outside a loop by an intermediate function that is called inside a loop",
        wrapperCallSiteCount: twoFrameLoopResidentQueryWrapperCallSites.length,
        wrapperCallSites: twoFrameLoopResidentQueryWrapperCallSites,
      },
      null,
      2,
    )}\n`,
  );
}
assert.deepEqual(
  loopResidentQueryWrapperCallSites,
  [],
  `fresh-index wrapper calls remain inside item loops in their host or one caller frame: ${loopResidentQueryWrapperCallSites
    .map(
      (site) =>
        `${site.id}[host=${site.loopNested},caller=${site.loopNestedCallerCallSiteIds.join("|") || "none"}]`,
    )
    .join(", ")}`,
);
assert.deepEqual(
  twoFrameLoopResidentQueryWrapperCallSites,
  [],
  `fresh-index wrapper calls remain inside item loops two caller frames out: ${twoFrameLoopResidentQueryWrapperCallSites
    .map((site) => `${site.wrapperCallSiteId}[paths=${site.paths.join("|")}]`)
    .join(", ")}`,
);
assert.ok(
  queryWrapperCallSites.length <= INITIAL_QUERY_WRAPPER_CALL_SITE_COUNT,
  `omena-query fresh-index wrapper population grew from the pinned initial ${INITIAL_QUERY_WRAPPER_CALL_SITE_COUNT} sites to ${queryWrapperCallSites.length}`,
);
const recordedBaseline = existsSync(baselinePath)
  ? (JSON.parse(readFileSync(baselinePath, "utf8")) as Partial<LineIndexAuthorityBaselineV0>)
  : undefined;
const initialQueryWrapperSourcePin =
  initialWrapperSourcePinArgument ?? recordedBaseline?.initialQueryWrapperSourcePin;
assert.match(
  initialQueryWrapperSourcePin ?? "",
  /^[0-9a-f]{40}$/u,
  "the initial fresh-index wrapper inventory requires a full source pin",
);
const initialQueryWrapperCallSites =
  initialWrapperSourcePinArgument === undefined
    ? recordedBaseline?.initialQueryWrapperCallSites
    : enumerateFreshIndexCallSites(
        trackedQueryProductionSourcesAtPin(initialWrapperSourcePinArgument),
        QUERY_FRESH_INDEX_WRAPPERS,
      );
assert.ok(initialQueryWrapperCallSites, "the initial fresh-index wrapper inventory is missing");
assert.equal(
  initialQueryWrapperCallSites.length,
  INITIAL_QUERY_WRAPPER_CALL_SITE_COUNT,
  "the initial fresh-index wrapper inventory must contain all 59 measured call sites",
);
const initialQueryWrapperIds = new Set(initialQueryWrapperCallSites.map((site) => site.id));
for (const site of queryWrapperCallSites) {
  assert.ok(
    initialQueryWrapperIds.has(site.id),
    `new fresh-index wrapper call site is outside the initial inventory: ${site.id}`,
  );
}
const migratedQueryWrapperCallSiteCount =
  initialQueryWrapperCallSites.length - queryWrapperCallSites.length;

const syntaxIndexPath = "rust/crates/omena-syntax/src/line_index.rs";
const syntaxIndexSource = readFileSync(path.join(repoRoot, syntaxIndexPath), "utf8");
assert.equal(
  countMatches(syntaxIndexSource, /pub struct OmenaLineIndexV0\b/g),
  1,
  "OmenaLineIndexV0 must have exactly one definition",
);
assert.match(syntaxIndexSource, /line_starts:\s*Vec<u32>/);
assert.match(syntaxIndexSource, /\.partition_point\(/);
assert.match(syntaxIndexSource, /pub fn position_for_byte_offset\b/);
assert.match(syntaxIndexSource, /pub fn byte_offset_for_position\b/);

const parserSource = sourceByPath.get("rust/crates/omena-parser/src/public_product.rs") ?? "";
assert.match(
  parserSource,
  /type SourceLineIndex\s*=\s*OmenaLineIndexV0\s*;/,
  "omena-parser must delegate its prior-art SourceLineIndex name to OmenaLineIndexV0",
);

const querySource = sourceByPath.get("rust/crates/omena-query/src/style.rs") ?? "";
const hoverBody = extractFunctionBody(querySource, "summarize_omena_query_style_hover_candidates");
assert.equal(
  countMatches(hoverBody, /omena_query_line_index\s*\(/g),
  1,
  "the hover candidate pass must build one shared line index per source",
);
for (const collector of [
  "collect_style_selector_hover_candidates_from_omena_parser_facts",
  "collect_custom_property_hover_candidates_from_omena_parser_facts",
  "collect_sass_symbol_hover_candidates_from_omena_parser_facts",
  "collect_sass_partial_evaluator_selector_candidates_from_omena_parser_facts",
]) {
  const collectorBody = extractFunctionBody(querySource, collector);
  assert.doesNotMatch(
    collectorBody,
    /OmenaLineIndexV0::new|omena_query_line_index\s*\(/,
    `${collector} rebuilds the line index per candidate family`,
  );
  assert.match(collectorBody, /parser_range_for_byte_span_with_line_index\s*\(/);
}

const implementationSources = trackedRustSources();
if (!implementationSources.includes(syntaxIndexPath)) {
  implementationSources.push(syntaxIndexPath);
}
const implementationCount = implementationSources
  .map((relativePath) => readFileSync(path.join(repoRoot, relativePath), "utf8"))
  .reduce((count, source) => count + countMatches(source, /impl\s+OmenaLineIndexV0\b/g), 0);
assert.equal(implementationCount, 1, "line-index scanner implementation census must equal one");

const workspaceMemberManifestPaths = cargoWorkspaceMemberManifestPaths();
if (injectWorkspaceMember) {
  workspaceMemberManifestPaths.push("rust/crates/injected-line-index-member/Cargo.toml");
  workspaceMemberManifestPaths.sort();
}
const publicApi = cargoPublicApi();
assert.match(publicApi, /pub struct omena_syntax::OmenaLineIndexV0/);
assert.match(publicApi, /pub fn omena_syntax::OmenaLineIndexV0::position_for_byte_offset/);
assert.match(publicApi, /pub fn omena_syntax::OmenaLineIndexV0::byte_offset_for_position/);

if (writeBaseline) {
  mkdirSync(path.dirname(baselinePath), { recursive: true });
  const baseline: LineIndexAuthorityBaselineV0 = {
    schemaVersion: "0",
    product: "omena-syntax.line-index-authority-baseline",
    workspaceMemberManifestPaths,
    initialQueryWrapperSourcePin,
    initialQueryWrapperCallSiteCount: INITIAL_QUERY_WRAPPER_CALL_SITE_COUNT,
    initialQueryWrapperCallSites,
    migratedQueryWrapperCallSiteCount,
    queryWrapperCallSites,
    queryConstructorCallSites,
  };
  writeFileSync(baselinePath, `${JSON.stringify(baseline, null, 2)}\n`);
  writeFileSync(publicApiSnapshotPath, publicApi);
} else {
  assert.ok(existsSync(baselinePath), "line-index authority baseline is missing; run with --write");
  assert.ok(
    existsSync(publicApiSnapshotPath),
    "omena-syntax public API snapshot is missing; run with --write",
  );
  const baseline = JSON.parse(readFileSync(baselinePath, "utf8")) as LineIndexAuthorityBaselineV0;
  assert.deepEqual(
    workspaceMemberManifestPaths,
    baseline.workspaceMemberManifestPaths,
    "Cargo workspace membership changed; line-index authority forbids adding a crate in this slice",
  );
  assert.equal(
    baseline.initialQueryWrapperCallSiteCount,
    INITIAL_QUERY_WRAPPER_CALL_SITE_COUNT,
    "the initial fresh-index wrapper population pin changed",
  );
  assert.equal(
    baseline.initialQueryWrapperSourcePin,
    initialQueryWrapperSourcePin,
    "the initial fresh-index wrapper source pin changed",
  );
  assert.deepEqual(
    baseline.initialQueryWrapperCallSites,
    initialQueryWrapperCallSites,
    "the initial fresh-index wrapper call-site inventory changed",
  );
  assert.equal(
    baseline.migratedQueryWrapperCallSiteCount,
    migratedQueryWrapperCallSiteCount,
    "the migrated fresh-index wrapper population changed; refresh only after reviewing every site",
  );
  assert.deepEqual(
    queryWrapperCallSites,
    baseline.queryWrapperCallSites,
    "omena-query fresh-index wrapper call sites changed; review and run the official writer",
  );
  assert.deepEqual(
    queryConstructorCallSites,
    baseline.queryConstructorCallSites,
    "omena-query line-index constructor call sites changed; review and run the official writer",
  );
  assert.equal(
    publicApi,
    normalizeOutput(readFileSync(publicApiSnapshotPath, "utf8")),
    "omena-syntax public API changed without an explicit line-index snapshot update",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-syntax.line-index-authority",
      forwardDefinitionSiteCount: 5,
      reverseDefinitionSiteCount: 3,
      implementationCount,
      hoverLineIndexBuildsPerSource: 1,
      sharedProductPassSiteCount: productPassSites.length,
      initialQueryWrapperCallSiteCount: INITIAL_QUERY_WRAPPER_CALL_SITE_COUNT,
      initialLoopResidentQueryWrapperCallSiteCount: initialQueryWrapperCallSites.filter(
        (site) => site.loopNested,
      ).length,
      migratedQueryWrapperCallSiteCount,
      survivingQueryWrapperCallSiteCount: queryWrapperCallSites.length,
      intraproceduralLoopResidentQueryWrapperCallSiteCount:
        intraproceduralLoopResidentQueryWrapperCallSites.length,
      callerLoopResidentQueryWrapperCallSiteCount: callerLoopResidentQueryWrapperCallSites.length,
      twoCallerFrameLoopResidentQueryWrapperCallSiteCount:
        twoFrameLoopResidentQueryWrapperCallSites.length,
      loopResidentQueryWrapperCallSiteCount: loopResidentQueryWrapperCallSites.length,
      queryConstructorCallSiteCount: queryConstructorCallSites.length,
      workspaceMemberCount: workspaceMemberManifestPaths.length,
      publicHelper: "OmenaLineIndexV0",
    },
    null,
    2,
  )}\n`,
);

function assertNoIndependentSourceScan(site: DefinitionSite, body: string): void {
  const forbidden = [
    /\.char_indices\s*\(/,
    /\.bytes\s*\(\)\s*\.filter/,
    /\.rfind\s*\(\s*['"]\\n['"]\s*\)/,
    /\.encode_utf16\s*\(\)\s*\.count\s*\(/,
    /source\s*\.\s*chars\s*\(\)\s*\.\s*count\s*\(/,
  ];
  for (const pattern of forbidden) {
    assert.doesNotMatch(body, pattern, `${site.id} restored an independent source scanner`);
  }
}

function extractFunctionBody(source: string, functionName: string): string {
  const signature = new RegExp(`\\bfn\\s+${functionName}\\b`);
  const match = signature.exec(source);
  assert.ok(match, `missing function ${functionName}`);
  const open = source.indexOf("{", match.index);
  assert.ok(open >= 0, `missing body for ${functionName}`);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(open + 1, index);
    }
  }
  throw new Error(`unterminated body for ${functionName}`);
}

function moveFirstMatchIntoSyntheticLoop(source: string, pattern: RegExp): string {
  const [offset] = matchOffsets(source, pattern);
  assert.notEqual(offset, undefined, `cannot inject loop around missing ${pattern.source}`);
  const matched = withoutGlobalFlag(pattern).exec(source.slice(offset));
  assert.ok(matched, `cannot recover injected match for ${pattern.source}`);
  const end = offset + matched[0].length;
  return `${source.slice(0, offset)}for _item in _items { ${source.slice(offset, end)} }${source.slice(end)}`;
}

function matchOffsets(source: string, pattern: RegExp): number[] {
  const globalPattern = new RegExp(
    pattern.source,
    pattern.flags.includes("g") ? pattern.flags : `${pattern.flags}g`,
  );
  return [...source.matchAll(globalPattern)].map((match) => match.index);
}

function assertLoopNestingScannerSelfTest(): void {
  const outside =
    "let line_index = OmenaLineIndexV0::new(source); for item in items { use(item); }";
  assert.equal(isLoopNested(outside, outside.indexOf("OmenaLineIndexV0")), false);
  const direct = "for item in items { let line_index = OmenaLineIndexV0::new(item); }";
  assert.equal(isLoopNested(direct, direct.indexOf("OmenaLineIndexV0")), true);
  const iterator = "items.iter().map(|item| parser_range_for_byte_span(source, item.span))";
  assert.equal(isLoopNested(iterator, iterator.indexOf("parser_range_for_byte_span")), true);
}

function isLoopNested(source: string, offset: number): boolean {
  return loopNestingDepth(source, offset) > 0;
}

function loopNestingDepth(source: string, offset: number): number {
  return loopRegions(maskRustLexemes(source)).filter(
    ({ start, end }) => start <= offset && offset < end,
  ).length;
}

function loopRegions(maskedSource: string): { readonly start: number; readonly end: number }[] {
  const regions: { start: number; end: number }[] = [];
  for (const match of maskedSource.matchAll(/\b(?:for|while|loop)\b/g)) {
    const open = maskedSource.indexOf("{", match.index);
    if (open < 0) continue;
    const statementEnd = maskedSource.indexOf(";", match.index);
    if (statementEnd >= 0 && statementEnd < open) continue;
    const close = findMatchingDelimiter(maskedSource, open, "{", "}");
    if (close >= 0) regions.push({ start: open + 1, end: close });
  }

  const iteratorMethods =
    /\.(?:all|and_then|any|dedup_by|filter|filter_map|find|find_map|flat_map|fold|for_each|get_or_insert_with|inspect|map|map_while|or_else|position|retain|rposition|scan|skip_while|sort_by|sort_by_key|take_while|try_fold|unwrap_or_else)\s*\(/g;
  for (const match of maskedSource.matchAll(iteratorMethods)) {
    const open = maskedSource.indexOf("(", match.index);
    if (open < 0) continue;
    const close = findMatchingDelimiter(maskedSource, open, "(", ")");
    if (close < 0) continue;
    const argument = maskedSource.slice(open + 1, close);
    const closure = /(?:^|,)\s*(?:move\s+)?(?:\|\||\|[^|]*\|)/m.exec(argument);
    if (closure === null) continue;
    const marker = closure[0];
    const pipeEndInMarker = Math.max(marker.lastIndexOf("||") + 2, marker.lastIndexOf("|") + 1);
    regions.push({ start: open + 1 + closure.index + pipeEndInMarker, end: close });
  }
  return regions;
}

function findMatchingDelimiter(
  source: string,
  openOffset: number,
  openCharacter: string,
  closeCharacter: string,
): number {
  let depth = 0;
  for (let index = openOffset; index < source.length; index += 1) {
    if (source[index] === openCharacter) depth += 1;
    if (source[index] === closeCharacter) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function trackedQueryProductionSources(): RustSourceFile[] {
  const relativePaths = execFileSync(
    "git",
    ["ls-files", "rust/crates/omena-query/src/*.rs", "rust/crates/omena-query/src/**/*.rs"],
    { cwd: repoRoot, encoding: "utf8" },
  )
    .trim()
    .split("\n")
    .filter(Boolean)
    .sort();
  const sourceFiles = relativePaths.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  const testOnlyPaths = testOnlyRustModulePaths(sourceFiles);
  return sourceFiles.filter(({ relativePath }) => !testOnlyPaths.has(relativePath));
}

function trackedRustProductionSources(): RustSourceFile[] {
  const sourceFiles = trackedRustSources().map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  const testOnlyPaths = testOnlyRustModulePaths(sourceFiles);
  return sourceFiles.filter(({ relativePath }) => !testOnlyPaths.has(relativePath));
}

function trackedQueryProductionSourcesAtPin(gitSha: string): RustSourceFile[] {
  return trackedQueryProductionSources().map(({ relativePath }) => ({
    relativePath,
    source: execFileSync("git", ["show", `${gitSha}:${relativePath}`], {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    }),
  }));
}

function enumerateFreshIndexCallSites(
  sourceFiles: readonly RustSourceFile[],
  callNames: readonly string[],
): FreshIndexCallSite[] {
  const sites: FreshIndexCallSite[] = [];
  const callerIdsByFunction = new Map<string, readonly string[]>();
  for (const { relativePath, source } of sourceFiles) {
    const maskedSource = maskCfgTestItems(source);
    const spans = rustFunctionSpans(maskedSource);
    for (const span of spans) {
      const body = maskedSource.slice(span.bodyStart, span.bodyEnd);
      for (const wrapperName of callNames) {
        const escapedName = wrapperName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        const offsets = matchOffsets(body, new RegExp(`\\b${escapedName}\\s*\\(`, "g"));
        for (const [index, bodyOffset] of offsets.entries()) {
          const sourceOffset = span.bodyStart + bodyOffset;
          const loopNestedCallerCallSiteIds =
            callerIdsByFunction.get(span.name) ??
            enumerateLoopNestedCallerCallSiteIds(sourceFiles, span.name);
          callerIdsByFunction.set(span.name, loopNestedCallerCallSiteIds);
          sites.push({
            id: `${relativePath}:${span.name}:${wrapperName}:${index + 1}`,
            relativePath,
            functionName: span.name,
            wrapperName,
            occurrenceWithinFunction: index + 1,
            loopNested: isLoopNested(body, bodyOffset),
            callerLoopNested: loopNestedCallerCallSiteIds.length > 0,
            loopNestedCallerCallSiteIds,
          });
          assert.ok(
            sourceOffset < span.bodyEnd,
            `${relativePath}:${span.name} call-site offset escaped its function`,
          );
        }
      }
    }
  }
  return sites.sort((left, right) => (left.id < right.id ? -1 : left.id > right.id ? 1 : 0));
}

function testOnlyRustModulePaths(sourceFiles: readonly RustSourceFile[]): Set<string> {
  const availablePaths = new Set(sourceFiles.map(({ relativePath }) => relativePath));
  const testOnlyPaths = new Set<string>();
  const declarationPattern =
    /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g;
  for (const { relativePath, source } of sourceFiles) {
    const fileName = path.posix.basename(relativePath, ".rs");
    const moduleDirectory = ["lib", "main", "mod"].includes(fileName)
      ? path.posix.dirname(relativePath)
      : path.posix.join(path.posix.dirname(relativePath), fileName);
    for (const match of maskRustLexemes(source).matchAll(declarationPattern)) {
      const moduleBase = path.posix.join(moduleDirectory, match[1]);
      for (const candidate of [`${moduleBase}.rs`, `${moduleBase}/mod.rs`]) {
        if (availablePaths.has(candidate)) testOnlyPaths.add(candidate);
      }
      for (const candidate of availablePaths) {
        if (candidate.startsWith(`${moduleBase}/`)) testOnlyPaths.add(candidate);
      }
    }
  }
  return testOnlyPaths;
}

function enumerateLoopNestedCallerCallSiteIds(
  sourceFiles: readonly RustSourceFile[],
  calleeName: string,
): string[] {
  const callerIds: string[] = [];
  const escapedName = calleeName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const callPattern = new RegExp(`\\b${escapedName}\\s*\\(`, "g");
  for (const { relativePath, source } of sourceFiles) {
    const maskedSource = maskCfgTestItems(source);
    for (const span of rustFunctionSpans(maskedSource)) {
      const body = maskedSource.slice(span.bodyStart, span.bodyEnd);
      const offsets = matchOffsets(body, callPattern);
      for (const [index, bodyOffset] of offsets.entries()) {
        if (isLoopNested(body, bodyOffset)) {
          callerIds.push(`${relativePath}:${span.name}:${calleeName}:${index + 1}`);
        }
      }
    }
  }
  return callerIds.sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
}

function enumerateTwoFrameLoopResidentWrapperSites(
  sourceFiles: readonly RustSourceFile[],
  wrapperCallSites: readonly FreshIndexCallSite[],
): TwoFrameLoopResidentWrapperSite[] {
  const callSitesByCallee = new Map<string, readonly RustFunctionCallSite[]>();
  const callsTo = (calleeName: string): readonly RustFunctionCallSite[] => {
    const recorded = callSitesByCallee.get(calleeName);
    if (recorded !== undefined) return recorded;
    const callSites = enumerateRustFunctionCallSites(sourceFiles, calleeName);
    callSitesByCallee.set(calleeName, callSites);
    return callSites;
  };

  return wrapperCallSites
    .map((wrapperCallSite) => {
      const paths = callsTo(wrapperCallSite.functionName)
        .filter((intermediateCall) => !intermediateCall.loopNested)
        .flatMap((intermediateCall) =>
          callsTo(intermediateCall.functionName)
            .filter((outerCall) => outerCall.loopNested)
            .map((outerCall) => `${outerCall.id} -> ${intermediateCall.id}`),
        )
        .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
      return {
        wrapperCallSiteId: wrapperCallSite.id,
        paths: [...new Set(paths)],
      };
    })
    .filter((site) => site.paths.length > 0)
    .sort((left, right) =>
      left.wrapperCallSiteId < right.wrapperCallSiteId
        ? -1
        : left.wrapperCallSiteId > right.wrapperCallSiteId
          ? 1
          : 0,
    );
}

function enumerateRustFunctionCallSites(
  sourceFiles: readonly RustSourceFile[],
  calleeName: string,
): RustFunctionCallSite[] {
  const escapedName = calleeName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const callPattern = new RegExp(`\\b${escapedName}\\s*\\(`, "g");
  const callSites: RustFunctionCallSite[] = [];
  for (const { relativePath, source } of sourceFiles) {
    const maskedSource = maskCfgTestItems(source);
    for (const span of rustFunctionSpans(maskedSource)) {
      const body = maskedSource.slice(span.bodyStart, span.bodyEnd);
      for (const [index, bodyOffset] of matchOffsets(body, callPattern).entries()) {
        callSites.push({
          id: `${relativePath}:${span.name}:${calleeName}:${index + 1}`,
          relativePath,
          functionName: span.name,
          calleeName,
          loopNested: isLoopNested(body, bodyOffset),
        });
      }
    }
  }
  return callSites.sort((left, right) => (left.id < right.id ? -1 : left.id > right.id ? 1 : 0));
}

function injectTwoFrameCallerCallIntoSyntheticLoop(
  sourceFiles: readonly RustSourceFile[],
  wrapperCallSites: readonly FreshIndexCallSite[],
  id: string,
): RustSourceFile[] {
  const target = wrapperCallSites.find((site) => site.id === id);
  assert.ok(target, `unknown two-frame wrapper loop injection ${id}`);
  const intermediateCall = enumerateRustFunctionCallSites(sourceFiles, target.functionName).find(
    (site) => !site.loopNested && site.functionName !== target.functionName,
  );
  assert.ok(intermediateCall, `no flat intermediate caller is available for ${id}`);

  const escapedName = intermediateCall.functionName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const callPattern = new RegExp(`\\b${escapedName}\\s*\\(`, "g");
  for (const sourceFile of sourceFiles) {
    const maskedSource = maskCfgTestItems(sourceFile.source);
    for (const span of rustFunctionSpans(maskedSource)) {
      if (span.name === intermediateCall.functionName) continue;
      const body = sourceFile.source.slice(span.bodyStart, span.bodyEnd);
      const bodyOffset = matchOffsets(body, callPattern).find(
        (offset) => !isLoopNested(body, offset),
      );
      if (bodyOffset === undefined) continue;
      const injectedBody = `${body.slice(0, bodyOffset)}for _item in _items { ${body.slice(bodyOffset)}`;
      const injectedSource = `${sourceFile.source.slice(0, span.bodyStart)}${injectedBody}${sourceFile.source.slice(span.bodyEnd)}}`;
      return sourceFiles.map((entry) =>
        entry.relativePath === sourceFile.relativePath
          ? { ...entry, source: injectedSource }
          : entry,
      );
    }
  }
  throw new Error(`no flat second-frame caller is available for ${id}`);
}

function injectCallSiteIntoSyntheticLoop(
  sourceFiles: readonly RustSourceFile[],
  callSites: readonly FreshIndexCallSite[],
  id: string,
): FreshIndexCallSite[] {
  const target = callSites.find((site) => site.id === id);
  assert.ok(target, `unknown wrapper loop injection ${id}`);
  const sourceFile = sourceFiles.find((entry) => entry.relativePath === target.relativePath);
  assert.ok(sourceFile, `missing source for wrapper loop injection ${id}`);
  const maskedSource = maskCfgTestItems(sourceFile.source);
  const span = rustFunctionSpans(maskedSource).find((entry) => entry.name === target.functionName);
  assert.ok(span, `missing function for wrapper loop injection ${id}`);
  const body = sourceFile.source.slice(span.bodyStart, span.bodyEnd);
  const escapedName = target.wrapperName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const offsets = matchOffsets(body, new RegExp(`\\b${escapedName}\\s*\\(`, "g"));
  const bodyOffset = offsets[target.occurrenceWithinFunction - 1];
  assert.notEqual(bodyOffset, undefined, `missing call for wrapper loop injection ${id}`);
  const injectedBody = `${body.slice(0, bodyOffset)}for _item in _items { ${body.slice(bodyOffset)}`;
  const injectedSource = `${sourceFile.source.slice(0, span.bodyStart)}${injectedBody}${sourceFile.source.slice(span.bodyEnd)}}`;
  return enumerateFreshIndexCallSites(
    sourceFiles.map((entry) =>
      entry.relativePath === target.relativePath ? { ...entry, source: injectedSource } : entry,
    ),
    QUERY_FRESH_INDEX_WRAPPERS,
  );
}

function injectCallerCallIntoSyntheticLoop(
  sourceFiles: readonly RustSourceFile[],
  callSites: readonly FreshIndexCallSite[],
  id: string,
): FreshIndexCallSite[] {
  const target = callSites.find((site) => site.id === id);
  assert.ok(target, `unknown caller wrapper loop injection ${id}`);
  const escapedName = target.functionName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const callPattern = new RegExp(`\\b${escapedName}\\s*\\(`, "g");
  for (const sourceFile of sourceFiles) {
    const maskedSource = maskCfgTestItems(sourceFile.source);
    for (const span of rustFunctionSpans(maskedSource)) {
      if (span.name === target.functionName) continue;
      const body = sourceFile.source.slice(span.bodyStart, span.bodyEnd);
      const bodyOffset = matchOffsets(body, callPattern).find(
        (offset) => !isLoopNested(body, offset),
      );
      if (bodyOffset === undefined) continue;
      const injectedBody = `${body.slice(0, bodyOffset)}for _item in _items { ${body.slice(bodyOffset)}`;
      const injectedSource = `${sourceFile.source.slice(0, span.bodyStart)}${injectedBody}${sourceFile.source.slice(span.bodyEnd)}}`;
      return enumerateFreshIndexCallSites(
        sourceFiles.map((entry) =>
          entry.relativePath === sourceFile.relativePath
            ? { ...entry, source: injectedSource }
            : entry,
        ),
        QUERY_FRESH_INDEX_WRAPPERS,
      );
    }
  }
  throw new Error(`no non-loop caller is available for caller wrapper loop injection ${id}`);
}

function rustFunctionSpans(source: string): RustFunctionSpan[] {
  const maskedSource = maskRustLexemes(source);
  const spans: RustFunctionSpan[] = [];
  for (const match of maskedSource.matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b/g)) {
    let open = maskedSource.indexOf("{", match.index);
    const semicolon = maskedSource.indexOf(";", match.index);
    if (open < 0 || (semicolon >= 0 && semicolon < open)) continue;
    const close = findMatchingDelimiter(maskedSource, open, "{", "}");
    assert.ok(close >= 0, `unterminated Rust function ${match[1]}`);
    spans.push({ name: match[1], bodyStart: open + 1, bodyEnd: close });
  }
  return spans;
}

function maskCfgTestItems(source: string): string {
  const masked = [...maskRustLexemes(source)];
  const snapshot = masked.join("");
  for (const match of snapshot.matchAll(/#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g)) {
    let cursor = match.index + match[0].length;
    while (/\s/.test(snapshot[cursor] ?? "")) cursor += 1;
    const open = snapshot.indexOf("{", cursor);
    const semicolon = snapshot.indexOf(";", cursor);
    const comma = snapshot.indexOf(",", cursor);
    let end = [semicolon, comma]
      .filter((offset) => offset >= 0)
      .reduce((smallest, offset) => Math.min(smallest, offset), Number.POSITIVE_INFINITY);
    if (open >= 0 && open < end) {
      const close = findMatchingDelimiter(snapshot, open, "{", "}");
      if (close >= 0) end = close;
    }
    if (!Number.isFinite(end)) continue;
    for (let index = match.index; index <= end; index += 1) {
      if (masked[index] !== "\n") masked[index] = " ";
    }
  }
  return masked.join("");
}

function maskRustLexemes(source: string): string {
  const output = [...source];
  const blank = (start: number, end: number): void => {
    for (let index = start; index < end; index += 1) {
      if (output[index] !== "\n") output[index] = " ";
    }
  };
  for (let index = 0; index < source.length;) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      blank(index, end < 0 ? source.length : end);
      index = end < 0 ? source.length : end;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let depth = 1;
      let cursor = index + 2;
      while (cursor < source.length && depth > 0) {
        if (source.startsWith("/*", cursor)) {
          depth += 1;
          cursor += 2;
        } else if (source.startsWith("*/", cursor)) {
          depth -= 1;
          cursor += 2;
        } else {
          cursor += 1;
        }
      }
      blank(index, cursor);
      index = cursor;
      continue;
    }
    const raw = /^(?:br|r)(#+)?"/.exec(source.slice(index));
    if (raw !== null) {
      const hashes = raw[1] ?? "";
      const delimiter = `"${hashes}`;
      const end = source.indexOf(delimiter, index + raw[0].length);
      const cursor = end < 0 ? source.length : end + delimiter.length;
      blank(index, cursor);
      index = cursor;
      continue;
    }
    const stringPrefixLength = source.startsWith('b"', index) ? 2 : source[index] === '"' ? 1 : 0;
    if (stringPrefixLength > 0) {
      let cursor = index + stringPrefixLength;
      while (cursor < source.length) {
        if (source[cursor] === "\\") cursor += 2;
        else if (source[cursor] === '"') {
          cursor += 1;
          break;
        } else cursor += 1;
      }
      blank(index, cursor);
      index = cursor;
      continue;
    }
    if (source[index] === "'") {
      let cursor = index + 1;
      if (source[cursor] === "\\") cursor += 2;
      else cursor += 1;
      if (source[cursor] === "'") {
        blank(index, cursor + 1);
        index = cursor + 1;
        continue;
      }
    }
    index += 1;
  }
  return output.join("");
}

function trackedRustSources(): string[] {
  return execFileSync("git", ["ls-files", "rust/crates/*/src/*.rs", "rust/crates/*/src/**/*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
  })
    .trim()
    .split("\n")
    .filter(Boolean);
}

function cargoWorkspaceMemberManifestPaths(): string[] {
  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--manifest-path", "rust/Cargo.toml", "--no-deps", "--format-version", "1"],
      {
        cwd: repoRoot,
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
      },
    ),
  ) as {
    readonly workspace_members: readonly string[];
    readonly packages: readonly { readonly id: string; readonly manifest_path: string }[];
  };
  const members = new Set(metadata.workspace_members);
  return metadata.packages
    .filter((entry) => members.has(entry.id))
    .map((entry) => path.relative(repoRoot, entry.manifest_path).replaceAll(path.sep, "/"))
    .sort();
}

function cargoPublicApi(): string {
  const output = execFileSync(
    "cargo",
    [
      "public-api",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-syntax",
      "-sss",
      "--color",
      "never",
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  return normalizeOutput(output);
}

function normalizeOutput(value: string): string {
  return `${value.replaceAll("\r\n", "\n").trimEnd()}\n`;
}

function countMatches(source: string, pattern: RegExp): number {
  return [...source.matchAll(pattern)].length;
}

function withoutGlobalFlag(pattern: RegExp): RegExp {
  return new RegExp(pattern.source, pattern.flags.replace("g", ""));
}

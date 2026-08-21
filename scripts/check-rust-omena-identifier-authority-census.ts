import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { maskRustCfgTestItems } from "./lib/rust-cfg-test-mask";

type Disposition = "sanctioned" | "named-exempt" | "unclassified";
type PrimitiveId = "str-eq" | "contains" | "insert" | "map-get" | "cmp";

interface ProductPathMatrix {
  readonly schemaVersion: "0";
  readonly product: "omena-css.product-path-matrix";
  readonly entries: readonly {
    readonly crate: string;
    readonly role: string;
  }[];
}

interface CensusSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly operation: string;
  readonly evidence: string;
  readonly disposition: Exclude<Disposition, "unclassified">;
  readonly reason?: string;
}

interface DiscoveredSite {
  readonly path: string;
  readonly line: number;
  readonly function: string;
  readonly operation: string;
  readonly evidence: string;
}

interface IdentifierAuthorityCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.identifier-authority.census";
  readonly policy: {
    readonly expectedSide: "committed-authored-table";
    readonly addedSiteAdoption: "forbidden";
    readonly owningCheck: "rust/omena-syntax-authority-raw-scan-census";
    readonly packageScript: "check:rust-omena-syntax-authority-raw-scan-census";
  };
  readonly sourceRoots: readonly ["rust/crates"];
  readonly scope: {
    readonly scannedCrates: readonly string[];
    readonly scannedCratesDigest: string;
    readonly engineCrates: readonly string[];
    readonly engineCratesDigest: string;
  };
  readonly preflight: {
    readonly productPin: "c18606086dc77101ad162c8d0ddb36ccae7e9d85";
    readonly measuredSiteCount: 50;
    readonly primitiveCounts: {
      readonly "str-eq": 24;
      readonly contains: 9;
      readonly insert: 10;
      readonly "map-get": 5;
      readonly cmp: 2;
    };
  };
  readonly typeSeal: {
    readonly path: "rust/crates/omena-syntax/src/ident.rs";
    readonly classNameDerives: readonly ["Debug", "Clone"];
    readonly classNameEqualityDerives: readonly [];
    readonly canonicalKeyDerives: readonly [
      "Debug",
      "Clone",
      "PartialEq",
      "Eq",
      "PartialOrd",
      "Ord",
      "Hash",
    ];
    readonly canonicalKeyFieldVisibility: "private";
    readonly canonicalKeyConstructor: "ClassNameV0::canonical_key";
    readonly equalityMethod: "ClassNameV0::same_as";
    readonly mutatorCrates: readonly string[];
    readonly privacyDiscriminator: string;
  };
  readonly egress: {
    readonly direction: "decrease-only";
    readonly baselineSiteCount: number;
    readonly currentSiteCount: number;
    readonly sites: readonly CensusSite[];
    readonly siteDigest: string;
  };
  readonly idiom: {
    readonly granularity: "line";
    readonly lexemeGlobs: readonly string[];
    readonly primitiveIds: readonly PrimitiveId[];
    readonly unlabelledBindingBlindSpot: string;
    readonly currentSiteCount: number;
    readonly primitiveCounts: Readonly<Record<PrimitiveId, number>>;
    readonly sanctionedSiteCount: number;
    readonly namedExemptSiteCount: number;
    readonly sites: readonly CensusSite[];
    readonly siteDigest: string;
  };
  readonly predicateCopies: {
    readonly derivation: "direct-character-predicate-body";
    readonly blindSpots: readonly string[];
    readonly currentSiteCount: number;
    readonly sites: readonly CensusSite[];
    readonly siteDigest: string;
  };
}

interface ExemptionRule {
  readonly path: string;
  readonly function: string;
  readonly operation: string;
  readonly evidence: string;
  readonly reason: string;
  readonly disposition?: Exclude<Disposition, "unclassified">;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const censusPath = path.join(repoRoot, "rust/omena-identifier-authority-census.json");
const writeMode = process.argv.includes("--write");
const injectClassNameEquality =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CLASSNAME_EQUALITY === "1";
const injectEgress = process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EGRESS === "1";
const injectLabelledComparison =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_LABELLED_COMPARISON === "1";
const injectUnlabelledComparison =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_UNLABELLED_COMPARISON === "1";
const injectPredicateCopy =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY === "1";
const injectPredicateCopyExplicit =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY_EXPLICIT === "1";
const injectPredicateCopyReversed =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY_REVERSED === "1";

const sourceRoots = ["rust/crates"] as const;
const lexemeGlobs = [
  "*_class_name",
  "*_class_names",
  "*_selector_name",
  "*_selector_names",
  "*_selector_universe",
  "required_class",
  "required_classes",
] as const;
const primitivePatterns = [
  ["str-eq", /(?:==|!=)/u],
  ["contains", /\.contains\s*\(/u],
  ["insert", /\.insert\s*\(/u],
  ["map-get", /\.get(?:_key_value)?\s*\(/u],
  ["cmp", /\.cmp\s*\(/u],
] as const satisfies readonly (readonly [PrimitiveId, RegExp])[];
const classBindingLexeme =
  /\b[A-Za-z0-9_]*(?:class_names?|selector_names?|selector_universe|required_class(?:es)?)[A-Za-z0-9_]*\b/u;
const mutatorCrates = [
  "omena-cascade",
  "omena-query",
  "omena-semantic",
  "omena-transform-passes",
  "omena-bridge",
  "omena-lsp-server",
  "omena-abstract-value",
  "omena-bundler",
] as const;

const egressExemptions: readonly ExemptionRule[] = [
  {
    path: "rust/crates/omena-parser/src/public_product.rs",
    function: "class_names_in_selector",
    operation: "ClassNameV0::into_raw",
    evidence: "let name = entry.name.into_raw();",
    reason:
      "The parser product preserves authored selector spelling together with its source span.",
  },
  {
    path: "rust/crates/omena-query/src/style/cascade_checker/runtime_state.rs",
    function: "query_selector_class_names",
    operation: "ClassNameV0::into_raw",
    evidence: "names.insert(entry.name.into_raw());",
    reason:
      "Cascade diagnostics preserve authored selector spelling after shared lexical extraction.",
  },
] as const;

const idiomExemptions: readonly ExemptionRule[] = [
  {
    path: "rust/crates/omena-cascade-proof/src/proof_kernel.rs",
    function: "module_export_observation_map_v0",
    operation: "str-eq",
    evidence: "|| canonical_class_name.decoded() != observation.key.canonical_class_name",
    reason:
      "The proof kernel compares a certificate identity with the spelling decoded by the shared syntax authority.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-bundler/src/lib.rs",
    function: "instance_reachability_inputs_closed_over_composes",
    operation: "str-eq",
    evidence: "changed |= target.class_names.len() != before;",
    reason: "This compares set cardinalities to detect fixed-point convergence, not identifiers.",
  },
  {
    path: "rust/crates/omena-parser/src/facts/selectors.rs",
    function: "resolve_selector_group",
    operation: "str-eq",
    evidence: "&& class_names.len() == 1",
    reason: "This is a cardinality check, not class-name equality.",
  },
  {
    path: "rust/crates/omena-lsp-server/src/code_actions.rs",
    function: "resolve_lsp_code_actions",
    operation: "map-get",
    evidence: 'let selector_name = payload.get("selectorName").and_then(Value::as_str)?;',
    reason: "This reads a JSON field name rather than a class-name map key.",
  },
  {
    path: "rust/crates/omena-query-core/src/lib.rs",
    function: "selector_universe_for_targets",
    operation: "map-get",
    evidence: "if let Some(selector_names) = style_selectors_by_path.get(target_style_path) {",
    reason: "This lookup is keyed by a normalized style path.",
  },
  {
    path: "rust/crates/engine-style-parser/src/lib.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_local_composes: local_selector_names.contains(selector),",
    reason: "The legacy differential parser remains outside product identifier authority.",
  },
  {
    path: "rust/crates/engine-style-parser/src/lib.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_imported_composes: imported_selector_names.contains(selector),",
    reason: "The legacy differential parser remains outside product identifier authority.",
  },
  {
    path: "rust/crates/engine-style-parser/src/lib.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_global_composes: global_selector_names.contains(selector),",
    reason: "The legacy differential parser remains outside product identifier authority.",
  },
  {
    path: "rust/crates/omena-semantic/src/css_modules_cross_file.rs",
    function: "collect_css_modules_composes_adjacency",
    operation: "str-eq",
    evidence: "let target_class_names = if target_style_path == *style_path {",
    reason: "This compares normalized style paths before selecting a class-name set.",
  },
  {
    path: "rust/crates/omena-semantic/src/source_evidence.rs",
    function: "selector_certainty_reason",
    operation: "str-eq",
    evidence: "if payload.selector_names.len() == 1 {",
    reason: "This is a cardinality check, not selector-name equality.",
  },
  {
    path: "rust/crates/omena-bridge/src/source_syntax.rs",
    function: "retire_template_prefix_reference",
    operation: "str-eq",
    evidence: "|| reference.selector_name.as_deref() != Some(prefix)",
    reason: "This retires an exact template prefix rather than joining complete class names.",
  },
  {
    path: "rust/crates/omena-diff-test/src/linked_emission.rs",
    function: "validate_authored_liveness_expectations_v0",
    operation: "contains",
    evidence: ".contains(expectation.class_name);",
    reason: "The differential harness compares authored expectation data with measured output.",
  },
  {
    path: "rust/crates/omena-diff-test/src/linked_emission.rs",
    function: "validate_authored_liveness_expectations_v0",
    operation: "contains",
    evidence: ".is_some_and(|names| names.contains(expectation.class_name));",
    reason: "The differential harness compares authored expectation data with measured output.",
  },
  {
    path: "rust/crates/omena-diff-test/src/linked_emission.rs",
    function: "rule_declaration_counts_for_class_v0",
    operation: "str-eq",
    evidence: "&& selector.name == class_name",
    reason: "The differential harness compares two already-emitted selector spellings.",
  },
  {
    path: "rust/crates/omena-lsp-server/src/source_document_cache.rs",
    function: "class_value_universes_from_value",
    operation: "map-get",
    evidence: 'class_names: strings_from_value(universe.get("classNames")?)?,',
    reason: "This reads a serialized field name rather than a class-name map key.",
  },
  {
    path: "rust/crates/omena-lsp-server/src/source_document_cache.rs",
    function: "source_elements_from_value",
    operation: "map-get",
    evidence: 'static_class_names: match fact.get("staticClassNames") {',
    reason: "This reads a serialized field name rather than a class-name map key.",
  },
  {
    path: "rust/crates/omena-lsp-server/src/source_type_facts.rs",
    function: "take_type_fact_prefix_references",
    operation: "str-eq",
    evidence: "&& reference.selector_name.as_deref() == Some(prefix)",
    reason: "This consumes an exact prefix reference rather than joining complete class names.",
  },
  {
    path: "rust/crates/omena-parser/src/closed_world/contract.rs",
    function: "composes_origin_symbol_is_reachable",
    operation: "str-eq",
    evidence: ".is_some_and(|class_names| class_names.iter().any(|name| name == symbol)),",
    reason: "Both operands come from the shared parser class-name extraction.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-parser/src/public_product.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_local_composes: local_selector_names.contains(selector),",
    reason: "Both operands come from the shared parser class-name extraction.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-parser/src/public_product.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_imported_composes: imported_selector_names.contains(selector),",
    reason: "Both operands come from the shared parser class-name extraction.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-parser/src/public_product.rs",
    function: "summarize_parser_evaluator_candidates",
    operation: "contains",
    evidence: "has_global_composes: global_selector_names.contains(selector),",
    reason: "Both operands come from the shared parser class-name extraction.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "summarize_omena_query_css_modules_interface_bundle_inner",
    operation: "insert",
    evidence: "emitted_class_names.insert(",
    reason:
      "The interface bundle indexes each emitted token by module path and exact authored spelling because distinct raw spellings have distinct module-scoped token preimages.",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "collect_css_modules_composes_adjacency_with_path_mappings",
    operation: "str-eq",
    evidence: "let target_class_names = if target_style_path == *style_path {",
    reason: "This compares normalized style paths before selecting a class-name set.",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "collect_css_modules_composes_adjacency_with_path_mappings",
    operation: "contains",
    evidence: "if !class_names.contains(&canonical_class_key(owner_selector_name)) {",
    reason: "Both operands are normalized through the shared canonical class-name key.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "collect_css_modules_composes_adjacency_with_path_mappings",
    operation: "contains",
    evidence: "if !target_class_names.contains(&canonical_class_key(target_selector_name)) {",
    reason: "Both operands are normalized through the shared canonical class-name key.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/style.rs",
    function: "collect_sass_partial_evaluator_selector_candidates_from_omena_parser_facts",
    operation: "insert",
    evidence: "if seen.insert((range_span.start, range_span.end, selector_name.clone())) {",
    reason: "This tuple insertion deduplicates parser facts by span and authored spelling.",
  },
  {
    path: "rust/crates/omena-query/src/style/transform.rs",
    function: "closed_world_bound_reachability_precision",
    operation: "contains",
    evidence: ".any(|name| !closed_world_class_names.contains(name.as_str()))",
    reason: "The closed-world carrier and query projection share the canonical class-name domain.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-query/src/style/transform/css_modules.rs",
    function: "derive_class_name_rewrites_for_module_instance",
    operation: "contains",
    evidence: "if !unique_class_names.contains(name) {",
    reason:
      "Token preimages preserve exact authored spelling so two spellings that decode alike can still receive distinct module-scoped rewrites.",
  },
  {
    path: "rust/crates/omena-query/src/types.rs",
    function: "flat_class_names_for_style_paths",
    operation: "contains",
    evidence: ".filter(|(_, names)| names.contains(class_name))",
    reason: "This partitions already-emitted tokens and does not compare source identifiers.",
  },
  {
    path: "rust/crates/omena-query/src/types.rs",
    function: "from_style_paths",
    operation: "contains",
    evidence: "if directly_attributed_class_names.contains(class_name) {",
    reason: "This partitions already-emitted tokens and does not compare source identifiers.",
  },
  {
    path: "rust/crates/omena-query/src/types.rs",
    function: "from_style_paths",
    operation: "contains",
    evidence: ".filter(|(_, names)| names.contains(class_name))",
    reason: "This partitions already-emitted tokens and does not compare source identifiers.",
  },
  {
    path: "rust/crates/omena-transform-passes/src/runtime/executor.rs",
    function: "module_export_observations_from_emitted_css_v0",
    operation: "contains",
    evidence: ".find(|(canonical, _)| observed_class_names.contains(*canonical))",
    reason:
      "Both operands are decoded through the shared syntax authority before emitted export observations are admitted.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-transform-passes/src/runtime/executor.rs",
    function: "run_hash_css_module_class_names_structural",
    operation: "str-eq",
    evidence: "if mutation_count == 0 && class_name_rewrites.is_empty() {",
    reason: "This checks whether transform outputs are empty rather than comparing identifiers.",
  },
  {
    path: "rust/crates/omena-transform-passes/src/domains/css_modules_classes.rs",
    function: "rewritten_class_name_for",
    operation: "str-eq",
    evidence: "(original_name == class_name).then_some(rewritten_name)",
    reason:
      "Exact authored spelling selects its distinct module-scoped rewrite before the canonical CSS identifier fallback is consulted.",
  },
] as const;

const productPathMatrix = JSON.parse(
  readFileSync(path.join(repoRoot, "rust/omena-product-path-matrix.json"), "utf8"),
) as ProductPathMatrix;
assert.equal(productPathMatrix.schemaVersion, "0", "product-path matrix schemaVersion");
assert.equal(productPathMatrix.product, "omena-css.product-path-matrix", "product-path matrix");
const engineCrates = productPathMatrix.entries
  .filter((entry) => entry.role === "R1" || entry.role === "R2")
  .map((entry) => entry.crate)
  .toSorted();
assert.ok(engineCrates.length > 0, "product-path matrix must identify engine crates");

const productionSources = trackedProductionSources();
const scannedCrates = [
  ...new Set(productionSources.map((sourcePath) => sourcePath.split("/")[2])),
].toSorted();
assert.ok(scannedCrates.length > 0, "identifier census source scope must be non-empty");
for (const crateName of engineCrates) {
  assert.ok(scannedCrates.includes(crateName), `engine crate is outside scan scope: ${crateName}`);
}

const existing = readExistingCensus();
const typeSeal = inspectTypeSeal();
const egressDiscovered = discoverEgressSites();
const idiomDiscovered = discoverIdiomSites();
const predicateDiscovered = discoverPredicateCopies();
const egressSites = classifySites(
  egressDiscovered,
  existing?.egress.sites,
  egressExemptions,
  "egress",
);
const idiomSites = classifySites(idiomDiscovered, existing?.idiom.sites, idiomExemptions, "idiom");
const predicateSites = classifyPredicateSites(predicateDiscovered, existing?.predicateCopies.sites);

const unclassified = [...egressSites, ...idiomSites, ...predicateSites].filter(
  (site) => site.disposition === "unclassified",
);
assert.deepEqual(
  unclassified,
  [],
  "identifier authority census found unclassified sites; route or exempt each site explicitly",
);

const classifiedEgressSites = egressSites as CensusSite[];
const classifiedIdiomSites = idiomSites as CensusSite[];
const classifiedPredicateSites = predicateSites as CensusSite[];
const baselineEgressSiteCount = existing?.egress.baselineSiteCount ?? classifiedEgressSites.length;
assert.ok(
  classifiedEgressSites.length <= baselineEgressSiteCount,
  `identifier egress count increased: baseline=${baselineEgressSiteCount} current=${classifiedEgressSites.length}`,
);

if (existing && writeMode) {
  assertNoAddedSites(existing.egress.sites, classifiedEgressSites, "identifier egress");
  assertNoAddedSites(existing.idiom.sites, classifiedIdiomSites, "identifier idiom");
  assertNoAddedSites(
    existing.predicateCopies.sites,
    classifiedPredicateSites,
    "identifier predicate copy",
  );
}

const census: IdentifierAuthorityCensus = {
  schemaVersion: "0",
  product: "omena.identifier-authority.census",
  policy: {
    expectedSide: "committed-authored-table",
    addedSiteAdoption: "forbidden",
    owningCheck: "rust/omena-syntax-authority-raw-scan-census",
    packageScript: "check:rust-omena-syntax-authority-raw-scan-census",
  },
  sourceRoots,
  scope: {
    scannedCrates,
    scannedCratesDigest: digest(scannedCrates),
    engineCrates,
    engineCratesDigest: digest(engineCrates),
  },
  preflight: {
    productPin: "c18606086dc77101ad162c8d0ddb36ccae7e9d85",
    measuredSiteCount: 50,
    primitiveCounts: {
      "str-eq": 24,
      contains: 9,
      insert: 10,
      "map-get": 5,
      cmp: 2,
    },
  },
  typeSeal,
  egress: {
    direction: "decrease-only",
    baselineSiteCount: baselineEgressSiteCount,
    currentSiteCount: classifiedEgressSites.length,
    sites: classifiedEgressSites,
    siteDigest: digest(classifiedEgressSites),
  },
  idiom: {
    granularity: "line",
    lexemeGlobs,
    primitiveIds: primitivePatterns.map(([id]) => id),
    unlabelledBindingBlindSpot:
      "Bindings without a class or selector label are intentionally invisible; typed carriers and API egresses are the backstops.",
    currentSiteCount: classifiedIdiomSites.length,
    primitiveCounts: Object.fromEntries(
      primitivePatterns.map(([id]) => [
        id,
        classifiedIdiomSites.filter((site) => site.operation === id).length,
      ]),
    ) as Record<PrimitiveId, number>,
    sanctionedSiteCount: classifiedIdiomSites.filter((site) => site.disposition === "sanctioned")
      .length,
    namedExemptSiteCount: classifiedIdiomSites.filter((site) => site.disposition === "named-exempt")
      .length,
    sites: classifiedIdiomSites,
    siteDigest: digest(classifiedIdiomSites),
  },
  predicateCopies: {
    derivation: "direct-character-predicate-body",
    blindSpots: [
      "Methods and functions whose character input is not the sole typed parameter are outside this syntactic arm.",
      "Character membership hidden behind a constant, helper call, or other indirection is outside this syntactic arm.",
    ],
    currentSiteCount: classifiedPredicateSites.length,
    sites: classifiedPredicateSites,
    siteDigest: digest(classifiedPredicateSites),
  },
};

const expected = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  assert.ok(
    !injectClassNameEquality &&
      !injectEgress &&
      !injectLabelledComparison &&
      !injectUnlabelledComparison &&
      !injectPredicateCopy &&
      !injectPredicateCopyExplicit &&
      !injectPredicateCopyReversed,
    "test injection cannot be combined with --write",
  );
  writeFileSync(censusPath, expected);
  const formatResult = spawnSync("pnpm", ["exec", "oxfmt", path.relative(repoRoot, censusPath)], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(
    formatResult.status,
    0,
    `failed to format generated census: ${(formatResult.stderr ?? "").trim()}`,
  );
} else {
  assert.ok(
    existsSync(censusPath),
    "identifier authority census is missing; run the package update script",
  );
  assert.deepEqual(
    JSON.parse(readFileSync(censusPath, "utf8")),
    census,
    "identifier authority census is stale; review new sites before regeneration",
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: census.product,
      scannedCrateCount: census.scope.scannedCrates.length,
      engineCrateCount: census.scope.engineCrates.length,
      classNameDerives: census.typeSeal.classNameDerives,
      classNameEqualityDerives: census.typeSeal.classNameEqualityDerives,
      egressSiteCount: census.egress.currentSiteCount,
      idiomSiteCount: census.idiom.currentSiteCount,
      idiomSanctionedSiteCount: census.idiom.sanctionedSiteCount,
      idiomNamedExemptSiteCount: census.idiom.namedExemptSiteCount,
      predicateCopySiteCount: census.predicateCopies.currentSiteCount,
    },
    null,
    2,
  )}\n`,
);

function inspectTypeSeal(): IdentifierAuthorityCensus["typeSeal"] {
  const relativePath = "rust/crates/omena-syntax/src/ident.rs" as const;
  let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  if (injectClassNameEquality) {
    source = source.replace(
      "#[derive(Debug, Clone)]\npub struct ClassNameV0",
      "#[derive(Debug, Clone, PartialEq)]\npub struct ClassNameV0",
    );
  }
  const classNameDerives = derivesForStruct(source, "ClassNameV0");
  const canonicalKeyDerives = derivesForStruct(source, "CanonicalClassKeyV0");
  assert.deepEqual(
    classNameDerives,
    ["Debug", "Clone"],
    "ClassNameV0 derives must not expose structural equality or ordering",
  );
  assert.deepEqual(
    canonicalKeyDerives,
    ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    "CanonicalClassKeyV0 must remain the equality, ordering, and hash carrier",
  );
  assert.match(
    source,
    /pub struct CanonicalClassKeyV0\s*\(\s*String\s*,\s*CanonicalClassKeySealV0\s*\)\s*;/u,
    "canonical class key fields must remain private and sealed",
  );
  assert.match(
    source,
    /pub fn same_as\s*\(&self,\s*other:\s*&Self\)\s*->\s*bool/u,
    "ClassNameV0 equality must remain behind same_as",
  );
  assert.match(
    source,
    /pub fn canonical_key\s*\(self\)\s*->\s*CanonicalClassKeyV0/u,
    "ClassNameV0 key construction must remain behind canonical_key",
  );
  const canonicalKeyBody = source.match(
    /pub fn canonical_key\s*\(self\)\s*->\s*CanonicalClassKeyV0\s*\{(?<body>[\s\S]*?)\n\s{4}\}/u,
  )?.groups?.body;
  assert.ok(canonicalKeyBody, "canonical_key body must remain inspectable");
  assert.match(
    canonicalKeyBody,
    /self\.decoded\.unwrap_or\(self\.raw\)/u,
    "canonical class key must consume the decoded spelling or unchanged raw spelling",
  );
  const constructorMatches = [...canonicalKeyBody.matchAll(/CanonicalClassKeyV0\s*\(/gu)];
  assert.equal(
    constructorMatches.length,
    1,
    "canonical class key must have one authority-owned construction site",
  );
  for (const crateName of mutatorCrates) {
    assert.notEqual(crateName, "omena-syntax", "mutator crate must be outside authority crate");
    assert.ok(
      existsSync(path.join(repoRoot, "rust/crates", crateName, "Cargo.toml")),
      `measured mutator crate is missing: ${crateName}`,
    );
  }
  return {
    path: relativePath,
    classNameDerives: ["Debug", "Clone"],
    classNameEqualityDerives: [],
    canonicalKeyDerives: ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    canonicalKeyFieldVisibility: "private",
    canonicalKeyConstructor: "ClassNameV0::canonical_key",
    equalityMethod: "ClassNameV0::same_as",
    mutatorCrates,
    privacyDiscriminator:
      "Rust field privacy is module-scoped, and every measured mutator is in a crate other than omena-syntax.",
  };
}

function derivesForStruct(source: string, structName: string): string[] {
  const match = source.match(
    new RegExp(
      `#\\s*\\[\\s*derive\\s*\\(([^)]*)\\)\\s*\\]\\s*pub\\s+struct\\s+${structName}\\b`,
      "u",
    ),
  );
  assert.ok(match, `derive list missing for ${structName}`);
  return match[1]
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
}

function discoverEgressSites(): DiscoveredSite[] {
  const discovered: DiscoveredSite[] = [];
  const sources = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  if (injectEgress) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_egress.rs",
      source:
        "use omena_syntax::ident::ClassNameV0;\nfn injected_identifier_egress(class: &ClassNameV0) { let n: String = class.raw().to_string(); drop(n); }\n",
    });
  }
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const candidatePatterns: readonly (readonly [string, RegExp])[] = [
      ["ClassNameV0::into_raw", /\b[A-Za-z_][A-Za-z0-9_]*\.name\.into_raw\s*\(\s*\)/gu],
      [
        "ClassNameV0::raw",
        source.includes("ClassNameV0") ? /\b[A-Za-z_][A-Za-z0-9_]*\.raw\s*\(\s*\)/gu : /$a/gu,
      ],
      [
        "ClassNameV0::into_inner",
        source.includes("ClassNameV0")
          ? /\b[A-Za-z_][A-Za-z0-9_]*\.into_inner\s*\(\s*\)/gu
          : /$a/gu,
      ],
      ["CanonicalClassKeyV0::as_str", /\.canonical_key\s*\(\s*\)\s*\.as_str\s*\(\s*\)/gu],
    ];
    for (const [operation, expression] of candidatePatterns) {
      expression.lastIndex = 0;
      for (const match of scannable.matchAll(expression)) {
        discovered.push(siteAt(relativePath, source, scannable, match.index, operation));
      }
    }
    const canonicalVariables = new Set<string>();
    for (const match of scannable.matchAll(
      /\blet\s+([A-Za-z_][A-Za-z0-9_]*)[^=;\n]*=\s*[^;\n]*\.canonical_key\s*\(\s*\)/gu,
    )) {
      canonicalVariables.add(match[1]);
    }
    for (const match of scannable.matchAll(
      /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:&\s*)?CanonicalClassKeyV0\b/gu,
    )) {
      canonicalVariables.add(match[1]);
    }
    for (const variable of canonicalVariables) {
      const expression = new RegExp(`\\b${variable}\\.as_str\\s*\\(\\s*\\)`, "gu");
      for (const match of scannable.matchAll(expression)) {
        discovered.push(
          siteAt(relativePath, source, scannable, match.index, "CanonicalClassKeyV0::as_str"),
        );
      }
    }
  }
  return uniqueSites(discovered);
}

function discoverIdiomSites(): DiscoveredSite[] {
  const sources = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  if (injectLabelledComparison) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_comparison.rs",
      source:
        "fn injected(left_class_name: String, right_class_name: String) -> bool { left_class_name == right_class_name }\n",
    });
  }
  if (injectUnlabelledComparison) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_unlabelled_comparison.rs",
      source: "fn injected(a: String, b: String) -> bool { a == b }\n",
    });
  }
  const discovered: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, true);
    const sourceLines = source.split(/\r?\n/u);
    for (const [index, line] of scannable.split(/\r?\n/u).entries()) {
      if (!classBindingLexeme.test(line)) continue;
      for (const [operation, expression] of primitivePatterns) {
        if (!expression.test(line)) continue;
        discovered.push({
          path: relativePath,
          line: index + 1,
          function: enclosingFunctionName(scannable, offsetForLine(scannable, index + 1)),
          operation,
          evidence: sourceLines[index]?.trim().replace(/\s+/gu, " ") ?? "",
        });
      }
    }
  }
  return uniqueSites(discovered);
}

function discoverPredicateCopies(): DiscoveredSite[] {
  const sources = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  if (injectPredicateCopy) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_predicate.rs",
      source:
        "fn unrelated_name(ch: char) -> bool { matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_') }\n",
    });
  }
  if (injectPredicateCopyExplicit) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_predicate_explicit.rs",
      source:
        "fn arbitrary_spelling(codepoint: char) -> bool { codepoint.is_alphanumeric() || codepoint == '-' || codepoint == '_' }\n",
    });
  }
  if (injectPredicateCopyReversed) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_identifier_predicate_reversed.rs",
      source:
        "fn another_spelling(byte: u8) -> bool { matches!(byte, b'-' | b'_') || byte.is_ascii_alphanumeric() }\n",
    });
  }
  const discovered: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    for (const functionBody of directCharacterPredicateBodies(source)) {
      discovered.push({
        path: relativePath,
        line: functionBody.line,
        function: functionBody.name,
        operation: "character-predicate-copy",
        evidence: functionBody.evidence,
      });
    }
  }
  return uniqueSites(discovered);
}

function directCharacterPredicateBodies(
  source: string,
): readonly { name: string; line: number; evidence: string }[] {
  const scannable = maskCommentsStringsAndTestItems(source, false);
  const found: { name: string; line: number; evidence: string }[] = [];
  const declaration =
    /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*:\s*(?:char|u8)\s*\)\s*->\s*bool\s*\{/gu;
  for (const match of scannable.matchAll(declaration)) {
    const openBrace = match.index + match[0].lastIndexOf("{");
    const closeBrace = matchingBrace(scannable, openBrace);
    if (closeBrace === undefined) continue;
    const body = maskCommentsStringsAndTestItems(
      source.slice(openBrace + 1, closeBrace),
      false,
      true,
    )
      .replace(/\s+/gu, "")
      .replace(/^return/u, "")
      .replace(/;$/u, "");
    const parameter = match[0].match(/\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:char|u8)\s*\)/u)?.[1];
    if (!parameter || !isIdentifierContinuationPredicate(body, parameter)) continue;
    found.push({
      name: match[1],
      line: lineNumberAt(source, match.index),
      evidence: source
        .slice(match.index, closeBrace + 1)
        .trim()
        .replace(/\s+/gu, " "),
    });
  }
  return found;
}

function isIdentifierContinuationPredicate(body: string, parameter: string): boolean {
  const escapedParameter = escapeRegExp(parameter);
  const acceptsHyphen = new RegExp(
    `(?:${escapedParameter}==b?'-'|b?'-'==${escapedParameter}|matches!\\(${escapedParameter},[^)]*b?'-')`,
    "u",
  ).test(body);
  const acceptsUnderscore = new RegExp(
    `(?:${escapedParameter}==b?'_'|b?'_'==${escapedParameter}|matches!\\(${escapedParameter},[^)]*b?'_')`,
    "u",
  ).test(body);
  const usesAlphanumericPredicate = new RegExp(
    `${escapedParameter}\\.is_(?:ascii_)?alphanumeric\\(\\)`,
    "u",
  ).test(body);
  const usesSharedCssPredicate = new RegExp(
    `is_css_name_start\\(${escapedParameter}\\).*${escapedParameter}\\.is_ascii_digit\\(\\)`,
    "u",
  ).test(body);
  const rangeBody = body.match(new RegExp(`matches!\\(${escapedParameter},([^)]*)\\)`, "u"))?.[1];
  const usesAlphaNumericRanges =
    rangeBody !== undefined &&
    /b?'[a-zA-Z0-9]'\.\.=b?'[a-zA-Z0-9]'/u.test(rangeBody) &&
    (rangeBody.match(/b?'[a-zA-Z0-9]'\.\.=b?'[a-zA-Z0-9]'/gu)?.length ?? 0) >= 3;
  return (
    (usesSharedCssPredicate || usesAlphanumericPredicate || usesAlphaNumericRanges) &&
    (usesSharedCssPredicate || (acceptsHyphen && acceptsUnderscore))
  );
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function classifySites(
  discovered: readonly DiscoveredSite[],
  previous: readonly CensusSite[] | undefined,
  exemptions: readonly ExemptionRule[],
  arm: string,
): readonly (CensusSite | (DiscoveredSite & { disposition: "unclassified" }))[] {
  const previousByKey = new Map((previous ?? []).map((site) => [stableSiteKey(site), site]));
  return discovered.map((site) => {
    const exemption = exemptions.find((rule) => stableSiteKey(rule) === stableSiteKey(site));
    if (exemption) {
      return {
        ...site,
        disposition: exemption.disposition ?? "named-exempt",
        reason: exemption.reason,
      } as const;
    }
    const prior = previousByKey.get(stableSiteKey(site));
    if (prior)
      return {
        ...site,
        disposition: prior.disposition,
        ...(prior.reason ? { reason: prior.reason } : {}),
      };
    if (!previous && writeMode) {
      return {
        ...site,
        disposition: "sanctioned",
        reason: `${arm} site reviewed in the initial authored census.`,
      } as const;
    }
    return { ...site, disposition: "unclassified" } as const;
  });
}

function classifyPredicateSites(
  discovered: readonly DiscoveredSite[],
  previous: readonly CensusSite[] | undefined,
): readonly (CensusSite | (DiscoveredSite & { disposition: "unclassified" }))[] {
  const previousByKey = new Map((previous ?? []).map((site) => [stableSiteKey(site), site]));
  return discovered.map((site) => {
    const prior = previousByKey.get(stableSiteKey(site));
    if (prior)
      return {
        ...site,
        disposition: prior.disposition,
        ...(prior.reason ? { reason: prior.reason } : {}),
      };
    if (!previous && writeMode) {
      const authority = site.path === "rust/crates/omena-syntax/src/ident.rs";
      return {
        ...site,
        disposition: authority ? "sanctioned" : "named-exempt",
        reason: authority
          ? "The shared syntax crate owns the identifier character predicate."
          : `${site.function} owns a non-class identifier grammar and remains visible as a named exemption.`,
      } as const;
    }
    return { ...site, disposition: "unclassified" } as const;
  });
}

function assertNoAddedSites(
  previous: readonly CensusSite[],
  current: readonly CensusSite[],
  label: string,
): void {
  const previousKeys = new Set(previous.map(stableSiteKey));
  const addedSites = current.filter((site) => !previousKeys.has(stableSiteKey(site)));
  assert.deepEqual(
    addedSites,
    [],
    `the committed authored table cannot adopt new ${label} sites during regeneration`,
  );
}

function siteAt(
  relativePath: string,
  source: string,
  scannable: string,
  offset: number,
  operation: string,
): DiscoveredSite {
  const line = lineNumberAt(source, offset);
  return {
    path: relativePath,
    line,
    function: enclosingFunctionName(scannable, offset),
    operation,
    evidence: source.split(/\r?\n/u)[line - 1]?.trim().replace(/\s+/gu, " ") ?? "",
  };
}

function uniqueSites<T extends DiscoveredSite>(sites: readonly T[]): T[] {
  const byKey = new Map<string, T>();
  for (const site of sites) byKey.set(stableSiteKey(site), site);
  return [...byKey.values()].toSorted(
    (left, right) =>
      left.path.localeCompare(right.path) ||
      left.line - right.line ||
      left.operation.localeCompare(right.operation),
  );
}

function stableSiteKey(
  site: Pick<DiscoveredSite, "path" | "function" | "operation" | "evidence">,
): string {
  return `${site.path}\u0000${site.function}\u0000${site.operation}\u0000${site.evidence}`;
}

function trackedProductionSources(): string[] {
  const result = spawnSync("git", ["ls-files", "rust/crates/**/*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `git ls-files failed: ${(result.stderr ?? "").trim()}`);
  return result.stdout
    .split(/\r?\n/u)
    .filter(Boolean)
    .filter((sourcePath) => sourcePath.includes("/src/"))
    .filter((sourcePath) => !sourcePath.includes("/tests/"))
    .filter((sourcePath) => !sourcePath.endsWith("/tests.rs"))
    .filter((sourcePath) => !sourcePath.endsWith("_test.rs"))
    .filter((sourcePath) => !sourcePath.includes("/src/bin/"))
    .filter((sourcePath) => !sourcePath.endsWith("_generated.rs"))
    .toSorted();
}

function maskCommentsStringsAndTestItems(
  source: string,
  preserveStringContents: boolean,
  preserveCharacterContents = false,
): string {
  const chars = [...source];
  let blockDepth = 0;
  let lineComment = false;
  let stringQuote = "";
  let escaped = false;
  for (let index = 0; index < chars.length; index += 1) {
    const current = chars[index];
    const next = chars[index + 1] ?? "";
    if (lineComment) {
      if (current === "\n") lineComment = false;
      else chars[index] = " ";
      continue;
    }
    if (blockDepth > 0) {
      if (current === "/" && next === "*") {
        chars[index] = chars[index + 1] = " ";
        blockDepth += 1;
        index += 1;
      } else if (current === "*" && next === "/") {
        chars[index] = chars[index + 1] = " ";
        blockDepth -= 1;
        index += 1;
      } else if (current !== "\n") {
        chars[index] = " ";
      }
      continue;
    }
    if (stringQuote) {
      if (!preserveStringContents && current !== "\n") chars[index] = " ";
      if (escaped) escaped = false;
      else if (current === "\\") escaped = true;
      else if (current === stringQuote) stringQuote = "";
      continue;
    }
    if (current === "/" && next === "/") {
      chars[index] = chars[index + 1] = " ";
      lineComment = true;
      index += 1;
      continue;
    }
    if (current === "/" && next === "*") {
      chars[index] = chars[index + 1] = " ";
      blockDepth = 1;
      index += 1;
      continue;
    }
    if (current === '"') {
      stringQuote = current;
      if (!preserveStringContents) chars[index] = " ";
      continue;
    }
    if (current === "'") {
      const characterEnd = rustCharacterLiteralEnd(chars, index);
      if (characterEnd !== undefined) {
        if (!preserveCharacterContents) {
          for (let cursor = index; cursor <= characterEnd; cursor += 1) {
            if (chars[cursor] !== "\n") chars[cursor] = " ";
          }
        }
        index = characterEnd;
      }
    }
  }
  return maskRustCfgTestItems(chars.join(""));
}

function rustCharacterLiteralEnd(chars: readonly string[], quoteIndex: number): number | undefined {
  let index = quoteIndex + 1;
  if (index >= chars.length || chars[index] === "\n") return undefined;
  if (chars[index] === "\\") {
    index += 1;
    if (chars[index] === "x") {
      index += 3;
    } else if (chars[index] === "u" && chars[index + 1] === "{") {
      index += 2;
      while (index < chars.length && chars[index] !== "}" && chars[index] !== "\n") index += 1;
      if (chars[index] !== "}") return undefined;
      index += 1;
    } else {
      index += 1;
    }
  } else {
    index += 1;
  }
  return chars[index] === "'" ? index : undefined;
}

function enclosingFunctionName(source: string, offset: number): string {
  let functionName = "<module>";
  for (const match of source.slice(0, offset).matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu)) {
    functionName = match[1];
  }
  return functionName;
}

function matchingBrace(source: string, openBrace: number): number | undefined {
  let depth = 0;
  for (let index = openBrace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return undefined;
}

function offsetForLine(source: string, line: number): number {
  let offset = 0;
  for (let index = 1; index < line; index += 1) {
    offset = source.indexOf("\n", offset) + 1;
  }
  return offset;
}

function lineNumberAt(source: string, offset: number): number {
  return source.slice(0, offset).split("\n").length;
}

function digest(value: unknown): string {
  return `sha256:${createHash("sha256").update(JSON.stringify(value)).digest("hex")}`;
}

function readExistingCensus(): IdentifierAuthorityCensus | undefined {
  if (!existsSync(censusPath)) return undefined;
  const parsed = JSON.parse(readFileSync(censusPath, "utf8")) as IdentifierAuthorityCensus;
  assert.equal(parsed.schemaVersion, "0", "identifier census schemaVersion");
  assert.equal(parsed.product, "omena.identifier-authority.census", "identifier census product");
  assert.equal(parsed.policy.expectedSide, "committed-authored-table", "expected-side policy");
  assert.equal(parsed.policy.addedSiteAdoption, "forbidden", "added-site policy");
  assert.equal(parsed.egress.currentSiteCount, parsed.egress.sites.length, "egress site count");
  assert.equal(parsed.egress.siteDigest, digest(parsed.egress.sites), "egress site digest");
  assert.equal(parsed.idiom.currentSiteCount, parsed.idiom.sites.length, "idiom site count");
  assert.equal(parsed.idiom.siteDigest, digest(parsed.idiom.sites), "idiom site digest");
  assert.equal(
    parsed.predicateCopies.currentSiteCount,
    parsed.predicateCopies.sites.length,
    "predicate-copy site count",
  );
  assert.equal(
    parsed.predicateCopies.siteDigest,
    digest(parsed.predicateCopies.sites),
    "predicate-copy site digest",
  );
  return parsed;
}

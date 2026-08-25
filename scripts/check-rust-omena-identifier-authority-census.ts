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

type PropertyIdentitySiteRole = "authority" | "producer" | "consumer" | "carrier";

interface PropertyIdentitySite extends DiscoveredSite {
  readonly role: PropertyIdentitySiteRole;
}

type ResidualPropertyConsumerClassification = "presentation" | "egress" | "identity-shaped";

interface ResidualPropertyConsumerSite extends DiscoveredSite {
  readonly classification: ResidualPropertyConsumerClassification;
}

interface ResidualPropertyCarrierConsumerRow {
  readonly carrier: CensusSite;
  readonly consumers: readonly ResidualPropertyConsumerSite[];
}

interface PropertyIdentityTypeSeal {
  readonly path: "rust/crates/omena-syntax/src/ident.rs";
  readonly authoredTextDerives: readonly ["Debug", "Clone", "serde::Serialize"];
  readonly authoredTextIdentityDerives: readonly [];
  readonly authoredTextFieldVisibility: "private";
  readonly authoredTextPresentationMethods: readonly ["Display", "write_into", "Serialize"];
  readonly propertyNameDerives: readonly ["Debug", "Clone"];
  readonly propertyNameEqualityDerives: readonly [];
  readonly standardKeyDerives: readonly [
    "Debug",
    "Clone",
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "Hash",
  ];
  readonly customKeyDerives: readonly [
    "Debug",
    "Clone",
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "Hash",
  ];
  readonly keyFieldVisibility: "private";
  readonly sharedDecoder: "decode_css_identifier_escapes";
  readonly standardCanonicalization: "trim-decode-ascii-lowercase";
  readonly customCanonicalization: "trim-decode-case-preserved";
  readonly equalityMethod: "PropertyNameV0::same_as";
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
    readonly meaningRevisionReviews: readonly {
      readonly previousSiteCount: number;
      readonly currentSiteCount: number;
      readonly disposition: "reviewed-selector-authority-rekey";
      readonly reason: string;
    }[];
    readonly baselineSiteCount: number;
    readonly currentSiteCount: number;
    readonly sites: readonly CensusSite[];
    readonly siteDigest: string;
  };
  readonly idiom: {
    readonly granularity: "line";
    readonly meaningRevisionReviews: readonly {
      readonly disposition: "reviewed-sealed-selector-key-migration";
      readonly reason: string;
    }[];
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
  readonly propertyIdentity: {
    readonly derivation: "authority-token-and-property-key-operation-scan";
    readonly rawSiteDerivation: "typed-property-operand-and-raw-key-dataflow-scan";
    readonly typeSeal: PropertyIdentityTypeSeal;
    readonly authorityInventoryDecreaseReviews: readonly {
      readonly previousSiteCount: number;
      readonly currentSiteCount: number;
      readonly disposition: "reviewed-structural-refactor";
      readonly reason: string;
    }[];
    readonly authoritySiteCount: number;
    readonly sites: readonly PropertyIdentitySite[];
    readonly siteDigest: string;
    readonly residualRawCarrierDirection: "decrease-only";
    readonly residualRawCarrierSiteCount: number;
    readonly residualRawCarrierSites: readonly CensusSite[];
    readonly residualRawCarrierSiteDigest: string;
    readonly residualCarrierConsumerDerivation: "resolved-type-access-and-parameter-use-scan";
    readonly residualCarrierConsumerRows: readonly ResidualPropertyCarrierConsumerRow[];
    readonly residualCarrierConsumerSiteCount: number;
    readonly residualIdentityShapedConsumerCount: 0;
    readonly residualCarrierConsumerDigest: string;
    readonly rawStringIdentitySiteCount: 0;
    readonly rawStringIdentitySites: readonly [];
    readonly rawStringIdentitySiteDigest: string;
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
const censusPath = process.env.OMENA_IDENTIFIER_AUTHORITY_CENSUS_PATH
  ? path.resolve(process.env.OMENA_IDENTIFIER_AUTHORITY_CENSUS_PATH)
  : path.join(repoRoot, "rust/omena-identifier-authority-census.json");
const writeMode = process.argv.includes("--write");
const injectClassNameEquality =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CLASSNAME_EQUALITY === "1";
const injectEgress = process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EGRESS === "1";
const injectReviewedIdiomOmission =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_REVIEWED_IDIOM_OMISSION === "1";
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
const injectPropertyStructuralEquality =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_STRUCTURAL_EQUALITY === "1";
const injectPropertyRoundtripEquality =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_ROUNDTRIP_EQUALITY === "1";
const injectPropertyRawMap =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_MAP === "1";
const injectPropertyRawCanonicalization =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_CANONICALIZATION === "1";
const injectPropertyFqnRawMap =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_FQN_RAW_MAP === "1";
const injectPropertyValuesRawMap =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_VALUES_RAW_MAP === "1";
const injectPropertySameLineRawOperation =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_SAME_LINE_RAW_OPERATION === "1";
const injectPropertyNewFileRawComparison =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_COMPARISON === "1";
const injectPropertyNewFileRawCanonicalization =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_CANONICALIZATION === "1";
const injectPropertyTrimChain =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_TRIM_CHAIN === "1";
const injectPropertyContextRawOperations =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CONTEXT_RAW_OPERATIONS === "1";
const injectPropertyAutomaticCarrier =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTOMATIC_CARRIER === "1";
const injectPropertyRealFileMutation =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_REAL_FILE_MUTATION === "1";
const injectPropertyCaseFold =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CASE_FOLD === "1";
const injectPropertyDecodeNeuter =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_DECODE_NEUTER === "1";
const injectPropertyAuthorityDecreaseLaundering =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTHORITY_DECREASE_LAUNDERING === "1";
const injectMigrateLowercaseComparison =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_LOWERCASE_COMPARISON === "1";
const injectMigrateFqnParameter =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_FQN_PARAMETER === "1";
const injectMigrateAliasParameter =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_ALIAS_PARAMETER === "1";
const injectMigrateBareParameter =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_BARE_PARAMETER === "1";
const injectMigrateClosureParameter =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_CLOSURE_PARAMETER === "1";
const injectAuthoredUppercaseTransform =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_UPPERCASE_TRANSFORM === "1";
const injectAuthoredTrimMatchesTransform =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_TRIM_MATCHES_TRANSFORM === "1";
const injectAuthoredStripPrefixTransform =
  process.env.OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_STRIP_PREFIX_TRANSFORM === "1";
const generatedFixtureManifestPath =
  process.env.OMENA_IDENTIFIER_AUTHORITY_GENERATED_FIXTURE_MANIFEST;

interface GeneratedFixtureManifest {
  readonly schemaVersion: "0";
  readonly sources: readonly MutableRustSource[];
  readonly expectedCellFunctions: readonly string[];
}

const generatedFixtureManifest = generatedFixtureManifestPath
  ? (JSON.parse(readFileSync(generatedFixtureManifestPath, "utf8")) as GeneratedFixtureManifest)
  : undefined;
if (generatedFixtureManifest) {
  assert.equal(generatedFixtureManifest.schemaVersion, "0", "generated fixture manifest schema");
  assert.ok(
    generatedFixtureManifest.sources.length > 0 &&
      generatedFixtureManifest.expectedCellFunctions.length > 0,
    "generated fixture manifest must be non-empty",
  );
}

const sourceRoots = ["rust/crates"] as const;
const authorityInventoryDecreaseReviews = [
  {
    previousSiteCount: 706,
    currentSiteCount: 705,
    disposition: "reviewed-indexed-custom-property-resolution",
    reason:
      "The custom-property SCC evaluator now resolves dependencies through the canonical graph node index and materializes the ordered environment once, removing one intermediate canonical-name carrier while raw-string identity and identity-shaped residual counts remain zero.",
  },
  {
    previousSiteCount: 702,
    currentSiteCount: 694,
    disposition: "reviewed-structural-refactor",
    reason:
      "The indexed custom-property dependency graph replaces eight recursive canonical-name traversal carrier lines with integer node indices; raw-string identity and identity-shaped residual counts remain zero.",
  },
] as const;
const egressMeaningRevisionReviews = [
  {
    previousSiteCount: 2,
    currentSiteCount: 3,
    disposition: "reviewed-selector-authority-rekey",
    reason:
      "The selector identity report is now a presentation projection of the sealed CST-issued selector key; the single new egress is the compatibility wire id and raw matching sites were removed.",
  },
] as const;
const idiomMeaningRevisionReviews = [
  {
    disposition: "reviewed-sealed-selector-key-migration",
    reason:
      "The matcher signature and element join changed from authored String storage to sealed decoded selector keys; the witness-only String insertion remains presentation data, and the CST authority now verifies its compound key before issuing the semantic projection key.",
  },
] as const;
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
    path: "rust/crates/omena-semantic/src/selector_identity.rs",
    function: "from_canonical_key",
    operation: "CanonicalClassKeyV0::as_str",
    evidence: "let local_name = key.as_str().to_string();",
    reason:
      "The compatibility report carrier projects its display id from the sealed selector authority key.",
    disposition: "sanctioned",
  },
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
    path: "rust/crates/omena-syntax/src/selector.rs",
    function: "canonical_class_key_for_source_span",
    operation: "contains",
    evidence: "&& compound.required_classes.contains(&key)",
    reason:
      "The sole selector authority verifies that a parser definition span names a class key carried by the same CST compound before issuing that key.",
    disposition: "sanctioned",
  },
  {
    path: "rust/crates/omena-cascade/src/selector.rs",
    function: "selector_match_branch_witness",
    operation: "insert",
    evidence: ".insert(required_class.as_str().to_string());",
    reason:
      "A failed sealed-key match is converted to authored-facing witness text only after the identity decision.",
    disposition: "named-exempt",
  },
  {
    path: "rust/crates/omena-cascade/src/selector.rs",
    function: "parse_simple_selector_signature_inner",
    operation: "insert",
    evidence: "required_classes.insert(ClassNameV0::new(name).canonical_key());",
    reason:
      "Selector signatures store the shared decoder's sealed class key rather than authored spelling.",
    disposition: "sanctioned",
  },
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
const propertyTypeSeal = inspectPropertyTypeSeal();
const propertyIdentitySites = discoverPropertyIdentitySites();
const residualSweepSources: MutableRustSource[] = productionSources.map((relativePath) => ({
  relativePath,
  source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
}));
applyMigrateConsumerProbeMutations(residualSweepSources);
const residualRawPropertyCarrierDiscovered =
  discoverResidualRawPropertyCarrierSites(residualSweepSources);
const residualRawPropertyCarrierSites = classifyResidualRawPropertyCarrierSites(
  residualRawPropertyCarrierDiscovered,
  existing?.propertyIdentity.residualRawCarrierSites,
);
const rawPropertyIdentitySites = discoverRawPropertyIdentitySites();
if (generatedFixtureManifest) {
  const detectedFunctions = new Set(rawPropertyIdentitySites.map((site) => site.function));
  const missingGeneratedCells = generatedFixtureManifest.expectedCellFunctions.filter(
    (functionName) => !detectedFunctions.has(functionName),
  );
  const detectedGeneratedCellCount =
    generatedFixtureManifest.expectedCellFunctions.length - missingGeneratedCells.length;
  process.stderr.write(`generatedPropertyIdentityCellCount=${detectedGeneratedCellCount}\n`);
  assert.deepEqual(
    missingGeneratedCells,
    [],
    "generated property-identity matrix has undetected origin x grammar x position cells",
  );
}
const egressSites = classifySites(
  egressDiscovered,
  existing?.egress.sites,
  egressExemptions,
  "egress",
);
const idiomSites = classifySites(idiomDiscovered, existing?.idiom.sites, idiomExemptions, "idiom");
const predicateSites = classifyPredicateSites(predicateDiscovered, existing?.predicateCopies.sites);

const unclassified = [
  ...egressSites,
  ...idiomSites,
  ...predicateSites,
  ...residualRawPropertyCarrierSites,
].filter((site) => site.disposition === "unclassified");
if (rawPropertyIdentitySites.length > 0) {
  process.stderr.write(`rawPropertyIdentitySiteCount=${rawPropertyIdentitySites.length}\n`);
}
if (injectPropertyAuthorityDecreaseLaundering) {
  process.stderr.write(`propertyAuthoritySiteCount=${propertyIdentitySites.length}\n`);
}
assert.deepEqual(
  unclassified,
  [],
  "identifier authority census found unclassified sites; route or exempt each site explicitly",
);

const classifiedEgressSites = egressSites as CensusSite[];
const classifiedIdiomSites = idiomSites as CensusSite[];
const classifiedPredicateSites = predicateSites as CensusSite[];
const classifiedResidualRawPropertyCarrierSites = residualRawPropertyCarrierSites as CensusSite[];
const residualCarrierConsumerRows = discoverResidualPropertyCarrierConsumers(
  classifiedResidualRawPropertyCarrierSites,
  residualSweepSources,
);
const residualCarrierConsumerSites = residualCarrierConsumerRows.flatMap((row) => row.consumers);
const residualIdentityShapedConsumerCount = residualCarrierConsumerSites.filter(
  (site) => site.classification === "identity-shaped",
).length;
if (residualIdentityShapedConsumerCount > 0) {
  process.stderr.write(
    `residualIdentityShapedConsumerCount=${residualIdentityShapedConsumerCount}\n`,
  );
  process.stderr.write(
    `residualIdentityShapedConsumers=${JSON.stringify(
      residualCarrierConsumerRows
        .filter((row) => row.consumers.some((site) => site.classification === "identity-shaped"))
        .map((row) => ({
          carrier: row.carrier.operation,
          path: row.carrier.path,
          consumers: row.consumers.filter((site) => site.classification === "identity-shaped"),
        })),
    )}\n`,
  );
}
assert.deepEqual(
  rawPropertyIdentitySites,
  [],
  "property identity census found raw-string identity sites; convert each boundary to PropertyNameV0 or a sealed canonical key",
);
assert.equal(
  residualIdentityShapedConsumerCount,
  0,
  "residual raw-property carriers have identity-shaped consumers; add a sealed key to each carrier and migrate those consumers",
);
const egressMeaningRevision = existing
  ? egressMeaningRevisionReviews.find(
      (review) =>
        review.previousSiteCount === existing.egress.currentSiteCount &&
        review.currentSiteCount === classifiedEgressSites.length,
    )
  : undefined;
const baselineEgressSiteCount = egressMeaningRevision
  ? egressMeaningRevision.currentSiteCount
  : (existing?.egress.baselineSiteCount ?? classifiedEgressSites.length);
assert.ok(
  classifiedEgressSites.length <= baselineEgressSiteCount,
  `identifier egress count increased: baseline=${baselineEgressSiteCount} current=${classifiedEgressSites.length}`,
);

if (existing && writeMode) {
  if (propertyIdentitySites.length < existing.propertyIdentity.authoritySiteCount) {
    assert.ok(
      authorityInventoryDecreaseReviews.some(
        (review) =>
          review.previousSiteCount === existing.propertyIdentity.authoritySiteCount &&
          review.currentSiteCount === propertyIdentitySites.length,
      ),
      `authority inventory decrease cannot be adopted by --write: previous=${existing.propertyIdentity.authoritySiteCount} current=${propertyIdentitySites.length}`,
    );
  }
  if (egressMeaningRevision) {
    const previousKeys = new Set(existing.egress.sites.map(stableSiteKey));
    const additions = classifiedEgressSites.filter(
      (site) => !previousKeys.has(stableSiteKey(site)),
    );
    assert.deepEqual(
      additions.map((site) => [site.path, site.function, site.operation]),
      [
        [
          "rust/crates/omena-semantic/src/selector_identity.rs",
          "from_canonical_key",
          "CanonicalClassKeyV0::as_str",
        ],
      ],
      "the reviewed selector-authority meaning revision permits only the report projection egress",
    );
  } else {
    assertNoAddedSites(existing.egress.sites, classifiedEgressSites, "identifier egress");
  }
  const previousIdiomSites = injectReviewedIdiomOmission
    ? existing.idiom.sites.filter(
        (site) =>
          !(
            site.path === "rust/crates/omena-cascade/src/selector.rs" &&
            site.function === "selector_match_branch_witness" &&
            site.operation === "insert"
          ),
      )
    : existing.idiom.sites;
  const previousIdiomKeys = new Set(previousIdiomSites.map(stableSiteKey));
  const idiomAdditions = classifiedIdiomSites.filter(
    (site) => !previousIdiomKeys.has(stableSiteKey(site)),
  );
  if (idiomAdditions.length > 0) {
    const reviewedIdiomAdditionKeys = [
      ["rust/crates/omena-cascade/src/selector.rs", "selector_match_branch_witness", "insert"],
      [
        "rust/crates/omena-cascade/src/selector.rs",
        "parse_simple_selector_signature_inner",
        "insert",
      ],
      [
        "rust/crates/omena-syntax/src/selector.rs",
        "canonical_class_key_for_source_span",
        "contains",
      ],
    ].map((key) => key.join("\0"));
    assert.deepEqual(
      idiomAdditions
        .map((site) => [site.path, site.function, site.operation].join("\0"))
        .toSorted(),
      reviewedIdiomAdditionKeys.toSorted(),
      "the reviewed selector-key meaning revision permits only matcher storage, witness, and CST authority issuance rows",
    );
  } else {
    assertNoAddedSites(existing.idiom.sites, classifiedIdiomSites, "identifier idiom");
  }
  assertNoAddedSites(
    existing.predicateCopies.sites,
    classifiedPredicateSites,
    "identifier predicate copy",
  );
  if (existing.propertyIdentity.residualRawCarrierSites) {
    assertNoAddedCarrierSites(
      existing.propertyIdentity.residualRawCarrierSites,
      residualRawPropertyCarrierSites as CensusSite[],
      "raw property carrier",
    );
  }
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
    meaningRevisionReviews: egressMeaningRevisionReviews,
    baselineSiteCount: baselineEgressSiteCount,
    currentSiteCount: classifiedEgressSites.length,
    sites: classifiedEgressSites,
    siteDigest: digest(classifiedEgressSites),
  },
  idiom: {
    granularity: "line",
    meaningRevisionReviews: idiomMeaningRevisionReviews,
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
  propertyIdentity: {
    derivation: "authority-token-and-property-key-operation-scan",
    rawSiteDerivation: "typed-property-operand-and-raw-key-dataflow-scan",
    typeSeal: propertyTypeSeal,
    authorityInventoryDecreaseReviews,
    authoritySiteCount: propertyIdentitySites.length,
    sites: propertyIdentitySites,
    siteDigest: digest(propertyIdentitySites),
    residualRawCarrierDirection: "decrease-only",
    residualRawCarrierSiteCount: classifiedResidualRawPropertyCarrierSites.length,
    residualRawCarrierSites: classifiedResidualRawPropertyCarrierSites,
    residualRawCarrierSiteDigest: digest(residualRawPropertyCarrierSites),
    residualCarrierConsumerDerivation: "resolved-type-access-and-parameter-use-scan",
    residualCarrierConsumerRows,
    residualCarrierConsumerSiteCount: residualCarrierConsumerSites.length,
    residualIdentityShapedConsumerCount: 0,
    residualCarrierConsumerDigest: digest(residualCarrierConsumerRows),
    rawStringIdentitySiteCount: 0,
    rawStringIdentitySites: [],
    rawStringIdentitySiteDigest: digest([]),
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
      !injectPredicateCopyReversed &&
      !injectPropertyStructuralEquality &&
      !injectPropertyRoundtripEquality &&
      !injectPropertyRawMap &&
      !injectPropertyRawCanonicalization &&
      !injectPropertyFqnRawMap &&
      !injectPropertyValuesRawMap &&
      !injectPropertySameLineRawOperation &&
      !injectPropertyNewFileRawComparison &&
      !injectPropertyNewFileRawCanonicalization &&
      !injectPropertyTrimChain &&
      !injectPropertyContextRawOperations &&
      !injectPropertyAutomaticCarrier &&
      !injectPropertyRealFileMutation &&
      !injectPropertyCaseFold &&
      !injectPropertyDecodeNeuter &&
      !injectPropertyAuthorityDecreaseLaundering &&
      !injectMigrateLowercaseComparison &&
      !injectMigrateFqnParameter &&
      !injectMigrateAliasParameter &&
      !injectMigrateBareParameter &&
      !injectMigrateClosureParameter &&
      !injectAuthoredUppercaseTransform &&
      !injectAuthoredTrimMatchesTransform &&
      !injectAuthoredStripPrefixTransform &&
      !generatedFixtureManifest,
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
      propertyAuthoritySiteCount: census.propertyIdentity.authoritySiteCount,
      residualRawPropertyCarrierSiteCount: census.propertyIdentity.residualRawCarrierSiteCount,
      residualCarrierConsumerSiteCount: census.propertyIdentity.residualCarrierConsumerSiteCount,
      residualIdentityShapedConsumerCount:
        census.propertyIdentity.residualIdentityShapedConsumerCount,
      rawPropertyIdentitySiteCount: census.propertyIdentity.rawStringIdentitySiteCount,
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

function inspectPropertyTypeSeal(): PropertyIdentityTypeSeal {
  const relativePath = "rust/crates/omena-syntax/src/ident.rs" as const;
  let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  if (injectPropertyCaseFold) {
    source = source.replace(
      /CanonicalCustomPropertyNameV0\(\s*decoded\.clone\(\),/u,
      "CanonicalCustomPropertyNameV0(decoded.to_ascii_lowercase(),",
    );
  }
  if (injectPropertyDecodeNeuter) {
    const decoderNeedle = "let decoded = decode_css_identifier_escapes(&authored).into_owned();";
    assert.equal(
      source.split(decoderNeedle).length - 1,
      2,
      "property decoder mutation must cover both explicit and inferred-kind constructors",
    );
    source = source.replaceAll(decoderNeedle, "let decoded = authored.clone();");
  }

  const propertyNameDerives = derivesForEnum(source, "PropertyNameV0");
  const authoredTextDerives = derivesForStruct(source, "AuthoredPropertyTextV0");
  const standardKeyDerives = derivesForStruct(source, "CanonicalStandardPropertyNameV0");
  const customKeyDerives = derivesForStruct(source, "CanonicalCustomPropertyNameV0");
  assert.deepEqual(
    authoredTextDerives,
    ["Debug", "Clone", "serde::Serialize"],
    "AuthoredPropertyTextV0 must remain presentation-only and structurally non-comparable",
  );
  assert.match(
    source,
    /pub struct AuthoredPropertyTextV0\s*\(\s*String\s*\)\s*;/u,
    "authored property text storage must remain private",
  );
  assert.match(
    source,
    /impl fmt::Display for AuthoredPropertyTextV0/u,
    "authored property text must expose presentation through Display",
  );
  const authoredImplStart = source.indexOf("impl AuthoredPropertyTextV0");
  const authoredImplOpen = source.indexOf("{", authoredImplStart);
  const authoredImplClose = matchingBrace(source, authoredImplOpen);
  assert.ok(
    authoredImplStart >= 0 && authoredImplOpen >= 0 && authoredImplClose !== undefined,
    "AuthoredPropertyTextV0 impl must remain inspectable",
  );
  const authoredImpl = source.slice(authoredImplOpen + 1, authoredImplClose);
  assert.deepEqual(
    [...authoredImpl.matchAll(/\bpub fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/gu)].map(
      (match) => match[1],
    ),
    ["new", "write_into"],
    "AuthoredPropertyTextV0 may expose only construction plus bounded presentation",
  );
  assert.doesNotMatch(
    authoredImpl,
    /pub fn as_str\s*\(&self\)/u,
    "authored property text must not expose a borrowable raw string",
  );
  assert.match(
    authoredImpl,
    /pub fn write_into\s*\(&self,\s*out:\s*&mut impl fmt::Write\)\s*->\s*fmt::Result/u,
    "authored property text must expose bounded writer-based presentation",
  );
  const propertyImplStart = source.indexOf("impl PropertyNameV0");
  const propertyImplOpen = source.indexOf("{", propertyImplStart);
  const propertyImplClose = matchingBrace(source, propertyImplOpen);
  assert.ok(
    propertyImplStart >= 0 && propertyImplOpen >= 0 && propertyImplClose !== undefined,
    "PropertyNameV0 impl must remain inspectable",
  );
  assert.doesNotMatch(
    source.slice(propertyImplOpen + 1, propertyImplClose),
    /pub fn authored\s*\(&self\)\s*->\s*&str/u,
    "PropertyNameV0 must not reintroduce the authored raw-string borrow",
  );
  assert.deepEqual(
    propertyNameDerives,
    ["Debug", "Clone"],
    "PropertyNameV0 derives must forbid raw structural equality and ordering",
  );
  assert.deepEqual(
    standardKeyDerives,
    ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    "CanonicalStandardPropertyNameV0 must remain a sealed equality carrier",
  );
  assert.deepEqual(
    customKeyDerives,
    ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    "CanonicalCustomPropertyNameV0 must remain a sealed equality carrier",
  );
  assert.match(
    source,
    /pub struct CanonicalStandardPropertyNameV0\s*\(\s*String\s*,\s*CanonicalStandardPropertyNameSealV0\s*\)\s*;/u,
    "standard property key fields must remain private and sealed",
  );
  assert.match(
    source,
    /pub struct CanonicalCustomPropertyNameV0\s*\(\s*String\s*,\s*CanonicalCustomPropertyNameSealV0\s*\)\s*;/u,
    "custom property key fields must remain private and sealed",
  );
  assert.match(
    source,
    /let decoded = decode_css_identifier_escapes\(&authored\)\.into_owned\(\);/u,
    "property identity must share the CSS identifier escape decoder",
  );
  assert.match(
    source,
    /CanonicalStandardPropertyNameV0\(\s*decoded\.to_ascii_lowercase\(\),/u,
    "standard property canonicalization must remain ASCII-case-insensitive",
  );
  assert.match(
    source,
    /CanonicalCustomPropertyNameV0\(\s*decoded\.clone\(\),/u,
    "custom property canonicalization must preserve decoded case",
  );
  assert.match(
    source,
    /pub fn property_names_same\([^)]*\)[^{]*\{\s*PropertyNameV0::from_authored\(left\)\.same_as\(&PropertyNameV0::from_authored\(right\)\)/su,
    "property comparison helper must delegate to PropertyNameV0::same_as",
  );

  return {
    path: relativePath,
    authoredTextDerives: ["Debug", "Clone", "serde::Serialize"],
    authoredTextIdentityDerives: [],
    authoredTextFieldVisibility: "private",
    authoredTextPresentationMethods: ["Display", "write_into", "Serialize"],
    propertyNameDerives: ["Debug", "Clone"],
    propertyNameEqualityDerives: [],
    standardKeyDerives: ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    customKeyDerives: ["Debug", "Clone", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash"],
    keyFieldVisibility: "private",
    sharedDecoder: "decode_css_identifier_escapes",
    standardCanonicalization: "trim-decode-ascii-lowercase",
    customCanonicalization: "trim-decode-case-preserved",
    equalityMethod: "PropertyNameV0::same_as",
  };
}

function discoverPropertyIdentitySites(): PropertyIdentitySite[] {
  const tokenPatterns = [
    ["PropertyNameV0::from_authored", /PropertyNameV0::from_authored\s*\(/u],
    ["PropertyNameV0::custom", /PropertyNameV0::custom\s*\(/u],
    ["PropertyNameV0::standard", /PropertyNameV0::standard\s*\(/u],
    ["PropertyNameV0::canonical_custom_key", /PropertyNameV0::canonical_custom_key\s*\(/u],
    ["PropertyNameV0::canonical_standard_key", /PropertyNameV0::canonical_standard_key\s*\(/u],
    [
      "PropertyNameV0::canonical_key",
      /\b(?:property|property_name|property_identity|canonical_property)\.canonical_key\s*\(\s*\)/u,
    ],
    ["sealed-property-key", /\b(?:[A-Za-z_][A-Za-z0-9_]*\.)?property_key\b/u],
    [
      "PropertyNameV0::same_as",
      /(?:\b(?:property|property_name|property_identity|canonical_property)\.same_as\s*\(|\.same_as\s*\(\s*&PropertyNameV0)/u,
    ],
    ["property_names_same", /\bproperty_names_same\s*\(/u],
    ["CanonicalPropertyKeyV0", /\bCanonicalPropertyKeyV0\b/u],
    ["CanonicalCustomPropertyNameV0", /\bCanonicalCustomPropertyNameV0\b/u],
    ["CanonicalStandardPropertyNameV0", /\bCanonicalStandardPropertyNameV0\b/u],
  ] as const;
  const sites: PropertyIdentitySite[] = [];
  for (const relativePath of productionSources) {
    let source = readFileSync(path.join(repoRoot, relativePath), "utf8");
    if (
      injectPropertyAuthorityDecreaseLaundering &&
      relativePath === "rust/crates/omena-lsp-server/src/lib.rs"
    ) {
      const authorityJoin = "target.property_key == candidate.property_key";
      assert.equal(
        source.split(authorityJoin).length - 1,
        1,
        "authority census decrease mutation must replace exactly one sealed join",
      );
      source = source.replace(
        authorityJoin,
        "target.name.to_string().eq_ignore_ascii_case(&candidate.name.to_string())",
      );
    }
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const sourceLines = source.split(/\r?\n/u);
    for (const [index, line] of scannable.split(/\r?\n/u).entries()) {
      const match = tokenPatterns.find(([, pattern]) => pattern.test(line));
      if (!match) continue;
      const evidence = sourceLines[index]?.trim().replace(/\s+/gu, " ") ?? "";
      const functionName = enclosingFunctionName(scannable, offsetForLine(scannable, index + 1));
      const operation = match[0];
      const role: PropertyIdentitySiteRole =
        relativePath === "rust/crates/omena-syntax/src/ident.rs"
          ? "authority"
          : /property_key\s*:|canonical_custom_key|canonical_key/u.test(evidence)
            ? "producer"
            : /property_names_same|same_as|\.get\s*\(|contains_key|\.entry\s*\(/u.test(evidence)
              ? "consumer"
              : /Canonical(?:Custom|Standard)?Property/u.test(evidence)
                ? "carrier"
                : "consumer";
      sites.push({
        path: relativePath,
        line: index + 1,
        function: functionName,
        operation,
        evidence,
        role,
      });
    }
  }
  return uniqueSites(sites);
}

function discoverRawPropertyIdentitySites(): DiscoveredSite[] {
  return uniqueSites([
    ...discoverTypedRawPropertyIdentitySites(),
    ...discoverAuthoredPropertyIdentityLaunderingSites(),
  ]);
}

function discoverAuthoredPropertyIdentityLaunderingSites(): DiscoveredSite[] {
  const sources: MutableRustSource[] = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  sources.push(
    ...(generatedFixtureManifest?.sources.map((source) => ({
      relativePath: source.relativePath,
      source: source.source,
    })) ?? []),
  );
  applyPropertyAuthorityDecreaseMutation(sources);
  applyMigrateConsumerProbeMutations(sources);
  const carrierFields = discoverAuthoredPropertyCarrierFields(sources);
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const authoredReturningFunctions = authoredReturningFunctionNames(sources, carrierFields);
  const sites: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const classifier = propertyOperandClassifier(
        functionSlice,
        carrierFields,
        false,
        aliasesByPath.get(relativePath),
        authoredReturningFunctions,
      );
      const tokens = rustSemanticTokens(functionSlice.scannable);
      for (const [tokenIndex, token] of tokens.entries()) {
        if (token.text !== "==" && token.text !== "!=") continue;
        const left = operationSideTokens(tokens, tokenIndex, "left");
        const right = operationSideTokens(tokens, tokenIndex, "right");
        if (
          !classifier.containsRawPropertyOrigin(left) &&
          !classifier.containsRawPropertyOrigin(right)
        )
          continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "authored-property-identity-laundering",
          ),
        );
      }
      for (const [tokenIndex, token] of tokens.entries()) {
        if (
          !/^(?:eq|ne|eq_ignore_ascii_case|cmp|partial_cmp|is_eq)$/u.test(token.text) ||
          tokens[tokenIndex - 1]?.text !== "." ||
          tokens[tokenIndex + 1]?.text !== "("
        ) {
          continue;
        }
        const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 1, "(", ")");
        const receiver = operationSideTokens(tokens, tokenIndex - 1, "left");
        const argumentTokens =
          closeParen === undefined ? [] : tokens.slice(tokenIndex + 2, closeParen);
        if (
          !classifier.containsRawPropertyOrigin(receiver) &&
          !classifier.containsRawPropertyOrigin(argumentTokens)
        ) {
          continue;
        }
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "authored-property-identity-method-laundering",
          ),
        );
      }
      for (let tokenIndex = 0; tokenIndex + 3 < tokens.length; tokenIndex += 1) {
        const ufcsEquality =
          (tokens[tokenIndex].text === "str" || tokens[tokenIndex].text === "PartialEq") &&
          tokens[tokenIndex + 1]?.text === "::" &&
          tokens[tokenIndex + 2]?.text === "eq" &&
          tokens[tokenIndex + 3]?.text === "(";
        if (!ufcsEquality) continue;
        const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 3, "(", ")");
        if (closeParen === undefined) continue;
        if (!classifier.containsRawPropertyOrigin(tokens.slice(tokenIndex + 4, closeParen)))
          continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + tokens[tokenIndex + 2].start,
            "authored-property-ufcs-identity-laundering",
          ),
        );
      }
      for (const [tokenIndex, token] of tokens.entries()) {
        if (
          token.text !== "to_ascii_lowercase" &&
          token.text !== "make_ascii_lowercase" &&
          token.text !== "to_lowercase" &&
          token.text !== "to_ascii_uppercase" &&
          token.text !== "make_ascii_uppercase" &&
          token.text !== "to_uppercase" &&
          token.text !== "trim_matches" &&
          token.text !== "strip_prefix"
        )
          continue;
        if (tokens[tokenIndex - 1]?.text !== "." || tokens[tokenIndex + 1]?.text !== "(") continue;
        const receiver = operationSideTokens(tokens, tokenIndex - 1, "left");
        if (!classifier.containsRawPropertyOrigin(receiver)) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "authored-property-canonicalization-laundering",
          ),
        );
      }
      for (const collection of rawStringCollectionDeclarations(functionSlice)) {
        if (!classifier.collectionReceivesPropertyKey(collection.binding)) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + collection.offset,
            "authored-property-map-laundering",
          ),
        );
        if (classifier.collectionUsesIdentityOperation(collection.binding)) {
          sites.push(
            siteAt(
              relativePath,
              source,
              scannable,
              functionSlice.bodyStart + collection.offset,
              "authored-property-collection-identity-laundering",
            ),
          );
        }
      }
      for (const [tokenIndex, token] of tokens.entries()) {
        if (token.text === "match") {
          const openBrace = tokens.findIndex(
            (candidate, index) => index > tokenIndex && candidate.text === "{",
          );
          if (
            openBrace > tokenIndex &&
            classifier.containsRawPropertyOrigin(tokens.slice(tokenIndex + 1, openBrace))
          ) {
            sites.push(
              siteAt(
                relativePath,
                source,
                scannable,
                functionSlice.bodyStart + token.start,
                "authored-property-match-identity-laundering",
              ),
            );
          }
        }
        if (
          token.text === "matches" &&
          tokens[tokenIndex + 1]?.text === "!" &&
          tokens[tokenIndex + 2]?.text === "("
        ) {
          const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 2, "(", ")");
          if (closeParen === undefined) continue;
          const firstArgument = firstArgumentTokens(tokens.slice(tokenIndex + 3, closeParen));
          if (!classifier.containsRawPropertyOrigin(firstArgument)) continue;
          sites.push(
            siteAt(
              relativePath,
              source,
              scannable,
              functionSlice.bodyStart + token.start,
              "authored-property-matches-identity-laundering",
            ),
          );
        }
      }
    }
  }
  return uniqueSites(sites);
}

function discoverAuthoredPropertyCarrierFields(
  sources: readonly { source: string }[],
): ReadonlyMap<string, ReadonlySet<string>> {
  const fields = new Map<string, Set<string>>();
  for (const { source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const match of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = match.index + match[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const typeName = match[1];
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of body.matchAll(
        /\b(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,}\n]+)/gu,
      )) {
        if (!/\bAuthoredPropertyTextV0\b/u.test(field[2])) continue;
        const typeFields = fields.get(typeName) ?? new Set<string>();
        typeFields.add(field[1]);
        fields.set(typeName, typeFields);
      }
    }
  }
  return fields;
}

function discoverResidualRawPropertyCarrierSites(
  sources: readonly MutableRustSource[],
): DiscoveredSite[] {
  const sites: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const match of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = match.index + match[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const typeName = match[1];
      const body = scannable.slice(openBrace + 1, closeBrace);
      const hasSealedPropertyIdentity =
        /\bCanonical(?:Custom|Standard)?Property(?:Name|Key)V0\b/u.test(body);
      for (const field of body.matchAll(
        /\b(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,}\n]+)/gu,
      )) {
        const fieldName = field[1];
        const rustType = field[2].trim().replace(/\s+/gu, " ");
        if (
          hasSealedPropertyIdentity ||
          !rawStringType(rustType) ||
          !rawPropertyCarrierMember(typeName, fieldName, scannable)
        )
          continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            openBrace + 1 + field.index,
            `residual-property-field:${typeName}.${fieldName}:${rustType}`,
          ),
        );
      }
    }

    for (const functionSlice of rustFunctionSlices(scannable)) {
      for (const parameter of functionSlice.signature.matchAll(
        /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?str|String)\b/gu,
      )) {
        const parameterName = parameter[1];
        if (
          !isPropertySemanticIdentifier(parameterName) &&
          !(
            functionSlice.propertyContext &&
            /^(?:name|left|right|actual|expected|target|candidate|declared)$/u.test(parameterName)
          )
        ) {
          continue;
        }
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart - functionSlice.signature.length + parameter.index,
            `residual-property-parameter:${functionSlice.name}.${parameterName}:${parameter[2].replace(/\s+/gu, " ")}`,
          ),
        );
      }
    }
  }
  return uniqueSites(sites);
}

function rawPropertyCarrierMember(typeName: string, fieldName: string, source?: string): boolean {
  if (isPropertySemanticIdentifier(fieldName)) return true;
  if (
    isPropertySemanticIdentifier(typeName) &&
    /^(?:name|names|text|key|keys|first|second|left|right|actual|expected|target|candidate|declared)$/u.test(
      fieldName,
    )
  ) {
    return true;
  }
  if (fieldName !== "name" || source === undefined) return false;
  if (
    new Set([
      "SassMigrationOracleCompilerV0",
      "DartSassCompilerV0",
      "StyleSelectorV2",
      "DifferentialCase",
      "OmenaScssEvalNativeCssFunctionV0",
      "SpecSourcePinV0",
    ]).has(typeName)
  ) {
    return false;
  }
  const typedBindings = [
    ...source.matchAll(
      new RegExp(
        String.raw`\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*&?\s*(?:mut\s+)?${escapeRegExp(typeName)}\b`,
        "gu",
      ),
    ),
  ].map((match) => match[1]);
  return typedBindings.some((binding) =>
    new RegExp(
      String.raw`\b${escapeRegExp(binding)}\.${fieldName}(?:\.(?:as_ref|as_str|clone|to_owned|to_string|trim|trim_end|trim_start)\s*\(\s*\))*\s*(?:==|!=)|\b${escapeRegExp(binding)}\.${fieldName}(?:\.(?:to_ascii_lowercase|make_ascii_lowercase|to_lowercase)\s*\()`,
      "u",
    ).test(source),
  );
}

function classifyResidualRawPropertyCarrierSites(
  discovered: readonly DiscoveredSite[],
  previous: readonly CensusSite[] | undefined,
): readonly (CensusSite | (DiscoveredSite & { disposition: "unclassified" }))[] {
  const previousByKey = new Map((previous ?? []).map((site) => [stableSiteKey(site), site]));
  const previousByCarrier = new Map(
    (previous ?? []).map((site) => [
      `${site.path}\u0000${site.function}\u0000${site.operation}`,
      site,
    ]),
  );
  return discovered.map((site) => {
    const prior =
      previousByKey.get(stableSiteKey(site)) ??
      previousByCarrier.get(`${site.path}\u0000${site.function}\u0000${site.operation}`);
    if (prior)
      return { ...site, disposition: prior.disposition, reason: prior.reason } as CensusSite;
    if (!previous && writeMode) {
      const boundaryParameter = site.operation.startsWith("residual-property-parameter:");
      const evidenceOnly = /(?:omena-diff-test|engine-style-parser|examples?\/|bench)/u.test(
        site.path,
      );
      return {
        ...site,
        disposition: "named-exempt",
        reason: evidenceOnly
          ? "Authored fixture or differential-evidence spelling; this site does not carry product identity."
          : boundaryParameter
            ? "Borrowed authored boundary spelling; the consumer must construct a sealed key before identity use."
            : "Serialized authored boundary or presentation spelling; identity decisions must use a separately carried sealed key.",
      } as const;
    }
    return { ...site, disposition: "unclassified" } as const;
  });
}

function discoverResidualPropertyCarrierConsumers(
  carriers: readonly CensusSite[],
  sources: readonly MutableRustSource[],
): ResidualPropertyCarrierConsumerRow[] {
  const carrierByField = new Map<string, CensusSite>();
  const carrierByParameter = new Map<string, CensusSite>();
  for (const carrier of carriers) {
    const field = carrier.operation.match(
      /^residual-property-field:([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*):/u,
    );
    if (field) carrierByField.set(`${field[1]}.${field[2]}`, carrier);
    const parameter = carrier.operation.match(
      /^residual-property-parameter:([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*):/u,
    );
    if (parameter)
      carrierByParameter.set(`${carrier.path}\u0000${parameter[1]}.${parameter[2]}`, carrier);
  }

  const aliasesByPath = rustTypeAliasesByPath(sources);
  const structFieldTypes = rustStructFieldTypes(sources, aliasesByPath);
  const consumersByCarrier = new Map<CensusSite, ResidualPropertyConsumerSite[]>();
  for (const carrier of carriers) consumersByCarrier.set(carrier, []);

  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const bindings = resolvedRustBindingsForFunction(
        functionSlice,
        scannable,
        aliases,
        structFieldTypes,
      );
      for (const access of functionSlice.scannable.matchAll(
        /\b([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)\b/gu,
      )) {
        const receiverType = bindings.get(access[1]);
        if (!receiverType) continue;
        const carrier = carrierByField.get(`${receiverType}.${access[2]}`);
        if (!carrier) continue;
        const offset = functionSlice.bodyStart + access.index;
        consumersByCarrier
          .get(carrier)
          ?.push(
            residualConsumerSiteAt(
              relativePath,
              source,
              scannable,
              offset,
              functionSlice,
              `${access[1]}.${access[2]}`,
            ),
          );
      }

      for (const [key, carrier] of carrierByParameter) {
        const [carrierPath, functionAndParameter] = key.split("\u0000");
        if (carrierPath !== relativePath) continue;
        const separator = functionAndParameter.lastIndexOf(".");
        const functionName = functionAndParameter.slice(0, separator);
        const parameterName = functionAndParameter.slice(separator + 1);
        if (functionName !== functionSlice.name) continue;
        const parameterUse = new RegExp(`\\b${escapeRegExp(parameterName)}\\b`, "gu");
        const authorityShadow = functionSlice.scannable.match(
          new RegExp(
            String.raw`\blet\s+${escapeRegExp(parameterName)}\s*=\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*PropertyNameV0\s*::[^;]*\b${escapeRegExp(parameterName)}\b[^;]*;`,
            "u",
          ),
        );
        const authorityShadowEnd = authorityShadow
          ? (authorityShadow.index ?? 0) + authorityShadow[0].length
          : undefined;
        for (const use of functionSlice.scannable.matchAll(parameterUse)) {
          if (authorityShadowEnd !== undefined && use.index >= authorityShadowEnd) continue;
          const offset = functionSlice.bodyStart + use.index;
          consumersByCarrier
            .get(carrier)
            ?.push(
              residualConsumerSiteAt(
                relativePath,
                source,
                scannable,
                offset,
                functionSlice,
                parameterName,
              ),
            );
        }
      }
    }
  }

  return carriers.map((carrier) => ({
    carrier,
    consumers: uniqueSites(consumersByCarrier.get(carrier) ?? []),
  }));
}

function residualConsumerSiteAt(
  relativePath: string,
  source: string,
  scannable: string,
  offset: number,
  functionSlice: RustFunctionSlice,
  expression: string,
): ResidualPropertyConsumerSite {
  const localOffset = offset - functionSlice.bodyStart;
  const contextStart = Math.max(
    0,
    Math.max(
      functionSlice.scannable.lastIndexOf(";", localOffset - 1),
      functionSlice.scannable.lastIndexOf("{", localOffset - 1),
      functionSlice.scannable.lastIndexOf("}", localOffset - 1),
    ) + 1,
  );
  const nextSemicolon = functionSlice.scannable.indexOf(";", localOffset);
  const contextEnd =
    nextSemicolon < 0
      ? Math.min(functionSlice.scannable.length, localOffset + 320)
      : Math.min(functionSlice.scannable.length, nextSemicolon + 1);
  const context = functionSlice.scannable.slice(contextStart, contextEnd);
  const sourceLine = source.split(/\r?\n/u)[lineNumberAt(source, offset) - 1] ?? "";
  const expressionCore = escapeRegExp(expression).replaceAll("\\.", String.raw`\s*\.\s*`);
  const expressionPattern = String.raw`(?<![A-Za-z0-9_.])${expressionCore}(?![A-Za-z0-9_])`;
  const authorityCall = new RegExp(
    String.raw`(?:PropertyNameV0\s*::\s*(?:from_authored|custom|standard|canonical_custom_key|canonical_standard_key)|property_names_same)\s*\([^;{}]*?${expressionPattern}[^;{}]*?\)`,
    "gu",
  );
  const identityContext = context.replace(authorityCall, "sealed_property_result");
  const identityLine = sourceLine.replace(authorityCall, "sealed_property_result");
  const directMethod = new RegExp(
    String.raw`${expressionPattern}(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*\s*\([^;{}]*?\))*\s*\.\s*(?:eq|ne|eq_ignore_ascii_case|cmp|partial_cmp|is_eq)\s*\(`,
    "u",
  );
  const directBinary = new RegExp(
    String.raw`(?:${expressionPattern}(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*\s*\([^;{}]*?\))*\s*(?:==|!=)|(?:==|!=)\s*&?\s*${expressionPattern})`,
    "u",
  );
  const collectionArgument = new RegExp(
    String.raw`\.\s*(?:insert|get|entry|contains_key|contains|remove|binary_search(?:_by|_by_key)?)\s*\([^;{}]*${expressionPattern}`,
    "u",
  );
  const directMatch = new RegExp(
    String.raw`(?:\bmatch\s+${expressionPattern}\b|\bmatches\s*!\s*\(\s*${expressionPattern}\b|\b(?:str|PartialEq)\s*::\s*eq\s*\([^;{}]*${expressionPattern})`,
    "u",
  );
  const identityShaped =
    directMethod.test(identityLine) ||
    directBinary.test(identityLine) ||
    collectionArgument.test(identityLine) ||
    directMatch.test(identityContext);
  const classification: ResidualPropertyConsumerClassification = identityShaped
    ? "identity-shaped"
    : /(?:serde|serialize|json\s*!|format\s*!|write\s*!|writeln\s*!|\.to_string\s*\(|\breturn\b)/u.test(
          context,
        )
      ? "egress"
      : "presentation";
  return {
    ...siteAt(
      relativePath,
      source,
      scannable,
      offset,
      `residual-property-consumer:${classification}`,
    ),
    classification,
  };
}

function rustTypeAliasesByPath(
  sources: readonly MutableRustSource[],
): ReadonlyMap<string, ReadonlyMap<string, string>> {
  const byPath = new Map<string, ReadonlyMap<string, string>>();
  for (const { relativePath, source } of sources) {
    const aliases = new Map<string, string>();
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const alias of scannable.matchAll(
      /\buse\s+(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Za-z_][A-Za-z0-9_]*)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/gu,
    )) {
      aliases.set(alias[2], alias[1]);
    }
    byPath.set(relativePath, aliases);
  }
  return byPath;
}

function rustStructFieldTypes(
  sources: readonly MutableRustSource[],
  aliasesByPath: ReadonlyMap<string, ReadonlyMap<string, string>>,
): ReadonlyMap<string, ReadonlyMap<string, string>> {
  const structures = new Map<string, Map<string, string>>();
  for (const { relativePath, source } of sources) {
    const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const structure of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = structure.index + structure[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const fields = structures.get(structure[1]) ?? new Map<string, string>();
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of body.matchAll(
        /\b(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,}\n]+)/gu,
      )) {
        fields.set(field[1], normalizeRustType(field[2], aliases));
      }
      structures.set(structure[1], fields);
    }
  }
  return structures;
}

function resolvedRustBindingsForFunction(
  functionSlice: RustFunctionSlice,
  source: string,
  aliases: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
): ReadonlyMap<string, string> {
  const bindings = new Map<string, string>();
  const text = `${functionSlice.signature}{${functionSlice.scannable}}`;
  for (const binding of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,)=;{]+(?:<[^;{=]+>)?)/gu,
  )) {
    bindings.set(binding[1], normalizeRustType(binding[2], aliases));
  }
  const selfType = enclosingImplType(source, functionSlice.bodyStart, aliases);
  if (selfType) bindings.set("self", selfType);

  for (let pass = 0; pass < 4; pass += 1) {
    let changed = false;
    for (const closure of functionSlice.scannable.matchAll(
      /\b([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\.iter\s*\(\s*\)(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^|]*\))*\s*\.(?:any|all|map|filter|filter_map|find|find_map|for_each|position|rposition)\s*\(\s*\|\s*&?\s*(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/gu,
    )) {
      const receiverType = resolveRustAccessType(closure[1], bindings, structFields);
      const elementType = rustCollectionElementType(receiverType, aliases) ?? receiverType;
      if (elementType && bindings.get(closure[2]) !== elementType) {
        bindings.set(closure[2], elementType);
        changed = true;
      }
    }
    if (!changed) break;
  }
  return bindings;
}

function normalizeRustType(typeSource: string, aliases: ReadonlyMap<string, string>): string {
  const cleaned = typeSource
    .replace(/&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?/gu, "")
    .replace(/\bmut\s+/gu, "")
    .trim();
  const identifiers = cleaned.match(/[A-Za-z_][A-Za-z0-9_]*/gu) ?? [];
  const named = identifiers.findLast((identifier) => /^[A-Z]/u.test(identifier));
  if (!named) return cleaned;
  return aliases.get(named) ?? named;
}

function rustCollectionElementType(
  typeSource: string | undefined,
  aliases: ReadonlyMap<string, string>,
): string | undefined {
  if (!typeSource) return undefined;
  const generic = typeSource.match(
    /(?:Vec|Box|Arc|Rc|Option|BTreeSet|HashSet)\s*<\s*([^,>]+)/u,
  )?.[1];
  const slice = typeSource.match(/\[\s*([^;\]]+)/u)?.[1];
  const candidate = generic ?? slice;
  return candidate ? normalizeRustType(candidate, aliases) : undefined;
}

function resolveRustAccessType(
  access: string,
  bindings: ReadonlyMap<string, string>,
  structFields: ReadonlyMap<string, ReadonlyMap<string, string>>,
): string | undefined {
  const [root, ...segments] = access.split(".");
  let current = bindings.get(root);
  for (const segment of segments) {
    if (!current) return undefined;
    current = structFields.get(current)?.get(segment);
  }
  return current;
}

function enclosingImplType(
  source: string,
  offset: number,
  aliases: ReadonlyMap<string, string>,
): string | undefined {
  let candidate: string | undefined;
  for (const implementation of source.matchAll(
    /\bimpl(?:\s*<[^>{}]*>)?\s+(?:[^{}]*?\s+for\s+)?([A-Za-z_][A-Za-z0-9_:]*)[^{}]*\{/gu,
  )) {
    if (implementation.index > offset) break;
    const openBrace = implementation.index + implementation[0].lastIndexOf("{");
    const closeBrace = matchingBrace(source, openBrace);
    if (closeBrace !== undefined && offset < closeBrace) {
      candidate = normalizeRustType(implementation[1], aliases);
    }
  }
  return candidate;
}

interface RustFunctionSlice {
  readonly name: string;
  readonly signature: string;
  readonly scannable: string;
  readonly bodyStart: number;
  readonly bodyEnd: number;
  readonly propertyContext: boolean;
}

interface RawStringCollectionDeclaration {
  readonly binding: string;
  readonly offset: number;
}

interface MutableRustSource {
  readonly relativePath: string;
  source: string;
}

function discoverTypedRawPropertyIdentitySites(): DiscoveredSite[] {
  const sources: MutableRustSource[] = productionSources.map((relativePath) => ({
    relativePath,
    source: readFileSync(path.join(repoRoot, relativePath), "utf8"),
  }));
  sources.push(
    ...(generatedFixtureManifest?.sources.map((source) => ({
      relativePath: source.relativePath,
      source: source.source,
    })) ?? []),
  );
  applyPropertyAuthorityDecreaseMutation(sources);
  if (injectPropertyRealFileMutation) {
    const winnerPath = "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs";
    const winnerSource = sources.find((source) => source.relativePath === winnerPath);
    assert.ok(winnerSource, "real-file property mutation target must be in census scope");
    const typedComparison =
      "PropertyNameV0::from_authored(&candidate.property).same_as(&pair_property)";
    assert.ok(
      winnerSource.source.includes(typedComparison),
      "real-file property mutation target must retain the typed comparison",
    );
    winnerSource.source = winnerSource.source.replace(
      typedComparison,
      "candidate.property == pair.property",
    );
  }
  if (injectPropertyStructuralEquality) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_property_identity.rs",
      source:
        "fn injected(left_property: String, right_property: String) -> bool { left_property == right_property }\n",
    });
  }
  if (injectPropertyRoundtripEquality) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_property_roundtrip.rs",
      source:
        "struct Input { property: String }\nfn injected(actual: &Input, expected: &str) -> bool { actual.property != expected.as_ref() }\n",
    });
  }
  if (injectPropertyRawMap) {
    sources.push({
      relativePath: "rust/crates/omena-query/src/injected_property_map.rs",
      source:
        "use std::collections::HashMap;\nfn injected() {\n  let custom_property_index: HashMap<&'static str, u8> = HashMap::new();\n  drop(custom_property_index);\n}\n",
    });
  }
  if (injectPropertyRawCanonicalization) {
    appendTrackedSourceMutation(
      sources,
      "fn normalize_property(left_property: &str) -> String { left_property.to_ascii_lowercase() }",
    );
  }
  if (injectPropertyFqnRawMap) {
    appendTrackedSourceMutation(
      sources,
      "fn injected_property_fqn_map() {\n  let custom_property_index: std::collections::BTreeMap<String, u8> = std::collections::BTreeMap::new();\n  drop(custom_property_index);\n}",
    );
  }
  if (injectPropertyValuesRawMap) {
    appendTrackedSourceMutation(
      sources,
      "fn injected_property_values_map() {\n  let custom_property_values: std::collections::BTreeMap<String, u8> = std::collections::BTreeMap::new();\n  drop(custom_property_values);\n}",
    );
  }
  if (injectPropertySameLineRawOperation) {
    appendTrackedSourceMutation(
      sources,
      "fn injected_same_line_property_operation(left_property: String, right_property: String) -> bool { let _ = omena_syntax::ident::PropertyNameV0::from_authored(&left_property); left_property == right_property }",
    );
  }
  if (injectPropertyNewFileRawComparison) {
    appendTrackedSourceMutation(
      sources,
      "struct PropertyOperands { first: String, second: String }\nfn compare(values: &PropertyOperands) -> bool { values.first == values.second }",
    );
  }
  if (injectPropertyNewFileRawCanonicalization) {
    appendTrackedSourceMutation(
      sources,
      "struct PropertySpelling { text: String }\nfn canonicalize(value: &PropertySpelling) -> String { value.text.to_lowercase() }",
    );
  }
  if (injectPropertyTrimChain) {
    appendTrackedSourceMutation(
      sources,
      "fn normalize_property_name(property_name: &str) -> String { property_name.trim().to_ascii_lowercase() }",
    );
  }
  if (injectPropertyContextRawOperations) {
    appendTrackedSourceMutation(
      sources,
      "fn normalize_property_axis_order(declared: &str) -> String { declared.trim().to_ascii_lowercase() }\nfn compare_property_axis_order(left_name: &str, right_name: &str) -> bool { left_name == right_name }",
    );
  }
  if (injectPropertyAutomaticCarrier) {
    appendTrackedSourceMutation(
      sources,
      "pub struct DeclaredEntry { pub name: String }\nfn join_declared_entries(target: &DeclaredEntry, candidate: &DeclaredEntry) -> bool { target.name == candidate.name }",
      "rust/crates/omena-cascade/src/axis_order.rs",
    );
  }
  if (injectAuthoredUppercaseTransform) {
    appendTrackedSourceMutation(
      sources,
      'fn authored_property_uppercase_probe(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { property.to_string().to_uppercase() == "--PROBE" }',
      "rust/crates/omena-cli/src/migrate/mod.rs",
    );
  }
  if (injectAuthoredTrimMatchesTransform) {
    appendTrackedSourceMutation(
      sources,
      "fn authored_property_trim_matches_probe(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { property.to_string().trim_matches('-') == \"probe\" }",
      "rust/crates/omena-cli/src/migrate/mod.rs",
    );
  }
  if (injectAuthoredStripPrefixTransform) {
    appendTrackedSourceMutation(
      sources,
      'fn authored_property_strip_prefix_probe(property: &omena_syntax::ident::AuthoredPropertyTextV0) -> bool { property.to_string().strip_prefix("--") == Some("probe") }',
      "rust/crates/omena-cli/src/migrate/mod.rs",
    );
  }

  const carrierFields = discoverRawPropertyCarrierFields(sources);
  const functionResultKinds = rustFunctionResultKinds(sources);
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const sites: DiscoveredSite[] = [];
  for (const { relativePath, source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    const stringAwareScannable = maskCommentsStringsAndTestItems(source, true);
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const classifier = propertyOperandClassifier(
        functionSlice,
        carrierFields,
        true,
        aliasesByPath.get(relativePath),
      );
      const tokens = rustSemanticTokens(functionSlice.scannable);
      for (const [tokenIndex, token] of tokens.entries()) {
        if (token.text !== "==" && token.text !== "!=") continue;
        const left = maskNonRawResultCalls(
          maskPropertyAuthorityCalls(operationSideTokens(tokens, tokenIndex, "left")),
          functionResultKinds,
        );
        const right = maskNonRawResultCalls(
          maskPropertyAuthorityCalls(operationSideTokens(tokens, tokenIndex, "right")),
          functionResultKinds,
        );
        if (
          !classifier.containsRawPropertyAccess(left) &&
          !classifier.containsRawPropertyAccess(right)
        )
          continue;
        const absoluteOffset = functionSlice.bodyStart + token.start;
        const sourceLine = source.split(/\r?\n/u)[lineNumberAt(source, absoluteOffset) - 1] ?? "";
        if (
          !classifier.containsRawPropertyAccess(
            maskPropertyAuthorityCalls(rustSemanticTokens(sourceLine)),
          )
        )
          continue;
        sites.push(
          siteAt(relativePath, source, scannable, absoluteOffset, "raw-property-comparison"),
        );
      }

      for (const [tokenIndex, token] of tokens.entries()) {
        if (
          !/^(?:eq|ne|eq_ignore_ascii_case|cmp|partial_cmp|is_eq)$/u.test(token.text) ||
          tokens[tokenIndex - 1]?.text !== "." ||
          tokens[tokenIndex + 1]?.text !== "("
        ) {
          continue;
        }
        const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 1, "(", ")");
        const receiver = operationSideTokens(tokens, tokenIndex - 1, "left");
        const argumentTokens =
          closeParen === undefined ? [] : tokens.slice(tokenIndex + 2, closeParen);
        if (
          !classifier.containsRawPropertyOrigin(receiver) &&
          !classifier.containsRawPropertyOrigin(argumentTokens)
        ) {
          continue;
        }
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "raw-property-identity-method",
          ),
        );
      }

      for (let tokenIndex = 0; tokenIndex + 3 < tokens.length; tokenIndex += 1) {
        const ufcsEquality =
          (tokens[tokenIndex].text === "str" || tokens[tokenIndex].text === "PartialEq") &&
          tokens[tokenIndex + 1]?.text === "::" &&
          tokens[tokenIndex + 2]?.text === "eq" &&
          tokens[tokenIndex + 3]?.text === "(";
        if (!ufcsEquality) continue;
        const closeParen = matchingTokenDelimiter(tokens, tokenIndex + 3, "(", ")");
        if (closeParen === undefined) continue;
        if (!classifier.containsRawPropertyOrigin(tokens.slice(tokenIndex + 4, closeParen)))
          continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + tokens[tokenIndex + 2].start,
            "raw-property-ufcs-identity",
          ),
        );
      }

      for (const [tokenIndex, token] of tokens.entries()) {
        if (
          token.text !== "to_ascii_lowercase" &&
          token.text !== "make_ascii_lowercase" &&
          token.text !== "to_lowercase" &&
          token.text !== "to_ascii_uppercase" &&
          token.text !== "make_ascii_uppercase" &&
          token.text !== "to_uppercase"
        )
          continue;
        if (tokens[tokenIndex - 1]?.text !== "." || tokens[tokenIndex + 1]?.text !== "(") continue;
        const receiver = operationSideTokens(tokens, tokenIndex - 1, "left");
        if (!classifier.containsRawPropertyOrigin(receiver)) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + token.start,
            "raw-property-canonicalization",
          ),
        );
      }

      for (const collection of rawStringCollectionDeclarations(functionSlice)) {
        const propertyKeyed =
          isPropertyMapBinding(collection.binding) ||
          classifier.collectionReceivesPropertyKey(collection.binding);
        if (!propertyKeyed) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            scannable,
            functionSlice.bodyStart + collection.offset,
            "raw-custom-property-map",
          ),
        );
      }

      const operand = String.raw`&?\s*[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*(?:\.(?:as_ref|as_str)\s*\(\s*\))*`;
      const rawPrefix = new RegExp(
        String.raw`(?<operand>${operand})\.starts_with\s*\(\s*"--"\s*\)`,
        "gu",
      );
      const stringAwareBody = stringAwareScannable.slice(
        functionSlice.bodyStart,
        functionSlice.bodyEnd,
      );
      for (const match of stringAwareBody.matchAll(rawPrefix)) {
        if (!classifier.isRawPropertyExpression(match.groups?.operand ?? "")) continue;
        sites.push(
          siteAt(
            relativePath,
            source,
            stringAwareScannable,
            functionSlice.bodyStart + match.index,
            "raw-custom-property-prefix",
          ),
        );
      }
    }
  }
  return uniqueSites(sites);
}

function appendTrackedSourceMutation(
  sources: MutableRustSource[],
  mutation: string,
  relativePath = "rust/crates/omena-cascade/src/axis_order.rs",
): void {
  const target = sources.find((source) => source.relativePath === relativePath);
  assert.ok(target, "tracked property mutation target must be in census scope");
  target.source = `${target.source.trimEnd()}\n\n${mutation}\n`;
}

function applyMigrateConsumerProbeMutations(sources: MutableRustSource[]): void {
  const targetPath = "rust/crates/omena-cli/src/migrate/mod.rs";
  if (injectMigrateLowercaseComparison) {
    appendTrackedSourceMutation(
      sources,
      'fn migrate_property_lowercase_probe(candidate: &TransformPassCascadeOracleCaseV0) -> bool { candidate.property.to_ascii_lowercase() == "--probe" }',
      targetPath,
    );
  }
  if (injectMigrateFqnParameter) {
    appendTrackedSourceMutation(
      sources,
      'fn migrate_property_fqn_probe(candidate: &omena_diff_test::TransformPassCascadeOracleCaseV0) -> bool { candidate.property == "--probe" }',
      targetPath,
    );
  }
  if (injectMigrateAliasParameter) {
    appendTrackedSourceMutation(
      sources,
      'use omena_diff_test::TransformPassCascadeOracleCaseV0 as MigratePropertyProbe;\nfn migrate_property_alias_probe(candidate: &MigratePropertyProbe) -> bool { candidate.property == "--probe" }',
      targetPath,
    );
  }
  if (injectMigrateBareParameter) {
    appendTrackedSourceMutation(
      sources,
      'fn migrate_property_bare_probe(candidate: &TransformPassCascadeOracleCaseV0) -> bool { candidate.property == "--probe" }',
      targetPath,
    );
  }
  if (injectMigrateClosureParameter) {
    appendTrackedSourceMutation(
      sources,
      'fn migrate_property_closure_probe(candidates: &[TransformPassCascadeOracleCaseV0]) -> bool { candidates.iter().any(|candidate| candidate.property == "--probe") }',
      targetPath,
    );
  }
}

function applyPropertyAuthorityDecreaseMutation(sources: MutableRustSource[]): void {
  if (!injectPropertyAuthorityDecreaseLaundering) return;
  const targetPath = "rust/crates/omena-lsp-server/src/lib.rs";
  const target = sources.find((source) => source.relativePath === targetPath);
  assert.ok(target, "authority decrease mutation target must be in census scope");
  const authorityJoin = "target.property_key == candidate.property_key";
  const launderingJoin =
    "target.name.to_string().eq_ignore_ascii_case(&candidate.name.to_string())";
  assert.equal(
    target.source.split(authorityJoin).length - 1,
    1,
    "authority decrease mutation must replace exactly one sealed join",
  );
  target.source = target.source.replace(authorityJoin, launderingJoin);
}

function discoverRawPropertyCarrierFields(
  sources: readonly { relativePath: string; source: string }[],
): ReadonlyMap<string, ReadonlySet<string>> {
  const fields = new Map<string, Set<string>>();
  for (const { source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const match of scannable.matchAll(/\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{/gu)) {
      const openBrace = match.index + match[0].lastIndexOf("{");
      const closeBrace = matchingBrace(scannable, openBrace);
      if (closeBrace === undefined) continue;
      const typeName = match[1];
      const body = scannable.slice(openBrace + 1, closeBrace);
      for (const field of body.matchAll(
        /\b(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,}\n]+)/gu,
      )) {
        const fieldName = field[1];
        if (rawStringType(field[2]) && rawPropertyCarrierMember(typeName, fieldName, scannable)) {
          const typeFields = fields.get(typeName) ?? new Set<string>();
          typeFields.add(fieldName);
          fields.set(typeName, typeFields);
        }
      }
    }
  }
  return fields;
}

function rustFunctionSlices(scannable: string): RustFunctionSlice[] {
  const functions: RustFunctionSlice[] = [];
  for (const match of scannable.matchAll(/\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu)) {
    const openParen = scannable.indexOf("(", match.index + match[0].length);
    if (openParen < 0) continue;
    const closeParen = matchingDelimiter(scannable, openParen, "(", ")");
    if (closeParen === undefined) continue;
    const openBrace = scannable.indexOf("{", closeParen + 1);
    const semicolon = scannable.indexOf(";", closeParen + 1);
    if (openBrace < 0 || (semicolon >= 0 && semicolon < openBrace)) continue;
    const closeBrace = matchingBrace(scannable, openBrace);
    if (closeBrace === undefined) continue;
    const signature = scannable.slice(match.index, openBrace);
    const propertyContext = isPropertyIdentityFunctionIdentifier(match[1]);
    functions.push({
      name: match[1],
      signature,
      scannable: scannable.slice(openBrace + 1, closeBrace),
      bodyStart: openBrace + 1,
      bodyEnd: closeBrace,
      propertyContext,
    });
  }
  return functions;
}

function propertyOperandClassifier(
  functionSlice: RustFunctionSlice,
  carrierFields: ReadonlyMap<string, ReadonlySet<string>>,
  includeRawParameters = true,
  aliases: ReadonlyMap<string, string> = new Map(),
  authoredReturningFunctions: ReadonlySet<string> = new Set(),
): {
  readonly isRawPropertyExpression: (expression: string) => boolean;
  readonly containsRawPropertyAccess: (tokens: readonly RustSemanticToken[]) => boolean;
  readonly containsRawPropertyOrigin: (tokens: readonly RustSemanticToken[]) => boolean;
  readonly collectionReceivesPropertyKey: (binding: string) => boolean;
  readonly collectionUsesIdentityOperation: (binding: string) => boolean;
} {
  const text = `${functionSlice.signature}{${functionSlice.scannable}}`;
  const rawBindings = new Set<string>();
  const propertyBindings = new Set<string>();
  const bindingTypes = new Map<string, string>();
  for (const match of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?str|String)\b/gu,
  )) {
    if (!includeRawParameters) continue;
    rawBindings.add(match[1]);
    if (
      isPropertySemanticIdentifier(match[1]) ||
      (functionSlice.propertyContext &&
        /^(?:name|left|right|actual|expected|target|candidate|declared)$/u.test(match[1]))
    ) {
      propertyBindings.add(match[1]);
    }
  }
  for (const match of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*&?\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?([A-Za-z_][A-Za-z0-9_:]*)\b/gu,
  )) {
    const typeName = normalizeRustType(match[2], aliases);
    if (typeName !== "AuthoredPropertyTextV0") continue;
    rawBindings.add(match[1]);
    propertyBindings.add(match[1]);
    bindingTypes.set(match[1], typeName);
  }
  for (const match of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?)?(?:mut\s+)?([A-Za-z_][A-Za-z0-9_:]*)/gu,
  )) {
    bindingTypes.set(match[1], normalizeRustType(match[2], aliases));
  }
  for (const match of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*&?\s*\[\s*([A-Za-z_][A-Za-z0-9_:]*)\s*\]/gu,
  )) {
    bindingTypes.set(match[1], normalizeRustType(match[2], aliases));
  }
  for (const closure of text.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\.iter\s*\(\s*\)(?:\.[A-Za-z_][A-Za-z0-9_]*\s*\([^|]*\))*\s*\.(?:any|all|map|filter|filter_map|find|find_map|for_each|position|rposition)\s*\(\s*\|\s*&?\s*(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)/gu,
  )) {
    const elementType = bindingTypes.get(closure[1]);
    if (elementType) bindingTypes.set(closure[2], elementType);
  }

  const accessIsRawProperty = (access: RustAccessPath): boolean => {
    return rawPropertyExpressionKind(
      access,
      rawBindings,
      propertyBindings,
      bindingTypes,
      carrierFields,
    );
  };

  const containsRawPropertyAccess = (tokens: readonly RustSemanticToken[]): boolean => {
    return rustAccessPaths(tokens).some(accessIsRawProperty);
  };
  const containsRawPropertyOrigin = (tokens: readonly RustSemanticToken[]): boolean => {
    return (
      rustAccessPaths(tokens).some((access) =>
        rawPropertyExpressionKind(
          access,
          rawBindings,
          propertyBindings,
          bindingTypes,
          carrierFields,
          true,
        ),
      ) ||
      tokens.some(
        (token, index) =>
          authoredReturningFunctions.has(token.text) && tokens[index + 1]?.text === "(",
      )
    );
  };

  for (let pass = 0; pass < 4; pass += 1) {
    let changed = false;
    for (const assignment of functionSlice.scannable.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^=;]*=\s*([^;]+);/gu,
    )) {
      if (
        !propertyBindings.has(assignment[1]) &&
        !/\b(?:PropertyNameV0|CanonicalPropertyKeyV0|CanonicalCustomPropertyNameV0|CanonicalStandardPropertyNameV0)\b|\.canonical_key\s*\(/u.test(
          assignment[2],
        ) &&
        authoredStringExpression(
          assignment[2],
          containsRawPropertyOrigin,
          authoredReturningFunctions,
        )
      ) {
        propertyBindings.add(assignment[1]);
        rawBindings.add(assignment[1]);
        changed = true;
      }
    }
    if (!changed) break;
  }

  return {
    isRawPropertyExpression(expression: string): boolean {
      return containsRawPropertyAccess(rustSemanticTokens(expression));
    },
    containsRawPropertyAccess,
    containsRawPropertyOrigin,
    collectionReceivesPropertyKey(binding: string): boolean {
      const tokens = rustSemanticTokens(functionSlice.scannable);
      for (let index = 0; index + 3 < tokens.length; index += 1) {
        if (
          tokens[index].text !== binding ||
          tokens[index + 1].text !== "." ||
          !["entry", "get", "contains_key", "remove", "insert", "push"].includes(
            tokens[index + 2].text,
          ) ||
          tokens[index + 3].text !== "("
        ) {
          continue;
        }
        const closeParen = matchingTokenDelimiter(tokens, index + 3, "(", ")");
        if (closeParen === undefined) continue;
        const argument = firstArgumentTokens(tokens.slice(index + 4, closeParen));
        if (containsRawPropertyAccess(argument)) return true;
      }
      return false;
    },
    collectionUsesIdentityOperation(binding: string): boolean {
      const tokens = rustSemanticTokens(functionSlice.scannable);
      for (let index = 0; index + 3 < tokens.length; index += 1) {
        if (
          tokens[index].text === binding &&
          tokens[index + 1].text === "." &&
          /^(?:get|entry|contains_key|contains|remove|sort(?:_by|_by_key|_by_cached_key|_unstable(?:_by|_by_key)?)?|dedup(?:_by|_by_key)?|binary_search(?:_by|_by_key)?)$/u.test(
            tokens[index + 2].text,
          ) &&
          tokens[index + 3].text === "("
        ) {
          return true;
        }
      }
      return false;
    },
  };
}

function rawStringValueExpression(expression: string): boolean {
  return /^\s*&?\s*[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*(?:\(\s*\))?)*\s*$/u.test(
    expression,
  );
}

function authoredStringExpression(
  expression: string,
  containsOrigin: (tokens: readonly RustSemanticToken[]) => boolean,
  authoredReturningFunctions: ReadonlySet<string>,
): boolean {
  const tokens = rustSemanticTokens(expression);
  if (!containsOrigin(tokens)) return false;
  return (
    !/(?:==|!=|\b(?:matches|match)\b|\.\s*(?:eq|ne|eq_ignore_ascii_case|cmp|partial_cmp|is_eq)\s*\()/u.test(
      expression,
    ) &&
    (/^\s*format\s*!\s*\(/u.test(expression) ||
      rawStringValueExpression(expression) ||
      (() => {
        const called = expression.match(
          /^\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*\(/u,
        )?.[1];
        return called !== undefined && authoredReturningFunctions.has(called);
      })())
  );
}

function authoredReturningFunctionNames(
  sources: readonly MutableRustSource[],
  carrierFields: ReadonlyMap<string, ReadonlySet<string>>,
): ReadonlySet<string> {
  const returning = new Set<string>();
  const aliasesByPath = rustTypeAliasesByPath(sources);
  const structFields = rustStructFieldTypes(sources, aliasesByPath);
  for (let pass = 0; pass < 3; pass += 1) {
    let changed = false;
    for (const { relativePath, source } of sources) {
      const scannable = maskCommentsStringsAndTestItems(source, false);
      const aliases = aliasesByPath.get(relativePath) ?? new Map<string, string>();
      for (const functionSlice of rustFunctionSlices(scannable)) {
        if (returning.has(functionSlice.name)) continue;
        const returnType = functionSlice.signature.match(
          /->\s*([^\n{]+?)(?:\s+where\b|\s*$)/u,
        )?.[1];
        if (!returnType || !rawStringType(returnType.trim())) continue;
        const bindings = resolvedRustBindingsForFunction(
          functionSlice,
          source,
          aliases,
          structFields,
        );
        const directAuthoredParameter = [...bindings].some(
          ([binding, typeName]) =>
            typeName === "AuthoredPropertyTextV0" &&
            new RegExp(`\\b${escapeRegExp(binding)}\\b`, "u").test(functionSlice.scannable),
        );
        const authoredFieldAccess = rustAccessPaths(
          rustSemanticTokens(functionSlice.scannable),
        ).some((access) => {
          let currentType = bindings.get(access.root);
          for (const segment of access.segments) {
            if (segment.method) break;
            if (currentType && carrierFields.get(currentType)?.has(segment.name)) return true;
            currentType = currentType
              ? structFields.get(currentType)?.get(segment.name)
              : undefined;
          }
          return false;
        });
        const callsAuthoredReturning = [...returning].some((name) =>
          new RegExp(`\\b${escapeRegExp(name)}\\s*\\(`, "u").test(functionSlice.scannable),
        );
        if (!directAuthoredParameter && !authoredFieldAccess && !callsAuthoredReturning) continue;
        returning.add(functionSlice.name);
        changed = true;
      }
    }
    if (!changed) break;
  }
  return returning;
}

interface RustSemanticToken {
  readonly text: string;
  readonly start: number;
  readonly end: number;
}

type RustFunctionResultKind = "raw-string" | "non-raw";

function rustFunctionResultKinds(
  sources: readonly { source: string }[],
): ReadonlyMap<string, RustFunctionResultKind> {
  const results = new Map<string, RustFunctionResultKind>();
  for (const { source } of sources) {
    const scannable = maskCommentsStringsAndTestItems(source, false);
    for (const functionSlice of rustFunctionSlices(scannable)) {
      const returnType = functionSlice.signature.match(/->\s*([^\n{]+?)(?:\s+where\b|\s*$)/u)?.[1];
      if (!returnType) continue;
      const kind: RustFunctionResultKind = rawStringType(returnType.trim())
        ? "raw-string"
        : "non-raw";
      const prior = results.get(functionSlice.name);
      if (prior === "raw-string" || kind === "raw-string")
        results.set(functionSlice.name, "raw-string");
      else results.set(functionSlice.name, "non-raw");
    }
  }
  return results;
}

function maskNonRawResultCalls(
  tokens: readonly RustSemanticToken[],
  resultKinds: ReadonlyMap<string, RustFunctionResultKind>,
): readonly RustSemanticToken[] {
  const masked: RustSemanticToken[] = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (
      resultKinds.get(token.text) !== "non-raw" ||
      tokens[index + 1]?.text !== "(" ||
      tokens[index - 1]?.text === "." ||
      tokens[index - 1]?.text === "::"
    ) {
      masked.push(token);
      continue;
    }
    const closeIndex = matchingTokenDelimiter(tokens, index + 1, "(", ")");
    if (closeIndex === undefined) {
      masked.push(token);
      continue;
    }
    masked.push({ text: "non_raw_result", start: token.start, end: tokens[closeIndex].end });
    index = closeIndex;
  }
  return masked;
}

function maskPropertyAuthorityCalls(
  tokens: readonly RustSemanticToken[],
): readonly RustSemanticToken[] {
  const masked: RustSemanticToken[] = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    const propertyNameCall =
      token.text === "PropertyNameV0" &&
      tokens[index + 1]?.text === "::" &&
      /^(?:from_authored|custom|standard|canonical_custom_key|canonical_standard_key)$/u.test(
        tokens[index + 2]?.text ?? "",
      ) &&
      tokens[index + 3]?.text === "(";
    const helperCall = token.text === "property_names_same" && tokens[index + 1]?.text === "(";
    const openIndex = propertyNameCall ? index + 3 : helperCall ? index + 1 : undefined;
    if (openIndex === undefined) {
      masked.push(token);
      continue;
    }
    const closeIndex = matchingTokenDelimiter(tokens, openIndex, "(", ")");
    if (closeIndex === undefined) {
      masked.push(token);
      continue;
    }
    masked.push({
      text: "sealed_property_result",
      start: token.start,
      end: tokens[closeIndex].end,
    });
    index = closeIndex;
  }
  return masked;
}

interface RustAccessPath {
  readonly root: string;
  readonly segments: readonly RustAccessSegment[];
}

interface RustAccessSegment {
  readonly name: string;
  readonly method: boolean;
}

function rustSemanticTokens(source: string): RustSemanticToken[] {
  const tokens: RustSemanticToken[] = [];
  const compoundSymbols = new Set(["==", "!=", "::", "&&", "||", "=>", "<=", ">="]);
  for (let index = 0; index < source.length;) {
    const character = source[index];
    if (/\s/u.test(character)) {
      index += 1;
      continue;
    }
    if (/[A-Za-z_]/u.test(character)) {
      let end = index + 1;
      while (end < source.length && /[A-Za-z0-9_]/u.test(source[end])) end += 1;
      tokens.push({ text: source.slice(index, end), start: index, end });
      index = end;
      continue;
    }
    const pair = source.slice(index, index + 2);
    if (compoundSymbols.has(pair)) {
      tokens.push({ text: pair, start: index, end: index + 2 });
      index += 2;
      continue;
    }
    tokens.push({ text: character, start: index, end: index + 1 });
    index += 1;
  }
  return tokens;
}

function rustAccessPaths(tokens: readonly RustSemanticToken[]): RustAccessPath[] {
  const paths: RustAccessPath[] = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const root = tokens[index];
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/u.test(root.text)) continue;
    if (tokens[index - 1]?.text === "." || tokens[index - 1]?.text === "::") continue;
    const segments: RustAccessSegment[] = [];
    let cursor = index + 1;
    while (tokens[cursor]?.text === ".") {
      const member = tokens[cursor + 1];
      if (!member || !/^[A-Za-z_][A-Za-z0-9_]*$/u.test(member.text)) break;
      if (tokens[cursor + 2]?.text === "(") {
        const closeParen = matchingTokenDelimiter(tokens, cursor + 2, "(", ")");
        if (closeParen === undefined) break;
        segments.push({ name: member.text, method: true });
        cursor = closeParen + 1;
      } else {
        segments.push({ name: member.text, method: false });
        cursor += 2;
      }
    }
    paths.push({ root: root.text, segments });
  }
  return paths;
}

function rawPropertyExpressionKind(
  access: RustAccessPath,
  rawBindings: ReadonlySet<string>,
  propertyBindings: ReadonlySet<string>,
  bindingTypes: ReadonlyMap<string, string>,
  carrierFields: ReadonlyMap<string, ReadonlySet<string>>,
  _arbitraryMethodsPreserveOrigin = false,
): boolean {
  let rawProperty = rawBindings.has(access.root) && propertyBindings.has(access.root);
  const rootType = bindingTypes.get(access.root);
  for (const segment of access.segments) {
    if (!segment.method) {
      rawProperty = Boolean(rootType && carrierFields.get(rootType)?.has(segment.name));
      continue;
    }
    if (!rawProperty) return false;
  }
  return rawProperty;
}

function operationSideTokens(
  tokens: readonly RustSemanticToken[],
  operatorIndex: number,
  direction: "left" | "right",
): readonly RustSemanticToken[] {
  const boundaries = new Set([";", "{", "}", ",", "&&", "||", "=>", "=", "|"]);
  const keywordBoundaries = new Set(["if", "let", "match", "return", "while"]);
  if (direction === "left") {
    let depth = 0;
    let start = operatorIndex;
    for (let index = operatorIndex - 1; index >= 0; index -= 1) {
      const token = tokens[index].text;
      if ([")", "]"].includes(token)) depth += 1;
      else if (["(", "["].includes(token)) {
        if (depth === 0) break;
        depth -= 1;
      }
      if (depth === 0 && (boundaries.has(token) || keywordBoundaries.has(token))) break;
      start = index;
    }
    return tokens.slice(start, operatorIndex);
  }

  let depth = 0;
  let end = operatorIndex + 1;
  for (let index = operatorIndex + 1; index < tokens.length; index += 1) {
    const token = tokens[index].text;
    if (["(", "["].includes(token)) depth += 1;
    else if ([")", "]"].includes(token)) {
      if (depth === 0) break;
      depth -= 1;
    }
    if (depth === 0 && (boundaries.has(token) || keywordBoundaries.has(token))) break;
    end = index + 1;
  }
  return tokens.slice(operatorIndex + 1, end);
}

function matchingTokenDelimiter(
  tokens: readonly RustSemanticToken[],
  openIndex: number,
  open: string,
  close: string,
): number | undefined {
  let depth = 0;
  for (let index = openIndex; index < tokens.length; index += 1) {
    if (tokens[index].text === open) depth += 1;
    else if (tokens[index].text === close) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return undefined;
}

function firstArgumentTokens(tokens: readonly RustSemanticToken[]): readonly RustSemanticToken[] {
  let depth = 0;
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index].text;
    if (["(", "[", "{"].includes(token)) depth += 1;
    else if ([")", "]", "}"].includes(token)) depth -= 1;
    else if (token === "," && depth === 0) return tokens.slice(0, index);
  }
  return tokens;
}

function rawStringCollectionDeclarations(
  functionSlice: RustFunctionSlice,
): RawStringCollectionDeclaration[] {
  const declarations: RawStringCollectionDeclaration[] = [];
  const start =
    /\b(?:let\s+(?:mut\s+)?)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?:(?:[A-Za-z_][A-Za-z0-9_]*::)+)?(?:BTreeMap|BTreeSet|HashMap|HashSet|Vec)\s*</gu;
  for (const match of functionSlice.scannable.matchAll(start)) {
    const openAngle = match.index + match[0].lastIndexOf("<");
    const closeAngle = matchingDelimiter(functionSlice.scannable, openAngle, "<", ">");
    if (closeAngle === undefined) continue;
    const genericArguments = functionSlice.scannable.slice(openAngle + 1, closeAngle);
    if (!rawStringType(firstTopLevelGenericArgument(genericArguments))) continue;
    declarations.push({ binding: match[1], offset: match.index });
  }
  return declarations;
}

function firstTopLevelGenericArgument(argumentsSource: string): string {
  let depth = 0;
  for (let index = 0; index < argumentsSource.length; index += 1) {
    const character = argumentsSource[index];
    if (character === "<" || character === "(" || character === "[") depth += 1;
    else if (character === ">" || character === ")" || character === "]") depth -= 1;
    else if (character === "," && depth === 0) return argumentsSource.slice(0, index).trim();
  }
  return argumentsSource.trim();
}

function rawStringType(typeSource: string): boolean {
  return /^(?:String|&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?str)\b/u.test(typeSource.trim());
}

function isPropertySemanticIdentifier(identifier: string): boolean {
  const words = semanticIdentifierWords(identifier);
  return words.includes("property") || words.includes("properties");
}

function isPropertyIdentityFunctionIdentifier(identifier: string): boolean {
  const words = semanticIdentifierWords(identifier);
  if (!words.includes("property") && !words.includes("properties")) return false;
  return words.some((word) =>
    [
      "canonical",
      "canonicalize",
      "compare",
      "equal",
      "equality",
      "identity",
      "join",
      "key",
      "keys",
      "name",
      "names",
      "normalize",
      "same",
    ].includes(word),
  );
}

function semanticIdentifierWords(identifier: string): string[] {
  return identifier
    .replace(/([a-z0-9])([A-Z])/gu, "$1_$2")
    .toLowerCase()
    .split(/[^a-z0-9]+/u)
    .filter(Boolean);
}

function isPropertyMapBinding(identifier: string): boolean {
  if (!isPropertySemanticIdentifier(identifier)) return false;
  const words = semanticIdentifierWords(identifier);
  return !words.some((word) =>
    ["selector", "selectors", "path", "paths", "uri", "uris", "file", "files"].includes(word),
  );
}

function matchingDelimiter(
  source: string,
  openOffset: number,
  open: string,
  close: string,
): number | undefined {
  let depth = 0;
  for (let index = openOffset; index < source.length; index += 1) {
    if (source[index] === open) depth += 1;
    else if (source[index] === close) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return undefined;
}

function derivesForStruct(source: string, structName: string): string[] {
  const match = source.match(
    new RegExp(
      `#\\s*\\[\\s*derive\\s*\\(([^)]*)\\)\\s*\\]\\s*(?:#\\s*\\[[^\\]]*\\]\\s*)*pub\\s+struct\\s+${structName}\\b`,
      "u",
    ),
  );
  assert.ok(match, `derive list missing for ${structName}`);
  return match[1]
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
}

function derivesForEnum(source: string, enumName: string): string[] {
  const match = source.match(
    new RegExp(`#\\s*\\[\\s*derive\\s*\\(([^)]*)\\)\\s*\\]\\s*pub\\s+enum\\s+${enumName}\\b`, "u"),
  );
  assert.ok(match, `derive list missing for ${enumName}`);
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
      [
        "CanonicalClassKeyV0::as_str",
        /\b([A-Za-z_][A-Za-z0-9_]*)\.canonical_key\s*\(\s*\)\s*\.as_str\s*\(\s*\)/gu,
      ],
    ];
    for (const [operation, expression] of candidatePatterns) {
      expression.lastIndex = 0;
      for (const match of scannable.matchAll(expression)) {
        if (
          operation === "CanonicalClassKeyV0::as_str" &&
          match[1]?.toLowerCase().includes("property")
        ) {
          continue;
        }
        discovered.push(siteAt(relativePath, source, scannable, match.index, operation));
      }
    }
    const canonicalVariables = new Set<string>();
    for (const match of scannable.matchAll(
      /\blet\s+([A-Za-z_][A-Za-z0-9_]*)[^=;\n]*=\s*[^;\n]*\.canonical_key\s*\(\s*\)/gu,
    )) {
      if (/property/u.test(match[0])) continue;
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

function assertNoAddedCarrierSites(
  previous: readonly CensusSite[],
  current: readonly CensusSite[],
  label: string,
): void {
  const previousKeys = new Set(previous.map(stableCarrierKey));
  const addedSites = current.filter((site) => !previousKeys.has(stableCarrierKey(site)));
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

function stableCarrierKey(site: Pick<DiscoveredSite, "path" | "function" | "operation">): string {
  return `${site.path}\u0000${site.function}\u0000${site.operation}`;
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
  assert.ok(parsed.propertyIdentity, "committed property-identity census is required");
  assert.equal(
    parsed.propertyIdentity.authoritySiteCount,
    parsed.propertyIdentity.sites.length,
    "property authority site count",
  );
  assert.equal(
    parsed.propertyIdentity.siteDigest,
    digest(parsed.propertyIdentity.sites),
    "property authority site digest",
  );
  assert.ok(
    parsed.propertyIdentity.residualRawCarrierSites,
    "committed residual raw property carriers are required",
  );
  assert.equal(
    parsed.propertyIdentity.residualRawCarrierSiteCount,
    parsed.propertyIdentity.residualRawCarrierSites.length,
    "residual raw property carrier site count",
  );
  assert.equal(
    parsed.propertyIdentity.residualRawCarrierSiteDigest,
    digest(parsed.propertyIdentity.residualRawCarrierSites),
    "residual raw property carrier site digest",
  );
  assert.equal(
    parsed.propertyIdentity.rawStringIdentitySiteCount,
    0,
    "raw property identity site count",
  );
  assert.equal(
    parsed.propertyIdentity.rawStringIdentitySiteDigest,
    digest(parsed.propertyIdentity.rawStringIdentitySites),
    "raw property identity site digest",
  );
  return parsed;
}

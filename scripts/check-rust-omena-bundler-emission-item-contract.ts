import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const publicApiSnapshot = read("rust/crates/omena-bundler/tests/snapshots/public-api.txt");
const parserFactsSource = read("rust/crates/omena-parser/src/facts/mod.rs");
const emissionSelectorFactsSource = read(
  "rust/crates/omena-parser/src/facts/emission_selectors.rs",
);
const parserExtensionSource = read("rust/crates/omena-parser/src/extension.rs");

const frozenBundlerFields = new Map<string, readonly string[]>([
  [
    "LinkerRuleV0",
    [
      "pub omena_bundler::LinkerRuleV0::range_end: u32",
      "pub omena_bundler::LinkerRuleV0::range_start: u32",
      "pub omena_bundler::LinkerRuleV0::selector_kind: omena_parser::facts::selectors::ParsedSelectorFactKind",
      "pub omena_bundler::LinkerRuleV0::selector_name: alloc::string::String",
    ],
  ],
  [
    "LinkerInputV0",
    [
      "pub omena_bundler::LinkerInputV0::class_names: alloc::vec::Vec<alloc::string::String>",
      "pub omena_bundler::LinkerInputV0::custom_property_names: alloc::vec::Vec<alloc::string::String>",
      "pub omena_bundler::LinkerInputV0::dependency_edges: alloc::vec::Vec<omena_bundler::LinkerDependencyEdgeV0>",
      "pub omena_bundler::LinkerInputV0::instance: omena_parser::closed_world::contract::ModuleInstanceKeyV0",
      "pub omena_bundler::LinkerInputV0::keyframe_names: alloc::vec::Vec<alloc::string::String>",
      "pub omena_bundler::LinkerInputV0::ordered_rules: alloc::vec::Vec<omena_bundler::LinkerRuleV0>",
      "pub omena_bundler::LinkerInputV0::source_path: alloc::string::String",
      "pub omena_bundler::LinkerInputV0::value_names: alloc::vec::Vec<alloc::string::String>",
    ],
  ],
  [
    "LinkedStylesheetRuleV0",
    [
      "pub omena_bundler::LinkedStylesheetRuleV0::global_order_index: u32",
      "pub omena_bundler::LinkedStylesheetRuleV0::module_instance: omena_parser::closed_world::contract::ModuleInstanceKeyV0",
      "pub omena_bundler::LinkedStylesheetRuleV0::range_end: u32",
      "pub omena_bundler::LinkedStylesheetRuleV0::range_start: u32",
      "pub omena_bundler::LinkedStylesheetRuleV0::selector_kind: &'static str",
      "pub omena_bundler::LinkedStylesheetRuleV0::selector_name: alloc::string::String",
    ],
  ],
  [
    "GlobalRuleOrderV0",
    [
      "pub omena_bundler::GlobalRuleOrderV0::rules: alloc::vec::Vec<omena_bundler::LinkedStylesheetRuleV0>",
    ],
  ],
  [
    "LinkedStylesheetV0",
    [
      "pub omena_bundler::LinkedStylesheetV0::closed_world_bundle: omena_parser::closed_world::contract::ClosedWorldBundleV0",
      "pub omena_bundler::LinkedStylesheetV0::emission_plan: omena_bundler::EmissionPlanV0",
      "pub omena_bundler::LinkedStylesheetV0::entrypoints: alloc::vec::Vec<omena_parser::closed_world::contract::ModuleInstanceKeyV0>",
      "pub omena_bundler::LinkedStylesheetV0::global_rule_order: omena_bundler::GlobalRuleOrderV0",
      "pub omena_bundler::LinkedStylesheetV0::module_instances: alloc::vec::Vec<omena_parser::closed_world::contract::ModuleInstanceKeyV0>",
      "pub omena_bundler::LinkedStylesheetV0::product: &'static str",
      "pub omena_bundler::LinkedStylesheetV0::schema_version: &'static str",
    ],
  ],
  [
    "LinkedEmissionArtifactV0",
    [
      "pub omena_bundler::LinkedEmissionArtifactV0::emitted_module_count: usize",
      "pub omena_bundler::LinkedEmissionArtifactV0::global_order_entry_count: usize",
      "pub omena_bundler::LinkedEmissionArtifactV0::module_regions: alloc::vec::Vec<omena_bundler::LinkedEmissionModuleRegionV0>",
      "pub omena_bundler::LinkedEmissionArtifactV0::order_entry_regions: alloc::vec::Vec<omena_bundler::LinkedEmissionOrderEntryRegionV0>",
      "pub omena_bundler::LinkedEmissionArtifactV0::output_css: alloc::string::String",
      "pub omena_bundler::LinkedEmissionArtifactV0::product: &'static str",
      "pub omena_bundler::LinkedEmissionArtifactV0::schema_version: &'static str",
    ],
  ],
  [
    "EmissionPlanV0",
    [
      "pub omena_bundler::EmissionPlanV0::cycle_groups: alloc::vec::Vec<omena_bundler::EmissionCycleGroupV0>",
      "pub omena_bundler::EmissionPlanV0::dependency_facts: alloc::vec::Vec<omena_bundler::EmissionDependencyFactV0>",
      "pub omena_bundler::EmissionPlanV0::entries: alloc::vec::Vec<omena_bundler::EmissionOrderKeyV0>",
      "pub omena_bundler::EmissionPlanV0::policy: omena_bundler::EmissionOrderingPolicyV0",
    ],
  ],
  [
    "EmissionOrderKeyV0",
    [
      "pub omena_bundler::EmissionOrderKeyV0::intra_module_ordinal: u32",
      "pub omena_bundler::EmissionOrderKeyV0::module_instance: omena_parser::closed_world::contract::ModuleInstanceKeyV0",
    ],
  ],
]);

const snapshotLines = publicApiSnapshot.split("\n");
for (const [typeName, expectedFields] of frozenBundlerFields) {
  const observedFields = snapshotLines.filter((line) =>
    line.startsWith(`pub omena_bundler::${typeName}::`),
  );
  assert.deepEqual(
    observedFields,
    expectedFields,
    `${typeName} field surface changed; carry new state in an additive non-exhaustive type`,
  );
}

const parsedStyleFactsBody = /pub struct ParsedStyleFacts\s*\{([\s\S]*?)\n\}/u.exec(
  parserFactsSource,
)?.[1];
assert.ok(parsedStyleFactsBody, "ParsedStyleFacts definition is missing");
const parsedStyleFactFields = parsedStyleFactsBody
  .split("\n")
  .map((line) => line.trim())
  .filter((line) => line.startsWith("pub "));
assert.deepEqual(
  parsedStyleFactFields,
  [
    "pub product: &'static str,",
    "pub dialect: StyleDialect,",
    "pub selector_count: usize,",
    "pub selectors: Vec<ParsedSelectorFact>,",
    "pub variable_count: usize,",
    "pub variables: Vec<ParsedVariableFact>,",
    "pub sass_symbol_count: usize,",
    "pub sass_symbols: Vec<ParsedSassSymbolFact>,",
    "pub sass_include_count: usize,",
    "pub sass_includes: Vec<ParsedSassIncludeFact>,",
    "pub sass_module_edge_count: usize,",
    "pub sass_module_edges: Vec<ParsedSassModuleEdgeFact>,",
    "pub sass_placeholder_definition_count: usize,",
    "pub sass_placeholder_definitions: Vec<ParsedSassPlaceholderDefinitionFact>,",
    "pub extend_target_count: usize,",
    "pub extend_targets: Vec<ParsedExtendTargetFact>,",
    "pub animation_count: usize,",
    "pub animations: Vec<ParsedAnimationFact>,",
    "pub css_module_value_count: usize,",
    "pub css_module_values: Vec<ParsedCssModuleValueFact>,",
    "pub css_module_value_import_edge_count: usize,",
    "pub css_module_value_import_edges: Vec<ParsedCssModuleValueImportEdgeFact>,",
    "pub css_module_value_definition_edge_count: usize,",
    "pub css_module_value_definition_edges: Vec<ParsedCssModuleValueDefinitionEdgeFact>,",
    "pub css_module_composes_count: usize,",
    "pub css_module_composes: Vec<ParsedCssModuleComposesFact>,",
    "pub css_module_composes_edge_count: usize,",
    "pub css_module_composes_edges: Vec<ParsedCssModuleComposesEdgeFact>,",
    "pub icss_count: usize,",
    "pub icss: Vec<ParsedIcssFact>,",
    "pub icss_import_edge_count: usize,",
    "pub icss_import_edges: Vec<ParsedIcssImportEdgeFact>,",
    "pub icss_export_edge_count: usize,",
    "pub icss_export_edges: Vec<ParsedIcssExportEdgeFact>,",
    "pub at_rule_count: usize,",
    "pub at_rules: Vec<ParsedAtRuleFact>,",
    "pub error_count: usize,",
  ],
  "ParsedStyleFacts field surface changed; carry new parser projections beside the frozen facts",
);

const emissionSelectorCarrierKinds = rustEnumVariants(
  emissionSelectorFactsSource,
  "ParsedEmissionSelectorFactKindV0",
);
assert.deepEqual(
  emissionSelectorCarrierKinds,
  ["Element", "Attribute", "Universal", "PseudoClass", "PseudoElement"],
  "the emission-selector carrier changed; update the bundler mapping and its corpus before accepting the new kind",
);

const cssAtRuleCarrierKinds = rustSyntaxKindsInFunction(
  parserExtensionSource,
  "pub(crate) fn at_rule_spec",
);
assert.deepEqual(
  cssAtRuleCarrierKinds,
  [
    "CharsetRule",
    "ColorProfileRule",
    "ContainerRule",
    "CounterStyleRule",
    "CustomMediaRule",
    "ElseRule",
    "FontFaceRule",
    "FontFeatureValuesAnnotationRule",
    "FontFeatureValuesCharacterVariantRule",
    "FontFeatureValuesHistoricalFormsRule",
    "FontFeatureValuesOrnamentsRule",
    "FontFeatureValuesRule",
    "FontFeatureValuesStylesetRule",
    "FontFeatureValuesStylisticRule",
    "FontFeatureValuesSwashRule",
    "FontPaletteValuesRule",
    "FunctionRule",
    "IfRule",
    "ImportRule",
    "KeyframesRule",
    "LayerRule",
    "MediaRule",
    "NamespaceRule",
    "NestRule",
    "PageMarginRule",
    "PageRule",
    "PositionTryRule",
    "PropertyRule",
    "ScopeRule",
    "StartingStyleRule",
    "SupportsRule",
    "ViewTransitionRule",
    "WhenRule",
  ],
  "the CSS at-rule carrier changed; update emission coverage before accepting the new syntax kind",
);

const scssAtRuleCarrierKinds = rustSyntaxKindsInFunction(
  parserExtensionSource,
  "pub(crate) fn scss_at_rule_spec",
);
assert.deepEqual(
  scssAtRuleCarrierKinds,
  [
    "ScssAtRootRule",
    "ScssContentRule",
    "ScssControlEach",
    "ScssControlElse",
    "ScssControlFor",
    "ScssControlIf",
    "ScssControlWhile",
    "ScssDebugRule",
    "ScssErrorRule",
    "ScssExtendRule",
    "ScssForwardRule",
    "ScssFunctionDeclaration",
    "ScssIncludeRule",
    "ScssMixinDeclaration",
    "ScssReturnRule",
    "ScssUseRule",
    "ScssWarnRule",
  ],
  "the SCSS at-rule carrier changed; update emission coverage before accepting the new syntax kind",
);

const emissionItemTests = runCargoTest([
  "test",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-bundler",
  "emission_item",
  "--",
  "--nocapture",
]);
for (const testName of [
  "emission_item_projection_parses_each_module_once",
  "emission_item_materializer_rejects_incomplete_module_coverage",
  "emission_items_for_empty_stylesheets_are_module_boundaries",
  "emission_items_for_supported_syntax_have_no_unknown_kinds",
  "emission_items_place_element_only_import_before_the_importer",
  "emission_items_do_not_relocate_an_element_rule_past_named_rules",
  "emission_item_placement_is_independent_of_module_names",
  "emission_items_preserve_cascade_layer_declaration_order",
  "emission_item_order_covers_non_class_rules_without_widening_legacy_order",
]) {
  assert.ok(emissionItemTests.includes(testName), `${testName} did not execute`);
}

const legacyByteTests = runCargoTest([
  "test",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-bundler",
  "--test",
  "linked_stylesheet_byte_identity",
  "--",
  "--nocapture",
]);
assert.ok(
  legacyByteTests.includes("linked_stylesheet_output_matches_committed_contract"),
  "the legacy linked-stylesheet byte contract did not execute",
);

const frozenFieldCount = [...frozenBundlerFields.values()].reduce(
  (total, fields) => total + fields.length,
  0,
);
process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-bundler.emission-item-contract",
      frozenBundlerTypeCount: frozenBundlerFields.size,
      frozenBundlerFieldCount: frozenFieldCount,
      parsedStyleFactFieldCount: parsedStyleFactFields.length,
      emissionSelectorCarrierKindCount: emissionSelectorCarrierKinds.length,
      atRuleCarrierKindCount: cssAtRuleCarrierKinds.length + scssAtRuleCarrierKinds.length,
      parserInvocationsPerFixtureModule: 1,
      regressionFixtureCount: 5,
      legacyByteIdentity: true,
    },
    null,
    2,
  )}\n`,
);

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function rustEnumVariants(source: string, enumName: string): readonly string[] {
  const body = new RegExp(`pub enum ${enumName}\\s*\\{([\\s\\S]*?)\\n\\}`, "u").exec(source)?.[1];
  assert.ok(body, `${enumName} definition is missing`);
  return body
    .split("\n")
    .map((line) => /^([A-Za-z][A-Za-z0-9_]*)\b/u.exec(line.trim())?.[1])
    .filter((variant): variant is string => variant !== undefined);
}

function rustSyntaxKindsInFunction(source: string, signature: string): readonly string[] {
  const signatureIndex = source.indexOf(signature);
  assert.notEqual(signatureIndex, -1, `${signature} is missing`);
  const bodyStart = source.indexOf("{", signatureIndex);
  assert.notEqual(bodyStart, -1, `${signature} has no body`);
  let depth = 0;
  let bodyEnd = -1;
  for (let index = bodyStart; index < source.length; index += 1) {
    if (source[index] === "{") {
      depth += 1;
    } else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        bodyEnd = index;
        break;
      }
    }
  }
  assert.notEqual(bodyEnd, -1, `${signature} body is unterminated`);
  return [
    ...new Set(
      [...source.slice(bodyStart + 1, bodyEnd).matchAll(/SyntaxKind::([A-Za-z0-9_]+)/gu)].map(
        (match) => match[1],
      ),
    ),
  ].toSorted();
}

function runCargoTest(args: readonly string[]): string {
  const run = spawnSync("cargo", args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  const output = `${run.stdout}${run.stderr}`;
  assert.equal(run.status, 0, output);
  const passedCount = [...output.matchAll(/test result: ok\. (\d+) passed;/gu)].reduce(
    (total, match) => total + Number(match[1]),
    0,
  );
  assert.ok(passedCount > 0, `cargo test filter matched no tests:\n${output}`);
  return output;
}

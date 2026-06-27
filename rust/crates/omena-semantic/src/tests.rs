use super::{
    CssModulesComposesEdgeFactV0, CssModulesCrossFileStyleFactsV0, CssModulesIcssExportEdgeFactV0,
    CssModulesIcssImportEdgeFactV0, CssModulesValueDefinitionEdgeFactV0,
    CssModulesValueImportEdgeFactV0, SassModuleForwardConfigurationRequestV0,
    SassModuleGraphConfigurationResolverV0, SassModuleGraphEdgeFactV0,
    SassModuleUseConfigurationRequestV0, StyleImportReachabilityEdgeFactV0,
    TheoryObservationHarnessInput, derive_sass_forward_effective_variable_overrides,
    derive_sass_forward_export_prefix_at_ordinal,
    derive_sass_module_forward_variable_overrides_at_ordinal,
    derive_sass_module_rule_variable_overrides_at_ordinal,
    filter_sass_forward_configurable_variable_names, parse_style_module,
    resolve_sass_module_effective_variable_overrides,
    sass_module_configuration_variables_are_valid, summarize_css_modules_cross_file_closure,
    summarize_lossless_cst_contract, summarize_omena_parser_style_semantic_boundary_from_source,
    summarize_parser_contract_facts, summarize_sass_module_configuration_signature,
    summarize_sass_module_graph_closure, summarize_sass_module_instance_identity_key,
    summarize_selector_identity_engine, summarize_semantic_promotion_evidence,
    summarize_semantic_promotion_evidence_with_source_input, summarize_source_input_evidence,
    summarize_style_import_reachability, summarize_style_semantic_boundary,
    summarize_style_semantic_facts, summarize_style_semantic_graph,
    summarize_style_semantic_graph_from_source, summarize_style_semantic_soa_tables,
    summarize_theory_observation_contract, summarize_theory_observation_harness,
};
use engine_input_producers::{
    ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn semantic_boundary_materializes_parser_once_per_analysis() {
    let (_, instrumentation) = omena_parser::with_omena_parser_parse_instrumentation(|| {
        summarize_omena_parser_style_semantic_boundary_from_source(
            "Component.module.scss",
            r#"
@use "./tokens";
.button {
  --brand: red;
  color: var(--brand);
}
"#,
        )
    });

    assert_eq!(instrumentation.parse_invocation_count, 1);
}

#[test]
fn semantic_graph_materializes_parser_once_per_analysis() {
    let (_, instrumentation) = omena_parser::with_omena_parser_parse_instrumentation(|| {
        let graph = summarize_style_semantic_graph_from_source(
            "Component.module.scss",
            r#"
@value primary: red;
.button {
  composes: base from "./base.module.scss";
  color: primary;
}
"#,
            &sample_engine_input(),
        );
        assert!(graph.is_some());
    });

    assert_eq!(instrumentation.parse_invocation_count, 1);
}

#[test]
fn css_modules_cross_file_closure_is_semantic_layer_owned() {
    let summary = summarize_css_modules_cross_file_closure(
        &[
            CssModulesCrossFileStyleFactsV0 {
                style_path: "/tmp/base.module.scss".to_string(),
                class_selector_names: vec!["foundation".to_string(), "base".to_string()],
                css_module_composes_edges: vec![CssModulesComposesEdgeFactV0 {
                    kind: "local",
                    owner_selector_names: vec!["base".to_string()],
                    target_names: vec!["foundation".to_string()],
                    import_source: None,
                }],
                css_module_value_definition_names: vec!["primary".to_string()],
                css_module_value_import_edges: Vec::new(),
                css_module_value_definition_edges: Vec::new(),
                icss_export_names: vec!["exported".to_string()],
                icss_import_edges: Vec::new(),
                icss_export_edges: Vec::new(),
            },
            CssModulesCrossFileStyleFactsV0 {
                style_path: "/tmp/app.module.scss".to_string(),
                class_selector_names: vec!["btn".to_string()],
                css_module_composes_edges: vec![CssModulesComposesEdgeFactV0 {
                    kind: "external",
                    owner_selector_names: vec!["btn".to_string()],
                    target_names: vec!["base".to_string()],
                    import_source: Some("./base.module.scss".to_string()),
                }],
                css_module_value_definition_names: vec!["accent".to_string()],
                css_module_value_import_edges: vec![CssModulesValueImportEdgeFactV0 {
                    remote_name: "primary".to_string(),
                    local_name: "accent".to_string(),
                    import_source: "./base.module.scss".to_string(),
                }],
                css_module_value_definition_edges: vec![CssModulesValueDefinitionEdgeFactV0 {
                    definition_name: "accent".to_string(),
                    reference_names: vec!["accent".to_string()],
                }],
                icss_export_names: vec!["forwarded".to_string()],
                icss_import_edges: vec![CssModulesIcssImportEdgeFactV0 {
                    local_name: "imported".to_string(),
                    remote_name: "exported".to_string(),
                    import_source: "./base.module.scss".to_string(),
                }],
                icss_export_edges: vec![CssModulesIcssExportEdgeFactV0 {
                    export_name: "forwarded".to_string(),
                    reference_names: vec!["imported".to_string()],
                }],
            },
        ],
        &[],
    );

    assert_eq!(
        summary.product,
        "omena-semantic.css-modules-cross-file-closure"
    );
    assert!(summary.capabilities.semantic_layer_owned);
    assert!(summary.capabilities.composes_closure_ready);
    assert!(summary.capabilities.value_graph_closure_ready);
    assert!(summary.capabilities.icss_export_import_closure_ready);
    assert!(summary.capabilities.cycle_detection_ready);
    assert!(summary.composes_closure_edges.iter().any(|edge| {
        edge.owner_selector_name == "btn"
            && edge.target_selector_name == "foundation"
            && edge.depth == 2
    }));
    assert!(summary.value_closure_edges.iter().any(|edge| {
        edge.value_name == "accent"
            && edge.target_style_path == "/tmp/base.module.scss"
            && edge.target_value_name == "primary"
    }));
    assert!(summary.icss_closure_edges.iter().any(|edge| {
        edge.name == "forwarded"
            && edge.target_style_path == "/tmp/base.module.scss"
            && edge.target_name == "exported"
    }));
}

#[derive(Debug, Default)]
struct TestSassGraphConfigurationResolver;

impl SassModuleGraphConfigurationResolverV0 for TestSassGraphConfigurationResolver {
    fn use_variable_overrides(
        &self,
        _request: SassModuleUseConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn forward_effective_variable_overrides(
        &self,
        request: SassModuleForwardConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String> {
        let mut overrides = request.inherited_variable_overrides.clone();
        if request.rule_ordinal == 0 && request.configurable_names.contains("brand") {
            overrides.insert("brand".to_string(), "red".to_string());
        }
        overrides
    }

    fn configurable_names(&self, target_style_path: &str) -> BTreeSet<String> {
        if target_style_path == "/tmp/tokens.scss" {
            BTreeSet::from(["brand".to_string()])
        } else {
            BTreeSet::new()
        }
    }
}

#[test]
fn sass_module_graph_closure_is_semantic_layer_owned() {
    let summary = summarize_sass_module_graph_closure(
        &[
            SassModuleGraphEdgeFactV0 {
                from_style_path: "/tmp/app.scss".to_string(),
                edge_kind: "sassUse",
                source: "./theme".to_string(),
                rule_ordinal: 0,
                namespace_kind: Some("named"),
                namespace: Some("theme".to_string()),
                forward_prefix: None,
                visibility_filter_kind: None,
                visibility_filter_names: Vec::new(),
                resolved_style_path: Some("/tmp/theme.scss".to_string()),
                status: "resolved",
                configuration_signature: "with:none".to_string(),
                configuration_variable_count: 0,
                invalid_configuration_variable_names: Vec::new(),
                module_instance_identity_key: Some("path:/tmp/theme.scss|with:none".to_string()),
            },
            SassModuleGraphEdgeFactV0 {
                from_style_path: "/tmp/theme.scss".to_string(),
                edge_kind: "sassForward",
                source: "./tokens".to_string(),
                rule_ordinal: 0,
                namespace_kind: None,
                namespace: None,
                forward_prefix: None,
                visibility_filter_kind: Some("show"),
                visibility_filter_names: vec!["brand".to_string()],
                resolved_style_path: Some("/tmp/tokens.scss".to_string()),
                status: "resolved",
                configuration_signature: "with|brand=red".to_string(),
                configuration_variable_count: 1,
                invalid_configuration_variable_names: Vec::new(),
                module_instance_identity_key: Some(
                    "path:/tmp/tokens.scss|with|brand=red".to_string(),
                ),
            },
            SassModuleGraphEdgeFactV0 {
                from_style_path: "/tmp/tokens.scss".to_string(),
                edge_kind: "sassUse",
                source: "./app".to_string(),
                rule_ordinal: 0,
                namespace_kind: None,
                namespace: None,
                forward_prefix: None,
                visibility_filter_kind: None,
                visibility_filter_names: Vec::new(),
                resolved_style_path: Some("/tmp/app.scss".to_string()),
                status: "resolved",
                configuration_signature: "with:none".to_string(),
                configuration_variable_count: 0,
                invalid_configuration_variable_names: Vec::new(),
                module_instance_identity_key: Some("path:/tmp/app.scss|with:none".to_string()),
            },
        ],
        &TestSassGraphConfigurationResolver,
    );

    assert_eq!(summary.product, "omena-semantic.sass-module-graph-closure");
    assert!(summary.capabilities.semantic_layer_owned);
    assert!(summary.capabilities.graph_closure_ready);
    assert!(summary.capabilities.cycle_detection_ready);
    assert!(summary.capabilities.namespace_show_hide_filter_ready);
    assert!(
        summary
            .capabilities
            .configured_module_instance_identity_ready
    );
    assert!(summary.graph_closure_edges.iter().any(|edge| {
        edge.from_style_path == "/tmp/app.scss"
            && edge.target_style_path == "/tmp/tokens.scss"
            && edge.depth == 2
            && edge.configuration_signature == "with|5:brand=3:red"
            && edge.module_instance_identity_key.as_deref()
                == Some("path:16:/tmp/tokens.scss|with|5:brand=3:red")
    }));
    assert!(summary.cycles.iter().any(|cycle| {
        cycle.path
            == vec![
                "/tmp/app.scss".to_string(),
                "/tmp/theme.scss".to_string(),
                "/tmp/tokens.scss".to_string(),
                "/tmp/app.scss".to_string(),
            ]
    }));
}

#[test]
fn sass_module_identity_key_is_semantic_layer_owned() {
    let overrides = BTreeMap::from([("brand".to_string(), "red".to_string())]);

    assert_eq!(
        summarize_sass_module_configuration_signature(&overrides),
        "with|5:brand=3:red"
    );
    assert_eq!(
        summarize_sass_module_instance_identity_key("/tmp/tokens.scss", &overrides),
        "path:16:/tmp/tokens.scss|with|5:brand=3:red"
    );
    assert_eq!(
        summarize_sass_module_configuration_signature(&BTreeMap::new()),
        "with:none"
    );
}

#[test]
fn sass_module_effective_overrides_are_semantic_layer_owned() {
    let brand_red = BTreeMap::from([("brand".to_string(), "red".to_string())]);
    let brand_blue = BTreeMap::from([("brand".to_string(), "blue".to_string())]);
    let mut loaded = BTreeMap::new();

    assert_eq!(
        resolve_sass_module_effective_variable_overrides(
            "/tmp/tokens.scss",
            &brand_red,
            &mut loaded,
        ),
        Some(brand_red.clone())
    );
    assert_eq!(
        resolve_sass_module_effective_variable_overrides(
            "/tmp/tokens.scss",
            &BTreeMap::new(),
            &mut loaded,
        ),
        Some(brand_red.clone()),
    );
    assert_eq!(
        resolve_sass_module_effective_variable_overrides(
            "/tmp/tokens.scss",
            &brand_blue,
            &mut loaded,
        ),
        None,
    );
    assert!(sass_module_configuration_variables_are_valid(
        &brand_red,
        &BTreeSet::from(["brand".to_string()]),
    ));
    assert!(!sass_module_configuration_variables_are_valid(
        &brand_red,
        &BTreeSet::new(),
    ));
}

#[test]
fn sass_module_rule_configuration_parsing_is_semantic_layer_owned() {
    let source = r#"
@use "./tokens" with ($brand_color: red, $space: 1rem);
@forward "./theme" as theme-* show $theme-brand, $theme-space with (
  $brand_color: blue !default,
  $space: 2rem
);
"#;

    assert_eq!(
        derive_sass_module_rule_variable_overrides_at_ordinal(source, "@use", 0),
        BTreeMap::from([
            ("brand-color".to_string(), "red".to_string()),
            ("space".to_string(), "1rem".to_string()),
        ]),
    );
    let forward_overrides = derive_sass_module_forward_variable_overrides_at_ordinal(source, 0);
    assert_eq!(
        forward_overrides
            .get("brand-color")
            .map(|entry| (entry.value.as_str(), entry.is_default)),
        Some(("blue", true)),
    );
    assert_eq!(
        forward_overrides
            .get("space")
            .map(|entry| (entry.value.as_str(), entry.is_default)),
        Some(("2rem", false)),
    );
    assert_eq!(
        derive_sass_forward_export_prefix_at_ordinal(source, 0).as_deref(),
        Some("theme-*"),
    );

    let inherited = BTreeMap::from([
        ("theme-brand".to_string(), "green".to_string()),
        ("theme-space".to_string(), "3rem".to_string()),
        ("theme-hidden".to_string(), "nope".to_string()),
    ]);
    let effective = derive_sass_forward_effective_variable_overrides(
        &forward_overrides,
        &inherited,
        Some("theme-*"),
        Some("show"),
        &["theme-brand".to_string(), "theme-space".to_string()],
        &BTreeSet::from([
            "brand".to_string(),
            "space".to_string(),
            "brand-color".to_string(),
        ]),
    );
    assert_eq!(
        effective,
        BTreeMap::from([
            ("brand".to_string(), "green".to_string()),
            ("brand-color".to_string(), "blue".to_string()),
            ("space".to_string(), "2rem".to_string()),
        ]),
    );
    assert_eq!(
        filter_sass_forward_configurable_variable_names(
            BTreeSet::from([
                "brand".to_string(),
                "space".to_string(),
                "hidden".to_string()
            ]),
            Some("theme-*"),
            Some("show"),
            &["theme-brand".to_string(), "theme-space".to_string()],
        ),
        BTreeSet::from(["theme-brand".to_string(), "theme-space".to_string()]),
    );
}

#[test]
fn style_import_reachability_is_semantic_layer_owned() {
    let summary = summarize_style_import_reachability(
        "/tmp/app.scss",
        &[
            StyleImportReachabilityEdgeFactV0 {
                from_style_path: "/tmp/app.scss".to_string(),
                target_style_path: "/tmp/theme.scss".to_string(),
            },
            StyleImportReachabilityEdgeFactV0 {
                from_style_path: "/tmp/theme.scss".to_string(),
                target_style_path: "/tmp/tokens.scss".to_string(),
            },
            StyleImportReachabilityEdgeFactV0 {
                from_style_path: "/tmp/tokens.scss".to_string(),
                target_style_path: "/tmp/unused.scss".to_string(),
            },
        ],
    );

    assert_eq!(summary.product, "omena-semantic.style-import-reachability");
    assert_eq!(summary.target_style_path, "/tmp/app.scss");
    assert_eq!(summary.edge_count, 3);
    assert!(summary.capabilities.semantic_layer_owned);
    assert!(summary.capabilities.transitive_reachability_ready);
    assert_eq!(
        summary
            .reachable_style_paths
            .iter()
            .map(|fact| (fact.style_path.as_str(), fact.distance, fact.order,))
            .collect::<Vec<_>>(),
        vec![
            ("/tmp/theme.scss", 1, 0),
            ("/tmp/tokens.scss", 2, 1),
            ("/tmp/unused.scss", 3, 2),
        ]
    );
}

#[test]
fn exposes_omena_parser_backed_semantic_boundary() {
    let summary = summarize_omena_parser_style_semantic_boundary_from_source(
        "Component.module.scss",
        r#"
@use "./tokens" as tokens;
$local: red;

@mixin tone($value) {
  color: $value;
}

.button {
  --brand: red;
  color: var(--brand);
  color: $local;
  @include tone(tokens.$accent);

  &__icon {
    animation: pulse 1s;
  }
}

@keyframes pulse {
  to { opacity: 1; }
}
"#,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.language, "scss");
    assert_eq!(
        summary.parser_facts.selectors.names,
        vec!["button".to_string(), "button__icon".to_string()]
    );
    assert_eq!(
        summary.parser_facts.custom_properties.decl_names,
        vec!["--brand".to_string()]
    );
    assert_eq!(
        summary.parser_facts.custom_properties.ref_names,
        vec!["--brand".to_string()]
    );
    assert_eq!(
        summary.semantic_facts.custom_properties.resolved_ref_names,
        vec!["--brand".to_string()]
    );
    assert_eq!(
        summary.parser_facts.sass.module_use_sources,
        vec!["./tokens".to_string()]
    );
    assert_eq!(
        summary.parser_facts.sass.mixin_include_names,
        vec!["tone".to_string()]
    );
    assert_eq!(
        summary.parser_facts.sass.variable_ref_names,
        vec![
            "accent".to_string(),
            "local".to_string(),
            "value".to_string()
        ]
    );
    assert_eq!(
        summary.parser_facts.keyframes.names,
        vec!["pulse".to_string()]
    );
    assert_eq!(summary.selector_identity_engine.canonical_id_count, 2);
    assert!(
        summary
            .lossless_cst_contract
            .span_invariants
            .byte_span_contract_ready
    );
}

#[test]
fn indexes_layer_container_and_scope_contexts_for_semantic_consumers() {
    let summary = summarize_omena_parser_style_semantic_boundary_from_source(
        "Component.module.css",
        r#"
@layer reset, components;
@layer components {
  @container card (inline-size > 40rem) {
    @scope (.card) to (.card__body) {
      .card { color: red; }
      .card__body { color: blue; }
    }
  }
}
"#,
    );
    let context_index = summary.semantic_facts.context_index;

    assert_eq!(context_index.product, "omena-semantic.style-context-index");
    assert!(
        context_index
            .ready_surfaces
            .contains(&"selectorContextMembership")
    );
    assert_eq!(
        context_index
            .layer_index
            .statement_layers
            .iter()
            .map(|layer| layer.name.as_str())
            .collect::<Vec<_>>(),
        vec!["reset", "components"]
    );
    assert_eq!(context_index.layer_index.named_layer_count, 2);
    assert_eq!(context_index.layer_index.block_layers.len(), 1);
    assert_eq!(
        context_index.layer_index.block_layers[0].name.as_deref(),
        Some("components")
    );
    assert_eq!(context_index.container_index.containers.len(), 1);
    assert_eq!(
        context_index.container_index.containers[0].name.as_deref(),
        Some("card")
    );
    assert_eq!(context_index.scope_index.scopes.len(), 1);
    assert_eq!(context_index.scope_index.scoped_selector_count, 2);
    assert!(
        context_index
            .scope_index
            .selector_memberships
            .iter()
            .any(|membership| membership.selector_name == "card")
    );
    assert!(
        context_index
            .container_index
            .selector_memberships
            .iter()
            .any(|membership| membership.selector_name == "card__body")
    );
}

#[test]
fn context_index_ignores_layer_tokens_inside_comments_strings_and_interpolation() {
    let summary = summarize_omena_parser_style_semantic_boundary_from_source(
        "Component.module.scss",
        r#"
/* @layer fakeComment; { */
.noise::before { content: "@layer fakeString; {"; }
.noise-#{"@layer fakeInterpolation; {"} { color: red; }

@layer reset, components;
@layer components {
  .card { content: "{"; color: red; }
  .card__body { color: blue; }
}
"#,
    );
    let context_index = summary.semantic_facts.context_index;

    assert_eq!(
        context_index
            .layer_index
            .statement_layers
            .iter()
            .map(|layer| layer.name.as_str())
            .collect::<Vec<_>>(),
        vec!["reset", "components"]
    );
    assert!(
        context_index
            .layer_index
            .statement_layers
            .iter()
            .all(|layer| {
                !matches!(
                    layer.name.as_str(),
                    "fakeComment" | "fakeString" | "fakeInterpolation"
                )
            })
    );
    assert_eq!(context_index.layer_index.block_layers.len(), 1);
    assert_eq!(
        context_index.layer_index.block_layers[0].name.as_deref(),
        Some("components")
    );
    assert!(
        context_index
            .layer_index
            .selector_memberships
            .iter()
            .any(|membership| membership.selector_name == "card")
    );
    assert!(
        context_index
            .layer_index
            .selector_memberships
            .iter()
            .any(|membership| membership.selector_name == "card__body")
    );
}

#[test]
fn exposes_semantic_soa_tables_with_typed_name_interners() {
    let boundary = summarize_omena_parser_style_semantic_boundary_from_source(
        "Component.module.scss",
        r#"
$local: red;

@mixin tone($value) {
  color: $value;
}

.button {
  --brand: red;
  color: var(--brand);
  color: $local;
  @include tone($local);

  &__icon {}
}
"#,
    );
    let db = salsa::DatabaseImpl::default();
    let tables = summarize_style_semantic_soa_tables(&boundary.semantic_facts, &db);

    assert_eq!(tables.schema_version, "0");
    assert_eq!(tables.product, "omena-semantic.soa-tables");
    assert!(tables.ready_surfaces.contains(&"semanticSoaTables"));
    assert!(tables.ready_surfaces.contains(&"semanticSoaNameTables"));
    assert_eq!(
        tables.selector_names.names,
        vec!["button".to_string(), "button__icon".to_string()]
    );
    assert_eq!(
        tables.custom_property_names.names,
        vec!["--brand".to_string()]
    );
    assert!(tables.sass_names.names.contains(&"local".to_string()));
    assert!(tables.sass_names.names.contains(&"tone".to_string()));
    assert_eq!(tables.total_row_count, tables.interned_row_count);
    assert_eq!(
        tables.total_row_count,
        tables.selector_names.row_indices.len()
            + tables.custom_property_names.row_indices.len()
            + tables.sass_names.row_indices.len()
    );
}

#[test]
fn keeps_omena_parser_nested_compound_selectors_rewrite_blocked() {
    let summary = summarize_omena_parser_style_semantic_boundary_from_source(
        "Component.module.scss",
        ".button { &.active { color: red; } }",
    );

    assert_eq!(
        summary.parser_facts.selectors.names,
        vec!["active".to_string(), "button".to_string()]
    );
    assert_eq!(
        summary.parser_facts.selectors.nested_unsafe_names,
        vec!["active".to_string()]
    );
    assert_eq!(
        summary
            .semantic_facts
            .selector_identity
            .nested_safety_counts
            .nested_unsafe,
        1
    );
    assert_eq!(
        summary
            .selector_identity_engine
            .rewrite_safety
            .blocked_canonical_ids,
        vec!["selector:active".to_string()]
    );
}

#[test]
fn exposes_semantic_summary_without_hiding_parser_contract_facts() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.scss",
        r#"
@use "./tokens" as tokens;
$local: red;

@mixin tone($value) {
  color: $value;
}

.button {
  color: $local;
  @include tone(tokens.$accent);

  &__icon {
    animation: pulse 1s;
  }
}

@keyframes pulse {
  from { opacity: 0; }
  to { opacity: 1; }
}
"#,
    )
    .ok_or_else(|| "SCSS module path should parse".to_string())?;

    let summary = summarize_style_semantic_boundary(&sheet);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.language, "scss");
    assert!(
        summary
            .parser_facts
            .lossless_cst
            .all_token_spans_within_source
    );
    assert!(
        summary
            .parser_facts
            .lossless_cst
            .all_node_spans_within_source
    );
    assert_eq!(
        summary.parser_facts.sass.module_use_sources,
        vec!["./tokens".to_string()]
    );
    assert_eq!(
        summary.semantic_facts.selector_identity.canonical_names,
        vec!["button".to_string(), "button__icon".to_string()]
    );
    assert_eq!(summary.selector_identity_engine.canonical_id_count, 2);
    assert_eq!(
        summary
            .selector_identity_engine
            .canonical_ids
            .iter()
            .map(|identity| identity.canonical_id.as_str())
            .collect::<Vec<_>>(),
        vec!["selector:button", "selector:button__icon"]
    );
    assert!(
        summary
            .selector_identity_engine
            .rewrite_safety
            .all_canonical_ids_rewrite_safe
    );
    assert_eq!(
        summary
            .semantic_facts
            .selector_identity
            .bem_suffix_safe_names,
        vec!["button__icon".to_string()]
    );
    assert_eq!(
        summary
            .semantic_facts
            .sass
            .selectors_with_resolved_variable_refs_names,
        vec!["button".to_string()]
    );
    assert_eq!(
        summary
            .semantic_facts
            .sass
            .selectors_with_resolved_mixin_includes_names,
        vec!["button".to_string()]
    );
    assert!(
        summary
            .lossless_cst_contract
            .span_invariants
            .byte_span_contract_ready
    );
    assert_eq!(
        summary.promotion_evidence.blocking_gaps,
        vec!["referenceSiteIdentity", "certaintyReason"]
    );
    Ok(())
}

#[test]
fn offers_narrow_semantic_and_parser_contract_accessors() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.scss",
        r#"
$color: red;

.button {
  color: $color;
}
"#,
    )
    .ok_or_else(|| "SCSS module path should parse".to_string())?;

    let parser_facts = summarize_parser_contract_facts(&sheet);
    let semantic_facts = summarize_style_semantic_facts(&sheet);

    assert_eq!(parser_facts.selectors.names, vec!["button".to_string()]);
    assert_eq!(
        parser_facts.sass.variable_decl_names,
        vec!["color".to_string()]
    );
    assert_eq!(
        semantic_facts
            .sass
            .selectors_with_resolved_variable_refs_names,
        vec!["button".to_string()]
    );
    assert!(
        semantic_facts
            .sass
            .selectors_with_unresolved_variable_refs_names
            .is_empty()
    );
    Ok(())
}

#[test]
fn exposes_selector_identity_as_dedicated_semantic_sub_engine() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.scss",
        r#"
.button {
  &__icon {}
  &.active {}
}
"#,
    )
    .ok_or_else(|| "SCSS module path should parse".to_string())?;

    let semantic_facts = summarize_style_semantic_facts(&sheet);
    let selector_identity = summarize_selector_identity_engine(&semantic_facts.selector_identity);

    assert_eq!(
        selector_identity.product,
        "omena-semantic.selector-identity"
    );
    assert_eq!(
        selector_identity
            .canonical_ids
            .iter()
            .map(|identity| {
                (
                    identity.canonical_id.as_str(),
                    identity.identity_kind,
                    identity.rewrite_safety,
                )
            })
            .collect::<Vec<_>>(),
        vec![
            ("selector:active", "localClass", "blocked"),
            ("selector:button", "localClass", "safe"),
            ("selector:button__icon", "bemSuffix", "safe")
        ]
    );
    assert_eq!(
        selector_identity.rewrite_safety.blocked_canonical_ids,
        vec!["selector:active".to_string()]
    );
    assert_eq!(
        selector_identity.rewrite_safety.blockers,
        vec!["nested-expansion"]
    );
    Ok(())
}

#[test]
fn exposes_promotion_evidence_gaps_without_hiding_ready_contracts() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.scss",
        r#"
@use "./tokens" as tokens;

.button {
  color: tokens.$accent;
}
"#,
    )
    .ok_or_else(|| "SCSS module path should parse".to_string())?;

    let parser_facts = summarize_parser_contract_facts(&sheet);
    let semantic_facts = summarize_style_semantic_facts(&sheet);
    let evidence = summarize_semantic_promotion_evidence(&parser_facts, &semantic_facts);

    assert_eq!(evidence.product, "omena-semantic.promotion-evidence");
    assert_eq!(
        evidence
            .items
            .iter()
            .find(|item| item.evidence == "selectorCanonicalId")
            .map(|item| item.status),
        Some("ready")
    );
    assert_eq!(
        evidence
            .items
            .iter()
            .find(|item| item.evidence == "sourceSpan")
            .map(|item| item.status),
        Some("ready")
    );
    assert_eq!(
        evidence
            .items
            .iter()
            .find(|item| item.evidence == "referenceSiteIdentity")
            .map(|item| item.status),
        Some("gap")
    );
    assert_eq!(
        evidence.next_priorities,
        vec!["referenceSiteIdentity", "certaintyReason", "bindingOrigin"]
    );
    Ok(())
}

#[test]
fn exposes_design_token_seed_promotion_evidence() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.css",
        r#"
:root {
  --color-gray-700: #767678;
}

.button {
  color: var(--color-gray-700);
  border-color: var(--missing);
}
"#,
    )
    .ok_or_else(|| "CSS module path should parse".to_string())?;

    let parser_facts = summarize_parser_contract_facts(&sheet);
    let semantic_facts = summarize_style_semantic_facts(&sheet);
    let evidence = summarize_semantic_promotion_evidence(&parser_facts, &semantic_facts);
    let design_token_seed = evidence
        .items
        .iter()
        .find(|item| item.evidence == "designTokenSeed")
        .ok_or_else(|| "expected design token seed evidence".to_string())?;

    assert_eq!(design_token_seed.status, "ready");
    assert_eq!(
        design_token_seed.provider,
        "ParserIndexCustomPropertyFactsV0"
    );
    assert_eq!(design_token_seed.observed_count, 3);
    Ok(())
}

#[test]
fn exposes_design_token_semantic_readiness_surface() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.css",
        r#"
:root {
  --color-gray-700: #767678;
}

@media (min-width: 600px) {
  .button {
    color: var(--color-gray-700);
  }
}

.ghost {
  border-color: var(--missing);
}
"#,
    )
    .ok_or_else(|| "CSS module path should parse".to_string())?;

    let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

    assert_eq!(summary.product, "omena-semantic.design-token-semantics");
    assert_eq!(summary.status, "context-aware-resolution-seed");
    assert_eq!(summary.resolution_scope, "same-file");
    assert_eq!(summary.declaration_count, 1);
    assert_eq!(summary.reference_count, 2);
    assert_eq!(summary.resolved_reference_count, 1);
    assert_eq!(summary.unresolved_reference_count, 1);
    assert_eq!(summary.selectors_with_references_count, 2);
    assert_eq!(summary.context_signal.declaration_context_selector_count, 1);
    assert_eq!(summary.context_signal.declaration_wrapper_context_count, 0);
    assert_eq!(summary.context_signal.media_context_selector_count, 1);
    assert_eq!(summary.context_signal.wrapper_context_count, 1);
    assert_eq!(summary.resolution_signal.declaration_fact_count, 1);
    assert_eq!(summary.resolution_signal.reference_fact_count, 2);
    assert_eq!(
        summary.resolution_signal.source_ordered_declaration_count,
        1
    );
    assert_eq!(summary.resolution_signal.source_ordered_reference_count, 2);
    assert_eq!(
        summary
            .resolution_signal
            .occurrence_resolved_reference_count,
        1
    );
    assert_eq!(
        summary
            .resolution_signal
            .occurrence_unresolved_reference_count,
        1
    );
    assert_eq!(summary.resolution_signal.root_declaration_count, 1);
    assert_eq!(
        summary.resolution_signal.selector_scoped_declaration_count,
        0
    );
    assert_eq!(
        summary.resolution_signal.wrapper_scoped_declaration_count,
        0
    );
    assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 1);
    assert_eq!(summary.cascade_ranking_signal.unranked_reference_count, 1);
    assert_eq!(
        summary
            .cascade_ranking_signal
            .source_order_winner_declaration_count,
        1
    );
    assert_eq!(
        summary
            .cascade_ranking_signal
            .source_order_shadowed_declaration_count,
        0
    );
    assert_eq!(
        summary
            .cascade_ranking_signal
            .repeated_name_declaration_count,
        0
    );
    assert!(summary.capabilities.same_file_resolution_ready);
    assert!(summary.capabilities.wrapper_context_signal_ready);
    assert!(summary.capabilities.source_order_signal_ready);
    assert!(summary.capabilities.source_order_cascade_ranking_ready);
    assert!(summary.capabilities.occurrence_resolution_signal_ready);
    assert!(summary.capabilities.selector_context_resolution_ready);
    assert!(summary.capabilities.theme_override_context_signal_ready);
    assert!(!summary.capabilities.cross_package_cascade_ranking_ready);
    assert!(!summary.capabilities.theme_override_context_ready);
    assert_eq!(
        summary.blocking_gaps,
        vec![
            "crossFileImportGraph",
            "crossPackageCascadeRanking",
            "themeOverrideContext",
            "unresolvedDesignTokenRefs"
        ]
    );
    assert_eq!(
        summary.next_priorities,
        vec![
            "crossFileImportGraph",
            "crossPackageCascadeRanking",
            "themeOverrideContext"
        ]
    );
    Ok(())
}

#[test]
fn exposes_design_token_occurrence_context_resolution_signal() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.css",
        r#"
:root {
  --surface: white;
}

.theme {
  --brand: #222;
}

.button {
  color: var(--brand);
  background: var(--surface);
}

.theme .button {
  border-color: var(--brand);
}
"#,
    )
    .ok_or_else(|| "CSS module path should parse".to_string())?;

    let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

    assert_eq!(summary.resolved_reference_count, 2);
    assert_eq!(summary.unresolved_reference_count, 1);
    assert_eq!(summary.resolution_signal.declaration_fact_count, 2);
    assert_eq!(summary.resolution_signal.reference_fact_count, 3);
    assert_eq!(
        summary.resolution_signal.source_ordered_declaration_count,
        2
    );
    assert_eq!(summary.resolution_signal.source_ordered_reference_count, 3);
    assert_eq!(
        summary
            .resolution_signal
            .occurrence_resolved_reference_count,
        2
    );
    assert_eq!(
        summary
            .resolution_signal
            .occurrence_unresolved_reference_count,
        1
    );
    assert_eq!(summary.resolution_signal.context_matched_reference_count, 2);
    assert_eq!(
        summary.resolution_signal.context_unmatched_reference_count,
        1
    );
    assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 2);
    assert_eq!(summary.cascade_ranking_signal.unranked_reference_count, 1);
    assert_eq!(
        summary
            .cascade_ranking_signal
            .source_order_winner_declaration_count,
        2
    );
    assert_eq!(
        summary
            .cascade_ranking_signal
            .source_order_shadowed_declaration_count,
        0
    );
    assert_eq!(summary.resolution_signal.root_declaration_count, 1);
    assert_eq!(
        summary.resolution_signal.selector_scoped_declaration_count,
        1
    );
    assert!(summary.capabilities.occurrence_resolution_signal_ready);
    assert!(summary.capabilities.source_order_signal_ready);
    assert!(summary.capabilities.selector_context_resolution_ready);
    Ok(())
}

#[test]
fn exposes_design_token_source_order_cascade_ranking_signal() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.css",
        r#"
:root {
  --surface: white;
}

:root {
  --surface: black;
}

.theme {
  --surface: gray;
}

.button {
  color: var(--surface);
}

.theme .button {
  background: var(--surface);
}
"#,
    )
    .ok_or_else(|| "CSS module path should parse".to_string())?;

    let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

    assert_eq!(summary.status, "same-file-cascade-ranking-seed");
    assert_eq!(summary.declaration_count, 1);
    assert_eq!(summary.reference_count, 1);
    assert_eq!(summary.resolved_reference_count, 1);
    assert_eq!(summary.unresolved_reference_count, 0);
    assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 2);
    assert_eq!(summary.cascade_ranking_signal.unranked_reference_count, 0);
    assert_eq!(
        summary
            .cascade_ranking_signal
            .source_order_winner_declaration_count,
        2
    );
    assert_eq!(
        summary
            .cascade_ranking_signal
            .source_order_shadowed_declaration_count,
        2
    );
    assert_eq!(
        summary
            .cascade_ranking_signal
            .repeated_name_declaration_count,
        3
    );
    assert_eq!(summary.cascade_ranking_signal.ranked_references.len(), 2);
    let first_ranked_reference = &summary.cascade_ranking_signal.ranked_references[0];
    assert_eq!(first_ranked_reference.reference_name, "--surface");
    assert_eq!(first_ranked_reference.reference_source_order, 0);
    assert_eq!(first_ranked_reference.winner_declaration_source_order, 1);
    assert_eq!(
        first_ranked_reference.shadowed_declaration_source_orders,
        vec![0]
    );
    assert_eq!(first_ranked_reference.candidate_declaration_count, 2);
    let second_ranked_reference = &summary.cascade_ranking_signal.ranked_references[1];
    assert_eq!(second_ranked_reference.reference_name, "--surface");
    assert_eq!(second_ranked_reference.reference_source_order, 1);
    assert_eq!(second_ranked_reference.winner_declaration_source_order, 2);
    assert_eq!(
        second_ranked_reference.shadowed_declaration_source_orders,
        vec![0, 1]
    );
    assert_eq!(second_ranked_reference.candidate_declaration_count, 3);
    assert!(summary.capabilities.source_order_cascade_ranking_ready);
    assert!(!summary.capabilities.cross_package_cascade_ranking_ready);
    Ok(())
}

#[test]
fn ranks_design_tokens_with_exact_conditional_context() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.css",
        r#":root { --surface: base; }
@media (min-width: 40rem) {
  :root { --surface: wide; }
  .button { color: var(--surface); }
}
@media (max-width: 20rem) {
  :root { --surface: narrow; }
}
"#,
    )
    .ok_or_else(|| "CSS module path should parse".to_string())?;

    let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

    assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 1);
    let ranked_reference = &summary.cascade_ranking_signal.ranked_references[0];
    assert_eq!(ranked_reference.reference_name, "--surface");
    assert_eq!(ranked_reference.winner_declaration_source_order, 1);
    assert_eq!(ranked_reference.shadowed_declaration_source_orders, vec![0]);
    assert_eq!(ranked_reference.candidate_declaration_count, 2);
    Ok(())
}

#[test]
fn ranks_theme_context_declarations_ahead_of_later_root_tokens() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.css",
        r#"
:root {
  --surface: white;
}

[data-theme="dark"] {
  --surface: black;
}

:root {
  --surface: beige;
}

[data-theme="dark"] .button {
  color: var(--surface);
}
"#,
    )
    .ok_or_else(|| "CSS module path should parse".to_string())?;

    let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

    assert_eq!(summary.status, "same-file-cascade-ranking-seed");
    assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 1);
    assert_eq!(
        summary
            .cascade_ranking_signal
            .theme_context_winner_reference_count,
        1
    );
    let ranked_reference = &summary.cascade_ranking_signal.ranked_references[0];
    assert_eq!(ranked_reference.reference_name, "--surface");
    assert_eq!(ranked_reference.winner_declaration_source_order, 1);
    assert_eq!(
        ranked_reference.shadowed_declaration_source_orders,
        vec![0, 2]
    );
    assert_eq!(ranked_reference.winner_context_kind, "selector");
    assert!(summary.capabilities.theme_override_context_ready);
    assert!(!summary.blocking_gaps.contains(&"themeOverrideContext"));
    assert!(!summary.next_priorities.contains(&"themeOverrideContext"));
    Ok(())
}

#[test]
fn ranks_unlayered_design_tokens_above_later_layered_tokens() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.css",
        r#"
.button {
  --surface: unlayered;
}

@layer components {
  .button {
    --surface: layered;
    color: var(--surface);
  }
}
"#,
    )
    .ok_or_else(|| "CSS module path should parse".to_string())?;

    let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

    assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 1);
    let ranked_reference = &summary.cascade_ranking_signal.ranked_references[0];
    assert_eq!(ranked_reference.reference_name, "--surface");
    assert_eq!(ranked_reference.winner_declaration_source_order, 0);
    assert_eq!(ranked_reference.winner_declaration_layer_rank, 2);
    assert_eq!(ranked_reference.winner_declaration_layer_name, None);
    assert_eq!(ranked_reference.shadowed_declaration_source_orders, vec![1]);
    Ok(())
}

#[test]
fn ranks_named_layer_order_above_later_layer_source_order() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.css",
        r#"
@layer reset, components;

@layer components {
  .button {
    --surface: components;
  }
}

@layer reset {
  .button {
    --surface: reset;
    color: var(--surface);
  }
}
"#,
    )
    .ok_or_else(|| "CSS module path should parse".to_string())?;

    let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

    assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 1);
    let ranked_reference = &summary.cascade_ranking_signal.ranked_references[0];
    assert_eq!(ranked_reference.reference_name, "--surface");
    assert_eq!(ranked_reference.winner_declaration_source_order, 0);
    assert_eq!(ranked_reference.winner_declaration_layer_rank, 1);
    assert_eq!(
        ranked_reference.winner_declaration_layer_name.as_deref(),
        Some("components")
    );
    assert_eq!(ranked_reference.shadowed_declaration_source_orders, vec![1]);
    Ok(())
}

#[test]
fn exposes_lossless_cst_contract_for_precise_consumers() -> Result<(), String> {
    let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
        .ok_or_else(|| "SCSS module path should parse".to_string())?;

    let parser_facts = summarize_parser_contract_facts(&sheet);
    let contract = summarize_lossless_cst_contract(&parser_facts.lossless_cst);

    assert_eq!(contract.product, "omena-semantic.lossless-cst-contract");
    assert!(contract.span_invariants.byte_span_contract_ready);
    assert!(contract.consumer_readiness.precise_rename_base_ready);
    assert!(contract.consumer_readiness.formatter_base_ready);
    assert!(!contract.consumer_readiness.recovery_diagnostics_observed);
    Ok(())
}

#[test]
fn exposes_source_input_evidence_for_reference_identity_and_certainty_reasons() {
    let evidence = summarize_source_input_evidence(&sample_engine_input());

    assert_eq!(evidence.product, "omena-semantic.source-input-evidence");
    assert_eq!(evidence.reference_site_identity.status, "ready");
    assert_eq!(evidence.reference_site_identity.reference_site_count, 2);
    assert_eq!(
        evidence.reference_site_identity.direct_reference_site_count,
        1
    );
    assert_eq!(
        evidence
            .reference_site_identity
            .expanded_reference_site_count,
        1
    );
    assert_eq!(
        evidence.reference_site_identity.editable_direct_site_count,
        1
    );
    assert_eq!(evidence.certainty_reason.status, "ready");
    assert_eq!(evidence.certainty_reason.expression_count, 2);
    assert_eq!(evidence.certainty_reason.exact_count, 1);
    assert_eq!(evidence.certainty_reason.inferred_count, 1);
    assert_eq!(evidence.binding_origin.status, "ready");
    assert_eq!(evidence.binding_origin.expression_count, 2);
    assert_eq!(evidence.binding_origin.direct_class_name_count, 1);
    assert_eq!(evidence.binding_origin.root_binding_count, 1);
    assert_eq!(
        evidence
            .binding_origin
            .expression_kind_counts
            .get("literal"),
        Some(&1)
    );
    assert_eq!(evidence.style_module_edge.status, "ready");
    assert_eq!(evidence.style_module_edge.source_style_edge_count, 2);
    assert_eq!(evidence.style_module_edge.distinct_style_module_count, 1);
    assert_eq!(
        evidence.style_module_edge.missing_style_document_edge_count,
        0
    );
    assert_eq!(evidence.value_domain_explanation.status, "ready");
    assert_eq!(evidence.value_domain_explanation.expression_count, 2);
    assert_eq!(evidence.value_domain_explanation.exact_expression_count, 1);
    assert_eq!(
        evidence
            .value_domain_explanation
            .constrained_expression_count,
        1
    );
    assert_eq!(evidence.value_domain_explanation.finite_value_count, 1);
    assert_eq!(evidence.value_domain_explanation.derivation_count, 2);
    assert_eq!(evidence.value_domain_explanation.derivation_step_count, 2);
    assert_eq!(
        evidence
            .value_domain_explanation
            .derivation_product_counts
            .get("omena-abstract-value.reduced-class-value-derivation"),
        Some(&2)
    );
    assert_eq!(
        evidence
            .value_domain_explanation
            .derivation_reduced_kind_counts
            .get("exact"),
        Some(&1)
    );
    assert_eq!(
        evidence
            .value_domain_explanation
            .derivation_reduced_kind_counts
            .get("prefix"),
        Some(&1)
    );
    assert_eq!(
        evidence
            .value_domain_explanation
            .derivation_operation_counts
            .get("baseFromFacts"),
        Some(&2)
    );
    assert_eq!(
        evidence
            .certainty_reason
            .reason_counts
            .get("single selector matched"),
        Some(&1)
    );
    assert_eq!(
        evidence
            .certainty_reason
            .reason_counts
            .get("constrained runtime shape matched a bounded selector set"),
        Some(&1)
    );
}

#[test]
fn source_input_evidence_upgrades_promotion_evidence_gaps() -> Result<(), String> {
    let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
        .ok_or_else(|| "SCSS module path should parse".to_string())?;
    let parser_facts = summarize_parser_contract_facts(&sheet);
    let semantic_facts = summarize_style_semantic_facts(&sheet);
    let evidence = summarize_semantic_promotion_evidence_with_source_input(
        &parser_facts,
        &semantic_facts,
        &sample_engine_input(),
    );

    assert_eq!(
        evidence
            .items
            .iter()
            .find(|item| item.evidence == "referenceSiteIdentity")
            .map(|item| item.status),
        Some("ready")
    );
    assert_eq!(
        evidence
            .items
            .iter()
            .find(|item| item.evidence == "bindingOrigin")
            .map(|item| item.status),
        Some("ready")
    );
    assert_eq!(
        evidence
            .items
            .iter()
            .find(|item| item.evidence == "styleModuleEdge")
            .map(|item| item.status),
        Some("ready")
    );
    assert_eq!(
        evidence
            .items
            .iter()
            .find(|item| item.evidence == "valueDomainExplanation")
            .map(|item| item.status),
        Some("ready")
    );
    assert_eq!(
        evidence
            .items
            .iter()
            .find(|item| item.evidence == "certaintyReason")
            .map(|item| item.status),
        Some("ready")
    );
    assert!(evidence.blocking_gaps.is_empty());
    assert!(evidence.next_priorities.is_empty());
    Ok(())
}

#[test]
fn exposes_style_semantic_graph_with_source_backed_promotion_evidence() -> Result<(), String> {
    let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
        .ok_or_else(|| "SCSS module path should parse".to_string())?;
    let graph = summarize_style_semantic_graph(&sheet, &sample_engine_input());

    assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
    assert_eq!(graph.language, "scss");
    assert_eq!(
        graph.selector_reference_engine.product,
        "omena-semantic.selector-references"
    );
    assert_eq!(graph.selector_reference_engine.selector_count, 2);
    assert_eq!(graph.selector_reference_engine.referenced_selector_count, 2);
    assert_eq!(graph.selector_reference_engine.total_reference_sites, 2);
    assert_eq!(graph.source_input_evidence.binding_origin.status, "ready");
    assert_eq!(
        graph
            .promotion_evidence
            .items
            .iter()
            .filter(|item| item.status == "gap")
            .count(),
        0
    );
    assert!(graph.promotion_evidence.blocking_gaps.is_empty());
    assert!(
        graph
            .lossless_cst_contract
            .span_invariants
            .byte_span_contract_ready
    );
    Ok(())
}

#[test]
fn summarizes_style_semantic_graph_from_source_for_host_consumers() -> Result<(), String> {
    let graph = summarize_style_semantic_graph_from_source(
        "/tmp/Component.module.scss",
        ".button { color: red; }",
        &sample_engine_input(),
    )
    .ok_or_else(|| "expected style semantic graph".to_string())?;

    assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
    assert_eq!(graph.language, "scss");
    assert_eq!(
        graph.selector_identity_engine.product,
        "omena-semantic.selector-identity"
    );
    assert_eq!(
        graph.selector_reference_engine.style_path,
        Some("/tmp/Component.module.scss".to_string())
    );
    assert_eq!(graph.selector_reference_engine.selector_count, 2);

    Ok(())
}

#[test]
fn style_semantic_graph_includes_css_modules_parser_fact_seed() -> Result<(), String> {
    let graph = summarize_style_semantic_graph_from_source(
            "/tmp/Component.module.scss",
            "@value primary: #fff; @value accent: primary; @value secondary as localSecondary from \"./tokens.module.scss\"; :export { primary: #fff; forwarded: imported; } :import(\"./tokens.css\") { imported: primary; } @keyframes fade { to { opacity: 1; } } .card { composes: base utility from \"./base.module.scss\"; animation: fade 1s; }",
            &sample_engine_input(),
        )
        .ok_or_else(|| "expected style semantic graph".to_string())?;

    let css_modules = graph.css_modules_semantics;
    assert_eq!(css_modules.product, "omena-semantic.css-modules-semantics");
    assert_eq!(css_modules.status, "parserFactSeed");
    assert_eq!(css_modules.resolution_scope, "perFileFactSummary");
    assert_eq!(css_modules.class_export_names, vec!["card"]);
    assert_eq!(css_modules.composes_edge_seed_count, 1);
    assert_eq!(css_modules.composes_external_edge_count, 1);
    assert_eq!(css_modules.composes_local_edge_count, 0);
    assert_eq!(css_modules.composes_global_edge_count, 0);
    assert_eq!(css_modules.composes_target_names, vec!["base", "utility"]);
    assert_eq!(
        css_modules.composes_import_sources,
        vec!["./base.module.scss"]
    );
    assert_eq!(
        css_modules.value_definition_names,
        vec!["accent", "localSecondary", "primary"]
    );
    assert_eq!(
        css_modules.value_reference_names,
        vec!["primary", "secondary"]
    );
    assert_eq!(
        css_modules.value_import_sources,
        vec!["./tokens.module.scss"]
    );
    assert_eq!(css_modules.value_import_edge_count, 1);
    assert_eq!(css_modules.value_definition_edge_count, 1);
    assert_eq!(css_modules.value_edge_seed_count, 2);
    assert_eq!(css_modules.icss_export_names, vec!["forwarded", "primary"]);
    assert_eq!(css_modules.icss_import_local_names, vec!["imported"]);
    assert_eq!(css_modules.icss_import_remote_names, vec!["primary"]);
    assert_eq!(css_modules.icss_import_sources, vec!["./tokens.css"]);
    assert_eq!(css_modules.icss_import_edge_count, 1);
    assert_eq!(css_modules.icss_export_edge_count, 1);
    assert_eq!(css_modules.icss_edge_seed_count, 2);
    assert_eq!(css_modules.keyframe_names, vec!["fade"]);
    assert_eq!(css_modules.animation_reference_names, vec!["fade"]);
    assert!(css_modules.capabilities.parser_fact_surface_ready);
    assert!(css_modules.capabilities.per_file_symbol_summary_ready);
    assert!(!css_modules.capabilities.cross_file_resolution_ready);
    assert!(!css_modules.capabilities.composes_closure_ready);

    Ok(())
}

#[test]
fn theory_observation_harness_reports_ready_semantic_graph() -> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.scss",
        ".button { &__icon { color: red; } }",
    )
    .ok_or_else(|| "SCSS module path should parse".to_string())?;
    let graph = summarize_style_semantic_graph(&sheet, &sample_engine_input());
    let observation = summarize_theory_observation_harness(&graph);

    assert_eq!(
        observation.product,
        "omena-semantic.theory-observation-harness"
    );
    assert_eq!(
        observation.graph_product,
        "omena-semantic.style-semantic-graph"
    );
    assert_eq!(observation.selector_identity.status, "ready");
    assert_eq!(observation.selector_identity.observed_selector_count, 2);
    assert_eq!(
        observation.selector_identity.rewrite_blocked_selector_count,
        0
    );
    assert!(observation.selector_identity.rename_safe);
    assert_eq!(observation.source_evidence.status, "ready");
    assert_eq!(observation.source_evidence.reference_site_count, 2);
    assert_eq!(
        observation
            .source_evidence
            .certainty_reason_counts
            .get("single selector matched"),
        Some(&1)
    );
    assert_eq!(observation.downstream_readiness.status, "ready");
    assert!(observation.downstream_readiness.downstream_check_ready);
    assert!(observation.downstream_readiness.precise_rename_ready);
    assert_eq!(observation.coupling_boundary.generic_observation_count, 4);
    assert_eq!(
        observation.coupling_boundary.cme_coupled_observation_count,
        2
    );
    assert_eq!(
        observation.coupling_boundary.split_recommendation,
        "keep-integrated-observe-boundary"
    );
    assert!(observation.blocking_gaps.is_empty());
    assert_eq!(
        observation.next_priorities,
        vec!["externalCorpus", "traitDogfooding"]
    );

    let contract = summarize_theory_observation_contract(&graph);
    assert_eq!(
        contract.product,
        "omena-semantic.theory-observation-contract"
    );
    assert_eq!(
        contract.observation_product,
        "omena-semantic.theory-observation-harness"
    );
    assert!(contract.ready);
    assert!(contract.publish_ready);
    assert_eq!(contract.selector_identity_status, "ready");
    assert_eq!(contract.source_evidence_status, "ready");
    assert_eq!(contract.downstream_readiness_status, "ready");
    assert!(contract.blocking_gaps.is_empty());
    assert!(contract.publish_blocking_gaps.is_empty());
    assert!(contract.observation_gaps.is_empty());
    assert_eq!(contract, graph.summarize_theory_observation_contract());
    Ok(())
}

#[test]
fn theory_observation_harness_marks_rewrite_blockers_without_hiding_graph_readiness()
-> Result<(), String> {
    let sheet = parse_style_module(
        "Component.module.scss",
        r#"
.button {
  &.active {}
}
"#,
    )
    .ok_or_else(|| "SCSS module path should parse".to_string())?;
    let graph = summarize_style_semantic_graph(&sheet, &sample_engine_input());
    let observation = summarize_theory_observation_harness(&graph);

    assert_eq!(observation.selector_identity.status, "partial");
    assert_eq!(
        observation.selector_identity.rewrite_blocked_selector_count,
        1
    );
    assert_eq!(
        observation.selector_identity.blockers,
        vec!["nested-expansion"]
    );
    assert!(observation.downstream_readiness.downstream_check_ready);
    assert!(!observation.downstream_readiness.precise_rename_ready);
    assert_eq!(observation.downstream_readiness.status, "partial");
    assert_eq!(
        observation.blocking_gaps,
        vec!["selectorRewriteSafety", "downstreamReadiness"]
    );

    let contract = graph.summarize_theory_observation_contract();
    assert!(!contract.ready);
    assert!(!contract.publish_ready);
    assert_eq!(
        contract.blocking_gaps,
        vec!["selectorRewriteSafety", "downstreamReadiness"]
    );
    assert_eq!(
        contract.publish_blocking_gaps,
        vec!["selectorRewriteSafety"]
    );
    assert_eq!(contract.observation_gaps, vec!["downstreamReadiness"]);
    Ok(())
}

#[test]
fn theory_observation_harness_exposes_cme_coupling_gaps() -> Result<(), String> {
    let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
        .ok_or_else(|| "SCSS module path should parse".to_string())?;
    let graph = summarize_style_semantic_graph(&sheet, &empty_engine_input());
    let observation = summarize_theory_observation_harness(&graph);

    assert_eq!(observation.selector_identity.status, "ready");
    assert_eq!(observation.source_evidence.status, "gap");
    assert_eq!(observation.source_evidence.reference_site_count, 0);
    assert_eq!(
        observation
            .source_evidence
            .explainable_certainty_reason_count,
        0
    );
    assert_eq!(observation.downstream_readiness.status, "gap");
    assert_eq!(
        observation.blocking_gaps,
        vec!["sourceEvidence", "downstreamReadiness"]
    );
    assert_eq!(
        observation.coupling_boundary.generic_surfaces,
        vec![
            "parserSemanticFacts",
            "designTokenSemantics",
            "selectorIdentity",
            "losslessCstContract"
        ]
    );
    assert_eq!(
        observation.coupling_boundary.cme_coupled_surfaces,
        vec!["sourceInputEvidence", "promotionEvidenceWithSourceInput"]
    );
    let contract = graph.summarize_theory_observation_contract();
    assert!(!contract.ready);
    assert!(contract.publish_ready);
    assert!(contract.publish_blocking_gaps.is_empty());
    assert_eq!(
        contract.observation_gaps,
        vec!["sourceEvidence", "downstreamReadiness"]
    );
    Ok(())
}

fn sample_engine_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![
                    ClassExpressionInputV2 {
                        id: "expr-literal".to_string(),
                        kind: "literal".to_string(),
                        scss_module_path: "/tmp/Component.module.scss".to_string(),
                        range: range(4, 12, 4, 18),
                        class_name: Some("button".to_string()),
                        root_binding_decl_id: None,
                        access_path: None,
                    },
                    ClassExpressionInputV2 {
                        id: "expr-prefix".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/Component.module.scss".to_string(),
                        range: range(5, 12, 5, 24),
                        class_name: None,
                        root_binding_decl_id: Some("decl-prefix".to_string()),
                        access_path: None,
                    },
                ],
            },
        }],
        styles: vec![StyleAnalysisInputV2 {
            file_path: "/tmp/Component.module.scss".to_string(),
            source: None,
            document: StyleDocumentV2 {
                selectors: vec![
                    StyleSelectorV2 {
                        name: "button".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("button".to_string()),
                        range: range(0, 1, 0, 7),
                        nested_safety: Some("flat".to_string()),
                        composes: None,
                        bem_suffix: None,
                    },
                    StyleSelectorV2 {
                        name: "button--primary".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("button--primary".to_string()),
                        range: range(1, 1, 1, 16),
                        nested_safety: Some("flat".to_string()),
                        composes: None,
                        bem_suffix: None,
                    },
                ],
            },
        }],
        type_facts: vec![
            TypeFactEntryV2 {
                file_path: "/tmp/Component.tsx".to_string(),
                expression_id: "expr-literal".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "exact".to_string(),
                    constraint_kind: None,
                    values: Some(vec!["button".to_string()]),
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: None,
                },
                control_flow_graph: None,
            },
            TypeFactEntryV2 {
                file_path: "/tmp/Component.tsx".to_string(),
                expression_id: "expr-prefix".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "constrained".to_string(),
                    constraint_kind: Some("prefix".to_string()),
                    values: None,
                    prefix: Some("button--".to_string()),
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: None,
                },
                control_flow_graph: None,
            },
        ],
    }
}

fn empty_engine_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: Vec::new(),
    }
}

fn range(
    start_line: usize,
    start_character: usize,
    end_line: usize,
    end_character: usize,
) -> RangeV2 {
    RangeV2 {
        start: PositionV2 {
            line: start_line,
            character: start_character,
        },
        end: PositionV2 {
            line: end_line,
            character: end_character,
        },
    }
}

#[test]
fn parser_position_uses_utf16_columns_for_non_ascii() {
    // Regression for the byte-offset-vs-UTF-16 column divergence (same bug class
    // as closed issue #40, tsgo position encoding): a multi-byte character before
    // the offset must count as its UTF-16 width, not its UTF-8 byte length.
    // "한글" = 2 chars, 3 UTF-8 bytes each (6 bytes), 1 UTF-16 code unit each.
    let pos = super::parser_position_for_byte_offset("한글x", 6);
    assert_eq!(pos.line, 0);
    assert_eq!(pos.character, 2, "two UTF-16 units, not six bytes");

    // Astral scalar U+1F600 is 4 UTF-8 bytes and 2 UTF-16 code units (surrogate pair).
    let astral = super::parser_position_for_byte_offset("😀y", 4);
    assert_eq!(astral.character, 2);

    // Column resets after a newline: 'b' sits one UTF-16 unit into line 1.
    let multiline = super::parser_position_for_byte_offset("a\n가b", 5);
    assert_eq!(multiline.line, 1);
    assert_eq!(multiline.character, 1);
}

use super::{
    AbstractClassValueProvenanceNodeV0, AbstractClassValueProvenanceV0, AbstractClassValueV0,
    AbstractPropertyValueCandidateV0, AbstractPropertyValueV0, ClassValueControlFlowBlockV0,
    ClassValueControlFlowGraphV0, ClassValueFlowGraphV0, ClassValueFlowNodeV0,
    ClassValueFlowTransferV0, CompositeClassValueInputV0, ExternalStringTypeFactsV0,
    KLimitedCallSiteFlowInputV0, MAX_FINITE_CLASS_VALUES, OneCfaCallSiteFlowInputV0,
    SelectorProjectionCertaintyV0, abstract_class_value_from_facts, abstract_class_value_is_subset,
    analyze_class_value_control_flow_graph, analyze_class_value_flow,
    analyze_class_value_flow_incremental, analyze_class_value_flow_incremental_batch_with_reuse,
    analyze_class_value_flow_incremental_with_database,
    analyze_class_value_flow_incremental_with_reuse, analyze_k_limited_call_site_flows,
    analyze_one_cfa_call_site_flows, bottom_class_value, char_inclusion_class_value,
    composite_class_value, concatenate_abstract_class_values,
    concatenate_reduced_class_value_products, derive_selector_projection_certainty,
    exact_class_value, finite_set_class_value, finite_values_from_facts,
    intersect_abstract_class_values, intersect_reduced_class_value_products,
    join_abstract_class_values, join_reduced_class_value_products,
    narrow_abstract_property_value_for_pseudo_state, prefix_class_value, prefix_suffix_class_value,
    project_abstract_value_selectors, reduce_class_value_product,
    reduced_abstract_class_value_from_facts, reduced_class_value_derivation_from_facts,
    reduced_class_value_product_is_subset, reduced_class_value_product_matches_string,
    reduced_value_domain_kind_from_facts, selector_certainty_from_facts,
    selector_certainty_shape_kind_from_facts, selector_certainty_shape_label_from_facts,
    suffix_class_value, summarize_abstract_class_value_provenance_tree,
    summarize_omena_abstract_value_domain, summarize_omena_abstract_value_flow_analysis,
    summarize_reduced_class_value_product, top_class_value, value_certainty_from_facts,
    value_certainty_shape_kind_from_facts, value_certainty_shape_label_from_facts,
};
use omena_incremental::OmenaIncrementalDatabaseV0;
use std::collections::BTreeMap;

#[test]
fn summarizes_domain_boundary_contract() {
    let summary = summarize_omena_abstract_value_domain();

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-abstract-value.domain");
    assert_eq!(summary.max_finite_class_values, MAX_FINITE_CLASS_VALUES);
    assert!(summary.domain_kinds.contains(&"exact"));
    assert!(summary.domain_kinds.contains(&"composite"));
    assert!(summary.reduced_product_structure_ready);
    assert_eq!(
        summary.reduced_product_axes,
        vec!["prefix", "suffix", "charInclusion", "lengthLowerBound"]
    );
    assert!(
        summary
            .reduced_product_operations
            .contains(&"matchesString")
    );
    assert!(
        summary
            .reduced_product_consumers
            .contains(&"semanticReachability")
    );
    assert!(
        summary
            .selector_projection_certainties
            .contains(&"inferred")
    );
    assert!(summary.provenance_tree_ready);
    assert!(summary.provenance_tree_scopes.contains(&"reducedProduct"));
    assert!(summary.provenance_tree_scopes.contains(&"flowResult"));

    let flow_summary = summarize_omena_abstract_value_flow_analysis();
    assert_eq!(flow_summary.schema_version, "0");
    assert_eq!(flow_summary.product, "omena-abstract-value.flow-analysis");
    assert_eq!(flow_summary.context_sensitivity, "1-cfa");
    assert_eq!(flow_summary.incremental_engine, "omena-incremental");
    assert!(flow_summary.analysis_scopes.contains(&"multiContextBatch"));
    assert!(flow_summary.analysis_scopes.contains(&"callSiteBatch"));
    assert!(
        flow_summary
            .analysis_scopes
            .contains(&"kLimitedCallSiteBatch")
    );
    assert!(flow_summary.analysis_scopes.contains(&"controlFlowGraph"));
    assert_eq!(
        flow_summary.reuse_policy,
        "reuse previous context analysis when its omena-incremental plan is clean"
    );
    assert!(flow_summary.transfer_kinds.contains(&"join"));
    assert!(flow_summary.transfer_kinds.contains(&"concatFacts"));
}

#[test]
fn normalizes_finite_sets_to_bottom_exact_or_sorted_unique_values() {
    assert_eq!(
        finite_set_class_value(Vec::<String>::new()),
        AbstractClassValueV0::Bottom
    );
    assert_eq!(
        finite_set_class_value(["button"]),
        exact_class_value("button")
    );
    assert_eq!(
        finite_set_class_value(["card", "button", "card"]),
        AbstractClassValueV0::FiniteSet {
            values: vec!["button".to_string(), "card".to_string()]
        }
    );
}

#[test]
fn maps_external_string_facts_to_stable_value_certainty_labels() {
    let exact = external_facts("exact").with_values(["button"]);
    assert_eq!(
        abstract_class_value_from_facts(&exact),
        exact_class_value("button")
    );
    assert_eq!(value_certainty_from_facts(&exact), Some("exact"));
    assert_eq!(value_certainty_shape_kind_from_facts(&exact), "exact");
    assert_eq!(value_certainty_shape_label_from_facts(&exact), "exact");
    assert_eq!(
        finite_values_from_facts(&exact),
        Some(vec!["button".to_string()])
    );

    let finite = external_facts("finiteSet").with_values(["card", "button", "card"]);
    assert_eq!(value_certainty_from_facts(&finite), Some("inferred"));
    assert_eq!(
        value_certainty_shape_kind_from_facts(&finite),
        "boundedFinite"
    );
    assert_eq!(
        value_certainty_shape_label_from_facts(&finite),
        "bounded finite (2)"
    );
    assert_eq!(selector_certainty_from_facts(&finite, 1, 3), "inferred");
    assert_eq!(
        selector_certainty_shape_label_from_facts(&finite, 1, 3),
        "bounded selector set (1)"
    );
}

#[test]
fn maps_constrained_external_string_facts_to_stable_shape_labels() {
    let edge = external_facts("constrained")
        .with_constraint_kind("prefixSuffix")
        .with_prefix("btn-")
        .with_suffix("-active")
        .with_min_len(11);

    assert_eq!(
        abstract_class_value_from_facts(&edge),
        AbstractClassValueV0::PrefixSuffix {
            prefix: "btn-".to_string(),
            suffix: "-active".to_string(),
            min_length: 11,
            provenance: None,
        }
    );
    assert_eq!(value_certainty_from_facts(&edge), Some("inferred"));
    assert_eq!(value_certainty_shape_kind_from_facts(&edge), "constrained");
    assert_eq!(
        value_certainty_shape_label_from_facts(&edge),
        "constrained prefix `btn-` + suffix `-active`"
    );
    assert_eq!(selector_certainty_from_facts(&edge, 1, 3), "inferred");
    assert_eq!(selector_certainty_from_facts(&edge, 3, 3), "inferred");
    assert_eq!(
        selector_certainty_from_facts(&external_facts("top"), 3, 3),
        "possible"
    );
    assert_eq!(
        selector_certainty_shape_kind_from_facts(&edge, 1, 3),
        "constrained"
    );
    assert_eq!(
        selector_certainty_shape_label_from_facts(&edge, 1, 3),
        "constrained edge selector set (1)"
    );
}

#[test]
fn narrows_property_values_to_single_stylesheet_pseudo_state() {
    let candidates = vec![
        property_candidate("color", "black", None),
        property_candidate("color", "white", Some("hover")),
        property_candidate("background", "var(--surface)", Some("hover")),
    ];

    let narrowed =
        narrow_abstract_property_value_for_pseudo_state("color", Some("hover"), &candidates);

    assert_eq!(narrowed.schema_version, "0");
    assert_eq!(
        narrowed.product,
        "omena-abstract-value.property-value-narrowing"
    );
    assert_eq!(narrowed.stylesheet_scope, "singleStylesheet");
    assert_eq!(narrowed.candidate_count, 3);
    assert_eq!(narrowed.matched_candidate_count, 2);
    assert_eq!(
        narrowed.value,
        AbstractPropertyValueV0::FiniteSet {
            property_name: "color".to_string(),
            values: vec!["black".to_string(), "white".to_string()],
            pseudo_states: vec!["hover".to_string()],
        }
    );
}

#[test]
fn narrows_property_values_to_custom_property_reference_annotation_target() {
    let candidates = vec![property_candidate(
        "background",
        "var(--surface)",
        Some("focus"),
    )];

    let narrowed =
        narrow_abstract_property_value_for_pseudo_state("background", Some("focus"), &candidates);

    assert_eq!(
        narrowed.value,
        AbstractPropertyValueV0::CustomPropertyReference {
            property_name: "background".to_string(),
            custom_property_name: "--surface".to_string(),
            pseudo_state: Some("focus".to_string()),
        }
    );
}

#[test]
fn widens_large_finite_sets_to_composite_when_edges_survive() {
    let values = (0..=MAX_FINITE_CLASS_VALUES)
        .map(|index| format!("btn-{index}-active"))
        .collect::<Vec<_>>();

    let value = finite_set_class_value(values);

    assert_eq!(
        value,
        AbstractClassValueV0::Composite {
            prefix: Some("btn-".to_string()),
            suffix: Some("-active".to_string()),
            min_length: Some("btn-0-active".len()),
            must_chars: "-abceintv".to_string(),
            may_chars: "-012345678abceintv".to_string(),
            may_include_other_chars: false,
            provenance: Some(AbstractClassValueProvenanceV0::FiniteSetWideningComposite),
        }
    );
}

#[test]
fn builds_char_inclusion_and_composite_values_with_normalized_chars() {
    assert_eq!(
        char_inclusion_class_value(
            "ba",
            "cad",
            Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars),
            false,
        ),
        AbstractClassValueV0::CharInclusion {
            must_chars: "ab".to_string(),
            may_chars: "abcd".to_string(),
            may_include_other_chars: false,
            provenance: Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars),
        }
    );

    assert_eq!(
        composite_class_value(CompositeClassValueInputV0 {
            prefix: Some("btn-".to_string()),
            suffix: Some("-active".to_string()),
            min_length: None,
            must_chars: "z".to_string(),
            may_chars: "za".to_string(),
            may_include_other_chars: true,
            provenance: None,
        }),
        AbstractClassValueV0::Composite {
            prefix: Some("btn-".to_string()),
            suffix: Some("-active".to_string()),
            min_length: Some("btn-z-active".len()),
            must_chars: "-abceintvz".to_string(),
            may_chars: "-abceintvz".to_string(),
            may_include_other_chars: true,
            provenance: None,
        }
    );
}

#[test]
fn intersects_finite_values_with_constrained_domains() {
    let finite = finite_set_class_value(["btn-primary", "card", "btn-secondary"]);
    let prefix = prefix_class_value("btn-", None);

    assert_eq!(
        intersect_abstract_class_values(&finite, &prefix),
        AbstractClassValueV0::FiniteSet {
            values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
        }
    );

    assert_eq!(
        intersect_abstract_class_values(
            &exact_class_value("card"),
            &prefix_class_value("btn-", None),
        ),
        AbstractClassValueV0::Bottom
    );
}

#[test]
fn intersects_prefix_suffix_and_char_constraints_into_reduced_product() {
    let edge = intersect_abstract_class_values(
        &prefix_class_value("btn-", None),
        &suffix_class_value("-active", None),
    );

    assert_eq!(
        edge,
        AbstractClassValueV0::PrefixSuffix {
            prefix: "btn-".to_string(),
            suffix: "-active".to_string(),
            min_length: "btn-active".len(),
            provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
        }
    );

    let reduced = intersect_abstract_class_values(
        &edge,
        &char_inclusion_class_value("ab", "-abceintv", None, false),
    );

    assert_eq!(
        reduced,
        AbstractClassValueV0::Composite {
            prefix: Some("btn-".to_string()),
            suffix: Some("-active".to_string()),
            min_length: Some("btn-active".len()),
            must_chars: "-abceintv".to_string(),
            may_chars: "-abceintv".to_string(),
            may_include_other_chars: false,
            provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
        }
    );

    let reduced_with_extra_required_char = intersect_abstract_class_values(
        &edge,
        &char_inclusion_class_value("z", "-abceintvz", None, false),
    );

    assert_eq!(
        reduced_with_extra_required_char,
        AbstractClassValueV0::Composite {
            prefix: Some("btn-".to_string()),
            suffix: Some("-active".to_string()),
            min_length: Some("btn-z-active".len()),
            must_chars: "-abceintvz".to_string(),
            may_chars: "-abceintvz".to_string(),
            may_include_other_chars: false,
            provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
        }
    );
}

#[test]
fn preserves_overlapping_prefix_suffix_reduced_product_targets() {
    let edge = intersect_abstract_class_values(
        &prefix_class_value("btn-primary", None),
        &suffix_class_value("primary", None),
    );

    assert_eq!(
        edge,
        AbstractClassValueV0::PrefixSuffix {
            prefix: "btn-primary".to_string(),
            suffix: "primary".to_string(),
            min_length: "btn-primary".len(),
            provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
        }
    );

    let selectors = selector_universe(["btn-primary", "btn-secondary", "card-primary"]);
    assert_eq!(
        projected_names(&edge, &selectors),
        vec!["btn-primary".to_string()]
    );
}

#[test]
fn rejects_incompatible_reduced_product_constraints() {
    assert_eq!(
        intersect_abstract_class_values(
            &prefix_class_value("btn-", None),
            &prefix_class_value("card-", None),
        ),
        AbstractClassValueV0::Bottom
    );

    assert_eq!(
        intersect_abstract_class_values(
            &prefix_class_value("btn-", None),
            &char_inclusion_class_value("", "abc", None, false),
        ),
        AbstractClassValueV0::Bottom
    );
}

#[test]
fn treats_closed_empty_character_domain_as_bottom() {
    assert_eq!(
        char_inclusion_class_value("", "", None, false),
        AbstractClassValueV0::Bottom
    );
    assert_eq!(
        intersect_abstract_class_values(
            &prefix_class_value("btn-", None),
            &char_inclusion_class_value("", "", None, false),
        ),
        AbstractClassValueV0::Bottom
    );
}

#[test]
fn reduced_product_laws_hold_over_selector_projection() {
    let selectors = selector_universe([
        "btn-primary",
        "btn-secondary",
        "btn-active",
        "card",
        "card-active",
        "nav-active",
    ]);
    let finite = finite_set_class_value([
        "btn-primary",
        "btn-secondary",
        "card",
        "card-active",
        "missing",
    ]);
    let prefix = prefix_class_value("btn-", None);
    let suffix = suffix_class_value("-active", None);
    let chars = char_inclusion_class_value("ab", "-abceintv", None, false);
    let composite = composite_class_value(CompositeClassValueInputV0 {
        prefix: Some("btn-".to_string()),
        suffix: Some("-active".to_string()),
        min_length: Some("btn-active".len()),
        must_chars: "ab".to_string(),
        may_chars: "-abceintv".to_string(),
        may_include_other_chars: false,
        provenance: None,
    });

    for (left, right) in [
        (&finite, &prefix),
        (&prefix, &suffix),
        (&suffix, &chars),
        (&prefix, &composite),
    ] {
        assert_projection_equivalent(
            &intersect_abstract_class_values(left, right),
            &intersect_abstract_class_values(right, left),
            &selectors,
        );
    }

    for value in [&finite, &prefix, &suffix, &chars, &composite] {
        assert_projection_equivalent(
            &intersect_abstract_class_values(value, value),
            value,
            &selectors,
        );
    }

    assert_eq!(
        intersect_abstract_class_values(&top_class_value(), &finite),
        finite
    );
    assert_eq!(
        intersect_abstract_class_values(&finite, &top_class_value()),
        finite
    );
    assert_eq!(
        intersect_abstract_class_values(&bottom_class_value(), &finite),
        bottom_class_value()
    );
    assert_eq!(
        intersect_abstract_class_values(&finite, &bottom_class_value()),
        bottom_class_value()
    );
}

#[test]
fn exposes_reduced_product_subset_relation_for_composite_domains() {
    let composite = composite_class_value(CompositeClassValueInputV0 {
        prefix: Some("btn-".to_string()),
        suffix: Some("-active".to_string()),
        min_length: Some("btn-active".len()),
        must_chars: "ab".to_string(),
        may_chars: "-abceintv".to_string(),
        may_include_other_chars: false,
        provenance: None,
    });
    let prefix = prefix_class_value("btn-", None);
    let suffix = suffix_class_value("-active", None);
    let chars = char_inclusion_class_value("ab", "-abceintv", None, false);
    let looser_chars = char_inclusion_class_value("a", "-abceintvz", None, false);
    let incompatible_chars = char_inclusion_class_value("z", "-abceintvz", None, false);

    assert!(abstract_class_value_is_subset(&composite, &prefix));
    assert!(abstract_class_value_is_subset(&composite, &suffix));
    assert!(abstract_class_value_is_subset(&composite, &chars));
    assert!(abstract_class_value_is_subset(&composite, &looser_chars));
    assert!(!abstract_class_value_is_subset(
        &composite,
        &incompatible_chars
    ));
    assert!(!abstract_class_value_is_subset(&prefix, &composite));
}

#[test]
fn exposes_reduced_product_as_explicit_axes() {
    let composite = composite_class_value(CompositeClassValueInputV0 {
        prefix: Some("btn-".to_string()),
        suffix: Some("-active".to_string()),
        min_length: Some("btn-active".len()),
        must_chars: "ab".to_string(),
        may_chars: "-abceintv".to_string(),
        may_include_other_chars: false,
        provenance: None,
    });

    let product = summarize_reduced_class_value_product(&composite);

    assert!(product.is_some());
    assert_eq!(
        product.as_ref().map(|summary| summary.schema_version),
        Some("0")
    );
    assert_eq!(
        product.as_ref().map(|summary| summary.product),
        Some("omena-abstract-value.reduced-product")
    );
    assert_eq!(
        product.as_ref().map(|summary| summary.source_value_kind),
        Some("composite")
    );
    assert_eq!(
        product
            .as_ref()
            .and_then(|summary| summary.prefix.as_ref())
            .map(|axis| axis.prefix.as_str()),
        Some("btn-")
    );
    assert_eq!(
        product
            .as_ref()
            .and_then(|summary| summary.suffix.as_ref())
            .map(|axis| axis.suffix.as_str()),
        Some("-active")
    );
    assert_eq!(
        product
            .as_ref()
            .map(|summary| summary.char_inclusion.must_chars.as_str()),
        Some("-abceintv")
    );
    assert_eq!(
        product
            .as_ref()
            .and_then(|summary| summary.char_inclusion.allowed_chars.as_deref()),
        Some("-abceintv")
    );
    assert_eq!(
        product
            .as_ref()
            .map(|summary| summary.char_inclusion.may_include_other_chars),
        Some(false)
    );
    assert_eq!(
        product.as_ref().and_then(|summary| summary.min_length),
        Some("btn-active".len())
    );
    assert_eq!(
        product.as_ref().map(|summary| summary.lower_bound_length),
        Some("btn-active".len())
    );
}

#[test]
fn exposes_reduced_product_domain_algebra() -> Result<(), &'static str> {
    let prefix =
        reduce_class_value_product(&prefix_class_value("btn-", None)).ok_or("prefix product")?;
    let suffix =
        reduce_class_value_product(&suffix_class_value("-active", None)).ok_or("suffix product")?;
    let chars =
        reduce_class_value_product(&char_inclusion_class_value("ab", "-abceintv", None, false))
            .ok_or("char-inclusion product")?;

    let edge = intersect_reduced_class_value_products(&prefix, &suffix)
        .ok_or("prefix-suffix intersection")?;
    let constrained = intersect_reduced_class_value_products(&edge, &chars)
        .ok_or("reduced-product intersection")?;

    assert_eq!(constrained.prefix.as_deref(), Some("btn-"));
    assert_eq!(constrained.suffix.as_deref(), Some("-active"));
    assert_eq!(constrained.allowed_chars.as_deref(), Some("-abceintv"));
    assert!(reduced_class_value_product_matches_string(
        &constrained,
        "btn-active"
    ));
    assert!(!reduced_class_value_product_matches_string(
        &constrained,
        "btn-secondary"
    ));
    assert!(reduced_class_value_product_is_subset(&constrained, &prefix));

    let primary_prefix = reduce_class_value_product(&prefix_class_value("btn-primary-", None))
        .ok_or("primary prefix product")?;
    let secondary_prefix = reduce_class_value_product(&prefix_class_value("btn-secondary-", None))
        .ok_or("secondary prefix product")?;
    let joined = join_reduced_class_value_products(&primary_prefix, &secondary_prefix)
        .ok_or("reduced-product join")?;
    assert_eq!(joined.prefix.as_deref(), Some("btn-"));

    let concatenated = concatenate_reduced_class_value_products(&prefix, &suffix)
        .ok_or("reduced-product concatenation")?;
    assert_eq!(concatenated.prefix.as_deref(), Some("btn-"));
    assert_eq!(concatenated.suffix.as_deref(), Some("-active"));
    Ok(())
}

#[test]
fn reduced_product_projection_matches_intersected_projection_sets() {
    let selectors = selector_universe([
        "btn-primary",
        "btn-secondary",
        "btn-active",
        "card",
        "card-active",
        "nav-active",
    ]);
    let finite = finite_set_class_value([
        "btn-primary",
        "btn-secondary",
        "card",
        "card-active",
        "missing",
    ]);
    let prefix = prefix_class_value("btn-", None);
    let suffix = suffix_class_value("-active", None);
    let prefix_suffix = intersect_abstract_class_values(&prefix, &suffix);

    assert_eq!(
        projected_names(
            &intersect_abstract_class_values(&finite, &prefix),
            &selectors
        ),
        vec!["btn-primary".to_string(), "btn-secondary".to_string()]
    );
    assert_eq!(
        projected_names(
            &intersect_abstract_class_values(&finite, &prefix),
            &selectors
        ),
        intersect_projected_names(&finite, &prefix, &selectors)
    );
    assert_eq!(
        projected_names(
            &intersect_abstract_class_values(&finite, &prefix_suffix),
            &selectors,
        ),
        intersect_projected_names(&finite, &prefix_suffix, &selectors)
    );
}

#[test]
fn joins_abstract_values_for_branch_merges() {
    assert_eq!(
        join_abstract_class_values(
            &exact_class_value("btn-primary"),
            &exact_class_value("btn-secondary"),
        ),
        AbstractClassValueV0::FiniteSet {
            values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
        }
    );

    assert_eq!(
        join_abstract_class_values(
            &prefix_class_value("btn-primary-", None),
            &prefix_class_value("btn-secondary-", None),
        ),
        prefix_class_value("btn-", Some(AbstractClassValueProvenanceV0::PrefixJoinLcp))
    );

    assert_eq!(
        join_abstract_class_values(
            &prefix_class_value("btn-", None),
            &exact_class_value("btn-primary"),
        ),
        prefix_class_value("btn-", None)
    );
}

#[test]
fn joins_reduced_product_constraints_for_branch_merges() {
    let primary = prefix_suffix_class_value(
        "btn-primary-",
        "-active",
        Some("btn-primary--active".len()),
        None,
    );
    let secondary = prefix_suffix_class_value(
        "btn-secondary-",
        "-active",
        Some("btn-secondary--active".len()),
        None,
    );

    let joined = join_abstract_class_values(&primary, &secondary);

    assert_eq!(
        joined,
        AbstractClassValueV0::Composite {
            prefix: Some("btn-".to_string()),
            suffix: Some("-active".to_string()),
            min_length: Some("btn-primary--active".len()),
            must_chars: "-abceinrtvy".to_string(),
            may_chars: "-abceinrtvy".to_string(),
            may_include_other_chars: true,
            provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
        }
    );
    assert_eq!(
        projected_names(
            &joined,
            &selector_universe([
                "btn--active",
                "btn-primary--active",
                "btn-secondary--active",
                "card-active",
            ]),
        ),
        vec![
            "btn-primary--active".to_string(),
            "btn-secondary--active".to_string(),
        ]
    );
}

#[test]
fn flow_join_preserves_reduced_product_branch_shape() {
    let graph = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@variant-active".to_string()),
        nodes: vec![
            flow_assign_node(
                "primary",
                external_facts("constrained")
                    .with_constraint_kind("prefixSuffix")
                    .with_prefix("btn-primary-")
                    .with_suffix("-active")
                    .with_min_len("btn-primary--active".len()),
            ),
            flow_assign_node(
                "secondary",
                external_facts("constrained")
                    .with_constraint_kind("prefixSuffix")
                    .with_prefix("btn-secondary-")
                    .with_suffix("-active")
                    .with_min_len("btn-secondary--active".len()),
            ),
            ClassValueFlowNodeV0 {
                id: "merge".to_string(),
                predecessors: vec!["primary".to_string(), "secondary".to_string()],
                transfer: ClassValueFlowTransferV0::Join,
            },
        ],
    };

    let analysis = analyze_class_value_flow(&graph);

    assert_eq!(
        flow_value(&analysis, "merge"),
        Some(&AbstractClassValueV0::Composite {
            prefix: Some("btn-".to_string()),
            suffix: Some("-active".to_string()),
            min_length: Some("btn-primary--active".len()),
            must_chars: "-abceinrtvy".to_string(),
            may_chars: "-abceinrtvy".to_string(),
            may_include_other_chars: true,
            provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
        })
    );
}

#[test]
fn concatenates_abstract_values_for_template_edges() {
    assert_eq!(
        concatenate_abstract_class_values(
            &exact_class_value("btn-"),
            &finite_set_class_value(["primary", "secondary"]),
        ),
        AbstractClassValueV0::FiniteSet {
            values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
        }
    );

    assert_eq!(
        concatenate_abstract_class_values(
            &prefix_class_value("btn-", None),
            &suffix_class_value("-active", None),
        ),
        prefix_suffix_class_value("btn-", "-active", Some("btn--active".len()), None)
    );

    assert_eq!(
        concatenate_abstract_class_values(
            &finite_set_class_value(["card-primary", "card-secondary"]),
            &prefix_class_value("--", None),
        ),
        prefix_class_value("card-", None)
    );
}

#[test]
fn concatenates_reduced_product_constraints_without_widening_to_top() {
    let left = composite_class_value(CompositeClassValueInputV0 {
        prefix: Some("btn-".to_string()),
        suffix: None,
        min_length: Some("btn-primary".len()),
        must_chars: "r".to_string(),
        may_chars: "btn-primary".to_string(),
        may_include_other_chars: true,
        provenance: None,
    });
    let right = char_inclusion_class_value("ae", "active", None, false);

    let concatenated = concatenate_abstract_class_values(&left, &right);

    assert_eq!(
        concatenated,
        AbstractClassValueV0::Composite {
            prefix: Some("btn-".to_string()),
            suffix: None,
            min_length: Some("btn-primary".len() + 2),
            must_chars: "-abenrt".to_string(),
            may_chars: "-abenrt".to_string(),
            may_include_other_chars: true,
            provenance: Some(AbstractClassValueProvenanceV0::CompositeConcat),
        }
    );
    assert_eq!(
        projected_names(
            &concatenated,
            &selector_universe(["btn-primary-active", "btn-icon", "card-active"]),
        ),
        vec!["btn-primary-active".to_string()]
    );
}

#[test]
fn flow_concat_preserves_reduced_product_shape() {
    let graph = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@composite-concat".to_string()),
        nodes: vec![
            ClassValueFlowNodeV0 {
                id: "base".to_string(),
                predecessors: Vec::new(),
                transfer: ClassValueFlowTransferV0::AssignFacts(
                    external_facts("constrained")
                        .with_constraint_kind("composite")
                        .with_prefix("btn-")
                        .with_char_must("r")
                        .with_min_len("btn-primary".len()),
                ),
            },
            ClassValueFlowNodeV0 {
                id: "active".to_string(),
                predecessors: vec!["base".to_string()],
                transfer: ClassValueFlowTransferV0::ConcatFacts(
                    external_facts("constrained")
                        .with_constraint_kind("charInclusion")
                        .with_char_must("ae")
                        .with_char_may("active"),
                ),
            },
        ],
    };

    let analysis = analyze_class_value_flow(&graph);

    assert_eq!(
        flow_value(&analysis, "active"),
        Some(&AbstractClassValueV0::Composite {
            prefix: Some("btn-".to_string()),
            suffix: None,
            min_length: Some("btn-primary".len() + 2),
            must_chars: "-abenrt".to_string(),
            may_chars: "-abceinrtv".to_string(),
            may_include_other_chars: false,
            provenance: Some(AbstractClassValueProvenanceV0::CompositeConcat),
        })
    );
}

#[test]
fn analyzes_flow_concat_facts_before_refinement() {
    let graph = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@concat".to_string()),
        nodes: vec![
            flow_assign_node("base", external_facts("exact").with_values(["btn-"])),
            ClassValueFlowNodeV0 {
                id: "variant".to_string(),
                predecessors: vec!["base".to_string()],
                transfer: ClassValueFlowTransferV0::ConcatFacts(
                    external_facts("finiteSet").with_values(["primary", "secondary", "icon"]),
                ),
            },
            ClassValueFlowNodeV0 {
                id: "btn-only".to_string(),
                predecessors: vec!["variant".to_string()],
                transfer: ClassValueFlowTransferV0::RefineFacts(
                    external_facts("constrained")
                        .with_constraint_kind("suffix")
                        .with_suffix("primary"),
                ),
            },
        ],
    };

    let analysis = analyze_class_value_flow(&graph);

    assert_eq!(
        flow_value(&analysis, "variant"),
        Some(&AbstractClassValueV0::FiniteSet {
            values: vec![
                "btn-icon".to_string(),
                "btn-primary".to_string(),
                "btn-secondary".to_string(),
            ]
        })
    );
    assert_eq!(
        flow_value(&analysis, "btn-only"),
        Some(&AbstractClassValueV0::Exact {
            value: "btn-primary".to_string()
        })
    );
    assert_eq!(analysis.nodes[1].transfer_kind, "concatFacts");
}

#[test]
fn analyzes_one_cfa_class_value_flow_with_branch_merge_and_refinement() {
    let graph = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@primary".to_string()),
        nodes: vec![
            flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
            flow_assign_node(
                "else-if",
                external_facts("exact").with_values(["btn-secondary"]),
            ),
            flow_assign_node("else", external_facts("exact").with_values(["card"])),
            ClassValueFlowNodeV0 {
                id: "merge".to_string(),
                predecessors: vec![
                    "then".to_string(),
                    "else-if".to_string(),
                    "else".to_string(),
                ],
                transfer: ClassValueFlowTransferV0::Join,
            },
            ClassValueFlowNodeV0 {
                id: "btn-only".to_string(),
                predecessors: vec!["merge".to_string()],
                transfer: ClassValueFlowTransferV0::RefineFacts(
                    external_facts("constrained")
                        .with_constraint_kind("prefix")
                        .with_prefix("btn-"),
                ),
            },
        ],
    };

    let analysis = analyze_class_value_flow(&graph);

    assert_eq!(analysis.schema_version, "0");
    assert_eq!(analysis.product, "omena-abstract-value.flow-analysis");
    assert_eq!(analysis.context_sensitivity, "1-cfa");
    assert_eq!(
        analysis.context_key.as_deref(),
        Some("Button.tsx:render@primary")
    );
    assert!(analysis.converged);

    assert_eq!(
        flow_value(&analysis, "merge"),
        Some(&AbstractClassValueV0::FiniteSet {
            values: vec![
                "btn-primary".to_string(),
                "btn-secondary".to_string(),
                "card".to_string(),
            ]
        })
    );
    assert_eq!(
        flow_value(&analysis, "btn-only"),
        Some(&AbstractClassValueV0::FiniteSet {
            values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
        })
    );
}

#[test]
fn analyzes_one_cfa_call_site_flows_with_context_discrimination() {
    let primary_graph = ClassValueFlowGraphV0 {
        context_key: None,
        nodes: vec![
            flow_assign_node(
                "variant",
                external_facts("exact").with_values(["btn-primary"]),
            ),
            ClassValueFlowNodeV0 {
                id: "exit".to_string(),
                predecessors: vec!["variant".to_string()],
                transfer: ClassValueFlowTransferV0::RefineFacts(
                    external_facts("constrained")
                        .with_constraint_kind("prefix")
                        .with_prefix("btn-"),
                ),
            },
        ],
    };
    let secondary_graph = ClassValueFlowGraphV0 {
        context_key: None,
        nodes: vec![
            flow_assign_node(
                "variant",
                external_facts("exact").with_values(["btn-secondary"]),
            ),
            ClassValueFlowNodeV0 {
                id: "exit".to_string(),
                predecessors: vec!["variant".to_string()],
                transfer: ClassValueFlowTransferV0::RefineFacts(
                    external_facts("constrained")
                        .with_constraint_kind("prefix")
                        .with_prefix("btn-"),
                ),
            },
        ],
    };

    let analysis = analyze_one_cfa_call_site_flows(&[
        OneCfaCallSiteFlowInputV0 {
            callee_key: "classForVariant".to_string(),
            call_site_id: "Button.tsx:10:className".to_string(),
            graph: primary_graph,
            exit_node_id: "exit".to_string(),
        },
        OneCfaCallSiteFlowInputV0 {
            callee_key: "classForVariant".to_string(),
            call_site_id: "Card.tsx:22:className".to_string(),
            graph: secondary_graph,
            exit_node_id: "exit".to_string(),
        },
    ]);

    assert_eq!(analysis.schema_version, "0");
    assert_eq!(
        analysis.product,
        "omena-abstract-value.one-cfa-call-site-flow"
    );
    assert_eq!(analysis.context_sensitivity, "1-cfa");
    assert_eq!(analysis.call_site_count, 2);
    assert_eq!(analysis.callee_count, 1);
    assert_eq!(
        analysis.entries[0].context_key,
        "classForVariant@Button.tsx:10:className"
    );
    assert_eq!(
        analysis.entries[1].context_key,
        "classForVariant@Card.tsx:22:className"
    );
    assert_eq!(
        analysis.entries[0].exit_value,
        exact_class_value("btn-primary")
    );
    assert_eq!(
        analysis.entries[1].exit_value,
        exact_class_value("btn-secondary")
    );
    assert_eq!(
        analysis.callee_summaries[0].joined_exit_value,
        AbstractClassValueV0::FiniteSet {
            values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
        }
    );
    assert_eq!(analysis.entries[0].derivation.steps.len(), 3);
    assert_eq!(
        analysis.entries[0].derivation.steps[0].operation,
        "contextFromCallSite"
    );
    assert_eq!(
        analysis.entries[0].derivation.steps[2].operation,
        "projectExitNode"
    );
    assert_eq!(analysis.entries[0].derivation.steps[2].result_kind, "exact");
}

#[test]
fn analyzes_k_cfa_limited_call_site_flows_with_context_stack_discrimination() {
    let analysis = analyze_k_limited_call_site_flows(
        &[
            KLimitedCallSiteFlowInputV0 {
                callee_key: "classForVariant".to_string(),
                call_site_stack: vec![
                    "RouteA.tsx:render".to_string(),
                    "Button.tsx:className".to_string(),
                ],
                graph: flow_exit_graph("btn-primary"),
                exit_node_id: "exit".to_string(),
            },
            KLimitedCallSiteFlowInputV0 {
                callee_key: "classForVariant".to_string(),
                call_site_stack: vec![
                    "RouteB.tsx:render".to_string(),
                    "Button.tsx:className".to_string(),
                ],
                graph: flow_exit_graph("btn-secondary"),
                exit_node_id: "exit".to_string(),
            },
        ],
        2,
    );

    assert_eq!(
        analysis.product,
        "omena-abstract-value.k-limited-call-site-flow"
    );
    assert_eq!(analysis.context_sensitivity, "2-cfa");
    assert_eq!(analysis.max_context_depth, 2);
    assert_eq!(
        analysis.entries[0].context_key,
        "classForVariant@RouteA.tsx:render > Button.tsx:className"
    );
    assert_eq!(
        analysis.entries[1].context_key,
        "classForVariant@RouteB.tsx:render > Button.tsx:className"
    );
    assert_eq!(
        analysis.callee_summaries[0].joined_exit_value,
        AbstractClassValueV0::FiniteSet {
            values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
        }
    );
}

#[test]
fn analyzes_control_flow_graph_with_reachability_pruning() {
    let graph = ClassValueControlFlowGraphV0 {
        context_key: Some("Button.tsx:render@cfg".to_string()),
        entry_block_id: "entry".to_string(),
        blocks: vec![
            ClassValueControlFlowBlockV0 {
                id: "entry".to_string(),
                nodes: vec![flow_assign_node(
                    "base",
                    external_facts("exact").with_values(["btn-primary"]),
                )],
                successor_block_ids: vec!["merge".to_string()],
            },
            ClassValueControlFlowBlockV0 {
                id: "dead".to_string(),
                nodes: vec![flow_assign_node(
                    "ghost",
                    external_facts("exact").with_values(["card"]),
                )],
                successor_block_ids: vec!["merge".to_string()],
            },
            ClassValueControlFlowBlockV0 {
                id: "merge".to_string(),
                nodes: vec![ClassValueFlowNodeV0 {
                    id: "exit".to_string(),
                    predecessors: vec!["base".to_string(), "ghost".to_string()],
                    transfer: ClassValueFlowTransferV0::Join,
                }],
                successor_block_ids: Vec::new(),
            },
        ],
    };

    let analysis = analyze_class_value_control_flow_graph(&graph);

    assert_eq!(
        analysis.product,
        "omena-abstract-value.control-flow-analysis"
    );
    assert_eq!(analysis.block_count, 3);
    assert_eq!(analysis.edge_count, 2);
    assert_eq!(analysis.reachable_block_count, 2);
    assert_eq!(analysis.unreachable_block_ids, vec!["dead".to_string()]);
    assert_eq!(
        flow_value(&analysis.flow_analysis, "exit"),
        Some(&exact_class_value("btn-primary"))
    );
    assert_eq!(
        analysis
            .blocks
            .iter()
            .find(|block| block.block_id == "dead")
            .map(|block| (&block.reachable, &block.exit_value)),
        Some((&false, &bottom_class_value()))
    );
}

#[test]
fn analyzes_class_value_flow_on_incremental_plan() {
    let graph = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@primary".to_string()),
        nodes: vec![
            flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
            flow_assign_node("else", external_facts("exact").with_values(["card"])),
            ClassValueFlowNodeV0 {
                id: "merge".to_string(),
                predecessors: vec!["then".to_string(), "else".to_string()],
                transfer: ClassValueFlowTransferV0::Join,
            },
        ],
    };

    let first = analyze_class_value_flow_incremental(&graph, None, 1);
    assert_eq!(
        first.product,
        "omena-abstract-value.incremental-flow-analysis"
    );
    assert!(!first.reused_previous_analysis);
    assert_eq!(first.incremental_plan.dirty_node_count, 3);
    assert_eq!(first.incremental_plan.new_node_count, 3);
    assert_eq!(
        flow_value(&first.analysis, "merge"),
        Some(&AbstractClassValueV0::FiniteSet {
            values: vec!["btn-primary".to_string(), "card".to_string()]
        })
    );

    let unchanged = analyze_class_value_flow_incremental(&graph, Some(&first.next_snapshot), 2);
    assert_eq!(unchanged.incremental_plan.dirty_node_count, 0);
    assert!(!unchanged.reused_previous_analysis);
    assert!(unchanged.analysis.converged);

    let changed_graph = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@primary".to_string()),
        nodes: vec![
            flow_assign_node(
                "then",
                external_facts("exact").with_values(["btn-secondary"]),
            ),
            flow_assign_node("else", external_facts("exact").with_values(["card"])),
            ClassValueFlowNodeV0 {
                id: "merge".to_string(),
                predecessors: vec!["then".to_string(), "else".to_string()],
                transfer: ClassValueFlowTransferV0::Join,
            },
        ],
    };
    let changed =
        analyze_class_value_flow_incremental(&changed_graph, Some(&first.next_snapshot), 3);

    assert_eq!(changed.incremental_plan.changed_input_count, 1);
    assert_eq!(changed.incremental_plan.dependency_dirty_count, 1);
    assert_eq!(
        flow_value(&changed.analysis, "merge"),
        Some(&AbstractClassValueV0::FiniteSet {
            values: vec!["btn-secondary".to_string(), "card".to_string()]
        })
    );
}

#[test]
fn reuses_previous_class_value_flow_analysis_when_incremental_plan_is_clean() {
    let graph = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@primary".to_string()),
        nodes: vec![
            flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
            flow_assign_node("else", external_facts("exact").with_values(["card"])),
            ClassValueFlowNodeV0 {
                id: "merge".to_string(),
                predecessors: vec!["then".to_string(), "else".to_string()],
                transfer: ClassValueFlowTransferV0::Join,
            },
        ],
    };
    let first = analyze_class_value_flow_incremental(&graph, None, 1);

    let reused = analyze_class_value_flow_incremental_with_reuse(
        &graph,
        Some(&first.next_snapshot),
        Some(&first.analysis),
        2,
    );

    assert_eq!(reused.incremental_plan.dirty_node_count, 0);
    assert!(reused.reused_previous_analysis);
    assert_eq!(reused.analysis, first.analysis);
}

#[test]
fn reuses_previous_class_value_flow_analysis_through_salsa_database() {
    let graph = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@primary".to_string()),
        nodes: vec![
            flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
            flow_assign_node("else", external_facts("exact").with_values(["card"])),
            ClassValueFlowNodeV0 {
                id: "merge".to_string(),
                predecessors: vec!["then".to_string(), "else".to_string()],
                transfer: ClassValueFlowTransferV0::Join,
            },
        ],
    };
    let mut incremental_database = OmenaIncrementalDatabaseV0::default();
    let first = analyze_class_value_flow_incremental_with_database(
        &graph,
        &mut incremental_database,
        None,
        1,
    );

    assert_eq!(
        first.next_snapshot.product,
        "omena-incremental.salsa-snapshot"
    );
    assert_eq!(first.incremental_plan.dirty_node_count, 3);
    assert!(!first.reused_previous_analysis);

    let reused = analyze_class_value_flow_incremental_with_database(
        &graph,
        &mut incremental_database,
        Some(&first.analysis),
        2,
    );

    assert_eq!(reused.incremental_plan.dirty_node_count, 0);
    assert!(reused.reused_previous_analysis);
    assert_eq!(reused.analysis, first.analysis);
}

#[test]
fn reuses_clean_contexts_in_incremental_flow_batch() {
    let primary = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@primary".to_string()),
        nodes: vec![
            flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
            flow_assign_node("else", external_facts("exact").with_values(["card"])),
            ClassValueFlowNodeV0 {
                id: "merge".to_string(),
                predecessors: vec!["then".to_string(), "else".to_string()],
                transfer: ClassValueFlowTransferV0::Join,
            },
        ],
    };
    let secondary = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@secondary".to_string()),
        nodes: vec![
            flow_assign_node(
                "base",
                external_facts("exact").with_values(["btn-secondary"]),
            ),
            ClassValueFlowNodeV0 {
                id: "refined".to_string(),
                predecessors: vec!["base".to_string()],
                transfer: ClassValueFlowTransferV0::RefineFacts(
                    external_facts("prefix").with_prefix("btn-"),
                ),
            },
        ],
    };
    let first = analyze_class_value_flow_incremental_batch_with_reuse(
        &[primary.clone(), secondary.clone()],
        &BTreeMap::new(),
        &BTreeMap::new(),
        1,
    );
    let previous_snapshots = first
        .entries
        .iter()
        .map(|entry| {
            (
                entry.context_key.clone(),
                entry.analysis.next_snapshot.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let previous_analyses = first
        .entries
        .iter()
        .map(|entry| (entry.context_key.clone(), entry.analysis.analysis.clone()))
        .collect::<BTreeMap<_, _>>();
    let changed_secondary = ClassValueFlowGraphV0 {
        context_key: Some("Button.tsx:render@secondary".to_string()),
        nodes: vec![
            flow_assign_node(
                "base",
                external_facts("exact").with_values(["btn-tertiary"]),
            ),
            ClassValueFlowNodeV0 {
                id: "refined".to_string(),
                predecessors: vec!["base".to_string()],
                transfer: ClassValueFlowTransferV0::RefineFacts(
                    external_facts("prefix").with_prefix("btn-"),
                ),
            },
        ],
    };

    let second = analyze_class_value_flow_incremental_batch_with_reuse(
        &[primary, changed_secondary],
        &previous_snapshots,
        &previous_analyses,
        2,
    );

    assert_eq!(
        second.product,
        "omena-abstract-value.incremental-flow-analysis-batch"
    );
    assert_eq!(second.context_count, 2);
    assert_eq!(second.reused_context_count, 1);
    assert_eq!(second.dirty_context_count, 1);
    assert!(second.entries[0].analysis.reused_previous_analysis);
    assert!(!second.entries[1].analysis.reused_previous_analysis);
    assert_eq!(
        flow_value(&second.entries[1].analysis.analysis, "refined"),
        Some(&AbstractClassValueV0::Exact {
            value: "btn-tertiary".to_string()
        })
    );
}

#[test]
fn reduces_external_facts_before_reporting_domain_kind() {
    let finite_with_prefix = external_facts("finiteSet")
        .with_values(["btn-primary", "card"])
        .with_constraint_kind("prefix")
        .with_prefix("btn-");

    assert_eq!(
        reduced_abstract_class_value_from_facts(&finite_with_prefix),
        exact_class_value("btn-primary")
    );
    assert_eq!(
        reduced_value_domain_kind_from_facts(&finite_with_prefix),
        "exact"
    );

    let constrained_with_values = external_facts("constrained")
        .with_values(["btn-primary", "card"])
        .with_constraint_kind("prefix")
        .with_prefix("btn-");

    assert_eq!(
        reduced_abstract_class_value_from_facts(&constrained_with_values),
        exact_class_value("btn-primary")
    );

    let finite_with_conflicting_prefix = external_facts("finiteSet")
        .with_values(["btn-primary", "card"])
        .with_constraint_kind("prefix")
        .with_prefix("nav-");

    assert_eq!(
        reduced_abstract_class_value_from_facts(&finite_with_conflicting_prefix),
        bottom_class_value()
    );
    assert_eq!(
        reduced_value_domain_kind_from_facts(&finite_with_conflicting_prefix),
        "bottom"
    );
    assert_eq!(
        reduced_value_domain_kind_from_facts(&external_facts("unknown")),
        "none"
    );
}

#[test]
fn explains_reduced_external_fact_derivation_steps() {
    let finite_with_prefix = external_facts("finiteSet")
        .with_values(["btn-primary", "card"])
        .with_constraint_kind("prefix")
        .with_prefix("btn-");

    let derivation = reduced_class_value_derivation_from_facts(&finite_with_prefix);

    assert_eq!(derivation.schema_version, "0");
    assert_eq!(
        derivation.product,
        "omena-abstract-value.reduced-class-value-derivation"
    );
    assert_eq!(derivation.input_fact_kind, "finiteSet");
    assert_eq!(derivation.input_constraint_kind.as_deref(), Some("prefix"));
    assert_eq!(derivation.input_value_count, 2);
    assert_eq!(derivation.reduced_kind, "exact");
    assert_eq!(derivation.steps.len(), 2);
    assert_eq!(derivation.steps[0].operation, "baseFromFacts");
    assert_eq!(derivation.steps[0].result_kind, "finiteSet");
    assert_eq!(derivation.steps[1].operation, "intersectConstraint");
    assert_eq!(derivation.steps[1].input_kind, Some("finiteSet"));
    assert_eq!(derivation.steps[1].refinement_kind, Some("prefix"));
    assert_eq!(derivation.steps[1].result_kind, "exact");
}

#[test]
fn explains_constrained_finite_value_derivation_steps() {
    let constrained_with_values = external_facts("constrained")
        .with_values(["btn-primary", "btn-secondary", "card"])
        .with_constraint_kind("prefix")
        .with_prefix("btn-");

    let derivation = reduced_class_value_derivation_from_facts(&constrained_with_values);

    assert_eq!(derivation.input_fact_kind, "constrained");
    assert_eq!(derivation.input_constraint_kind.as_deref(), Some("prefix"));
    assert_eq!(derivation.input_value_count, 3);
    assert_eq!(derivation.reduced_kind, "finiteSet");
    assert_eq!(derivation.steps.len(), 2);
    assert_eq!(derivation.steps[0].operation, "baseFromFacts");
    assert_eq!(derivation.steps[0].result_kind, "prefix");
    assert_eq!(derivation.steps[1].operation, "intersectFiniteValues");
    assert_eq!(derivation.steps[1].input_kind, Some("prefix"));
    assert_eq!(derivation.steps[1].refinement_kind, Some("finiteSet"));
    assert_eq!(derivation.steps[1].result_kind, "finiteSet");
}

#[test]
fn carries_result_provenance_in_reduced_derivation_steps() {
    let widened = external_facts("finiteSet").with_values([
        "btn-alpha-active",
        "btn-beta-active",
        "btn-gamma-active",
        "btn-delta-active",
        "btn-epsilon-active",
        "btn-zeta-active",
        "btn-eta-active",
        "btn-theta-active",
        "btn-iota-active",
    ]);

    let derivation = reduced_class_value_derivation_from_facts(&widened);

    assert_eq!(derivation.reduced_kind, "composite");
    assert_eq!(derivation.steps.len(), 1);
    assert_eq!(derivation.steps[0].operation, "baseFromFacts");
    assert_eq!(derivation.steps[0].result_kind, "composite");
    assert_eq!(
        derivation.steps[0].result_provenance,
        Some(AbstractClassValueProvenanceV0::FiniteSetWideningComposite)
    );
}

#[test]
fn summarizes_exact_value_provenance_tree() {
    let value = exact_class_value("button");
    let tree = summarize_abstract_class_value_provenance_tree(&value);

    assert_eq!(tree.schema_version, "0");
    assert_eq!(tree.product, "omena-abstract-value.provenance-tree");
    assert_eq!(tree.value_kind, "exact");
    assert_eq!(tree.value, value);
    assert_eq!(tree.value_provenance, None);
    assert_eq!(tree.root.operation, "exactLiteral");
    assert_eq!(tree.root.result_kind, "exact");
    assert_eq!(tree.root.detail.as_deref(), Some("value=button"));
    assert!(tree.root.children.is_empty());
}

#[test]
fn summarizes_finite_widening_provenance_tree() {
    let value = finite_set_class_value([
        "btn-alpha-active",
        "btn-beta-active",
        "btn-gamma-active",
        "btn-delta-active",
        "btn-epsilon-active",
        "btn-zeta-active",
        "btn-eta-active",
        "btn-theta-active",
        "btn-iota-active",
    ]);

    let tree = summarize_abstract_class_value_provenance_tree(&value);

    assert_eq!(tree.value_kind, "composite");
    assert_eq!(
        tree.value_provenance,
        Some(AbstractClassValueProvenanceV0::FiniteSetWideningComposite)
    );
    assert_eq!(tree.root.operation, "finiteSetWidening");
    assert_eq!(
        tree.root.reason,
        "large finite set widened to preserved edge and character constraints"
    );
    assert_provenance_child(&tree.root.children, "prefixConstraint", "prefix=btn-");
    assert_provenance_child(&tree.root.children, "suffixConstraint", "suffix=-active");
    assert_provenance_child(&tree.root.children, "lengthConstraint", "minLength=14");
    assert_provenance_child(
        &tree.root.children,
        "characterMustConstraint",
        "mustChars=-abceintv",
    );
}

#[test]
fn summarizes_reduced_product_join_provenance_tree() {
    let value = intersect_abstract_class_values(
        &prefix_class_value("btn-", None),
        &suffix_class_value("-active", None),
    );

    let tree = summarize_abstract_class_value_provenance_tree(&value);

    assert_eq!(tree.value_kind, "prefixSuffix");
    assert_eq!(
        tree.value_provenance,
        Some(AbstractClassValueProvenanceV0::CompositeJoin)
    );
    assert_eq!(tree.root.operation, "reducedProductJoin");
    assert_eq!(
        tree.root.reason,
        "reduced product combined compatible constraints from multiple domains"
    );
    assert_provenance_child(&tree.root.children, "prefixConstraint", "prefix=btn-");
    assert_provenance_child(&tree.root.children, "suffixConstraint", "suffix=-active");
    assert_provenance_child(&tree.root.children, "lengthConstraint", "minLength=10");
}

#[test]
fn projects_exact_and_finite_values_into_selector_universe() {
    let selectors = selector_universe(["button", "card", "link"]);

    let exact = project_abstract_value_selectors(&exact_class_value("button"), &selectors);
    assert_eq!(exact.selector_names, vec!["button".to_string()]);
    assert_eq!(exact.certainty, SelectorProjectionCertaintyV0::Exact);

    let finite = project_abstract_value_selectors(
        &finite_set_class_value(["button", "missing"]),
        &selectors,
    );
    assert_eq!(finite.selector_names, vec!["button".to_string()]);
    assert_eq!(finite.certainty, SelectorProjectionCertaintyV0::Inferred);
}

#[test]
fn projects_constrained_values_into_selector_universe() {
    let selectors = selector_universe(["btn-primary", "btn-secondary", "card", "link-active"]);

    let prefix = project_abstract_value_selectors(
        &prefix_class_value("btn-", Some(AbstractClassValueProvenanceV0::PrefixJoinLcp)),
        &selectors,
    );
    assert_eq!(
        prefix.selector_names,
        vec!["btn-primary".to_string(), "btn-secondary".to_string()]
    );
    assert_eq!(prefix.certainty, SelectorProjectionCertaintyV0::Inferred);

    let edge = project_abstract_value_selectors(
        &prefix_suffix_class_value("btn-", "primary", None, None),
        &selectors,
    );
    assert_eq!(edge.selector_names, vec!["btn-primary".to_string()]);
    assert_eq!(edge.certainty, SelectorProjectionCertaintyV0::Inferred);

    let chars = project_abstract_value_selectors(
        &char_inclusion_class_value("ac", "acdr", None, false),
        &selectors,
    );
    assert_eq!(chars.selector_names, vec!["card".to_string()]);
    assert_eq!(chars.certainty, SelectorProjectionCertaintyV0::Inferred);
}

#[test]
fn derives_projection_certainty_from_domain_and_selector_coverage() {
    assert_eq!(
        derive_selector_projection_certainty(&AbstractClassValueV0::Bottom, 0, 3),
        SelectorProjectionCertaintyV0::Possible
    );
    assert_eq!(
        derive_selector_projection_certainty(&prefix_class_value("btn-", None), 3, 3,),
        SelectorProjectionCertaintyV0::Inferred
    );
    assert_eq!(
        derive_selector_projection_certainty(&AbstractClassValueV0::Top, 3, 3),
        SelectorProjectionCertaintyV0::Possible
    );
}

fn selector_universe(values: impl IntoIterator<Item = &'static str>) -> Vec<String> {
    values.into_iter().map(str::to_string).collect()
}

fn assert_provenance_child(
    children: &[AbstractClassValueProvenanceNodeV0],
    operation: &str,
    detail: &str,
) {
    assert!(
        children.iter().any(|child| {
            child.operation == operation && child.detail.as_deref() == Some(detail)
        }),
        "missing provenance child operation={operation} detail={detail}: {children:#?}"
    );
}

fn assert_projection_equivalent(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
    selectors: &[String],
) {
    assert_eq!(
        projected_names(left, selectors),
        projected_names(right, selectors)
    );
}

fn projected_names(value: &AbstractClassValueV0, selectors: &[String]) -> Vec<String> {
    project_abstract_value_selectors(value, selectors).selector_names
}

fn intersect_projected_names(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
    selectors: &[String],
) -> Vec<String> {
    let right_names = projected_names(right, selectors)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    projected_names(left, selectors)
        .into_iter()
        .filter(|name| right_names.contains(name))
        .collect()
}

fn flow_assign_node(id: &str, facts: ExternalStringTypeFactsV0) -> ClassValueFlowNodeV0 {
    ClassValueFlowNodeV0 {
        id: id.to_string(),
        predecessors: Vec::new(),
        transfer: ClassValueFlowTransferV0::AssignFacts(facts),
    }
}

fn property_candidate(
    property_name: &str,
    value: &str,
    pseudo_state: Option<&str>,
) -> AbstractPropertyValueCandidateV0 {
    AbstractPropertyValueCandidateV0 {
        property_name: property_name.to_string(),
        value: value.to_string(),
        pseudo_state: pseudo_state.map(str::to_string),
    }
}

fn flow_exit_graph(value: &str) -> ClassValueFlowGraphV0 {
    ClassValueFlowGraphV0 {
        context_key: None,
        nodes: vec![
            flow_assign_node(
                "value",
                ExternalStringTypeFactsV0 {
                    kind: "exact".to_string(),
                    constraint_kind: None,
                    values: Some(vec![value.to_string()]),
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                },
            ),
            ClassValueFlowNodeV0 {
                id: "exit".to_string(),
                predecessors: vec!["value".to_string()],
                transfer: ClassValueFlowTransferV0::Join,
            },
        ],
    }
}

fn flow_value<'a>(
    analysis: &'a super::ClassValueFlowAnalysisV0,
    id: &str,
) -> Option<&'a AbstractClassValueV0> {
    analysis
        .nodes
        .iter()
        .find(|node| node.id == id)
        .map(|node| &node.value)
}

fn external_facts(kind: &str) -> ExternalStringTypeFactsV0 {
    ExternalStringTypeFactsV0 {
        kind: kind.to_string(),
        constraint_kind: None,
        values: None,
        prefix: None,
        suffix: None,
        min_len: None,
        max_len: None,
        char_must: None,
        char_may: None,
        may_include_other_chars: None,
    }
}

trait ExternalFactsTestExt {
    fn with_values(self, values: impl IntoIterator<Item = &'static str>) -> Self;
    fn with_constraint_kind(self, value: &'static str) -> Self;
    fn with_prefix(self, value: &'static str) -> Self;
    fn with_suffix(self, value: &'static str) -> Self;
    fn with_min_len(self, value: usize) -> Self;
    fn with_char_must(self, value: &'static str) -> Self;
    fn with_char_may(self, value: &'static str) -> Self;
}

impl ExternalFactsTestExt for ExternalStringTypeFactsV0 {
    fn with_values(mut self, values: impl IntoIterator<Item = &'static str>) -> Self {
        self.values = Some(values.into_iter().map(str::to_string).collect());
        self
    }

    fn with_constraint_kind(mut self, value: &'static str) -> Self {
        self.constraint_kind = Some(value.to_string());
        self
    }

    fn with_prefix(mut self, value: &'static str) -> Self {
        self.prefix = Some(value.to_string());
        self
    }

    fn with_suffix(mut self, value: &'static str) -> Self {
        self.suffix = Some(value.to_string());
        self
    }

    fn with_min_len(mut self, value: usize) -> Self {
        self.min_len = Some(value);
        self
    }

    fn with_char_must(mut self, value: &'static str) -> Self {
        self.char_must = Some(value.to_string());
        self
    }

    fn with_char_may(mut self, value: &'static str) -> Self {
        self.char_may = Some(value.to_string());
        self
    }
}

use std::collections::BTreeMap;

use super::InstanceReachabilityDerivationV0;

pub(super) fn assert_instance_reachability_fan_out(
    expected_count: usize,
    emitted_count: usize,
    derivations: &[Option<InstanceReachabilityDerivationV0>],
) {
    // FALSIFIER: id=query-instance-reachability-fan-out-count class=accounting via=configured_sass_edges_select_distinct_instances_without_reparsing producer=can-fail owner=query-carrier-contract entry=one-row-per-finalized-configuration
    assert_eq!(emitted_count, expected_count);
    // FALSIFIER: id=query-instance-reachability-derivation class=accounting via=configured_sass_edges_select_distinct_instances_without_reparsing producer=can-fail owner=query-carrier-contract entry=path-union-derivation-visible
    assert!(derivations.iter().all(|derivation| {
        *derivation == Some(InstanceReachabilityDerivationV0::PathUnionNoInstanceDiscriminator)
    }));
}

pub(super) fn assert_resolution_authority_census(
    expected_edge_count: usize,
    disclosed_edge_count: usize,
    legacy_edge_count: usize,
) {
    // FALSIFIER: id=query-resolution-authority-edge-census class=accounting via=configured_sass_edges_select_distinct_instances_without_reparsing producer=can-fail owner=query-carrier-contract entry=one-disclosure-per-product-edge
    assert_eq!(disclosed_edge_count, expected_edge_count);
    // FALSIFIER: id=query-resolution-authority-legacy-census class=accounting via=configured_sass_edges_select_distinct_instances_without_reparsing producer=can-fail owner=query-carrier-contract entry=product-producer-fully-resolved
    assert_eq!(legacy_edge_count, 0);
}

pub(super) fn assert_pretty_render_measurement(actual: &BTreeMap<String, bool>) {
    let expected = BTreeMap::from([
        ("src/app.css".to_string(), false),
        ("src/tokens.css".to_string(), false),
        ("src/width.css".to_string(), true),
    ]);
    // FALSIFIER: id=query-linked-source-map-pretty-measurement class=placement via=linked_bundle_source_map_uses_materialized_module_offsets producer=can-fail owner=query-carrier-contract entry=pretty-width-measurement-pinned
    assert_eq!(actual, &expected);
}

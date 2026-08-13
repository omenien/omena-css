use std::collections::BTreeMap;

use super::{
    BundleDependencyResolutionDisclosureV0, BundleResolutionAuthorityV0,
    ClosedWorldModuleReachabilityEvidenceV0, LinkedStylesheetWithEmissionItemsV0, LinkerInputV0,
    ModuleInstanceKeyV0, TransformBundleLinkErrorV0,
};

pub(super) fn assert_configured_instance_reachability(
    red_names: &[String],
    blue_names: &[String],
    evidence: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldModuleReachabilityEvidenceV0>,
    red: &ModuleInstanceKeyV0,
    blue: &ModuleInstanceKeyV0,
) {
    // FALSIFIER: id=bundler-configured-red-reachability class=liveness via=instance_reachability_keeps_configured_consumers_distinct producer=can-fail owner=bundler-carrier-contract entry=red-instance-keeps-only-red-live-set
    assert_eq!(red_names, ["alpha"]);
    // FALSIFIER: id=bundler-configured-blue-reachability class=liveness via=instance_reachability_keeps_configured_consumers_distinct producer=can-fail owner=bundler-carrier-contract entry=blue-instance-keeps-only-blue-live-set
    assert_eq!(blue_names, ["beta"]);
    // FALSIFIER: id=bundler-configured-red-evidence class=accounting via=instance_reachability_keeps_configured_consumers_distinct producer=can-fail owner=bundler-carrier-contract entry=red-instance-evidence-supplied
    assert_eq!(
        evidence.get(red),
        Some(&ClosedWorldModuleReachabilityEvidenceV0::Supplied)
    );
    // FALSIFIER: id=bundler-configured-blue-evidence class=accounting via=instance_reachability_keeps_configured_consumers_distinct producer=can-fail owner=bundler-carrier-contract entry=blue-instance-evidence-supplied
    assert_eq!(
        evidence.get(blue),
        Some(&ClosedWorldModuleReachabilityEvidenceV0::Supplied)
    );
}

pub(super) fn assert_duplicate_instance_reachability(names: &[String]) {
    // FALSIFIER: id=bundler-duplicate-instance-reachability class=liveness via=instance_reachability_unions_duplicate_rows producer=can-fail owner=bundler-carrier-contract entry=duplicate-producer-rows-are-unioned
    assert_eq!(names, ["alpha", "beta"]);
}

pub(super) fn assert_legacy_path_union(input: &LinkerInputV0) {
    // FALSIFIER: id=bundler-legacy-path-class-union class=liveness via=legacy_path_reachability_unions_normalized_rows_across_symbol_sets producer=can-fail owner=bundler-carrier-contract entry=normalized-alias-classes-unioned
    assert_eq!(input.class_names, ["alpha", "beta"]);
    // FALSIFIER: id=bundler-legacy-path-keyframe-union class=liveness via=legacy_path_reachability_unions_normalized_rows_across_symbol_sets producer=can-fail owner=bundler-carrier-contract entry=normalized-alias-keyframes-unioned
    assert_eq!(input.keyframe_names, ["enter", "leave"]);
    // FALSIFIER: id=bundler-legacy-path-value-union class=liveness via=legacy_path_reachability_unions_normalized_rows_across_symbol_sets producer=can-fail owner=bundler-carrier-contract entry=normalized-alias-values-unioned
    assert_eq!(input.value_names, ["primary", "secondary"]);
    // FALSIFIER: id=bundler-legacy-path-custom-property-union class=liveness via=legacy_path_reachability_unions_normalized_rows_across_symbol_sets producer=can-fail owner=bundler-carrier-contract entry=normalized-alias-custom-properties-unioned
    assert_eq!(input.custom_property_names, ["--primary", "--secondary"]);
}

pub(super) fn assert_incomplete_composes_target_evidence(
    evidence: &BTreeMap<ModuleInstanceKeyV0, ClosedWorldModuleReachabilityEvidenceV0>,
    source: &ModuleInstanceKeyV0,
    target: &ModuleInstanceKeyV0,
) {
    // FALSIFIER: id=bundler-incomplete-composes-source-evidence class=accounting via=incomplete_composes_carrier_marks_closure_target_evidence_absent producer=can-fail owner=bundler-carrier-contract entry=source-reachability-remains-supplied
    assert_eq!(
        evidence.get(source),
        Some(&ClosedWorldModuleReachabilityEvidenceV0::Supplied)
    );
    // FALSIFIER: id=bundler-incomplete-composes-target-evidence class=accounting via=incomplete_composes_carrier_marks_closure_target_evidence_absent producer=can-fail owner=bundler-carrier-contract entry=closure-target-evidence-is-typed-absent
    assert_eq!(
        evidence.get(target),
        Some(&ClosedWorldModuleReachabilityEvidenceV0::ModuleReachabilityInputAbsent)
    );
}

pub(super) fn assert_incomplete_composes_target_symbols(
    source_names: &[String],
    target_names: &[String],
) {
    // FALSIFIER: id=bundler-incomplete-composes-source-symbols class=liveness via=incomplete_composes_carrier_keeps_closure_target_symbols_fail_open producer=can-fail owner=bundler-carrier-contract entry=source-symbols-use-valid-reachability
    assert_eq!(source_names, ["card"]);
    // FALSIFIER: id=bundler-incomplete-composes-target-symbols class=liveness via=incomplete_composes_carrier_keeps_closure_target_symbols_fail_open producer=can-fail owner=bundler-carrier-contract entry=closure-target-symbols-fail-open
    assert_eq!(target_names, ["base", "other"]);
}

pub(super) fn assert_resolution_authority(
    strict_error: &Result<LinkedStylesheetWithEmissionItemsV0, TransformBundleLinkErrorV0>,
    inferred: &[&BundleDependencyResolutionDisclosureV0],
    strict_disclosures: &[BundleDependencyResolutionDisclosureV0],
) {
    // FALSIFIER: id=bundler-strict-missing-edge class=accounting via=resolution_authority_is_enforced_per_dependency_edge producer=can-fail owner=bundler-carrier-contract entry=strict-mode-names-missing-edge
    assert_eq!(
        strict_error,
        &Err(TransformBundleLinkErrorV0::UnresolvedDependencyEdge {
            source_path: "src/app.css".to_string(),
            import_source: "./theme".to_string(),
            import_ordinal: Some(1),
        })
    );
    // FALSIFIER: id=bundler-legacy-unmatched-edge-count class=accounting via=resolution_authority_is_enforced_per_dependency_edge producer=can-fail owner=bundler-carrier-contract entry=one-unmatched-edge-disclosed
    assert_eq!(inferred.len(), 1);
    // FALSIFIER: id=bundler-legacy-unmatched-edge-source class=placement via=resolution_authority_is_enforced_per_dependency_edge producer=can-fail owner=bundler-carrier-contract entry=unmatched-edge-source-disclosed
    assert_eq!(inferred[0].import_source, "./theme");
    // FALSIFIER: id=bundler-legacy-unmatched-edge-ordinal class=placement via=resolution_authority_is_enforced_per_dependency_edge producer=can-fail owner=bundler-carrier-contract entry=unmatched-edge-ordinal-disclosed
    assert_eq!(inferred[0].import_ordinal, Some(1));
    // FALSIFIER: id=bundler-strict-resolved-edge-count class=accounting via=resolution_authority_is_enforced_per_dependency_edge producer=can-fail owner=bundler-carrier-contract entry=all-strict-edges-disclosed
    assert_eq!(strict_disclosures.len(), 2);
    // FALSIFIER: id=bundler-strict-resolved-authority class=accounting via=resolution_authority_is_enforced_per_dependency_edge producer=can-fail owner=bundler-carrier-contract entry=all-strict-edges-resolved
    assert!(
        strict_disclosures
            .iter()
            .all(|disclosure| disclosure.authority == BundleResolutionAuthorityV0::Resolved)
    );
}

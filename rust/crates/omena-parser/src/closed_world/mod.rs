//! Typed closed-world inputs shared by the linker and transform consumers.

mod authority;
mod contract;

pub use authority::summarize_closed_world_reachability_bitset_parity_v0;
pub use contract::{
    ClosedWorldBundleBuildErrorV0, ClosedWorldBundleV0, ClosedWorldInterfaceHashAvailabilityV0,
    ClosedWorldInterfaceHashEntryV0, ClosedWorldInterfaceHashSetV0, ClosedWorldLinkedModuleV0,
    ClosedWorldModuleMetadataV0, ClosedWorldReachabilityBitsetParityReportV0,
    ClosedWorldSourcePrecisionSummaryV0, ConfigurationHashV0, ModuleIdV0, ModuleInstanceKeyV0,
    ModuleQualifiedSymbolSetV0, OpenWorldSnapshotV0, ReachabilityIndexV0,
};

#[cfg(test)]
mod tests {
    use super::{
        ClosedWorldBundleBuildErrorV0, ClosedWorldBundleV0, ClosedWorldInterfaceHashAvailabilityV0,
        ClosedWorldLinkedModuleV0, ClosedWorldModuleMetadataV0,
        ClosedWorldSourcePrecisionSummaryV0, ConfigurationHashV0, ModuleIdV0, ModuleInstanceKeyV0,
        summarize_closed_world_reachability_bitset_parity_v0,
    };

    #[test]
    fn module_instance_key_distinguishes_configurations() {
        let module = ModuleIdV0::new("src/theme.scss");
        let light =
            ModuleInstanceKeyV0::new(module.clone(), ConfigurationHashV0::new("with:brand=light"));
        let dark = ModuleInstanceKeyV0::new(module, ConfigurationHashV0::new("with:brand=dark"));

        assert_ne!(light, dark);
        assert_eq!(light.module().as_str(), "src/theme.scss");
        assert_eq!(dark.configuration().as_str(), "with:brand=dark");
    }

    #[test]
    fn closed_world_bundle_constructor_computes_reachability() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let tokens = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/tokens.css"));
        let modules = vec![
            ClosedWorldLinkedModuleV0::new(app.clone())
                .with_dependency(tokens.clone())
                .with_class_name("button"),
            ClosedWorldLinkedModuleV0::new(tokens.clone())
                .with_custom_property_name("--brand")
                .with_value_name("spacing"),
        ];

        let bundle = ClosedWorldBundleV0::try_from_linked_modules(vec![app], modules)
            .map_err(|err| format!("{err:?}"))?;

        assert_eq!(bundle.linked_modules().len(), 2);
        assert!(bundle.reachability().module_instances().contains(&tokens));
        assert!(
            bundle
                .reachability()
                .class_names()
                .contains(&"button".to_string())
        );
        assert!(
            bundle
                .reachability()
                .custom_property_names()
                .contains(&"--brand".to_string())
        );
        Ok(())
    }

    #[test]
    fn closed_world_bundle_closure_hash_is_content_addressed() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let module = ClosedWorldLinkedModuleV0::new(app.clone()).with_class_name("button");
        let first = ClosedWorldBundleV0::try_from_linked_modules(vec![app.clone()], vec![module])
            .map_err(|err| format!("{err:?}"))?;
        let same = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![ClosedWorldLinkedModuleV0::new(app.clone()).with_class_name("button")],
        )
        .map_err(|err| format!("{err:?}"))?;
        let changed = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![ClosedWorldLinkedModuleV0::new(app).with_class_name("card")],
        )
        .map_err(|err| format!("{err:?}"))?;

        assert_eq!(first.closure_hash(), same.closure_hash());
        assert_ne!(first.closure_hash(), changed.closure_hash());
        Ok(())
    }

    #[test]
    fn module_qualified_reachability_distinguishes_known_dead_modules() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let detached = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/detached.css"));
        let bundle = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![
                ClosedWorldLinkedModuleV0::new(app.clone())
                    .with_class_name("shared")
                    .with_keyframe_name("spin")
                    .with_value_name("tone")
                    .with_custom_property_name("--brand"),
                ClosedWorldLinkedModuleV0::new(detached.clone())
                    .with_class_name("shared")
                    .with_keyframe_name("spin")
                    .with_value_name("tone")
                    .with_custom_property_name("--brand"),
            ],
        )
        .map_err(|err| format!("{err:?}"))?;

        let app_symbols = bundle
            .reachability()
            .symbols_for_module(&app)
            .ok_or_else(|| "entrypoint symbol set should exist".to_string())?;
        let detached_symbols = bundle
            .reachability()
            .symbols_for_module(&detached)
            .ok_or_else(|| "known detached module should remain distinguishable".to_string())?;

        assert!(app_symbols.is_reachable());
        assert_eq!(app_symbols.class_names(), &["shared".to_string()]);
        assert_eq!(app_symbols.keyframe_names(), &["spin".to_string()]);
        assert_eq!(app_symbols.value_names(), &["tone".to_string()]);
        assert_eq!(
            app_symbols.custom_property_names(),
            &["--brand".to_string()]
        );
        assert!(!detached_symbols.is_reachable());
        assert!(detached_symbols.class_names().is_empty());
        assert!(detached_symbols.keyframe_names().is_empty());
        assert!(detached_symbols.value_names().is_empty());
        assert!(detached_symbols.custom_property_names().is_empty());
        assert!(
            bundle
                .reachability()
                .class_names()
                .contains(&"shared".to_string())
        );
        assert!(
            bundle
                .reachability()
                .symbols_for_module(&ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new(
                    "src/unknown.css",
                )))
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn module_qualified_ownership_does_not_change_closure_hash() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let dependency = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/dependency.css"));
        let first = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![
                ClosedWorldLinkedModuleV0::new(app.clone())
                    .with_dependency(dependency.clone())
                    .with_class_name("shared"),
                ClosedWorldLinkedModuleV0::new(dependency.clone()),
            ],
        )
        .map_err(|err| format!("{err:?}"))?;
        let second = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![
                ClosedWorldLinkedModuleV0::new(app).with_dependency(dependency.clone()),
                ClosedWorldLinkedModuleV0::new(dependency).with_class_name("shared"),
            ],
        )
        .map_err(|err| format!("{err:?}"))?;

        assert_eq!(
            first.reachability().module_instances(),
            second.reachability().module_instances()
        );
        assert_eq!(
            first.reachability().class_names(),
            second.reachability().class_names()
        );
        assert_ne!(
            first.reachability().module_qualified_symbols(),
            second.reachability().module_qualified_symbols()
        );
        assert_eq!(first.closure_hash(), second.closure_hash());
        Ok(())
    }

    #[test]
    fn closed_world_bitset_reachability_preserves_closure_hash() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let tokens = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/tokens.css"));
        let theme = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/theme.css"));
        let report = summarize_closed_world_reachability_bitset_parity_v0(
            vec![app.clone()],
            vec![
                ClosedWorldLinkedModuleV0::new(app)
                    .with_dependency(tokens.clone())
                    .with_class_name("button"),
                ClosedWorldLinkedModuleV0::new(tokens.clone())
                    .with_dependency(theme.clone())
                    .with_value_name("spacing"),
                ClosedWorldLinkedModuleV0::new(theme)
                    .with_keyframe_name("fade")
                    .with_custom_property_name("--brand"),
            ],
        )
        .map_err(|err| format!("{err:?}"))?;

        assert!(report.reachability_equal, "{report:#?}");
        assert!(report.closure_hash_equal, "{report:#?}");
        assert!(report.module_instance_count >= 3, "{report:#?}");
        assert!(report.symbol_name_count >= 3, "{report:#?}");
        Ok(())
    }

    #[test]
    fn closed_world_bundle_rejects_missing_dependency() {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let missing = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/missing.css"));
        let err = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![ClosedWorldLinkedModuleV0::new(app.clone()).with_dependency(missing.clone())],
        );

        assert_eq!(
            err,
            Err(ClosedWorldBundleBuildErrorV0::MissingDependency {
                module: app,
                dependency: missing,
            })
        );
    }

    #[test]
    fn closed_world_bundle_preserves_legacy_json_when_metadata_is_absent() -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let bundle = ClosedWorldBundleV0::try_from_linked_modules(
            vec![app.clone()],
            vec![ClosedWorldLinkedModuleV0::new(app)],
        )
        .map_err(|err| format!("{err:?}"))?;
        let json = serde_json::to_value(&bundle).map_err(|err| err.to_string())?;

        assert!(json.get("interfaceHashes").is_none());
        assert!(json.get("sourcePrecision").is_none());
        assert!(bundle.interface_hashes().all_absent());
        assert_eq!(bundle.source_precision(), None);
        Ok(())
    }

    #[test]
    fn closed_world_bundle_keys_interface_hashes_and_precision_by_module_instance()
    -> Result<(), String> {
        let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/app.css"));
        let tokens = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/tokens.css"));
        let bundle = ClosedWorldBundleV0::try_from_linked_modules_with_metadata(
            vec![app.clone()],
            vec![
                ClosedWorldLinkedModuleV0::new(app)
                    .with_dependency(tokens.clone())
                    .with_class_name("app"),
                ClosedWorldLinkedModuleV0::new(tokens.clone()),
            ],
            vec![
                ClosedWorldModuleMetadataV0::new(ModuleInstanceKeyV0::unconfigured(
                    ModuleIdV0::new("src/app.css"),
                ))
                .with_interface_hash("blake3:app")
                .with_source_precision(ClosedWorldSourcePrecisionSummaryV0 {
                    conservative_source_count: 1,
                    ..ClosedWorldSourcePrecisionSummaryV0::default()
                }),
            ],
        )
        .map_err(|err| format!("{err:?}"))?;

        assert_eq!(bundle.interface_hashes().entries().len(), 2);
        assert!(bundle.interface_hashes().entries().iter().any(|entry| {
            entry.module_instance == tokens
                && entry.availability == ClosedWorldInterfaceHashAvailabilityV0::Absent
        }));
        assert!(bundle.interface_hashes().entries().iter().any(|entry| {
            matches!(
                &entry.availability,
                ClosedWorldInterfaceHashAvailabilityV0::Known { interface_hash }
                    if interface_hash == "blake3:app"
            )
        }));
        assert_eq!(
            bundle.source_precision(),
            Some(ClosedWorldSourcePrecisionSummaryV0 {
                conservative_source_count: 1,
                ..ClosedWorldSourcePrecisionSummaryV0::default()
            })
        );
        Ok(())
    }
}

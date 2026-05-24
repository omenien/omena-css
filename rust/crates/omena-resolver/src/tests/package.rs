use super::*;

#[test]
fn resolves_package_manifest_style_exports() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
        "/fake/workspace/src/App.module.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                .to_string(),
        }],
    );

    assert_eq!(resolution.product, "omena-resolver.style-module-resolution");
    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
    assert!(resolution.candidate_count > 0);
    assert!(
        resolution
            .candidates
            .contains(&"/fake/workspace/node_modules/@design/tokens/dist/theme.css".to_string())
    );
}

#[test]
fn resolves_package_manifest_subpath_export_patterns() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/dist/themes/dark.css"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/themes/dark",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./themes/*":{"style":"./dist/themes/*.css"}}}"#
                .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/themes/dark.css")
    );
}

#[test]
fn resolves_package_manifest_subpath_export_patterns_by_specificity() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/dist/broad/dark/button.css",
        "/fake/workspace/node_modules/@design/tokens/dist/specific/button.css",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/themes/dark/button",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./themes/*":{"style":"./dist/broad/*.css"},"./themes/dark/*":{"style":"./dist/specific/*.css"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/specific/button.css")
    );
}

#[test]
fn package_export_null_subpath_blocks_pattern_and_file_fallback() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/private/theme.css",
        "/fake/workspace/node_modules/@design/tokens/dist/private/theme.css",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/private/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./private/*":null,"./*":{"style":"./dist/*.css"}}}"#.to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "unresolved");
    assert!(resolution.resolved_style_path.is_none());
    assert!(
        !resolution
            .candidates
            .contains(&"/fake/workspace/node_modules/@design/tokens/private/theme.css".to_string())
    );
    assert!(!resolution.candidates.contains(
        &"/fake/workspace/node_modules/@design/tokens/dist/private/theme.css".to_string()
    ));
}

#[test]
fn resolves_package_manifest_export_conditions_in_object_order() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
        "/fake/workspace/node_modules/@design/tokens/dist/theme.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./theme":{"style":"./dist/theme.css","sass":"./dist/theme.scss"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
}

#[test]
fn resolves_package_manifest_export_array_fallbacks() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/dist/theme.css"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./theme":[{"import":"./dist/theme.js"},{"style":"./dist/theme.css"}]}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
}

#[test]
fn ignores_non_sass_package_manifest_export_conditions() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/dist/theme.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./theme":{"import":"./dist/theme.mjs","require":"./dist/theme.cjs","sass":"./dist/theme.scss"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.scss")
    );
}

#[test]
fn skips_non_style_default_package_manifest_export_conditions() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/dist/theme.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./theme":{"default":"./dist/theme.js","sass":"./dist/theme.scss"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.scss")
    );
}

#[test]
fn respects_style_default_package_manifest_export_condition_order() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
        "/fake/workspace/node_modules/@design/tokens/dist/theme.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./theme":{"default":"./dist/theme.css","sass":"./dist/theme.scss"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
}

#[test]
fn resolves_package_manifest_exports_before_legacy_top_level_fields() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/legacy.scss",
        "/fake/workspace/node_modules/@design/tokens/modern.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"sass":"./legacy.scss","exports":{"sass":"./modern.scss","style":"./modern.css","default":"./default.css"}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/modern.scss")
    );
}

#[test]
fn resolves_sass_pkg_url_package_manifest_exports() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/dist/theme.css"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "pkg:@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
}

#[test]
fn resolves_sass_pkg_url_package_manifest_export_patterns() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/scss/themes/dark.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "pkg:@design/tokens/themes/dark",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./themes/*":{"sass":"./scss/themes/*.scss"}}}"#
                .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/scss/themes/dark.scss")
    );
}

#[test]
fn resolves_sass_pkg_url_package_manifest_export_patterns_by_specificity() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/scss/broad/dark/button.scss",
        "/fake/workspace/node_modules/@design/tokens/scss/specific/button.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "pkg:@design/tokens/themes/dark/button",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./themes/*":{"sass":"./scss/broad/*.scss"},"./themes/dark/*":{"sass":"./scss/specific/*.scss"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/scss/specific/button.scss")
    );
}

#[test]
fn resolves_sass_pkg_url_export_conditions_by_node_package_importer_priority() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
        "/fake/workspace/node_modules/@design/tokens/dist/theme.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "pkg:@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./theme":{"style":"./dist/theme.css","sass":"./dist/theme.scss"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.scss")
    );
}

#[test]
fn resolves_sass_pkg_url_default_condition_after_sass_and_style_priority() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
        "/fake/workspace/node_modules/@design/tokens/dist/theme.default.css",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "pkg:@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./theme":{"default":"./dist/theme.default.css","style":"./dist/theme.css"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
}

#[test]
fn ignores_scss_condition_for_sass_pkg_url_exports() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
        "/fake/workspace/node_modules/@design/tokens/dist/theme.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "pkg:@design/tokens/theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source:
                r#"{"exports":{"./theme":{"scss":"./dist/theme.scss","style":"./dist/theme.css"}}}"#
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
}

#[test]
fn resolves_sass_pkg_url_top_level_sass_before_style() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/theme.css",
        "/fake/workspace/node_modules/@design/tokens/theme.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "pkg:@design/tokens",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"sass":"./theme.scss","style":"./theme.css"}"#.to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/theme.scss")
    );
}

#[test]
fn ignores_top_level_scss_for_sass_pkg_url_root() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/tokens/theme.css",
        "/fake/workspace/node_modules/@design/tokens/theme.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "pkg:@design/tokens",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"scss":"./theme.scss","style":"./theme.css"}"#.to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/theme.css")
    );
}

#[test]
fn falls_back_to_package_root_index_when_manifest_has_no_style_entry() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/src/index.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "@design/tokens",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"name":"@design/tokens"}"#.to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/src/index.scss")
    );
}

#[test]
fn resolves_package_imports_to_relative_style_targets() {
    let available_style_paths = BTreeSet::from(["/fake/workspace/src/theme.css"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "#theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/package.json".to_string(),
            package_json_source: r##"{"imports":{"#theme":{"style":"./src/theme.css"}}}"##
                .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageImportStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/theme.css")
    );
}

#[test]
fn resolves_package_import_array_fallbacks_to_relative_style_targets() {
    let available_style_paths = BTreeSet::from(["/fake/workspace/src/theme.css"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "#theme",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/package.json".to_string(),
            package_json_source:
                r##"{"imports":{"#theme":[{"node":"./src/theme.js"},{"style":"./src/theme.css"}]}}"##
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "packageImportStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/theme.css")
    );
}

#[test]
fn resolves_package_imports_to_external_package_targets() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/dist/theme.css"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "#theme",
        &available_style_paths,
        &[
            OmenaResolverStylePackageManifestV0 {
                package_json_path: "/fake/workspace/package.json".to_string(),
                package_json_source: r##"{"imports":{"#theme":"@design/tokens/theme"}}"##
                    .to_string(),
            },
            OmenaResolverStylePackageManifestV0 {
                package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                    .to_string(),
                package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                    .to_string(),
            },
        ],
    );

    assert_eq!(resolution.resolution_kind, "packageImportStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
}

#[test]
fn resolves_package_import_patterns_to_external_package_targets() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/node_modules/@design/tokens/dist/themes/dark.css"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "#theme/dark",
        &available_style_paths,
        &[
            OmenaResolverStylePackageManifestV0 {
                package_json_path: "/fake/workspace/package.json".to_string(),
                package_json_source: r##"{"imports":{"#theme/*":"@design/tokens/themes/*"}}"##
                    .to_string(),
            },
            OmenaResolverStylePackageManifestV0 {
                package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                    .to_string(),
                package_json_source:
                    r#"{"exports":{"./themes/*":{"style":"./dist/themes/*.css"}}}"#.to_string(),
            },
        ],
    );

    assert_eq!(resolution.resolution_kind, "packageImportStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/themes/dark.css")
    );
}

#[test]
fn resolves_package_import_patterns_by_specificity() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/node_modules/@design/broad/dist/dark/button.css",
        "/fake/workspace/node_modules/@design/specific/dist/button.css",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "#theme/dark/button",
        &available_style_paths,
        &[
            OmenaResolverStylePackageManifestV0 {
                package_json_path: "/fake/workspace/package.json".to_string(),
                package_json_source:
                    r##"{"imports":{"#theme/*":"@design/broad/*","#theme/dark/*":"@design/specific/*"}}"##
                        .to_string(),
            },
            OmenaResolverStylePackageManifestV0 {
                package_json_path: "/fake/workspace/node_modules/@design/broad/package.json"
                    .to_string(),
                package_json_source: r#"{"exports":{"./*":{"style":"./dist/*.css"}}}"#
                    .to_string(),
            },
            OmenaResolverStylePackageManifestV0 {
                package_json_path: "/fake/workspace/node_modules/@design/specific/package.json"
                    .to_string(),
                package_json_source: r#"{"exports":{"./*":{"style":"./dist/*.css"}}}"#
                    .to_string(),
            },
        ],
    );

    assert_eq!(resolution.resolution_kind, "packageImportStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/specific/dist/button.css")
    );
}

#[test]
fn package_import_null_exact_entry_blocks_pattern_fallback() {
    let available_style_paths = BTreeSet::from(["/fake/workspace/src/fallback/private.css"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "#theme/private",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/package.json".to_string(),
            package_json_source:
                r##"{"imports":{"#theme/private":null,"#theme/*":{"style":"./src/fallback/*.css"}}}"##
                    .to_string(),
        }],
    );

    assert_eq!(resolution.resolution_kind, "unresolved");
    assert!(resolution.resolved_style_path.is_none());
    assert!(
        !resolution
            .candidates
            .contains(&"/fake/workspace/src/fallback/private.css".to_string())
    );
}

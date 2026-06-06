use super::*;
use crate::summarize_omena_resolver_style_resolution_policy_v0;

#[test]
fn style_resolution_policy_reports_ordered_candidate_contract() {
    let policy = summarize_omena_resolver_style_resolution_policy_v0();

    assert_eq!(policy.product, "omena-resolver.style-resolution-policy");
    assert_eq!(policy.candidate_strategy, "orderedFirstExistingCandidate");
    assert_eq!(policy.network_access, "neverFetch");
    assert_eq!(
        policy.steps.iter().map(|step| step.key).collect::<Vec<_>>(),
        vec![
            "externalUrlBoundary",
            "bundlerPathMapping",
            "tsconfigPathMapping",
            "sassPkgImporter",
            "fileRelativeOrAbsolute",
            "packageManifestSubpath",
            "nodePackageFallback",
            "sassLoadPathRoot",
        ]
    );
    assert!(policy.ready_surfaces.contains(&"resolutionPolicyReport"));
}

#[test]
fn resolves_bundler_path_mapped_style_modules_before_tsconfig_paths() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/src/bundler/Button.module.scss",
        "/fake/workspace/src/tsconfig/Button.module.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_path_mappings(
        "/fake/workspace/src/App.module.scss",
        "@styles/Button",
        &available_style_paths,
        &[],
        &[OmenaResolverBundlerPathAliasMappingV0 {
            pattern: "@styles".to_string(),
            target_path: "/fake/workspace/src/bundler".to_string(),
        }],
        &[OmenaResolverTsconfigPathMappingV0 {
            base_path: "/fake/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/tsconfig/*".to_string()],
        }],
    );

    assert_eq!(resolution.resolution_kind, "bundlerPathStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/bundler/Button.module.scss")
    );
}

#[test]
fn resolves_bundler_path_mappings_by_first_match_not_longest_match() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/src/first/deep/Button.module.scss",
        "/fake/workspace/src/second/Button.module.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_path_mappings(
        "/fake/workspace/src/App.module.scss",
        "@theme/deep/Button",
        &available_style_paths,
        &[],
        &[
            OmenaResolverBundlerPathAliasMappingV0 {
                pattern: "@theme".to_string(),
                target_path: "/fake/workspace/src/first".to_string(),
            },
            OmenaResolverBundlerPathAliasMappingV0 {
                pattern: "@theme/deep".to_string(),
                target_path: "/fake/workspace/src/second".to_string(),
            },
        ],
        &[],
    );

    assert_eq!(resolution.resolution_kind, "bundlerPathStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/first/deep/Button.module.scss")
    );
    assert!(
        !resolution
            .candidates
            .contains(&"/fake/workspace/src/second/Button.module.scss".to_string())
    );
}

#[test]
fn resolves_bundler_exact_aliases_without_prefix_matching() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/src/exact/theme.module.scss",
        "/fake/workspace/src/prefix/theme/Button.module.scss",
    ]);
    let exact_resolution = summarize_omena_resolver_style_module_resolution_with_path_mappings(
        "/fake/workspace/src/App.module.scss",
        "@theme",
        &available_style_paths,
        &[],
        &[OmenaResolverBundlerPathAliasMappingV0 {
            pattern: "@theme$".to_string(),
            target_path: "/fake/workspace/src/exact/theme.module.scss".to_string(),
        }],
        &[],
    );
    let prefix_resolution = summarize_omena_resolver_style_module_resolution_with_path_mappings(
        "/fake/workspace/src/App.module.scss",
        "@theme/Button",
        &available_style_paths,
        &[],
        &[OmenaResolverBundlerPathAliasMappingV0 {
            pattern: "@theme$".to_string(),
            target_path: "/fake/workspace/src/exact/theme.module.scss".to_string(),
        }],
        &[],
    );

    assert_eq!(
        exact_resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/exact/theme.module.scss")
    );
    assert_eq!(exact_resolution.resolution_kind, "bundlerPathStyleModule");
    assert_eq!(prefix_resolution.resolution_kind, "unresolved");
    assert!(prefix_resolution.resolved_style_path.is_none());
    assert!(
        !prefix_resolution
            .candidates
            .contains(&"/fake/workspace/src/exact/theme.module.scss".to_string())
    );
}

#[test]
fn summarizes_specifier_runtime_with_bundler_path_aliases() {
    let available_style_paths = BTreeSet::from(["/fake/workspace/src/bundler/Button.module.scss"]);
    let runtime = summarize_omena_resolver_specifier_resolution_runtime_with_path_mappings(
        "/fake/workspace/src/App.module.scss",
        &["@styles/Button".to_string()],
        &available_style_paths,
        &[],
        &[OmenaResolverBundlerPathAliasMappingV0 {
            pattern: "@styles".to_string(),
            target_path: "/fake/workspace/src/bundler".to_string(),
        }],
        &[],
    );

    assert!(runtime.ready_surfaces.contains(&"bundlerPathAliasMapping"));
    assert_eq!(runtime.resolved_specifier_count, 1);
    assert_eq!(runtime.entries[0].resolution_kind, "bundlerPathStyleModule");
}

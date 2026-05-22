use super::*;

#[test]
fn resolves_tsconfig_path_mapped_style_modules() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/src/styles/Button.module.scss",
        "/fake/workspace/src/styles/_Theme.module.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
        "/fake/workspace/src/components/App.tsx",
        "@styles/Button",
        &available_style_paths,
        &[],
        &[OmenaResolverTsconfigPathMappingV0 {
            base_path: "/fake/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/styles/*".to_string()],
        }],
    );

    assert_eq!(resolution.product, "omena-resolver.style-module-resolution");
    assert_eq!(resolution.resolution_kind, "tsconfigPathStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/styles/Button.module.scss")
    );
    assert!(
        resolution
            .candidates
            .contains(&"/fake/workspace/src/styles/Button.module.scss".to_string())
    );
}

#[test]
fn resolves_tsconfig_exact_path_mapping_before_wildcards() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/src/exact/Button.module.scss",
        "/fake/workspace/src/wildcard/Button.module.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
        "/fake/workspace/src/components/App.tsx",
        "@styles/Button",
        &available_style_paths,
        &[],
        &[
            OmenaResolverTsconfigPathMappingV0 {
                base_path: "/fake/workspace".to_string(),
                pattern: "@styles/*".to_string(),
                target_patterns: vec!["src/wildcard/*".to_string()],
            },
            OmenaResolverTsconfigPathMappingV0 {
                base_path: "/fake/workspace".to_string(),
                pattern: "@styles/Button".to_string(),
                target_patterns: vec!["src/exact/Button".to_string()],
            },
        ],
    );

    assert_eq!(resolution.resolution_kind, "tsconfigPathStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/exact/Button.module.scss")
    );
}

#[test]
fn resolves_tsconfig_longest_prefix_mapping_independent_of_order() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/src/broad/Button.module.scss",
        "/fake/workspace/src/styles/Button.module.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
        "/fake/workspace/src/components/App.tsx",
        "@/styles/Button",
        &available_style_paths,
        &[],
        &[
            OmenaResolverTsconfigPathMappingV0 {
                base_path: "/fake/workspace".to_string(),
                pattern: "@/*".to_string(),
                target_patterns: vec!["src/broad/*".to_string()],
            },
            OmenaResolverTsconfigPathMappingV0 {
                base_path: "/fake/workspace".to_string(),
                pattern: "@/styles/*".to_string(),
                target_patterns: vec!["src/styles/*".to_string()],
            },
        ],
    );

    assert_eq!(resolution.resolution_kind, "tsconfigPathStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/styles/Button.module.scss")
    );
}

#[test]
fn does_not_fallback_to_less_specific_tsconfig_mapping() {
    let available_style_paths = BTreeSet::from(["/fake/workspace/src/broad/Button.module.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
        "/fake/workspace/src/components/App.tsx",
        "@/styles/Button",
        &available_style_paths,
        &[],
        &[
            OmenaResolverTsconfigPathMappingV0 {
                base_path: "/fake/workspace".to_string(),
                pattern: "@/*".to_string(),
                target_patterns: vec!["src/broad/*".to_string()],
            },
            OmenaResolverTsconfigPathMappingV0 {
                base_path: "/fake/workspace".to_string(),
                pattern: "@/styles/*".to_string(),
                target_patterns: vec!["src/missing/*".to_string()],
            },
        ],
    );

    assert_eq!(resolution.resolution_kind, "unresolved");
    assert!(resolution.resolved_style_path.is_none());
    assert!(
        resolution
            .candidates
            .contains(&"/fake/workspace/src/missing/Button.module.scss".to_string())
    );
    assert!(
        !resolution
            .candidates
            .contains(&"/fake/workspace/src/broad/styles/Button.module.scss".to_string())
    );
}

#[test]
fn resolves_tsconfig_path_mapped_sass_partials() {
    let available_style_paths = BTreeSet::from(["/fake/workspace/src/styles/_Theme.module.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
        "/fake/workspace/src/components/App.tsx",
        "@styles/Theme",
        &available_style_paths,
        &[],
        &[OmenaResolverTsconfigPathMappingV0 {
            base_path: "/fake/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/styles/*".to_string()],
        }],
    );

    assert_eq!(resolution.resolution_kind, "tsconfigPathStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/styles/_Theme.module.scss")
    );
}

#[test]
fn resolves_percent_encoded_tsconfig_targets() {
    let available_style_paths =
        BTreeSet::from(["/fake/workspace/src/styles/Brand Tokens.module.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
        "/fake/workspace/src/components/App.tsx",
        "@styles/Brand%20Tokens",
        &available_style_paths,
        &[],
        &[OmenaResolverTsconfigPathMappingV0 {
            base_path: "/fake/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/styles/*".to_string()],
        }],
    );

    assert_eq!(resolution.resolution_kind, "tsconfigPathStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/styles/Brand Tokens.module.scss")
    );
}

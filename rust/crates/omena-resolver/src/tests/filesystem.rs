use super::*;

#[cfg(unix)]
#[test]
fn resolves_style_modules_by_canonical_filesystem_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_resolver_style_identity")?;
    let real_src = root.join("real/src");
    let link_src = root.join("linked-src");
    fs::create_dir_all(real_src.as_path())?;
    let app = link_src.join("App.module.scss");
    let real_tokens = real_src.join("_tokens.scss");
    fs::write(real_tokens.as_path(), "$brand: red;")?;
    std::os::unix::fs::symlink(real_src.as_path(), link_src.as_path())?;

    let app_text = app.to_string_lossy().to_string();
    let real_tokens_text = real_tokens.to_string_lossy().to_string();
    let available_style_paths = BTreeSet::from([real_tokens_text.as_str()]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        app_text.as_str(),
        "./tokens",
        &available_style_paths,
        &[],
    );

    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some(real_tokens_text.as_str())
    );
    assert_eq!(resolution.resolution_kind, "relativeStyleModule");
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[cfg(unix)]
#[test]
fn summarizes_package_symlink_chain_for_style_module_resolution()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_resolver_package_symlink_chain")?;
    let source = root.join("src/App.module.scss");
    let real_package = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
    let linked_scope = root.join("node_modules/@design");
    let linked_package = linked_scope.join("tokens");
    let style = real_package.join("src/index.scss");
    fs::create_dir_all(
        source
            .parent()
            .ok_or_else(|| std::io::Error::other("source"))?,
    )?;
    fs::create_dir_all(
        style
            .parent()
            .ok_or_else(|| std::io::Error::other("style"))?,
    )?;
    fs::create_dir_all(linked_scope.as_path())?;
    fs::write(source.as_path(), r#"@use "@design/tokens" as tokens;"#)?;
    fs::write(
        real_package.join("package.json"),
        r#"{"sass":"src/index.scss"}"#,
    )?;
    fs::write(style.as_path(), "$brand: #fff;")?;
    std::os::unix::fs::symlink(real_package.as_path(), linked_package.as_path())?;

    let source_text = source.to_string_lossy().to_string();
    let style_text = style.to_string_lossy().to_string();
    let linked_package_json = linked_package.join("package.json");
    let linked_package_json_text = linked_package_json.to_string_lossy().to_string();
    let linked_style_text = linked_package
        .join("src/index.scss")
        .to_string_lossy()
        .to_string();
    let linked_package_text = linked_package.to_string_lossy().to_string();
    let real_package_text = real_package.to_string_lossy().to_string();
    let available_style_paths = BTreeSet::from([style_text.as_str()]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        source_text.as_str(),
        "@design/tokens",
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: linked_package_json_text,
            package_json_source: r#"{"sass":"src/index.scss"}"#.to_string(),
        }],
    );

    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some(style_text.as_str()),
        "{resolution:?}",
    );
    assert_eq!(resolution.resolution_kind, "packageStyleModule");
    assert_eq!(
        resolution.symlink_chain.product,
        "omena-resolver.symlink-chain-inspection"
    );
    assert!(resolution.symlink_chain.link_count > 0, "{resolution:?}");
    assert_eq!(
        resolution.symlink_chain.requested_path, linked_style_text,
        "{resolution:?}",
    );
    let link = resolution
        .symlink_chain
        .links
        .iter()
        .find(|link| link.link_path == linked_package_text)
        .ok_or_else(|| std::io::Error::other("missing symlink link"))?;
    assert_eq!(link.target_path, real_package_text, "{resolution:?}");
    assert!(link.target_was_absolute, "{resolution:?}");
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[cfg(unix)]
#[test]
fn memoizes_style_identity_canonicalize_until_generation_invalidation()
-> Result<(), Box<dyn std::error::Error>> {
    let _counter_guard = style_identity_counter_test_guard()?;
    let root = temp_dir("omena_resolver_identity_canonicalize_cache")?;
    let style = root.join("src/App.module.scss");
    fs::create_dir_all(
        style
            .parent()
            .ok_or_else(|| std::io::Error::other("style parent"))?,
    )?;
    fs::write(style.as_path(), ".app {}")?;
    let style_text = style.to_string_lossy().to_string();

    reset_omena_resolver_style_identity_cache_for_test();
    let first = canonicalize_omena_resolver_style_identity_path(&style_text);
    let first_count = omena_resolver_style_identity_canonicalize_syscall_count_for_test();
    let second = canonicalize_omena_resolver_style_identity_path(&style_text);
    let second_count = omena_resolver_style_identity_canonicalize_syscall_count_for_test();

    assert_eq!(first, second);
    assert!(first_count > 0, "expected first call to hit the filesystem");
    assert_eq!(
        second_count, first_count,
        "expected repeated identity lookup to reuse the generation cache"
    );

    invalidate_omena_resolver_style_identity_cache();
    let third = canonicalize_omena_resolver_style_identity_path(&style_text);
    let third_count = omena_resolver_style_identity_canonicalize_syscall_count_for_test();

    assert_eq!(third, first);
    assert!(
        third_count > second_count,
        "expected generation invalidation to force a fresh canonicalize"
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[cfg(unix)]
#[test]
fn refreshes_missing_style_identity_after_filesystem_generation_change()
-> Result<(), Box<dyn std::error::Error>> {
    let _counter_guard = style_identity_counter_test_guard()?;
    let root = temp_dir("omena_resolver_identity_create_after_miss")?;
    let real_package = root.join("real-package");
    let linked_package = root.join("node_modules/@design/tokens");
    let linked_parent = linked_package
        .parent()
        .ok_or_else(|| std::io::Error::other("linked package parent"))?;
    fs::create_dir_all(linked_parent)?;

    let linked_package_text = linked_package.to_string_lossy().to_string();
    reset_omena_resolver_style_identity_cache_for_test();
    let missing = canonicalize_omena_resolver_style_identity_path(&linked_package_text);

    assert_eq!(
        missing,
        normalize_style_path(linked_package.clone()),
        "a missing path should fall back to its normalized lexical identity"
    );

    fs::create_dir_all(real_package.as_path())?;
    std::os::unix::fs::symlink(real_package.as_path(), linked_package.as_path())?;
    invalidate_omena_resolver_style_identity_cache();
    let refreshed = canonicalize_omena_resolver_style_identity_path(&linked_package_text);

    assert_eq!(
        refreshed,
        normalize_style_path(fs::canonicalize(real_package.as_path())?),
        "a filesystem generation change must not serve the stale missing-path fallback"
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[cfg(unix)]
#[test]
fn memoizes_symlink_chain_read_link_until_generation_invalidation()
-> Result<(), Box<dyn std::error::Error>> {
    let _counter_guard = style_identity_counter_test_guard()?;
    let root = temp_dir("omena_resolver_identity_read_link_cache")?;
    let real_src = root.join("real/src");
    let link_src = root.join("linked-src");
    let style = link_src.join("App.module.scss");
    fs::create_dir_all(real_src.as_path())?;
    std::os::unix::fs::symlink(real_src.as_path(), link_src.as_path())?;
    let style_text = style.to_string_lossy().to_string();

    reset_omena_resolver_style_identity_cache_for_test();
    let first = inspect_omena_resolver_symlink_chain_v0(&style_text);
    let first_count = omena_resolver_style_identity_read_link_syscall_count_for_test();
    let second = inspect_omena_resolver_symlink_chain_v0(&style_text);
    let second_count = omena_resolver_style_identity_read_link_syscall_count_for_test();

    assert!(
        first
            .links
            .iter()
            .any(|link| link.link_path == normalize_style_path(link_src.clone())),
        "{first:?}"
    );
    assert_eq!(second, first);
    assert!(
        first_count > 0,
        "expected first chain inspection to call read_link"
    );
    assert_eq!(
        second_count, first_count,
        "expected repeated chain inspection to reuse read_link cache entries"
    );

    invalidate_omena_resolver_style_identity_cache();
    let third = inspect_omena_resolver_symlink_chain_v0(&style_text);
    let third_count = omena_resolver_style_identity_read_link_syscall_count_for_test();

    assert_eq!(third, first);
    assert!(
        third_count > second_count,
        "expected generation invalidation to force fresh read_link probes"
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[cfg(unix)]
#[test]
fn precomputed_identity_index_reuses_confirmation_maps() -> Result<(), Box<dyn std::error::Error>> {
    let _counter_guard = style_identity_counter_test_guard()?;
    let root = temp_dir("omena_resolver_confirmation_identity_index")?;
    let real_src = root.join("real/src");
    let link_src = root.join("linked-src");
    let real_style = real_src.join("Button.module.scss");
    let linked_style = link_src.join("Button.module.scss");
    fs::create_dir_all(real_src.as_path())?;
    fs::write(real_style.as_path(), ".button {}")?;
    std::os::unix::fs::symlink(real_src.as_path(), link_src.as_path())?;

    let real_style_text = real_style.to_string_lossy().to_string();
    let linked_style_text = linked_style.to_string_lossy().to_string();
    let available_style_paths = BTreeSet::from([real_style_text.as_str()]);
    let candidates = vec![linked_style_text];

    reset_omena_resolver_style_identity_cache_for_test();
    let identity_index =
        build_omena_resolver_style_module_confirmation_identity_index(&available_style_paths, &[]);
    let build_count = omena_resolver_style_identity_index_build_count_for_test();
    let build_work_count = omena_resolver_style_identity_index_build_work_count_for_test();
    let first = confirm_omena_resolver_style_module_candidate_with_options(
        candidates.as_slice(),
        &available_style_paths,
        &[],
        OmenaResolverStyleModuleConfirmationOptionsV0 {
            identity_index: Some(&identity_index),
            ..OmenaResolverStyleModuleConfirmationOptionsV0::default()
        },
    );
    let second = confirm_omena_resolver_style_module_candidate_with_options(
        candidates.as_slice(),
        &available_style_paths,
        &[],
        OmenaResolverStyleModuleConfirmationOptionsV0 {
            identity_index: Some(&identity_index),
            ..OmenaResolverStyleModuleConfirmationOptionsV0::default()
        },
    );

    assert_eq!(
        first.resolved_style_path.as_deref(),
        Some(real_style_text.as_str())
    );
    assert_eq!(first, second);
    assert_eq!(
        omena_resolver_style_identity_index_build_count_for_test(),
        build_count,
        "precomputed confirmation identity index must avoid per-confirm map rebuilds"
    );
    assert_eq!(
        omena_resolver_style_identity_index_build_work_count_for_test(),
        build_work_count,
        "precomputed confirmation identity index must avoid per-confirm identity-map work"
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn resolves_percent_encoded_style_path_segments() {
    let available_style_paths = BTreeSet::from(["/fake/workspace/src/styles/My Theme.module.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/components/App.tsx",
        "../styles/My%20Theme",
        &available_style_paths,
        &[],
    );

    assert_eq!(resolution.resolution_kind, "relativeStyleModule");
    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/fake/workspace/src/styles/My Theme.module.scss")
    );
    assert!(
        resolution
            .candidates
            .contains(&"/fake/workspace/src/styles/My Theme.module.scss".to_string())
    );
}

#[test]
fn resolves_load_path_rooted_use_against_workspace_root() {
    // RFC-0007-I (#49): a load-path-rooted `@use 'src/scss/design-system.scss'` (dart-sass
    // `--load-path=<pkg-root>`) must join the in-graph target instead of flagging it missing.
    // The consumer lives in a *subdirectory*, so the file-relative join
    // (`/pkg-root/components/src/scss/design-system.scss`) misses and only the load-path-rooted
    // candidate (`/pkg-root/src/scss/design-system.scss`) reaches the target.
    let available_style_paths = BTreeSet::from([
        "/pkg-root/src/scss/design-system.scss",
        "/pkg-root/components/consumer.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
        "/pkg-root/components/consumer.scss",
        "src/scss/design-system.scss",
        &available_style_paths,
        &[],
        &[],
        &[],
        &["/pkg-root"],
    );

    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/pkg-root/src/scss/design-system.scss"),
        "{resolution:?}"
    );
    assert_eq!(resolution.resolution_kind, "loadPathStyleModule");
}

#[test]
fn load_path_rooted_use_remains_unresolved_without_roots() {
    // The same load-path-rooted `@use` from a subdirectory consumer is the FP: without load-path
    // roots the file-relative and bare-package routes never reach the target, so it stays
    // unresolved (this is exactly the firing that #49 reports).
    let available_style_paths = BTreeSet::from([
        "/pkg-root/src/scss/design-system.scss",
        "/pkg-root/components/consumer.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
        "/pkg-root/components/consumer.scss",
        "src/scss/design-system.scss",
        &available_style_paths,
        &[],
        &[],
        &[],
        &[],
    );

    assert!(resolution.resolved_style_path.is_none(), "{resolution:?}");
}

#[test]
fn flags_load_path_rooted_use_with_no_on_disk_candidate() {
    // Over-correction guard: a genuinely missing load-path target (no on-disk/known candidate at
    // any root) MUST still be unresolved so `missingSassSymbol` keeps firing for real gaps.
    let available_style_paths = BTreeSet::from(["/pkg-root/components/consumer.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
        "/pkg-root/components/consumer.scss",
        "src/scss/design-system.scss",
        &available_style_paths,
        &[],
        &[],
        &[],
        &["/pkg-root"],
    );

    assert!(resolution.resolved_style_path.is_none(), "{resolution:?}");
    assert_eq!(resolution.resolution_kind, "unresolved");
}

#[test]
fn load_path_rooting_does_not_shadow_real_external_package_route() {
    // Over-correction guard: a real bare-package `@use '@design/tokens'` must still route through
    // the package resolver (node_modules), not get hijacked by a same-suffix load-path guess.
    let available_style_paths =
        BTreeSet::from(["/pkg-root/node_modules/@design/tokens/src/index.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
        "/pkg-root/src/App.module.scss",
        "@design/tokens",
        &available_style_paths,
        &[],
        &[],
        &[],
        &["/pkg-root"],
    );

    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/pkg-root/node_modules/@design/tokens/src/index.scss"),
        "{resolution:?}"
    );
    assert_eq!(resolution.resolution_kind, "packageStyleModule");
}

#[test]
fn load_path_rooting_keeps_file_relative_route_unchanged() {
    // Over-correction guard / control: `@use './design-system.scss'` (file-relative) must keep
    // resolving as `relativeStyleModule` even with load-path roots configured.
    let available_style_paths = BTreeSet::from([
        "/pkg-root/src/scss/design-system.scss",
        "/pkg-root/src/scss/consumer.scss",
    ]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
        "/pkg-root/src/scss/consumer.scss",
        "./design-system.scss",
        &available_style_paths,
        &[],
        &[],
        &[],
        &["/pkg-root"],
    );

    assert_eq!(
        resolution.resolved_style_path.as_deref(),
        Some("/pkg-root/src/scss/design-system.scss"),
        "{resolution:?}"
    );
    assert_eq!(resolution.resolution_kind, "relativeStyleModule");
}

#[test]
fn load_path_rooting_skips_extensionless_bare_specifiers() {
    // Shape guard: an extensionless single-segment-ish bare specifier (no style extension) is not
    // load-path-shaped, so a same-named on-disk file under a root must NOT be joined via load path
    // (that route belongs to the package resolver). Here the only available path is a guess that a
    // load-path join would have produced; it must stay unresolved.
    let available_style_paths = BTreeSet::from(["/pkg-root/tokens/index.scss"]);
    let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
        "/pkg-root/consumer.scss",
        "tokens/index",
        &available_style_paths,
        &[],
        &[],
        &[],
        &["/pkg-root"],
    );

    // `tokens/index` (no style extension) is not load-path-shaped, so the load-path candidate
    // class never runs; it falls through the bare-package route, which does not invent
    // `/pkg-root/tokens/index.scss`.
    assert!(
        resolution.resolution_kind != "loadPathStyleModule",
        "{resolution:?}"
    );
}

#[test]
fn ignores_external_style_module_sources() {
    let available_style_paths = BTreeSet::new();
    let resolution = summarize_omena_resolver_style_module_resolution(
        "/fake/workspace/src/App.module.scss",
        "sass:map",
        &available_style_paths,
        &[],
    );

    assert_eq!(resolution.resolution_kind, "externalIgnored");
    assert!(resolution.candidates.is_empty());
    assert!(resolution.resolved_style_path.is_none());
}

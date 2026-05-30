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

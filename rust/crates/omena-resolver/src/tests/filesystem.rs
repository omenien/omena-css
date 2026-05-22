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

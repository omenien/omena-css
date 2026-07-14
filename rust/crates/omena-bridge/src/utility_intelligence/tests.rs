use std::{fs, path::Path, time::SystemTime};

use super::*;
use crate::summarize_omena_bridge_source_syntax_index;

#[test]
fn static_tailwind_config_preserves_enumerated_pattern_and_unresolved_parts() {
    let report = summarize_omena_bridge_utility_class_intelligence_for_config(
        Path::new("tailwind.config.ts"),
        r##"export default {
          safelist: ["flex items-center", /^data-/],
          theme: { extend: {
            colors: { brand: { 500: "#123456" } },
            spacing: { 18: "4.5rem" }
          } },
          plugins: [customPlugin],
        }"##,
    );

    let entry = &report.class_value_universes[0];
    assert!(entry.class_names.contains(&"bg-brand-500".to_string()));
    assert!(entry.class_names.contains(&"p-18".to_string()));
    assert!(entry.class_names.contains(&"flex".to_string()));
    assert!(
        entry
            .patterns
            .iter()
            .any(|pattern| pattern.source == "bg-[<value>]")
    );
    assert!(
        entry
            .patterns
            .iter()
            .any(|pattern| pattern.source == "/^data-/")
    );
    assert!(
        entry
            .unresolved
            .iter()
            .any(|item| item.reason == "executable-plugin")
    );
    assert!(
        entry
            .unresolved
            .iter()
            .any(|item| item.reason == "default-theme-not-expanded")
    );
}

#[test]
fn executable_config_is_a_typed_partial_failure() {
    let report = summarize_omena_bridge_utility_class_intelligence_for_config(
        Path::new("uno.config.ts"),
        "export default () => ({ shortcuts: makeShortcuts() })",
    );

    assert_eq!(report.enumerated_class_count(), 0);
    assert_eq!(report.pattern_count(), 0);
    assert_eq!(report.unresolved_count(), 1);
    assert_eq!(
        report.class_value_universes[0].unresolved[0].reason,
        "config-export-unresolved"
    );
}

#[test]
fn static_uno_config_collects_shortcuts_and_rule_patterns() {
    let report = summarize_omena_bridge_utility_class_intelligence_for_config(
        Path::new("uno.config.ts"),
        r#"export default {
          shortcuts: { button: "px-4 py-2" },
          rules: [
            [/^icon-(.+)$/, ([, name]) => ({ "--icon": name })],
            ["card", { display: "grid" }],
          ],
          presets: [],
        }"#,
    );

    let entry = &report.class_value_universes[0];
    assert!(entry.class_names.contains(&"button".to_string()));
    assert!(
        entry
            .patterns
            .iter()
            .any(|pattern| pattern.source == "/^icon-(.+)$/")
    );
    assert!(
        entry
            .patterns
            .iter()
            .any(|pattern| pattern.source == "card")
    );
    assert!(entry.unresolved.is_empty(), "{:?}", entry.unresolved);
}

#[test]
fn uno_presets_keep_unlisted_utilities_indeterminate() {
    let report = summarize_omena_bridge_utility_class_intelligence_for_config(
        Path::new("uno.config.ts"),
        "export default { presets: [presetUno()] }",
    );

    assert!(
        report
            .unresolved()
            .any(|item| item.reason == "presets-not-expanded")
    );
    let signal = classify_omena_bridge_utility_class(&report, "p-4");
    assert_eq!(
        signal.membership,
        UtilityClassMembershipKindV0::Indeterminate
    );
    assert!(!signal.undefined_class_signal);
}

#[test]
fn utility_membership_fails_closed_when_config_is_partial() {
    let mut report = summarize_omena_bridge_utility_class_intelligence_for_config(
        Path::new("tailwind.config.ts"),
        r##"export default { theme: { extend: { colors: { brand: "#123" } } } }"##,
    );
    let enumerated = classify_omena_bridge_utility_class(&report, "bg-brand");
    let pattern = classify_omena_bridge_utility_class(&report, "bg-[color:var(--brand)]");
    let indeterminate = classify_omena_bridge_utility_class(&report, "plugin-generated-class");
    assert_eq!(
        enumerated.membership,
        UtilityClassMembershipKindV0::Enumerated
    );
    assert_eq!(pattern.membership, UtilityClassMembershipKindV0::Pattern);
    assert_eq!(
        indeterminate.membership,
        UtilityClassMembershipKindV0::Indeterminate
    );
    assert!(!indeterminate.undefined_class_signal);

    report.class_value_universes[0].unresolved.clear();
    let outside = classify_omena_bridge_utility_class(&report, "definitely-outside");
    assert_eq!(outside.membership, UtilityClassMembershipKindV0::Outside);
    assert!(outside.undefined_class_signal);
}

#[test]
fn config_population_attaches_static_source_tokens_to_existing_provider_plane() {
    let source = r#"export function Card() { return <div className="flex bg-brand p-18" />; }"#;
    let mut index = summarize_omena_bridge_source_syntax_index(source, Vec::new(), Vec::new());
    let report = summarize_omena_bridge_utility_class_intelligence_for_config(
        Path::new("tailwind.config.ts"),
        r##"export default { safelist: ["flex"], theme: { extend: {
          colors: { brand: "#123" }, spacing: { 18: "4.5rem" }
        } } }"##,
    );
    append_omena_bridge_utility_class_intelligence(&mut index, source, &report);

    assert_eq!(
        index
            .domain_class_references
            .iter()
            .filter(|reference| reference.plugin_id == UTILITY_PROVIDER_ID)
            .count(),
        3
    );
    assert!(index.class_value_universes.iter().any(|entry| {
        entry.plugin_id == UTILITY_PROVIDER_ID
            && entry.class_names.contains(&"bg-brand".to_string())
    }));
}

#[test]
fn explicit_config_path_wins_over_discovery() -> Result<(), String> {
    let suffix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let root = std::env::temp_dir().join(format!("omena-utility-config-{suffix}"));
    fs::create_dir_all(root.join("configs")).map_err(|error| error.to_string())?;
    fs::write(
        root.join("tailwind.config.js"),
        "module.exports = { safelist: ['discovered'] }",
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        root.join("configs/custom.ts"),
        "export default { safelist: ['explicit'] }",
    )
    .map_err(|error| error.to_string())?;

    let report = load_omena_bridge_workspace_utility_class_intelligence(
        root.as_path(),
        Some(Path::new("configs/custom.ts")),
    );
    assert_eq!(
        report.config_paths,
        vec![root.join("configs/custom.ts").display().to_string()]
    );
    assert_eq!(
        report.class_value_universes[0].class_names,
        vec!["explicit"]
    );
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

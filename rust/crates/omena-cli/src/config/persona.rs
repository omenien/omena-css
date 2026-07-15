pub(super) const PERSONA_PRESET_IDS: &[&str] = &[
    "workspace-maintenance",
    "design-system-governance",
    "build-integration",
    "migration-safety",
    "assurance-gates",
    "semantic-research",
];

pub(super) fn persona_preset_source(id: &str) -> Option<&'static str> {
    if !PERSONA_PRESET_IDS.contains(&id) {
        return None;
    }

    match id {
        "workspace-maintenance" => Some(include_str!(
            "../../persona-presets/workspace-maintenance.toml"
        )),
        "design-system-governance" => Some(include_str!(
            "../../persona-presets/design-system-governance.toml"
        )),
        "build-integration" => Some(include_str!("../../persona-presets/build-integration.toml")),
        "migration-safety" => Some(include_str!("../../persona-presets/migration-safety.toml")),
        "assurance-gates" => Some(include_str!("../../persona-presets/assurance-gates.toml")),
        "semantic-research" => Some(include_str!("../../persona-presets/semantic-research.toml")),
        _ => unreachable!("the persona roster and embedded source match must stay aligned"),
    }
}

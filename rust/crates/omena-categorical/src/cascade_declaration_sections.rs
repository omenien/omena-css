use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeDeclarationSectionMapV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    #[deprecated(
        since = "0.4.0",
        note = "use section_map_id(); removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    pub sheaf_id: String,
    pub declaration_count: usize,
    pub restriction_count: usize,
}

impl CascadeDeclarationSectionMapV0 {
    #[allow(deprecated)]
    pub fn section_map_id(&self) -> &str {
        cascade_declaration_section_map_id_from_legacy_field_v0(self)
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn cascade_declaration_section_map_id_from_legacy_field_v0(
    section_map: &CascadeDeclarationSectionMapV0,
) -> &str {
    section_map.sheaf_id.as_str()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeRestrictionRecordV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub from_object_id: String,
    pub to_object_id: String,
    pub preserves_cascade_key_order: bool,
}

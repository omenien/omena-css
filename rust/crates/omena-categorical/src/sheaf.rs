use serde::Serialize;

#[deprecated(
    since = "0.4.0",
    note = "use CascadeDeclarationSectionMapV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeDeclarationSheafV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub sheaf_id: String,
    pub declaration_count: usize,
    pub restriction_count: usize,
}

#[deprecated(
    since = "0.4.0",
    note = "use CascadeRestrictionRecordV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestrictionMorphismV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub from_object_id: String,
    pub to_object_id: String,
    pub preserves_cascade_key_order: bool,
}

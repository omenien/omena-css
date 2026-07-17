//! CSS property metadata lookups (inheritance, initial values) over the generated authority.

use crate::property_metadata_idl_generated::{
    CSS_PROPERTY_METADATA_RECORDS_V1, CssPropertyMetadataRecordStaticV1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssPropertyInitialValueV0 {
    Literal(&'static str),
    GuaranteedInvalid,
}

pub fn css_property_metadata_for_property(
    property: &str,
) -> Option<&'static CssPropertyMetadataRecordStaticV1> {
    CSS_PROPERTY_METADATA_RECORDS_V1
        .binary_search_by_key(&property, |record| record.canonical_name)
        .ok()
        .map(|index| &CSS_PROPERTY_METADATA_RECORDS_V1[index])
}

pub fn css_property_is_inherited(property: &str) -> bool {
    if property.starts_with("--") {
        return true;
    }

    css_property_metadata_for_property(property)
        .and_then(|record| record.inherited)
        .unwrap_or(false)
}

pub fn css_property_initial_value(property: &str) -> CssPropertyInitialValueV0 {
    if property.starts_with("--") {
        return CssPropertyInitialValueV0::GuaranteedInvalid;
    }

    css_property_metadata_for_property(property)
        .and_then(|record| record.initial_value)
        .map(CssPropertyInitialValueV0::Literal)
        .unwrap_or(CssPropertyInitialValueV0::Literal("initial"))
}

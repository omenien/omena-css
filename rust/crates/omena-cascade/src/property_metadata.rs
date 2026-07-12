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
        .iter()
        .find(|record| record.canonical_name == property)
}

pub fn css_property_is_inherited(property: &str) -> bool {
    if property.starts_with("--") {
        return true;
    }

    css_property_metadata_for_property(property).is_some_and(|record| record.inherited)
}

pub fn css_property_initial_value(property: &str) -> CssPropertyInitialValueV0 {
    if property.starts_with("--") {
        return CssPropertyInitialValueV0::GuaranteedInvalid;
    }

    CssPropertyInitialValueV0::Literal(
        css_property_metadata_for_property(property)
            .map(|record| record.initial_value)
            .unwrap_or("initial"),
    )
}

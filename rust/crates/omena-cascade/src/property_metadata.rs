//! CSS property metadata lookups (inheritance, initial values) over the generated authority.

use omena_syntax::ident::{CanonicalStandardPropertyNameV0, PropertyNameV0};

use crate::property_metadata_idl_generated::{
    CSS_PROPERTY_METADATA_RECORDS_V1, CssPropertyMetadataRecordStaticV1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssPropertyInitialValueV0 {
    Literal(&'static str),
    GuaranteedInvalid,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssPropertyInheritanceV0 {
    Inherited,
    NotInherited,
    Unknown,
}

pub fn css_property_metadata_for_property(
    property: &str,
) -> Option<&'static CssPropertyMetadataRecordStaticV1> {
    let property = PropertyNameV0::from_authored(property);
    css_property_metadata_for_property_in_records(
        property.as_standard_key()?,
        CSS_PROPERTY_METADATA_RECORDS_V1,
    )
}

#[doc(hidden)]
pub fn css_property_metadata_for_property_in_records(
    property: &CanonicalStandardPropertyNameV0,
    records: &'static [CssPropertyMetadataRecordStaticV1],
) -> Option<&'static CssPropertyMetadataRecordStaticV1> {
    records
        .binary_search_by_key(&property.as_str(), |record| record.canonical_name)
        .ok()
        .map(|index| &records[index])
}

pub fn css_property_is_inherited(property: &str) -> CssPropertyInheritanceV0 {
    let property = PropertyNameV0::from_authored(property);
    let Some(property) = property.as_standard_key() else {
        return CssPropertyInheritanceV0::Inherited;
    };

    match css_property_metadata_for_property_in_records(property, CSS_PROPERTY_METADATA_RECORDS_V1)
        .and_then(|record| record.inherited)
    {
        Some(true) => CssPropertyInheritanceV0::Inherited,
        Some(false) => CssPropertyInheritanceV0::NotInherited,
        None => CssPropertyInheritanceV0::Unknown,
    }
}

pub fn css_property_initial_value(property: &str) -> CssPropertyInitialValueV0 {
    let property = PropertyNameV0::from_authored(property);
    let Some(property) = property.as_standard_key() else {
        return CssPropertyInitialValueV0::GuaranteedInvalid;
    };

    css_property_metadata_for_property_in_records(property, CSS_PROPERTY_METADATA_RECORDS_V1)
        .and_then(|record| record.initial_value)
        .map(CssPropertyInitialValueV0::Literal)
        .unwrap_or(CssPropertyInitialValueV0::Unknown)
}

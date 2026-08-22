use omena_syntax::ident::{
    CanonicalCustomPropertyNameV0, ClassNameV0, PropertyNameV0, is_css_name_continue,
};

pub(crate) fn canonical_custom_property_key(name: &str) -> Option<CanonicalCustomPropertyNameV0> {
    let property_key = PropertyNameV0::from_authored(name).as_custom_key()?;
    (property_key.as_str().len() > 2).then_some(property_key)
}

pub(crate) fn normalize_custom_property_name(name: &str) -> Option<&str> {
    let name = name.trim();
    if canonical_custom_property_key(name).is_some() {
        return Some(name);
    }
    None
}

pub(crate) fn css_identifier_text_is_plain(text: &str) -> bool {
    text.chars().all(is_css_name_continue)
}

pub(crate) fn css_identifier_names_match(left: &str, right: &str) -> bool {
    ClassNameV0::new(left).same_as(&ClassNameV0::new(right))
}

//! Compile-time metadata attributes for Omena CSS spec and transform surfaces.
//!
//! These macros intentionally validate only local attribute shape. Cross-item
//! checks such as global pass ordinal continuity belong to the generated
//! manifest/spec-audit layer that consumes these attributes.

use proc_macro::TokenStream;
use std::{collections::BTreeMap, str::FromStr};

/// Attach CSS specification metadata to a syntax or semantic item.
///
/// Supported forms:
///
/// ```ignore
/// #[spec(webref = "css-color/properties/color", priority = "P0")]
/// #[spec(na = "print-margin-descriptor")]
/// ```
#[proc_macro_attribute]
pub fn spec(attr: TokenStream, item: TokenStream) -> TokenStream {
    match validate_spec_attr(attr.to_string().as_str()) {
        Ok(()) => item,
        Err(error) => item_with_compile_error(item, error.as_str()),
    }
}

/// Attach transform-pass metadata to a pass item.
///
/// Supported form:
///
/// ```ignore
/// #[pass(id = "color-compression", ordinal = 5, layer = "value-normalization")]
/// ```
#[proc_macro_attribute]
pub fn pass(attr: TokenStream, item: TokenStream) -> TokenStream {
    match validate_pass_attr(attr.to_string().as_str()) {
        Ok(()) => item,
        Err(error) => item_with_compile_error(item, error.as_str()),
    }
}

fn validate_spec_attr(input: &str) -> Result<(), String> {
    let args = parse_meta_args(input)?;
    let webref = args.get("webref");
    let not_applicable = args.get("na");
    if webref.is_some() == not_applicable.is_some() {
        return Err("spec requires exactly one of `webref` or `na`".to_string());
    }
    if let Some(value) = webref {
        validate_webref(value)?;
        let priority = args
            .get("priority")
            .ok_or_else(|| "spec with `webref` requires `priority`".to_string())?;
        validate_priority(priority)?;
    }
    if let Some(value) = not_applicable {
        validate_not_applicable(value)?;
    }
    reject_unknown_keys(args.keys(), &["webref", "na", "priority", "since"])?;
    Ok(())
}

fn validate_pass_attr(input: &str) -> Result<(), String> {
    let args = parse_meta_args(input)?;
    let id = args
        .get("id")
        .ok_or_else(|| "pass requires `id`".to_string())?;
    let ordinal = args
        .get("ordinal")
        .ok_or_else(|| "pass requires `ordinal`".to_string())?;
    let layer = args
        .get("layer")
        .ok_or_else(|| "pass requires `layer`".to_string())?;
    validate_pass_id(id)?;
    validate_ordinal(ordinal)?;
    validate_layer(layer)?;
    reject_unknown_keys(args.keys(), &["id", "ordinal", "layer", "requires"])?;
    Ok(())
}

fn parse_meta_args(input: &str) -> Result<BTreeMap<String, String>, String> {
    let mut args = BTreeMap::new();
    for segment in split_meta_segments(input) {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once('=') else {
            return Err(format!(
                "metadata argument `{trimmed}` must use `key = value`"
            ));
        };
        let key = raw_key.trim();
        if !is_ident_key(key) {
            return Err(format!("metadata key `{key}` is not supported"));
        }
        if args.contains_key(key) {
            return Err(format!("metadata key `{key}` is duplicated"));
        }
        args.insert(key.to_string(), parse_meta_value(raw_value.trim())?);
    }
    Ok(args)
}

fn split_meta_segments(input: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for char in input.chars() {
        if in_string {
            current.push(char);
            if escaped {
                escaped = false;
            } else if char == '\\' {
                escaped = true;
            } else if char == '"' {
                in_string = false;
            }
            continue;
        }
        match char {
            '"' => {
                in_string = true;
                current.push(char);
            }
            ',' => {
                segments.push(current);
                current = String::new();
            }
            _ => current.push(char),
        }
    }
    segments.push(current);
    segments
}

fn parse_meta_value(raw_value: &str) -> Result<String, String> {
    if raw_value.starts_with('"') {
        if !raw_value.ends_with('"') || raw_value.len() < 2 {
            return Err(format!("metadata string `{raw_value}` is unterminated"));
        }
        return Ok(raw_value[1..raw_value.len() - 1].to_string());
    }
    if raw_value.is_empty() || raw_value.chars().any(char::is_whitespace) {
        return Err(format!(
            "metadata bare value `{raw_value}` is not supported"
        ));
    }
    Ok(raw_value.to_string())
}

fn validate_webref(value: &str) -> Result<(), String> {
    if value.is_empty() || !value.contains('/') || value.chars().any(char::is_whitespace) {
        return Err("spec `webref` must be a non-empty path-like identifier".to_string());
    }
    Ok(())
}

fn validate_not_applicable(value: &str) -> Result<(), String> {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return Err("spec `na` must be a non-empty identifier".to_string());
    }
    Ok(())
}

fn validate_priority(value: &str) -> Result<(), String> {
    match value {
        "P0" | "P1" | "P2" | "P3" => Ok(()),
        _ => Err("spec `priority` must be one of P0, P1, P2, or P3".to_string()),
    }
}

fn validate_pass_id(value: &str) -> Result<(), String> {
    if is_kebab_identifier(value) {
        Ok(())
    } else {
        Err("pass `id` must be a lowercase kebab-case identifier".to_string())
    }
}

fn validate_ordinal(value: &str) -> Result<(), String> {
    if value.parse::<u16>().is_ok() {
        Ok(())
    } else {
        Err("pass `ordinal` must be an unsigned integer".to_string())
    }
}

fn validate_layer(value: &str) -> Result<(), String> {
    if is_kebab_identifier(value) {
        Ok(())
    } else {
        Err("pass `layer` must be a lowercase kebab-case identifier".to_string())
    }
}

fn reject_unknown_keys<'a>(
    keys: impl Iterator<Item = &'a String>,
    allowed: &[&str],
) -> Result<(), String> {
    for key in keys {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("metadata key `{key}` is not supported here"));
        }
    }
    Ok(())
}

fn is_ident_key(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|char| char.is_ascii_alphanumeric() || char == '_')
}

fn is_kebab_identifier(value: &str) -> bool {
    if value.is_empty() || value.starts_with('-') || value.ends_with('-') {
        return false;
    }
    value
        .chars()
        .all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-')
}

fn item_with_compile_error(item: TokenStream, message: &str) -> TokenStream {
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    let compile_error = format!("compile_error!(\"{escaped}\");");
    let mut output =
        TokenStream::from_str(compile_error.as_str()).unwrap_or_else(|_| TokenStream::new());
    output.extend(item);
    output
}

#[cfg(test)]
mod tests {
    use super::{validate_pass_attr, validate_spec_attr};

    fn validation_error(result: Result<(), String>) -> String {
        match result {
            Ok(()) => "validation unexpectedly passed".to_string(),
            Err(error) => error,
        }
    }

    #[test]
    fn accepts_webref_spec_metadata() {
        assert!(
            validate_spec_attr(r#"webref = "css-color/properties/color", priority = "P0""#).is_ok()
        );
    }

    #[test]
    fn accepts_not_applicable_spec_metadata() {
        assert!(validate_spec_attr(r#"na = "print-margin-descriptor""#).is_ok());
    }

    #[test]
    fn rejects_spec_metadata_without_single_source() {
        let error = validation_error(validate_spec_attr(
            r#"webref = "css-color/properties/color", na = "manual", priority = "P0""#,
        ));
        assert!(error.contains("exactly one"));
    }

    #[test]
    fn rejects_webref_spec_without_priority() {
        let error = validation_error(validate_spec_attr(
            r#"webref = "css-color/properties/color""#,
        ));
        assert!(error.contains("priority"));
    }

    #[test]
    fn accepts_pass_metadata() {
        assert!(
            validate_pass_attr(
                r#"id = "color-compression", ordinal = 5, layer = "value-normalization""#
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_non_kebab_pass_id() {
        let error = validation_error(validate_pass_attr(
            r#"id = "ColorCompression", ordinal = 5, layer = "value-normalization""#,
        ));
        assert!(error.contains("kebab-case"));
    }

    #[test]
    fn rejects_non_numeric_pass_ordinal() {
        let error = validation_error(validate_pass_attr(
            r#"id = "color-compression", ordinal = "fifth", layer = "value-normalization""#,
        ));
        assert!(error.contains("ordinal"));
    }
}

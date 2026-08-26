//! Registered custom-property projection into the cascade computed-value model.

use std::collections::BTreeMap;

use omena_cascade::{
    CascadeComputedValueInputV0, CascadeComputedValueResultV0, CascadeRegisteredCustomPropertyV0,
    CascadeRegisteredValueVerdictV0, CascadeValue, CustomPropertyEnv,
    compute_cascade_computed_value,
};
use omena_query_checker_orchestrator::active_omena_checker_custom_property_registrations_v0;
use omena_query_core::{CssValueValidationClassV0, validate_registered_property_value_v0};
use omena_query_transform_runner::parse_static_css_cascade_value;
use omena_syntax::ident::{AuthoredPropertyTextV0, PropertyNameV0};
use serde::Serialize;

use super::cascade_checker::{
    collect_query_checker_cascade_declarations,
    collect_query_checker_custom_property_registrations,
    query_runtime_cascade_declaration_from_input,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryRegisteredCustomPropertyComputedValueV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_uri: String,
    pub selector: String,
    pub property: AuthoredPropertyTextV0,
    pub registration_applied: bool,
    pub registration_projection_complete: bool,
    pub matched_value_count: usize,
    pub unmatched_value_count: usize,
    pub unknown_value_count: usize,
    pub computed_value: CascadeComputedValueResultV0,
}

impl PartialEq for OmenaQueryRegisteredCustomPropertyComputedValueV0 {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.product == other.product
            && self.style_uri == other.style_uri
            && self.selector == other.selector
            && self
                .property
                .to_property_name()
                .same_as(&other.property.to_property_name())
            && self.registration_applied == other.registration_applied
            && self.registration_projection_complete == other.registration_projection_complete
            && self.matched_value_count == other.matched_value_count
            && self.unmatched_value_count == other.unmatched_value_count
            && self.unknown_value_count == other.unknown_value_count
            && self.computed_value == other.computed_value
    }
}

impl Eq for OmenaQueryRegisteredCustomPropertyComputedValueV0 {}

pub fn summarize_omena_query_registered_custom_property_computed_value_v0(
    style_uri: &str,
    source: &str,
    selector: &str,
    property: &str,
    parent_computed_value: Option<CascadeValue>,
) -> OmenaQueryRegisteredCustomPropertyComputedValueV0 {
    let registration_inputs =
        collect_query_checker_custom_property_registrations(style_uri, source);
    let active_registrations =
        active_omena_checker_custom_property_registrations_v0(registration_inputs.as_slice());
    let property_key = PropertyNameV0::canonical_custom_key(property);
    let property_name = PropertyNameV0::from_authored(property);
    let active_registration = active_registrations.get(&property_key);
    let mut verdicts = BTreeMap::new();
    let mut matched_value_count = 0usize;
    let mut unmatched_value_count = 0usize;
    let mut unknown_value_count = 0usize;
    let declarations = collect_query_checker_cascade_declarations(source)
        .into_iter()
        .filter(|declaration| {
            declaration
                .input
                .property
                .to_property_name()
                .same_as(&property_name)
                && declaration.input.selector.as_str() == selector
                && declaration.input.condition_context.is_empty()
        })
        .map(|declaration| {
            let mut cascade_declaration =
                query_runtime_cascade_declaration_from_input(&declaration.input);
            cascade_declaration.value =
                parse_static_css_cascade_value(declaration.input.value.as_str())
                    .unwrap_or(CascadeValue::GuaranteedInvalid);
            if let Some(registration) = active_registration {
                let verdict = match validate_registered_property_value_v0(
                    registration.syntax.as_str(),
                    declaration.input.value.as_str(),
                )
                .class
                {
                    CssValueValidationClassV0::Valid => {
                        matched_value_count += 1;
                        CascadeRegisteredValueVerdictV0::Matched
                    }
                    CssValueValidationClassV0::Invalid => {
                        unmatched_value_count += 1;
                        CascadeRegisteredValueVerdictV0::Unmatched
                    }
                    CssValueValidationClassV0::NotValidatable => {
                        unknown_value_count += 1;
                        CascadeRegisteredValueVerdictV0::Unknown
                    }
                };
                verdicts.insert(cascade_declaration.id.clone(), verdict);
            }
            cascade_declaration
        })
        .collect::<Vec<_>>();

    let (registered_custom_property, registration_projection_complete) = match active_registration {
        Some(registration) => {
            let initial_value = registration
                .initial_value
                .as_deref()
                .map(parse_static_css_cascade_value)
                .unwrap_or(Some(CascadeValue::GuaranteedInvalid));
            match initial_value {
                Some(initial_value) => (
                    Some(CascadeRegisteredCustomPropertyV0 {
                        name: registration.name.clone(),
                        inherits: registration.inherits,
                        initial_value,
                        declaration_value_verdicts: verdicts,
                    }),
                    true,
                ),
                None => (None, false),
            }
        }
        None => (None, true),
    };
    let registration_applied = registered_custom_property.is_some();
    let computed_value = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: AuthoredPropertyTextV0::new(property),
        declarations,
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value,
        registered_custom_property,
        standard_property_value_verdicts: BTreeMap::new(),
    });

    OmenaQueryRegisteredCustomPropertyComputedValueV0 {
        schema_version: "0",
        product: "omena-query.registered-custom-property-computed-value",
        style_uri: style_uri.to_string(),
        selector: selector.to_string(),
        property: AuthoredPropertyTextV0::new(property),
        registration_applied,
        registration_projection_complete,
        matched_value_count,
        unmatched_value_count,
        unknown_value_count,
        computed_value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_cascade::{ComputedCascadeIndeterminateReasonV0, ComputedCascadeValueStatusV0};

    #[test]
    fn registered_computed_value_identity_uses_sealed_property_keys() {
        let plain = summarize_omena_query_registered_custom_property_computed_value_v0(
            "tokens.css",
            "",
            ":root",
            "--foo",
            None,
        );
        let mut escaped = plain.clone();
        escaped.property = AuthoredPropertyTextV0::new(r"--f\6f o");
        escaped.computed_value.property = AuthoredPropertyTextV0::new(r"--f\6f o");
        assert_eq!(plain, escaped);

        let mut different_case = plain.clone();
        different_case.property = AuthoredPropertyTextV0::new("--FOO");
        different_case.computed_value.property = AuthoredPropertyTextV0::new("--FOO");
        assert_ne!(plain, different_case);
    }

    #[test]
    fn registered_properties_use_typed_syntax_inheritance_and_initial_values() {
        let source = r#"
@property --gap {
  syntax: '<length>';
  inherits: false;
  initial-value: 8px;
}
.valid { --gap: 12px; }
.invalid { --gap: red; }
"#;

        let valid = summarize_omena_query_registered_custom_property_computed_value_v0(
            "tokens.css",
            source,
            ".valid",
            "--gap",
            Some(CascadeValue::Literal("16px".to_string())),
        );
        assert!(valid.registration_applied);
        assert!(valid.registration_projection_complete);
        assert_eq!(valid.matched_value_count, 1);
        assert_eq!(valid.unmatched_value_count, 0);
        assert_eq!(
            valid.computed_value.status,
            ComputedCascadeValueStatusV0::Resolved
        );
        assert_eq!(
            valid.computed_value.value,
            CascadeValue::Literal("12px".to_string())
        );

        let invalid = summarize_omena_query_registered_custom_property_computed_value_v0(
            "tokens.css",
            source,
            ".invalid",
            "--gap",
            Some(CascadeValue::Literal("16px".to_string())),
        );
        assert_eq!(invalid.matched_value_count, 0);
        assert_eq!(invalid.unmatched_value_count, 1);
        assert_eq!(
            invalid.computed_value.status,
            ComputedCascadeValueStatusV0::InvalidAtComputedValueTime
        );
        assert_eq!(
            invalid.computed_value.value,
            CascadeValue::Literal("8px".to_string())
        );
        assert!(invalid.computed_value.invalid_at_computed_value_time);

        let absent = summarize_omena_query_registered_custom_property_computed_value_v0(
            "tokens.css",
            source,
            ".absent",
            "--gap",
            Some(CascadeValue::Literal("16px".to_string())),
        );
        assert_eq!(
            absent.computed_value.status,
            ComputedCascadeValueStatusV0::Initial
        );
        assert_eq!(
            absent.computed_value.value,
            CascadeValue::Literal("8px".to_string())
        );
    }

    #[test]
    fn indeterminate_registered_syntax_has_a_typed_reason() {
        let source = r#"
@property --gap {
  syntax: '<length>';
  inherits: false;
  initial-value: 8px;
}
.deferred { --gap: var(--runtime-value); }
"#;

        let result = summarize_omena_query_registered_custom_property_computed_value_v0(
            "tokens.css",
            source,
            ".deferred",
            "--gap",
            None,
        );

        assert_eq!(result.unknown_value_count, 1);
        assert_eq!(
            result.computed_value.status,
            ComputedCascadeValueStatusV0::Indeterminate
        );
        assert_eq!(result.computed_value.value, CascadeValue::Indeterminate);
        assert!(!result.computed_value.invalid_at_computed_value_time);
        assert_eq!(
            result.computed_value.indeterminate_reason,
            Some(ComputedCascadeIndeterminateReasonV0::RegisteredPropertySyntaxIndeterminate)
        );
    }

    #[test]
    fn property_registration_joins_escape_identity_but_not_custom_case() {
        let escaped = summarize_omena_query_registered_custom_property_computed_value_v0(
            "tokens.css",
            r#"
@property --f\6f o {
  syntax: '<length>';
  inherits: false;
  initial-value: 8px;
}
.target { --foo: 12px; }
"#,
            ".target",
            "--foo",
            None,
        );
        assert!(escaped.registration_applied);
        assert_eq!(escaped.matched_value_count, 1);

        let distinct_case = summarize_omena_query_registered_custom_property_computed_value_v0(
            "tokens.css",
            r#"
@property --FOO {
  syntax: '<length>';
  inherits: false;
  initial-value: 8px;
}
.target { --foo: 12px; }
"#,
            ".target",
            "--foo",
            None,
        );
        assert!(!distinct_case.registration_applied);
    }

    #[test]
    fn unregistered_custom_properties_preserve_the_existing_computed_path() {
        let report = summarize_omena_query_registered_custom_property_computed_value_v0(
            "tokens.css",
            ".target { --legacy-gap: 12px; }",
            ".target",
            "--legacy-gap",
            Some(CascadeValue::Literal("16px".to_string())),
        );

        assert!(!report.registration_applied);
        assert!(report.registration_projection_complete);
        assert_eq!(report.matched_value_count, 0);
        assert_eq!(report.unmatched_value_count, 0);
        assert_eq!(report.unknown_value_count, 0);
        assert_eq!(
            report.computed_value.status,
            ComputedCascadeValueStatusV0::Resolved
        );
        assert_eq!(
            report.computed_value.value,
            CascadeValue::Literal("12px".to_string())
        );
        assert_eq!(
            report.computed_value.derivation_steps,
            vec![
                "cascadeWinnerSelected",
                "computedValueResolutionStarted",
                "computedValueResolved",
            ]
        );
    }
}

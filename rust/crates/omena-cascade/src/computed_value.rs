//! Computed-value resolution over cascade winners and custom-property environments.
//!
//! This module owns the seed implementation for turning a cascaded declaration
//! into a computed value witness without hiding indeterminate cascade outcomes.

use omena_syntax::ident::{PropertyNameKindV0, PropertyNameV0};

use crate::{
    CascadeComputedValueInputV0, CascadeComputedValueResultV0, CascadeOutcome,
    CascadeRegisteredCustomPropertyV0, CascadeRegisteredValueVerdictV0,
    CascadeStandardValueVerdictV0, CascadeValue, ComputedCascadeIndeterminateReasonV0,
    ComputedCascadeValueStatusV0, CssPropertyInheritanceV0, CssPropertyInitialValueV0,
    cascade_property, css_property_initial_value, css_property_is_inherited,
    substitute_custom_properties,
};

/// Grammar-authority port used after `var()` substitution has made a standard
/// property value concrete.
pub trait CascadeStandardValueValidatorV0 {
    fn validate_standard_property_value(
        &self,
        property: &str,
        value: &str,
    ) -> CascadeStandardValueVerdictV0;
}

pub fn compute_cascade_computed_value(
    input: CascadeComputedValueInputV0,
) -> CascadeComputedValueResultV0 {
    compute_cascade_computed_value_inner(input, None)
}

pub fn compute_cascade_computed_value_with_standard_value_validator_v0(
    input: CascadeComputedValueInputV0,
    validator: &dyn CascadeStandardValueValidatorV0,
) -> CascadeComputedValueResultV0 {
    compute_cascade_computed_value_inner(input, Some(validator))
}

fn compute_cascade_computed_value_inner(
    input: CascadeComputedValueInputV0,
    standard_value_validator: Option<&dyn CascadeStandardValueValidatorV0>,
) -> CascadeComputedValueResultV0 {
    let CascadeComputedValueInputV0 {
        property,
        declarations,
        custom_property_env,
        parent_computed_value,
        registered_custom_property,
        standard_property_value_verdicts,
    } = input;
    let property_identity = PropertyNameV0::from_authored(&property);
    let registered_custom_property = registered_custom_property.filter(|registration| {
        property_identity.same_as(&PropertyNameV0::custom(&registration.name))
    });
    let outcome = cascade_property(declarations, &property);
    if let Some(result) = computed_value_from_indeterminate_cascade_outcome(&property, &outcome) {
        return result;
    }
    let (
        winner_declaration_id,
        cascaded_value,
        registered_value_verdict,
        standard_value_verdict,
        mut derivation_steps,
    ) = match outcome {
        CascadeOutcome::Definite { winner, .. } => {
            let registered_value_verdict =
                registered_custom_property.as_ref().map(|registration| {
                    registration
                        .declaration_value_verdicts
                        .get(winner.id.as_str())
                        .copied()
                        .unwrap_or(CascadeRegisteredValueVerdictV0::Unknown)
                });
            let standard_value_verdict = (property_identity.kind() == PropertyNameKindV0::Standard)
                .then(|| {
                    standard_property_value_verdicts
                        .get(winner.id.as_str())
                        .copied()
                })
                .flatten();
            (
                Some(winner.id),
                winner.value,
                registered_value_verdict,
                standard_value_verdict,
                vec!["cascadeWinnerSelected", "computedValueResolutionStarted"],
            )
        }
        CascadeOutcome::Inherit => {
            match property_inheritance(&property, registered_custom_property.as_ref()) {
                CssPropertyInheritanceV0::Inherited => (
                    None,
                    CascadeValue::Inherit,
                    None,
                    None,
                    vec!["noCascadeWinner", "inheritanceOrInitialSelected"],
                ),
                CssPropertyInheritanceV0::NotInherited => (
                    None,
                    CascadeValue::Initial,
                    None,
                    None,
                    vec!["noCascadeWinner", "inheritanceOrInitialSelected"],
                ),
                CssPropertyInheritanceV0::Unknown => {
                    return computed_value_from_unknown_metadata(
                            property,
                            None,
                            ComputedCascadeIndeterminateReasonV0::PropertyInheritanceMetadataUnavailable,
                            vec!["noCascadeWinner", "propertyInheritanceMetadataUnavailable"],
                        );
                }
            }
        }
        CascadeOutcome::RankedSet(_) | CascadeOutcome::Top => {
            unreachable!("indeterminate cascade outcomes return before winner resolution")
        }
    };

    match registered_value_verdict {
        Some(CascadeRegisteredValueVerdictV0::Unmatched) => {
            derivation_steps.push("registeredPropertySyntaxUnmatched");
            derivation_steps.push("invalidAtComputedValueTimeFallsBackAsUnset");
            return computed_value_from_unset(
                property,
                winner_declaration_id,
                parent_computed_value,
                true,
                derivation_steps,
                registered_custom_property.as_ref(),
            );
        }
        Some(CascadeRegisteredValueVerdictV0::Unknown) => {
            derivation_steps.push("registeredPropertySyntaxIndeterminate");
            return computed_value_from_unknown_metadata(
                property,
                winner_declaration_id,
                ComputedCascadeIndeterminateReasonV0::RegisteredPropertySyntaxIndeterminate,
                derivation_steps,
            );
        }
        Some(CascadeRegisteredValueVerdictV0::Matched) | None => {}
    }

    let mut standard_syntax_deferred_by_var_reference = false;
    let mut standard_syntax_verdict_unavailable = false;
    match standard_value_verdict {
        Some(CascadeStandardValueVerdictV0::Unmatched) => {
            derivation_steps.push("standardPropertySyntaxUnmatched");
            derivation_steps.push("invalidAtComputedValueTimeFallsBackAsUnset");
            return computed_value_from_unset(
                property,
                winner_declaration_id,
                parent_computed_value,
                true,
                derivation_steps,
                registered_custom_property.as_ref(),
            );
        }
        Some(CascadeStandardValueVerdictV0::Unknown)
            if cascade_value_contains_var_reference(&cascaded_value) =>
        {
            derivation_steps.push("standardPropertySyntaxDeferredByVarReference");
            standard_syntax_deferred_by_var_reference = true;
        }
        Some(CascadeStandardValueVerdictV0::Unknown) => {
            derivation_steps.push("standardPropertySyntaxIndeterminate");
            return computed_value_from_unknown_metadata(
                property,
                winner_declaration_id,
                ComputedCascadeIndeterminateReasonV0::StandardPropertySyntaxIndeterminate,
                derivation_steps,
            );
        }
        Some(CascadeStandardValueVerdictV0::Matched) => {
            derivation_steps.push("standardPropertySyntaxMatched");
        }
        None if property_identity.kind() == PropertyNameKindV0::Standard => {
            derivation_steps.push("standardPropertySyntaxVerdictUnavailable");
            standard_syntax_verdict_unavailable = true;
        }
        None => {}
    }

    let substituted_value = substitute_custom_properties(&cascaded_value, &custom_property_env);
    if substituted_value == CascadeValue::GuaranteedInvalid {
        derivation_steps.push("substitutionProducedGuaranteedInvalid");
        derivation_steps.push("invalidAtComputedValueTimeFallsBackAsUnset");
        return computed_value_from_unset(
            property,
            winner_declaration_id,
            parent_computed_value,
            true,
            derivation_steps,
            registered_custom_property.as_ref(),
        );
    }

    if standard_syntax_deferred_by_var_reference
        && !matches!(
            substituted_value,
            CascadeValue::Initial | CascadeValue::Inherit | CascadeValue::Unset
        )
    {
        let post_substitution_verdict = standard_value_validator
            .and_then(|validator| {
                render_substituted_standard_value(&substituted_value).map(|value| {
                    validator.validate_standard_property_value(&property, value.as_str())
                })
            })
            .unwrap_or(CascadeStandardValueVerdictV0::Unknown);
        match post_substitution_verdict {
            CascadeStandardValueVerdictV0::Matched => {
                derivation_steps.push("postSubstitutionStandardPropertySyntaxMatched");
            }
            CascadeStandardValueVerdictV0::Unmatched => {
                derivation_steps.push("postSubstitutionStandardPropertySyntaxUnmatched");
                derivation_steps.push("invalidAtComputedValueTimeFallsBackAsUnset");
                return computed_value_from_unset(
                    property,
                    winner_declaration_id,
                    parent_computed_value,
                    true,
                    derivation_steps,
                    registered_custom_property.as_ref(),
                );
            }
            CascadeStandardValueVerdictV0::Unknown => {
                derivation_steps.push("postSubstitutionStandardPropertySyntaxIndeterminate");
                return computed_value_from_unknown_metadata(
                    property,
                    winner_declaration_id,
                    ComputedCascadeIndeterminateReasonV0::StandardPropertySyntaxIndeterminate,
                    derivation_steps,
                );
            }
        }
    }

    if standard_syntax_verdict_unavailable
        && !matches!(
            substituted_value,
            CascadeValue::Initial | CascadeValue::Inherit | CascadeValue::Unset
        )
    {
        return computed_value_from_unknown_metadata(
            property,
            winner_declaration_id,
            ComputedCascadeIndeterminateReasonV0::StandardPropertySyntaxIndeterminate,
            derivation_steps,
        );
    }

    match substituted_value {
        CascadeValue::Unset => computed_value_from_unset(
            property,
            winner_declaration_id,
            parent_computed_value,
            false,
            {
                derivation_steps.push("unsetKeywordResolved");
                derivation_steps
            },
            registered_custom_property.as_ref(),
        ),
        CascadeValue::Inherit => computed_value_from_inherit(
            property,
            winner_declaration_id,
            parent_computed_value,
            {
                derivation_steps.push("inheritKeywordResolved");
                derivation_steps
            },
            registered_custom_property.as_ref(),
        ),
        CascadeValue::Initial => computed_value_from_initial(
            property,
            winner_declaration_id,
            {
                derivation_steps.push("initialKeywordResolved");
                derivation_steps
            },
            registered_custom_property.as_ref(),
        ),
        value => {
            derivation_steps.push("computedValueResolved");
            CascadeComputedValueResultV0 {
                schema_version: "0",
                product: "omena-cascade.computed-value",
                property,
                status: ComputedCascadeValueStatusV0::Resolved,
                value,
                winner_declaration_id,
                inherited: false,
                used_initial_value: false,
                invalid_at_computed_value_time: false,
                indeterminate_reason: None,
                fallback_indeterminate_reason: None,
                derivation_steps,
            }
        }
    }
}

fn render_substituted_standard_value(value: &CascadeValue) -> Option<String> {
    match value {
        CascadeValue::Literal(value) => Some(value.clone()),
        CascadeValue::Composite(parts) => parts.iter().try_fold(String::new(), |mut text, part| {
            text.push_str(render_substituted_standard_value(part)?.as_str());
            Some(text)
        }),
        CascadeValue::Initial => Some("initial".to_string()),
        CascadeValue::Inherit => Some("inherit".to_string()),
        CascadeValue::Unset => Some("unset".to_string()),
        CascadeValue::Var { .. }
        | CascadeValue::Indeterminate
        | CascadeValue::GuaranteedInvalid => None,
    }
}

fn cascade_value_contains_var_reference(value: &CascadeValue) -> bool {
    match value {
        CascadeValue::Var { .. } => true,
        CascadeValue::Composite(values) => values.iter().any(cascade_value_contains_var_reference),
        CascadeValue::Literal(_)
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::Indeterminate
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => false,
    }
}

fn computed_value_from_unset(
    property: String,
    winner_declaration_id: Option<String>,
    parent_computed_value: Option<CascadeValue>,
    invalid_at_computed_value_time: bool,
    mut derivation_steps: Vec<&'static str>,
    registered_custom_property: Option<&CascadeRegisteredCustomPropertyV0>,
) -> CascadeComputedValueResultV0 {
    match property_inheritance(&property, registered_custom_property) {
        CssPropertyInheritanceV0::Inherited => {
            derivation_steps.push("unsetForInheritedPropertyUsesInheritance");
            return computed_value_from_inherit(
                property,
                winner_declaration_id,
                parent_computed_value,
                derivation_steps,
                registered_custom_property,
            )
            .with_invalid_at_computed_value_time(invalid_at_computed_value_time);
        }
        CssPropertyInheritanceV0::Unknown => {
            derivation_steps.push("propertyInheritanceMetadataUnavailable");
            return computed_value_from_unknown_metadata(
                property,
                winner_declaration_id,
                ComputedCascadeIndeterminateReasonV0::PropertyInheritanceMetadataUnavailable,
                derivation_steps,
            )
            .with_invalid_at_computed_value_time(invalid_at_computed_value_time);
        }
        CssPropertyInheritanceV0::NotInherited => {}
    }

    derivation_steps.push("unsetForNonInheritedPropertyUsesInitial");
    computed_value_from_initial(
        property,
        winner_declaration_id,
        derivation_steps,
        registered_custom_property,
    )
    .with_invalid_at_computed_value_time(invalid_at_computed_value_time)
}

fn computed_value_from_inherit(
    property: String,
    winner_declaration_id: Option<String>,
    parent_computed_value: Option<CascadeValue>,
    mut derivation_steps: Vec<&'static str>,
    registered_custom_property: Option<&CascadeRegisteredCustomPropertyV0>,
) -> CascadeComputedValueResultV0 {
    match parent_computed_value {
        Some(CascadeValue::Indeterminate) => {
            derivation_steps.push("inheritedFromIndeterminateParent");
            computed_value_from_unknown_metadata(
                property,
                winner_declaration_id,
                ComputedCascadeIndeterminateReasonV0::InheritedFromIndeterminateParent,
                derivation_steps,
            )
        }
        Some(value) => {
            derivation_steps.push("parentComputedValueUsed");
            CascadeComputedValueResultV0 {
                schema_version: "0",
                product: "omena-cascade.computed-value",
                property,
                status: ComputedCascadeValueStatusV0::Inherited,
                value,
                winner_declaration_id,
                inherited: true,
                used_initial_value: false,
                invalid_at_computed_value_time: false,
                indeterminate_reason: None,
                fallback_indeterminate_reason: None,
                derivation_steps,
            }
        }
        None => {
            derivation_steps.push("missingParentFallsBackToInitial");
            computed_value_from_initial(
                property,
                winner_declaration_id,
                derivation_steps,
                registered_custom_property,
            )
        }
    }
}

fn computed_value_from_initial(
    property: String,
    winner_declaration_id: Option<String>,
    mut derivation_steps: Vec<&'static str>,
    registered_custom_property: Option<&CascadeRegisteredCustomPropertyV0>,
) -> CascadeComputedValueResultV0 {
    if let Some(registration) = registered_custom_property {
        derivation_steps.push("registeredPropertyInitialValueUsed");
        return CascadeComputedValueResultV0 {
            schema_version: "0",
            product: "omena-cascade.computed-value",
            value: registration.initial_value.clone(),
            property,
            status: ComputedCascadeValueStatusV0::Initial,
            winner_declaration_id,
            inherited: false,
            used_initial_value: true,
            invalid_at_computed_value_time: false,
            indeterminate_reason: None,
            fallback_indeterminate_reason: None,
            derivation_steps,
        };
    }
    derivation_steps.push("initialValueTableConsulted");
    let canonical_property = PropertyNameV0::from_authored(&property);
    match css_property_initial_value(canonical_property.canonical_name()) {
        CssPropertyInitialValueV0::Literal(value) => CascadeComputedValueResultV0 {
            schema_version: "0",
            product: "omena-cascade.computed-value",
            value: CascadeValue::Literal(value.to_string()),
            property,
            status: ComputedCascadeValueStatusV0::Initial,
            winner_declaration_id,
            inherited: false,
            used_initial_value: true,
            invalid_at_computed_value_time: false,
            indeterminate_reason: None,
            fallback_indeterminate_reason: None,
            derivation_steps,
        },
        CssPropertyInitialValueV0::GuaranteedInvalid => CascadeComputedValueResultV0 {
            schema_version: "0",
            product: "omena-cascade.computed-value",
            value: CascadeValue::GuaranteedInvalid,
            property,
            status: ComputedCascadeValueStatusV0::Initial,
            winner_declaration_id,
            inherited: false,
            used_initial_value: true,
            invalid_at_computed_value_time: false,
            indeterminate_reason: None,
            fallback_indeterminate_reason: None,
            derivation_steps,
        },
        CssPropertyInitialValueV0::Unknown => {
            derivation_steps.push("propertyInitialValueMetadataUnavailable");
            computed_value_from_unknown_metadata(
                property,
                winner_declaration_id,
                ComputedCascadeIndeterminateReasonV0::PropertyInitialValueMetadataUnavailable,
                derivation_steps,
            )
        }
    }
}

fn property_inheritance(
    property: &str,
    registered_custom_property: Option<&CascadeRegisteredCustomPropertyV0>,
) -> CssPropertyInheritanceV0 {
    // Preserve unknown metadata for the caller's fail-closed diagnostic path.
    match registered_custom_property {
        Some(registration) if registration.inherits => CssPropertyInheritanceV0::Inherited,
        Some(_) => CssPropertyInheritanceV0::NotInherited,
        None => {
            let property = PropertyNameV0::from_authored(property);
            css_property_is_inherited(property.canonical_name())
        }
    }
}

fn computed_value_from_unknown_metadata(
    property: String,
    winner_declaration_id: Option<String>,
    reason: ComputedCascadeIndeterminateReasonV0,
    derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    CascadeComputedValueResultV0 {
        schema_version: "0",
        product: "omena-cascade.computed-value",
        property,
        status: ComputedCascadeValueStatusV0::Indeterminate,
        value: CascadeValue::Indeterminate,
        winner_declaration_id,
        inherited: false,
        used_initial_value: false,
        invalid_at_computed_value_time: false,
        indeterminate_reason: Some(reason),
        fallback_indeterminate_reason: None,
        derivation_steps,
    }
}

pub(crate) fn computed_value_from_indeterminate_cascade_outcome(
    property: &str,
    outcome: &CascadeOutcome,
) -> Option<CascadeComputedValueResultV0> {
    match outcome {
        CascadeOutcome::RankedSet(_) | CascadeOutcome::Top => {
            Some(computed_value_from_unknown_metadata(
                property.to_string(),
                None,
                ComputedCascadeIndeterminateReasonV0::CascadeOutcomeIndeterminate,
                vec!["cascadeOutcomeIndeterminate"],
            ))
        }
        CascadeOutcome::Definite { .. } | CascadeOutcome::Inherit => None,
    }
}

impl CascadeComputedValueResultV0 {
    fn with_invalid_at_computed_value_time(mut self, invalid_at_computed_value_time: bool) -> Self {
        if invalid_at_computed_value_time {
            self.status = ComputedCascadeValueStatusV0::InvalidAtComputedValueTime;
            self.invalid_at_computed_value_time = true;
            if self.value == CascadeValue::Indeterminate {
                self.value = CascadeValue::GuaranteedInvalid;
                self.fallback_indeterminate_reason = self.indeterminate_reason.take();
            }
        }
        self
    }
}

//! Computed-value resolution over cascade winners and custom-property environments.
//!
//! This module owns the seed implementation for turning a cascaded declaration
//! into a computed value witness without hiding indeterminate cascade outcomes.

use crate::{
    CascadeComputedValueInputV0, CascadeComputedValueResultV0, CascadeOutcome,
    CascadeRegisteredCustomPropertyV0, CascadeRegisteredValueVerdictV0, CascadeValue,
    ComputedCascadeValueStatusV0, CssPropertyInheritanceV0, CssPropertyInitialValueV0,
    cascade_property, css_property_initial_value, css_property_is_inherited,
    substitute_custom_properties,
};

pub fn compute_cascade_computed_value(
    input: CascadeComputedValueInputV0,
) -> CascadeComputedValueResultV0 {
    let CascadeComputedValueInputV0 {
        property,
        declarations,
        custom_property_env,
        parent_computed_value,
        registered_custom_property,
    } = input;
    let registered_custom_property =
        registered_custom_property.filter(|registration| registration.name == property);
    let outcome = cascade_property(declarations, &property);
    let (winner_declaration_id, cascaded_value, registered_value_verdict, mut derivation_steps) =
        match outcome {
            CascadeOutcome::Definite { winner, .. } => {
                let registered_value_verdict =
                    registered_custom_property.as_ref().map(|registration| {
                        registration
                            .declaration_value_verdicts
                            .get(winner.id.as_str())
                            .copied()
                            .unwrap_or(CascadeRegisteredValueVerdictV0::Unknown)
                    });
                (
                    Some(winner.id),
                    winner.value,
                    registered_value_verdict,
                    vec!["cascadeWinnerSelected", "computedValueResolutionStarted"],
                )
            }
            CascadeOutcome::Inherit => {
                match property_inheritance(&property, registered_custom_property.as_ref()) {
                    CssPropertyInheritanceV0::Inherited => (
                        None,
                        CascadeValue::Inherit,
                        None,
                        vec!["noCascadeWinner", "inheritanceOrInitialSelected"],
                    ),
                    CssPropertyInheritanceV0::NotInherited => (
                        None,
                        CascadeValue::Initial,
                        None,
                        vec!["noCascadeWinner", "inheritanceOrInitialSelected"],
                    ),
                    CssPropertyInheritanceV0::Unknown => {
                        return computed_value_from_unknown_metadata(
                            property,
                            None,
                            vec!["noCascadeWinner", "propertyInheritanceMetadataUnavailable"],
                        );
                    }
                }
            }
            CascadeOutcome::RankedSet(_) | CascadeOutcome::Top => {
                return CascadeComputedValueResultV0 {
                    schema_version: "0",
                    product: "omena-cascade.computed-value",
                    property,
                    status: ComputedCascadeValueStatusV0::InvalidAtComputedValueTime,
                    value: CascadeValue::GuaranteedInvalid,
                    winner_declaration_id: None,
                    inherited: false,
                    used_initial_value: false,
                    invalid_at_computed_value_time: true,
                    derivation_steps: vec!["cascadeOutcomeIndeterminate"],
                };
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
                derivation_steps,
            );
        }
        Some(CascadeRegisteredValueVerdictV0::Matched) | None => {}
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
                derivation_steps,
            }
        }
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
                derivation_steps,
            );
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
            derivation_steps,
        };
    }
    derivation_steps.push("initialValueTableConsulted");
    match css_property_initial_value(&property) {
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
            derivation_steps,
        },
        CssPropertyInitialValueV0::Unknown => {
            derivation_steps.push("propertyInitialValueMetadataUnavailable");
            computed_value_from_unknown_metadata(property, winner_declaration_id, derivation_steps)
        }
    }
}

fn property_inheritance(
    property: &str,
    registered_custom_property: Option<&CascadeRegisteredCustomPropertyV0>,
) -> CssPropertyInheritanceV0 {
    match registered_custom_property {
        Some(registration) if registration.inherits => CssPropertyInheritanceV0::Inherited,
        Some(_) => CssPropertyInheritanceV0::NotInherited,
        None => css_property_is_inherited(property),
    }
}

fn computed_value_from_unknown_metadata(
    property: String,
    winner_declaration_id: Option<String>,
    derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    CascadeComputedValueResultV0 {
        schema_version: "0",
        product: "omena-cascade.computed-value",
        property,
        status: ComputedCascadeValueStatusV0::InvalidAtComputedValueTime,
        value: CascadeValue::GuaranteedInvalid,
        winner_declaration_id,
        inherited: false,
        used_initial_value: false,
        invalid_at_computed_value_time: true,
        derivation_steps,
    }
}

impl CascadeComputedValueResultV0 {
    fn with_invalid_at_computed_value_time(mut self, invalid_at_computed_value_time: bool) -> Self {
        if invalid_at_computed_value_time {
            self.status = ComputedCascadeValueStatusV0::InvalidAtComputedValueTime;
            self.invalid_at_computed_value_time = true;
        }
        self
    }
}

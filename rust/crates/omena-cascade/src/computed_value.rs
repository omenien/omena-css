//! Computed-value resolution over cascade winners and custom-property environments.
//!
//! This module owns the seed implementation for turning a cascaded declaration
//! into a computed value witness without hiding indeterminate cascade outcomes.

use crate::{
    CascadeComputedValueInputV0, CascadeComputedValueResultV0, CascadeOutcome, CascadeValue,
    ComputedCascadeValueStatusV0, cascade_property, substitute_custom_properties,
};

pub fn compute_cascade_computed_value(
    input: CascadeComputedValueInputV0,
) -> CascadeComputedValueResultV0 {
    let property = input.property.clone();
    let outcome = cascade_property(input.declarations, &property);
    let (winner_declaration_id, cascaded_value, mut derivation_steps) = match outcome {
        CascadeOutcome::Definite { winner, .. } => (
            Some(winner.id),
            winner.value,
            vec!["cascadeWinnerSelected", "computedValueResolutionStarted"],
        ),
        CascadeOutcome::Inherit => (
            None,
            if property_is_inherited(&property) {
                CascadeValue::Inherit
            } else {
                CascadeValue::Initial
            },
            vec!["noCascadeWinner", "inheritanceOrInitialSelected"],
        ),
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

    let substituted_value =
        substitute_custom_properties(&cascaded_value, &input.custom_property_env);
    if substituted_value == CascadeValue::GuaranteedInvalid {
        derivation_steps.push("substitutionProducedGuaranteedInvalid");
        derivation_steps.push("invalidAtComputedValueTimeFallsBackAsUnset");
        return computed_value_from_unset(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            true,
            derivation_steps,
        );
    }

    match substituted_value {
        CascadeValue::Unset => computed_value_from_unset(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            false,
            {
                derivation_steps.push("unsetKeywordResolved");
                derivation_steps
            },
        ),
        CascadeValue::Inherit => computed_value_from_inherit(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            {
                derivation_steps.push("inheritKeywordResolved");
                derivation_steps
            },
        ),
        CascadeValue::Initial => computed_value_from_initial(property, winner_declaration_id, {
            derivation_steps.push("initialKeywordResolved");
            derivation_steps
        }),
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
) -> CascadeComputedValueResultV0 {
    if property_is_inherited(&property) {
        derivation_steps.push("unsetForInheritedPropertyUsesInheritance");
        return computed_value_from_inherit(
            property,
            winner_declaration_id,
            parent_computed_value,
            derivation_steps,
        )
        .with_invalid_at_computed_value_time(invalid_at_computed_value_time);
    }

    derivation_steps.push("unsetForNonInheritedPropertyUsesInitial");
    computed_value_from_initial(property, winner_declaration_id, derivation_steps)
        .with_invalid_at_computed_value_time(invalid_at_computed_value_time)
}

fn computed_value_from_inherit(
    property: String,
    winner_declaration_id: Option<String>,
    parent_computed_value: Option<CascadeValue>,
    mut derivation_steps: Vec<&'static str>,
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
            computed_value_from_initial(property, winner_declaration_id, derivation_steps)
        }
    }
}

fn computed_value_from_initial(
    property: String,
    winner_declaration_id: Option<String>,
    mut derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    derivation_steps.push("initialValueTableConsulted");
    CascadeComputedValueResultV0 {
        schema_version: "0",
        product: "omena-cascade.computed-value",
        value: initial_cascade_value_for_property(&property),
        property,
        status: ComputedCascadeValueStatusV0::Initial,
        winner_declaration_id,
        inherited: false,
        used_initial_value: true,
        invalid_at_computed_value_time: false,
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

fn property_is_inherited(property: &str) -> bool {
    property.starts_with("--")
        || matches!(
            property,
            "color"
                | "cursor"
                | "direction"
                | "font"
                | "font-family"
                | "font-size"
                | "font-style"
                | "font-variant"
                | "font-weight"
                | "letter-spacing"
                | "line-height"
                | "text-align"
                | "text-indent"
                | "text-transform"
                | "visibility"
                | "white-space"
                | "word-spacing"
        )
}

fn initial_cascade_value_for_property(property: &str) -> CascadeValue {
    if property.starts_with("--") {
        return CascadeValue::GuaranteedInvalid;
    }

    let value = match property {
        "background-color" | "border-color" | "caret-color" | "outline-color" => "transparent",
        "border-style" | "display" => "none",
        "border-width" | "margin" | "padding" => "0",
        "box-shadow" | "text-shadow" => "none",
        "color" => "canvastext",
        "cursor" => "auto",
        "font-family" => "serif",
        "font-size" => "medium",
        "font-style" | "font-variant" | "font-weight" => "normal",
        "letter-spacing" | "line-height" | "word-spacing" => "normal",
        "opacity" => "1",
        "text-align" => "start",
        "text-indent" => "0",
        "text-transform" => "none",
        "visibility" => "visible",
        "white-space" => "normal",
        _ => "initial",
    };
    CascadeValue::Literal(value.to_string())
}

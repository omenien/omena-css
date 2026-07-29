use serde::{Serialize, ser::SerializeStruct};

use super::{
    OmenaQueryCascadeLayerTopologyIncompleteV0, OmenaQueryRuntimeStateScenarioEvidenceV0,
    OmenaQueryRuntimeStateScenarioV0,
};

const UNKNOWN_ACTIVATION_ID_PREFIX: &str = "\0omena-query:unknown-activation:";
const CASCADE_LAYER_TOPOLOGY_DRIVER: &str = "cascadeLayerTopologyCompleteness";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultCertainty {
    Definite,
    Indeterminate,
    Unknown,
}

pub(crate) fn runtime_state_unknown_activation_declaration_id(declaration_id: &str) -> String {
    format!("{UNKNOWN_ACTIVATION_ID_PREFIX}{declaration_id}")
}

fn collect_unknown_activation_declaration_ids(
    scenario: &OmenaQueryRuntimeStateScenarioV0,
) -> Vec<&str> {
    scenario
        .declaration_ids
        .iter()
        .filter_map(|declaration_id| declaration_id.strip_prefix(UNKNOWN_ACTIVATION_ID_PREFIX))
        .collect()
}

fn serialized_declaration_ids(scenario: &OmenaQueryRuntimeStateScenarioV0) -> Vec<&str> {
    scenario
        .declaration_ids
        .iter()
        .map(|declaration_id| {
            declaration_id
                .strip_prefix(UNKNOWN_ACTIVATION_ID_PREFIX)
                .unwrap_or(declaration_id.as_str())
        })
        .collect()
}

pub(crate) fn runtime_state_result_certainty_labels(
    scenarios: &[OmenaQueryRuntimeStateScenarioV0],
    confidence_tier: &str,
    cascade_layer_topology_incomplete: bool,
) -> (&'static str, &'static str) {
    // The legacy confidence tier remains the 0.x environment-coverage field.
    // Re-keying or deprecating it is a major-version API decision; result
    // certainty is serialized as a separate axis for compatible consumers.
    let conditional_environment = confidence_tier == "conditionalDefinite";
    let certainty = if cascade_layer_topology_incomplete {
        ResultCertainty::Indeterminate
    } else if scenarios
        .iter()
        .any(|scenario| !scenario.unknown_activation_declaration_ids().is_empty())
    {
        ResultCertainty::Unknown
    } else if !scenarios.is_empty()
        && scenarios
            .iter()
            .all(|scenario| scenario.winner_declaration_id.is_some())
    {
        ResultCertainty::Definite
    } else {
        ResultCertainty::Indeterminate
    };

    match (conditional_environment, certainty) {
        (false, ResultCertainty::Definite) => {
            ("staticDefinite", "staticDefiniteWithinModeledEnvironment")
        }
        (false, ResultCertainty::Indeterminate) => (
            "staticIndeterminate",
            "staticIndeterminateWithinModeledEnvironment",
        ),
        (false, ResultCertainty::Unknown) => {
            ("staticUnknown", "staticUnknownWithinModeledEnvironment")
        }
        (true, ResultCertainty::Definite) => (
            "conditionalDefinite",
            "conditionalDefiniteWithinModeledEnvironment",
        ),
        (true, ResultCertainty::Indeterminate) => (
            "conditionalIndeterminate",
            "conditionalIndeterminateWithinModeledEnvironment",
        ),
        (true, ResultCertainty::Unknown) => (
            "conditionalUnknown",
            "conditionalUnknownWithinModeledEnvironment",
        ),
    }
}

impl OmenaQueryRuntimeStateScenarioEvidenceV0 {
    /// Returns the typed layer-topology disclosure projected into this result.
    pub fn cascade_layer_topology_incomplete(
        &self,
    ) -> Option<OmenaQueryCascadeLayerTopologyIncompleteV0> {
        self.driver_summaries
            .iter()
            .find(|driver| {
                driver.driver == CASCADE_LAYER_TOPOLOGY_DRIVER && driver.status == "incomplete"
            })
            .map(|driver| OmenaQueryCascadeLayerTopologyIncompleteV0 {
                unresolved_count: driver.scenario_count,
            })
    }

    pub fn result_certainty(&self) -> &'static str {
        runtime_state_result_certainty_labels(
            self.scenarios.as_slice(),
            self.confidence_tier,
            self.cascade_layer_topology_incomplete().is_some(),
        )
        .0
    }

    pub fn result_certainty_within_modeled_environment(&self) -> &'static str {
        runtime_state_result_certainty_labels(
            self.scenarios.as_slice(),
            self.confidence_tier,
            self.cascade_layer_topology_incomplete().is_some(),
        )
        .1
    }
}

impl Serialize for OmenaQueryRuntimeStateScenarioEvidenceV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let cascade_layer_topology_incomplete = self.cascade_layer_topology_incomplete();
        let field_count = 13
            + usize::from(!self.static_condition_pruning.is_empty())
            + usize::from(!self.inline_style_overrides.is_empty())
            + usize::from(cascade_layer_topology_incomplete.is_some());
        let mut state =
            serializer.serialize_struct("OmenaQueryRuntimeStateScenarioEvidenceV0", field_count)?;
        state.serialize_field("schemaVersion", self.schema_version)?;
        state.serialize_field("product", self.product)?;
        state.serialize_field("selector", &self.selector)?;
        state.serialize_field("selectorClassNames", &self.selector_class_names)?;
        state.serialize_field("propertyName", &self.property_name)?;
        state.serialize_field("scenarioJoinKind", self.scenario_join_kind)?;
        state.serialize_field("confidenceTier", self.confidence_tier)?;
        state.serialize_field(
            "confidenceTierWithinModeledEnvironment",
            self.confidence_tier_within_modeled_environment,
        )?;
        state.serialize_field("resultCertainty", self.result_certainty())?;
        state.serialize_field(
            "resultCertaintyWithinModeledEnvironment",
            self.result_certainty_within_modeled_environment(),
        )?;
        state.serialize_field("staticBoundary", &self.static_boundary)?;
        state.serialize_field("driverSummaries", &self.driver_summaries)?;
        state.serialize_field("scenarios", &self.scenarios)?;
        if !self.static_condition_pruning.is_empty() {
            state.serialize_field("staticConditionPruning", &self.static_condition_pruning)?;
        }
        if !self.inline_style_overrides.is_empty() {
            state.serialize_field("inlineStyleOverrides", &self.inline_style_overrides)?;
        }
        if let Some(topology_incomplete) = &cascade_layer_topology_incomplete {
            state.serialize_field("cascadeLayerTopologyIncomplete", topology_incomplete)?;
        }
        state.end()
    }
}

impl OmenaQueryRuntimeStateScenarioV0 {
    pub fn unknown_activation_declaration_ids(&self) -> Vec<&str> {
        collect_unknown_activation_declaration_ids(self)
    }
}

impl Serialize for OmenaQueryRuntimeStateScenarioV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let unknown_activation_declaration_ids = self.unknown_activation_declaration_ids();
        let declaration_ids = serialized_declaration_ids(self);
        let field_count = 4
            + usize::from(self.pseudo_state.is_some())
            + usize::from(!unknown_activation_declaration_ids.is_empty())
            + usize::from(self.winner_declaration_id.is_some())
            + usize::from(self.winner_value.is_some());
        let mut state =
            serializer.serialize_struct("OmenaQueryRuntimeStateScenarioV0", field_count)?;
        state.serialize_field("scenarioKind", self.scenario_kind)?;
        if let Some(pseudo_state) = self.pseudo_state.as_ref() {
            state.serialize_field("pseudoState", pseudo_state)?;
        }
        state.serialize_field("conditionContext", &self.condition_context)?;
        state.serialize_field("declarationIds", &declaration_ids)?;
        if !unknown_activation_declaration_ids.is_empty() {
            state.serialize_field(
                "unknownActivationDeclarationIds",
                &unknown_activation_declaration_ids,
            )?;
        }
        if let Some(winner_declaration_id) = self.winner_declaration_id.as_ref() {
            state.serialize_field("winnerDeclarationId", winner_declaration_id)?;
        }
        if let Some(winner_value) = self.winner_value.as_ref() {
            state.serialize_field("winnerValue", winner_value)?;
        }
        state.serialize_field("propertyValueNarrowing", &self.property_value_narrowing)?;
        state.end()
    }
}

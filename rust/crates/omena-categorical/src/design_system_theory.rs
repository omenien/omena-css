use std::collections::BTreeSet;

use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemTheoryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_id: String,
    pub sorts: Vec<DesignSystemSortV0>,
    pub function_symbols: Vec<DesignSystemFunctionSymbolV0>,
    pub axioms: Vec<DesignSystemAxiomV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemSortV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub sort_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemFunctionSymbolV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub symbol_name: String,
    pub domain_sorts: Vec<String>,
    pub codomain_sort: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemAxiomV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub axiom_id: String,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemModelV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub model_id: String,
    pub theory_id: String,
    pub source_product: &'static str,
    pub project_id: String,
    pub summary_hash: String,
    pub summary_edge_count: usize,
    pub edge_kind_counts: Vec<DesignSystemEdgeKindCountV0>,
    pub sort_interpretations: Vec<SortInterpretationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemProjectSummaryInputV0 {
    pub project_id: String,
    pub source_product: &'static str,
    pub summary_hash: String,
    pub summary_edge_count: usize,
    pub edge_kind_counts: Vec<DesignSystemEdgeKindCountV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemEdgeKindCountV0 {
    pub edge_kind: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SortInterpretationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub sort_name: String,
    pub element_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemInvariantSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub invariant_id: String,
    pub invariant_kind: &'static str,
    pub model_count: usize,
    pub source_products: Vec<&'static str>,
    pub model_hashes: Vec<String>,
    pub differing_sort_names: Vec<String>,
    pub accepted: bool,
}

pub fn empty_design_system_theory_v0(theory_id: impl Into<String>) -> DesignSystemTheoryV0 {
    DesignSystemTheoryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.design-system-theory",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        theory_id: theory_id.into(),
        sorts: Vec::new(),
        function_symbols: Vec::new(),
        axioms: Vec::new(),
    }
}

pub fn design_system_model_from_project_summary_v0(
    theory_id: impl Into<String>,
    input: DesignSystemProjectSummaryInputV0,
) -> DesignSystemModelV0 {
    let theory_id = theory_id.into();
    let mut edge_kind_counts = input.edge_kind_counts;
    edge_kind_counts.sort();
    let mut sort_interpretations = edge_kind_counts
        .iter()
        .map(|entry| sort_interpretation_v0(format!("edgeKind:{}", entry.edge_kind), entry.count))
        .collect::<Vec<_>>();
    sort_interpretations.push(sort_interpretation_v0(
        "summaryEdge".to_string(),
        input.summary_edge_count,
    ));
    sort_interpretations.sort_by(|left, right| left.sort_name.cmp(&right.sort_name));

    DesignSystemModelV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.design-system-model",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        model_id: format!(
            "design-system-model:{}:{}",
            input.project_id, input.summary_hash
        ),
        theory_id,
        source_product: input.source_product,
        project_id: input.project_id,
        summary_hash: input.summary_hash,
        summary_edge_count: input.summary_edge_count,
        edge_kind_counts,
        sort_interpretations,
    }
}

pub fn compare_design_system_models_for_invariant_v0(
    invariant_id: impl Into<String>,
    models: &[DesignSystemModelV0],
) -> DesignSystemInvariantSummaryV0 {
    let differing_sort_names = differing_design_system_model_sort_names_v0(models);
    let accepted = models.len() >= 2 && differing_sort_names.is_empty();
    DesignSystemInvariantSummaryV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.design-system-invariant-summary",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        invariant_id: invariant_id.into(),
        invariant_kind: "crossProjectEdgeKindSymmetry",
        model_count: models.len(),
        source_products: models.iter().map(|model| model.source_product).collect(),
        model_hashes: models
            .iter()
            .map(|model| model.summary_hash.clone())
            .collect(),
        differing_sort_names,
        accepted,
    }
}

fn differing_design_system_model_sort_names_v0(models: &[DesignSystemModelV0]) -> Vec<String> {
    let Some(first) = models.first() else {
        return Vec::new();
    };
    let baseline = first
        .sort_interpretations
        .iter()
        .map(|sort| (sort.sort_name.as_str(), sort.element_count))
        .collect::<Vec<_>>();
    let mut differing_sort_names = BTreeSet::new();

    for model in models.iter().skip(1) {
        for (sort_name, baseline_count) in &baseline {
            let current_count = model
                .sort_interpretations
                .iter()
                .find(|sort| sort.sort_name == *sort_name)
                .map(|sort| sort.element_count);
            if current_count != Some(*baseline_count) {
                differing_sort_names.insert((*sort_name).to_string());
            }
        }
        for sort in &model.sort_interpretations {
            if !baseline
                .iter()
                .any(|(sort_name, _)| *sort_name == sort.sort_name)
            {
                differing_sort_names.insert(sort.sort_name.clone());
            }
        }
    }

    differing_sort_names.into_iter().collect()
}

fn sort_interpretation_v0(sort_name: String, element_count: usize) -> SortInterpretationV0 {
    SortInterpretationV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.sort-interpretation",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        sort_name,
        element_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_project_design_system_invariant_uses_summary_edge_kind_counts() {
        let first = design_system_model_from_project_summary_v0(
            "theory",
            input("project-a", "hash-a", &[("sourceSelectorReference", 1)]),
        );
        let matching = design_system_model_from_project_summary_v0(
            "theory",
            input("project-b", "hash-b", &[("sourceSelectorReference", 1)]),
        );
        let changed = design_system_model_from_project_summary_v0(
            "theory",
            input("project-c", "hash-c", &[("sourceSelectorReference", 2)]),
        );

        let accepted = compare_design_system_models_for_invariant_v0(
            "cross-project",
            &[first.clone(), matching],
        );
        let rejected =
            compare_design_system_models_for_invariant_v0("cross-project", &[first, changed]);

        assert!(accepted.accepted);
        assert_eq!(accepted.model_count, 2);
        assert_eq!(accepted.differing_sort_names, Vec::<String>::new());
        assert!(!rejected.accepted);
        assert_eq!(
            rejected.differing_sort_names,
            vec![
                "edgeKind:sourceSelectorReference".to_string(),
                "summaryEdge".to_string()
            ]
        );
    }

    fn input(
        project_id: &str,
        summary_hash: &str,
        edge_kind_counts: &[(&str, usize)],
    ) -> DesignSystemProjectSummaryInputV0 {
        DesignSystemProjectSummaryInputV0 {
            project_id: project_id.to_string(),
            source_product: "omena-query.cross-file-summary",
            summary_hash: summary_hash.to_string(),
            summary_edge_count: edge_kind_counts.iter().map(|(_, count)| *count).sum(),
            edge_kind_counts: edge_kind_counts
                .iter()
                .map(|(edge_kind, count)| DesignSystemEdgeKindCountV0 {
                    edge_kind: (*edge_kind).to_string(),
                    count: *count,
                })
                .collect(),
        }
    }
}

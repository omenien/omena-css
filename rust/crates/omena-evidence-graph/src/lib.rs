use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const EVIDENCE_GRAPH_SCHEMA_VERSION_V0: &str = "0";
pub const EVIDENCE_GRAPH_PRODUCT_V0: &str = "omena-evidence-graph.graph";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GuaranteeKindV0 {
    Floor,
    SampledFixtureWitness,
    SchedulerPriorityFixtureWitness,
    MetricInputFixtureWitness,
    IncrementalLayerEvidenceOnly,
    AlphaRenamingStableHashFixtureWitness,
    NotClaimedExactTraversal,
}

impl GuaranteeKindV0 {
    pub const fn for_label_less_family() -> Self {
        Self::Floor
    }

    pub const fn existing_label(self) -> Option<&'static str> {
        match self {
            Self::Floor => None,
            Self::SampledFixtureWitness => Some("sampledFixtureWitnessNotEquivalenceProof"),
            Self::SchedulerPriorityFixtureWitness => Some("fixtureWitnessSchedulerPriority"),
            Self::MetricInputFixtureWitness => Some("fixtureWitnessMetricInput"),
            Self::IncrementalLayerEvidenceOnly => Some("m6IncrementalLayerEvidenceOnly"),
            Self::AlphaRenamingStableHashFixtureWitness => {
                Some("fixtureWitnessAlphaRenamingStableHash")
            }
            Self::NotClaimedExactTraversal => Some("notClaimedExactTraversal"),
        }
    }

    pub fn from_existing_label(label: &str) -> Option<Self> {
        match label {
            "sampledFixtureWitnessNotEquivalenceProof" => Some(Self::SampledFixtureWitness),
            "fixtureWitnessSchedulerPriority" => Some(Self::SchedulerPriorityFixtureWitness),
            "fixtureWitnessMetricInput" => Some(Self::MetricInputFixtureWitness),
            "m6IncrementalLayerEvidenceOnly" => Some(Self::IncrementalLayerEvidenceOnly),
            "fixtureWitnessAlphaRenamingStableHash" => {
                Some(Self::AlphaRenamingStableHashFixtureWitness)
            }
            "notClaimedExactTraversal" => Some(Self::NotClaimedExactTraversal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceNodeKeyV0 {
    pub query_identity: String,
    pub input_identity: String,
}

impl EvidenceNodeKeyV0 {
    pub fn new(query_identity: impl Into<String>, input_identity: impl Into<String>) -> Self {
        Self {
            query_identity: query_identity.into(),
            input_identity: input_identity.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceNodeSeedV0 {
    pub key: EvidenceNodeKeyV0,
    pub provenance: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<EvidenceAnalysisPrecisionV0>,
    pub guarantee: GuaranteeKindV0,
}

impl EvidenceNodeSeedV0 {
    pub fn new(
        key: EvidenceNodeKeyV0,
        provenance: Vec<String>,
        guarantee: GuaranteeKindV0,
    ) -> Self {
        Self::with_precision(key, provenance, None, guarantee)
    }

    pub fn with_precision(
        key: EvidenceNodeKeyV0,
        provenance: Vec<String>,
        precision: Option<EvidenceAnalysisPrecisionV0>,
        guarantee: GuaranteeKindV0,
    ) -> Self {
        Self {
            key,
            provenance,
            precision,
            guarantee,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceAnalysisPrecisionV0 {
    pub product: String,
    pub value_domain: String,
    pub flow_sensitivity: String,
    pub context_sensitivity: String,
    pub revision_axis: String,
}

impl EvidenceAnalysisPrecisionV0 {
    pub fn new(
        product: impl Into<String>,
        value_domain: impl Into<String>,
        flow_sensitivity: impl Into<String>,
        context_sensitivity: impl Into<String>,
        revision_axis: impl Into<String>,
    ) -> Self {
        Self {
            product: product.into(),
            value_domain: value_domain.into(),
            flow_sensitivity: flow_sensitivity.into(),
            context_sensitivity: context_sensitivity.into(),
            revision_axis: revision_axis.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceDemandEdgeV0 {
    pub from_query_identity: String,
    pub to_node_key: EvidenceNodeKeyV0,
    pub edge_kind: String,
}

impl EvidenceDemandEdgeV0 {
    pub fn new(
        from_query_identity: impl Into<String>,
        to_node_key: EvidenceNodeKeyV0,
        edge_kind: impl Into<String>,
    ) -> Self {
        Self {
            from_query_identity: from_query_identity.into(),
            to_node_key,
            edge_kind: edge_kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceNodeV0 {
    pub key: EvidenceNodeKeyV0,
    pub provenance: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<EvidenceAnalysisPrecisionV0>,
    pub guarantee: GuaranteeKindV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceGraphV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub nodes: Vec<EvidenceNodeV0>,
    pub edges: Vec<EvidenceDemandEdgeV0>,
}

impl EvidenceGraphV0 {
    pub fn node_input_identities(&self) -> BTreeSet<String> {
        self.nodes
            .iter()
            .map(|node| node.key.input_identity.clone())
            .collect()
    }

    pub fn edge_input_identities(&self) -> BTreeSet<String> {
        self.edges
            .iter()
            .map(|edge| edge.to_node_key.input_identity.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceGraphBuildErrorV0 {
    MissingDemandNode(EvidenceNodeKeyV0),
}

pub fn build_salsa_demand_evidence_graph_v0(
    all_node_seeds: impl IntoIterator<Item = EvidenceNodeSeedV0>,
    demand_edges: impl IntoIterator<Item = EvidenceDemandEdgeV0>,
) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
    build_evidence_graph_from_edges_v0(all_node_seeds, demand_edges)
}

pub fn build_evidence_graph_from_edges_v0(
    all_node_seeds: impl IntoIterator<Item = EvidenceNodeSeedV0>,
    demand_edges: impl IntoIterator<Item = EvidenceDemandEdgeV0>,
) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
    let all_nodes = all_node_seeds
        .into_iter()
        .map(|seed| (seed.key.clone(), seed))
        .collect::<BTreeMap<_, _>>();
    let edges = demand_edges.into_iter().collect::<Vec<_>>();
    let demanded_keys = edges
        .iter()
        .map(|edge| edge.to_node_key.clone())
        .collect::<BTreeSet<_>>();

    let mut nodes = Vec::new();
    for key in demanded_keys {
        let Some(seed) = all_nodes.get(&key) else {
            return Err(EvidenceGraphBuildErrorV0::MissingDemandNode(key));
        };
        nodes.push(EvidenceNodeV0 {
            key: seed.key.clone(),
            provenance: seed.provenance.clone(),
            precision: seed.precision.clone(),
            guarantee: seed.guarantee,
        });
    }

    Ok(EvidenceGraphV0 {
        schema_version: EVIDENCE_GRAPH_SCHEMA_VERSION_V0,
        product: EVIDENCE_GRAPH_PRODUCT_V0,
        nodes,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_seed(input_identity: &str) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            EvidenceNodeKeyV0::new("memo_style_fact_entry", input_identity),
            vec!["fixture-provenance".to_string()],
            GuaranteeKindV0::for_label_less_family(),
        )
    }

    #[test]
    fn guarantee_kind_round_trips_existing_labels_without_upgrading_label_less_nodes() {
        for kind in [
            GuaranteeKindV0::SampledFixtureWitness,
            GuaranteeKindV0::SchedulerPriorityFixtureWitness,
            GuaranteeKindV0::MetricInputFixtureWitness,
            GuaranteeKindV0::IncrementalLayerEvidenceOnly,
            GuaranteeKindV0::AlphaRenamingStableHashFixtureWitness,
            GuaranteeKindV0::NotClaimedExactTraversal,
        ] {
            let label = kind.existing_label();
            assert_eq!(
                label.and_then(GuaranteeKindV0::from_existing_label),
                Some(kind)
            );
        }
        assert_eq!(
            GuaranteeKindV0::for_label_less_family(),
            GuaranteeKindV0::Floor
        );
        assert_eq!(GuaranteeKindV0::Floor.existing_label(), None);
    }

    #[test]
    fn salsa_demand_graph_keys_on_edges_not_the_full_node_list() -> Result<(), &'static str> {
        let graph = build_salsa_demand_evidence_graph_v0(
            [
                node_seed("/workspace/src/App.module.scss"),
                node_seed("/workspace/src/_theme.scss"),
            ],
            [EvidenceDemandEdgeV0::new(
                "memo_workspace_diagnostics_substrate",
                EvidenceNodeKeyV0::new("memo_style_fact_entry", "/workspace/src/App.module.scss"),
                "salsa-demand-read",
            )],
        )
        .map_err(|_| "demand edge must target a known node")?;

        assert_eq!(
            graph.node_input_identities(),
            BTreeSet::from(["/workspace/src/App.module.scss".to_string()])
        );
        assert_eq!(
            graph.edge_input_identities(),
            BTreeSet::from(["/workspace/src/App.module.scss".to_string()])
        );
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);
        Ok(())
    }

    #[test]
    fn salsa_demand_graph_rejects_fabricated_edges() {
        let result = build_salsa_demand_evidence_graph_v0(
            [node_seed("/workspace/src/App.module.scss")],
            [EvidenceDemandEdgeV0::new(
                "memo_workspace_diagnostics_substrate",
                EvidenceNodeKeyV0::new("memo_style_fact_entry", "/workspace/src/_missing.scss"),
                "salsa-demand-read",
            )],
        );

        assert_eq!(
            result,
            Err(EvidenceGraphBuildErrorV0::MissingDemandNode(
                EvidenceNodeKeyV0::new("memo_style_fact_entry", "/workspace/src/_missing.scss")
            ))
        );
    }

    #[test]
    fn graph_serializes_without_shape_specific_runtime_dependencies() -> Result<(), &'static str> {
        let graph = build_salsa_demand_evidence_graph_v0(
            [node_seed("/workspace/src/App.module.scss")],
            [EvidenceDemandEdgeV0::new(
                "memo_workspace_diagnostics_substrate",
                EvidenceNodeKeyV0::new("memo_style_fact_entry", "/workspace/src/App.module.scss"),
                "salsa-demand-read",
            )],
        )
        .map_err(|_| "demand edge must target a known node")?;
        let json = serde_json::to_value(&graph).map_err(|_| "graph must serialize")?;
        assert_eq!(json["schemaVersion"], "0");
        assert_eq!(json["product"], EVIDENCE_GRAPH_PRODUCT_V0);
        assert_eq!(json["nodes"][0]["guarantee"], "floor");
        Ok(())
    }

    #[test]
    fn graph_preserves_optional_precision_payload() -> Result<(), &'static str> {
        let key = EvidenceNodeKeyV0::new("source_diagnostic_precision", "missingSelector");
        let graph = build_salsa_demand_evidence_graph_v0(
            [EvidenceNodeSeedV0::with_precision(
                key.clone(),
                vec!["omena-query.source-syntax-index".to_string()],
                Some(EvidenceAnalysisPrecisionV0::new(
                    "omena-query.analysis-precision",
                    "classValueResolution",
                    "sourceSyntaxIndex",
                    "perSourceReference",
                    "OmenaQuerySourceDiagnosticsForFileV0.input",
                )),
                GuaranteeKindV0::for_label_less_family(),
            )],
            [EvidenceDemandEdgeV0::new(
                "source_diagnostic_precision",
                key,
                "diagnostic-evidence",
            )],
        )
        .map_err(|_| "precision edge must target a known node")?;

        let precision = graph.nodes[0]
            .precision
            .as_ref()
            .ok_or("precision payload must round-trip through the graph")?;
        assert_eq!(precision.value_domain, "classValueResolution");
        assert_eq!(precision.flow_sensitivity, "sourceSyntaxIndex");
        assert_eq!(precision.context_sensitivity, "perSourceReference");
        Ok(())
    }
}

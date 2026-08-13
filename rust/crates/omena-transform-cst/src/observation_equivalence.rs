//! Observation-indexed equivalence for transform outputs.
//!
//! This relation checks projections supplied by an observation carrier. It
//! cannot establish that the selected profile models every browser-observable
//! effect; that remains a corpus and modelling obligation.

use std::collections::BTreeSet;
use std::fmt;

use serde::Serialize;

use super::{
    ObservationKindV0, PassObservationSurfaceV0, TransformPassDescriptorV0,
    TransformPassObservationRecordV0, default_transform_pass_descriptors,
    default_transform_pass_observation_records,
};

pub const OBSERVATION_KIND_COUNT_V0: usize = 16;

pub const fn all_observation_kinds_v0() -> [ObservationKindV0; OBSERVATION_KIND_COUNT_V0] {
    [
        ObservationKindV0::SelectorMatching,
        ObservationKindV0::CascadeWinner,
        ObservationKindV0::CascadeWinnerEquality,
        ObservationKindV0::ExportedClassNames,
        ObservationKindV0::CustomPropertyComputedValue,
        ObservationKindV0::KeyframesReachability,
        ObservationKindV0::SourceMapTrace,
        ObservationKindV0::LayerRank,
        ObservationKindV0::Specificity,
        ObservationKindV0::Inheritance,
        ObservationKindV0::DeclarationOrder,
        ObservationKindV0::TargetPredicate,
        ObservationKindV0::ModuleResolution,
        ObservationKindV0::ImportContext,
        ObservationKindV0::ValueGraphReachability,
        ObservationKindV0::SemanticMarker,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum TransformObserverV0 {
    RawBytes,
    Contract(ObservationKindV0),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum TransformObserverClassV0 {
    RawRepresentation,
    OutputProjection,
    ReadOnlyInput,
}

pub const fn observation_kind_observer_class_v0(
    kind: ObservationKindV0,
) -> TransformObserverClassV0 {
    match kind {
        ObservationKindV0::TargetPredicate => TransformObserverClassV0::ReadOnlyInput,
        ObservationKindV0::SelectorMatching
        | ObservationKindV0::CascadeWinner
        | ObservationKindV0::CascadeWinnerEquality
        | ObservationKindV0::ExportedClassNames
        | ObservationKindV0::CustomPropertyComputedValue
        | ObservationKindV0::KeyframesReachability
        | ObservationKindV0::SourceMapTrace
        | ObservationKindV0::LayerRank
        | ObservationKindV0::Specificity
        | ObservationKindV0::Inheritance
        | ObservationKindV0::DeclarationOrder
        | ObservationKindV0::ModuleResolution
        | ObservationKindV0::ImportContext
        | ObservationKindV0::ValueGraphReachability
        | ObservationKindV0::SemanticMarker => TransformObserverClassV0::OutputProjection,
    }
}

impl TransformObserverV0 {
    pub const fn observer_class(self) -> TransformObserverClassV0 {
        match self {
            Self::RawBytes => TransformObserverClassV0::RawRepresentation,
            Self::Contract(kind) => observation_kind_observer_class_v0(kind),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransformObservationProjectionV0 {
    kind: ObservationKindV0,
    value: String,
}

impl TransformObservationProjectionV0 {
    pub fn new(kind: ObservationKindV0, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }

    pub const fn kind(&self) -> ObservationKindV0 {
        self.kind
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransformObservationOutputErrorV0 {
    DuplicateProjection { kind: ObservationKindV0 },
    MissingProjection { kind: ObservationKindV0 },
    ProjectionCount { expected: usize, actual: usize },
}

impl fmt::Display for TransformObservationOutputErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateProjection { kind } => {
                write!(formatter, "duplicate observation projection: {kind:?}")
            }
            Self::MissingProjection { kind } => {
                write!(formatter, "missing observation projection: {kind:?}")
            }
            Self::ProjectionCount { expected, actual } => write!(
                formatter,
                "observation projection count is {actual}; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for TransformObservationOutputErrorV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransformObservationOutputV0 {
    raw_bytes: Vec<u8>,
    projections: Vec<TransformObservationProjectionV0>,
}

impl TransformObservationOutputV0 {
    pub fn new(
        raw_bytes: impl Into<Vec<u8>>,
        mut projections: Vec<TransformObservationProjectionV0>,
    ) -> Result<Self, TransformObservationOutputErrorV0> {
        let mut observed = BTreeSet::new();
        for projection in &projections {
            if !observed.insert(projection.kind) {
                return Err(TransformObservationOutputErrorV0::DuplicateProjection {
                    kind: projection.kind,
                });
            }
        }
        for kind in all_observation_kinds_v0() {
            if !observed.contains(&kind) {
                return Err(TransformObservationOutputErrorV0::MissingProjection { kind });
            }
        }
        if projections.len() != OBSERVATION_KIND_COUNT_V0 {
            return Err(TransformObservationOutputErrorV0::ProjectionCount {
                expected: OBSERVATION_KIND_COUNT_V0,
                actual: projections.len(),
            });
        }
        projections.sort_by_key(TransformObservationProjectionV0::kind);
        Ok(Self {
            raw_bytes: raw_bytes.into(),
            projections,
        })
    }

    pub fn raw_bytes(&self) -> &[u8] {
        self.raw_bytes.as_slice()
    }

    pub fn projections(&self) -> &[TransformObservationProjectionV0] {
        self.projections.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransformObservationProfileErrorV0 {
    EmptyProfileId,
    DuplicateObserver { observer: TransformObserverV0 },
}

impl fmt::Display for TransformObservationProfileErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProfileId => formatter.write_str("observation profile id is empty"),
            Self::DuplicateObserver { observer } => {
                write!(
                    formatter,
                    "duplicate observation profile observer: {observer:?}"
                )
            }
        }
    }
}

impl std::error::Error for TransformObservationProfileErrorV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransformObservationProfileV0 {
    profile_id: String,
    observers: Vec<TransformObserverV0>,
}

impl TransformObservationProfileV0 {
    pub fn new(
        profile_id: impl Into<String>,
        observers: Vec<TransformObserverV0>,
    ) -> Result<Self, TransformObservationProfileErrorV0> {
        let profile_id = profile_id.into();
        if profile_id.is_empty() {
            return Err(TransformObservationProfileErrorV0::EmptyProfileId);
        }
        let mut observed = BTreeSet::new();
        for observer in &observers {
            if !observed.insert(*observer) {
                return Err(TransformObservationProfileErrorV0::DuplicateObserver {
                    observer: *observer,
                });
            }
        }
        Ok(Self {
            profile_id,
            observers,
        })
    }

    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    pub fn observers(&self) -> &[TransformObserverV0] {
        self.observers.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
#[non_exhaustive]
pub enum ObservationProjectionValueV0 {
    RawBytes(Vec<u8>),
    Contract(String),
}

pub fn project_transform_observation_v0(
    output: &TransformObservationOutputV0,
    observer: TransformObserverV0,
) -> ObservationProjectionValueV0 {
    match observer {
        TransformObserverV0::RawBytes => {
            ObservationProjectionValueV0::RawBytes(output.raw_bytes.clone())
        }
        TransformObserverV0::Contract(kind) => {
            let Some(projection) = output
                .projections
                .iter()
                .find(|projection| projection.kind == kind)
            else {
                unreachable!("TransformObservationOutputV0 validates all 16 projections")
            };
            ObservationProjectionValueV0::Contract(projection.value.clone())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransformObservationEquivalenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub profile_id: String,
    pub compared_observer_count: usize,
    pub differing_observers: Vec<TransformObserverV0>,
    pub equivalent: bool,
}

pub fn compare_transform_observation_outputs_v0(
    profile: &TransformObservationProfileV0,
    left: &TransformObservationOutputV0,
    right: &TransformObservationOutputV0,
) -> TransformObservationEquivalenceV0 {
    let differing_observers = profile
        .observers
        .iter()
        .copied()
        .filter(|observer| {
            project_transform_observation_v0(left, *observer)
                != project_transform_observation_v0(right, *observer)
        })
        .collect::<Vec<_>>();
    TransformObservationEquivalenceV0 {
        schema_version: "0",
        product: "omena-transform-cst.observation-indexed-equivalence",
        profile_id: profile.profile_id.clone(),
        compared_observer_count: profile.observers.len(),
        equivalent: differing_observers.is_empty(),
        differing_observers,
    }
}

/// Compare one authority-produced observation projection.
///
/// The caller supplies values computed by the owning parser or semantic
/// authority; this function owns the observation-indexed equality relation.
pub fn compare_transform_observation_projection_values_v0(
    profile_id: impl Into<String>,
    kind: ObservationKindV0,
    left_projection: &str,
    right_projection: &str,
) -> TransformObservationEquivalenceV0 {
    let observer = TransformObserverV0::Contract(kind);
    let equivalent = left_projection == right_projection;
    TransformObservationEquivalenceV0 {
        schema_version: "0",
        product: "omena-transform-cst.observation-indexed-equivalence",
        profile_id: profile_id.into(),
        compared_observer_count: 1,
        differing_observers: if equivalent {
            Vec::new()
        } else {
            vec![observer]
        },
        equivalent,
    }
}

/// Compare exact output bytes through the same observation relation used by
/// committed transform-independence data.
pub fn compare_raw_transform_observation_bytes_v0(
    profile_id: impl Into<String>,
    left: &[u8],
    right: &[u8],
) -> TransformObservationEquivalenceV0 {
    let equivalent = left == right;
    TransformObservationEquivalenceV0 {
        schema_version: "0",
        product: "omena-transform-cst.observation-indexed-equivalence",
        profile_id: profile_id.into(),
        compared_observer_count: 1,
        differing_observers: if equivalent {
            Vec::new()
        } else {
            vec![TransformObserverV0::RawBytes]
        },
        equivalent,
    }
}

pub fn observation_indexed_equivalent_v0(
    profile: &TransformObservationProfileV0,
    left: &TransformObservationOutputV0,
    right: &TransformObservationOutputV0,
) -> bool {
    compare_transform_observation_outputs_v0(profile, left, right).equivalent
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransformObservationMatrixV0 {
    descriptors: Vec<TransformPassDescriptorV0>,
    observation_records: Vec<TransformPassObservationRecordV0>,
}

impl TransformObservationMatrixV0 {
    pub fn descriptors(&self) -> &[TransformPassDescriptorV0] {
        self.descriptors.as_slice()
    }

    pub fn observation_records(&self) -> &[TransformPassObservationRecordV0] {
        self.observation_records.as_slice()
    }

    pub fn unknown_gap_count(&self) -> usize {
        self.observation_records
            .iter()
            .filter(|record| matches!(record.surface, PassObservationSurfaceV0::UnknownGap { .. }))
            .count()
    }
}

/// Extract the observation matrix through its public producers.
///
/// Keeping this call boundary explicit prevents the proof relation from
/// acquiring a second parser for the descriptor source table.
pub fn default_transform_observation_matrix_v0() -> TransformObservationMatrixV0 {
    TransformObservationMatrixV0 {
        descriptors: default_transform_pass_descriptors(),
        observation_records: default_transform_pass_observation_records(),
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ObservationTruthTableRowV0 {
        case_id: String,
        pass_id: String,
        observation_kind: String,
        left_projection: String,
        right_projection: String,
        expected_equivalent: bool,
    }

    fn parse_observation_kind(value: &str) -> Option<ObservationKindV0> {
        all_observation_kinds_v0().into_iter().find(|kind| {
            serde_json::to_value(kind)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .as_deref()
                == Some(value)
        })
    }

    fn output_with_override(
        raw: &str,
        kind: ObservationKindV0,
        value: &str,
    ) -> Result<TransformObservationOutputV0, TransformObservationOutputErrorV0> {
        let projections = all_observation_kinds_v0()
            .into_iter()
            .map(|candidate| {
                let projection = if candidate == kind {
                    value.to_string()
                } else {
                    format!("{candidate:?}:stable")
                };
                TransformObservationProjectionV0::new(candidate, projection)
            })
            .collect();
        TransformObservationOutputV0::new(raw.as_bytes(), projections)
    }

    #[test]
    fn public_matrix_extraction_is_complete_without_table_parsing() {
        let matrix = default_transform_observation_matrix_v0();
        assert_eq!(matrix.descriptors().len(), 44);
        assert_eq!(matrix.observation_records().len(), 44);
        assert_eq!(matrix.unknown_gap_count(), 0);
        assert_eq!(all_observation_kinds_v0().len(), OBSERVATION_KIND_COUNT_V0);
    }

    #[test]
    fn committed_truth_table_covers_every_observation_kind()
    -> Result<(), Box<dyn std::error::Error>> {
        let rows = serde_json::from_str::<Vec<ObservationTruthTableRowV0>>(include_str!(
            "../data/observation-equivalence-truth-table-v0.json"
        ))?;
        let matrix = default_transform_observation_matrix_v0();
        let mut covered = BTreeSet::new();
        for row in rows {
            let kind = parse_observation_kind(row.observation_kind.as_str())
                .ok_or_else(|| format!("unknown observation kind in {}", row.case_id))?;
            let record = matrix
                .observation_records()
                .iter()
                .find(|record| record.id == row.pass_id)
                .ok_or_else(|| format!("unknown pass in {}", row.case_id))?;
            let PassObservationSurfaceV0::Declared(contract) = &record.surface else {
                return Err(format!("unknown observation surface in {}", row.case_id).into());
            };
            assert!(
                contract.observes.contains(&kind) || contract.preserves.contains(&kind),
                "{} does not declare {:?}",
                row.pass_id,
                kind
            );
            let profile = TransformObservationProfileV0::new(
                format!("truth-table:{}", row.case_id),
                vec![TransformObserverV0::Contract(kind)],
            )?;
            let left = output_with_override("left", kind, row.left_projection.as_str())?;
            let right = output_with_override("right", kind, row.right_projection.as_str())?;
            assert_eq!(
                observation_indexed_equivalent_v0(&profile, &left, &right),
                row.expected_equivalent,
                "{}",
                row.case_id
            );
            covered.insert(kind);
        }
        assert_eq!(covered, all_observation_kinds_v0().into_iter().collect());
        Ok(())
    }

    #[test]
    fn declared_projection_relation_distinguishes_semantic_and_raw_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let left = output_with_override(
            ".a { color: red; }",
            ObservationKindV0::SelectorMatching,
            "selector:.a;winner:color=red",
        )?;
        let right = output_with_override(
            ".a{color:red}",
            ObservationKindV0::SelectorMatching,
            "selector:.a;winner:color=red",
        )?;
        let parsed = TransformObservationProfileV0::new(
            "parsed-semantics",
            vec![TransformObserverV0::Contract(
                ObservationKindV0::SelectorMatching,
            )],
        )?;
        let raw =
            TransformObservationProfileV0::new("raw-bytes", vec![TransformObserverV0::RawBytes])?;

        let parsed_equivalent = observation_indexed_equivalent_v0(&parsed, &left, &right);
        let raw_equivalent = observation_indexed_equivalent_v0(&raw, &left, &right);
        println!(
            "declaredOnly=true parsedProfile={} rawProfile={}",
            parsed_equivalent, raw_equivalent
        );
        assert!(parsed_equivalent);
        assert!(!raw_equivalent);
        Ok(())
    }

    #[test]
    fn declared_projection_relation_scopes_a_cascade_winner_change()
    -> Result<(), Box<dyn std::error::Error>> {
        let matrix = default_transform_observation_matrix_v0();
        let whitespace = matrix
            .observation_records()
            .iter()
            .find(|record| record.id == "whitespace-strip")
            .ok_or("whitespace-strip observation record missing")?;
        let PassObservationSurfaceV0::Declared(contract) = &whitespace.surface else {
            return Err("whitespace-strip observation surface is unknown".into());
        };
        assert!(
            contract
                .preserves
                .contains(&ObservationKindV0::CascadeWinner)
        );
        assert!(
            contract
                .preserves
                .contains(&ObservationKindV0::SourceMapTrace)
        );

        let left = output_with_override(
            ".a{color:red}",
            ObservationKindV0::CascadeWinner,
            "winner:red",
        )?;
        let right = output_with_override(
            ".a{color:blue}",
            ObservationKindV0::CascadeWinner,
            "winner:blue",
        )?;
        let cascade = TransformObservationProfileV0::new(
            "cascade-winner",
            vec![TransformObserverV0::Contract(
                ObservationKindV0::CascadeWinner,
            )],
        )?;
        let disjoint = TransformObservationProfileV0::new(
            "source-map-trace",
            vec![TransformObserverV0::Contract(
                ObservationKindV0::SourceMapTrace,
            )],
        )?;

        let cascade_equivalent = observation_indexed_equivalent_v0(&cascade, &left, &right);
        let disjoint_equivalent = observation_indexed_equivalent_v0(&disjoint, &left, &right);
        println!(
            "declaredOnly=true cascadeProfile={} disjointSourceMapProfile={}",
            cascade_equivalent, disjoint_equivalent
        );
        assert!(!cascade_equivalent);
        assert!(disjoint_equivalent);
        Ok(())
    }

    #[test]
    fn target_predicate_is_explicitly_read_only_and_matches_the_matrix_asymmetry() {
        let matrix = default_transform_observation_matrix_v0();
        let mut observed = 0usize;
        let mut preserved = 0usize;
        for record in matrix.observation_records() {
            let PassObservationSurfaceV0::Declared(contract) = &record.surface else {
                continue;
            };
            observed += usize::from(
                contract
                    .observes
                    .contains(&ObservationKindV0::TargetPredicate),
            );
            preserved += usize::from(
                contract
                    .preserves
                    .contains(&ObservationKindV0::TargetPredicate),
            );
        }
        assert_eq!(observed, 13);
        assert_eq!(preserved, 0);
        assert_eq!(
            observation_kind_observer_class_v0(ObservationKindV0::TargetPredicate),
            TransformObserverClassV0::ReadOnlyInput
        );
    }
}

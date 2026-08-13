//! Committed transform independence data and trace-monoid operations.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use omena_cascade_proof::{
    CanonicalRewriteAssumptionsV0, REWRITE_CERTIFICATE_SCHEMA_VERSION_V0,
    REWRITE_RULE_CATALOG_SCHEMA_VERSION_V0, RewriteCertificateEnvelopeV0, RewriteCertificateV0,
    RewriteIssuanceTokenV0, RewriteOperatorV0, RewritePatternV0, RewriteRuleCatalogV0,
    RewriteRuleV0, RewriteSideConditionKindV0, RewriteSubstitutionEntryV0, RewriteTermV0,
    SideConditionCertV0, TransformIndependenceCertV0, TransformIndependenceObservationCertRowV0,
    check_rewrite_certificate_v0,
};
use omena_transform_cst::{
    ObservationKindV0, PassAssumptionKindV0, PassObservationSurfaceV0, TransformObserverV0,
    TransformPassDescriptorV0, TransformPassKind, TransformPassObservationRecordV0,
    all_transform_pass_kinds, compare_raw_transform_observation_bytes_v0,
    default_transform_observation_matrix_v0,
};
use serde::{Deserialize, Serialize};

const INDEPENDENCE_DATA_JSON_V0: &str =
    include_str!("../data/transform-catalog-independence-v0.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformCatalogIndependenceDispositionV0 {
    Independent,
    Partial,
    Dependent,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogObservationProfileDataV0 {
    pub profile_id: String,
    pub observers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogIndependenceObservationRowV0 {
    pub fixture_id: String,
    pub input_css: String,
    pub observer: String,
    pub left_then_right: String,
    pub right_then_left: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogDescriptorEdgeJustificationV0 {
    pub kind: String,
    pub from_pass_id: String,
    pub to_pass_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogIndependenceJustificationV0 {
    pub observation_profile_id: Option<String>,
    pub descriptor_edge: Option<TransformCatalogDescriptorEdgeJustificationV0>,
    pub left_preserves_right_preconditions: Vec<String>,
    pub right_preserves_left_preconditions: Vec<String>,
    pub observation_rows: Vec<TransformCatalogIndependenceObservationRowV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogIndependenceEntryV0 {
    pub left_pass_id: String,
    pub right_pass_id: String,
    pub disposition: TransformCatalogIndependenceDispositionV0,
    pub justification: TransformCatalogIndependenceJustificationV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogIndependenceDataV0 {
    pub schema_version: String,
    pub product: String,
    pub profiles: Vec<TransformCatalogObservationProfileDataV0>,
    pub entries: Vec<TransformCatalogIndependenceEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformCatalogIndependenceErrorV0 {
    pub pair: Option<(String, String)>,
    pub message: String,
}

impl TransformCatalogIndependenceErrorV0 {
    fn global(message: impl Into<String>) -> Self {
        Self {
            pair: None,
            message: message.into(),
        }
    }

    fn pair(
        left_pass_id: impl Into<String>,
        right_pass_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            pair: Some((left_pass_id.into(), right_pass_id.into())),
            message: message.into(),
        }
    }
}

impl fmt::Display for TransformCatalogIndependenceErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((left, right)) = &self.pair {
            write!(
                formatter,
                "independence pair {left}/{right}: {}",
                self.message
            )
        } else {
            formatter.write_str(self.message.as_str())
        }
    }
}

impl std::error::Error for TransformCatalogIndependenceErrorV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCatalogScheduleEquivalenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub left_canonical_pass_ids: Vec<&'static str>,
    pub right_canonical_pass_ids: Vec<&'static str>,
    pub equivalent: bool,
}

pub fn default_transform_catalog_independence_data_v0()
-> Result<TransformCatalogIndependenceDataV0, TransformCatalogIndependenceErrorV0> {
    let data = parse_transform_catalog_independence_data_v0()?;
    validate_transform_catalog_independence_data_v0(&data)?;
    Ok(data)
}

fn parse_transform_catalog_independence_data_v0()
-> Result<TransformCatalogIndependenceDataV0, TransformCatalogIndependenceErrorV0> {
    serde_json::from_str(INDEPENDENCE_DATA_JSON_V0).map_err(|error| {
        TransformCatalogIndependenceErrorV0::global(format!(
            "committed independence data is not valid JSON: {error}"
        ))
    })
}

pub fn validate_transform_catalog_independence_data_v0(
    data: &TransformCatalogIndependenceDataV0,
) -> Result<(), TransformCatalogIndependenceErrorV0> {
    if data.schema_version != "0" {
        return Err(TransformCatalogIndependenceErrorV0::global(
            "independence data schema version is not 0",
        ));
    }
    if data.product != "omena-lawvere.transform-catalog-independence" {
        return Err(TransformCatalogIndependenceErrorV0::global(
            "independence data product is not canonical",
        ));
    }
    let mut profiles = BTreeMap::new();
    for profile in &data.profiles {
        if profile.profile_id.is_empty() || profile.observers.is_empty() {
            return Err(TransformCatalogIndependenceErrorV0::global(
                "independence observation profile is empty",
            ));
        }
        if profiles
            .insert(profile.profile_id.as_str(), profile)
            .is_some()
        {
            return Err(TransformCatalogIndependenceErrorV0::global(format!(
                "duplicate independence observation profile: {}",
                profile.profile_id
            )));
        }
        let mut observers = BTreeSet::new();
        for observer in &profile.observers {
            if parse_observer_v0(observer).is_none() || !observers.insert(observer.as_str()) {
                return Err(TransformCatalogIndependenceErrorV0::global(format!(
                    "profile {} has an unknown or duplicate observer: {observer}",
                    profile.profile_id
                )));
            }
        }
    }

    let matrix = default_transform_observation_matrix_v0();
    let mut pairs = BTreeSet::new();
    for entry in &data.entries {
        let Some(left) = pass_kind_from_id_v0(entry.left_pass_id.as_str()) else {
            return Err(pair_error(entry, "left pass id does not resolve"));
        };
        let Some(right) = pass_kind_from_id_v0(entry.right_pass_id.as_str()) else {
            return Err(pair_error(entry, "right pass id does not resolve"));
        };
        if left == right {
            return Err(pair_error(entry, "independence pair is reflexive"));
        }
        let pair_key = normalized_pair_v0(left, right);
        if !pairs.insert(pair_key) {
            return Err(pair_error(entry, "independence pair is duplicated"));
        }
        match entry.disposition {
            TransformCatalogIndependenceDispositionV0::Independent => {
                validate_independent_entry_v0(entry, left, right, &profiles, &matrix)?;
            }
            TransformCatalogIndependenceDispositionV0::Dependent => {
                let Some(edge) = &entry.justification.descriptor_edge else {
                    return Err(pair_error(
                        entry,
                        "dependent pair has no descriptor-edge justification",
                    ));
                };
                if !descriptor_edge_resolves_v0(edge, matrix.descriptors()) {
                    return Err(pair_error(
                        entry,
                        "dependent descriptor-edge justification does not resolve",
                    ));
                }
            }
            TransformCatalogIndependenceDispositionV0::Partial => {
                let profile_resolves = entry
                    .justification
                    .observation_profile_id
                    .as_deref()
                    .is_some_and(|profile_id| profiles.contains_key(profile_id));
                let edge_resolves = entry
                    .justification
                    .descriptor_edge
                    .as_ref()
                    .is_some_and(|edge| descriptor_edge_resolves_v0(edge, matrix.descriptors()));
                if !profile_resolves && !edge_resolves {
                    return Err(pair_error(
                        entry,
                        "partial justification resolves to neither a profile nor a descriptor edge",
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_independent_entry_v0(
    entry: &TransformCatalogIndependenceEntryV0,
    left: TransformPassKind,
    right: TransformPassKind,
    profiles: &BTreeMap<&str, &TransformCatalogObservationProfileDataV0>,
    matrix: &omena_transform_cst::TransformObservationMatrixV0,
) -> Result<(), TransformCatalogIndependenceErrorV0> {
    let Some(profile_id) = entry.justification.observation_profile_id.as_deref() else {
        return Err(pair_error(
            entry,
            "independent pair has no observation profile",
        ));
    };
    let Some(profile) = profiles.get(profile_id).copied() else {
        return Err(pair_error(
            entry,
            "independent observation profile does not resolve",
        ));
    };
    if entry.justification.observation_rows.is_empty() {
        return Err(pair_error(
            entry,
            "independent pair has no observation rows",
        ));
    }
    let profile_observers = profile
        .observers
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut covered = BTreeSet::new();
    let mut row_keys = BTreeSet::new();
    for row in &entry.justification.observation_rows {
        let raw_relation = compare_raw_transform_observation_bytes_v0(
            format!("independence:{}:{}", entry.left_pass_id, row.fixture_id),
            row.left_then_right.as_bytes(),
            row.right_then_left.as_bytes(),
        );
        if row.fixture_id.is_empty()
            || !profile_observers.contains(row.observer.as_str())
            || !raw_relation.equivalent
            || !row_keys.insert((row.fixture_id.as_str(), row.observer.as_str()))
        {
            return Err(pair_error(
                entry,
                "observation rows are unresolved, duplicated, or non-commuting",
            ));
        }
        covered.insert(row.observer.as_str());
    }
    if covered != profile_observers {
        return Err(pair_error(
            entry,
            "observation rows do not cover the named profile",
        ));
    }

    let left_record = observation_record_v0(left, matrix.observation_records())
        .ok_or_else(|| pair_error(entry, "left pass has no observation record"))?;
    let right_record = observation_record_v0(right, matrix.observation_records())
        .ok_or_else(|| pair_error(entry, "right pass has no observation record"))?;
    for observer in &profile.observers {
        if let Some(TransformObserverV0::Contract(kind)) = parse_observer_v0(observer)
            && (!record_preserves_v0(left_record, kind) || !record_preserves_v0(right_record, kind))
        {
            return Err(pair_error(
                entry,
                format!("profile observer is not preserved by both passes: {observer}"),
            ));
        }
    }

    let left_preconditions = record_precondition_labels_v0(left_record)?;
    let right_preconditions = record_precondition_labels_v0(right_record)?;
    if sorted_unique_v0(&entry.justification.left_preserves_right_preconditions)?
        != right_preconditions
        || sorted_unique_v0(&entry.justification.right_preserves_left_preconditions)?
            != left_preconditions
    {
        return Err(pair_error(
            entry,
            "independent pair does not preserve both passes' declared preconditions",
        ));
    }
    let disqualifying = disqualifying_descriptor_edges_v0(left, right, matrix.descriptors());
    if !disqualifying.is_empty() || entry.justification.descriptor_edge.is_some() {
        return Err(pair_error(
            entry,
            format!(
                "independent pair is disqualified by descriptor topology: {}",
                disqualifying.join(",")
            ),
        ));
    }
    Ok(())
}

fn pair_error(
    entry: &TransformCatalogIndependenceEntryV0,
    message: impl Into<String>,
) -> TransformCatalogIndependenceErrorV0 {
    TransformCatalogIndependenceErrorV0::pair(
        entry.left_pass_id.clone(),
        entry.right_pass_id.clone(),
        message,
    )
}

fn sorted_unique_v0(values: &[String]) -> Result<Vec<String>, TransformCatalogIndependenceErrorV0> {
    let mut sorted = values.to_vec();
    sorted.sort();
    let original_len = sorted.len();
    sorted.dedup();
    if sorted.len() != original_len || sorted.iter().any(String::is_empty) {
        return Err(TransformCatalogIndependenceErrorV0::global(
            "precondition preservation has an empty or duplicate value",
        ));
    }
    Ok(sorted)
}

fn record_precondition_labels_v0(
    record: &TransformPassObservationRecordV0,
) -> Result<Vec<String>, TransformCatalogIndependenceErrorV0> {
    let PassObservationSurfaceV0::Declared(contract) = &record.surface else {
        return Err(TransformCatalogIndependenceErrorV0::global(format!(
            "pass {} has an undeclared observation surface",
            record.id
        )));
    };
    let mut labels = contract
        .requires
        .iter()
        .copied()
        .map(assumption_label_v0)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    labels.sort();
    Ok(labels)
}

fn record_preserves_v0(record: &TransformPassObservationRecordV0, kind: ObservationKindV0) -> bool {
    matches!(
        &record.surface,
        PassObservationSurfaceV0::Declared(contract) if contract.preserves.contains(&kind)
    )
}

fn observation_record_v0(
    kind: TransformPassKind,
    records: &[TransformPassObservationRecordV0],
) -> Option<&TransformPassObservationRecordV0> {
    records.iter().find(|record| record.kind == kind)
}

fn descriptor_edge_resolves_v0(
    edge: &TransformCatalogDescriptorEdgeJustificationV0,
    descriptors: &[TransformPassDescriptorV0],
) -> bool {
    let Some(from) = descriptors
        .iter()
        .find(|descriptor| descriptor.id == edge.from_pass_id)
    else {
        return false;
    };
    let Some(to) = descriptors
        .iter()
        .find(|descriptor| descriptor.id == edge.to_pass_id)
    else {
        return false;
    };
    match edge.kind.as_str() {
        "dependsOn" => to.depends_on.contains(&from.id),
        "conflictsWith" => {
            from.conflicts_with.contains(&to.id) || to.conflicts_with.contains(&from.id)
        }
        _ => false,
    }
}

fn disqualifying_descriptor_edges_v0(
    left: TransformPassKind,
    right: TransformPassKind,
    descriptors: &[TransformPassDescriptorV0],
) -> Vec<String> {
    let Some(left_descriptor) = descriptors
        .iter()
        .find(|descriptor| descriptor.kind == left)
    else {
        return vec![format!("missingDescriptor:{}", left.id())];
    };
    let Some(right_descriptor) = descriptors
        .iter()
        .find(|descriptor| descriptor.kind == right)
    else {
        return vec![format!("missingDescriptor:{}", right.id())];
    };
    let mut edges = Vec::new();
    if left_descriptor.depends_on.contains(&right.id()) {
        edges.push(format!("dependsOn:{}:{}", right.id(), left.id()));
    }
    if right_descriptor.depends_on.contains(&left.id()) {
        edges.push(format!("dependsOn:{}:{}", left.id(), right.id()));
    }
    if left_descriptor.conflicts_with.contains(&right.id())
        || right_descriptor.conflicts_with.contains(&left.id())
    {
        edges.push(format!("conflictsWith:{}:{}", left.id(), right.id()));
    }
    edges
}

pub fn transform_catalog_passes_are_independent_v0(
    left: TransformPassKind,
    right: TransformPassKind,
) -> Result<bool, TransformCatalogIndependenceErrorV0> {
    let data = default_transform_catalog_independence_data_v0()?;
    Ok(entry_for_pair_v0(&data, left, right).is_some_and(|entry| {
        entry.disposition == TransformCatalogIndependenceDispositionV0::Independent
    }))
}

pub fn canonicalize_transform_catalog_schedule_v0(
    schedule: &[TransformPassKind],
) -> Result<Vec<TransformPassKind>, TransformCatalogIndependenceErrorV0> {
    let data = default_transform_catalog_independence_data_v0()?;
    let mut canonical = schedule.to_vec();
    if canonical.len() < 2 {
        return Ok(canonical);
    }
    loop {
        let mut changed = false;
        for index in 0..canonical.len() - 1 {
            let left = canonical[index];
            let right = canonical[index + 1];
            if left.id().as_bytes() > right.id().as_bytes()
                && entry_for_pair_v0(&data, left, right).is_some_and(|entry| {
                    entry.disposition == TransformCatalogIndependenceDispositionV0::Independent
                })
            {
                canonical.swap(index, index + 1);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Ok(canonical)
}

pub fn transform_catalog_schedules_equivalent_v0(
    left: &[TransformPassKind],
    right: &[TransformPassKind],
) -> Result<TransformCatalogScheduleEquivalenceV0, TransformCatalogIndependenceErrorV0> {
    let left_canonical = canonicalize_transform_catalog_schedule_v0(left)?;
    let right_canonical = canonicalize_transform_catalog_schedule_v0(right)?;
    Ok(TransformCatalogScheduleEquivalenceV0 {
        schema_version: "0",
        product: "omena-lawvere.transform-catalog-schedule-equivalence",
        left_canonical_pass_ids: left_canonical.iter().map(|kind| kind.id()).collect(),
        right_canonical_pass_ids: right_canonical.iter().map(|kind| kind.id()).collect(),
        equivalent: left_canonical == right_canonical,
    })
}

pub fn transform_catalog_independence_layers_v0(
    requested: &[TransformPassKind],
) -> Result<Vec<Vec<TransformPassKind>>, TransformCatalogIndependenceErrorV0> {
    let data = default_transform_catalog_independence_data_v0()?;
    Ok(transform_catalog_independence_layers_from_data_v0(
        requested, &data,
    ))
}

pub(crate) fn transform_catalog_independence_layers_from_data_v0(
    requested: &[TransformPassKind],
    data: &TransformCatalogIndependenceDataV0,
) -> Vec<Vec<TransformPassKind>> {
    let mut layers: Vec<Vec<TransformPassKind>> = Vec::new();
    for pass in requested {
        let mut placed = false;
        for layer in &mut layers {
            if layer.iter().all(|candidate| {
                entry_for_pair_v0(data, *candidate, *pass).is_some_and(|entry| {
                    entry.disposition == TransformCatalogIndependenceDispositionV0::Independent
                })
            }) {
                layer.push(*pass);
                placed = true;
                break;
            }
        }
        if !placed {
            layers.push(vec![*pass]);
        }
    }
    layers
}

pub(crate) fn checked_adjacent_swap_token_v0(
    left: TransformPassKind,
    right: TransformPassKind,
    data: &TransformCatalogIndependenceDataV0,
) -> Result<RewriteIssuanceTokenV0, TransformCatalogIndependenceErrorV0> {
    let Some(entry) = entry_for_pair_v0(data, left, right) else {
        return Err(TransformCatalogIndependenceErrorV0::pair(
            left.id(),
            right.id(),
            "pair is absent from committed independence data",
        ));
    };
    if entry.disposition != TransformCatalogIndependenceDispositionV0::Independent {
        return Err(pair_error(entry, "pair is not declared independent"));
    }
    let Some(profile_id) = entry.justification.observation_profile_id.as_deref() else {
        return Err(pair_error(entry, "independent pair has no profile"));
    };
    let Some(profile) = data
        .profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
    else {
        return Err(pair_error(entry, "independent profile does not resolve"));
    };
    let matrix = default_transform_observation_matrix_v0();
    let left_record = observation_record_v0(left, matrix.observation_records())
        .ok_or_else(|| pair_error(entry, "left observation record is absent"))?;
    let right_record = observation_record_v0(right, matrix.observation_records())
        .ok_or_else(|| pair_error(entry, "right observation record is absent"))?;
    let before = adjacent_schedule_pair_term_v0(left, right);
    let after = adjacent_schedule_pair_term_v0(right, left);
    let catalog = adjacent_schedule_swap_catalog_v0();
    let certificate = RewriteCertificateEnvelopeV0 {
        schema_version: REWRITE_CERTIFICATE_SCHEMA_VERSION_V0.to_owned(),
        max_depth: 1,
        max_nodes: 1,
        certificate: RewriteCertificateV0::Rewrite {
            rule_id: "adjacent-independent-schedule-swap-v0".to_owned(),
            substitution: vec![
                RewriteSubstitutionEntryV0 {
                    variable: "left".to_owned(),
                    term: RewriteTermV0::atom(left.id()),
                },
                RewriteSubstitutionEntryV0 {
                    variable: "right".to_owned(),
                    term: RewriteTermV0::atom(right.id()),
                },
            ],
            side_condition: SideConditionCertV0::TransformIndependence {
                certificate: Box::new(TransformIndependenceCertV0 {
                    left_pass_id: left.id().to_owned(),
                    right_pass_id: right.id().to_owned(),
                    observation_profile_id: profile.profile_id.clone(),
                    profile_observers: profile.observers.clone(),
                    observation_rows: entry
                        .justification
                        .observation_rows
                        .iter()
                        .map(|row| TransformIndependenceObservationCertRowV0 {
                            fixture_id: row.fixture_id.clone(),
                            observer: row.observer.clone(),
                            left_then_right: row.left_then_right.clone(),
                            right_then_left: row.right_then_left.clone(),
                        })
                        .collect(),
                    left_preconditions: record_precondition_labels_v0(left_record)?,
                    right_preconditions: record_precondition_labels_v0(right_record)?,
                    left_preserves_right_preconditions: entry
                        .justification
                        .left_preserves_right_preconditions
                        .clone(),
                    right_preserves_left_preconditions: entry
                        .justification
                        .right_preserves_left_preconditions
                        .clone(),
                    disqualifying_descriptor_edges: disqualifying_descriptor_edges_v0(
                        left,
                        right,
                        matrix.descriptors(),
                    ),
                }),
            },
        },
    };
    check_rewrite_certificate_v0(
        &before,
        &after,
        &catalog,
        &certificate,
        &CanonicalRewriteAssumptionsV0::default(),
    )
    .map_err(|rejection| {
        pair_error(
            entry,
            format!("S1 checker rejected reorder certificate: {rejection:?}"),
        )
    })
    .and_then(|token| {
        if token.matches_endpoints_v0(&before, &after) && token.matches_catalog_v0(&catalog) {
            Ok(token)
        } else {
            Err(pair_error(
                entry,
                "S1 checker token is not bound to the adjacent swap endpoints",
            ))
        }
    })
}

pub(crate) fn adjacent_schedule_pair_term_v0(
    left: TransformPassKind,
    right: TransformPassKind,
) -> RewriteTermV0 {
    RewriteTermV0::apply(
        "adjacentSchedulePair",
        vec![
            RewriteTermV0::atom(left.id()),
            RewriteTermV0::atom(right.id()),
        ],
    )
}

pub(crate) fn adjacent_schedule_swap_catalog_v0() -> RewriteRuleCatalogV0 {
    RewriteRuleCatalogV0 {
        schema_version: REWRITE_RULE_CATALOG_SCHEMA_VERSION_V0.to_owned(),
        operators: vec![RewriteOperatorV0 {
            operator: "adjacentSchedulePair".to_owned(),
            arity: 2,
        }],
        rules: vec![RewriteRuleV0 {
            rule_id: "adjacent-independent-schedule-swap-v0".to_owned(),
            before_pattern: RewritePatternV0::apply(
                "adjacentSchedulePair",
                vec![
                    RewritePatternV0::variable("left"),
                    RewritePatternV0::variable("right"),
                ],
            ),
            after_pattern: RewritePatternV0::apply(
                "adjacentSchedulePair",
                vec![
                    RewritePatternV0::variable("right"),
                    RewritePatternV0::variable("left"),
                ],
            ),
            side_condition_kind: RewriteSideConditionKindV0::TransformIndependence,
        }],
    }
}

fn entry_for_pair_v0(
    data: &TransformCatalogIndependenceDataV0,
    left: TransformPassKind,
    right: TransformPassKind,
) -> Option<&TransformCatalogIndependenceEntryV0> {
    data.entries.iter().find(|entry| {
        (entry.left_pass_id == left.id() && entry.right_pass_id == right.id())
            || (entry.left_pass_id == right.id() && entry.right_pass_id == left.id())
    })
}

fn normalized_pair_v0(
    left: TransformPassKind,
    right: TransformPassKind,
) -> (&'static str, &'static str) {
    if left.id().as_bytes() <= right.id().as_bytes() {
        (left.id(), right.id())
    } else {
        (right.id(), left.id())
    }
}

fn pass_kind_from_id_v0(pass_id: &str) -> Option<TransformPassKind> {
    all_transform_pass_kinds()
        .into_iter()
        .find(|kind| kind.id() == pass_id)
}

fn parse_observer_v0(value: &str) -> Option<TransformObserverV0> {
    match value {
        "rawBytes" => Some(TransformObserverV0::RawBytes),
        "selectorMatching" => Some(TransformObserverV0::Contract(
            ObservationKindV0::SelectorMatching,
        )),
        "cascadeWinner" => Some(TransformObserverV0::Contract(
            ObservationKindV0::CascadeWinner,
        )),
        "cascadeWinnerEquality" => Some(TransformObserverV0::Contract(
            ObservationKindV0::CascadeWinnerEquality,
        )),
        "exportedClassNames" => Some(TransformObserverV0::Contract(
            ObservationKindV0::ExportedClassNames,
        )),
        "customPropertyComputedValue" => Some(TransformObserverV0::Contract(
            ObservationKindV0::CustomPropertyComputedValue,
        )),
        "keyframesReachability" => Some(TransformObserverV0::Contract(
            ObservationKindV0::KeyframesReachability,
        )),
        "sourceMapTrace" => Some(TransformObserverV0::Contract(
            ObservationKindV0::SourceMapTrace,
        )),
        "layerRank" => Some(TransformObserverV0::Contract(ObservationKindV0::LayerRank)),
        "specificity" => Some(TransformObserverV0::Contract(
            ObservationKindV0::Specificity,
        )),
        "inheritance" => Some(TransformObserverV0::Contract(
            ObservationKindV0::Inheritance,
        )),
        "declarationOrder" => Some(TransformObserverV0::Contract(
            ObservationKindV0::DeclarationOrder,
        )),
        "targetPredicate" => Some(TransformObserverV0::Contract(
            ObservationKindV0::TargetPredicate,
        )),
        "moduleResolution" => Some(TransformObserverV0::Contract(
            ObservationKindV0::ModuleResolution,
        )),
        "importContext" => Some(TransformObserverV0::Contract(
            ObservationKindV0::ImportContext,
        )),
        "valueGraphReachability" => Some(TransformObserverV0::Contract(
            ObservationKindV0::ValueGraphReachability,
        )),
        "semanticMarker" => Some(TransformObserverV0::Contract(
            ObservationKindV0::SemanticMarker,
        )),
        _ => None,
    }
}

const fn assumption_label_v0(kind: PassAssumptionKindV0) -> &'static str {
    match kind {
        PassAssumptionKindV0::TokenBoundary => "tokenBoundary",
        PassAssumptionKindV0::SourceMapProvenance => "sourceMapProvenance",
        PassAssumptionKindV0::EquivalentLiteralValue => "equivalentLiteralValue",
        PassAssumptionKindV0::SelectorSpecificity => "selectorSpecificity",
        PassAssumptionKindV0::LonghandShorthandEquivalence => "longhandShorthandEquivalence",
        PassAssumptionKindV0::DeclarationOrder => "declarationOrder",
        PassAssumptionKindV0::TargetEnvironment => "targetEnvironment",
        PassAssumptionKindV0::Directionality => "directionality",
        PassAssumptionKindV0::NestedSelectorExpansion => "nestedSelectorExpansion",
        PassAssumptionKindV0::ScopedMatching => "scopedMatching",
        PassAssumptionKindV0::LayerOrder => "layerOrder",
        PassAssumptionKindV0::StaticPredicate => "staticPredicate",
        PassAssumptionKindV0::ImportWrapperProvenance => "importWrapperProvenance",
        PassAssumptionKindV0::ModuleNamespace => "moduleNamespace",
        PassAssumptionKindV0::SelectorIdentityMap => "selectorIdentityMap",
        PassAssumptionKindV0::ValueGraph => "valueGraph",
        PassAssumptionKindV0::CustomPropertyFixedPoint => "customPropertyFixedPoint",
        PassAssumptionKindV0::ClosedWorldReachability => "closedWorldReachability",
        PassAssumptionKindV0::EmissionTrace => "emissionTrace",
        PassAssumptionKindV0::SemanticMarkerRetention => "semanticMarkerRetention",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, VecDeque};

    use super::*;

    const BRUTE_FORCE_BOUND_V0: usize = 4;

    fn oracle_reachable_schedules_v0(
        start: Vec<TransformPassKind>,
    ) -> Result<BTreeSet<Vec<TransformPassKind>>, TransformCatalogIndependenceErrorV0> {
        assert!(start.len() <= BRUTE_FORCE_BOUND_V0);
        let mut reached = BTreeSet::from([start.clone()]);
        let mut queue = VecDeque::from([start]);
        while let Some(schedule) = queue.pop_front() {
            for index in 0..schedule.len().saturating_sub(1) {
                if transform_catalog_passes_are_independent_v0(
                    schedule[index],
                    schedule[index + 1],
                )? {
                    let mut adjacent_swap = schedule.clone();
                    adjacent_swap.swap(index, index + 1);
                    if reached.insert(adjacent_swap.clone()) {
                        queue.push_back(adjacent_swap);
                    }
                }
            }
        }
        Ok(reached)
    }

    fn oracle_permutations_v0(values: &[TransformPassKind]) -> Vec<Vec<TransformPassKind>> {
        fn visit(
            values: &mut Vec<TransformPassKind>,
            index: usize,
            output: &mut Vec<Vec<TransformPassKind>>,
        ) {
            if index == values.len() {
                output.push(values.clone());
                return;
            }
            for candidate in index..values.len() {
                values.swap(index, candidate);
                visit(values, index + 1, output);
                values.swap(index, candidate);
            }
        }
        assert!(values.len() <= BRUTE_FORCE_BOUND_V0);
        let mut values = values.to_vec();
        let mut output = Vec::new();
        visit(&mut values, 0, &mut output);
        output
    }

    #[test]
    fn committed_independence_data_resolves_profiles_edges_and_preconditions()
    -> Result<(), TransformCatalogIndependenceErrorV0> {
        let data = default_transform_catalog_independence_data_v0()?;
        assert_eq!(data.entries.len(), 2);
        assert_eq!(
            data.entries
                .iter()
                .filter(|entry| entry.disposition
                    == TransformCatalogIndependenceDispositionV0::Independent)
                .count(),
            1
        );
        Ok(())
    }

    #[test]
    fn canonical_schedule_agrees_with_independent_bounded_oracle()
    -> Result<(), TransformCatalogIndependenceErrorV0> {
        let universe = vec![
            TransformPassKind::NumberCompression,
            TransformPassKind::ColorCompression,
            TransformPassKind::CommentStrip,
            TransformPassKind::WhitespaceStrip,
        ];
        let left = universe.clone();
        let mut right = left.clone();
        right.swap(0, 1);
        let reachable = oracle_reachable_schedules_v0(left.clone())?;
        assert!(reachable.contains(&right));
        let report = transform_catalog_schedules_equivalent_v0(&left, &right)?;
        assert!(report.equivalent);

        let candidates = oracle_permutations_v0(&universe);
        for candidate_left in &candidates {
            let reachable = oracle_reachable_schedules_v0(candidate_left.clone())?;
            for candidate_right in &candidates {
                assert_eq!(
                    transform_catalog_schedules_equivalent_v0(candidate_left, candidate_right)?
                        .equivalent,
                    reachable.contains(candidate_right),
                    "canonical/oracle disagreement for {:?} vs {:?}",
                    candidate_left,
                    candidate_right,
                );
            }
        }
        println!(
            "R18 bound={} permutations={} transposedPair=number-compression/color-compression canonicalOracleAgreement=true",
            BRUTE_FORCE_BOUND_V0,
            candidates.len()
        );
        Ok(())
    }

    #[test]
    fn dependent_pair_injection_is_rejected_by_data_and_s1_checker()
    -> Result<(), TransformCatalogIndependenceErrorV0> {
        let mut data = parse_transform_catalog_independence_data_v0()?;
        let Some(dependent) = data.entries.iter_mut().find(|entry| {
            entry.left_pass_id == TransformPassKind::ColorMixLowering.id()
                && entry.right_pass_id == TransformPassKind::ColorFunctionLowering.id()
        }) else {
            return Err(TransformCatalogIndependenceErrorV0::global(
                "dependent fixture pair should exist",
            ));
        };
        dependent.disposition = TransformCatalogIndependenceDispositionV0::Independent;
        dependent.justification.observation_profile_id = Some("exact-emission-bytes-v0".to_owned());
        dependent.justification.left_preserves_right_preconditions =
            vec!["targetEnvironment".to_owned()];
        dependent.justification.right_preserves_left_preconditions =
            vec!["targetEnvironment".to_owned()];
        dependent.justification.observation_rows =
            vec![TransformCatalogIndependenceObservationRowV0 {
                fixture_id: "nested-color-divergence".to_owned(),
                input_css: ".card { color: color-mix(in srgb, color(srgb 1 0 0), blue); }"
                    .to_owned(),
                observer: "rawBytes".to_owned(),
                left_then_right: ".card { color: color-mix(in srgb, rgb(255 0 0), blue); }"
                    .to_owned(),
                // Malicious producer claim: hide the measured right-order divergence. The
                // descriptor-conflict half must still reject the declaration and certificate.
                right_then_left: ".card { color: color-mix(in srgb, rgb(255 0 0), blue); }"
                    .to_owned(),
            }];

        let validation = validate_transform_catalog_independence_data_v0(&data);
        assert!(validation.as_ref().is_err_and(|error| {
            let message = error.to_string();
            message.contains("color-mix-lowering/color-function-lowering")
                && message.contains("descriptor topology")
        }));
        let checker = checked_adjacent_swap_token_v0(
            TransformPassKind::ColorMixLowering,
            TransformPassKind::ColorFunctionLowering,
            &data,
        );
        assert!(checker.as_ref().is_err_and(|error| {
            let message = error.to_string();
            message.contains("S1 checker rejected")
                && message.contains("color-mix-lowering/color-function-lowering")
        }));
        println!(
            "R19 pair=color-mix-lowering/color-function-lowering oracleOutputsDiffer=true dataValidation={validation:?} checker={checker:?}"
        );
        Ok(())
    }

    #[test]
    fn empty_independence_data_collapses_parallel_width_to_one()
    -> Result<(), TransformCatalogIndependenceErrorV0> {
        let mut data = parse_transform_catalog_independence_data_v0()?;
        data.entries.clear();
        let layers = transform_catalog_independence_layers_from_data_v0(
            &[
                TransformPassKind::NumberCompression,
                TransformPassKind::ColorCompression,
            ],
            &data,
        );
        assert_eq!(layers.len(), 2);
        assert_eq!(layers.iter().map(Vec::len).max(), Some(1));
        println!("R20 WIRING_CHECK entries=0 layers=2 maxParallelWidth=1");
        Ok(())
    }

    #[test]
    fn gate_supplied_independence_data_satisfies_product_invariants()
    -> Result<(), TransformCatalogIndependenceErrorV0> {
        let source = std::env::var("OMENA_PROOF_KERNEL_INDEPENDENCE_JSON")
            .unwrap_or_else(|_| INDEPENDENCE_DATA_JSON_V0.to_owned());
        let data: TransformCatalogIndependenceDataV0 = serde_json::from_str(source.as_str())
            .map_err(|error| {
                TransformCatalogIndependenceErrorV0::global(format!(
                    "gate-supplied independence data is not valid JSON: {error}"
                ))
            })?;
        validate_transform_catalog_independence_data_v0(&data)?;
        let independent_pairs = data
            .entries
            .iter()
            .filter(|entry| {
                entry.disposition == TransformCatalogIndependenceDispositionV0::Independent
            })
            .count();
        assert!(
            independent_pairs > 0,
            "gate-supplied data lost every independent pair"
        );
        let layers = transform_catalog_independence_layers_from_data_v0(
            &[
                TransformPassKind::NumberCompression,
                TransformPassKind::ColorCompression,
            ],
            &data,
        );
        let max_parallel_width = layers.iter().map(Vec::len).max().unwrap_or_default();
        assert!(
            max_parallel_width > 1,
            "gate-supplied independence data cannot justify a parallel layer"
        );
        println!(
            "injectedDataValidation=Ok independentPairs={independent_pairs} maxParallelWidth={max_parallel_width}"
        );
        Ok(())
    }
}

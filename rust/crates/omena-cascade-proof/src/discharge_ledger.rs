use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};

use crate::CanonicalSmtInputV0;

pub const DISCHARGE_LEDGER_SCHEMA_VERSION_V1: &str = "1";
pub const DISCHARGE_LEDGER_PRODUCT_V1: &str = "omena-cascade-proof.discharge-ledger";
const DISCHARGE_LEDGER_LOOKUP_PRODUCT_V0: &str = "omena-cascade-proof.discharge-ledger.lookup";
const DISCHARGE_LEDGER_SOURCE_V1: &str = include_str!("../discharge-ledger/ledger.v1.json");

const DISCHARGE_LEDGER_THEORY_SIGNATURE_HASH_V1: &str =
    "af0723fe418abe97660ae5c057cd8c7dbd3202d4deae9060afeedf7b69dc055b";
const DISCHARGE_LEDGER_SPEC_DIGEST_V1: &str =
    "4360d5b5e3bd0afeb02df5da14042cab9abe3c7c9e9c179fef3018eca66caacb";
const DISCHARGE_LEDGER_ENCODER_CONTENT_HASH_V1: &str =
    "cd6f6aee560fc4e5a09a8dc4f5903d0120fb99dc0407d069d49a4c260af608dc";
const DISCHARGE_LEDGER_SOLVER_VERSION_V1: &str = "z3-crate-0.20.2-gh-release";

static DISCHARGE_LEDGER_INDEX_V1: OnceLock<
    Result<DischargeLedgerIndexV1, DischargeLedgerIndexErrorV0>,
> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DischargeLedgerLookupStatusV0 {
    Matched,
    Missing,
    Stale,
    Malformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DischargeLedgerVerdictV0 {
    Accepted,
    Rejected,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DischargeLedgerLookupV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub cell_key: String,
    pub status: DischargeLedgerLookupStatusV0,
    pub obligation_family: Option<String>,
    pub cell_family: Option<String>,
    pub verdict: Option<DischargeLedgerVerdictV0>,
    pub boundedness_kind: Option<String>,
    pub floor_reason: Option<&'static str>,
}

impl DischargeLedgerLookupV0 {
    pub fn can_apply_family_stamp(&self) -> bool {
        self.status == DischargeLedgerLookupStatusV0::Matched
            && self.verdict == Some(DischargeLedgerVerdictV0::Accepted)
    }
}

#[derive(Debug, Clone)]
struct DischargeLedgerIndexV1 {
    entries: HashMap<String, DischargeLedgerEntryV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DischargeLedgerIndexErrorV0 {
    Malformed,
    Stale,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DischargeLedgerV1 {
    schema_version: String,
    product: String,
    pins: DischargeLedgerPinsV1,
    entries: Vec<DischargeLedgerEntryV1>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DischargeLedgerPinsV1 {
    theory_signature_hash: String,
    spec_digest: String,
    encoder_content_hash: String,
    solver_version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DischargeLedgerEntryV1 {
    obligation_family: String,
    cell_family: String,
    cell_key: String,
    verdict: DischargeLedgerVerdictV0,
    boundedness: DischargeBoundednessV1,
    reference_matches_solver: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DischargeBoundednessV1 {
    kind: String,
}

pub fn discharge_ledger_cell_key_v0(input: &CanonicalSmtInputV0) -> String {
    blake3::hash(input.smtlib2_script.as_bytes())
        .to_hex()
        .to_string()
}

pub fn lookup_discharge_ledger_entry_v0(input: &CanonicalSmtInputV0) -> DischargeLedgerLookupV0 {
    let cell_key = discharge_ledger_cell_key_v0(input);
    let index = DISCHARGE_LEDGER_INDEX_V1
        .get_or_init(|| build_discharge_ledger_index_v1(DISCHARGE_LEDGER_SOURCE_V1));
    lookup_discharge_ledger_cell_v1(cell_key, index.as_ref())
}

fn lookup_discharge_ledger_cell_v1(
    cell_key: String,
    index: Result<&DischargeLedgerIndexV1, &DischargeLedgerIndexErrorV0>,
) -> DischargeLedgerLookupV0 {
    let Ok(index) = index else {
        let (status, reason) = match index.err().copied() {
            Some(DischargeLedgerIndexErrorV0::Stale) => (
                DischargeLedgerLookupStatusV0::Stale,
                "ledger pins do not match the runtime",
            ),
            _ => (
                DischargeLedgerLookupStatusV0::Malformed,
                "ledger artifact cannot be read",
            ),
        };
        return floor_lookup(cell_key, status, reason);
    };
    let Some(entry) = index.entries.get(&cell_key) else {
        return floor_lookup(
            cell_key,
            DischargeLedgerLookupStatusV0::Missing,
            "ledger cell is absent",
        );
    };
    if !entry.reference_matches_solver {
        return floor_lookup(
            cell_key,
            DischargeLedgerLookupStatusV0::Stale,
            "ledger entry has no reference agreement",
        );
    }
    DischargeLedgerLookupV0 {
        schema_version: "0",
        product: DISCHARGE_LEDGER_LOOKUP_PRODUCT_V0,
        cell_key,
        status: DischargeLedgerLookupStatusV0::Matched,
        obligation_family: Some(entry.obligation_family.clone()),
        cell_family: Some(entry.cell_family.clone()),
        verdict: Some(entry.verdict),
        boundedness_kind: Some(entry.boundedness.kind.clone()),
        floor_reason: (entry.verdict != DischargeLedgerVerdictV0::Accepted)
            .then_some("ledger cell is not an accepted discharge"),
    }
}

fn floor_lookup(
    cell_key: String,
    status: DischargeLedgerLookupStatusV0,
    reason: &'static str,
) -> DischargeLedgerLookupV0 {
    DischargeLedgerLookupV0 {
        schema_version: "0",
        product: DISCHARGE_LEDGER_LOOKUP_PRODUCT_V0,
        cell_key,
        status,
        obligation_family: None,
        cell_family: None,
        verdict: None,
        boundedness_kind: None,
        floor_reason: Some(reason),
    }
}

fn build_discharge_ledger_index_v1(
    source: &str,
) -> Result<DischargeLedgerIndexV1, DischargeLedgerIndexErrorV0> {
    let ledger = serde_json::from_str::<DischargeLedgerV1>(source)
        .map_err(|_| DischargeLedgerIndexErrorV0::Malformed)?;
    if ledger.schema_version != DISCHARGE_LEDGER_SCHEMA_VERSION_V1
        || ledger.product != DISCHARGE_LEDGER_PRODUCT_V1
    {
        return Err(DischargeLedgerIndexErrorV0::Malformed);
    }
    if !pins_match_current_runtime_v1(&ledger.pins) {
        return Err(DischargeLedgerIndexErrorV0::Stale);
    }
    let mut entries = HashMap::with_capacity(ledger.entries.len());
    for entry in ledger.entries {
        entries.insert(entry.cell_key.clone(), entry);
    }
    Ok(DischargeLedgerIndexV1 { entries })
}

fn pins_match_current_runtime_v1(pins: &DischargeLedgerPinsV1) -> bool {
    pins.theory_signature_hash == DISCHARGE_LEDGER_THEORY_SIGNATURE_HASH_V1
        && pins.spec_digest == DISCHARGE_LEDGER_SPEC_DIGEST_V1
        && pins.encoder_content_hash == DISCHARGE_LEDGER_ENCODER_CONTENT_HASH_V1
        && pins.solver_version == DISCHARGE_LEDGER_SOLVER_VERSION_V1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        StubSmtBackendV0, smt_prove_longhand_merge_v0, smt_prove_scope_flatten_candidate_v0,
    };
    use omena_cascade::{LonghandMergeInputV0, ScopeFlattenInputV0};

    #[test]
    fn ledger_lookup_matches_committed_longhand_merge_cell() {
        let longhands = vec![
            LonghandMergeInputV0 {
                property: "margin-top".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 1,
            },
            LonghandMergeInputV0 {
                property: "margin-right".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 2,
            },
            LonghandMergeInputV0 {
                property: "margin-bottom".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 3,
            },
            LonghandMergeInputV0 {
                property: "margin-left".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 4,
            },
        ];
        let proof = smt_prove_longhand_merge_v0(
            "margin",
            &["margin-top", "margin-right", "margin-bottom", "margin-left"],
            &longhands,
            &StubSmtBackendV0::default(),
        );
        let lookup = lookup_discharge_ledger_entry_v0(&proof.canonical_input);

        assert_eq!(lookup.status, DischargeLedgerLookupStatusV0::Matched);
        assert_eq!(lookup.cell_family.as_deref(), Some("longhandMerge"));
        assert_eq!(lookup.verdict, Some(DischargeLedgerVerdictV0::Accepted));
        assert!(lookup.can_apply_family_stamp());
    }

    #[test]
    fn ledger_lookup_falls_back_when_pins_drift() {
        let proof = smt_prove_scope_flatten_candidate_v0(
            ScopeFlattenInputV0 {
                root_selector: ":root".to_string(),
                limit_selector: None,
                scoped_rule_count: 1,
                peer_scope_count: 0,
                competing_unscoped_rule_count: 0,
                inside_layer: false,
            },
            &StubSmtBackendV0::default(),
        );
        let stale_source = DISCHARGE_LEDGER_SOURCE_V1.replacen(
            DISCHARGE_LEDGER_SPEC_DIGEST_V1,
            "0000000000000000000000000000000000000000000000000000000000000000",
            1,
        );
        let cell_key = discharge_ledger_cell_key_v0(&proof.canonical_input);
        let index = build_discharge_ledger_index_v1(&stale_source);
        let lookup = lookup_discharge_ledger_cell_v1(cell_key, index.as_ref());

        assert_eq!(lookup.status, DischargeLedgerLookupStatusV0::Stale);
        assert_eq!(
            lookup.floor_reason,
            Some("ledger pins do not match the runtime")
        );
        assert!(!lookup.can_apply_family_stamp());
    }

    #[test]
    fn ledger_lookup_falls_back_for_unknown_cells() {
        let input = crate::canonical_smt_input_v0(
            "not-in-ledger",
            "not_in_ledger",
            vec!["require:not-indexed=true".to_string()],
        );
        let lookup = lookup_discharge_ledger_entry_v0(&input);

        assert_eq!(lookup.status, DischargeLedgerLookupStatusV0::Missing);
        assert_eq!(lookup.floor_reason, Some("ledger cell is absent"));
        assert!(!lookup.can_apply_family_stamp());
    }
}

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::external_corpus_envelope_idl_generated::{
    ExternalCorpusDialectV1Json, ExternalCorpusDifferentialManifestV1Json,
    ExternalCorpusEnvelopeV1Json, ExternalCorpusExpectationKindV1Json, ExternalCorpusStageV1Json,
};

const OSS_CORPUS_FARM_MANIFEST_SOURCE: &str = include_str!("../oss-corpus-farm/manifest.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffOssCorpusFarmManifestReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub entry_count: usize,
    pub repository_count: usize,
    pub dialect_count: usize,
    pub all_entries_follow_generated_envelope_shape: bool,
    pub all_entries_stage1_advisory: bool,
    pub all_entries_out_of_scope: bool,
    pub all_entries_have_permissive_spdx: bool,
    pub all_entry_pins_are_sha_locked: bool,
    pub all_recorded_shas_match_source_pins: bool,
    pub all_sparse_paths_are_bounded: bool,
    pub all_chunk_hashes_match: bool,
    pub dialects: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OssCorpusFarmManifestV0 {
    schema_version: String,
    product: String,
    mode: String,
    fixtures: Vec<serde_json::Value>,
}

pub fn summarize_oss_corpus_farm_manifest_v0() -> OmenaDiffOssCorpusFarmManifestReportV0 {
    let Some(manifest) = parse_oss_corpus_farm_manifest() else {
        return empty_oss_corpus_farm_manifest_report();
    };
    assert_eq!(manifest.schema_version, "0");
    assert_eq!(manifest.product, "omena-diff-test.oss-corpus-farm.manifest");
    assert_eq!(manifest.mode, "pinned-repo-fact-set");
    let Ok(generated_manifest) = serde_json::from_str::<ExternalCorpusDifferentialManifestV1Json>(
        OSS_CORPUS_FARM_MANIFEST_SOURCE,
    ) else {
        return empty_oss_corpus_farm_manifest_report();
    };
    assert_eq!(generated_manifest.product, manifest.product);
    assert_eq!(generated_manifest.fixtures.len(), manifest.fixtures.len());

    let entries = parse_oss_corpus_farm_entries(&manifest);
    let all_entries_follow_generated_envelope_shape = entries.len() == manifest.fixtures.len();
    let repositories = entries
        .iter()
        .map(|entry| entry.source.repository.as_str())
        .collect::<BTreeSet<_>>();
    let dialects = entries
        .iter()
        .filter_map(|entry| entry.dialect.as_ref())
        .map(dialect_label)
        .collect::<BTreeSet<_>>();
    let all_entries_stage1_advisory = entries
        .iter()
        .all(|entry| matches!(entry.stage, ExternalCorpusStageV1Json::Stage1Advisory));
    let all_entries_out_of_scope = entries.iter().all(|entry| {
        matches!(
            entry.expectation_kind,
            Some(ExternalCorpusExpectationKindV1Json::OutOfScope)
        )
    });
    let all_entries_have_permissive_spdx = entries.iter().all(entry_records_permissive_license);
    let all_entry_pins_are_sha_locked = entries
        .iter()
        .all(|entry| source_pin_sha(entry).is_some_and(is_sha));
    let all_recorded_shas_match_source_pins = entries.iter().all(recorded_sha_matches_source_pin);
    let all_sparse_paths_are_bounded = entries.iter().all(|entry| {
        !entry.source.sparse_paths.is_empty()
            && entry
                .source
                .sparse_paths
                .iter()
                .all(|path| is_bounded_sparse_path(path))
    });
    let all_chunk_hashes_match = entries.iter().all(entry_chunk_hashes_match);

    OmenaDiffOssCorpusFarmManifestReportV0 {
        schema_version: "0",
        product: "omena-diff-test.oss-corpus-farm.manifest-report",
        entry_count: entries.len(),
        repository_count: repositories.len(),
        dialect_count: dialects.len(),
        all_entries_follow_generated_envelope_shape,
        all_entries_stage1_advisory,
        all_entries_out_of_scope,
        all_entries_have_permissive_spdx,
        all_entry_pins_are_sha_locked,
        all_recorded_shas_match_source_pins,
        all_sparse_paths_are_bounded,
        all_chunk_hashes_match,
        dialects: dialects.into_iter().collect(),
    }
}

fn empty_oss_corpus_farm_manifest_report() -> OmenaDiffOssCorpusFarmManifestReportV0 {
    OmenaDiffOssCorpusFarmManifestReportV0 {
        schema_version: "0",
        product: "omena-diff-test.oss-corpus-farm.manifest-report",
        entry_count: 0,
        repository_count: 0,
        dialect_count: 0,
        all_entries_follow_generated_envelope_shape: false,
        all_entries_stage1_advisory: false,
        all_entries_out_of_scope: false,
        all_entries_have_permissive_spdx: false,
        all_entry_pins_are_sha_locked: false,
        all_recorded_shas_match_source_pins: false,
        all_sparse_paths_are_bounded: false,
        all_chunk_hashes_match: false,
        dialects: vec![],
    }
}

fn parse_oss_corpus_farm_manifest() -> Option<OssCorpusFarmManifestV0> {
    serde_json::from_str(OSS_CORPUS_FARM_MANIFEST_SOURCE).ok()
}

fn parse_oss_corpus_farm_entries(
    manifest: &OssCorpusFarmManifestV0,
) -> Vec<ExternalCorpusEnvelopeV1Json> {
    manifest
        .fixtures
        .iter()
        .filter_map(|fixture| {
            serde_json::from_value::<ExternalCorpusEnvelopeV1Json>(fixture.clone()).ok()
        })
        .collect()
}

fn source_pin_sha(entry: &ExternalCorpusEnvelopeV1Json) -> Option<&str> {
    entry.source.pin.rsplit_once('@').map(|(_, sha)| sha)
}

fn is_sha(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn recorded_sha_matches_source_pin(entry: &ExternalCorpusEnvelopeV1Json) -> bool {
    let Some(source_sha) = source_pin_sha(entry) else {
        return false;
    };
    let refs = entry
        .provenance
        .as_ref()
        .map(|provenance| provenance.oracle_pin_refs.as_slice())
        .unwrap_or(&[]);
    refs.iter()
        .filter_map(|value| value.strip_prefix("repo-sha:"))
        .any(|recorded_sha| recorded_sha == source_sha)
}

fn entry_records_permissive_license(entry: &ExternalCorpusEnvelopeV1Json) -> bool {
    let generation_refs = entry.generation.oracle_pin_refs.as_deref().unwrap_or(&[]);
    let provenance_refs = entry
        .provenance
        .as_ref()
        .map(|provenance| provenance.oracle_pin_refs.as_slice())
        .unwrap_or(&[]);
    generation_refs
        .iter()
        .chain(provenance_refs.iter())
        .any(|value| value == "spdx:MIT")
}

fn is_bounded_sparse_path(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != "/"
        && !value.starts_with('/')
        && !value.split('/').any(|part| part == "..")
}

fn entry_chunk_hashes_match(entry: &ExternalCorpusEnvelopeV1Json) -> bool {
    !entry.chunks.is_empty()
        && entry.chunks.iter().all(|chunk| {
            is_sha256(&chunk.sha256)
                && read_chunk_source(chunk.path.as_str())
                    .map(|source| sha256_hex(source.as_bytes()) == chunk.sha256)
                    .unwrap_or(false)
        })
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn read_chunk_source(relative_path: &str) -> Option<String> {
    if !is_bounded_sparse_path(relative_path) {
        return None;
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("oss-corpus-farm");
    fs::read_to_string(root.join(relative_path)).ok()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn dialect_label(dialect: &ExternalCorpusDialectV1Json) -> &'static str {
    match dialect {
        ExternalCorpusDialectV1Json::Css => "css",
        ExternalCorpusDialectV1Json::Scss => "scss",
        ExternalCorpusDialectV1Json::Sass => "sass",
        ExternalCorpusDialectV1Json::Less => "less",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oss_corpus_farm_manifest_entries_use_generated_envelope_shape() {
        let report = summarize_oss_corpus_farm_manifest_v0();
        assert!(report.entry_count >= 3);
        assert!(report.repository_count >= 1);
        assert!(report.dialect_count >= 3);
        assert!(report.dialects.contains(&"css"));
        assert!(report.dialects.contains(&"scss"));
        assert!(report.dialects.contains(&"less"));
        assert!(report.all_entries_follow_generated_envelope_shape);
        assert!(report.all_entries_stage1_advisory);
        assert!(report.all_entries_out_of_scope);
        assert!(report.all_entries_have_permissive_spdx);
        assert!(report.all_entry_pins_are_sha_locked);
        assert!(report.all_recorded_shas_match_source_pins);
        assert!(report.all_sparse_paths_are_bounded);
        assert!(report.all_chunk_hashes_match);
    }
}

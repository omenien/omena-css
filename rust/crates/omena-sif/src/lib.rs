//! Sass Interface File v1 contracts for Omena CSS.
//!
//! SIF v1 is the local, deterministic artifact that lets resolver and
//! diagnostic layers stop treating external Sass modules as blind
//! `externalIgnored` gaps. This crate is deliberately pure data plus hashing:
//! it performs no Sass evaluation, package execution, filesystem traversal, or
//! network access.

use serde::{Deserialize, Serialize};
use serde_json::Value;

mod generator;

pub use generator::*;

pub const OMENA_SIF_VERSION_V1: &str = "1";
pub const OMENA_SIF_HASH_ALGORITHM_V1: &str = "blake3";
pub const OMENA_SIF_V1_SCHEMA_JSON: &str = include_str!("../schema/sif-v1.schema.json");
pub const OMENA_LOCK_V1_SCHEMA_JSON: &str = include_str!("../schema/lock-v1.schema.json");

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OmenaSifDigestV1(String);

impl OmenaSifDigestV1 {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn from_blake3_bytes(bytes: &[u8]) -> Self {
        let hash = blake3::hash(bytes);
        Self(format!("{OMENA_SIF_HASH_ALGORITHM_V1}:{}", hash.to_hex()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifV1 {
    pub sif_version: String,
    pub canonical_url: String,
    pub generator: OmenaSifGeneratorV1,
    pub source: OmenaSifSourceV1,
    pub exports: OmenaSifExportsV1,
    pub dependencies: Vec<OmenaSifDependencyInterfaceHashV1>,
    pub fingerprints: OmenaSifFingerprintChainV1,
}

impl OmenaSifV1 {
    pub fn from_static_exports(
        canonical_url: impl Into<String>,
        generator: OmenaSifGeneratorV1,
        source: OmenaSifSourceV1,
        exports: OmenaSifExportsV1,
        dependencies: Vec<OmenaSifDependencyInterfaceHashV1>,
        source_bytes: &[u8],
    ) -> Result<Self, serde_json::Error> {
        let fingerprints = compute_omena_sif_fingerprint_chain_v1(
            source_bytes,
            generator.toolchain_id.as_str(),
            &exports,
            &dependencies,
        )?;
        Ok(Self {
            sif_version: OMENA_SIF_VERSION_V1.to_string(),
            canonical_url: canonical_url.into(),
            generator,
            source,
            exports,
            dependencies: sorted_omena_sif_dependencies_v1(dependencies),
            fingerprints,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifGeneratorV1 {
    pub name: String,
    pub version: String,
    pub toolchain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifSourceV1 {
    pub syntax: OmenaSifSourceSyntaxV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OmenaSifSourceSyntaxV1 {
    Css,
    Scss,
    Sass,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifExportsV1 {
    pub variables: Vec<OmenaSifVariableExportV1>,
    pub mixins: Vec<OmenaSifCallableExportV1>,
    pub functions: Vec<OmenaSifCallableExportV1>,
    pub placeholders: Vec<OmenaSifPlaceholderExportV1>,
    pub forwards: Vec<OmenaSifForwardExportV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifVariableExportV1 {
    pub name: String,
    pub defaulted: bool,
    pub value_repr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifCallableExportV1 {
    pub name: String,
    pub parameters: Vec<OmenaSifParameterV1>,
    pub accepts_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifParameterV1 {
    pub name: String,
    pub default_value_repr: Option<String>,
    pub variadic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifPlaceholderExportV1 {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifForwardExportV1 {
    pub canonical_url: String,
    pub prefix: Option<String>,
    pub show: Vec<String>,
    pub hide: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifDependencyInterfaceHashV1 {
    pub canonical_url: String,
    pub interface_hash: OmenaSifDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifFingerprintChainV1 {
    pub hash_algorithm: String,
    pub leaf_hash: OmenaSifDigestV1,
    pub interface_hash: OmenaSifDigestV1,
    pub transitive_hash: OmenaSifDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLockV1 {
    pub lockfile_version: String,
    pub entries: Vec<OmenaLockSifEntryV1>,
}

impl OmenaLockV1 {
    pub fn new(entries: Vec<OmenaLockSifEntryV1>) -> Self {
        Self {
            lockfile_version: OMENA_SIF_VERSION_V1.to_string(),
            entries: sorted_omena_lock_entries_v1(entries),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLockSifEntryV1 {
    pub canonical_url: String,
    pub sif_path: String,
    pub sif_hash: OmenaSifDigestV1,
    pub interface_hash: OmenaSifDigestV1,
    pub transitive_hash: OmenaSifDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLockVerificationReportV1 {
    pub lockfile_version: String,
    pub frozen: bool,
    pub verified: bool,
    pub entries_checked: usize,
    pub issues: Vec<OmenaLockVerificationIssueV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLockVerificationIssueV1 {
    pub canonical_url: String,
    pub sif_path: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSifInterfaceHashInputV1<'a> {
    sif_version: &'a str,
    toolchain_id: &'a str,
    exports: &'a OmenaSifExportsV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSifTransitiveHashInputV1<'a> {
    sif_version: &'a str,
    interface_hash: &'a OmenaSifDigestV1,
    dependencies: Vec<OmenaSifDependencyInterfaceHashV1>,
}

pub fn compute_omena_sif_leaf_hash_v1(source_bytes: &[u8]) -> OmenaSifDigestV1 {
    OmenaSifDigestV1::from_blake3_bytes(source_bytes)
}

pub fn compute_omena_sif_interface_hash_v1(
    toolchain_id: &str,
    exports: &OmenaSifExportsV1,
) -> Result<OmenaSifDigestV1, serde_json::Error> {
    let input = OmenaSifInterfaceHashInputV1 {
        sif_version: OMENA_SIF_VERSION_V1,
        toolchain_id,
        exports,
    };
    let canonical_bytes = write_omena_canonical_json_bytes_v1(&input)?;
    Ok(OmenaSifDigestV1::from_blake3_bytes(&canonical_bytes))
}

pub fn compute_omena_sif_transitive_hash_v1(
    interface_hash: &OmenaSifDigestV1,
    dependencies: &[OmenaSifDependencyInterfaceHashV1],
) -> Result<OmenaSifDigestV1, serde_json::Error> {
    let input = OmenaSifTransitiveHashInputV1 {
        sif_version: OMENA_SIF_VERSION_V1,
        interface_hash,
        dependencies: sorted_omena_sif_dependencies_v1(dependencies.to_vec()),
    };
    let canonical_bytes = write_omena_canonical_json_bytes_v1(&input)?;
    Ok(OmenaSifDigestV1::from_blake3_bytes(&canonical_bytes))
}

pub fn compute_omena_sif_fingerprint_chain_v1(
    source_bytes: &[u8],
    toolchain_id: &str,
    exports: &OmenaSifExportsV1,
    dependencies: &[OmenaSifDependencyInterfaceHashV1],
) -> Result<OmenaSifFingerprintChainV1, serde_json::Error> {
    let leaf_hash = compute_omena_sif_leaf_hash_v1(source_bytes);
    let interface_hash = compute_omena_sif_interface_hash_v1(toolchain_id, exports)?;
    let transitive_hash = compute_omena_sif_transitive_hash_v1(&interface_hash, dependencies)?;

    Ok(OmenaSifFingerprintChainV1 {
        hash_algorithm: OMENA_SIF_HASH_ALGORITHM_V1.to_string(),
        leaf_hash,
        interface_hash,
        transitive_hash,
    })
}

pub fn compute_omena_sif_artifact_hash_v1(
    sif: &OmenaSifV1,
) -> Result<OmenaSifDigestV1, serde_json::Error> {
    let canonical_bytes = write_omena_sif_json_v1(sif)?.into_bytes();
    Ok(OmenaSifDigestV1::from_blake3_bytes(&canonical_bytes))
}

pub fn build_omena_lock_sif_entry_v1(
    sif_path: impl Into<String>,
    sif: &OmenaSifV1,
) -> Result<OmenaLockSifEntryV1, serde_json::Error> {
    Ok(OmenaLockSifEntryV1 {
        canonical_url: sif.canonical_url.clone(),
        sif_path: sif_path.into(),
        sif_hash: compute_omena_sif_artifact_hash_v1(sif)?,
        interface_hash: sif.fingerprints.interface_hash.clone(),
        transitive_hash: sif.fingerprints.transitive_hash.clone(),
    })
}

pub fn read_omena_lock_json_v1(source: &str) -> Result<OmenaLockV1, serde_json::Error> {
    serde_json::from_str(source)
}

pub fn write_omena_lock_json_v1(lock: &OmenaLockV1) -> Result<String, serde_json::Error> {
    write_omena_canonical_json_string_v1(&OmenaLockV1::new(lock.entries.clone()))
}

pub fn verify_omena_lock_frozen_v1<F>(
    lock: &OmenaLockV1,
    mut load_sif_json: F,
) -> OmenaLockVerificationReportV1
where
    F: FnMut(&OmenaLockSifEntryV1) -> Result<String, String>,
{
    let mut issues = Vec::new();

    for entry in &lock.entries {
        let sif_json = match load_sif_json(entry) {
            Ok(sif_json) => sif_json,
            Err(error) => {
                push_omena_lock_issue_v1(&mut issues, entry, "loadFailed", error);
                continue;
            }
        };
        let sif = match read_omena_sif_json_v1(&sif_json) {
            Ok(sif) => sif,
            Err(error) => {
                push_omena_lock_issue_v1(
                    &mut issues,
                    entry,
                    "parseFailed",
                    format!("failed to parse SIF JSON: {error}"),
                );
                continue;
            }
        };
        let actual_sif_hash = match compute_omena_sif_artifact_hash_v1(&sif) {
            Ok(hash) => hash,
            Err(error) => {
                push_omena_lock_issue_v1(
                    &mut issues,
                    entry,
                    "hashFailed",
                    format!("failed to hash canonical SIF JSON: {error}"),
                );
                continue;
            }
        };

        if sif.canonical_url != entry.canonical_url {
            push_omena_lock_issue_v1(
                &mut issues,
                entry,
                "canonicalUrlMismatch",
                format!(
                    "lock expected {}, SIF declared {}",
                    entry.canonical_url, sif.canonical_url
                ),
            );
        }
        if actual_sif_hash != entry.sif_hash {
            push_omena_lock_issue_v1(
                &mut issues,
                entry,
                "sifHashMismatch",
                format!(
                    "lock expected {}, SIF canonical artifact hash is {}",
                    entry.sif_hash.as_str(),
                    actual_sif_hash.as_str()
                ),
            );
        }
        if sif.fingerprints.interface_hash != entry.interface_hash {
            push_omena_lock_issue_v1(
                &mut issues,
                entry,
                "interfaceHashMismatch",
                format!(
                    "lock expected {}, SIF interface hash is {}",
                    entry.interface_hash.as_str(),
                    sif.fingerprints.interface_hash.as_str()
                ),
            );
        }
        if sif.fingerprints.transitive_hash != entry.transitive_hash {
            push_omena_lock_issue_v1(
                &mut issues,
                entry,
                "transitiveHashMismatch",
                format!(
                    "lock expected {}, SIF transitive hash is {}",
                    entry.transitive_hash.as_str(),
                    sif.fingerprints.transitive_hash.as_str()
                ),
            );
        }
    }

    OmenaLockVerificationReportV1 {
        lockfile_version: lock.lockfile_version.clone(),
        frozen: true,
        verified: issues.is_empty(),
        entries_checked: lock.entries.len(),
        issues,
    }
}

pub fn read_omena_sif_json_v1(source: &str) -> Result<OmenaSifV1, serde_json::Error> {
    serde_json::from_str(source)
}

pub fn write_omena_sif_json_v1(sif: &OmenaSifV1) -> Result<String, serde_json::Error> {
    write_omena_canonical_json_string_v1(sif)
}

pub fn write_omena_canonical_json_bytes_v1<T: Serialize>(
    value: &T,
) -> Result<Vec<u8>, serde_json::Error> {
    Ok(write_omena_canonical_json_string_v1(value)?.into_bytes())
}

pub fn write_omena_canonical_json_string_v1<T: Serialize>(
    value: &T,
) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    let mut output = String::new();
    write_canonical_json_value_v1(&value, &mut output)?;
    Ok(output)
}

fn sorted_omena_sif_dependencies_v1(
    mut dependencies: Vec<OmenaSifDependencyInterfaceHashV1>,
) -> Vec<OmenaSifDependencyInterfaceHashV1> {
    dependencies.sort_by(|left, right| {
        left.canonical_url
            .cmp(&right.canonical_url)
            .then(left.interface_hash.cmp(&right.interface_hash))
    });
    dependencies
}

fn sorted_omena_lock_entries_v1(mut entries: Vec<OmenaLockSifEntryV1>) -> Vec<OmenaLockSifEntryV1> {
    entries.sort_by(|left, right| {
        left.canonical_url
            .cmp(&right.canonical_url)
            .then(left.sif_path.cmp(&right.sif_path))
    });
    entries
}

fn push_omena_lock_issue_v1(
    issues: &mut Vec<OmenaLockVerificationIssueV1>,
    entry: &OmenaLockSifEntryV1,
    code: &str,
    message: String,
) {
    issues.push(OmenaLockVerificationIssueV1 {
        canonical_url: entry.canonical_url.clone(),
        sif_path: entry.sif_path.clone(),
        code: code.to_string(),
        message,
    });
}

fn write_canonical_json_value_v1(
    value: &Value,
    output: &mut String,
) -> Result<(), serde_json::Error> {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => output.push_str(&serde_json::to_string(value)?),
        Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json_value_v1(value, output)?;
            }
            output.push(']');
        }
        Value::Object(map) => {
            output.push('{');
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|(left_key, _), (right_key, _)| left_key.cmp(right_key));
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key)?);
                output.push(':');
                write_canonical_json_value_v1(value, output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn schema_file_is_valid_json() -> Result<(), serde_json::Error> {
        let schema: Value = serde_json::from_str(OMENA_SIF_V1_SCHEMA_JSON)?;
        assert_eq!(
            schema.get("title").and_then(Value::as_str),
            Some("Omena Sass Interface File v1")
        );
        Ok(())
    }

    #[test]
    fn lock_schema_file_is_valid_json() -> Result<(), serde_json::Error> {
        let schema: Value = serde_json::from_str(OMENA_LOCK_V1_SCHEMA_JSON)?;
        assert_eq!(
            schema.get("title").and_then(Value::as_str),
            Some("Omena Lockfile v1")
        );
        Ok(())
    }

    #[test]
    fn canonical_json_sorts_object_keys_recursively() -> Result<(), serde_json::Error> {
        let value = json!({
            "z": 1,
            "a": {
                "d": true,
                "b": ["x", { "y": null, "c": 2 }]
            }
        });

        assert_eq!(
            write_omena_canonical_json_string_v1(&value)?,
            r#"{"a":{"b":["x",{"c":2,"y":null}],"d":true},"z":1}"#
        );
        Ok(())
    }

    #[test]
    fn leaf_hash_tracks_source_bytes_with_algorithm_tag() {
        let first = compute_omena_sif_leaf_hash_v1(b"$color: red;");
        let second = compute_omena_sif_leaf_hash_v1(b"$color: blue;");

        assert_ne!(first, second);
        assert!(first.as_str().starts_with("blake3:"));
        assert_eq!(first.as_str().len(), "blake3:".len() + 64);
    }

    #[test]
    fn interface_hash_ignores_source_bytes_and_urls() -> Result<(), serde_json::Error> {
        let exports = fixture_exports();
        let first = compute_omena_sif_interface_hash_v1("omena-sifgen@0.1", &exports)?;
        let second = OmenaSifV1::from_static_exports(
            "pkg:a/_tokens.scss",
            fixture_generator(),
            fixture_source(),
            exports.clone(),
            Vec::new(),
            b"$color: red;",
        )?
        .fingerprints
        .interface_hash;
        let third = OmenaSifV1::from_static_exports(
            "pkg:b/_tokens.scss",
            fixture_generator(),
            fixture_source(),
            exports,
            Vec::new(),
            b"$color: blue;",
        )?
        .fingerprints
        .interface_hash;

        assert_eq!(first, second);
        assert_eq!(second, third);
        Ok(())
    }

    #[test]
    fn transitive_hash_sorts_dependencies_before_hashing() -> Result<(), serde_json::Error> {
        let interface_hash = OmenaSifDigestV1::from_blake3_bytes(b"self-interface");
        let first = vec![
            OmenaSifDependencyInterfaceHashV1 {
                canonical_url: "pkg:z/_index.scss".to_string(),
                interface_hash: OmenaSifDigestV1::from_blake3_bytes(b"z"),
            },
            OmenaSifDependencyInterfaceHashV1 {
                canonical_url: "pkg:a/_index.scss".to_string(),
                interface_hash: OmenaSifDigestV1::from_blake3_bytes(b"a"),
            },
        ];
        let second = vec![first[1].clone(), first[0].clone()];

        assert_eq!(
            compute_omena_sif_transitive_hash_v1(&interface_hash, &first)?,
            compute_omena_sif_transitive_hash_v1(&interface_hash, &second)?
        );
        Ok(())
    }

    #[test]
    fn sif_json_round_trips_through_canonical_writer() -> Result<(), serde_json::Error> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;

        let json = write_omena_sif_json_v1(&sif)?;
        let decoded = read_omena_sif_json_v1(&json)?;

        assert_eq!(decoded, sif);
        assert!(json.starts_with(r#"{"canonicalUrl":"pkg:design-system/_tokens.scss","#));
        Ok(())
    }

    #[test]
    fn sif_artifact_hash_tracks_canonical_sif_json() -> Result<(), serde_json::Error> {
        let first = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let mut second = first.clone();
        second.source.syntax = OmenaSifSourceSyntaxV1::Css;

        assert_ne!(
            compute_omena_sif_artifact_hash_v1(&first)?,
            compute_omena_sif_artifact_hash_v1(&second)?
        );
        Ok(())
    }

    #[test]
    fn lock_json_writer_sorts_entries_deterministically() -> Result<(), serde_json::Error> {
        let first = fixture_sif("pkg:z/_tokens.scss", b"$z: red;")?;
        let second = fixture_sif("pkg:a/_tokens.scss", b"$a: red;")?;
        let lock = OmenaLockV1 {
            lockfile_version: OMENA_SIF_VERSION_V1.to_string(),
            entries: vec![
                build_omena_lock_sif_entry_v1("sif/z.sif.json", &first)?,
                build_omena_lock_sif_entry_v1("sif/a.sif.json", &second)?,
            ],
        };

        let json = write_omena_lock_json_v1(&lock)?;
        assert!(json.contains(r#""lockfileVersion":"1""#));
        assert!(
            json.find("pkg:a/_tokens.scss") < json.find("pkg:z/_tokens.scss"),
            "{json}"
        );
        Ok(())
    }

    #[test]
    fn lock_frozen_verification_passes_for_matching_sifs() -> Result<(), serde_json::Error> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let sif_json = write_omena_sif_json_v1(&sif)?;
        let lock = OmenaLockV1::new(vec![build_omena_lock_sif_entry_v1(
            "sif/design-system.sif.json",
            &sif,
        )?]);
        let mut files = BTreeMap::new();
        files.insert("sif/design-system.sif.json".to_string(), sif_json);

        let report = verify_omena_lock_frozen_v1(&lock, |entry| {
            files
                .get(&entry.sif_path)
                .cloned()
                .ok_or_else(|| format!("missing {}", entry.sif_path))
        });

        assert!(report.verified, "{report:?}");
        assert_eq!(report.entries_checked, 1);
        assert!(report.issues.is_empty());
        Ok(())
    }

    #[test]
    fn lock_frozen_verification_fails_for_changed_sif() -> Result<(), serde_json::Error> {
        let locked_sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let changed_sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: blue !default;")?;
        let changed_json = write_omena_sif_json_v1(&changed_sif)?;
        let lock = OmenaLockV1::new(vec![build_omena_lock_sif_entry_v1(
            "sif/design-system.sif.json",
            &locked_sif,
        )?]);

        let report = verify_omena_lock_frozen_v1(&lock, |_entry| Ok(changed_json.clone()));

        assert!(!report.verified, "{report:?}");
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "sifHashMismatch"),
            "{report:?}"
        );
        Ok(())
    }

    fn fixture_generator() -> OmenaSifGeneratorV1 {
        OmenaSifGeneratorV1 {
            name: "omena-sifgen".to_string(),
            version: "0.1.0".to_string(),
            toolchain_id: "omena-sifgen@0.1".to_string(),
        }
    }

    fn fixture_source() -> OmenaSifSourceV1 {
        OmenaSifSourceV1 {
            syntax: OmenaSifSourceSyntaxV1::Scss,
        }
    }

    fn fixture_sif(
        canonical_url: &str,
        source_bytes: &[u8],
    ) -> Result<OmenaSifV1, serde_json::Error> {
        OmenaSifV1::from_static_exports(
            canonical_url,
            fixture_generator(),
            fixture_source(),
            fixture_exports(),
            Vec::new(),
            source_bytes,
        )
    }

    fn fixture_exports() -> OmenaSifExportsV1 {
        OmenaSifExportsV1 {
            variables: vec![OmenaSifVariableExportV1 {
                name: "$color".to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            }],
            mixins: vec![OmenaSifCallableExportV1 {
                name: "button".to_string(),
                parameters: vec![OmenaSifParameterV1 {
                    name: "$variant".to_string(),
                    default_value_repr: Some("primary".to_string()),
                    variadic: false,
                }],
                accepts_content: true,
            }],
            functions: Vec::new(),
            placeholders: vec![OmenaSifPlaceholderExportV1 {
                name: "%surface".to_string(),
            }],
            forwards: vec![OmenaSifForwardExportV1 {
                canonical_url: "pkg:design-system/_colors.scss".to_string(),
                prefix: None,
                show: vec!["$color".to_string()],
                hide: Vec::new(),
            }],
        }
    }
}

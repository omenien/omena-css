use crate::OmenaSifAttestationReferenceV1;
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaSifNpmProvenanceMetadataErrorV1 {
    MalformedJson,
    MalformedMetadata,
    MissingPackageName,
    PackageSubjectMismatch,
    MissingPackageVersion,
    VersionSubjectMismatch,
    InvalidProvenanceShape,
    ProvenanceSubjectMismatch,
}

impl OmenaSifNpmProvenanceMetadataErrorV1 {
    pub fn code(self) -> &'static str {
        match self {
            Self::MalformedJson => "malformedJson",
            Self::MalformedMetadata => "malformedMetadata",
            Self::MissingPackageName => "missingPackageName",
            Self::PackageSubjectMismatch => "packageSubjectMismatch",
            Self::MissingPackageVersion => "missingPackageVersion",
            Self::VersionSubjectMismatch => "versionSubjectMismatch",
            Self::InvalidProvenanceShape => "invalidProvenanceShape",
            Self::ProvenanceSubjectMismatch => "provenanceSubjectMismatch",
        }
    }
}

impl std::fmt::Display for OmenaSifNpmProvenanceMetadataErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for OmenaSifNpmProvenanceMetadataErrorV1 {}

pub fn ingest_omena_sif_npm_provenance_metadata_v1(
    source: &str,
    expected_package_selector: &str,
) -> Result<Vec<OmenaSifAttestationReferenceV1>, OmenaSifNpmProvenanceMetadataErrorV1> {
    let metadata = serde_json::from_str::<Value>(source)
        .map_err(|_| OmenaSifNpmProvenanceMetadataErrorV1::MalformedJson)?;
    let metadata = metadata
        .as_object()
        .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::MalformedMetadata)?;
    let package_name = required_nonempty_string(metadata.get("name"))
        .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::MissingPackageName)?;
    let expected_package_name = npm_package_name_from_selector(expected_package_selector)
        .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::PackageSubjectMismatch)?;
    if package_name != expected_package_name {
        return Err(OmenaSifNpmProvenanceMetadataErrorV1::PackageSubjectMismatch);
    }

    let mut references = BTreeSet::new();
    if metadata_contains_provenance(&Value::Object(metadata.clone())) {
        let version = required_nonempty_string(metadata.get("version"))
            .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::MissingPackageVersion)?;
        collect_version_provenance(metadata, package_name, version, &mut references)?;
    }

    if let Some(versions) = metadata.get("versions") {
        let versions = versions
            .as_object()
            .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::MalformedMetadata)?;
        for (expected_version, version_metadata) in versions {
            let version_metadata = version_metadata
                .as_object()
                .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::MalformedMetadata)?;
            let version_package_name = required_nonempty_string(version_metadata.get("name"))
                .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::MissingPackageName)?;
            if version_package_name != package_name {
                return Err(OmenaSifNpmProvenanceMetadataErrorV1::PackageSubjectMismatch);
            }
            let version = required_nonempty_string(version_metadata.get("version"))
                .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::MissingPackageVersion)?;
            if version != expected_version {
                return Err(OmenaSifNpmProvenanceMetadataErrorV1::VersionSubjectMismatch);
            }
            collect_version_provenance(version_metadata, package_name, version, &mut references)?;
        }
    }

    Ok(references.into_iter().collect())
}

fn required_nonempty_string(value: Option<&Value>) -> Option<&str> {
    value?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn npm_package_name_from_selector(selector: &str) -> Option<&str> {
    let selector = selector.strip_prefix("pkg:").unwrap_or(selector).trim();
    if selector.starts_with('@') {
        let package_end = selector
            .char_indices()
            .filter(|(_, character)| *character == '/')
            .nth(1)
            .map(|(index, _)| index)
            .unwrap_or(selector.len());
        let package_name = &selector[..package_end];
        return package_name.contains('/').then_some(package_name);
    }
    selector.split('/').next().filter(|value| !value.is_empty())
}

fn metadata_contains_provenance(metadata: &Value) -> bool {
    metadata.pointer("/dist/attestations/provenance").is_some()
        || metadata.pointer("/attestations/provenance").is_some()
}

fn collect_version_provenance(
    metadata: &serde_json::Map<String, Value>,
    package_name: &str,
    version: &str,
    references: &mut BTreeSet<OmenaSifAttestationReferenceV1>,
) -> Result<(), OmenaSifNpmProvenanceMetadataErrorV1> {
    let metadata = Value::Object(metadata.clone());
    for pointer in ["/dist/attestations/provenance", "/attestations/provenance"] {
        if let Some(provenance) = metadata.pointer(pointer) {
            collect_provenance_value(provenance, package_name, version, references)?;
        }
    }
    Ok(())
}

fn collect_provenance_value(
    value: &Value,
    package_name: &str,
    version: &str,
    references: &mut BTreeSet<OmenaSifAttestationReferenceV1>,
) -> Result<(), OmenaSifNpmProvenanceMetadataErrorV1> {
    match value {
        Value::String(reference) => insert_reference(
            "npm-provenance",
            reference,
            package_name,
            version,
            references,
        ),
        Value::Array(values) if !values.is_empty() => {
            for value in values {
                collect_provenance_value(value, package_name, version, references)?;
            }
            Ok(())
        }
        Value::Object(object) if !object.is_empty() => {
            let mut recognized = false;
            for key in ["url", "uri", "reference", "bundle", "provenance"] {
                if let Some(reference) = object.get(key) {
                    recognized = true;
                    let reference = reference
                        .as_str()
                        .ok_or(OmenaSifNpmProvenanceMetadataErrorV1::InvalidProvenanceShape)?;
                    insert_reference(
                        format!("npm-provenance.{key}").as_str(),
                        reference,
                        package_name,
                        version,
                        references,
                    )?;
                }
            }
            if !recognized {
                return Err(OmenaSifNpmProvenanceMetadataErrorV1::InvalidProvenanceShape);
            }
            Ok(())
        }
        _ => Err(OmenaSifNpmProvenanceMetadataErrorV1::InvalidProvenanceShape),
    }
}

fn insert_reference(
    kind: &str,
    reference: &str,
    package_name: &str,
    version: &str,
    references: &mut BTreeSet<OmenaSifAttestationReferenceV1>,
) -> Result<(), OmenaSifNpmProvenanceMetadataErrorV1> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Err(OmenaSifNpmProvenanceMetadataErrorV1::InvalidProvenanceShape);
    }
    if is_remote_provenance_reference(reference)
        && !npm_attestation_reference_binds_subject(reference, package_name, version)
    {
        return Err(OmenaSifNpmProvenanceMetadataErrorV1::ProvenanceSubjectMismatch);
    }
    references.insert(OmenaSifAttestationReferenceV1 {
        kind: kind.to_string(),
        reference: reference.to_string(),
    });
    Ok(())
}

fn is_remote_provenance_reference(reference: &str) -> bool {
    let lowercase = reference.to_ascii_lowercase();
    lowercase.starts_with("https://") || lowercase.starts_with("http://")
}

fn npm_attestation_reference_binds_subject(
    reference: &str,
    package_name: &str,
    version: &str,
) -> bool {
    let reference = reference.to_ascii_lowercase();
    let Some(subject_segment) = reference
        .split_once("/attestations/")
        .map(|(_, suffix)| suffix)
        .and_then(|suffix| suffix.split(['/', '?', '#']).next())
    else {
        return false;
    };
    let package_name = package_name.to_ascii_lowercase();
    let version = version.to_ascii_lowercase();
    let raw_subject = format!("{package_name}@{version}");
    let path_encoded_name = package_name.replace('/', "%2f");
    let path_encoded_subject = format!("{path_encoded_name}@{version}");
    let fully_encoded_name = path_encoded_name.replace('@', "%40");
    let fully_encoded_subject = format!("{fully_encoded_name}%40{version}");
    subject_segment == raw_subject
        || subject_segment == path_encoded_subject
        || subject_segment == fully_encoded_subject
}

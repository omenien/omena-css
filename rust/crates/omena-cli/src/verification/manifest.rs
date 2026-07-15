use serde::{Deserialize, Serialize};

const VERIFICATION_MANIFEST_SOURCE: &str = include_str!("../../verification-targets.json");
const CONFIG_SCHEMA_MANIFEST_SOURCE: &str = include_str!("../../config-schema-census.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum VerificationScopeV0 {
    UserWorkspace,
    EngineSelf,
}

impl VerificationScopeV0 {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::UserWorkspace => "userWorkspace",
            Self::EngineSelf => "engineSelf",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(super) enum VerificationAvailabilityV0 {
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "not-yet")]
    NotYet,
    #[serde(rename = "skipped")]
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum VerificationExecutorV0 {
    ParserFacts,
    ModuleGraphDiagnostics,
    FormatIdempotence,
    BundleAdmission,
    ModulesDrift,
    SassExternalComparison,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VerificationEvidenceReferenceV0 {
    pub(crate) path: String,
    pub(crate) symbol: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VerificationTargetV0 {
    pub(super) id: String,
    pub(super) scope: VerificationScopeV0,
    pub(super) availability: VerificationAvailabilityV0,
    pub(super) executor: Option<VerificationExecutorV0>,
    pub(super) description: String,
    pub(super) evidence: Vec<VerificationEvidenceReferenceV0>,
    pub(super) limitation: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CiAdapterV0 {
    pub(crate) verb: String,
    pub(crate) executor: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VerificationManifestV0 {
    schema_version: String,
    product: String,
    pub(super) targets: Vec<VerificationTargetV0>,
    pub(crate) ci_adapters: Vec<CiAdapterV0>,
}

pub(super) fn verification_manifest() -> Result<VerificationManifestV0, String> {
    let manifest: VerificationManifestV0 = serde_json::from_str(VERIFICATION_MANIFEST_SOURCE)
        .map_err(|error| format!("failed to decode the verification target manifest: {error}"))?;
    if manifest.schema_version != "0" || manifest.product != "omena-cli.verification-targets" {
        return Err("unsupported verification target manifest contract".to_string());
    }
    Ok(manifest)
}

pub(crate) fn ci_adapters() -> Result<Vec<CiAdapterV0>, String> {
    Ok(verification_manifest()?.ci_adapters)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConfigSchemaManifestV0 {
    schema_version: String,
    product: String,
    translation_validation: Vec<TranslationValidationBindingV0>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TranslationValidationBindingV0 {
    pub(super) value: String,
    pub(super) report_kind: Option<String>,
    pub(super) engine_arm: Option<String>,
}

pub(super) fn translation_validation_binding(
    value: &str,
) -> Result<TranslationValidationBindingV0, String> {
    let manifest: ConfigSchemaManifestV0 = serde_json::from_str(CONFIG_SCHEMA_MANIFEST_SOURCE)
        .map_err(|error| format!("failed to decode the config vocabulary manifest: {error}"))?;
    if manifest.schema_version != "0" || manifest.product != "omena-cli.config-schema-census" {
        return Err("unsupported config vocabulary manifest contract".to_string());
    }
    manifest
        .translation_validation
        .into_iter()
        .find(|binding| binding.value == value)
        .ok_or_else(|| format!("translation validation vocabulary has no binding for {value}"))
}

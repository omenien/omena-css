use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaConfig {
    #[serde(deserialize_with = "deserialize_path_list")]
    pub(crate) extends: Vec<PathBuf>,
    pub(crate) workspace: OmenaWorkspaceConfig,
    pub(crate) style: OmenaStyleConfig,
    pub(crate) lint: OmenaLintConfig,
    pub(crate) format: OmenaFormatConfig,
    pub(crate) minify: OmenaMinifyConfig,
    pub(crate) modules: OmenaModulesConfig,
    pub(crate) sass: OmenaSassConfig,
    pub(crate) intelligence: OmenaIntelligenceConfig,
    pub(crate) verify: OmenaVerifyConfig,
    pub(crate) ci: OmenaCiConfig,
    pub(crate) build: OmenaBuildConfig,
    pub(crate) overrides: Vec<OmenaConfigOverride>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaWorkspaceConfig {
    pub(crate) roots: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaStyleConfig {
    pub(crate) languages: Vec<String>,
    pub(crate) source_languages: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaLintConfig {
    pub(crate) profile: Option<String>,
    pub(crate) stylelint_compat: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaFormatConfig {
    pub(crate) mode: Option<String>,
    pub(crate) line_width: Option<u16>,
    pub(crate) indent_width: Option<u8>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaMinifyConfig {
    pub(crate) profile: Option<String>,
    pub(crate) target: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaModulesConfig {
    pub(crate) typed_definitions: Option<bool>,
    pub(crate) hash_strategy: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaSassConfig {
    pub(crate) oracle: Option<String>,
    pub(crate) sif: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaIntelligenceConfig {
    pub(crate) tailwind: OmenaTailwindConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaTailwindConfig {
    pub(crate) enabled: Option<bool>,
    pub(crate) class_functions: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OmenaTranslationValidationMode {
    #[default]
    Off,
    Staged,
}

impl OmenaTranslationValidationMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Staged => "staged",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaVerifyConfig {
    pub(crate) evidence: Option<String>,
    pub(crate) translation_validation: OmenaTranslationValidationMode,
    pub(crate) external_corpus: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaCiConfig {
    pub(crate) precision_regression: Option<String>,
    pub(crate) transform_rejection: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaConfigOverride {
    #[serde(alias = "pattern", deserialize_with = "deserialize_string_list")]
    pub(crate) patterns: Vec<String>,
    pub(crate) workspace: Option<OmenaWorkspaceConfig>,
    pub(crate) style: Option<OmenaStyleConfig>,
    pub(crate) lint: Option<OmenaLintConfig>,
    pub(crate) format: Option<OmenaFormatConfig>,
    pub(crate) minify: Option<OmenaMinifyConfig>,
    pub(crate) modules: Option<OmenaModulesConfig>,
    pub(crate) sass: Option<OmenaSassConfig>,
    pub(crate) intelligence: Option<OmenaIntelligenceConfig>,
    pub(crate) verify: Option<OmenaVerifyConfig>,
    pub(crate) ci: Option<OmenaCiConfig>,
    pub(crate) build: Option<OmenaBuildConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaBuildConfig {
    #[serde(alias = "pass")]
    pub(crate) passes: Option<Vec<String>>,
    pub(crate) minify: Option<bool>,
    #[serde(alias = "target_query", alias = "target-query")]
    pub(crate) target_query: Option<String>,
    #[serde(alias = "closed_style_world", alias = "closed-style-world")]
    pub(crate) closed_style_world: Option<bool>,
    #[serde(alias = "tree_shake", alias = "tree-shake")]
    pub(crate) tree_shake: Option<bool>,
    pub(crate) bundle: Option<bool>,
    #[serde(alias = "source_map", alias = "source-map")]
    pub(crate) source_map: Option<bool>,
    pub(crate) output: Option<PathBuf>,
    #[serde(alias = "source", alias = "source_paths", alias = "source-paths")]
    pub(crate) sources: Option<Vec<PathBuf>>,
    #[serde(
        alias = "package_manifest",
        alias = "package_manifests",
        alias = "package-manifest",
        alias = "package-manifests"
    )]
    pub(crate) package_manifests: Option<Vec<PathBuf>>,
    #[serde(
        alias = "bundle_entry",
        alias = "bundle_entries",
        alias = "bundle-entry",
        alias = "bundle-entries"
    )]
    pub(crate) bundle_entries: Option<Vec<PathBuf>>,
    #[serde(alias = "split_out_dir", alias = "split-out-dir")]
    pub(crate) split_out_dir: Option<PathBuf>,
    #[serde(alias = "context_json", alias = "context-json")]
    pub(crate) context_json: Option<PathBuf>,
    #[serde(alias = "engine_input_json", alias = "engine-input-json")]
    pub(crate) engine_input_json: Option<PathBuf>,
    #[serde(alias = "input_source_map", alias = "input-source-map")]
    pub(crate) input_source_maps: Option<Vec<String>>,
    #[serde(
        alias = "allow_logical_to_physical",
        alias = "allow-logical-to-physical"
    )]
    pub(crate) allow_logical_to_physical: Option<bool>,
    #[serde(alias = "allow_scope_flatten", alias = "allow-scope-flatten")]
    pub(crate) allow_scope_flatten: Option<bool>,
    #[serde(alias = "allow_layer_flatten", alias = "allow-layer-flatten")]
    pub(crate) allow_layer_flatten: Option<bool>,
    #[serde(
        alias = "enable_supports_static_eval",
        alias = "enable-supports-static-eval"
    )]
    pub(crate) enable_supports_static_eval: Option<bool>,
    #[serde(alias = "enable_media_static_eval", alias = "enable-media-static-eval")]
    pub(crate) enable_media_static_eval: Option<bool>,
    #[serde(
        alias = "enable_container_static_eval",
        alias = "enable-container-static-eval"
    )]
    pub(crate) enable_container_static_eval: Option<bool>,
    #[serde(
        alias = "drop_dark_mode_media_queries",
        alias = "drop-dark-mode-media-queries"
    )]
    pub(crate) drop_dark_mode_media_queries: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrManyStrings {
    One(String),
    Many(Vec<String>),
}

fn deserialize_string_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<OneOrManyStrings>::deserialize(deserializer)? {
            None => Vec::new(),
            Some(OneOrManyStrings::One(value)) => vec![value],
            Some(OneOrManyStrings::Many(values)) => values,
        },
    )
}

fn deserialize_path_list<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_string_list(deserializer)
        .map(|values| values.into_iter().map(PathBuf::from).collect())
}

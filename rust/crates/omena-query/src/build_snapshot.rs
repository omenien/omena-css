use omena_query_transform_runner::{
    TargetTransformOptionsV0, all_transform_pass_kinds, plan_target_transforms_from_query,
    plan_transform_passes,
};
use omena_resolver::omena_resolver_style_identity_generation;
use omena_sif::{compute_omena_sif_leaf_hash_v1, write_omena_canonical_json_bytes_v1};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0};

const BUILD_SNAPSHOT_SCHEMA_VERSION_V0: &str = "0";
const BUILD_SNAPSHOT_PRODUCT_V0: &str = "omena-query.build-snapshot-digest";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBuildSnapshotContentInputV0 {
    pub path: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBuildSnapshotIdentityInputV0 {
    pub target_path: String,
    pub style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    pub package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    #[serde(default)]
    pub config_inputs: Vec<OmenaQueryBuildSnapshotContentInputV0>,
    #[serde(default)]
    pub pass_ids: Vec<String>,
    pub target_query: Option<String>,
    #[serde(default)]
    pub target_options: TargetTransformOptionsV0,
    #[serde(default)]
    pub adapter_environment: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBuildSnapshotContentDigestV0 {
    pub path: String,
    pub content_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBuildSnapshotIdentityV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub content_hash_algorithm: &'static str,
    pub digest: String,
    pub target_source_digest: String,
    pub style_source_digests: Vec<OmenaQueryBuildSnapshotContentDigestV0>,
    pub package_manifest_digests: Vec<OmenaQueryBuildSnapshotContentDigestV0>,
    pub config_digests: Vec<OmenaQueryBuildSnapshotContentDigestV0>,
    pub resolver_generation: u64,
    pub target_data_snapshot_id: String,
    pub engine_abi_version: String,
    pub pass_plan_digest: String,
    pub requested_pass_ids: Vec<String>,
    pub ordered_pass_ids: Vec<String>,
    pub unknown_pass_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct BuildSnapshotNativeIdentityV0 {
    resolver_generation: u64,
    target_data_snapshot_id: String,
    engine_abi_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct BuildSnapshotDigestMaterialV0<'a> {
    schema_version: &'static str,
    product: &'static str,
    target_path: &'a str,
    target_source_digest: &'a str,
    style_source_digests: &'a [OmenaQueryBuildSnapshotContentDigestV0],
    package_manifest_digests: &'a [OmenaQueryBuildSnapshotContentDigestV0],
    config_digests: &'a [OmenaQueryBuildSnapshotContentDigestV0],
    native_identity: &'a BuildSnapshotNativeIdentityV0,
    pass_plan_digest: &'a str,
    requested_pass_ids: &'a [String],
    ordered_pass_ids: &'a [String],
    unknown_pass_ids: &'a [String],
    target_query: &'a Option<String>,
    target_options: TargetTransformOptionsV0,
    adapter_environment: &'a Value,
}

pub fn compute_omena_query_build_snapshot_identity_v0(
    input: &OmenaQueryBuildSnapshotIdentityInputV0,
) -> Result<OmenaQueryBuildSnapshotIdentityV0, String> {
    let target_plan = plan_target_transforms_from_query(
        input.target_query.as_deref().unwrap_or("modern"),
        input.target_options,
    );
    let native_identity = BuildSnapshotNativeIdentityV0 {
        resolver_generation: omena_resolver_style_identity_generation(),
        target_data_snapshot_id: target_plan.target_data_snapshot_id,
        engine_abi_version: format!("omena-query@{}", env!("CARGO_PKG_VERSION")),
    };
    compute_build_snapshot_identity_with_native_facts(input, native_identity)
}

fn compute_build_snapshot_identity_with_native_facts(
    input: &OmenaQueryBuildSnapshotIdentityInputV0,
    native_identity: BuildSnapshotNativeIdentityV0,
) -> Result<OmenaQueryBuildSnapshotIdentityV0, String> {
    let style_source_digests = digest_style_sources(input.style_sources.as_slice())?;
    let package_manifest_digests = digest_package_manifests(input.package_manifests.as_slice())?;
    let config_digests = digest_content_inputs(input.config_inputs.as_slice(), "config")?;
    let target_source_digest = style_source_digests
        .iter()
        .find(|entry| entry.path == input.target_path)
        .map(|entry| entry.content_digest.clone())
        .ok_or_else(|| {
            format!(
                "build snapshot target {} is absent from styleSources",
                input.target_path
            )
        })?;

    let available_passes = all_transform_pass_kinds();
    let mut requested_passes = Vec::new();
    let mut unknown_pass_ids = Vec::new();
    for pass_id in &input.pass_ids {
        match available_passes
            .iter()
            .copied()
            .find(|candidate| candidate.id() == pass_id)
        {
            Some(pass) => requested_passes.push(pass),
            None => unknown_pass_ids.push(pass_id.clone()),
        }
    }
    let pass_plan = plan_transform_passes(requested_passes.as_slice());
    let ordered_pass_ids = pass_plan
        .ordered_pass_ids
        .iter()
        .map(|pass_id| (*pass_id).to_owned())
        .collect::<Vec<_>>();
    let pass_plan_digest = canonical_digest(&pass_plan)?;

    let material = BuildSnapshotDigestMaterialV0 {
        schema_version: BUILD_SNAPSHOT_SCHEMA_VERSION_V0,
        product: BUILD_SNAPSHOT_PRODUCT_V0,
        target_path: input.target_path.as_str(),
        target_source_digest: target_source_digest.as_str(),
        style_source_digests: style_source_digests.as_slice(),
        package_manifest_digests: package_manifest_digests.as_slice(),
        config_digests: config_digests.as_slice(),
        native_identity: &native_identity,
        pass_plan_digest: pass_plan_digest.as_str(),
        requested_pass_ids: input.pass_ids.as_slice(),
        ordered_pass_ids: ordered_pass_ids.as_slice(),
        unknown_pass_ids: unknown_pass_ids.as_slice(),
        target_query: &input.target_query,
        target_options: input.target_options,
        adapter_environment: &input.adapter_environment,
    };
    let digest = canonical_digest(&material)?;

    Ok(OmenaQueryBuildSnapshotIdentityV0 {
        schema_version: BUILD_SNAPSHOT_SCHEMA_VERSION_V0,
        product: BUILD_SNAPSHOT_PRODUCT_V0,
        content_hash_algorithm: "blake3",
        digest,
        target_source_digest,
        style_source_digests,
        package_manifest_digests,
        config_digests,
        resolver_generation: native_identity.resolver_generation,
        target_data_snapshot_id: native_identity.target_data_snapshot_id,
        engine_abi_version: native_identity.engine_abi_version,
        pass_plan_digest,
        requested_pass_ids: input.pass_ids.clone(),
        ordered_pass_ids,
        unknown_pass_ids,
    })
}

fn digest_style_sources(
    inputs: &[OmenaQueryStyleSourceInputV0],
) -> Result<Vec<OmenaQueryBuildSnapshotContentDigestV0>, String> {
    digest_path_sources(
        inputs
            .iter()
            .map(|input| (input.style_path.as_str(), input.style_source.as_str())),
        "style source",
    )
}

fn digest_package_manifests(
    inputs: &[OmenaQueryStylePackageManifestV0],
) -> Result<Vec<OmenaQueryBuildSnapshotContentDigestV0>, String> {
    digest_path_sources(
        inputs.iter().map(|input| {
            (
                input.package_json_path.as_str(),
                input.package_json_source.as_str(),
            )
        }),
        "package manifest",
    )
}

fn digest_content_inputs(
    inputs: &[OmenaQueryBuildSnapshotContentInputV0],
    kind: &str,
) -> Result<Vec<OmenaQueryBuildSnapshotContentDigestV0>, String> {
    digest_path_sources(
        inputs
            .iter()
            .map(|input| (input.path.as_str(), input.source.as_str())),
        kind,
    )
}

fn digest_path_sources<'a>(
    inputs: impl IntoIterator<Item = (&'a str, &'a str)>,
    kind: &str,
) -> Result<Vec<OmenaQueryBuildSnapshotContentDigestV0>, String> {
    let mut digests = inputs
        .into_iter()
        .map(|(path, source)| OmenaQueryBuildSnapshotContentDigestV0 {
            path: path.to_owned(),
            content_digest: compute_omena_sif_leaf_hash_v1(source.as_bytes())
                .as_str()
                .to_owned(),
        })
        .collect::<Vec<_>>();
    digests.sort_by(|left, right| left.path.cmp(&right.path));
    for pair in digests.windows(2) {
        if pair[0].path == pair[1].path {
            return Err(format!("duplicate {kind} path: {}", pair[0].path));
        }
    }
    Ok(digests)
}

fn canonical_digest(value: &impl Serialize) -> Result<String, String> {
    let bytes = write_omena_canonical_json_bytes_v1(value)
        .map_err(|error| format!("build snapshot canonicalization failed: {error}"))?;
    Ok(compute_omena_sif_leaf_hash_v1(bytes.as_slice())
        .as_str()
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_resolver::invalidate_omena_resolver_style_identity_cache;

    fn fixture_input() -> OmenaQueryBuildSnapshotIdentityInputV0 {
        OmenaQueryBuildSnapshotIdentityInputV0 {
            target_path: "/workspace/App.module.css".to_owned(),
            style_sources: vec![
                OmenaQueryStyleSourceInputV0 {
                    style_path: "/workspace/App.module.css".to_owned(),
                    style_source: "@import \"./tokens.module.css\";".to_owned(),
                },
                OmenaQueryStyleSourceInputV0 {
                    style_path: "/workspace/tokens.module.css".to_owned(),
                    style_source: ":root { --brand: red; }".to_owned(),
                },
            ],
            package_manifests: vec![OmenaQueryStylePackageManifestV0 {
                package_json_path: "/workspace/package.json".to_owned(),
                package_json_source: "{\"exports\":{}}".to_owned(),
            }],
            config_inputs: vec![OmenaQueryBuildSnapshotContentInputV0 {
                path: "/workspace/omena.config.json".to_owned(),
                source: "{\"build\":{\"minify\":false}}".to_owned(),
            }],
            pass_ids: vec!["comment-strip".to_owned()],
            target_query: Some("chrome 122".to_owned()),
            target_options: TargetTransformOptionsV0::default(),
            adapter_environment: serde_json::json!({"sourceMap": true}),
        }
    }

    fn native_facts() -> BuildSnapshotNativeIdentityV0 {
        BuildSnapshotNativeIdentityV0 {
            resolver_generation: 7,
            target_data_snapshot_id: "blake3:target-data".to_owned(),
            engine_abi_version: "omena-query@0.4.0".to_owned(),
        }
    }

    #[test]
    fn every_content_family_changes_the_build_snapshot_digest() -> Result<(), String> {
        let baseline = fixture_input();
        let baseline_digest =
            compute_build_snapshot_identity_with_native_facts(&baseline, native_facts())?.digest;

        let mut source_changed = baseline.clone();
        source_changed.style_sources[1].style_source = ":root { --brand: blue; }".to_owned();
        let mut manifest_changed = baseline.clone();
        manifest_changed.package_manifests[0].package_json_source =
            "{\"exports\":{\".\":\"./blue.css\"}}".to_owned();
        let mut config_changed = baseline.clone();
        config_changed.config_inputs[0].source = "{\"build\":{\"minify\":true}}".to_owned();

        for (family, changed) in [
            ("style-source", source_changed),
            ("package-manifest", manifest_changed),
            ("config", config_changed),
        ] {
            let digest =
                compute_build_snapshot_identity_with_native_facts(&changed, native_facts())?.digest;
            assert_ne!(
                digest, baseline_digest,
                "{family} content must change the build snapshot digest"
            );
        }
        Ok(())
    }

    #[test]
    fn every_native_identity_family_changes_the_build_snapshot_digest() -> Result<(), String> {
        let input = fixture_input();
        let baseline = compute_build_snapshot_identity_with_native_facts(&input, native_facts())?;

        let mut resolver_changed = native_facts();
        resolver_changed.resolver_generation += 1;
        let mut target_data_changed = native_facts();
        target_data_changed.target_data_snapshot_id = "blake3:new-target-data".to_owned();
        let mut abi_changed = native_facts();
        abi_changed.engine_abi_version = "omena-query@next".to_owned();

        for (family, facts) in [
            ("resolver-generation", resolver_changed),
            ("target-data-snapshot", target_data_changed),
            ("engine-abi", abi_changed),
        ] {
            let changed = compute_build_snapshot_identity_with_native_facts(&input, facts)?;
            assert_ne!(
                changed.digest, baseline.digest,
                "{family} must change the build snapshot digest"
            );
        }
        Ok(())
    }

    #[test]
    fn live_resolver_generation_changes_the_public_build_snapshot_digest() -> Result<(), String> {
        let input = fixture_input();
        let before = compute_omena_query_build_snapshot_identity_v0(&input)?;
        invalidate_omena_resolver_style_identity_cache();
        let after = compute_omena_query_build_snapshot_identity_v0(&input)?;

        assert_eq!(after.resolver_generation, before.resolver_generation + 1);
        assert_ne!(after.digest, before.digest);
        Ok(())
    }

    #[test]
    fn pass_plan_and_target_query_changes_are_bound() -> Result<(), String> {
        let baseline = fixture_input();
        let baseline_identity =
            compute_build_snapshot_identity_with_native_facts(&baseline, native_facts())?;

        let mut pass_changed = baseline.clone();
        pass_changed.pass_ids.push("color-compression".to_owned());
        let pass_identity =
            compute_build_snapshot_identity_with_native_facts(&pass_changed, native_facts())?;
        assert_ne!(
            pass_identity.pass_plan_digest,
            baseline_identity.pass_plan_digest
        );
        assert_ne!(pass_identity.digest, baseline_identity.digest);

        let mut target_changed = baseline;
        target_changed.target_query = Some("safari 16.2".to_owned());
        let target_identity =
            compute_build_snapshot_identity_with_native_facts(&target_changed, native_facts())?;
        assert_ne!(target_identity.digest, baseline_identity.digest);
        Ok(())
    }

    #[test]
    fn duplicate_or_missing_style_paths_fail_closed() -> Result<(), String> {
        let mut missing = fixture_input();
        missing.style_sources.remove(0);
        let missing_error = match compute_build_snapshot_identity_with_native_facts(
            &missing,
            native_facts(),
        ) {
            Err(error) => error,
            Ok(_) => return Err("a missing target source must fail closed".to_owned()),
        };
        assert!(
            missing_error.contains("absent from styleSources"),
            "missing target source failure must identify the absent style source"
        );

        let mut duplicate = fixture_input();
        duplicate
            .style_sources
            .push(duplicate.style_sources[0].clone());
        let duplicate_error = match compute_build_snapshot_identity_with_native_facts(
            &duplicate,
            native_facts(),
        ) {
            Err(error) => error,
            Ok(_) => return Err("a duplicate style source path must fail closed".to_owned()),
        };
        assert!(
            duplicate_error.contains("duplicate style source path"),
            "duplicate style source failure must identify the duplicated path"
        );
        Ok(())
    }
}

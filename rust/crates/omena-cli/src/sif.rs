use crate::{
    commands::SifCommand,
    io::read_source,
    paths::path_string,
    workspace_edit_transaction::{
        ExpectedContentDigestV0, FileEditV0, WorkspaceEditPostconditionV0,
        WorkspaceEditSafetyClassV0, WorkspaceEditTransaction,
    },
};
use omena_sif::{
    OmenaSifSourceSyntaxV1, OmenaSifStaticGeneratorInputV1, OmenaSifTrustTierV1,
    build_omena_sif_published_attestation_subject_v1, generate_static_omena_lif_exports_v1,
    generate_static_omena_sif_v1, read_omena_sif_json_v1, write_omena_lif_exports_json_v1,
    write_omena_sif_json_v1, write_omena_sif_published_attestation_subject_json_v1,
};
use std::path::{Path, PathBuf};

pub(crate) fn sif_command(command: SifCommand) -> Result<(), String> {
    match command {
        SifCommand::Generate {
            path,
            canonical_url,
            output,
            syntax,
            json,
        } => generate_sif(path, canonical_url, output, syntax, json),
        SifCommand::GenerateAttestationSubject {
            sif,
            trust_tier,
            output,
            json,
        } => generate_attestation_subject(sif, trust_tier, output, json),
        SifCommand::GenerateLifExports {
            path,
            output,
            syntax,
            json,
        } => generate_lif_exports(path, output, syntax, json),
    }
}

fn generate_attestation_subject(
    sif_path: PathBuf,
    trust_tier: String,
    output: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let expected_output_digest = output
        .as_deref()
        .map(ExpectedContentDigestV0::observe)
        .transpose()
        .map_err(|error| error.to_string())?;
    let source = read_source(&sif_path)?;
    let expected_source_digest =
        ExpectedContentDigestV0::from_bytes(sif_path.as_path(), source.as_bytes());
    let sif = read_omena_sif_json_v1(source.as_str()).map_err(|error| {
        format!(
            "failed to parse canonical SIF {}: {error}",
            path_string(&sif_path)
        )
    })?;
    let trust_tier = match trust_tier.as_str() {
        "t2" => OmenaSifTrustTierV1::T2,
        "t3" => OmenaSifTrustTierV1::T3,
        value => {
            return Err(format!(
                "unsupported published SIF trust tier '{value}'; expected t2 or t3"
            ));
        }
    };
    let subject = build_omena_sif_published_attestation_subject_v1(&sif, trust_tier)
        .map_err(|error| format!("failed to build published SIF attestation subject: {error}"))?;
    let subject_json = write_omena_sif_published_attestation_subject_json_v1(&subject)
        .map_err(|error| format!("failed to serialize published SIF subject: {error}"))?;
    let wrote_output = output.is_some();
    if let Some(output_path) = output {
        commit_sif_json_output(
            output_path.as_path(),
            subject_json.as_bytes(),
            expected_source_digest,
            expected_output_digest.ok_or_else(|| {
                "SIF attestation output digest was not captured before analysis".to_string()
            })?,
        )?;
        if !json {
            println!(
                "generated SIF attestation subject: {}",
                path_string(&output_path)
            );
        }
    }
    if !wrote_output || json {
        println!("{subject_json}");
    }
    Ok(())
}

fn generate_sif(
    path: PathBuf,
    canonical_url: Option<String>,
    output: Option<PathBuf>,
    syntax: Option<String>,
    json: bool,
) -> Result<(), String> {
    let expected_output_digest = output
        .as_deref()
        .map(ExpectedContentDigestV0::observe)
        .transpose()
        .map_err(|error| error.to_string())?;
    let source = read_source(&path)?;
    let expected_source_digest =
        ExpectedContentDigestV0::from_bytes(path.as_path(), source.as_bytes());
    let syntax = match syntax {
        Some(syntax) => parse_sif_source_syntax(&syntax)?,
        None => infer_sif_source_syntax(&path),
    };
    let canonical_url = canonical_url.unwrap_or_else(|| path_string(&path));
    let sif = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
        canonical_url: &canonical_url,
        source: &source,
        syntax,
    })
    .map_err(|error| format!("failed to generate SIF: {error}"))?;
    let sif_json = write_omena_sif_json_v1(&sif)
        .map_err(|error| format!("failed to serialize SIF: {error}"))?;
    let wrote_output = output.is_some();

    if let Some(output_path) = output {
        commit_sif_json_output(
            output_path.as_path(),
            sif_json.as_bytes(),
            expected_source_digest,
            expected_output_digest
                .ok_or_else(|| "SIF output digest was not captured before analysis".to_string())?,
        )?;
        if !json {
            println!("generated SIF: {}", path_string(&output_path));
        }
    }

    if !wrote_output || json {
        println!("{sif_json}");
    }

    Ok(())
}

fn generate_lif_exports(
    path: PathBuf,
    output: Option<PathBuf>,
    syntax: Option<String>,
    json: bool,
) -> Result<(), String> {
    let expected_output_digest = output
        .as_deref()
        .map(ExpectedContentDigestV0::observe)
        .transpose()
        .map_err(|error| error.to_string())?;
    let source = read_source(&path)?;
    let expected_source_digest =
        ExpectedContentDigestV0::from_bytes(path.as_path(), source.as_bytes());
    let syntax = match syntax {
        Some(syntax) => parse_sif_source_syntax(&syntax)?,
        None => infer_sif_source_syntax(&path),
    };
    let canonical_url = path_string(&path);
    let exports = generate_static_omena_lif_exports_v1(OmenaSifStaticGeneratorInputV1 {
        canonical_url: &canonical_url,
        source: &source,
        syntax,
    });
    let exports_json = write_omena_lif_exports_json_v1(&exports)
        .map_err(|error| format!("failed to serialize LIF exports: {error}"))?;
    let wrote_output = output.is_some();

    if let Some(output_path) = output {
        commit_sif_json_output(
            output_path.as_path(),
            exports_json.as_bytes(),
            expected_source_digest,
            expected_output_digest.ok_or_else(|| {
                "LIF exports output digest was not captured before analysis".to_string()
            })?,
        )?;
        if !json {
            println!("generated LIF exports: {}", path_string(&output_path));
        }
    }

    if !wrote_output || json {
        println!("{exports_json}");
    }

    Ok(())
}

fn commit_sif_json_output(
    output_path: &Path,
    content: &[u8],
    expected_source_digest: ExpectedContentDigestV0,
    expected_output_digest: ExpectedContentDigestV0,
) -> Result<(), String> {
    WorkspaceEditTransaction::new(None, WorkspaceEditSafetyClassV0::EvidenceRequired)
        .expect(expected_source_digest)
        .expect(expected_output_digest)
        .edit(
            FileEditV0::new(output_path, content)
                .with_postcondition(WorkspaceEditPostconditionV0::json_reparse())
                .with_postcondition(WorkspaceEditPostconditionV0::byte_identity(content)),
        )
        .commit()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_sif_source_syntax(syntax: &str) -> Result<OmenaSifSourceSyntaxV1, String> {
    match syntax {
        "css" => Ok(OmenaSifSourceSyntaxV1::Css),
        "scss" => Ok(OmenaSifSourceSyntaxV1::Scss),
        "sass" => Ok(OmenaSifSourceSyntaxV1::Sass),
        "less" => Ok(OmenaSifSourceSyntaxV1::Less),
        _ => Err(format!(
            "unsupported SIF source syntax '{syntax}'; expected css, scss, sass, or less"
        )),
    }
}

fn infer_sif_source_syntax(path: &Path) -> OmenaSifSourceSyntaxV1 {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("css") => OmenaSifSourceSyntaxV1::Css,
        Some("sass") => OmenaSifSourceSyntaxV1::Sass,
        Some("less") => OmenaSifSourceSyntaxV1::Less,
        _ => OmenaSifSourceSyntaxV1::Scss,
    }
}

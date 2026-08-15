use crate::{commands::SifCommand, io::read_source, paths::path_string};
use omena_sif::{
    OmenaSifSourceSyntaxV1, OmenaSifStaticGeneratorInputV1, OmenaSifTrustTierV1,
    build_omena_sif_published_attestation_subject_v1, generate_static_omena_lif_exports_v1,
    generate_static_omena_sif_v1, read_omena_sif_json_v1, write_omena_lif_exports_json_v1,
    write_omena_sif_json_v1, write_omena_sif_published_attestation_subject_json_v1,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

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
    let source = read_source(&sif_path)?;
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
        fs::write(&output_path, &subject_json).map_err(|error| {
            format!(
                "failed to write published SIF subject to {}: {error}",
                path_string(&output_path)
            )
        })?;
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
    let source = read_source(&path)?;
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
        fs::write(&output_path, &sif_json).map_err(|error| {
            format!(
                "failed to write SIF artifact to {}: {error}",
                path_string(&output_path)
            )
        })?;
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
    let source = read_source(&path)?;
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
        fs::write(&output_path, &exports_json).map_err(|error| {
            format!(
                "failed to write LIF exports to {}: {error}",
                path_string(&output_path)
            )
        })?;
        if !json {
            println!("generated LIF exports: {}", path_string(&output_path));
        }
    }

    if !wrote_output || json {
        println!("{exports_json}");
    }

    Ok(())
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

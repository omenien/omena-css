use omena_query::{
    OmenaQueryEngineInputV2, OmenaQueryExplainInputV0, OmenaQueryExplainResponseV0,
    OmenaQueryExplainSymbolKindV0, ParserPositionV0,
    execute_omena_query_transform_passes_from_source, explain_omena_query,
    explain_omena_query_tree_shake_for_style_source, read_omena_query_cascade_at_position,
    resolve_omena_query_source_precision_for_source,
    summarize_omena_query_style_diagnostics_for_file,
};

use crate::{
    commands::{ExplainCommand, ExplainSymbolKind},
    io::{read_context_json, read_source},
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};

pub(crate) fn explain_command(command: ExplainCommand) -> Result<(), String> {
    let (response, json) = resolve_explain_command(command)?;
    if json {
        print_json(CliOutputMetadataV0::new("omena-cli.explain"), &response)
    } else {
        let rendered = serde_json::to_string_pretty(&response)
            .map_err(|error| format!("failed to render explain response: {error}"))?;
        println!("{rendered}");
        Ok(())
    }
}

pub(crate) fn resolve_explain_command(
    command: ExplainCommand,
) -> Result<(OmenaQueryExplainResponseV0, bool), String> {
    match command {
        ExplainCommand::Diagnostic { path, code, json } => {
            let source = read_source(&path)?;
            let style_path = path_string(&path);
            let diagnostics =
                summarize_omena_query_style_diagnostics_for_file(&style_path, &source, &[]);
            let diagnostic = diagnostics
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code == code)
                .ok_or_else(|| format!("diagnostic {code:?} was not produced for {style_path}"))?;
            Ok((
                explain_omena_query(OmenaQueryExplainInputV0::Diagnostic {
                    style_path: &style_path,
                    diagnostic,
                }),
                json,
            ))
        }
        ExplainCommand::Transform {
            path,
            pass_id,
            json,
        } => {
            let source = read_source(&path)?;
            let style_path = path_string(&path);
            let execution = execute_omena_query_transform_passes_from_source(
                &style_path,
                &source,
                std::slice::from_ref(&pass_id),
            );
            if !execution.unknown_pass_ids.is_empty() {
                return Err(format!("unknown transform pass {pass_id:?}"));
            }
            let (decision_ordinal, decision) = execution
                .execution
                .decisions
                .iter()
                .enumerate()
                .find(|(_, decision)| decision.compatibility_outcome().pass_id == pass_id)
                .ok_or_else(|| format!("transform pass {pass_id:?} produced no decision"))?;
            Ok((
                explain_omena_query(OmenaQueryExplainInputV0::Transform {
                    decision,
                    decision_ordinal,
                }),
                json,
            ))
        }
        ExplainCommand::WhyNotTreeShaken {
            path,
            symbol_kind,
            symbol,
            context_json,
            json,
        } => {
            let source = read_source(&path)?;
            let style_path = path_string(&path);
            let context = read_context_json(Some(context_json.as_path()))?;
            let response = explain_omena_query_tree_shake_for_style_source(
                &style_path,
                &source,
                &context,
                explain_symbol_kind(symbol_kind),
                &symbol,
            )
            .ok_or_else(|| {
                "closed-world reachability is unavailable; provide a context with reachable roots"
                    .to_string()
            })?;
            Ok((response, json))
        }
        ExplainCommand::Precision {
            path,
            variable,
            byte_offset,
            source_language,
            json,
        } => {
            let source = read_source(&path)?;
            let source_path = path_string(&path);
            let reference = resolve_omena_query_source_precision_for_source(
                &source_path,
                &source,
                source_language.as_deref(),
                &variable,
                byte_offset,
            );
            Ok((
                explain_omena_query(OmenaQueryExplainInputV0::Precision {
                    reference: &reference,
                }),
                json,
            ))
        }
        ExplainCommand::Cascade {
            path,
            line,
            character,
            json,
        } => {
            let source = read_source(&path)?;
            let style_path = path_string(&path);
            let result = read_omena_query_cascade_at_position(
                &style_path,
                &source,
                &empty_engine_input(),
                ParserPositionV0 { line, character },
            )
            .ok_or_else(|| {
                format!("failed to read cascade information for {style_path}:{line}:{character}")
            })?;
            Ok((
                explain_omena_query(OmenaQueryExplainInputV0::Cascade { result: &result }),
                json,
            ))
        }
        ExplainCommand::Bundle { chunk, json } => Ok((
            explain_omena_query(OmenaQueryExplainInputV0::BundleUnavailable {
                chunk_reference: &chunk,
            }),
            json,
        )),
    }
}

fn explain_symbol_kind(kind: ExplainSymbolKind) -> OmenaQueryExplainSymbolKindV0 {
    match kind {
        ExplainSymbolKind::Class => OmenaQueryExplainSymbolKindV0::Class,
        ExplainSymbolKind::Keyframes => OmenaQueryExplainSymbolKindV0::Keyframes,
        ExplainSymbolKind::Value => OmenaQueryExplainSymbolKindV0::Value,
        ExplainSymbolKind::CustomProperty => OmenaQueryExplainSymbolKindV0::CustomProperty,
    }
}

fn empty_engine_input() -> OmenaQueryEngineInputV2 {
    OmenaQueryEngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, time::SystemTime};

    use omena_query::OmenaQueryExplainAvailabilityV0;

    use super::*;

    #[test]
    fn six_explain_targets_route_through_the_shared_egress() -> Result<(), String> {
        let root = temp_dir("six-targets");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let diagnostic_path = root.join("diagnostic.scss");
        let style_path = root.join("fixture.css");
        let source_path = root.join("fixture.ts");
        let context_path = root.join("context.json");
        fs::write(&diagnostic_path, "@import 'legacy';\n").map_err(|error| error.to_string())?;
        fs::write(
            &style_path,
            ":root { --tone: red; }\n.button { color: var(--tone); }\n",
        )
        .map_err(|error| error.to_string())?;
        let source = "const className = 'button';\nclassName;";
        fs::write(&source_path, source).map_err(|error| error.to_string())?;
        fs::write(&context_path, r#"{"reachableClassNames":["button"]}"#)
            .map_err(|error| error.to_string())?;

        let commands = vec![
            ExplainCommand::Diagnostic {
                path: diagnostic_path,
                code: "deprecatedSassImport".to_string(),
                json: true,
            },
            ExplainCommand::Transform {
                path: style_path.clone(),
                pass_id: "print-css".to_string(),
                json: true,
            },
            ExplainCommand::WhyNotTreeShaken {
                path: style_path.clone(),
                symbol_kind: ExplainSymbolKind::Class,
                symbol: "button".to_string(),
                context_json: context_path,
                json: true,
            },
            ExplainCommand::Precision {
                path: source_path,
                variable: "className".to_string(),
                byte_offset: source
                    .rfind("className")
                    .ok_or_else(|| "fixture source reference is missing".to_string())?,
                source_language: Some("typescript".to_string()),
                json: true,
            },
            ExplainCommand::Cascade {
                path: style_path,
                line: 1,
                character: 22,
                json: true,
            },
            ExplainCommand::Bundle {
                chunk: "main".to_string(),
                json: true,
            },
        ];

        let mut availabilities = Vec::new();
        for command in commands {
            let (response, json) = resolve_explain_command(command)?;
            assert!(json);
            availabilities.push(response.availability());
        }
        assert_eq!(
            availabilities,
            vec![
                OmenaQueryExplainAvailabilityV0::Available,
                OmenaQueryExplainAvailabilityV0::Available,
                OmenaQueryExplainAvailabilityV0::Available,
                OmenaQueryExplainAvailabilityV0::Available,
                OmenaQueryExplainAvailabilityV0::Available,
                OmenaQueryExplainAvailabilityV0::NotYetAvailable,
            ]
        );

        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        std::env::temp_dir().join(format!(
            "omena-explain-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}

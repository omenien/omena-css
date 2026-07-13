use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use omena_checker::{
    FixSafetyAssessmentV0, FixSafetyEvidenceInputV0, FixSafetyV0, compute_fix_safety,
};
use omena_query::{
    FactPrecision, OmenaParserStyleDialect, OmenaQueryReferenceLocationV0,
    OmenaQueryRollbackReceiptV0, OmenaQueryRollbackScopeV0, OmenaQueryWorkspaceTextEditV0,
    ParserByteSpanV0, ParserRangeV0, summarize_omena_query_custom_property_occurrence_index,
    summarize_omena_query_refs_for_workspace_class,
    summarize_omena_query_rename_plan_for_workspace_class,
    summarize_omena_query_sass_module_cross_file_resolution_for_workspace,
    summarize_omena_query_sass_module_source_edges,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    commands::{MigrateCommand, MigrationModeArgs},
    io::{read_package_manifests, read_source_documents, read_style_sources},
    lint::discover_workspace_files,
    output::{CliOutputMetadataV0, print_json, write_json_artifact},
    paths::{cli_file_uri_to_path, path_string},
    text_edit::{apply_byte_edit, byte_span_for_range, range_for_byte_span},
    write_safety::{
        SourceWriteEvidenceV0, SourceWriteModeV0, SourceWriteReportV0, apply_write_with_safety,
    },
};

const MIGRATION_PLAN_SCHEMA_VERSION: &str = "0";
const MIGRATION_PLAN_PRODUCT: &str = "omena-cli.migration-plan";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum MigrationCodemodV0 {
    CssModulesRename,
    SassImportToUse,
    TokenRename,
}

impl MigrationCodemodV0 {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CssModulesRename => "cssModulesRename",
            Self::SassImportToUse => "sassImportToUse",
            Self::TokenRename => "tokenRename",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationEvidenceV0 {
    id: String,
    kind: String,
    source: String,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationEditEvidenceV0 {
    primary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    supporting: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationEditV0 {
    id: String,
    uri: String,
    range: ParserRangeV0,
    byte_span: ParserByteSpanV0,
    expected_text: String,
    replacement_text: String,
    expected_source_sha256: String,
    safety_evidence: FixSafetyEvidenceInputV0,
    evidence: MigrationEditEvidenceV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationBlockerV0 {
    code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    uri: Option<String>,
    detail: String,
    evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationInverseEditV0 {
    edit_id: String,
    uri: String,
    byte_span: ParserByteSpanV0,
    expected_text: String,
    replacement_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationRollbackPlanV0 {
    receipt_typed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt: Option<OmenaQueryRollbackReceiptV0>,
    inverse_edits: Vec<MigrationInverseEditV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationPlanV0 {
    schema_version: String,
    product: String,
    codemod: MigrationCodemodV0,
    workspace_root: String,
    edits: Vec<MigrationEditV0>,
    safe_edits: Vec<String>,
    review_edits: Vec<String>,
    blockers: Vec<MigrationBlockerV0>,
    evidence: Vec<MigrationEvidenceV0>,
    rollback: MigrationRollbackPlanV0,
}

#[derive(Debug, Clone)]
struct MigrationEditDraftV0 {
    uri: String,
    range: ParserRangeV0,
    byte_span: ParserByteSpanV0,
    expected_text: String,
    replacement_text: String,
    expected_source_sha256: String,
    safety_evidence: FixSafetyEvidenceInputV0,
    evidence: MigrationEditEvidenceV0,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationApplyFileV0 {
    path: String,
    input_content_signature: String,
    output_content_signature: String,
    edit_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationApplyReportV0 {
    schema_version: &'static str,
    product: &'static str,
    codemod: MigrationCodemodV0,
    plan_path: String,
    approve_review: bool,
    applied_edit_count: usize,
    applied_file_count: usize,
    files: Vec<MigrationApplyFileV0>,
    write_reports: Vec<SourceWriteReportV0>,
    rollback: MigrationRollbackPlanV0,
}

struct PreparedMigrationWriteV0 {
    path: PathBuf,
    content: String,
    assessment: FixSafetyAssessmentV0,
    input_content_signature: String,
    output_content_signature: String,
    edit_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SassMigrationOracleRequestV0 {
    schema_version: &'static str,
    product: &'static str,
    workspace_root: String,
    files: Vec<SassMigrationOracleFileV0>,
    edits: Vec<SassMigrationOracleEditV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SassMigrationOracleFileV0 {
    path: String,
    source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SassMigrationOracleEditV0 {
    uri: String,
    start: usize,
    end: usize,
    replacement_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SassMigrationOracleResultV0 {
    schema_version: String,
    product: String,
    compiler: SassMigrationOracleCompilerV0,
    all_matched: bool,
    results: Vec<SassMigrationOracleFileResultV0>,
}

#[derive(Debug, Deserialize)]
struct SassMigrationOracleCompilerV0 {
    name: String,
    package: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SassMigrationOracleFileResultV0 {
    uri: String,
    matched: bool,
    before_status: Option<i32>,
    after_status: Option<i32>,
    before_css_sha256: Option<String>,
    after_css_sha256: Option<String>,
    before_stderr: String,
    after_stderr: String,
}

pub(crate) fn migrate_command(command: MigrateCommand) -> Result<(), String> {
    match command {
        MigrateCommand::CssModulesRename {
            selector_name,
            new_name,
            root,
            target_style,
            mode,
        } => run_migration_mode(MigrationCodemodV0::CssModulesRename, mode, || {
            build_css_modules_rename_plan(selector_name, new_name, root, target_style)
        }),
        MigrateCommand::SassImportToUse { root, mode } => {
            run_migration_mode(MigrationCodemodV0::SassImportToUse, mode, || {
                build_sass_import_to_use_plan(root)
            })
        }
        MigrateCommand::TokenRename {
            token_name,
            new_name,
            root,
            mode,
        } => run_migration_mode(MigrationCodemodV0::TokenRename, mode, || {
            build_token_rename_plan(token_name, new_name, root)
        }),
    }
}

fn build_token_rename_plan(
    token_name: Option<String>,
    new_name: Option<String>,
    root: Option<PathBuf>,
) -> Result<MigrationPlanV0, String> {
    let token_name = normalize_custom_property_name(
        token_name
            .as_deref()
            .ok_or_else(|| "token-rename planning requires TOKEN_NAME".to_string())?,
    )?;
    let new_name = normalize_custom_property_name(
        new_name
            .as_deref()
            .ok_or_else(|| "token-rename planning requires NEW_NAME".to_string())?,
    )?;
    if token_name == new_name {
        return Err("custom-property name and replacement name must differ".to_string());
    }

    let workspace_root = resolve_workspace_root(root.as_deref())?;
    let files = discover_workspace_files(workspace_root.as_path())?;
    let style_sources = read_style_sources(files.style_paths.as_slice())?;
    let index = summarize_omena_query_custom_property_occurrence_index(style_sources.as_slice());
    let sources = style_sources
        .iter()
        .map(|source| (source.style_path.clone(), source.style_source.clone()))
        .collect::<BTreeMap<_, _>>();
    let authority_evidence = MigrationEvidenceV0 {
        id: "custom-property-workspace-occurrences".to_string(),
        kind: "customPropertyOccurrenceIndex".to_string(),
        source: "omena-query".to_string(),
        detail: format!(
            "query-owned parser fact projection indexed {} custom-property occurrences",
            index.occurrence_count
        ),
    };
    let fallback_evidence = MigrationEvidenceV0 {
        id: "custom-property-fallback-references".to_string(),
        kind: "customPropertyValueDependency".to_string(),
        source: "omena-parser-variable-facts".to_string(),
        detail: "var() references with fallback values require explicit review".to_string(),
    };
    let mut drafts = Vec::new();
    let mut blockers = Vec::new();
    for occurrence in index
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.name == token_name)
    {
        let Some(source) = sources.get(occurrence.uri.as_str()) else {
            blockers.push(MigrationBlockerV0 {
                code: "customPropertySourceMissing".to_string(),
                uri: Some(occurrence.uri.clone()),
                detail: "custom-property occurrence source is not in the workspace index"
                    .to_string(),
                evidence_refs: vec![authority_evidence.id.clone()],
            });
            continue;
        };
        let Some(expected_text) = source.get(occurrence.byte_span.start..occurrence.byte_span.end)
        else {
            blockers.push(MigrationBlockerV0 {
                code: "invalidCustomPropertyRange".to_string(),
                uri: Some(occurrence.uri.clone()),
                detail: "custom-property occurrence range is outside UTF-8 source boundaries"
                    .to_string(),
                evidence_refs: vec![authority_evidence.id.clone()],
            });
            continue;
        };
        drafts.push(MigrationEditDraftV0 {
            uri: occurrence.uri.clone(),
            range: occurrence.range,
            byte_span: occurrence.byte_span,
            expected_text: expected_text.to_string(),
            replacement_text: new_name.clone(),
            expected_source_sha256: content_sha256(source.as_bytes()),
            safety_evidence: if occurrence.has_fallback {
                conservative_workspace_safety()
            } else {
                exact_workspace_safety()
            },
            evidence: MigrationEditEvidenceV0 {
                primary: authority_evidence.id.clone(),
                supporting: occurrence
                    .has_fallback
                    .then(|| fallback_evidence.id.clone())
                    .into_iter()
                    .collect(),
            },
        });
    }
    if drafts.is_empty() && blockers.is_empty() {
        blockers.push(MigrationBlockerV0 {
            code: "customPropertyNotFound".to_string(),
            uri: None,
            detail: format!("custom property '{token_name}' was not found in the workspace index"),
            evidence_refs: vec![authority_evidence.id.clone()],
        });
    }

    finalize_migration_plan(
        MigrationCodemodV0::TokenRename,
        workspace_root.as_path(),
        drafts,
        blockers,
        vec![authority_evidence, fallback_evidence],
    )
}

fn build_sass_import_to_use_plan(root: Option<PathBuf>) -> Result<MigrationPlanV0, String> {
    build_sass_import_to_use_plan_with_oracle(root, run_sass_migration_oracle)
}

fn build_sass_import_to_use_plan_with_oracle(
    root: Option<PathBuf>,
    oracle: impl FnOnce(&SassMigrationOracleRequestV0) -> Result<SassMigrationOracleResultV0, String>,
) -> Result<MigrationPlanV0, String> {
    let workspace_root = resolve_workspace_root(root.as_deref())?;
    let files = discover_workspace_files(workspace_root.as_path())?;
    let sass_paths = files
        .style_paths
        .iter()
        .filter(|path| sass_dialect_for_path(path).is_some())
        .cloned()
        .collect::<Vec<_>>();
    let style_sources = read_style_sources(sass_paths.as_slice())?;
    let package_manifests = read_package_manifests(files.package_manifest_paths.as_slice())?;
    let module_resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        style_sources.as_slice(),
        package_manifests.as_slice(),
        &[],
        &[],
    );
    let parser_evidence = MigrationEvidenceV0 {
        id: "sass-module-graph-import-edges".to_string(),
        kind: "sassModuleGraph".to_string(),
        source: "omena-parser-via-omena-query".to_string(),
        detail: "canonical Sass module edges provide import targets, source spans, and media qualification"
            .to_string(),
    };
    let resolution_evidence = MigrationEvidenceV0 {
        id: "sass-module-graph-resolution".to_string(),
        kind: "sassModuleGraphResolution".to_string(),
        source: "omena-query-semantic-substrate".to_string(),
        detail: format!(
            "resolved {} of {} module edges with {} cycles",
            module_resolution.resolved_module_edge_count,
            module_resolution.module_edge_count,
            module_resolution.cycle_count
        ),
    };
    let resolved_imports = module_resolution
        .edges
        .iter()
        .filter(|edge| edge.edge_kind == "sassImport")
        .map(|edge| {
            (
                (edge.from_style_path.as_str(), edge.source.as_str()),
                edge.status,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let cyclic_styles = module_resolution
        .cycles
        .iter()
        .flat_map(|cycle| cycle.path.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut evidence = vec![parser_evidence.clone(), resolution_evidence.clone()];
    let mut blockers = Vec::new();
    let mut drafts = Vec::new();
    let mut statement_spans = BTreeSet::new();

    for style in &style_sources {
        let path = Path::new(style.style_path.as_str());
        let Some(dialect) = sass_dialect_for_path(path) else {
            continue;
        };
        let source_edges =
            summarize_omena_query_sass_module_source_edges(style.style_source.as_str(), dialect);
        for edge in source_edges.iter().filter(|edge| edge.kind == "sassImport") {
            let oracle_evidence_id =
                stable_evidence_id("dart-sass-oracle", style.style_path.as_str());
            if edge.media_qualified || sass_import_is_plain_css(edge.source.as_str()) {
                blockers.push(MigrationBlockerV0 {
                    code: "plainCssImport".to_string(),
                    uri: Some(style.style_path.clone()),
                    detail: format!(
                        "@import target '{}' has CSS import semantics and cannot become @use",
                        edge.source
                    ),
                    evidence_refs: vec![parser_evidence.id.clone()],
                });
                continue;
            }
            let resolution_status = resolved_imports
                .get(&(style.style_path.as_str(), edge.source.as_str()))
                .copied();
            if resolution_status != Some("resolved") {
                blockers.push(MigrationBlockerV0 {
                    code: "sassModuleTargetUnresolved".to_string(),
                    uri: Some(style.style_path.clone()),
                    detail: format!(
                        "@import target '{}' is not resolved by the canonical module graph",
                        edge.source
                    ),
                    evidence_refs: vec![resolution_evidence.id.clone()],
                });
                continue;
            }
            if cyclic_styles.contains(style.style_path.as_str()) {
                blockers.push(MigrationBlockerV0 {
                    code: "sassModuleCycle".to_string(),
                    uri: Some(style.style_path.clone()),
                    detail: "@import participates in a module cycle that @use rejects".to_string(),
                    evidence_refs: vec![resolution_evidence.id.clone()],
                });
                continue;
            }
            let (start, end, replacement_text) = match sass_import_statement_edit(
                style.style_source.as_str(),
                edge.byte_span,
                dialect,
            ) {
                Ok(edit) => edit,
                Err(detail) => {
                    blockers.push(MigrationBlockerV0 {
                        code: "unsupportedSassImportShape".to_string(),
                        uri: Some(style.style_path.clone()),
                        detail,
                        evidence_refs: vec![parser_evidence.id.clone()],
                    });
                    continue;
                }
            };
            if !statement_spans.insert((style.style_path.clone(), start, end)) {
                blockers.push(MigrationBlockerV0 {
                    code: "groupedSassImport".to_string(),
                    uri: Some(style.style_path.clone()),
                    detail: "multiple import targets share one statement and require manual restructuring"
                        .to_string(),
                    evidence_refs: vec![parser_evidence.id.clone()],
                });
                continue;
            }
            let range =
                range_for_byte_span(style.style_source.as_str(), start, end).ok_or_else(|| {
                    "Sass import statement is outside UTF-8 source boundaries".to_string()
                })?;
            drafts.push(MigrationEditDraftV0 {
                uri: style.style_path.clone(),
                range,
                byte_span: ParserByteSpanV0 { start, end },
                expected_text: style.style_source[start..end].to_string(),
                replacement_text,
                expected_source_sha256: content_sha256(style.style_source.as_bytes()),
                safety_evidence: exact_workspace_safety(),
                evidence: MigrationEditEvidenceV0 {
                    primary: parser_evidence.id.clone(),
                    supporting: vec![oracle_evidence_id],
                },
            });
        }
    }

    if drafts.is_empty() {
        if blockers.is_empty() {
            blockers.push(MigrationBlockerV0 {
                code: "sassImportNotFound".to_string(),
                uri: None,
                detail: "no eligible Sass @import statements were found".to_string(),
                evidence_refs: vec![parser_evidence.id.clone()],
            });
        }
    } else {
        let request = SassMigrationOracleRequestV0 {
            schema_version: "0",
            product: "omena-cli.sass-migration-oracle-request",
            workspace_root: path_string(workspace_root.as_path()),
            files: style_sources
                .iter()
                .map(|style| SassMigrationOracleFileV0 {
                    path: style.style_path.clone(),
                    source: style.style_source.clone(),
                })
                .collect(),
            edits: drafts
                .iter()
                .map(|draft| SassMigrationOracleEditV0 {
                    uri: draft.uri.clone(),
                    start: draft.byte_span.start,
                    end: draft.byte_span.end,
                    replacement_text: draft.replacement_text.clone(),
                })
                .collect(),
        };
        let oracle_result = oracle(&request);
        fold_sass_oracle_result(&request, oracle_result, &mut evidence, &mut blockers)?;
    }

    finalize_migration_plan(
        MigrationCodemodV0::SassImportToUse,
        workspace_root.as_path(),
        drafts,
        blockers,
        evidence,
    )
}

fn fold_sass_oracle_result(
    request: &SassMigrationOracleRequestV0,
    result: Result<SassMigrationOracleResultV0, String>,
    evidence: &mut Vec<MigrationEvidenceV0>,
    blockers: &mut Vec<MigrationBlockerV0>,
) -> Result<(), String> {
    let edited_uris = request
        .edits
        .iter()
        .map(|edit| edit.uri.as_str())
        .collect::<BTreeSet<_>>();
    match result {
        Ok(result) => {
            if result.schema_version != "0"
                || result.product != "omena-cli.sass-migration-oracle-result"
                || result.compiler.name != "dart-sass"
                || result.compiler.package != "sass"
                || result.compiler.version != "1.101.0"
            {
                return Err("Sass migration oracle returned an unsupported contract".to_string());
            }
            let by_uri = result
                .results
                .iter()
                .map(|item| (item.uri.as_str(), item))
                .collect::<BTreeMap<_, _>>();
            if by_uri.len() != result.results.len()
                || by_uri.keys().any(|uri| !edited_uris.contains(uri))
                || result.all_matched != result.results.iter().all(|item| item.matched)
            {
                return Err(
                    "Sass migration oracle returned inconsistent result coverage".to_string(),
                );
            }
            for uri in edited_uris {
                let evidence_id = stable_evidence_id("dart-sass-oracle", uri);
                let Some(file_result) = by_uri.get(uri) else {
                    evidence.push(MigrationEvidenceV0 {
                        id: evidence_id.clone(),
                        kind: "dartSassCompileEquivalence".to_string(),
                        source: "dart-sass@1.101.0".to_string(),
                        detail: "oracle result omitted this edited Sass source".to_string(),
                    });
                    blockers.push(MigrationBlockerV0 {
                        code: "sassOracleMissingResult".to_string(),
                        uri: Some(uri.to_string()),
                        detail: "Dart Sass did not return a result for an edited source"
                            .to_string(),
                        evidence_refs: vec![evidence_id],
                    });
                    continue;
                };
                evidence.push(MigrationEvidenceV0 {
                    id: evidence_id.clone(),
                    kind: "dartSassCompileEquivalence".to_string(),
                    source: "dart-sass@1.101.0".to_string(),
                    detail: format!(
                        "beforeStatus={:?}; afterStatus={:?}; beforeCss={:?}; afterCss={:?}; allMatched={}",
                        file_result.before_status,
                        file_result.after_status,
                        file_result.before_css_sha256,
                        file_result.after_css_sha256,
                        result.all_matched
                    ),
                });
                if !file_result.matched {
                    blockers.push(MigrationBlockerV0 {
                        code: "sassOracleMismatch".to_string(),
                        uri: Some(uri.to_string()),
                        detail: format!(
                            "Dart Sass output diverged after migration; before stderr='{}'; after stderr='{}'",
                            compact_oracle_stderr(file_result.before_stderr.as_str()),
                            compact_oracle_stderr(file_result.after_stderr.as_str())
                        ),
                        evidence_refs: vec![evidence_id],
                    });
                }
            }
        }
        Err(detail) => {
            for uri in edited_uris {
                let evidence_id = stable_evidence_id("dart-sass-oracle", uri);
                evidence.push(MigrationEvidenceV0 {
                    id: evidence_id.clone(),
                    kind: "dartSassCompileEquivalence".to_string(),
                    source: "dart-sass@1.101.0".to_string(),
                    detail: format!("oracle unavailable: {detail}"),
                });
                blockers.push(MigrationBlockerV0 {
                    code: "sassOracleUnavailable".to_string(),
                    uri: Some(uri.to_string()),
                    detail: detail.clone(),
                    evidence_refs: vec![evidence_id],
                });
            }
        }
    }
    Ok(())
}

fn run_sass_migration_oracle(
    request: &SassMigrationOracleRequestV0,
) -> Result<SassMigrationOracleResultV0, String> {
    let repo_root = find_repo_root_for_oracle()?;
    let script = repo_root.join("scripts/run-sass-migration-oracle.ts");
    let mut child = Command::new("node")
        .args(["--import", "tsx"])
        .arg(script)
        .current_dir(repo_root.as_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start the Dart Sass oracle: {error}"))?;
    let request_json = serde_json::to_vec(request)
        .map_err(|error| format!("failed to serialize the Sass oracle request: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Dart Sass oracle stdin is unavailable".to_string())?;
    std::io::copy(&mut request_json.as_slice(), &mut stdin)
        .map_err(|error| format!("failed to send the Sass oracle request: {error}"))?;
    drop(stdin);
    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for the Dart Sass oracle: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Dart Sass oracle failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    serde_json::from_slice(output.stdout.as_slice())
        .map_err(|error| format!("failed to parse the Dart Sass oracle result: {error}"))
}

fn find_repo_root_for_oracle() -> Result<PathBuf, String> {
    let current = std::env::current_dir()
        .map_err(|error| format!("failed to resolve the current directory: {error}"))?;
    current
        .ancestors()
        .find(|candidate| {
            candidate
                .join("scripts/run-sass-migration-oracle.ts")
                .is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            "could not locate scripts/run-sass-migration-oracle.ts from the current directory"
                .to_string()
        })
}

fn sass_import_statement_edit(
    source: &str,
    target_span: ParserByteSpanV0,
    dialect: OmenaParserStyleDialect,
) -> Result<(usize, usize, String), String> {
    let source_token = source
        .get(target_span.start..target_span.end)
        .ok_or_else(|| "Sass import target span is outside the source".to_string())?;
    let before = source
        .get(..target_span.start)
        .ok_or_else(|| "Sass import target start is outside the source".to_string())?;
    let lower_before = before.to_ascii_lowercase();
    let start = lower_before
        .rfind("@import")
        .ok_or_else(|| "Sass import edge has no enclosing @import rule".to_string())?;
    if !source[start + "@import".len()..target_span.start]
        .chars()
        .all(char::is_whitespace)
    {
        return Err("Sass import has unsupported tokens before its target".to_string());
    }
    let remainder = &source[target_span.end..];
    let relative_end = match dialect {
        OmenaParserStyleDialect::Scss => remainder
            .find(';')
            .map(|index| index + 1)
            .ok_or_else(|| "SCSS import statement has no terminating semicolon".to_string())?,
        OmenaParserStyleDialect::Sass => remainder.find('\n').unwrap_or(remainder.len()),
        _ => return Err("only SCSS and indented Sass imports can be migrated".to_string()),
    };
    let end = target_span.end + relative_end;
    let suffix = source[target_span.end..end]
        .trim()
        .trim_end_matches(';')
        .trim();
    if !suffix.is_empty() {
        return Err("grouped or qualified Sass imports require manual restructuring".to_string());
    }
    let terminator = if matches!(dialect, OmenaParserStyleDialect::Scss) {
        ";"
    } else {
        ""
    };
    Ok((start, end, format!("@use {source_token} as *{terminator}")))
}

fn sass_import_is_plain_css(source: &str) -> bool {
    let source = source.to_ascii_lowercase();
    source.ends_with(".css")
        || source.starts_with("http://")
        || source.starts_with("https://")
        || source.starts_with("url(")
}

fn sass_dialect_for_path(path: &Path) -> Option<OmenaParserStyleDialect> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("scss") => Some(OmenaParserStyleDialect::Scss),
        Some("sass") => Some(OmenaParserStyleDialect::Sass),
        _ => None,
    }
}

fn stable_evidence_id(prefix: &str, source: &str) -> String {
    let digest = Sha256::digest(source.as_bytes());
    format!("{prefix}-{}", &hex_digest(digest.as_slice())[..16])
}

fn compact_oracle_stderr(stderr: &str) -> String {
    stderr.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn build_css_modules_rename_plan(
    selector_name: Option<String>,
    new_name: Option<String>,
    root: Option<PathBuf>,
    target_style: Option<PathBuf>,
) -> Result<MigrationPlanV0, String> {
    let selector_name = normalize_selector_name(
        selector_name
            .as_deref()
            .ok_or_else(|| "css-modules-rename planning requires SELECTOR_NAME".to_string())?,
    )?;
    let new_name = normalize_selector_name(
        new_name
            .as_deref()
            .ok_or_else(|| "css-modules-rename planning requires NEW_NAME".to_string())?,
    )?;
    if selector_name == new_name {
        return Err("selector name and replacement name must differ".to_string());
    }

    let workspace_root = resolve_workspace_root(root.as_deref())?;
    let files = discover_workspace_files(workspace_root.as_path())?;
    let module_style_paths = files
        .style_paths
        .iter()
        .filter(|path| is_css_module_path(path))
        .cloned()
        .collect::<Vec<_>>();
    let style_sources = read_style_sources(module_style_paths.as_slice())?;
    let source_documents = read_source_documents(files.source_paths.as_slice())?;
    let package_manifests = read_package_manifests(files.package_manifest_paths.as_slice())?;
    let target_style = target_style
        .as_deref()
        .map(|path| resolve_target_path(workspace_root.as_path(), path))
        .transpose()?
        .map(|path| path_string(path.as_path()));

    let rename = summarize_omena_query_rename_plan_for_workspace_class(
        selector_name.as_str(),
        new_name.as_str(),
        target_style.as_deref(),
        style_sources.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
    );
    let references = summarize_omena_query_refs_for_workspace_class(
        selector_name.as_str(),
        target_style.as_deref(),
        true,
        style_sources.as_slice(),
        source_documents.as_slice(),
        package_manifests.as_slice(),
    );
    let sources = workspace_source_map(&style_sources, &source_documents);
    let exact_locations = rename
        .edits
        .iter()
        .map(|edit| (edit.uri.as_str(), edit.range))
        .collect::<BTreeSet<_>>();
    let authority_evidence = MigrationEvidenceV0 {
        id: "css-modules-workspace-occurrences".to_string(),
        kind: "selectorOccurrenceIndex".to_string(),
        source: "omena-query".to_string(),
        detail: format!(
            "query-owned workspace rename selected {} exact edits from {} resolved locations",
            rename.edit_count, references.location_count
        ),
    };
    let dynamic_evidence = MigrationEvidenceV0 {
        id: "css-modules-non-exact-references".to_string(),
        kind: "selectorReferencePrecision".to_string(),
        source: "omena-query".to_string(),
        detail: "resolved prefix or dynamic references are retained for manual review".to_string(),
    };
    let mut drafts = Vec::new();
    let mut blockers = Vec::new();
    for edit in &rename.edits {
        match draft_from_workspace_edit(
            edit,
            &sources,
            exact_workspace_safety(),
            MigrationEditEvidenceV0 {
                primary: authority_evidence.id.clone(),
                supporting: Vec::new(),
            },
        ) {
            Ok(draft) => drafts.push(draft),
            Err(detail) => blockers.push(MigrationBlockerV0 {
                code: "invalidSelectorEditRange".to_string(),
                uri: Some(edit.uri.clone()),
                detail,
                evidence_refs: vec![authority_evidence.id.clone()],
            }),
        }
    }
    for location in references.locations.iter().filter(|location| {
        location.role == "reference"
            && !exact_locations.contains(&(location.uri.as_str(), location.range))
    }) {
        match draft_from_review_location(
            location,
            new_name.as_str(),
            &sources,
            MigrationEditEvidenceV0 {
                primary: dynamic_evidence.id.clone(),
                supporting: vec![authority_evidence.id.clone()],
            },
        ) {
            Ok(draft) => drafts.push(draft),
            Err(detail) => blockers.push(MigrationBlockerV0 {
                code: "invalidDynamicSelectorRange".to_string(),
                uri: Some(location.uri.clone()),
                detail,
                evidence_refs: vec![dynamic_evidence.id.clone()],
            }),
        }
    }
    if drafts.is_empty() && blockers.is_empty() {
        blockers.push(MigrationBlockerV0 {
            code: "selectorNotFound".to_string(),
            uri: target_style.clone(),
            detail: format!("selector '.{selector_name}' was not found in the workspace index"),
            evidence_refs: vec![authority_evidence.id.clone()],
        });
    }

    finalize_migration_plan(
        MigrationCodemodV0::CssModulesRename,
        workspace_root.as_path(),
        drafts,
        blockers,
        vec![authority_evidence, dynamic_evidence],
    )
}

fn draft_from_workspace_edit(
    edit: &OmenaQueryWorkspaceTextEditV0,
    sources: &BTreeMap<String, String>,
    safety_evidence: FixSafetyEvidenceInputV0,
    evidence: MigrationEditEvidenceV0,
) -> Result<MigrationEditDraftV0, String> {
    draft_from_range(
        edit.uri.as_str(),
        edit.range,
        edit.new_text.as_str(),
        sources,
        safety_evidence,
        evidence,
    )
}

fn draft_from_review_location(
    location: &OmenaQueryReferenceLocationV0,
    replacement_text: &str,
    sources: &BTreeMap<String, String>,
    evidence: MigrationEditEvidenceV0,
) -> Result<MigrationEditDraftV0, String> {
    draft_from_range(
        location.uri.as_str(),
        location.range,
        replacement_text,
        sources,
        manual_review_safety(),
        evidence,
    )
}

fn draft_from_range(
    uri: &str,
    range: ParserRangeV0,
    replacement_text: &str,
    sources: &BTreeMap<String, String>,
    safety_evidence: FixSafetyEvidenceInputV0,
    evidence: MigrationEditEvidenceV0,
) -> Result<MigrationEditDraftV0, String> {
    let source = source_for_uri(sources, uri)
        .ok_or_else(|| format!("workspace source {uri} is not indexed"))?;
    let (start, end) = byte_span_for_range(source, range)
        .ok_or_else(|| format!("query range for {uri} is outside the indexed source"))?;
    let expected_text = source
        .get(start..end)
        .ok_or_else(|| format!("query range for {uri} is not on UTF-8 boundaries"))?;
    Ok(MigrationEditDraftV0 {
        uri: uri.to_string(),
        range,
        byte_span: ParserByteSpanV0 { start, end },
        expected_text: expected_text.to_string(),
        replacement_text: replacement_text.to_string(),
        expected_source_sha256: content_sha256(source.as_bytes()),
        safety_evidence,
        evidence,
    })
}

fn workspace_source_map(
    style_sources: &[omena_query::OmenaQueryStyleSourceInputV0],
    source_documents: &[omena_query::OmenaQuerySourceDocumentInputV0],
) -> BTreeMap<String, String> {
    style_sources
        .iter()
        .map(|source| (source.style_path.clone(), source.style_source.clone()))
        .chain(
            source_documents
                .iter()
                .map(|source| (source.source_path.clone(), source.source_source.clone())),
        )
        .collect()
}

fn source_for_uri<'a>(sources: &'a BTreeMap<String, String>, uri: &str) -> Option<&'a str> {
    sources
        .get(uri)
        .or_else(|| {
            cli_file_uri_to_path(uri)
                .as_ref()
                .and_then(|path| sources.get(path_string(path.as_path()).as_str()))
        })
        .map(String::as_str)
}

fn resolve_workspace_root(root: Option<&Path>) -> Result<PathBuf, String> {
    let root = match root {
        Some(root) => root.to_path_buf(),
        None => std::env::current_dir()
            .map_err(|error| format!("failed to resolve the current directory: {error}"))?,
    };
    let canonical = fs::canonicalize(root.as_path()).map_err(|error| {
        format!(
            "failed to resolve workspace root {}: {error}",
            root.display()
        )
    })?;
    if canonical.is_file() {
        canonical
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("workspace entry {} has no parent", canonical.display()))
    } else {
        Ok(canonical)
    }
}

fn resolve_target_path(root: &Path, target: &Path) -> Result<PathBuf, String> {
    let candidate = if target.is_absolute() {
        target.to_path_buf()
    } else {
        root.join(target)
    };
    fs::canonicalize(candidate.as_path()).map_err(|error| {
        format!(
            "failed to resolve target style {}: {error}",
            candidate.display()
        )
    })
}

fn normalize_selector_name(name: &str) -> Result<String, String> {
    let name = name.trim().strip_prefix('.').unwrap_or(name.trim());
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err("selector names must be non-empty and contain no whitespace".to_string());
    }
    Ok(name.to_string())
}

fn normalize_custom_property_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    let normalized = if name.starts_with("--") {
        name.to_string()
    } else {
        format!("--{name}")
    };
    if normalized.len() <= 2 || normalized.chars().any(char::is_whitespace) {
        return Err(
            "custom-property names must be non-empty and contain no whitespace".to_string(),
        );
    }
    Ok(normalized)
}

fn is_css_module_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains(".module."))
}

fn exact_workspace_safety() -> FixSafetyEvidenceInputV0 {
    FixSafetyEvidenceInputV0 {
        syntax_preserving: true,
        local_semantics_required: true,
        local_semantics_ready: true,
        closed_world_required: true,
        closed_world_ready: true,
        reference_precision_required: true,
        reference_precision: Some(FactPrecision::Exact),
    }
}

fn conservative_workspace_safety() -> FixSafetyEvidenceInputV0 {
    FixSafetyEvidenceInputV0 {
        reference_precision: Some(FactPrecision::Conservative),
        ..exact_workspace_safety()
    }
}

fn manual_review_safety() -> FixSafetyEvidenceInputV0 {
    FixSafetyEvidenceInputV0 {
        syntax_preserving: true,
        local_semantics_required: true,
        local_semantics_ready: false,
        closed_world_required: true,
        closed_world_ready: true,
        reference_precision_required: true,
        reference_precision: Some(FactPrecision::Heuristic),
    }
}

fn run_migration_mode(
    codemod: MigrationCodemodV0,
    mode: MigrationModeArgs,
    build_plan: impl FnOnce() -> Result<MigrationPlanV0, String>,
) -> Result<(), String> {
    match (mode.plan, mode.apply) {
        (Some(plan_path), None) => {
            let plan = build_plan()?;
            validate_migration_plan(&plan)?;
            write_json_artifact(plan_path.as_path(), &plan)?;
            if mode.json {
                print_json(CliOutputMetadataV0::new("omena-cli.migrate.plan"), &plan)?;
            } else {
                println!(
                    "wrote {} migration plan with {} edits",
                    codemod.as_str(),
                    plan.edits.len()
                );
            }
            Ok(())
        }
        (None, Some(plan_path)) => {
            let report = apply_migration_plan(codemod, plan_path.as_path(), mode.approve_review)?;
            if mode.json {
                print_json(CliOutputMetadataV0::new("omena-cli.migrate.apply"), &report)?;
            } else {
                println!(
                    "applied {} edits across {} files",
                    report.applied_edit_count, report.applied_file_count
                );
            }
            Ok(())
        }
        _ => Err("exactly one of --plan or --apply is required".to_string()),
    }
}

fn finalize_migration_plan(
    codemod: MigrationCodemodV0,
    workspace_root: &Path,
    drafts: Vec<MigrationEditDraftV0>,
    mut blockers: Vec<MigrationBlockerV0>,
    mut evidence: Vec<MigrationEvidenceV0>,
) -> Result<MigrationPlanV0, String> {
    let mut edits = drafts
        .into_iter()
        .map(|draft| MigrationEditV0 {
            id: migration_edit_id(codemod, &draft),
            uri: draft.uri,
            range: draft.range,
            byte_span: draft.byte_span,
            expected_text: draft.expected_text,
            replacement_text: draft.replacement_text,
            expected_source_sha256: draft.expected_source_sha256,
            safety_evidence: draft.safety_evidence,
            evidence: draft.evidence,
        })
        .collect::<Vec<_>>();
    edits.sort_by(migration_edit_order);
    evidence.sort_by(|left, right| left.id.cmp(&right.id));
    blockers.sort_by(|left, right| {
        (&left.code, &left.uri, &left.detail).cmp(&(&right.code, &right.uri, &right.detail))
    });

    let mut safe_edits = Vec::new();
    let mut review_edits = Vec::new();
    for edit in &edits {
        match compute_fix_safety(edit.safety_evidence).safety {
            FixSafetyV0::Safe => safe_edits.push(edit.id.clone()),
            FixSafetyV0::Conservative | FixSafetyV0::ManualReview => {
                review_edits.push(edit.id.clone());
            }
        }
    }
    safe_edits.sort();
    review_edits.sort();
    let inverse_edits = build_inverse_edits(edits.as_slice());
    let plan = MigrationPlanV0 {
        schema_version: MIGRATION_PLAN_SCHEMA_VERSION.to_string(),
        product: MIGRATION_PLAN_PRODUCT.to_string(),
        codemod,
        workspace_root: path_string(workspace_root),
        edits,
        safe_edits,
        review_edits,
        blockers,
        evidence,
        rollback: MigrationRollbackPlanV0 {
            receipt_typed: false,
            receipt: None,
            inverse_edits,
        },
    };
    validate_migration_plan(&plan)?;
    Ok(plan)
}

fn validate_migration_plan(plan: &MigrationPlanV0) -> Result<(), String> {
    if plan.schema_version != MIGRATION_PLAN_SCHEMA_VERSION
        || plan.product != MIGRATION_PLAN_PRODUCT
    {
        return Err("unsupported migration plan schema or product".to_string());
    }
    if plan.edits.is_empty() && plan.blockers.is_empty() {
        return Err("migration plan contains neither edits nor blockers".to_string());
    }

    let evidence_ids = plan
        .evidence
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    if evidence_ids.len() != plan.evidence.len()
        || plan
            .evidence
            .windows(2)
            .any(|pair| pair[0].id >= pair[1].id)
    {
        return Err("migration evidence ids must be unique and sorted".to_string());
    }

    let mut edit_ids = BTreeSet::new();
    let mut expected_safe = Vec::new();
    let mut expected_review = Vec::new();
    let mut previous_by_uri = BTreeMap::<&str, usize>::new();
    for edit in &plan.edits {
        if !edit_ids.insert(edit.id.as_str()) {
            return Err(format!("duplicate migration edit id {}", edit.id));
        }
        if edit.evidence.primary.is_empty()
            || !evidence_ids.contains(edit.evidence.primary.as_str())
        {
            return Err(format!("edit {} has no valid primary evidence", edit.id));
        }
        if edit
            .evidence
            .supporting
            .iter()
            .any(|reference| !evidence_ids.contains(reference.as_str()))
        {
            return Err(format!(
                "edit {} has an unknown evidence reference",
                edit.id
            ));
        }
        if edit.expected_source_sha256.is_empty()
            || edit.byte_span.start > edit.byte_span.end
            || edit.expected_text.len() != edit.byte_span.end - edit.byte_span.start
        {
            return Err(format!(
                "edit {} has an invalid source precondition",
                edit.id
            ));
        }
        if let Some(previous_end) = previous_by_uri.insert(&edit.uri, edit.byte_span.end)
            && edit.byte_span.start < previous_end
        {
            return Err(format!("edit {} overlaps another edit", edit.id));
        }
        match compute_fix_safety(edit.safety_evidence).safety {
            FixSafetyV0::Safe => expected_safe.push(edit.id.clone()),
            FixSafetyV0::Conservative | FixSafetyV0::ManualReview => {
                expected_review.push(edit.id.clone());
            }
        }
    }
    let mut ordered = plan.edits.clone();
    ordered.sort_by(migration_edit_order);
    if ordered != plan.edits {
        return Err("migration edits must use deterministic source order".to_string());
    }
    expected_safe.sort();
    expected_review.sort();
    if plan.safe_edits != expected_safe || plan.review_edits != expected_review {
        return Err("migration safety partitions do not match FixSafety".to_string());
    }
    for blocker in &plan.blockers {
        if blocker.evidence_refs.is_empty()
            || blocker
                .evidence_refs
                .iter()
                .any(|reference| !evidence_ids.contains(reference.as_str()))
        {
            return Err(format!("blocker {} has no valid evidence", blocker.code));
        }
    }
    if plan.rollback.inverse_edits != build_inverse_edits(plan.edits.as_slice()) {
        return Err(
            "rollback templates must exactly reverse every migration edit in final-source coordinates"
                .to_string(),
        );
    }
    if plan.rollback.receipt_typed || plan.rollback.receipt.is_some() {
        return Err("migration plans cannot contain a pre-issued rollback receipt".to_string());
    }
    Ok(())
}

fn apply_migration_plan(
    expected_codemod: MigrationCodemodV0,
    plan_path: &Path,
    approve_review: bool,
) -> Result<MigrationApplyReportV0, String> {
    let source = fs::read_to_string(plan_path)
        .map_err(|error| format!("failed to read {}: {error}", plan_path.display()))?;
    let plan: MigrationPlanV0 = serde_json::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", plan_path.display()))?;
    validate_migration_plan(&plan)?;
    if plan.codemod != expected_codemod {
        return Err(format!(
            "plan codemod {} does not match requested {}",
            plan.codemod.as_str(),
            expected_codemod.as_str()
        ));
    }
    if !plan.blockers.is_empty() {
        return Err(format!(
            "migration plan has {} blocking findings",
            plan.blockers.len()
        ));
    }
    if !plan.review_edits.is_empty() && !approve_review {
        return Err(format!(
            "migration plan has {} review edits; inspect it and pass --approve-review to allow conservative edits",
            plan.review_edits.len()
        ));
    }

    let assessments = plan
        .edits
        .iter()
        .map(|edit| (edit.id.as_str(), compute_fix_safety(edit.safety_evidence)))
        .collect::<BTreeMap<_, _>>();
    if let Some(edit) = plan.edits.iter().find(|edit| {
        assessments
            .get(edit.id.as_str())
            .is_some_and(|assessment| assessment.safety == FixSafetyV0::ManualReview)
    }) {
        return Err(format!(
            "edit {} remains manual-review-only under the shared write-safety policy",
            edit.id
        ));
    }

    let prepared = prepare_migration_writes(&plan, &assessments)?;
    let mode = if approve_review {
        SourceWriteModeV0::AllowConservative
    } else {
        SourceWriteModeV0::SafeOnly
    };
    let mut write_reports = Vec::new();
    let mut files = Vec::new();
    let mut applied_edit_count = 0;
    for write in prepared {
        let report = apply_write_with_safety(
            write.path.as_path(),
            write.content.as_bytes(),
            &write.assessment,
            mode,
            SourceWriteEvidenceV0::MigrationPlan {
                reviewed: approve_review || plan.review_edits.is_empty(),
            },
        )
        .map_err(|error| error.to_string())?;
        applied_edit_count += write.edit_ids.len();
        files.push(MigrationApplyFileV0 {
            path: path_string(write.path.as_path()),
            input_content_signature: write.input_content_signature,
            output_content_signature: write.output_content_signature,
            edit_ids: write.edit_ids,
        });
        write_reports.push(report);
    }

    let receipt = source_rollback_receipt(plan.codemod, plan.edits.as_slice());
    if !receipt.covers_inverse_patch(
        plan.rollback.inverse_edits.len(),
        migration_input_content_signature(&plan.edits).as_str(),
    ) {
        return Err("migration apply receipt does not cover the inverse patch".to_string());
    }
    let rollback = MigrationRollbackPlanV0 {
        receipt_typed: true,
        receipt: Some(receipt),
        inverse_edits: plan.rollback.inverse_edits,
    };

    Ok(MigrationApplyReportV0 {
        schema_version: "0",
        product: "omena-cli.migration-apply-report",
        codemod: plan.codemod,
        plan_path: path_string(plan_path),
        approve_review,
        applied_edit_count,
        applied_file_count: files.len(),
        files,
        write_reports,
        rollback,
    })
}

fn prepare_migration_writes(
    plan: &MigrationPlanV0,
    assessments: &BTreeMap<&str, FixSafetyAssessmentV0>,
) -> Result<Vec<PreparedMigrationWriteV0>, String> {
    let mut edits_by_uri = BTreeMap::<&str, Vec<&MigrationEditV0>>::new();
    for edit in &plan.edits {
        edits_by_uri
            .entry(edit.uri.as_str())
            .or_default()
            .push(edit);
    }

    let mut prepared = Vec::new();
    for (uri, mut edits) in edits_by_uri {
        let path = cli_file_uri_to_path(uri).unwrap_or_else(|| PathBuf::from(uri));
        let source = fs::read_to_string(path.as_path()).map_err(|error| {
            format!(
                "failed to read migration target {}: {error}",
                path.display()
            )
        })?;
        let source_signature = content_sha256(source.as_bytes());
        if edits
            .iter()
            .any(|edit| edit.expected_source_sha256 != source_signature)
        {
            return Err(format!(
                "migration target {} changed after the plan was created",
                path.display()
            ));
        }
        edits.sort_by_key(|edit| std::cmp::Reverse(edit.byte_span.start));
        let mut content = source;
        for edit in &edits {
            let actual = content
                .get(edit.byte_span.start..edit.byte_span.end)
                .ok_or_else(|| format!("edit {} is outside {}", edit.id, path.display()))?;
            if actual != edit.expected_text {
                return Err(format!(
                    "edit {} precondition no longer matches {}",
                    edit.id,
                    path.display()
                ));
            }
            content = apply_byte_edit(
                content.as_str(),
                edit.byte_span.start,
                edit.byte_span.end,
                edit.replacement_text.as_str(),
            )?;
        }
        let assessment = edits
            .iter()
            .filter_map(|edit| assessments.get(edit.id.as_str()))
            .max_by_key(|assessment| assessment.safety)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "migration target {} has no safety assessment",
                    path.display()
                )
            })?;
        let edit_ids = edits.iter().map(|edit| edit.id.clone()).collect::<Vec<_>>();
        prepared.push(PreparedMigrationWriteV0 {
            path,
            output_content_signature: content_sha256(content.as_bytes()),
            content,
            assessment,
            input_content_signature: source_signature,
            edit_ids,
        });
    }
    Ok(prepared)
}

fn migration_edit_id(codemod: MigrationCodemodV0, draft: &MigrationEditDraftV0) -> String {
    let mut hasher = Sha256::new();
    hasher.update(codemod.as_str().as_bytes());
    hasher.update([0]);
    hasher.update(draft.uri.as_bytes());
    hasher.update([0]);
    hasher.update(draft.byte_span.start.to_string().as_bytes());
    hasher.update([0]);
    hasher.update(draft.byte_span.end.to_string().as_bytes());
    hasher.update([0]);
    hasher.update(draft.replacement_text.as_bytes());
    format!("edit-{}", hex_digest(hasher.finalize().as_slice()))
}

fn build_inverse_edits(edits: &[MigrationEditV0]) -> Vec<MigrationInverseEditV0> {
    let mut edits_by_uri = BTreeMap::<&str, Vec<&MigrationEditV0>>::new();
    for edit in edits {
        edits_by_uri
            .entry(edit.uri.as_str())
            .or_default()
            .push(edit);
    }

    let mut inverse_edits = Vec::with_capacity(edits.len());
    for (_, mut source_edits) in edits_by_uri {
        source_edits.sort_by_key(|edit| edit.byte_span.start);
        let mut cumulative_delta = 0_i64;
        for edit in source_edits {
            let final_start = (edit.byte_span.start as i64 + cumulative_delta) as usize;
            inverse_edits.push(MigrationInverseEditV0 {
                edit_id: edit.id.clone(),
                uri: edit.uri.clone(),
                byte_span: ParserByteSpanV0 {
                    start: final_start,
                    end: final_start + edit.replacement_text.len(),
                },
                expected_text: edit.replacement_text.clone(),
                replacement_text: edit.expected_text.clone(),
            });
            cumulative_delta +=
                edit.replacement_text.len() as i64 - edit.expected_text.len() as i64;
        }
    }
    inverse_edits.sort_by(|left, right| {
        (
            &left.uri,
            left.byte_span.start,
            left.byte_span.end,
            &left.edit_id,
        )
            .cmp(&(
                &right.uri,
                right.byte_span.start,
                right.byte_span.end,
                &right.edit_id,
            ))
    });
    inverse_edits
}

fn source_rollback_receipt(
    codemod: MigrationCodemodV0,
    edits: &[MigrationEditV0],
) -> OmenaQueryRollbackReceiptV0 {
    OmenaQueryRollbackReceiptV0 {
        pass_id: format!("source.migration.{}", codemod.as_str()),
        attempted_mutation_count: Some(edits.len()),
        input_content_signature: migration_input_content_signature(edits),
        output_preserved_content_signature: None,
        restorable: OmenaQueryRollbackScopeV0::InversePatch,
    }
}

fn migration_input_content_signature(edits: &[MigrationEditV0]) -> String {
    let input_content_signatures = edits
        .iter()
        .map(|edit| {
            format!(
                "{}#{}",
                edit.uri.as_str(),
                edit.expected_source_sha256.as_str()
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    content_sha256(input_content_signatures.join("\n").as_bytes())
}

fn migration_edit_order(left: &MigrationEditV0, right: &MigrationEditV0) -> std::cmp::Ordering {
    (
        &left.uri,
        left.byte_span.start,
        left.byte_span.end,
        &left.id,
    )
        .cmp(&(
            &right.uri,
            right.byte_span.start,
            right.byte_span.end,
            &right.id,
        ))
}

fn content_sha256(content: &[u8]) -> String {
    format!("sha256:{}", hex_digest(Sha256::digest(content).as_slice()))
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        },
    )
}

#[cfg(test)]
mod tests;

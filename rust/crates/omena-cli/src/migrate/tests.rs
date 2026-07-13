use super::*;
use omena_query::ParserPositionV0;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn migration_plan_is_deterministic_and_evidence_bound() -> Result<(), String> {
    let root = fixture_directory("deterministic-plan");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let source_path = root.join("button.module.css");
    let source = ".old { color: red; }\n";
    fs::write(&source_path, source).map_err(|error| error.to_string())?;
    let plan = fixture_plan(&root, &source_path, source, FixSafetyV0::Safe)?;
    let first = serde_json::to_vec_pretty(&plan).map_err(|error| error.to_string())?;
    let second = serde_json::to_vec_pretty(&plan).map_err(|error| error.to_string())?;
    assert_eq!(first, second);
    assert_eq!(plan.safe_edits.len(), 1);
    assert!(plan.review_edits.is_empty());
    assert!(!plan.rollback.receipt_typed);
    assert!(plan.rollback.receipt.is_none());
    validate_migration_plan(&plan)?;

    let mut forged_receipt = plan.clone();
    forged_receipt.rollback.receipt_typed = true;
    assert!(
        validate_migration_plan(&forged_receipt)
            .err()
            .is_some_and(|error| error.contains("pre-issued"))
    );

    let mut invalid = plan;
    invalid.edits[0].evidence.primary.clear();
    assert!(validate_migration_plan(&invalid).is_err());
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn writing_a_plan_does_not_mutate_sources() -> Result<(), String> {
    let root = fixture_directory("plan-only");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let source_path = root.join("button.module.css");
    let plan_path = root.join("migration.json");
    let source = ".old { color: red; }\n";
    fs::write(&source_path, source).map_err(|error| error.to_string())?;
    let plan = fixture_plan(&root, &source_path, source, FixSafetyV0::Safe)?;
    write_json_artifact(&plan_path, &plan)?;
    assert_eq!(
        fs::read_to_string(&source_path).map_err(|error| error.to_string())?,
        source
    );
    assert!(plan_path.is_file());
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn review_edits_require_approval_before_any_write() -> Result<(), String> {
    let root = fixture_directory("review-approval");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let source_path = root.join("button.module.css");
    let plan_path = root.join("migration.json");
    let source = ".old { color: red; }\n";
    fs::write(&source_path, source).map_err(|error| error.to_string())?;
    let plan = fixture_plan(&root, &source_path, source, FixSafetyV0::Conservative)?;
    write_json_artifact(&plan_path, &plan)?;
    let error = apply_migration_plan(MigrationCodemodV0::CssModulesRename, &plan_path, false)
        .err()
        .ok_or_else(|| "review edit unexpectedly applied without approval".to_string())?;
    assert!(error.contains("review edits"));
    assert_eq!(
        fs::read_to_string(&source_path).map_err(|error| error.to_string())?,
        source
    );
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn approved_conservative_plan_applies_through_the_shared_gate() -> Result<(), String> {
    let root = fixture_directory("approved-plan");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let source_path = root.join("button.module.css");
    let plan_path = root.join("migration.json");
    let source = ".old { color: red; }\n";
    fs::write(&source_path, source).map_err(|error| error.to_string())?;
    let plan = fixture_plan(&root, &source_path, source, FixSafetyV0::Conservative)?;
    write_json_artifact(&plan_path, &plan)?;
    let report = apply_migration_plan(MigrationCodemodV0::CssModulesRename, &plan_path, true)?;
    assert_eq!(report.applied_edit_count, 1);
    assert_eq!(report.write_reports.len(), 1);
    assert!(report.rollback.receipt_typed);
    assert!(
        report.rollback.receipt.as_ref().is_some_and(|receipt| {
            receipt.restorable == OmenaQueryRollbackScopeV0::InversePatch
        })
    );
    assert_eq!(
        fs::read_to_string(&source_path).map_err(|error| error.to_string())?,
        ".new { color: red; }\n"
    );
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn css_modules_rename_uses_exact_and_dynamic_workspace_occurrences() -> Result<(), String> {
    let root = fixture_directory("css-modules-rename");
    fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    let style_path = root.join("src/Button.module.css");
    let source_path = root.join("src/Button.tsx");
    fs::write(
        &style_path,
        ".button-primary { color: red; }\n.button-secondary { color: blue; }\n",
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &source_path,
        concat!(
            "import classNames from \"classnames/bind\";\n",
            "import styles from \"./Button.module.css\";\n",
            "const cx = classNames.bind(styles);\n",
            "const exact = cx(\"button-primary\");\n",
            "const dynamic = cx(`button-${variant}`);\n",
        ),
    )
    .map_err(|error| error.to_string())?;

    let plan = build_css_modules_rename_plan(
        Some("button-primary".to_string()),
        Some("control-primary".to_string()),
        Some(root.clone()),
        Some(style_path.clone()),
    )?;
    assert_eq!(plan.codemod, MigrationCodemodV0::CssModulesRename);
    assert!(plan.blockers.is_empty());
    assert_eq!(plan.safe_edits.len(), 2);
    assert_eq!(
        plan.edits
            .iter()
            .filter(|edit| plan.safe_edits.contains(&edit.id))
            .map(|edit| edit.expected_text.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["button-primary"])
    );
    assert_eq!(plan.review_edits.len(), 1);
    let review = plan
        .edits
        .iter()
        .find(|edit| plan.review_edits.contains(&edit.id))
        .ok_or_else(|| "dynamic occurrence was not retained".to_string())?;
    assert_eq!(review.expected_text, "button-");
    assert_eq!(
        compute_fix_safety(review.safety_evidence).safety,
        FixSafetyV0::ManualReview
    );
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn sass_import_migration_requires_matching_oracle_evidence() -> Result<(), String> {
    let root = fixture_directory("sass-import-oracle");
    fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    fs::write(root.join("src/_tokens.scss"), "$tone: red;\n").map_err(|error| error.to_string())?;
    let entry_path = root.join("src/entry.scss");
    fs::write(
        &entry_path,
        "@import \"tokens\";\n.card { color: $tone; }\n",
    )
    .map_err(|error| error.to_string())?;

    let matched =
        build_sass_import_to_use_plan_with_oracle(Some(root.clone()), matching_sass_oracle_result)?;
    assert!(matched.blockers.is_empty());
    assert_eq!(matched.safe_edits.len(), 1);
    assert_eq!(matched.edits[0].expected_text, "@import \"tokens\";");
    assert_eq!(matched.edits[0].replacement_text, "@use \"tokens\" as *;");

    let diverged = build_sass_import_to_use_plan_with_oracle(Some(root.clone()), |request| {
        Ok(sass_oracle_result(request, false))
    })?;
    assert!(
        diverged
            .blockers
            .iter()
            .any(|blocker| blocker.code == "sassOracleMismatch")
    );

    let inconsistent = build_sass_import_to_use_plan_with_oracle(Some(root.clone()), |request| {
        let mut result = sass_oracle_result(request, true);
        result.all_matched = false;
        Ok(result)
    });
    assert!(
        inconsistent
            .err()
            .is_some_and(|error| error.contains("inconsistent result coverage"))
    );
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn media_qualified_sass_imports_are_blocked_before_oracle_execution() -> Result<(), String> {
    let root = fixture_directory("sass-media-import");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    fs::write(root.join("entry.scss"), "@import \"print\" print;\n")
        .map_err(|error| error.to_string())?;
    let plan = build_sass_import_to_use_plan_with_oracle(Some(root.clone()), |_| {
        Err("the oracle must not run for plain CSS imports".to_string())
    })?;
    assert!(plan.edits.is_empty());
    assert!(
        plan.blockers
            .iter()
            .any(|blocker| blocker.code == "plainCssImport")
    );
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn token_rename_indexes_registrations_declarations_and_fallback_references() -> Result<(), String> {
    let root = fixture_directory("custom-property-rename");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    fs::write(
        root.join("tokens.css"),
        concat!(
            "@property --brand { syntax: \"<color>\"; inherits: true; initial-value: red; }\n",
            ":root { --brand: red; }\n",
            ".plain { color: var(--brand); }\n",
            ".fallback { color: var(--brand, blue); }\n",
        ),
    )
    .map_err(|error| error.to_string())?;
    let plan = build_token_rename_plan(
        Some("brand".to_string()),
        Some("accent".to_string()),
        Some(root.clone()),
    )?;
    assert!(plan.blockers.is_empty());
    assert_eq!(plan.edits.len(), 4);
    assert_eq!(plan.safe_edits.len(), 3);
    assert_eq!(plan.review_edits.len(), 1);
    let review = plan
        .edits
        .iter()
        .find(|edit| plan.review_edits.contains(&edit.id))
        .ok_or_else(|| "fallback reference was not retained for review".to_string())?;
    assert_eq!(review.expected_text, "--brand");
    assert_eq!(review.replacement_text, "--accent");
    assert_eq!(
        compute_fix_safety(review.safety_evidence).safety,
        FixSafetyV0::Conservative
    );
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

fn matching_sass_oracle_result(
    request: &SassMigrationOracleRequestV0,
) -> Result<SassMigrationOracleResultV0, String> {
    Ok(sass_oracle_result(request, true))
}

fn sass_oracle_result(
    request: &SassMigrationOracleRequestV0,
    matched: bool,
) -> SassMigrationOracleResultV0 {
    let results = request
        .edits
        .iter()
        .map(|edit| SassMigrationOracleFileResultV0 {
            uri: edit.uri.clone(),
            matched,
            before_status: Some(0),
            after_status: Some(0),
            before_css_sha256: Some("before".to_string()),
            after_css_sha256: Some(if matched { "before" } else { "after" }.to_string()),
            before_stderr: String::new(),
            after_stderr: String::new(),
        })
        .collect();
    SassMigrationOracleResultV0 {
        schema_version: "0".to_string(),
        product: "omena-cli.sass-migration-oracle-result".to_string(),
        compiler: SassMigrationOracleCompilerV0 {
            name: "dart-sass".to_string(),
            package: "sass".to_string(),
            version: "1.101.0".to_string(),
        },
        all_matched: matched,
        results,
    }
}

#[test]
fn inverse_patches_use_final_source_coordinates() -> Result<(), String> {
    let root = fixture_directory("inverse-patch-coordinates");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let source_path = root.join("selectors.module.css");
    let source = ".a {} .long {}\n";
    fs::write(&source_path, source).map_err(|error| error.to_string())?;
    let evidence = MigrationEvidenceV0 {
        id: "selector-occurrences".to_string(),
        kind: "selectorOccurrenceIndex".to_string(),
        source: "omena-query".to_string(),
        detail: "exact selector occurrences".to_string(),
    };
    let drafts = [(1, 2, "a", "expanded"), (7, 11, "long", "x")]
        .into_iter()
        .map(|(start, end, expected_text, replacement_text)| {
            Ok(MigrationEditDraftV0 {
                uri: source_path.to_string_lossy().into_owned(),
                range: range_for_byte_span(source, start, end)
                    .ok_or_else(|| "fixture range is invalid".to_string())?,
                byte_span: ParserByteSpanV0 { start, end },
                expected_text: expected_text.to_string(),
                replacement_text: replacement_text.to_string(),
                expected_source_sha256: content_sha256(source.as_bytes()),
                safety_evidence: safety_input(FixSafetyV0::Safe),
                evidence: MigrationEditEvidenceV0 {
                    primary: evidence.id.clone(),
                    supporting: Vec::new(),
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let plan = finalize_migration_plan(
        MigrationCodemodV0::CssModulesRename,
        &root,
        drafts,
        Vec::new(),
        vec![evidence],
    )?;
    let mut final_source = source.to_string();
    for edit in plan.edits.iter().rev() {
        final_source = apply_byte_edit(
            final_source.as_str(),
            edit.byte_span.start,
            edit.byte_span.end,
            edit.replacement_text.as_str(),
        )?;
    }
    for inverse in plan.rollback.inverse_edits.iter().rev() {
        assert_eq!(
            final_source.get(inverse.byte_span.start..inverse.byte_span.end),
            Some(inverse.expected_text.as_str())
        );
        final_source = apply_byte_edit(
            final_source.as_str(),
            inverse.byte_span.start,
            inverse.byte_span.end,
            inverse.replacement_text.as_str(),
        )?;
    }
    assert_eq!(final_source, source);
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

fn fixture_plan(
    root: &Path,
    source_path: &Path,
    source: &str,
    safety: FixSafetyV0,
) -> Result<MigrationPlanV0, String> {
    let evidence = MigrationEvidenceV0 {
        id: "evidence-selector-definition".to_string(),
        kind: "selectorDefinition".to_string(),
        source: "omena-query".to_string(),
        detail: "exact selector definition".to_string(),
    };
    finalize_migration_plan(
        MigrationCodemodV0::CssModulesRename,
        root,
        vec![MigrationEditDraftV0 {
            uri: source_path.to_string_lossy().into_owned(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 0,
                    character: 1,
                },
                end: ParserPositionV0 {
                    line: 0,
                    character: 4,
                },
            },
            byte_span: ParserByteSpanV0 { start: 1, end: 4 },
            expected_text: "old".to_string(),
            replacement_text: "new".to_string(),
            expected_source_sha256: content_sha256(source.as_bytes()),
            safety_evidence: safety_input(safety),
            evidence: MigrationEditEvidenceV0 {
                primary: evidence.id.clone(),
                supporting: Vec::new(),
            },
        }],
        Vec::new(),
        vec![evidence],
    )
}

fn safety_input(safety: FixSafetyV0) -> FixSafetyEvidenceInputV0 {
    match safety {
        FixSafetyV0::Safe => FixSafetyEvidenceInputV0 {
            syntax_preserving: true,
            local_semantics_required: false,
            local_semantics_ready: false,
            closed_world_required: false,
            closed_world_ready: false,
            reference_precision_required: false,
            reference_precision: None,
        },
        FixSafetyV0::Conservative => FixSafetyEvidenceInputV0 {
            syntax_preserving: true,
            local_semantics_required: false,
            local_semantics_ready: false,
            closed_world_required: false,
            closed_world_ready: false,
            reference_precision_required: true,
            reference_precision: Some(omena_query::FactPrecision::Conservative),
        },
        FixSafetyV0::ManualReview => FixSafetyEvidenceInputV0 {
            syntax_preserving: false,
            local_semantics_required: false,
            local_semantics_ready: false,
            closed_world_required: false,
            closed_world_ready: false,
            reference_precision_required: false,
            reference_precision: None,
        },
    }
}

fn fixture_directory(label: &str) -> PathBuf {
    let serial = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    std::env::temp_dir().join(format!(
        "omena-cli-migrate-{label}-{}-{nanos}-{serial}",
        std::process::id()
    ))
}

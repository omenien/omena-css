use super::plan::{
    content_sha256, migration_input_content_signature, source_rollback_receipt,
    validate_migration_plan,
};
use super::{BTreeMap, FixSafetyAssessmentV0, FixSafetyV0, Path, PathBuf};
use super::{
    MigrationApplyFileV0, MigrationApplyReportV0, MigrationCodemodV0, MigrationEditV0,
    MigrationPlanV0, MigrationRollbackPlanV0, PreparedMigrationWriteV0,
};
use super::{
    SourceWriteEvidenceV0, SourceWriteModeV0, apply_byte_edit, apply_write_with_safety,
    cli_file_uri_to_path, compute_fix_safety, fs, path_string,
};

pub(super) fn apply_migration_plan(
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

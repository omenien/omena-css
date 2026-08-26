use super::*;

pub(super) fn finalize_migration_plan(
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

pub(super) fn validate_migration_plan(plan: &MigrationPlanV0) -> Result<(), String> {
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

pub(super) fn build_inverse_edits(edits: &[MigrationEditV0]) -> Vec<MigrationInverseEditV0> {
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

pub(super) fn source_rollback_receipt(
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

pub(super) fn migration_input_content_signature(edits: &[MigrationEditV0]) -> String {
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

pub(super) fn content_sha256(content: &[u8]) -> String {
    format!("sha256:{}", hex_digest(Sha256::digest(content).as_slice()))
}

pub(super) fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        },
    )
}

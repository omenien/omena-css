use std::{fs, path::PathBuf};

use omena_checker::{
    FixSafetyAssessmentV0, FixSafetyEvidenceInputV0, FixSafetyV0, compute_fix_safety,
};
use omena_query::ParserRangeV0;
use serde::Serialize;

use crate::{
    paths::{cli_file_uri_to_path, path_string},
    write_safety::{
        SourceWriteErrorV0, SourceWriteEvidenceV0, SourceWriteModeV0, SourceWriteRejectionV0,
        apply_write_with_safety,
    },
};

#[derive(Debug, Clone)]
pub(super) struct LintFixCandidateV0 {
    rule_id: String,
    output_path: PathBuf,
    range: ParserRangeV0,
    new_text: String,
    assessment: FixSafetyAssessmentV0,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LintFixSuggestionV0 {
    rule_id: String,
    output_path: String,
    range: ParserRangeV0,
    new_text: String,
    safety: FixSafetyV0,
    precision_backed: bool,
    rationale: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LintWriteStatusV0 {
    pub(super) requested: bool,
    candidate_edit_count: usize,
    safe_edit_count: usize,
    conservative_edit_count: usize,
    manual_review_edit_count: usize,
    pub(super) applied_edit_count: usize,
    rejection_count: usize,
    pub(super) status: &'static str,
    owner: &'static str,
    suggestions: Vec<LintFixSuggestionV0>,
    rejections: Vec<SourceWriteRejectionV0>,
}

pub(super) fn lint_fix_candidate(
    rule_id: &str,
    output_uri: &str,
    range: ParserRangeV0,
    new_text: &str,
) -> LintFixCandidateV0 {
    let output_path = cli_file_uri_to_path(output_uri).unwrap_or_else(|| PathBuf::from(output_uri));
    let assessment = compute_fix_safety(FixSafetyEvidenceInputV0 {
        syntax_preserving: true,
        local_semantics_required: true,
        local_semantics_ready: false,
        closed_world_required: true,
        closed_world_ready: false,
        reference_precision_required: true,
        reference_precision: None,
    });
    LintFixCandidateV0 {
        rule_id: rule_id.to_string(),
        output_path,
        range,
        new_text: new_text.to_string(),
        assessment,
    }
}

pub(super) fn apply_lint_fix_requests(
    candidates: &[LintFixCandidateV0],
    write: bool,
) -> Result<LintWriteStatusV0, String> {
    let suggestions = candidates
        .iter()
        .map(|candidate| LintFixSuggestionV0 {
            rule_id: candidate.rule_id.clone(),
            output_path: path_string(candidate.output_path.as_path()),
            range: candidate.range,
            new_text: candidate.new_text.clone(),
            safety: candidate.assessment.safety,
            precision_backed: candidate.assessment.precision_backed,
            rationale: candidate.assessment.rationale.clone(),
        })
        .collect::<Vec<_>>();
    let mut applied_edit_count = 0;
    let mut rejections = Vec::new();
    if write {
        for candidate in candidates {
            let source = fs::read_to_string(candidate.output_path.as_path()).map_err(|error| {
                format!(
                    "failed to read lint fix target {}: {error}",
                    path_string(candidate.output_path.as_path())
                )
            })?;
            let edited = apply_text_edit(
                source.as_str(),
                candidate.range,
                candidate.new_text.as_str(),
            )?;
            match apply_write_with_safety(
                candidate.output_path.as_path(),
                edited.as_bytes(),
                &candidate.assessment,
                SourceWriteModeV0::SafeOnly,
                SourceWriteEvidenceV0::LintFix,
            ) {
                Ok(_) => applied_edit_count += 1,
                Err(SourceWriteErrorV0::Rejected(rejection)) => rejections.push(rejection),
                Err(error) => return Err(error.to_string()),
            }
        }
    }
    let safe_edit_count = count_fix_safety(candidates, FixSafetyV0::Safe);
    let conservative_edit_count = count_fix_safety(candidates, FixSafetyV0::Conservative);
    let manual_review_edit_count = count_fix_safety(candidates, FixSafetyV0::ManualReview);
    let status = if applied_edit_count > 0 {
        "appliedSafeEdits"
    } else if write && !rejections.is_empty() {
        "rejectedByFixSafety"
    } else if candidates.is_empty() {
        "waitingForRuleLinkedSourceEdit"
    } else {
        "manualReviewOnly"
    };
    Ok(LintWriteStatusV0 {
        requested: write,
        candidate_edit_count: candidates.len(),
        safe_edit_count,
        conservative_edit_count,
        manual_review_edit_count,
        applied_edit_count,
        rejection_count: rejections.len(),
        status,
        owner: "omena lint",
        suggestions,
        rejections,
    })
}

fn count_fix_safety(candidates: &[LintFixCandidateV0], safety: FixSafetyV0) -> usize {
    candidates
        .iter()
        .filter(|candidate| candidate.assessment.safety == safety)
        .count()
}

fn apply_text_edit(source: &str, range: ParserRangeV0, new_text: &str) -> Result<String, String> {
    let start = byte_offset_for_position(source, range.start.line, range.start.character)
        .ok_or_else(|| "lint fix start position is outside the target source".to_string())?;
    let end = byte_offset_for_position(source, range.end.line, range.end.character)
        .ok_or_else(|| "lint fix end position is outside the target source".to_string())?;
    if start > end {
        return Err("lint fix range is reversed".to_string());
    }
    let mut edited = String::with_capacity(source.len() + new_text.len());
    edited.push_str(&source[..start]);
    edited.push_str(new_text);
    edited.push_str(&source[end..]);
    Ok(edited)
}

fn byte_offset_for_position(
    source: &str,
    target_line: usize,
    target_character: usize,
) -> Option<usize> {
    let mut line = 0;
    let mut line_start = 0;
    for (offset, character) in source.char_indices() {
        if line == target_line {
            line_start = offset;
            break;
        }
        if character == '\n' {
            line += 1;
            line_start = offset + character.len_utf8();
        }
    }
    if line != target_line {
        if target_line == line && line_start == source.len() {
            return (target_character == 0).then_some(source.len());
        }
        return None;
    }
    let line_source = source[line_start..]
        .split_once('\n')
        .map_or(&source[line_start..], |(line_source, _)| line_source);
    let mut utf16_offset = 0;
    for (byte_offset, character) in line_source.char_indices() {
        if utf16_offset == target_character {
            return Some(line_start + byte_offset);
        }
        utf16_offset += character.len_utf16();
        if utf16_offset > target_character {
            return None;
        }
    }
    (utf16_offset == target_character).then_some(line_start + line_source.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_query::ParserPositionV0;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn manual_review_lint_fix_is_reported_and_rejected_without_writing() -> Result<(), String> {
        let path = fixture_path("manual-review.css");
        fs::write(&path, ".known {}\n").map_err(|error| error.to_string())?;
        let candidate = lint_fix_candidate(
            "missing-static-class",
            path_string(path.as_path()).as_str(),
            end_range(),
            ".missing {}\n",
        );
        let preview = apply_lint_fix_requests(std::slice::from_ref(&candidate), false)?;
        assert_eq!(preview.manual_review_edit_count, 1);
        assert_eq!(preview.status, "manualReviewOnly");

        let denied = apply_lint_fix_requests(std::slice::from_ref(&candidate), true)?;
        assert_eq!(denied.applied_edit_count, 0);
        assert_eq!(denied.rejection_count, 1);
        assert_eq!(denied.status, "rejectedByFixSafety");
        assert_eq!(
            fs::read_to_string(&path).map_err(|error| error.to_string())?,
            ".known {}\n"
        );
        fs::remove_file(path).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn conservative_lint_fix_requires_an_explicit_write_mode() -> Result<(), String> {
        let path = fixture_path("conservative.css");
        fs::write(&path, ".known {}\n").map_err(|error| error.to_string())?;
        let mut candidate = lint_fix_candidate(
            "missing-static-class",
            path_string(path.as_path()).as_str(),
            end_range(),
            ".missing {}\n",
        );
        candidate.assessment = compute_fix_safety(FixSafetyEvidenceInputV0 {
            syntax_preserving: true,
            local_semantics_required: false,
            local_semantics_ready: false,
            closed_world_required: false,
            closed_world_ready: false,
            reference_precision_required: true,
            reference_precision: Some(omena_query::FactPrecision::Conservative),
        });
        let denied = apply_lint_fix_requests(&[candidate], true)?;
        assert_eq!(denied.conservative_edit_count, 1);
        assert_eq!(denied.applied_edit_count, 0);
        assert_eq!(denied.rejection_count, 1);
        assert_eq!(
            fs::read_to_string(&path).map_err(|error| error.to_string())?,
            ".known {}\n"
        );
        fs::remove_file(path).map_err(|error| error.to_string())?;
        Ok(())
    }

    #[test]
    fn utf16_text_edit_positions_preserve_non_ascii_prefixes() -> Result<(), String> {
        let source = "/* 🍊 */\n.known {}\n";
        let range = ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 9,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 9,
            },
        };
        assert_eq!(
            apply_text_edit(source, range, "\n.missing {}")?,
            "/* 🍊 */\n.known {}\n.missing {}\n"
        );
        Ok(())
    }

    fn end_range() -> ParserRangeV0 {
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 0,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 0,
            },
        }
    }

    fn fixture_path(label: &str) -> PathBuf {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "omena-lint-fix-{label}-{}-{id}",
            std::process::id()
        ))
    }
}

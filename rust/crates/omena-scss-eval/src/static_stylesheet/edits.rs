use super::StaticStylesheetEvaluationEdit;

pub(super) fn apply_static_stylesheet_evaluation_edits(
    source: &str,
    edits: Vec<StaticStylesheetEvaluationEdit>,
) -> Option<String> {
    let edits = normalize_static_stylesheet_evaluation_edits(source, edits)?;
    Some(apply_normalized_static_stylesheet_evaluation_edits(
        source, &edits,
    ))
}

pub(super) fn normalize_static_stylesheet_evaluation_edits(
    source: &str,
    mut edits: Vec<StaticStylesheetEvaluationEdit>,
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    edits.sort_by_key(|edit| edit.start);
    edits.dedup_by(|left, right| {
        left.start == right.start && left.end == right.end && left.replacement == right.replacement
    });
    let mut previous_end = 0usize;
    for edit in &edits {
        if edit.start < previous_end || edit.start > edit.end || edit.end > source.len() {
            return None;
        }
        previous_end = edit.end;
    }
    Some(edits)
}

pub(super) fn apply_normalized_static_stylesheet_evaluation_edits(
    source: &str,
    edits: &[StaticStylesheetEvaluationEdit],
) -> String {
    let mut output = source.to_string();
    for edit in edits.iter().rev() {
        output.replace_range(edit.start..edit.end, edit.replacement.as_str());
    }
    output
}

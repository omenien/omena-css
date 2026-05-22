use super::*;
use std::borrow::Cow;

pub(super) fn merge_transform_context(
    mut merged: TransformExecutionContextV0,
    context: &TransformExecutionContextV0,
) -> TransformExecutionContextV0 {
    merged.closed_style_world = merged.closed_style_world || context.closed_style_world;
    merged.drop_dark_mode_media_queries =
        merged.drop_dark_mode_media_queries || context.drop_dark_mode_media_queries;
    merge_context_list(
        &mut merged.reachable_class_names,
        &context.reachable_class_names,
    );
    merge_context_list(
        &mut merged.reachable_keyframe_names,
        &context.reachable_keyframe_names,
    );
    merge_context_list(
        &mut merged.reachable_value_names,
        &context.reachable_value_names,
    );
    merge_context_list(
        &mut merged.reachable_custom_property_names,
        &context.reachable_custom_property_names,
    );

    if context.scss_module_evaluation.is_some() {
        merged.scss_module_evaluation = context.scss_module_evaluation.clone();
    }
    if context.less_module_evaluation.is_some() {
        merged.less_module_evaluation = context.less_module_evaluation.clone();
    }
    if !context.import_inlines.is_empty() {
        merge_context_records_by_key(
            &mut merged.import_inlines,
            &context.import_inlines,
            |inline| inline.import_source.as_str(),
        );
    }
    if !context.class_name_rewrites.is_empty() {
        merge_context_records_by_key(
            &mut merged.class_name_rewrites,
            &context.class_name_rewrites,
            |rewrite| rewrite.original_name.as_str(),
        );
    }
    if !context.css_module_composes_resolutions.is_empty() {
        merge_context_records_by_key(
            &mut merged.css_module_composes_resolutions,
            &context.css_module_composes_resolutions,
            |resolution| resolution.local_class_name.as_str(),
        );
    }
    if !context.css_module_value_resolutions.is_empty() {
        merge_context_records_by_key(
            &mut merged.css_module_value_resolutions,
            &context.css_module_value_resolutions,
            |resolution| resolution.local_name.as_str(),
        );
    }
    if !context.design_token_routes.is_empty() {
        merge_context_records_by_key(
            &mut merged.design_token_routes,
            &context.design_token_routes,
            |route| route.token_name.as_str(),
        );
    }

    expand_reachable_class_names_through_composes(&mut merged);
    merged
}

fn expand_reachable_class_names_through_composes(context: &mut TransformExecutionContextV0) {
    let mut changed = true;
    while changed {
        changed = false;
        for resolution in &context.css_module_composes_resolutions {
            if !class_name_is_reachable(
                &resolution.local_class_name,
                &context.reachable_class_names,
            ) {
                continue;
            }
            for exported_class_name in &resolution.exported_class_names {
                if !class_name_is_reachable(exported_class_name, &context.reachable_class_names) {
                    context
                        .reachable_class_names
                        .push(exported_class_name.clone());
                    changed = true;
                }
            }
        }
    }
    context.reachable_class_names.sort();
    context.reachable_class_names.dedup();
}

fn class_name_is_reachable(class_name: &str, reachable_class_names: &[String]) -> bool {
    let Some(normalized_class_name) = normalize_reachable_class_name(class_name) else {
        return false;
    };
    reachable_class_names
        .iter()
        .filter_map(|name| normalize_reachable_class_name(name))
        .any(|name| css_identifier_names_match(name, normalized_class_name))
}

fn normalize_reachable_class_name(name: &str) -> Option<&str> {
    let name = name.trim().strip_prefix('.').unwrap_or(name.trim());
    (!name.is_empty()).then_some(name)
}

pub(super) fn css_identifier_names_match(left: &str, right: &str) -> bool {
    left == right || decode_css_identifier_escapes(left) == decode_css_identifier_escapes(right)
}

pub(super) fn decode_css_identifier_escapes(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut output = String::with_capacity(text.len());
    let mut index = 0usize;
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch != '\\' {
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }

        index += ch.len_utf8();
        let Some(next) = text[index..].chars().next() else {
            output.push('\\');
            break;
        };
        if next.is_ascii_hexdigit() {
            let hex_start = index;
            let mut hex_end = index;
            let mut digit_count = 0usize;
            while hex_end < text.len() && digit_count < 6 {
                let Some(candidate) = text[hex_end..].chars().next() else {
                    break;
                };
                if !candidate.is_ascii_hexdigit() {
                    break;
                }
                hex_end += candidate.len_utf8();
                digit_count += 1;
            }
            if let Some(decoded) = u32::from_str_radix(&text[hex_start..hex_end], 16)
                .ok()
                .and_then(char::from_u32)
            {
                output.push(decoded);
            }
            index = hex_end;
            if let Some(terminator) = text[index..].chars().next()
                && terminator.is_ascii_whitespace()
            {
                index += terminator.len_utf8();
            }
            continue;
        }

        output.push(next);
        index += next.len_utf8();
    }

    Cow::Owned(output)
}

pub(super) fn merge_target_options_transform_context(
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
) -> TransformExecutionContextV0 {
    let mut merged = context.clone();
    if target_options.drop_dark_mode_media_queries {
        merged.drop_dark_mode_media_queries = true;
    }
    merged
}

pub(super) fn find_target_style_source<'a>(
    target_style_path: &str,
    style_sources: &'a [OmenaQueryStyleSourceInputV0],
) -> Option<&'a str> {
    style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
        .map(|source| source.style_source.as_str())
}

fn merge_context_list(target: &mut Vec<String>, additional: &[String]) {
    for item in additional {
        if !target.contains(item) {
            target.push(item.clone());
        }
    }
    target.sort();
}

fn merge_context_records_by_key<T, F>(target: &mut Vec<T>, overrides: &[T], key: F)
where
    T: Clone,
    F: Fn(&T) -> &str,
{
    for item in overrides {
        let item_key = key(item);
        if let Some(existing) = target.iter_mut().find(|existing| key(existing) == item_key) {
            *existing = item.clone();
        } else {
            target.push(item.clone());
        }
    }
    target.sort_by(|left, right| key(left).cmp(key(right)));
}

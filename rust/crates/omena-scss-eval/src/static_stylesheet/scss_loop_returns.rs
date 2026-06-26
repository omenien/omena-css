use std::collections::BTreeMap;

use crate::static_loop_frames::{
    parse_static_scss_each_loop_binding_frames, parse_static_scss_for_loop_header,
    static_scss_for_loop_values,
};

use super::{
    bind_static_scss_function_local_variables_before,
    bind_static_scss_function_local_variables_in_range, canonical_static_scss_variable_name,
    model::{
        StaticScssFunctionDeclaration, StaticScssFunctionResolutionContext,
        StaticScssFunctionReturnClause, StaticScssLoopHeader,
    },
    resolve_static_scss_function_value_with_bindings,
    value_resolution_model::{
        StaticStylesheetAbstractResolution, StaticStylesheetResolutionOutcome,
        StaticStylesheetResolutionReason, top_static_abstract_value,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum StaticScssLoopReturnResolution {
    Active(StaticStylesheetAbstractResolution),
    Inactive,
    Unknown(StaticStylesheetResolutionReason),
}

pub(super) fn resolve_static_scss_loop_return_clause(
    declaration: &StaticScssFunctionDeclaration,
    clause: &StaticScssFunctionReturnClause,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> StaticScssLoopReturnResolution {
    let Some(frames) = static_scss_loop_binding_frames_for_headers(
        declaration,
        clause.loop_headers.as_slice(),
        argument_values,
        fuel,
        context,
    ) else {
        return StaticScssLoopReturnResolution::Unknown(
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if frames.is_empty() {
        return StaticScssLoopReturnResolution::Inactive;
    }

    for frame in frames {
        let mut frame_values = argument_values.clone();
        for (name, value) in frame {
            frame_values.insert(canonical_static_scss_variable_name(name.as_str()), value);
        }
        let loop_body_start = clause
            .loop_headers
            .last()
            .map(|header| header.body_start)
            .unwrap_or(declaration.body_start);
        let frame_values = match bind_static_scss_function_local_variables_in_range(
            declaration,
            &frame_values,
            loop_body_start,
            clause.span_start,
            fuel,
            context,
        ) {
            Ok(frame_values) => frame_values,
            Err(resolution) => return StaticScssLoopReturnResolution::Unknown(resolution.reason),
        };
        let active = match static_scss_return_clause_is_active(clause, &frame_values, fuel, context)
        {
            Ok(active) => active,
            Err(resolution) => return StaticScssLoopReturnResolution::Unknown(resolution.reason),
        };
        if !active {
            continue;
        }
        return StaticScssLoopReturnResolution::Active(
            resolve_static_scss_function_value_with_bindings(
                clause.value.as_str(),
                &frame_values,
                clause.span_start,
                fuel,
                context,
            ),
        );
    }

    StaticScssLoopReturnResolution::Inactive
}

fn static_scss_return_clause_is_active(
    clause: &StaticScssFunctionReturnClause,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Result<bool, StaticStylesheetAbstractResolution> {
    let Some(condition) = clause.condition.as_ref() else {
        return Ok(true);
    };
    let condition_resolution = resolve_static_scss_function_value_with_bindings(
        condition.as_str(),
        argument_values,
        clause.span_start,
        fuel,
        context,
    );
    if condition_resolution.outcome == StaticStylesheetResolutionOutcome::Top {
        return Err(top_static_abstract_value(condition_resolution.reason));
    }
    let Some(condition_value) = condition_resolution.rendered_value else {
        return Err(top_static_abstract_value(condition_resolution.reason));
    };
    let Some(truthy) = (context.truthiness_evaluator)(condition_value.as_str()) else {
        return Err(top_static_abstract_value(
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        ));
    };
    Ok(truthy)
}

fn static_scss_loop_binding_frames_for_headers(
    declaration: &StaticScssFunctionDeclaration,
    headers: &[StaticScssLoopHeader],
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<Vec<Vec<(String, String)>>> {
    if headers.is_empty() {
        return None;
    }

    let mut frames = vec![Vec::<(String, String)>::new()];
    for header in headers {
        let mut next_frames = Vec::new();
        for frame in frames {
            let mut frame_values = argument_values.clone();
            for (name, value) in &frame {
                frame_values.insert(
                    canonical_static_scss_variable_name(name.as_str()),
                    value.clone(),
                );
            }
            let frame_values = bind_static_scss_function_local_variables_before(
                declaration,
                &frame_values,
                header.span_start,
                fuel,
                context,
            )
            .ok()?;
            let header_frames =
                static_scss_loop_binding_frames(declaration, header, &frame_values, fuel, context)?;
            for header_frame in header_frames {
                let mut combined = frame.clone();
                combined.extend(header_frame);
                next_frames.push(combined);
                if next_frames.len() > 64 {
                    return None;
                }
            }
        }
        frames = next_frames;
    }

    Some(frames)
}

fn static_scss_loop_binding_frames(
    declaration: &StaticScssFunctionDeclaration,
    header: &StaticScssLoopHeader,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<Vec<Vec<(String, String)>>> {
    let header_text = header.text.as_str();
    let position = header.span_start;
    let trimmed = header_text.trim_start();
    if trimmed.to_ascii_lowercase().starts_with("@for") {
        return static_scss_for_loop_binding_frames(
            trimmed,
            argument_values,
            fuel,
            context,
            position,
        );
    }
    if trimmed.to_ascii_lowercase().starts_with("@each") {
        return parse_static_scss_each_loop_binding_frames(trimmed, |source| {
            let resolution = resolve_static_scss_function_value_with_bindings(
                source,
                argument_values,
                position,
                fuel,
                context,
            );
            if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
                return None;
            }
            resolution.rendered_value
        });
    }
    if trimmed.to_ascii_lowercase().starts_with("@while") {
        return static_scss_while_loop_binding_frames(
            declaration,
            header,
            argument_values,
            fuel,
            context,
        );
    }
    None
}

fn static_scss_for_loop_binding_frames(
    header: &str,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    position: usize,
) -> Option<Vec<Vec<(String, String)>>> {
    let for_header = parse_static_scss_for_loop_header(header)?;
    let start = parse_static_scss_for_loop_bound(
        for_header.start_bound,
        argument_values,
        fuel,
        context,
        position,
    )?;
    let end = parse_static_scss_for_loop_bound(
        for_header.end_bound,
        argument_values,
        fuel,
        context,
        position,
    )?;
    let values = static_scss_for_loop_values(start, end, for_header.includes_end)?;
    Some(
        values
            .into_iter()
            .map(|value| vec![(for_header.binding.clone(), value.to_string())])
            .collect(),
    )
}

fn parse_static_scss_for_loop_bound(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    position: usize,
) -> Option<i32> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        value,
        argument_values,
        position,
        fuel,
        context,
    );
    resolution.rendered_value?.parse::<i32>().ok()
}

fn static_scss_while_loop_binding_frames(
    declaration: &StaticScssFunctionDeclaration,
    header: &StaticScssLoopHeader,
    argument_values: &BTreeMap<String, String>,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<Vec<Vec<(String, String)>>> {
    let condition = static_scss_while_condition(header.text.as_str())?;
    if header.body_start >= header.body_end {
        return None;
    }
    let mut frames = Vec::new();
    let mut current_values = argument_values.clone();
    let body_end_position = header.body_end.saturating_sub(1);

    for _ in 0..64 {
        let active = static_scss_while_condition_is_active(
            condition,
            &current_values,
            header.span_start,
            fuel,
            context,
        )?;
        if !active {
            return Some(frames);
        }
        frames.push(
            current_values
                .iter()
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect(),
        );
        let next_values = bind_static_scss_function_local_variables_in_range(
            declaration,
            &current_values,
            header.body_start,
            body_end_position,
            fuel,
            context,
        )
        .ok()?;
        if next_values == current_values {
            return None;
        }
        current_values = next_values;
    }

    None
}

fn static_scss_while_condition(header: &str) -> Option<&str> {
    let trimmed = header.trim_start();
    let keyword = trimmed.get(.."@while".len())?;
    if !keyword.eq_ignore_ascii_case("@while") {
        return None;
    }
    Some(trimmed.get("@while".len()..)?.trim()).filter(|condition| !condition.is_empty())
}

fn static_scss_while_condition_is_active(
    condition: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    fuel: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<bool> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        condition,
        argument_values,
        position,
        fuel,
        context,
    );
    if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
        return None;
    }
    let condition_value = resolution.rendered_value?;
    (context.truthiness_evaluator)(condition_value.as_str())
}

use crate::{
    domains::number::numeric_prefix_end,
    helpers::{
        ascii::normalize_ascii_whitespace,
        values::{split_top_level_value_arguments, split_top_level_whitespace_value_components},
    },
};

pub(crate) fn compress_transition_value(value: &str) -> Option<String> {
    let transitions = split_top_level_value_arguments(value)?;
    let mut compressed = Vec::with_capacity(transitions.len());
    let mut changed = false;

    for transition in transitions {
        let replacement = compress_single_transition_value(&transition)?;
        changed |= replacement != normalize_ascii_whitespace(&transition);
        compressed.push(replacement);
    }

    changed.then(|| compressed.join(","))
}

fn compress_single_transition_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [property, duration, timing, delay] = components.as_slice() else {
        return None;
    };
    if !is_css_time_value(duration) || !is_css_time_value(delay) {
        return None;
    }

    let duration_is_zero = is_zero_css_time_value(duration);
    let delay_is_zero = is_zero_css_time_value(delay);
    let timing_is_default = timing.eq_ignore_ascii_case("ease");

    let mut output = vec![normalize_transition_property(property)];
    if !duration_is_zero || !delay_is_zero {
        output.push(duration.clone());
    }
    if !timing_is_default {
        output.push(timing.clone());
    }
    if !delay_is_zero {
        output.push(delay.clone());
    }

    Some(output.join(" "))
}

fn normalize_transition_property(property: &str) -> String {
    if property.eq_ignore_ascii_case("all") {
        "all".to_string()
    } else {
        property.to_string()
    }
}

pub(crate) fn compress_animation_value(value: &str) -> Option<String> {
    let animations = split_top_level_value_arguments(value)?;
    let mut compressed = Vec::with_capacity(animations.len());
    let mut changed = false;

    for animation in animations {
        let replacement = compress_single_animation_value(&animation)?;
        changed |= replacement != normalize_ascii_whitespace(&animation);
        compressed.push(replacement);
    }

    changed.then(|| compressed.join(","))
}

fn compress_single_animation_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [first, second, third, fourth, fifth, sixth, seventh, eighth] = components.as_slice()
    else {
        return None;
    };

    if animation_tail_is_default(second, third, fourth, fifth, sixth, seventh, eighth) {
        return Some(first.clone());
    }
    if animation_head_is_default(first, second, third, fourth, fifth, sixth, seventh) {
        return Some(eighth.clone());
    }

    None
}

fn animation_tail_is_default(
    duration: &str,
    timing: &str,
    delay: &str,
    iteration_count: &str,
    direction: &str,
    fill_mode: &str,
    play_state: &str,
) -> bool {
    is_zero_css_time_value(duration)
        && timing.eq_ignore_ascii_case("ease")
        && is_zero_css_time_value(delay)
        && iteration_count == "1"
        && direction.eq_ignore_ascii_case("normal")
        && fill_mode.eq_ignore_ascii_case("none")
        && play_state.eq_ignore_ascii_case("running")
}

fn animation_head_is_default(
    duration: &str,
    timing: &str,
    delay: &str,
    iteration_count: &str,
    direction: &str,
    fill_mode: &str,
    play_state: &str,
) -> bool {
    animation_tail_is_default(
        duration,
        timing,
        delay,
        iteration_count,
        direction,
        fill_mode,
        play_state,
    )
}

fn is_css_time_value(value: &str) -> bool {
    css_time_value_parts(value).is_some()
}

fn is_zero_css_time_value(value: &str) -> bool {
    css_time_value_parts(value).is_some_and(|(number, _)| {
        number
            .parse::<f64>()
            .is_ok_and(|parsed| parsed.is_finite() && parsed == 0.0)
    })
}

fn css_time_value_parts(value: &str) -> Option<(&str, &str)> {
    let split = numeric_prefix_end(value)?;
    if split == value.len() {
        return None;
    }
    let (number, unit) = value.split_at(split);
    matches!(unit.to_ascii_lowercase().as_str(), "s" | "ms").then_some((number, unit))
}

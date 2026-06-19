use omena_value_lattice::{
    SrgbColor, StaticSrgbColorWithAlpha, parse_color_function_value, parse_color_mix_value,
    parse_oklab_oklch_value, parse_static_hsl_function_color_with_alpha,
    parse_static_hwb_function_color_with_alpha, parse_static_rgb_function_color_with_alpha,
    parse_static_srgb_color_with_alpha, parse_whole_function_value_arguments,
};

use super::{
    format_static_less_channel_number, parse_static_less_alpha_unit_interval,
    parse_static_less_hsv_unit_interval, parse_static_less_positive_degrees,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StaticLessColorTransformMode {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct StaticLessHslChannels {
    pub(super) hue: f64,
    pub(super) saturation: f64,
    pub(super) lightness: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct StaticLessHsvChannels {
    pub(super) hue: f64,
    pub(super) saturation: f64,
    pub(super) value: f64,
}

pub(super) fn static_less_channel_delta(
    current: f64,
    amount: f64,
    mode: StaticLessColorTransformMode,
) -> f64 {
    match mode {
        StaticLessColorTransformMode::Absolute => amount,
        StaticLessColorTransformMode::Relative => current * amount / 100.0,
    }
}

pub(super) fn static_less_unit_interval_delta(
    current: f64,
    amount: f64,
    mode: StaticLessColorTransformMode,
) -> f64 {
    match mode {
        StaticLessColorTransformMode::Absolute => amount,
        StaticLessColorTransformMode::Relative => current * amount,
    }
}

pub(super) fn parse_static_less_color_transform_arguments(
    value: &str,
    function_name: &str,
) -> Option<(String, String, StaticLessColorTransformMode)> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    match arguments.as_slice() {
        [color, amount] => Some((
            color.trim().to_string(),
            amount.trim().to_string(),
            StaticLessColorTransformMode::Absolute,
        )),
        [color, amount, method] if method.trim().eq_ignore_ascii_case("relative") => Some((
            color.trim().to_string(),
            amount.trim().to_string(),
            StaticLessColorTransformMode::Relative,
        )),
        _ => None,
    }
}

pub(super) fn static_less_hsl_channels(color: StaticSrgbColorWithAlpha) -> StaticLessHslChannels {
    let red = f64::from(color.color.red) / 255.0;
    let green = f64::from(color.color.green) / 255.0;
    let blue = f64::from(color.color.blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) / 2.0;
    let delta = max - min;

    if delta == 0.0 {
        return StaticLessHslChannels {
            hue: 0.0,
            saturation: 0.0,
            lightness: lightness * 100.0,
        };
    }

    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let hue_sector = if max == red {
        ((green - blue) / delta).rem_euclid(6.0)
    } else if max == green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    StaticLessHslChannels {
        hue: hue_sector * 60.0,
        saturation: saturation * 100.0,
        lightness: lightness * 100.0,
    }
}

pub(super) fn static_less_hsv_channels(color: StaticSrgbColorWithAlpha) -> StaticLessHsvChannels {
    let red = f64::from(color.color.red) / 255.0;
    let green = f64::from(color.color.green) / 255.0;
    let blue = f64::from(color.color.blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let saturation = if max == 0.0 { 0.0 } else { delta / max };

    if delta == 0.0 {
        return StaticLessHsvChannels {
            hue: 0.0,
            saturation: saturation * 100.0,
            value: max * 100.0,
        };
    }

    let hue_sector = if max == red {
        ((green - blue) / delta).rem_euclid(6.0)
    } else if max == green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    StaticLessHsvChannels {
        hue: hue_sector * 60.0,
        saturation: saturation * 100.0,
        value: max * 100.0,
    }
}

pub(super) fn format_static_less_color_from_hsl_channels(
    original_color: StaticSrgbColorWithAlpha,
    channels: StaticLessHslChannels,
) -> Option<String> {
    let hue = format_static_less_channel_number(channels.hue.rem_euclid(360.0));
    let saturation = format_static_less_channel_number(channels.saturation.clamp(0.0, 100.0));
    let lightness = format_static_less_channel_number(channels.lightness.clamp(0.0, 100.0));
    let color = parse_static_hsl_function_color_with_alpha(&format!(
        "hsl({hue}, {saturation}%, {lightness}%)"
    ))?;
    Some(format_static_less_color_with_alpha(
        color,
        original_color.alpha.unwrap_or(1.0),
    ))
}

pub(super) fn format_static_less_mixed_color(
    first: StaticSrgbColorWithAlpha,
    second: StaticSrgbColorWithAlpha,
    weight_percentage: f64,
) -> String {
    let first_alpha = first.alpha.unwrap_or(1.0);
    let second_alpha = second.alpha.unwrap_or(1.0);
    let first_stop = (weight_percentage.clamp(0.0, 100.0)) / 100.0;
    let channel_weight = static_less_mix_channel_weight(first_stop, first_alpha, second_alpha);
    let inverse_channel_weight = 1.0 - channel_weight;
    let alpha = first_alpha * first_stop + second_alpha * (1.0 - first_stop);
    let color = StaticSrgbColorWithAlpha {
        color: SrgbColor {
            red: static_less_mix_channel(
                first.color.red,
                second.color.red,
                channel_weight,
                inverse_channel_weight,
            ),
            green: static_less_mix_channel(
                first.color.green,
                second.color.green,
                channel_weight,
                inverse_channel_weight,
            ),
            blue: static_less_mix_channel(
                first.color.blue,
                second.color.blue,
                channel_weight,
                inverse_channel_weight,
            ),
        },
        alpha: None,
    };
    format_static_less_color_with_alpha(color, alpha)
}

pub(super) fn parse_static_less_hsv_color(
    hue: &str,
    saturation: &str,
    value: &str,
    alpha: &str,
) -> Option<String> {
    let hue = parse_static_less_positive_degrees(hue)?;
    let saturation = parse_static_less_hsv_unit_interval(saturation)?;
    let value = parse_static_less_hsv_unit_interval(value)?;
    let alpha = parse_static_less_alpha_unit_interval(alpha)?;
    let hue = hue.rem_euclid(360.0);
    let sector = ((hue / 60.0).floor() as usize) % 6;
    let fraction = (hue / 60.0) - sector as f64;
    let candidates = [
        value,
        value * (1.0 - saturation),
        value * (1.0 - fraction * saturation),
        value * (1.0 - (1.0 - fraction) * saturation),
    ];
    let permutation = match sector {
        0 => [0, 3, 1],
        1 => [2, 0, 1],
        2 => [1, 0, 3],
        3 => [1, 2, 0],
        4 => [3, 1, 0],
        _ => [0, 1, 2],
    };
    Some(format_static_less_color_with_alpha(
        StaticSrgbColorWithAlpha {
            color: SrgbColor {
                red: static_less_blend_channel(candidates[permutation[0]] * 255.0),
                green: static_less_blend_channel(candidates[permutation[1]] * 255.0),
                blue: static_less_blend_channel(candidates[permutation[2]] * 255.0),
            },
            alpha: None,
        },
        alpha,
    ))
}

fn static_less_mix_channel_weight(first_stop: f64, first_alpha: f64, second_alpha: f64) -> f64 {
    let raw_weight = first_stop * 2.0 - 1.0;
    let alpha_delta = first_alpha - second_alpha;
    let weighted_alpha_delta = raw_weight * alpha_delta;
    let adjusted = if (weighted_alpha_delta + 1.0).abs() < f64::EPSILON {
        raw_weight
    } else {
        (raw_weight + alpha_delta) / (1.0 + weighted_alpha_delta)
    };
    (adjusted + 1.0) / 2.0
}

fn static_less_mix_channel(first: u8, second: u8, first_weight: f64, second_weight: f64) -> u8 {
    (f64::from(first) * first_weight + f64::from(second) * second_weight)
        .round()
        .clamp(0.0, 255.0) as u8
}

pub(super) fn format_static_less_blended_color(
    first: StaticSrgbColorWithAlpha,
    second: StaticSrgbColorWithAlpha,
    blend_channel: impl Fn(f64, f64) -> f64,
) -> String {
    let backdrop_alpha = first.alpha.unwrap_or(1.0);
    let source_alpha = second.alpha.unwrap_or(1.0);
    let alpha = source_alpha + backdrop_alpha * (1.0 - source_alpha);
    format_static_less_color_with_alpha(
        StaticSrgbColorWithAlpha {
            color: SrgbColor {
                red: static_less_blend_result_channel(
                    first.color.red,
                    second.color.red,
                    backdrop_alpha,
                    source_alpha,
                    alpha,
                    &blend_channel,
                ),
                green: static_less_blend_result_channel(
                    first.color.green,
                    second.color.green,
                    backdrop_alpha,
                    source_alpha,
                    alpha,
                    &blend_channel,
                ),
                blue: static_less_blend_result_channel(
                    first.color.blue,
                    second.color.blue,
                    backdrop_alpha,
                    source_alpha,
                    alpha,
                    &blend_channel,
                ),
            },
            alpha: None,
        },
        alpha,
    )
}

fn static_less_blend_result_channel(
    backdrop: u8,
    source: u8,
    backdrop_alpha: f64,
    source_alpha: f64,
    alpha: f64,
    blend_channel: &impl Fn(f64, f64) -> f64,
) -> u8 {
    let backdrop = f64::from(backdrop) / 255.0;
    let source = f64::from(source) / 255.0;
    let blended = blend_channel(backdrop, source);
    let result = if alpha > 0.0 {
        (source_alpha * source
            + backdrop_alpha * (backdrop - source_alpha * (backdrop + source - blended)))
            / alpha
    } else {
        blended
    };
    static_less_blend_channel(result * 255.0)
}

pub(super) fn static_less_multiply_value(backdrop: f64, source: f64) -> f64 {
    backdrop * source
}

pub(super) fn static_less_screen_value(backdrop: f64, source: f64) -> f64 {
    backdrop + source - backdrop * source
}

pub(super) fn static_less_overlay_value(backdrop: f64, source: f64) -> f64 {
    if backdrop * 2.0 <= 1.0 {
        return static_less_multiply_value(backdrop * 2.0, source);
    }
    static_less_screen_value(backdrop * 2.0 - 1.0, source)
}

pub(super) fn static_less_softlight_value(backdrop: f64, source: f64) -> f64 {
    let mut distance = 1.0;
    let mut factor = backdrop;
    if source > 0.5 {
        factor = 1.0;
        distance = if backdrop > 0.25 {
            backdrop.sqrt()
        } else {
            ((16.0 * backdrop - 12.0) * backdrop + 4.0) * backdrop
        };
    }
    backdrop - (1.0 - 2.0 * source) * factor * (distance - backdrop)
}

pub(super) fn static_less_exclusion_value(backdrop: f64, source: f64) -> f64 {
    backdrop + source - 2.0 * backdrop * source
}

pub(super) fn static_less_average_value(backdrop: f64, source: f64) -> f64 {
    (backdrop + source) / 2.0
}

pub(super) fn static_less_negation_value(backdrop: f64, source: f64) -> f64 {
    1.0 - (backdrop + source - 1.0).abs()
}

fn static_less_blend_channel(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

pub(super) fn parse_static_less_color_argument(value: &str) -> Option<StaticSrgbColorWithAlpha> {
    parse_static_srgb_color_with_alpha(value)
        .or_else(|| parse_static_rgb_function_color_with_alpha(value))
        .or_else(|| parse_static_hsl_function_color_with_alpha(value))
        .or_else(|| parse_static_hwb_function_color_with_alpha(value))
        .or_else(|| {
            parse_color_function_value(value)
                .or_else(|| parse_color_mix_value(value))
                .or_else(|| parse_oklab_oklch_value(value))
                .and_then(|value| parse_static_less_color_argument(value.as_str()))
        })
}

pub(super) fn static_less_opaque_srgb_color(
    red: u8,
    green: u8,
    blue: u8,
) -> StaticSrgbColorWithAlpha {
    StaticSrgbColorWithAlpha {
        color: SrgbColor { red, green, blue },
        alpha: None,
    }
}

pub(super) fn parse_static_less_hsv_channel_value(
    value: &str,
    function_name: &str,
    channel: impl FnOnce(StaticLessHsvChannels) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    Some(format_static_less_channel_number(channel(
        static_less_hsv_channels(color),
    )))
}

pub(super) fn parse_static_less_luma_or_luminance_value(
    value: &str,
    function_name: &str,
    channel: impl FnOnce(SrgbColor) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    let alpha = color.alpha.unwrap_or(1.0);
    Some(format!(
        "{}%",
        format_static_less_channel_number(channel(color.color) * alpha * 100.0)
    ))
}

pub(super) fn static_less_luma(color: SrgbColor) -> f64 {
    0.2126 * static_less_linear_rgb_channel(color.red)
        + 0.7152 * static_less_linear_rgb_channel(color.green)
        + 0.0722 * static_less_linear_rgb_channel(color.blue)
}

pub(super) fn static_less_luminance(color: SrgbColor) -> f64 {
    (0.2126 * f64::from(color.red)
        + 0.7152 * f64::from(color.green)
        + 0.0722 * f64::from(color.blue))
        / 255.0
}

fn static_less_linear_rgb_channel(channel: u8) -> f64 {
    let channel = f64::from(channel) / 255.0;
    if channel <= 0.03928 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

pub(super) fn static_less_alpha_byte(alpha: f64) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub(super) fn format_static_less_color_with_alpha(
    color: StaticSrgbColorWithAlpha,
    alpha: f64,
) -> String {
    if (alpha - 1.0).abs() < f64::EPSILON {
        return format!(
            "#{:02x}{:02x}{:02x}",
            color.color.red, color.color.green, color.color.blue
        );
    }
    format!(
        "rgba({}, {}, {}, {})",
        color.color.red,
        color.color.green,
        color.color.blue,
        format_static_less_channel_number(alpha)
    )
}

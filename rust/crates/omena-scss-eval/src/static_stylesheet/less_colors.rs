use omena_value_lattice::{
    SrgbColor, StaticSrgbColorWithAlpha, parse_basic_named_srgb_color, parse_color_function_value,
    parse_color_mix_value, parse_numeric_value_with_unit, parse_oklab_oklch_value,
    parse_static_hsl_function_color_with_alpha, parse_static_hwb_function_color_with_alpha,
    parse_static_rgb_function_color_with_alpha, parse_static_srgb_color_with_alpha,
    parse_whole_function_value_arguments,
};

use super::{
    format_static_less_channel_number, format_static_less_number,
    less_strings::static_less_quoted_string_contents,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticLessColorTransformMode {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticLessHslChannels {
    hue: f64,
    saturation: f64,
    lightness: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticLessHsvChannels {
    hue: f64,
    saturation: f64,
    value: f64,
}

pub(super) fn parse_static_less_rgb_color_value(value: &str) -> Option<String> {
    let color = parse_static_rgb_function_color_with_alpha(value)?;
    Some(format_static_less_color_with_alpha(
        color,
        color.alpha.unwrap_or(1.0),
    ))
}

pub(super) fn parse_static_less_red_value(value: &str) -> Option<String> {
    parse_static_less_color_channel_value(value, "red", |color| f64::from(color.color.red))
}

pub(super) fn parse_static_less_green_value(value: &str) -> Option<String> {
    parse_static_less_color_channel_value(value, "green", |color| f64::from(color.color.green))
}

pub(super) fn parse_static_less_blue_value(value: &str) -> Option<String> {
    parse_static_less_color_channel_value(value, "blue", |color| f64::from(color.color.blue))
}

pub(super) fn parse_static_less_alpha_value(value: &str) -> Option<String> {
    parse_static_less_color_channel_value(value, "alpha", |color| color.alpha.unwrap_or(1.0))
}

pub(super) fn parse_static_less_hue_value(value: &str) -> Option<String> {
    parse_static_less_hsl_channel_value(value, "hue", |channels| channels.hue)
}

pub(super) fn parse_static_less_saturation_value(value: &str) -> Option<String> {
    parse_static_less_hsl_channel_value(value, "saturation", |channels| channels.saturation)
        .map(|value| format!("{value}%"))
}

pub(super) fn parse_static_less_lightness_value(value: &str) -> Option<String> {
    parse_static_less_hsl_channel_value(value, "lightness", |channels| channels.lightness)
        .map(|value| format!("{value}%"))
}

pub(super) fn parse_static_less_hsv_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "hsv")?;
    let [hue, saturation, value] = arguments.as_slice() else {
        return None;
    };
    parse_static_less_hsv_color(hue.trim(), saturation.trim(), value.trim(), "1")
}

pub(super) fn parse_static_less_hsva_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "hsva")?;
    let [hue, saturation, value, alpha] = arguments.as_slice() else {
        return None;
    };
    parse_static_less_hsv_color(hue.trim(), saturation.trim(), value.trim(), alpha.trim())
}

pub(super) fn parse_static_less_hsvhue_value(value: &str) -> Option<String> {
    parse_static_less_hsv_channel_value(value, "hsvhue", |channels| channels.hue)
}

pub(super) fn parse_static_less_hsvsaturation_value(value: &str) -> Option<String> {
    parse_static_less_hsv_channel_value(value, "hsvsaturation", |channels| channels.saturation)
        .map(|value| format!("{value}%"))
}

pub(super) fn parse_static_less_hsvvalue_value(value: &str) -> Option<String> {
    parse_static_less_hsv_channel_value(value, "hsvvalue", |channels| channels.value)
        .map(|value| format!("{value}%"))
}

pub(super) fn parse_static_less_luma_value(value: &str) -> Option<String> {
    parse_static_less_luma_or_luminance_value(value, "luma", static_less_luma)
}

pub(super) fn parse_static_less_luminance_value(value: &str) -> Option<String> {
    parse_static_less_luma_or_luminance_value(value, "luminance", static_less_luminance)
}

pub(super) fn parse_static_less_contrast_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "contrast")?;
    let (color, dark, light, threshold) = match arguments.as_slice() {
        [color] => (color.as_str(), None, None, None),
        [color, dark] => (color.as_str(), Some(dark.as_str()), None, None),
        [color, dark, light] => (
            color.as_str(),
            Some(dark.as_str()),
            Some(light.as_str()),
            None,
        ),
        [color, dark, light, threshold] => (
            color.as_str(),
            Some(dark.as_str()),
            Some(light.as_str()),
            Some(threshold.as_str()),
        ),
        _ => return None,
    };
    let color = parse_static_less_color_argument(color.trim())?;
    let mut dark = match dark {
        Some(dark) => parse_static_less_color_argument(dark.trim())?,
        None => static_less_opaque_srgb_color(0, 0, 0),
    };
    let mut light = match light {
        Some(light) => parse_static_less_color_argument(light.trim())?,
        None => static_less_opaque_srgb_color(255, 255, 255),
    };
    if static_less_luma(dark.color) > static_less_luma(light.color) {
        std::mem::swap(&mut dark, &mut light);
    }
    let threshold = threshold
        .map(|threshold| parse_static_less_threshold_number(threshold.trim()))
        .unwrap_or(Some(0.43))?;
    let selected = if static_less_luma(color.color) < threshold {
        light
    } else {
        dark
    };
    Some(format_static_less_color_with_alpha(
        selected,
        selected.alpha.unwrap_or(1.0),
    ))
}

pub(super) fn parse_static_less_color_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "color")?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = color.trim();
    if let Some(hex) = parse_static_less_quoted_hex_color_literal(color) {
        return Some(hex);
    }
    if let Some(named) = static_less_quoted_string_contents(color)
        .as_deref()
        .and_then(parse_basic_named_srgb_color)
    {
        return Some(format_static_less_color_with_alpha(
            StaticSrgbColorWithAlpha {
                color: named,
                alpha: None,
            },
            1.0,
        ));
    }
    let color = parse_static_less_color_argument(color)?;
    Some(format_static_less_color_with_alpha(
        color,
        color.alpha.unwrap_or(1.0),
    ))
}

pub(super) fn parse_static_less_argb_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "argb")?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    Some(format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        static_less_alpha_byte(color.alpha.unwrap_or(1.0)),
        color.color.red,
        color.color.green,
        color.color.blue
    ))
}

pub(super) fn parse_static_less_fade_value(value: &str) -> Option<String> {
    parse_static_less_alpha_transform_value(value, "fade", |_, amount, _| amount)
}

pub(super) fn parse_static_less_fadein_value(value: &str) -> Option<String> {
    parse_static_less_alpha_transform_value(value, "fadein", |current, amount, mode| {
        current + static_less_unit_interval_delta(current, amount, mode)
    })
}

pub(super) fn parse_static_less_fadeout_value(value: &str) -> Option<String> {
    parse_static_less_alpha_transform_value(value, "fadeout", |current, amount, mode| {
        current - static_less_unit_interval_delta(current, amount, mode)
    })
}

pub(super) fn parse_static_less_mix_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "mix")?;
    let (first, second, weight) = match arguments.as_slice() {
        [first, second] => (first.as_str(), second.as_str(), 50.0),
        [first, second, weight] => (
            first.as_str(),
            second.as_str(),
            parse_static_less_percentage_points(weight.trim())?,
        ),
        _ => return None,
    };
    let first = parse_static_less_color_argument(first.trim())?;
    let second = parse_static_less_color_argument(second.trim())?;
    Some(format_static_less_mixed_color(first, second, weight))
}

pub(super) fn parse_static_less_tint_value(value: &str) -> Option<String> {
    parse_static_less_tone_mix_value(value, "tint", "white")
}

pub(super) fn parse_static_less_shade_value(value: &str) -> Option<String> {
    parse_static_less_tone_mix_value(value, "shade", "black")
}

fn parse_static_less_tone_mix_value(
    value: &str,
    function_name: &str,
    base_color: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color, weight] = arguments.as_slice() else {
        return None;
    };
    let base_color = parse_static_less_color_argument(base_color)?;
    let color = parse_static_less_color_argument(color.trim())?;
    let weight = parse_static_less_percentage_points(weight.trim())?;
    Some(format_static_less_mixed_color(base_color, color, weight))
}

pub(super) fn parse_static_less_multiply_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "multiply", static_less_multiply_value)
}

pub(super) fn parse_static_less_screen_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "screen", static_less_screen_value)
}

pub(super) fn parse_static_less_overlay_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "overlay", static_less_overlay_value)
}

pub(super) fn parse_static_less_softlight_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "softlight", static_less_softlight_value)
}

pub(super) fn parse_static_less_hardlight_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "hardlight", |backdrop, source| {
        static_less_overlay_value(source, backdrop)
    })
}

pub(super) fn parse_static_less_difference_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "difference", |backdrop, source| {
        (backdrop - source).abs()
    })
}

pub(super) fn parse_static_less_exclusion_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "exclusion", static_less_exclusion_value)
}

pub(super) fn parse_static_less_average_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "average", static_less_average_value)
}

pub(super) fn parse_static_less_negation_value(value: &str) -> Option<String> {
    parse_static_less_blend_value(value, "negation", static_less_negation_value)
}

fn parse_static_less_blend_value(
    value: &str,
    function_name: &str,
    blend_channel: impl Fn(f64, f64) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [first, second] = arguments.as_slice() else {
        return None;
    };
    let first = parse_static_less_color_argument(first.trim())?;
    let second = parse_static_less_color_argument(second.trim())?;
    Some(format_static_less_blended_color(
        first,
        second,
        blend_channel,
    ))
}

pub(super) fn parse_static_less_lighten_value(value: &str) -> Option<String> {
    parse_static_less_hsl_amount_transform_value(value, "lighten", |mut channels, amount, mode| {
        channels.lightness = (channels.lightness
            + static_less_channel_delta(channels.lightness, amount, mode))
        .clamp(0.0, 100.0);
        channels
    })
}

pub(super) fn parse_static_less_darken_value(value: &str) -> Option<String> {
    parse_static_less_hsl_amount_transform_value(value, "darken", |mut channels, amount, mode| {
        channels.lightness = (channels.lightness
            - static_less_channel_delta(channels.lightness, amount, mode))
        .clamp(0.0, 100.0);
        channels
    })
}

pub(super) fn parse_static_less_saturate_value(value: &str) -> Option<String> {
    parse_static_less_hsl_amount_transform_value(value, "saturate", |mut channels, amount, mode| {
        channels.saturation = (channels.saturation
            + static_less_channel_delta(channels.saturation, amount, mode))
        .clamp(0.0, 100.0);
        channels
    })
}

pub(super) fn parse_static_less_desaturate_value(value: &str) -> Option<String> {
    parse_static_less_hsl_amount_transform_value(
        value,
        "desaturate",
        |mut channels, amount, mode| {
            channels.saturation = (channels.saturation
                - static_less_channel_delta(channels.saturation, amount, mode))
            .clamp(0.0, 100.0);
            channels
        },
    )
}

pub(super) fn parse_static_less_spin_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "spin")?;
    let [color, amount] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    let mut channels = static_less_hsl_channels(color);
    channels.hue =
        (channels.hue + parse_static_less_angle_degrees(amount.trim())?).rem_euclid(360.0);
    format_static_less_color_from_hsl_channels(color, channels)
}

pub(super) fn parse_static_less_greyscale_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "greyscale")?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    let mut channels = static_less_hsl_channels(color);
    channels.saturation = 0.0;
    format_static_less_color_from_hsl_channels(color, channels)
}

fn parse_static_less_hsl_amount_transform_value(
    value: &str,
    function_name: &str,
    transform: impl FnOnce(
        StaticLessHslChannels,
        f64,
        StaticLessColorTransformMode,
    ) -> StaticLessHslChannels,
) -> Option<String> {
    let (color, amount, mode) = parse_static_less_color_transform_arguments(value, function_name)?;
    let color = parse_static_less_color_argument(color.as_str())?;
    let amount = parse_static_less_percentage_points(amount.as_str())?;
    format_static_less_color_from_hsl_channels(
        color,
        transform(static_less_hsl_channels(color), amount, mode),
    )
}

fn parse_static_less_alpha_transform_value(
    value: &str,
    function_name: &str,
    transform: impl FnOnce(f64, f64, StaticLessColorTransformMode) -> f64,
) -> Option<String> {
    let (color, amount, mode) = parse_static_less_color_transform_arguments(value, function_name)?;
    let color = parse_static_less_color_argument(color.as_str())?;
    let amount = parse_static_less_alpha_amount(amount.as_str())?;
    let alpha = transform(color.alpha.unwrap_or(1.0), amount, mode).clamp(0.0, 1.0);
    Some(format_static_less_color_with_alpha(color, alpha))
}

fn parse_static_less_color_channel_value(
    value: &str,
    function_name: &str,
    channel: impl FnOnce(StaticSrgbColorWithAlpha) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    Some(format_static_less_number(channel(color)))
}

fn parse_static_less_hsl_channel_value(
    value: &str,
    function_name: &str,
    channel: impl FnOnce(StaticLessHslChannels) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_less_color_argument(color.trim())?;
    Some(format_static_less_channel_number(channel(
        static_less_hsl_channels(color),
    )))
}

fn static_less_channel_delta(current: f64, amount: f64, mode: StaticLessColorTransformMode) -> f64 {
    match mode {
        StaticLessColorTransformMode::Absolute => amount,
        StaticLessColorTransformMode::Relative => current * amount / 100.0,
    }
}

fn static_less_unit_interval_delta(
    current: f64,
    amount: f64,
    mode: StaticLessColorTransformMode,
) -> f64 {
    match mode {
        StaticLessColorTransformMode::Absolute => amount,
        StaticLessColorTransformMode::Relative => current * amount,
    }
}

fn parse_static_less_color_transform_arguments(
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

fn parse_static_less_alpha_amount(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() || !matches!(parsed.unit, "" | "%") {
        return None;
    }
    Some((parsed.value / 100.0).clamp(0.0, 1.0))
}

fn parse_static_less_alpha_unit_interval(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() || !matches!(parsed.unit, "" | "%") {
        return None;
    }
    let value = if parsed.unit == "%" {
        parsed.value / 100.0
    } else {
        parsed.value
    };
    (0.0..=1.0).contains(&value).then_some(value)
}

fn parse_static_less_hsv_unit_interval(value: &str) -> Option<f64> {
    parse_static_less_alpha_unit_interval(value)
}

fn parse_static_less_threshold_number(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() || !matches!(parsed.unit, "" | "%") {
        return None;
    }
    Some(if parsed.unit == "%" {
        parsed.value / 100.0
    } else {
        parsed.value
    })
}

fn parse_static_less_percentage_points(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() || !matches!(parsed.unit, "" | "%") {
        return None;
    }
    Some(parsed.value)
}

fn parse_static_less_positive_degrees(value: &str) -> Option<f64> {
    let degrees = parse_static_less_angle_degrees(value)?;
    (degrees >= 0.0).then_some(degrees)
}

fn parse_static_less_angle_degrees(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() {
        return None;
    }
    match parsed.unit.to_ascii_lowercase().as_str() {
        "" | "deg" => Some(parsed.value),
        "rad" => Some(parsed.value.to_degrees()),
        "grad" => Some(parsed.value * 0.9),
        _ => None,
    }
}

fn parse_static_less_quoted_hex_color_literal(value: &str) -> Option<String> {
    let text = static_less_quoted_string_contents(value)?;
    let hex = text.strip_prefix('#')?;
    matches!(hex.len(), 3 | 4 | 6 | 8)
        .then_some(hex)?
        .chars()
        .all(|ch| ch.is_ascii_hexdigit())
        .then_some(text)
}

fn static_less_hsl_channels(color: StaticSrgbColorWithAlpha) -> StaticLessHslChannels {
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

fn static_less_hsv_channels(color: StaticSrgbColorWithAlpha) -> StaticLessHsvChannels {
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

fn format_static_less_color_from_hsl_channels(
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

fn format_static_less_mixed_color(
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

fn parse_static_less_hsv_color(
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

fn format_static_less_blended_color(
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

fn static_less_multiply_value(backdrop: f64, source: f64) -> f64 {
    backdrop * source
}

fn static_less_screen_value(backdrop: f64, source: f64) -> f64 {
    backdrop + source - backdrop * source
}

fn static_less_overlay_value(backdrop: f64, source: f64) -> f64 {
    if backdrop * 2.0 <= 1.0 {
        return static_less_multiply_value(backdrop * 2.0, source);
    }
    static_less_screen_value(backdrop * 2.0 - 1.0, source)
}

fn static_less_softlight_value(backdrop: f64, source: f64) -> f64 {
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

fn static_less_exclusion_value(backdrop: f64, source: f64) -> f64 {
    backdrop + source - 2.0 * backdrop * source
}

fn static_less_average_value(backdrop: f64, source: f64) -> f64 {
    (backdrop + source) / 2.0
}

fn static_less_negation_value(backdrop: f64, source: f64) -> f64 {
    1.0 - (backdrop + source - 1.0).abs()
}

fn static_less_blend_channel(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn parse_static_less_color_argument(value: &str) -> Option<StaticSrgbColorWithAlpha> {
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

fn static_less_opaque_srgb_color(red: u8, green: u8, blue: u8) -> StaticSrgbColorWithAlpha {
    StaticSrgbColorWithAlpha {
        color: SrgbColor { red, green, blue },
        alpha: None,
    }
}

fn parse_static_less_hsv_channel_value(
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

fn parse_static_less_luma_or_luminance_value(
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

fn static_less_luma(color: SrgbColor) -> f64 {
    0.2126 * static_less_linear_rgb_channel(color.red)
        + 0.7152 * static_less_linear_rgb_channel(color.green)
        + 0.0722 * static_less_linear_rgb_channel(color.blue)
}

fn static_less_luminance(color: SrgbColor) -> f64 {
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

fn static_less_alpha_byte(alpha: f64) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn format_static_less_color_with_alpha(color: StaticSrgbColorWithAlpha, alpha: f64) -> String {
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

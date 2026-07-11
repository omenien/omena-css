use crate::{
    io::read_source,
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};
use omena_query::{
    summarize_omena_query_consumer_check_style_source, summarize_omena_query_style_document,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PerceptualCheckCliReportV0 {
    pub(crate) schema_version: &'static str,
    pub(crate) product: &'static str,
    pub(crate) command: &'static str,
    pub(crate) claim_level: &'static str,
    pub(crate) style_path: String,
    pub(crate) language: &'static str,
    pub(crate) fact_source_products: Vec<&'static str>,
    pub(crate) selector_count: usize,
    pub(crate) custom_property_declaration_count: usize,
    pub(crate) custom_property_reference_count: usize,
    pub(crate) diagnostic_count: usize,
    pub(crate) color_machinery_source: &'static str,
    pub(crate) json_schema_ready: bool,
    pub(crate) downstream_tool_scaffold_ready: bool,
    pub(crate) consumes_omena_facts: bool,
    pub(crate) wcag_algorithm_ready: bool,
    pub(crate) wcag_exact_color_contrast_bound_count: usize,
    pub(crate) wcag_exact_color_contrast_bounds: Vec<PerceptualExactColorContrastBoundV0>,
    pub(crate) apca_algorithm_ready: bool,
    pub(crate) oklab_perceptual_operator_ready: bool,
    pub(crate) full_perceptual_algorithm_ready: bool,
    pub(crate) public_safety_claim_ready: bool,
    pub(crate) supported_claims: Vec<&'static str>,
    pub(crate) deferred_claims: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PerceptualExactColorContrastBoundV0 {
    pub(crate) schema_version: &'static str,
    pub(crate) product: &'static str,
    pub(crate) feature_gate: &'static str,
    pub(crate) claim_level: &'static str,
    pub(crate) selector_name: String,
    pub(crate) foreground_property: &'static str,
    pub(crate) background_property: &'static str,
    pub(crate) foreground: String,
    pub(crate) background: String,
    pub(crate) foreground_luminance: f64,
    pub(crate) background_luminance: f64,
    pub(crate) contrast_ratio: f64,
    pub(crate) wcag_aa_normal_text_threshold: f64,
    pub(crate) passes_aa_normal_text: bool,
    pub(crate) public_safety_claim_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PerceptualExactSrgbColorV0 {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PerceptualDeclarationColorV0 {
    property: &'static str,
    value: String,
    color: PerceptualExactSrgbColorV0,
}

pub(crate) fn perceptual_check(path: PathBuf, json: bool) -> Result<(), String> {
    let report = perceptual_check_summary(&path)?;

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.perceptual-check"),
            &report,
        )?;
        return Ok(());
    }

    println!("product: {}", report.product);
    println!("file: {}", report.style_path);
    println!("language: {}", report.language);
    println!("claim level: {}", report.claim_level);
    println!("selectors: {}", report.selector_count);
    println!(
        "custom property declarations: {}",
        report.custom_property_declaration_count
    );
    println!(
        "custom property references: {}",
        report.custom_property_reference_count
    );
    println!("diagnostics: {}", report.diagnostic_count);
    println!(
        "downstream scaffold ready: {}",
        report.downstream_tool_scaffold_ready
    );
    println!(
        "WCAG exact color contrast bounds: {}",
        report.wcag_exact_color_contrast_bound_count
    );
    println!(
        "full perceptual algorithm ready: {}",
        report.full_perceptual_algorithm_ready
    );
    Ok(())
}

pub(crate) fn perceptual_check_summary(path: &Path) -> Result<PerceptualCheckCliReportV0, String> {
    let source = read_source(path)?;
    let style_path = path_string(path);
    let style_document = summarize_omena_query_style_document(&style_path, &source)
        .ok_or_else(|| format!("failed to read style document facts for {style_path}"))?;
    let check = summarize_omena_query_consumer_check_style_source(&style_path, &source);
    let wcag_exact_color_contrast_bounds = collect_wcag_exact_color_contrast_bounds_v0(&source);
    let wcag_exact_color_contrast_bound_count = wcag_exact_color_contrast_bounds.len();

    Ok(PerceptualCheckCliReportV0 {
        schema_version: "0",
        product: "omena-cli.perceptual-check",
        command: "perceptual-check",
        claim_level: "fixtureWitnessExactColorWcagContrast",
        style_path,
        language: style_document.language,
        fact_source_products: vec![style_document.product, check.product],
        selector_count: style_document.selector_names.len(),
        custom_property_declaration_count: style_document.custom_property_decl_names.len(),
        custom_property_reference_count: style_document.custom_property_ref_names.len(),
        diagnostic_count: style_document
            .diagnostic_count
            .max(check.parser_error_count),
        color_machinery_source: "omena-cli.perceptual-check.exact-srgb-wcag",
        json_schema_ready: true,
        downstream_tool_scaffold_ready: true,
        consumes_omena_facts: true,
        wcag_algorithm_ready: wcag_exact_color_contrast_bound_count > 0,
        wcag_exact_color_contrast_bound_count,
        wcag_exact_color_contrast_bounds,
        apca_algorithm_ready: false,
        oklab_perceptual_operator_ready: false,
        full_perceptual_algorithm_ready: false,
        public_safety_claim_ready: false,
        supported_claims: vec![
            "perceptual-check CLI report",
            "JSON output schema",
            "Omena fact-level input consumption",
            "WCAG contrast bound for exact sRGB color/background pairs",
        ],
        deferred_claims: vec![
            "non-exact and cascade-computed color contrast",
            "APCA algorithm",
            "OKLab perceptual operator",
            "full perceptual algorithm",
            "public safety claim",
        ],
    })
}

fn collect_wcag_exact_color_contrast_bounds_v0(
    source: &str,
) -> Vec<PerceptualExactColorContrastBoundV0> {
    let mut bounds = Vec::new();
    for block in source.split('}') {
        let Some((selector_text, declaration_text)) = block.split_once('{') else {
            continue;
        };
        let selector_name =
            extract_first_class_selector_name_v0(selector_text).unwrap_or_else(|| {
                selector_text
                    .trim()
                    .split(',')
                    .next()
                    .unwrap_or("<unknown>")
                    .trim()
                    .to_string()
            });
        let mut foreground = None;
        let mut background = None;
        for declaration in declaration_text.split(';') {
            let Some((property, value)) = declaration.split_once(':') else {
                continue;
            };
            let property = property.trim().to_ascii_lowercase();
            let value = strip_declaration_priority_v0(value.trim());
            let Some(color) = parse_perceptual_exact_srgb_color_v0(value) else {
                continue;
            };
            match property.as_str() {
                "color" => {
                    foreground = Some(PerceptualDeclarationColorV0 {
                        property: "color",
                        value: value.to_string(),
                        color,
                    });
                }
                "background" | "background-color" => {
                    background = Some(PerceptualDeclarationColorV0 {
                        property: if property == "background" {
                            "background"
                        } else {
                            "background-color"
                        },
                        value: value.to_string(),
                        color,
                    });
                }
                _ => {}
            }
        }
        let (Some(foreground), Some(background)) = (foreground, background) else {
            continue;
        };
        let foreground_luminance = wcag_relative_luminance_v0(foreground.color);
        let background_luminance = wcag_relative_luminance_v0(background.color);
        let contrast_ratio = wcag_contrast_ratio_v0(foreground_luminance, background_luminance);
        bounds.push(PerceptualExactColorContrastBoundV0 {
            schema_version: "0",
            product: "omena-cli.perceptual-check.wcag-exact-color-contrast",
            feature_gate: "wcag-exact-color-contrast-v0",
            claim_level: "fixtureWitnessExactColorWcagContrast",
            selector_name,
            foreground_property: foreground.property,
            background_property: background.property,
            foreground: foreground.value,
            background: background.value,
            foreground_luminance,
            background_luminance,
            contrast_ratio,
            wcag_aa_normal_text_threshold: 4.5,
            passes_aa_normal_text: contrast_ratio >= 4.5,
            public_safety_claim_ready: false,
        });
    }
    bounds
}

fn extract_first_class_selector_name_v0(selector_text: &str) -> Option<String> {
    let start = selector_text.find('.')? + 1;
    let name = selector_text[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn strip_declaration_priority_v0(value: &str) -> &str {
    value
        .strip_suffix("!important")
        .map(str::trim)
        .unwrap_or(value)
}

fn parse_perceptual_exact_srgb_color_v0(value: &str) -> Option<PerceptualExactSrgbColorV0> {
    let trimmed = value.trim();
    parse_perceptual_hex_color_v0(trimmed)
        .or_else(|| parse_perceptual_basic_named_color_v0(trimmed))
        .or_else(|| parse_perceptual_rgb_function_v0(trimmed))
}

fn parse_perceptual_hex_color_v0(value: &str) -> Option<PerceptualExactSrgbColorV0> {
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let mut chars = hex.chars();
            Some(PerceptualExactSrgbColorV0 {
                red: parse_repeated_hex_digit_v0(chars.next()?)?,
                green: parse_repeated_hex_digit_v0(chars.next()?)?,
                blue: parse_repeated_hex_digit_v0(chars.next()?)?,
            })
        }
        6 => Some(PerceptualExactSrgbColorV0 {
            red: u8::from_str_radix(hex.get(0..2)?, 16).ok()?,
            green: u8::from_str_radix(hex.get(2..4)?, 16).ok()?,
            blue: u8::from_str_radix(hex.get(4..6)?, 16).ok()?,
        }),
        _ => None,
    }
}

fn parse_repeated_hex_digit_v0(ch: char) -> Option<u8> {
    let value = ch.to_digit(16)? as u8;
    Some(value * 17)
}

fn parse_perceptual_basic_named_color_v0(value: &str) -> Option<PerceptualExactSrgbColorV0> {
    match value.to_ascii_lowercase().as_str() {
        "black" => Some(PerceptualExactSrgbColorV0 {
            red: 0,
            green: 0,
            blue: 0,
        }),
        "white" => Some(PerceptualExactSrgbColorV0 {
            red: 255,
            green: 255,
            blue: 255,
        }),
        "red" => Some(PerceptualExactSrgbColorV0 {
            red: 255,
            green: 0,
            blue: 0,
        }),
        "green" => Some(PerceptualExactSrgbColorV0 {
            red: 0,
            green: 128,
            blue: 0,
        }),
        "blue" => Some(PerceptualExactSrgbColorV0 {
            red: 0,
            green: 0,
            blue: 255,
        }),
        _ => None,
    }
}

fn parse_perceptual_rgb_function_v0(value: &str) -> Option<PerceptualExactSrgbColorV0> {
    let inner = value
        .strip_prefix("rgb(")
        .or_else(|| value.strip_prefix("rgba("))?
        .strip_suffix(')')?;
    if inner.contains('/') {
        return None;
    }
    let components = inner
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let [red, green, blue] = components.as_slice() else {
        return None;
    };
    Some(PerceptualExactSrgbColorV0 {
        red: parse_perceptual_rgb_channel_v0(red)?,
        green: parse_perceptual_rgb_channel_v0(green)?,
        blue: parse_perceptual_rgb_channel_v0(blue)?,
    })
}

fn parse_perceptual_rgb_channel_v0(value: &str) -> Option<u8> {
    let parsed = value.parse::<u8>().ok()?;
    Some(parsed)
}

fn wcag_relative_luminance_v0(color: PerceptualExactSrgbColorV0) -> f64 {
    0.2126 * wcag_linear_srgb_channel_v0(color.red)
        + 0.7152 * wcag_linear_srgb_channel_v0(color.green)
        + 0.0722 * wcag_linear_srgb_channel_v0(color.blue)
}

fn wcag_linear_srgb_channel_v0(channel: u8) -> f64 {
    let value = f64::from(channel) / 255.0;
    if value <= 0.039_28 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn wcag_contrast_ratio_v0(left_luminance: f64, right_luminance: f64) -> f64 {
    let lighter = left_luminance.max(right_luminance);
    let darker = left_luminance.min(right_luminance);
    (lighter + 0.05) / (darker + 0.05)
}

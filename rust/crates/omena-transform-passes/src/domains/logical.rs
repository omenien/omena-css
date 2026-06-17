use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::helpers::{
    declarations::{SimpleDeclarationSlice, collect_simple_declarations_in_block},
    source_rewrite::replace_source_ranges,
    tokens::matching_right_brace_index,
    values::split_top_level_whitespace_value_components,
};

pub(crate) fn lower_css_logical_to_physical_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            let Some(axis_mapping) = static_logical_axis_mapping_for_declarations(&declarations)
            else {
                index = close_index + 1;
                continue;
            };
            for declaration in declarations {
                let Some(physical_declaration) = physical_declaration_for_logical_declaration(
                    &declaration.property,
                    &declaration.value,
                    axis_mapping,
                ) else {
                    continue;
                };
                replacements.push((declaration.start, declaration.end, physical_declaration));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineDirection {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhysicalSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl PhysicalSide {
    fn name(self) -> &'static str {
        match self {
            PhysicalSide::Top => "top",
            PhysicalSide::Right => "right",
            PhysicalSide::Bottom => "bottom",
            PhysicalSide::Left => "left",
        }
    }

    fn is_vertical_edge(self) -> bool {
        matches!(self, PhysicalSide::Top | PhysicalSide::Bottom)
    }

    fn is_horizontal_edge(self) -> bool {
        matches!(self, PhysicalSide::Right | PhysicalSide::Left)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LogicalAxisMapping {
    block_start: PhysicalSide,
    block_end: PhysicalSide,
    inline_start: PhysicalSide,
    inline_end: PhysicalSide,
    block_size_property: &'static str,
    inline_size_property: &'static str,
}

fn static_logical_axis_mapping_for_declarations(
    declarations: &[SimpleDeclarationSlice],
) -> Option<LogicalAxisMapping> {
    let writing_mode = declarations
        .iter()
        .rev()
        .find(|declaration| declaration.property == "writing-mode")
        .map(|declaration| match declaration.value.as_str() {
            "horizontal-tb" => Some(WritingMode::HorizontalTb),
            "vertical-rl" => Some(WritingMode::VerticalRl),
            "vertical-lr" => Some(WritingMode::VerticalLr),
            _ => None,
        })
        .unwrap_or(Some(WritingMode::HorizontalTb))?;
    let direction = declarations
        .iter()
        .rev()
        .find(|declaration| declaration.property == "direction")
        .and_then(|declaration| match declaration.value.as_str() {
            "ltr" => Some(InlineDirection::Ltr),
            "rtl" => Some(InlineDirection::Rtl),
            _ => None,
        })?;

    Some(logical_axis_mapping(writing_mode, direction))
}

fn logical_axis_mapping(
    writing_mode: WritingMode,
    direction: InlineDirection,
) -> LogicalAxisMapping {
    match writing_mode {
        WritingMode::HorizontalTb => LogicalAxisMapping {
            block_start: PhysicalSide::Top,
            block_end: PhysicalSide::Bottom,
            inline_start: inline_start_side(direction, PhysicalSide::Left, PhysicalSide::Right),
            inline_end: inline_end_side(direction, PhysicalSide::Left, PhysicalSide::Right),
            block_size_property: "height",
            inline_size_property: "width",
        },
        WritingMode::VerticalRl => LogicalAxisMapping {
            block_start: PhysicalSide::Right,
            block_end: PhysicalSide::Left,
            inline_start: inline_start_side(direction, PhysicalSide::Top, PhysicalSide::Bottom),
            inline_end: inline_end_side(direction, PhysicalSide::Top, PhysicalSide::Bottom),
            block_size_property: "width",
            inline_size_property: "height",
        },
        WritingMode::VerticalLr => LogicalAxisMapping {
            block_start: PhysicalSide::Left,
            block_end: PhysicalSide::Right,
            inline_start: inline_start_side(direction, PhysicalSide::Top, PhysicalSide::Bottom),
            inline_end: inline_end_side(direction, PhysicalSide::Top, PhysicalSide::Bottom),
            block_size_property: "width",
            inline_size_property: "height",
        },
    }
}

fn inline_start_side(
    direction: InlineDirection,
    ltr_side: PhysicalSide,
    rtl_side: PhysicalSide,
) -> PhysicalSide {
    match direction {
        InlineDirection::Ltr => ltr_side,
        InlineDirection::Rtl => rtl_side,
    }
}

fn inline_end_side(
    direction: InlineDirection,
    ltr_side: PhysicalSide,
    rtl_side: PhysicalSide,
) -> PhysicalSide {
    match direction {
        InlineDirection::Ltr => rtl_side,
        InlineDirection::Rtl => ltr_side,
    }
}

fn physical_declaration_for_logical_declaration(
    property: &str,
    value: &str,
    axis_mapping: LogicalAxisMapping,
) -> Option<String> {
    if let Some(physical_property) = physical_property_for_logical_property(property, axis_mapping)
    {
        return Some(format!("{physical_property}: {value};"));
    }

    if let Some((start_property, end_property)) =
        physical_pair_properties_for_logical_pair(property, axis_mapping)
    {
        let (start_value, end_value) = logical_pair_values(value)?;
        return Some(format!(
            "{start_property}: {start_value}; {end_property}: {end_value};"
        ));
    }

    if let Some((start_property, end_property)) =
        physical_pair_properties_for_logical_mirror(property, axis_mapping)
    {
        return Some(format!(
            "{start_property}: {value}; {end_property}: {value};"
        ));
    }

    None
}

fn physical_property_for_logical_property(
    property: &str,
    axis_mapping: LogicalAxisMapping,
) -> Option<String> {
    match property {
        "block-size" => Some(axis_mapping.block_size_property.to_string()),
        "inline-size" => Some(axis_mapping.inline_size_property.to_string()),
        "max-block-size" => Some(format!("max-{}", axis_mapping.block_size_property)),
        "max-inline-size" => Some(format!("max-{}", axis_mapping.inline_size_property)),
        "min-block-size" => Some(format!("min-{}", axis_mapping.block_size_property)),
        "min-inline-size" => Some(format!("min-{}", axis_mapping.inline_size_property)),
        "inset-block-start" => Some(axis_mapping.block_start.name().to_string()),
        "inset-block-end" => Some(axis_mapping.block_end.name().to_string()),
        "inset-inline-start" => Some(axis_mapping.inline_start.name().to_string()),
        "inset-inline-end" => Some(axis_mapping.inline_end.name().to_string()),
        "margin-block-start" => Some(side_property("margin", axis_mapping.block_start)),
        "margin-block-end" => Some(side_property("margin", axis_mapping.block_end)),
        "margin-inline-start" => Some(side_property("margin", axis_mapping.inline_start)),
        "margin-inline-end" => Some(side_property("margin", axis_mapping.inline_end)),
        "padding-inline-start" => Some(side_property("padding", axis_mapping.inline_start)),
        "padding-inline-end" => Some(side_property("padding", axis_mapping.inline_end)),
        "padding-block-start" => Some(side_property("padding", axis_mapping.block_start)),
        "padding-block-end" => Some(side_property("padding", axis_mapping.block_end)),
        "border-block-start-color" => Some(border_side_property(
            axis_mapping.block_start,
            Some("color"),
        )),
        "border-block-end-color" => {
            Some(border_side_property(axis_mapping.block_end, Some("color")))
        }
        "border-inline-start-color" => Some(border_side_property(
            axis_mapping.inline_start,
            Some("color"),
        )),
        "border-inline-end-color" => {
            Some(border_side_property(axis_mapping.inline_end, Some("color")))
        }
        "border-inline-start-style" => Some(border_side_property(
            axis_mapping.inline_start,
            Some("style"),
        )),
        "border-inline-end-style" => {
            Some(border_side_property(axis_mapping.inline_end, Some("style")))
        }
        "border-block-start-style" => Some(border_side_property(
            axis_mapping.block_start,
            Some("style"),
        )),
        "border-block-end-style" => {
            Some(border_side_property(axis_mapping.block_end, Some("style")))
        }
        "border-inline-start-width" => Some(border_side_property(
            axis_mapping.inline_start,
            Some("width"),
        )),
        "border-inline-end-width" => {
            Some(border_side_property(axis_mapping.inline_end, Some("width")))
        }
        "border-block-start-width" => Some(border_side_property(
            axis_mapping.block_start,
            Some("width"),
        )),
        "border-block-end-width" => {
            Some(border_side_property(axis_mapping.block_end, Some("width")))
        }
        "border-block-start" => Some(border_side_property(axis_mapping.block_start, None)),
        "border-block-end" => Some(border_side_property(axis_mapping.block_end, None)),
        "border-inline-start" => Some(border_side_property(axis_mapping.inline_start, None)),
        "border-inline-end" => Some(border_side_property(axis_mapping.inline_end, None)),
        "border-start-start-radius" => {
            corner_radius_property(axis_mapping.block_start, axis_mapping.inline_start)
        }
        "border-start-end-radius" => {
            corner_radius_property(axis_mapping.block_start, axis_mapping.inline_end)
        }
        "border-end-start-radius" => {
            corner_radius_property(axis_mapping.block_end, axis_mapping.inline_start)
        }
        "border-end-end-radius" => {
            corner_radius_property(axis_mapping.block_end, axis_mapping.inline_end)
        }
        _ => None,
    }
}

fn physical_pair_properties_for_logical_pair(
    property: &str,
    axis_mapping: LogicalAxisMapping,
) -> Option<(String, String)> {
    match property {
        "inset-block" => Some(side_pair(axis_mapping.block_start, axis_mapping.block_end)),
        "inset-inline" => Some(side_pair(
            axis_mapping.inline_start,
            axis_mapping.inline_end,
        )),
        "margin-block" => Some(side_property_pair(
            "margin",
            axis_mapping.block_start,
            axis_mapping.block_end,
        )),
        "margin-inline" => Some(side_property_pair(
            "margin",
            axis_mapping.inline_start,
            axis_mapping.inline_end,
        )),
        "padding-block" => Some(side_property_pair(
            "padding",
            axis_mapping.block_start,
            axis_mapping.block_end,
        )),
        "padding-inline" => Some(side_property_pair(
            "padding",
            axis_mapping.inline_start,
            axis_mapping.inline_end,
        )),
        "scroll-margin-block" => Some(side_property_pair(
            "scroll-margin",
            axis_mapping.block_start,
            axis_mapping.block_end,
        )),
        "scroll-margin-inline" => Some(side_property_pair(
            "scroll-margin",
            axis_mapping.inline_start,
            axis_mapping.inline_end,
        )),
        "scroll-padding-block" => Some(side_property_pair(
            "scroll-padding",
            axis_mapping.block_start,
            axis_mapping.block_end,
        )),
        "scroll-padding-inline" => Some(side_property_pair(
            "scroll-padding",
            axis_mapping.inline_start,
            axis_mapping.inline_end,
        )),
        "border-block-color" => Some(border_side_property_pair(
            axis_mapping.block_start,
            axis_mapping.block_end,
            Some("color"),
        )),
        "border-inline-color" => Some(border_side_property_pair(
            axis_mapping.inline_start,
            axis_mapping.inline_end,
            Some("color"),
        )),
        "border-block-style" => Some(border_side_property_pair(
            axis_mapping.block_start,
            axis_mapping.block_end,
            Some("style"),
        )),
        "border-inline-style" => Some(border_side_property_pair(
            axis_mapping.inline_start,
            axis_mapping.inline_end,
            Some("style"),
        )),
        "border-block-width" => Some(border_side_property_pair(
            axis_mapping.block_start,
            axis_mapping.block_end,
            Some("width"),
        )),
        "border-inline-width" => Some(border_side_property_pair(
            axis_mapping.inline_start,
            axis_mapping.inline_end,
            Some("width"),
        )),
        _ => None,
    }
}

fn physical_pair_properties_for_logical_mirror(
    property: &str,
    axis_mapping: LogicalAxisMapping,
) -> Option<(String, String)> {
    match property {
        "border-block" => Some(border_side_property_pair(
            axis_mapping.block_start,
            axis_mapping.block_end,
            None,
        )),
        "border-inline" => Some(border_side_property_pair(
            axis_mapping.inline_start,
            axis_mapping.inline_end,
            None,
        )),
        _ => None,
    }
}

fn side_pair(start: PhysicalSide, end: PhysicalSide) -> (String, String) {
    (start.name().to_string(), end.name().to_string())
}

fn side_property_pair(prefix: &str, start: PhysicalSide, end: PhysicalSide) -> (String, String) {
    (side_property(prefix, start), side_property(prefix, end))
}

fn side_property(prefix: &str, side: PhysicalSide) -> String {
    format!("{prefix}-{}", side.name())
}

fn border_side_property_pair(
    start: PhysicalSide,
    end: PhysicalSide,
    suffix: Option<&str>,
) -> (String, String) {
    (
        border_side_property(start, suffix),
        border_side_property(end, suffix),
    )
}

fn border_side_property(side: PhysicalSide, suffix: Option<&str>) -> String {
    match suffix {
        Some(suffix) => format!("border-{}-{suffix}", side.name()),
        None => format!("border-{}", side.name()),
    }
}

fn corner_radius_property(block_side: PhysicalSide, inline_side: PhysicalSide) -> Option<String> {
    let vertical_side = if block_side.is_vertical_edge() {
        block_side
    } else if inline_side.is_vertical_edge() {
        inline_side
    } else {
        return None;
    };
    let horizontal_side = if block_side.is_horizontal_edge() {
        block_side
    } else if inline_side.is_horizontal_edge() {
        inline_side
    } else {
        return None;
    };
    Some(format!(
        "border-{}-{}-radius",
        vertical_side.name(),
        horizontal_side.name()
    ))
}

fn logical_pair_values(value: &str) -> Option<(String, String)> {
    let components = split_top_level_whitespace_value_components(value)?;
    match components.as_slice() {
        [both] => Some((both.clone(), both.clone())),
        [start, end] => Some((start.clone(), end.clone())),
        _ => None,
    }
}

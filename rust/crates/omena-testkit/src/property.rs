use proptest::prelude::*;

const SCSS_MODULE_PATHS: &[&str] = &[
    "\"tokens\"",
    "\"./tokens\"",
    "\"../shared/theme\"",
    "\"sass:color\"",
    "\"sass:math\"",
];

const CSS_PROPERTY_NAMES: &[&str] = &[
    "color",
    "background",
    "margin",
    "padding",
    "border-color",
    "animation",
];

const SCSS_STATIC_VALUES: &[&str] = &[
    "red",
    "blue",
    "currentColor",
    "1rem",
    "calc(1rem + 2px)",
    "color-mix(in oklch, red, blue)",
];

const SCSS_NESTED_SUFFIXES: &[&str] = &["__item", "--active", ":hover"];

pub(crate) fn arb_scss() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            arb_scss_use_rule(),
            arb_scss_import_rule(),
            arb_scss_variable_declaration(),
            arb_scss_mixin_declaration(),
            arb_scss_style_rule(),
            arb_scss_nested_rule(),
            arb_scss_media_rule(),
        ],
        1..8,
    )
    .prop_map(|items| items.join("\n"))
}

fn arb_ident() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,8}"
}

fn arb_scss_module_path() -> impl Strategy<Value = String> {
    prop::sample::select(SCSS_MODULE_PATHS).prop_map(str::to_string)
}

fn arb_scss_use_rule() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_scss_module_path().prop_map(|path| format!("@use {path};")),
        (arb_scss_module_path(), arb_ident())
            .prop_map(|(path, namespace)| { format!("@use {path} as {namespace};") }),
    ]
}

fn arb_scss_import_rule() -> impl Strategy<Value = String> {
    arb_scss_module_path().prop_map(|path| format!("@import {path};"))
}

fn arb_scss_variable_name() -> impl Strategy<Value = String> {
    arb_ident().prop_map(|ident| format!("${ident}"))
}

fn arb_scss_variable_declaration() -> impl Strategy<Value = String> {
    (arb_scss_variable_name(), arb_scss_value())
        .prop_map(|(name, value)| format!("{name}: {value};"))
}

fn arb_css_property_name() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(CSS_PROPERTY_NAMES).prop_map(str::to_string),
        arb_ident().prop_map(|ident| format!("--{ident}")),
    ]
}

fn arb_scss_value() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(SCSS_STATIC_VALUES).prop_map(str::to_string),
        arb_scss_variable_name(),
        arb_ident().prop_map(|ident| format!("var(--{ident}, 1rem)")),
        (0u16..240).prop_map(|value| format!("{value}px")),
    ]
}

fn arb_scss_declaration() -> impl Strategy<Value = String> {
    (arb_css_property_name(), arb_scss_value())
        .prop_map(|(property, value)| format!("  {property}: {value};"))
}

fn arb_scss_declaration_list() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_scss_declaration(), 1..5)
}

fn arb_scss_selector() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_ident().prop_map(|ident| format!(".{ident}")),
        arb_ident().prop_map(|ident| format!("#{ident}")),
        (arb_ident(), arb_ident())
            .prop_map(|(element, class_name)| { format!("{element}.{class_name}") }),
        (arb_ident(), arb_ident())
            .prop_map(|(class_name, modifier)| { format!(".{class_name}--{modifier}") }),
    ]
}

fn arb_scss_style_rule() -> impl Strategy<Value = String> {
    (arb_scss_selector(), arb_scss_declaration_list()).prop_map(|(selector, declarations)| {
        format!("{selector} {{\n{}\n}}", declarations.join("\n"))
    })
}

fn arb_scss_nested_rule() -> impl Strategy<Value = String> {
    (
        arb_ident(),
        prop::sample::select(SCSS_NESTED_SUFFIXES).prop_map(str::to_string),
        arb_scss_declaration_list(),
    )
        .prop_map(|(class_name, suffix, declarations)| {
            format!(
                ".{class_name} {{\n  &{suffix} {{\n{}\n  }}\n}}",
                declarations.join("\n")
            )
        })
}

fn arb_scss_mixin_declaration() -> impl Strategy<Value = String> {
    (arb_ident(), arb_scss_declaration_list()).prop_map(|(name, declarations)| {
        format!("@mixin {name}($tone) {{\n{}\n}}", declarations.join("\n"))
    })
}

fn arb_scss_media_rule() -> impl Strategy<Value = String> {
    (arb_scss_selector(), arb_scss_declaration_list()).prop_map(|(selector, declarations)| {
        format!(
            "@media (min-width: 40rem) {{\n{selector} {{\n{}\n}}\n}}",
            declarations.join("\n")
        )
    })
}

proptest! {
    #[test]
    fn parser_does_not_panic(source in arb_scss()) {
        let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let parse_result = omena_parser::parse(&source, omena_parser::StyleDialect::Scss);
            let _ = parse_result.syntax();
            let _ = omena_parser::collect_style_facts(&source, omena_parser::StyleDialect::Scss);
        }));

        prop_assert!(parsed.is_ok(), "parser panicked for generated SCSS: {source:?}");
    }
}

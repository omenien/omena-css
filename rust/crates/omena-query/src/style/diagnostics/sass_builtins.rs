use std::collections::BTreeSet;

use omena_parser::{ParsedSassModuleEdgeFact, ParsedSassModuleEdgeFactKind};

pub(super) fn is_omena_query_sass_builtin_symbol_reference_resolved(
    edges: &[ParsedSassModuleEdgeFact],
    symbol_kind: &'static str,
    namespace: Option<&str>,
    name: &str,
) -> bool {
    edges
        .iter()
        .filter(|edge| edge.kind == ParsedSassModuleEdgeFactKind::Use)
        .filter_map(|edge| {
            sass_builtin_module_name(edge.source.as_str()).map(|module| (edge, module))
        })
        .any(|(edge, module)| {
            let namespace_matches =
                match (namespace, edge.namespace_kind, edge.namespace.as_deref()) {
                    (Some(reference_namespace), Some("default" | "alias"), Some(use_namespace)) => {
                        reference_namespace == use_namespace
                    }
                    (None, Some("wildcard"), _) => true,
                    _ => false,
                };
            namespace_matches && sass_builtin_module_has_symbol(module, symbol_kind, name)
        })
}

pub(super) fn sass_builtin_module_name(source: &str) -> Option<&str> {
    source.strip_prefix("sass:")
}

pub(super) fn builtin_sass_symbol_exports(module: &str) -> BTreeSet<(&'static str, String)> {
    let mut exports = BTreeSet::new();
    for name in sass_builtin_module_function_names(module) {
        exports.insert(("function", (*name).to_string()));
    }
    for name in sass_builtin_module_mixin_names(module) {
        exports.insert(("mixin", (*name).to_string()));
    }
    for name in sass_builtin_module_variable_names(module) {
        exports.insert(("variable", (*name).to_string()));
    }
    exports
}

fn sass_builtin_module_has_symbol(module: &str, symbol_kind: &'static str, name: &str) -> bool {
    match symbol_kind {
        "function" => sass_builtin_module_function_names(module).contains(&name),
        "mixin" => sass_builtin_module_mixin_names(module).contains(&name),
        "variable" => sass_builtin_module_variable_names(module).contains(&name),
        _ => false,
    }
}

pub(super) fn sass_builtin_module_function_names(module: &str) -> &'static [&'static str] {
    match module {
        "color" => &[
            "adjust",
            "alpha",
            "blue",
            "channel",
            "change",
            "complement",
            "desaturate",
            "fade-in",
            "fade-out",
            "grayscale",
            "green",
            "hsl",
            "hsla",
            "hue",
            "ie-hex-str",
            "invert",
            "is-legacy",
            "is-missing",
            "is-powerless",
            "lighten",
            "lightness",
            "mix",
            "opacify",
            "opacity",
            "red",
            "same",
            "saturate",
            "saturation",
            "scale",
            "space",
            "to-gamut",
            "to-space",
            "transparentize",
        ],
        "math" => &[
            "abs",
            "acos",
            "asin",
            "atan",
            "atan2",
            "ceil",
            "clamp",
            "compatible",
            "cos",
            "div",
            "floor",
            "hypot",
            "is-unitless",
            "log",
            "max",
            "min",
            "percentage",
            "pow",
            "random",
            "round",
            "sin",
            "sqrt",
            "tan",
            "unit",
        ],
        "list" => &[
            "append",
            "index",
            "is-bracketed",
            "join",
            "length",
            "separator",
            "set-nth",
            "slash",
            "nth",
            "zip",
        ],
        "map" => &[
            "deep-merge",
            "deep-remove",
            "get",
            "has-key",
            "keys",
            "merge",
            "remove",
            "set",
            "values",
        ],
        "string" => &[
            "index",
            "insert",
            "length",
            "quote",
            "slice",
            "split",
            "to-lower-case",
            "to-upper-case",
            "unique-id",
            "unquote",
        ],
        "selector" => &[
            "append",
            "extend",
            "is-superselector",
            "nest",
            "parse",
            "replace",
            "simple-selectors",
            "unify",
        ],
        "meta" => &[
            "accepts-content",
            "calc-args",
            "calc-name",
            "call",
            "content-exists",
            "feature-exists",
            "function-exists",
            "get-function",
            // `meta.get-mixin` is a real `sass:meta` function added in Sass 1.77.
            "get-mixin",
            "global-variable-exists",
            "inspect",
            "keywords",
            "mixin-exists",
            "module-functions",
            "module-mixins",
            "module-variables",
            "type-of",
            "variable-exists",
        ],
        _ => &[],
    }
}

pub(super) fn sass_builtin_module_mixin_names(module: &str) -> &'static [&'static str] {
    match module {
        // `meta.apply` is a real `sass:meta` mixin added in Sass 1.77.
        "meta" => &["apply", "load-css"],
        _ => &[],
    }
}

fn sass_builtin_module_variable_names(module: &str) -> &'static [&'static str] {
    match module {
        "math" => &["e", "epsilon", "max-safe-integer", "min-safe-integer", "pi"],
        _ => &[],
    }
}

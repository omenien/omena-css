use omena_value_lattice::{matching_function_call_end, split_top_level_value_arguments_owned};

pub(crate) fn reduce_static_scss_metadata_with_context<F, M, V, G>(
    value: &str,
    mut function_exists: F,
    mut mixin_exists: M,
    mut variable_exists: V,
    mut global_variable_exists: G,
) -> Option<String>
where
    F: FnMut(&str) -> Option<bool>,
    M: FnMut(&str) -> Option<bool>,
    V: FnMut(&str) -> Option<bool>,
    G: FnMut(&str) -> Option<bool>,
{
    let mut current = value.to_string();
    let mut changed = false;
    for _ in 0..8 {
        let Some(next) = reduce_static_scss_metadata_calls(
            current.as_str(),
            &mut function_exists,
            &mut mixin_exists,
            &mut variable_exists,
            &mut global_variable_exists,
        ) else {
            break;
        };
        if next == current {
            break;
        }
        current = next;
        changed = true;
    }
    changed.then_some(current)
}

fn reduce_static_scss_metadata_calls<F, M, V, G>(
    value: &str,
    function_exists: &mut F,
    mixin_exists: &mut M,
    variable_exists: &mut V,
    global_variable_exists: &mut G,
) -> Option<String>
where
    F: FnMut(&str) -> Option<bool>,
    M: FnMut(&str) -> Option<bool>,
    V: FnMut(&str) -> Option<bool>,
    G: FnMut(&str) -> Option<bool>,
{
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }

        let Some(call) = static_scss_metadata_call_at(value, index) else {
            index += ch.len_utf8();
            continue;
        };
        let left_paren_index = index + call.name.len();
        let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
            index += ch.len_utf8();
            continue;
        };
        let Some(call_value) = value.get(index..close_index + ')'.len_utf8()) else {
            index += ch.len_utf8();
            continue;
        };
        let Some(exists) = resolve_static_scss_metadata_call(
            call_value,
            call.kind,
            function_exists,
            mixin_exists,
            variable_exists,
            global_variable_exists,
        ) else {
            index += ch.len_utf8();
            continue;
        };
        output.push_str(&value[cursor..index]);
        output.push_str(if exists { "true" } else { "false" });
        index = close_index + ')'.len_utf8();
        cursor = index;
        changed = true;
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssMetadataKind {
    Function,
    Mixin,
    Variable,
    GlobalVariable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticScssMetadataCall<'a> {
    name: &'a str,
    kind: StaticScssMetadataKind,
}

fn static_scss_metadata_call_at(value: &str, index: usize) -> Option<StaticScssMetadataCall<'_>> {
    const NAMES: [(&str, StaticScssMetadataKind); 8] = [
        ("meta.function-exists", StaticScssMetadataKind::Function),
        ("function-exists", StaticScssMetadataKind::Function),
        ("meta.mixin-exists", StaticScssMetadataKind::Mixin),
        ("mixin-exists", StaticScssMetadataKind::Mixin),
        ("meta.variable-exists", StaticScssMetadataKind::Variable),
        ("variable-exists", StaticScssMetadataKind::Variable),
        (
            "meta.global-variable-exists",
            StaticScssMetadataKind::GlobalVariable,
        ),
        (
            "global-variable-exists",
            StaticScssMetadataKind::GlobalVariable,
        ),
    ];
    let tail = value.get(index..)?;
    NAMES.iter().find_map(|(name, kind)| {
        (static_scss_metadata_function_left_boundary(value, index)
            && tail.len() > name.len()
            && tail[..name.len()].eq_ignore_ascii_case(name)
            && tail[name.len()..].starts_with('('))
        .then_some(StaticScssMetadataCall { name, kind: *kind })
    })
}

fn static_scss_metadata_function_left_boundary(value: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    value
        .get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_none_or(|ch| !ch.is_ascii_alphanumeric() && !matches!(ch, '_' | '-' | '.'))
}

fn resolve_static_scss_metadata_call<F, M, V, G>(
    value: &str,
    kind: StaticScssMetadataKind,
    function_exists: &mut F,
    mixin_exists: &mut M,
    variable_exists: &mut V,
    global_variable_exists: &mut G,
) -> Option<bool>
where
    F: FnMut(&str) -> Option<bool>,
    M: FnMut(&str) -> Option<bool>,
    V: FnMut(&str) -> Option<bool>,
    G: FnMut(&str) -> Option<bool>,
{
    let trimmed = value.trim();
    let function_name = static_scss_metadata_function_name(trimmed, kind)?;
    let arguments = split_top_level_value_arguments_owned(
        trimmed
            .get(function_name.len()..)?
            .strip_prefix('(')?
            .strip_suffix(')')?,
    )?;
    if arguments.len() != 1 {
        return None;
    }
    let queried_name = parse_static_scss_metadata_name_argument(arguments[0].as_str())?;
    match kind {
        StaticScssMetadataKind::Function => {
            if let Some(exists) = function_exists(queried_name.as_str()) {
                return Some(exists);
            }
            if static_scss_known_global_builtin_function_exists(queried_name.as_str()) {
                return Some(true);
            }
        }
        StaticScssMetadataKind::Mixin => {
            if let Some(exists) = mixin_exists(queried_name.as_str()) {
                return Some(exists);
            }
        }
        StaticScssMetadataKind::Variable => {
            return variable_exists(queried_name.as_str());
        }
        StaticScssMetadataKind::GlobalVariable => {
            return global_variable_exists(queried_name.as_str());
        }
    }
    static_scss_metadata_name_is_safe(queried_name.as_str()).then_some(false)
}

fn static_scss_metadata_function_name(
    value: &str,
    kind: StaticScssMetadataKind,
) -> Option<&'static str> {
    [
        "meta.function-exists",
        "function-exists",
        "meta.mixin-exists",
        "mixin-exists",
        "meta.variable-exists",
        "variable-exists",
        "meta.global-variable-exists",
        "global-variable-exists",
    ]
    .into_iter()
    .find(|name| static_scss_value_starts_with_name(value, name))
    .filter(|name| {
        matches!(
            (kind, *name),
            (
                StaticScssMetadataKind::Function,
                "meta.function-exists" | "function-exists"
            ) | (
                StaticScssMetadataKind::Mixin,
                "meta.mixin-exists" | "mixin-exists"
            ) | (
                StaticScssMetadataKind::Variable,
                "meta.variable-exists" | "variable-exists"
            ) | (
                StaticScssMetadataKind::GlobalVariable,
                "meta.global-variable-exists" | "global-variable-exists"
            )
        )
    })
}

fn static_scss_value_starts_with_name(value: &str, name: &str) -> bool {
    value.len() > name.len()
        && value[..name.len()].eq_ignore_ascii_case(name)
        && value[name.len()..].starts_with('(')
}

fn parse_static_scss_metadata_name_argument(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() >= 2
        && matches!(value.as_bytes().first(), Some(b'"' | b'\''))
        && value.as_bytes().first() == value.as_bytes().last()
    {
        if value.get(1..value.len() - 1)?.contains('\\') {
            return None;
        }
        return Some(canonical_static_scss_function_name(
            value.get(1..value.len() - 1)?,
        ));
    }
    static_scss_metadata_name_is_safe(value).then(|| canonical_static_scss_metadata_name(value))
}

fn static_scss_metadata_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn canonical_static_scss_metadata_name(name: &str) -> String {
    name.trim().replace('_', "-")
}

fn canonical_static_scss_function_name(name: &str) -> String {
    canonical_static_scss_metadata_name(name)
}

fn static_scss_known_global_builtin_function_exists(name: &str) -> bool {
    matches!(
        canonical_static_scss_function_name(name).as_str(),
        "abs"
            | "adjust-color"
            | "adjust-hue"
            | "alpha"
            | "append"
            | "blue"
            | "call"
            | "ceil"
            | "comparable"
            | "complement"
            | "darken"
            | "desaturate"
            | "fade-in"
            | "fade-out"
            | "feature-exists"
            | "floor"
            | "function-exists"
            | "global-variable-exists"
            | "grayscale"
            | "green"
            | "hsl"
            | "hsla"
            | "ie-hex-str"
            | "if"
            | "index"
            | "inspect"
            | "invert"
            | "is-bracketed"
            | "is-superselector"
            | "join"
            | "keywords"
            | "length"
            | "lighten"
            | "list-separator"
            | "map-get"
            | "map-has-key"
            | "map-keys"
            | "map-merge"
            | "map-remove"
            | "map-values"
            | "max"
            | "min"
            | "mix"
            | "mixin-exists"
            | "nth"
            | "opacify"
            | "opacity"
            | "percentage"
            | "quote"
            | "random"
            | "red"
            | "rgb"
            | "rgba"
            | "round"
            | "saturate"
            | "scale-color"
            | "selector-append"
            | "selector-extend"
            | "selector-nest"
            | "selector-parse"
            | "selector-replace"
            | "selector-unify"
            | "set-nth"
            | "simple-selectors"
            | "str-index"
            | "str-insert"
            | "str-length"
            | "str-slice"
            | "to-lower-case"
            | "to-upper-case"
            | "transparentize"
            | "type-of"
            | "unique-id"
            | "unit"
            | "unitless"
            | "unquote"
            | "variable-exists"
            | "zip"
    )
}

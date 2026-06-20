use omena_value_lattice::parse_whole_function_value_arguments;

use super::collection::{
    canonical_static_scss_map_key, parse_static_scss_map_entries,
    static_scss_collection_member_is_static, static_scss_existing_nested_map_child_entries,
    static_scss_map_contains_key, static_scss_map_entry_index, static_scss_map_entry_value,
    static_scss_render_comma_list, static_scss_render_map_entries,
    static_scss_update_nested_map_entries,
};

pub(super) fn parse_static_scss_map_get_value(value: &str) -> Option<String> {
    parse_static_scss_map_get_value_with_name(value, "map-get")
}

pub(super) fn parse_static_scss_map_get_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_get_value_with_name(value, "map.get")
}

pub(super) fn parse_static_scss_map_has_key_value(value: &str) -> Option<String> {
    parse_static_scss_map_has_key_value_with_name(value, "map-has-key")
}

pub(super) fn parse_static_scss_map_has_key_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_has_key_value_with_name(value, "map.has-key")
}

pub(super) fn parse_static_scss_map_keys_value(value: &str) -> Option<String> {
    parse_static_scss_map_keys_value_with_name(value, "map-keys")
}

pub(super) fn parse_static_scss_map_keys_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_keys_value_with_name(value, "map.keys")
}

pub(super) fn parse_static_scss_map_values_value(value: &str) -> Option<String> {
    parse_static_scss_map_values_value_with_name(value, "map-values")
}

pub(super) fn parse_static_scss_map_values_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_values_value_with_name(value, "map.values")
}

pub(super) fn parse_static_scss_map_merge_value(value: &str) -> Option<String> {
    parse_static_scss_map_merge_value_with_name(value, "map-merge")
}

pub(super) fn parse_static_scss_map_merge_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_merge_value_with_name(value, "map.merge")
}

pub(super) fn parse_static_scss_map_deep_merge_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_deep_merge_value_with_name(value, "map.deep-merge")
}

pub(super) fn parse_static_scss_map_remove_value(value: &str) -> Option<String> {
    parse_static_scss_map_remove_value_with_name(value, "map-remove")
}

pub(super) fn parse_static_scss_map_remove_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_remove_value_with_name(value, "map.remove")
}

pub(super) fn parse_static_scss_map_deep_remove_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_deep_remove_value_with_name(value, "map.deep-remove")
}

pub(super) fn parse_static_scss_map_set_value(value: &str) -> Option<String> {
    parse_static_scss_map_set_value_with_name(value, "map.set")
}

fn parse_static_scss_map_get_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, keys @ ..] = arguments.as_slice() else {
        return None;
    };
    if keys.is_empty() {
        return None;
    }
    let mut current_map = map.trim().to_string();
    for (index, key) in keys.iter().enumerate() {
        let key = canonical_static_scss_map_key(key)?;
        let value = static_scss_map_entry_value(current_map.as_str(), key.as_str())?;
        if index + 1 == keys.len() {
            return Some(value);
        }
        current_map = value;
    }
    None
}

fn parse_static_scss_map_has_key_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, keys @ ..] = arguments.as_slice() else {
        return None;
    };
    if keys.is_empty() {
        return None;
    }
    let mut current_map = map.trim().to_string();
    for (index, key) in keys.iter().enumerate() {
        let key = canonical_static_scss_map_key(key)?;
        if index + 1 == keys.len() {
            return Some(
                static_scss_map_contains_key(current_map.as_str(), key.as_str()).to_string(),
            );
        }
        let Some(value) = static_scss_map_entry_value(current_map.as_str(), key.as_str()) else {
            return Some("false".to_string());
        };
        current_map = value;
    }
    None
}

fn parse_static_scss_map_keys_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map] = arguments.as_slice() else {
        return None;
    };
    let keys = parse_static_scss_map_entries(map)?
        .into_iter()
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    static_scss_render_comma_list(keys)
}

fn parse_static_scss_map_values_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map] = arguments.as_slice() else {
        return None;
    };
    let values = parse_static_scss_map_entries(map)?
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    static_scss_render_comma_list(values)
}

fn parse_static_scss_map_merge_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [left_map, merge_args @ ..] = arguments.as_slice() else {
        return None;
    };
    let (right_map, merge_path) = merge_args.split_last()?;
    let left_entries = parse_static_scss_map_entries(left_map)?;
    let right_entries = parse_static_scss_map_entries(right_map)?;
    let merged = if merge_path.is_empty() {
        static_scss_merge_map_entries(left_entries, right_entries)?
    } else {
        static_scss_update_nested_map_entries(left_entries, merge_path, |target_entries| {
            static_scss_merge_map_entries(target_entries, right_entries)
        })?
    };
    static_scss_render_map_entries(merged)
}

fn static_scss_merge_map_entries(
    mut left_entries: Vec<(String, String)>,
    right_entries: Vec<(String, String)>,
) -> Option<Vec<(String, String)>> {
    for (right_key, right_value) in right_entries {
        let right_canonical_key = canonical_static_scss_map_key(right_key.as_str())?;
        if let Some(index) =
            static_scss_map_entry_index(left_entries.as_slice(), right_canonical_key.as_str())?
        {
            left_entries[index].1 = right_value;
        } else {
            left_entries.push((right_key, right_value));
        }
    }
    Some(left_entries)
}

fn parse_static_scss_map_deep_merge_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [left_map, right_map] = arguments.as_slice() else {
        return None;
    };
    let merged = static_scss_deep_merge_map_entries(
        parse_static_scss_map_entries(left_map)?,
        parse_static_scss_map_entries(right_map)?,
    )?;
    static_scss_render_map_entries(merged)
}

fn static_scss_deep_merge_map_entries(
    mut left_entries: Vec<(String, String)>,
    right_entries: Vec<(String, String)>,
) -> Option<Vec<(String, String)>> {
    for (right_key, right_value) in right_entries {
        let right_canonical_key = canonical_static_scss_map_key(right_key.as_str())?;
        let merged_value = if let Some(index) =
            static_scss_map_entry_index(left_entries.as_slice(), right_canonical_key.as_str())?
        {
            match (
                parse_static_scss_map_entries(left_entries[index].1.as_str()),
                parse_static_scss_map_entries(right_value.as_str()),
            ) {
                (Some(left_child), Some(right_child)) => static_scss_render_map_entries(
                    static_scss_deep_merge_map_entries(left_child, right_child)?,
                )?,
                _ => right_value,
            }
        } else {
            right_value
        };
        if let Some(index) =
            static_scss_map_entry_index(left_entries.as_slice(), right_canonical_key.as_str())?
        {
            left_entries[index].1 = merged_value;
        } else {
            left_entries.push((right_key, merged_value));
        }
    }
    Some(left_entries)
}

fn parse_static_scss_map_remove_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, keys @ ..] = arguments.as_slice() else {
        return None;
    };
    if keys.is_empty() {
        return None;
    }
    let entries = static_scss_remove_map_entries(parse_static_scss_map_entries(map)?, keys)?;
    static_scss_render_map_entries(entries)
}

fn parse_static_scss_map_deep_remove_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, path @ ..] = arguments.as_slice() else {
        return None;
    };
    if path.is_empty() {
        return None;
    }
    let entries = static_scss_deep_remove_map_entries(parse_static_scss_map_entries(map)?, path)?;
    static_scss_render_map_entries(entries)
}

fn parse_static_scss_map_set_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, set_args @ ..] = arguments.as_slice() else {
        return None;
    };
    let [path_and_key @ .., value] = set_args else {
        return None;
    };
    let (key, set_path) = path_and_key.split_last()?;
    let mut entries = parse_static_scss_map_entries(map)?;
    if set_path.is_empty() {
        entries = static_scss_set_map_entry(entries, key, value)?;
    } else {
        entries = static_scss_update_nested_map_entries(entries, set_path, |target_entries| {
            static_scss_set_map_entry(target_entries, key, value)
        })?;
    }
    static_scss_render_map_entries(entries)
}

fn static_scss_set_map_entry(
    mut entries: Vec<(String, String)>,
    key: &str,
    value: &str,
) -> Option<Vec<(String, String)>> {
    let set_key = canonical_static_scss_map_key(key)?;
    if !static_scss_collection_member_is_static(value) {
        return None;
    }
    if let Some(index) = static_scss_map_entry_index(entries.as_slice(), set_key.as_str())? {
        entries[index].1 = value.trim().to_string();
    } else {
        entries.push((key.trim().to_string(), value.trim().to_string()));
    }
    Some(entries)
}

fn static_scss_remove_map_entries(
    entries: Vec<(String, String)>,
    keys: &[String],
) -> Option<Vec<(String, String)>> {
    let remove_keys = keys
        .iter()
        .map(|key| canonical_static_scss_map_key(key))
        .collect::<Option<Vec<_>>>()?;
    let mut retained_entries = Vec::new();
    for (key, value) in entries {
        let candidate_key = canonical_static_scss_map_key(key.as_str())?;
        if !remove_keys.contains(&candidate_key) {
            retained_entries.push((key, value));
        }
    }
    Some(retained_entries)
}

fn static_scss_deep_remove_map_entries(
    mut entries: Vec<(String, String)>,
    path: &[String],
) -> Option<Vec<(String, String)>> {
    let Some((key, remaining_path)) = path.split_first() else {
        return Some(entries);
    };
    let canonical_key = canonical_static_scss_map_key(key)?;
    if remaining_path.is_empty() {
        if let Some(index) =
            static_scss_map_entry_index(entries.as_slice(), canonical_key.as_str())?
        {
            entries.remove(index);
        }
        return Some(entries);
    }
    let Some(index) = static_scss_map_entry_index(entries.as_slice(), canonical_key.as_str())?
    else {
        return Some(entries);
    };
    let Some(child_entries) =
        static_scss_existing_nested_map_child_entries(entries[index].1.as_str())?
    else {
        return Some(entries);
    };
    let updated_child_entries = static_scss_deep_remove_map_entries(child_entries, remaining_path)?;
    entries[index].1 = static_scss_render_map_entries(updated_child_entries)?;
    Some(entries)
}

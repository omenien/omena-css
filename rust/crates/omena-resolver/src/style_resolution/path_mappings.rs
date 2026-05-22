use std::path::{Path, PathBuf};

use crate::types::{OmenaResolverBundlerPathAliasMappingV0, OmenaResolverTsconfigPathMappingV0};

use super::push_unique_pathbuf;

pub(super) fn tsconfig_style_module_base_candidates(
    source: &str,
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for (mapping, pattern_match) in
        best_tsconfig_path_mapping_matches(source, tsconfig_path_mappings)
    {
        for target_pattern in &mapping.target_patterns {
            let substituted_target =
                substitute_tsconfig_path_pattern(target_pattern, pattern_match);
            push_unique_pathbuf(
                &mut candidates,
                Path::new(&mapping.base_path).join(substituted_target),
            );
        }
    }
    candidates
}

pub(super) fn bundler_style_module_base_candidates(
    source: &str,
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for mapping in bundler_path_mappings {
        let Some(remaining) = match_bundler_path_alias_pattern(&mapping.pattern, source) else {
            continue;
        };
        let base_path = if remaining.is_empty() {
            PathBuf::from(&mapping.target_path)
        } else {
            Path::new(&mapping.target_path).join(remaining)
        };
        push_unique_pathbuf(&mut candidates, base_path);
        break;
    }
    candidates
}

pub(super) fn source_matches_bundler_path_mapping(
    source: &str,
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
) -> bool {
    bundler_path_mappings
        .iter()
        .any(|mapping| match_bundler_path_alias_pattern(&mapping.pattern, source).is_some())
}

fn match_bundler_path_alias_pattern<'source>(
    pattern: &str,
    source: &'source str,
) -> Option<&'source str> {
    if pattern.is_empty() {
        return None;
    }
    if let Some(exact_pattern) = pattern.strip_suffix('$') {
        return (source == exact_pattern).then_some("");
    }
    if source == pattern {
        return Some("");
    }
    let prefix = if pattern.ends_with('/') {
        pattern.to_string()
    } else {
        format!("{pattern}/")
    };
    source.strip_prefix(prefix.as_str())
}

fn best_tsconfig_path_mapping_matches<'mapping, 'source>(
    source: &'source str,
    tsconfig_path_mappings: &'mapping [OmenaResolverTsconfigPathMappingV0],
) -> Vec<(&'mapping OmenaResolverTsconfigPathMappingV0, &'source str)> {
    let mut matches = tsconfig_path_mappings
        .iter()
        .enumerate()
        .filter_map(|(index, mapping)| {
            let priority = tsconfig_path_mapping_priority(&mapping.pattern, source)?;
            let pattern_match = match_tsconfig_path_pattern(&mapping.pattern, source)?;
            Some((index, priority, mapping, pattern_match))
        })
        .collect::<Vec<_>>();
    // TypeScript resolves path mappings through the best pattern, not the
    // first matching entry, so a less-specific alias must not shadow it.
    matches.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let Some(best_priority) = matches.first().map(|(_, priority, _, _)| *priority) else {
        return Vec::new();
    };
    matches
        .into_iter()
        .take_while(|(_, priority, _, _)| *priority == best_priority)
        .map(|(_, _, mapping, pattern_match)| (mapping, pattern_match))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TsconfigPathMappingPriority {
    exact_rank: u8,
    prefix_len: usize,
    suffix_len: usize,
    pattern_len: usize,
}

fn tsconfig_path_mapping_priority(
    pattern: &str,
    source: &str,
) -> Option<TsconfigPathMappingPriority> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        if suffix.contains('*') || !source.starts_with(prefix) || !source.ends_with(suffix) {
            return None;
        }
        return Some(TsconfigPathMappingPriority {
            exact_rank: 0,
            prefix_len: prefix.len(),
            suffix_len: suffix.len(),
            pattern_len: pattern.len(),
        });
    }
    (pattern == source).then_some(TsconfigPathMappingPriority {
        exact_rank: 1,
        prefix_len: pattern.len(),
        suffix_len: 0,
        pattern_len: pattern.len(),
    })
}

pub(super) fn source_matches_tsconfig_path_mapping(
    source: &str,
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> bool {
    tsconfig_path_mappings
        .iter()
        .any(|mapping| match_tsconfig_path_pattern(&mapping.pattern, source).is_some())
}

fn match_tsconfig_path_pattern<'a>(pattern: &str, source: &'a str) -> Option<&'a str> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        if suffix.contains('*') || !source.starts_with(prefix) || !source.ends_with(suffix) {
            return None;
        }
        return Some(&source[prefix.len()..source.len() - suffix.len()]);
    }
    (pattern == source).then_some("")
}

fn substitute_tsconfig_path_pattern(target_pattern: &str, pattern_match: &str) -> String {
    if target_pattern.contains('*') {
        target_pattern.replace('*', pattern_match)
    } else {
        target_pattern.to_string()
    }
}

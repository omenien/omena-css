use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use omena_query::{
    OmenaQueryExternalSifInputV0, ParserPositionV0, ParserRangeV0,
    is_omena_query_sass_symbol_declaration_kind as is_sass_symbol_declaration_kind,
    omena_query_sass_symbol_kind_from_candidate_kind as sass_symbol_kind_from_candidate_kind,
    resolve_omena_query_sass_module_use_sources_for_candidate,
    summarize_omena_query_sass_module_sources,
};
use serde_json::{Value, json};

use crate::{
    LspShellState, normalize_path, resolve_lsp_style_uri_for_specifier,
    sass_forward_edges_for_document,
    state::{LspStyleHoverCandidate, LspTextDocumentState},
    style_hover_candidates_for_uri, style_text_for_uri, unapply_sass_forward_prefix,
};

pub(super) fn external_sif_forward_canonical_url_candidates(
    base_canonical_url: &str,
    source: &str,
) -> Vec<String> {
    let mut candidates = BTreeSet::from([source.to_string()]);
    if !source.starts_with("sass:")
        && !source.starts_with("http://")
        && !source.starts_with("https://")
        && !source.starts_with("file://")
        && !source.starts_with("pkg:")
        && let Some(base_file_path) = base_canonical_url.strip_prefix("file://")
    {
        let base_path = Path::new(base_file_path);
        let joined = if source.starts_with('/') {
            PathBuf::from(source)
        } else {
            base_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(source)
        };
        push_external_sif_file_uri_candidates(&mut candidates, joined.as_path());
    }
    candidates.into_iter().collect()
}

pub(super) fn external_sif_forward_visibility_allows(
    forward: &omena_sif::OmenaSifForwardExportV1,
    family: &'static str,
    name: &str,
) -> bool {
    let matches_filter = |filter_name: &String| {
        let exposed_name = apply_sass_forward_prefix(forward.prefix.as_deref(), name);
        filter_name == &exposed_name
            || filter_name.trim_start_matches('$') == exposed_name
            || (family != "variable" && filter_name.trim_start_matches('@') == exposed_name)
    };
    if !forward.show.is_empty() {
        return forward.show.iter().any(matches_filter);
    }
    !forward.hide.iter().any(matches_filter)
}

pub(super) fn external_sif_canonical_urls_match(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let Some(left_path) = external_sif_canonical_url_path(left) else {
        return false;
    };
    let Some(right_path) = external_sif_canonical_url_path(right) else {
        return false;
    };
    normalize_path(Path::new(left_path.as_str()).to_path_buf())
        == normalize_path(Path::new(right_path.as_str()).to_path_buf())
}

pub(super) fn sass_symbol_names_match(left: &str, right: &str) -> bool {
    fold_sass_symbol_name(left.trim_start_matches('$')) == fold_sass_symbol_name(right)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternalSifSassSymbolTarget {
    pub(crate) canonical_url: String,
    pub(crate) interface_hash: String,
    pub(crate) family: &'static str,
    pub(crate) name: String,
    pub(crate) value_repr: Option<String>,
}

pub(crate) fn external_sif_sass_symbol_target_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Option<ExternalSifSassSymbolTarget> {
    if is_sass_symbol_declaration_kind(candidate.kind) {
        return None;
    }
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind)?;
    let sources =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())?;
    let mut visiting = BTreeSet::new();
    for source in resolve_omena_query_sass_module_use_sources_for_candidate(
        &sources,
        candidate.namespace.as_deref(),
    ) {
        if let Some(target) = external_sif_sass_symbol_target_for_module_source(
            state,
            document,
            source.as_str(),
            family,
            candidate.name.as_str(),
            &mut visiting,
        ) {
            return Some(target);
        }
    }
    if candidate.namespace.is_some() {
        return None;
    }
    for forward_edge in sass_forward_edges_for_document(document) {
        let Some(private_candidate) =
            forward_edge.private_candidate_for_forwarded_public_candidate(candidate)
        else {
            continue;
        };
        if let Some(mut target) = external_sif_sass_symbol_target_for_module_source(
            state,
            document,
            forward_edge.source.as_str(),
            family,
            private_candidate.name.as_str(),
            &mut visiting,
        ) {
            target.name = candidate.name.clone();
            return Some(target);
        }
    }
    None
}

fn external_sif_sass_symbol_target_for_module_source(
    state: &LspShellState,
    document: &LspTextDocumentState,
    source: &str,
    family: &'static str,
    name: &str,
    visiting: &mut BTreeSet<String>,
) -> Option<ExternalSifSassSymbolTarget> {
    let external_sif = external_sif_for_module_source(state, document, source)?;
    external_sif_exported_sass_symbol_target(
        external_sif,
        state.resolution.external_sifs.as_slice(),
        family,
        name,
        visiting,
    )
}

fn external_sif_for_module_source<'a>(
    state: &'a LspShellState,
    document: &LspTextDocumentState,
    source: &str,
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    let find = |candidate: &str| {
        state.resolution.external_sifs.iter().find(|external_sif| {
            external_sif_canonical_urls_match(external_sif.canonical_url.as_str(), candidate)
                || external_sif_canonical_urls_match(
                    external_sif.sif.canonical_url.as_str(),
                    candidate,
                )
        })
    };
    // Raw-key match FIRST: non-relative specifiers are exactly how the
    // bridge keys its alias entries, and running them through the
    // filesystem resolver when no workspace target exists is the
    // resolver's expensive failure path (seconds of candidate
    // canonicalization for a lookup a map already answers).
    if let Some(external_sif) = find(source) {
        return Some(external_sif);
    }
    // Relative specifiers can match a SIF only under their RESOLVED url
    // (the bridge keys chain entries by file url); resolution here is the
    // cheap success path.
    if !source.starts_with('.') {
        return None;
    }
    let uri = resolve_lsp_style_uri_for_specifier(state, document, source)?;
    find(uri.as_str())
}

fn external_sif_exported_sass_symbol_target(
    external_sif: &OmenaQueryExternalSifInputV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    family: &'static str,
    name: &str,
    visiting: &mut BTreeSet<String>,
) -> Option<ExternalSifSassSymbolTarget> {
    if !visiting.insert(external_sif.sif.canonical_url.clone()) {
        return None;
    }
    let direct = external_sif_direct_sass_symbol_target(external_sif, family, name);
    if direct.is_some() {
        visiting.remove(external_sif.sif.canonical_url.as_str());
        return direct;
    }
    for forward in &external_sif.sif.exports.forwards {
        let Some(private_name) = unapply_sass_forward_prefix(forward.prefix.as_deref(), name)
        else {
            continue;
        };
        if !external_sif_forward_visibility_allows(forward, family, private_name.as_str()) {
            continue;
        }
        let Some(forwarded_sif) = external_sif_for_forward(external_sif, forward, external_sifs)
        else {
            continue;
        };
        if let Some(mut target) = external_sif_exported_sass_symbol_target(
            forwarded_sif,
            external_sifs,
            family,
            private_name.as_str(),
            visiting,
        ) {
            target.name = name.to_string();
            visiting.remove(external_sif.sif.canonical_url.as_str());
            return Some(target);
        }
    }
    visiting.remove(external_sif.sif.canonical_url.as_str());
    None
}

fn external_sif_direct_sass_symbol_target(
    external_sif: &OmenaQueryExternalSifInputV0,
    family: &'static str,
    name: &str,
) -> Option<ExternalSifSassSymbolTarget> {
    let (name, value_repr) = match family {
        "variable" => external_sif
            .sif
            .exports
            .variables
            .iter()
            .find(|variable| sass_symbol_names_match(variable.name.as_str(), name))
            .map(|variable| {
                (
                    variable.name.trim_start_matches('$').to_string(),
                    variable.value_repr.clone(),
                )
            })?,
        "mixin" => external_sif
            .sif
            .exports
            .mixins
            .iter()
            .find(|mixin| sass_symbol_names_match(mixin.name.as_str(), name))
            .map(|mixin| (mixin.name.clone(), None))?,
        "function" => external_sif
            .sif
            .exports
            .functions
            .iter()
            .find(|function| sass_symbol_names_match(function.name.as_str(), name))
            .map(|function| (function.name.clone(), None))?,
        _ => return None,
    };
    Some(ExternalSifSassSymbolTarget {
        canonical_url: external_sif.sif.canonical_url.clone(),
        interface_hash: external_sif
            .sif
            .fingerprints
            .interface_hash
            .as_str()
            .to_string(),
        family,
        name,
        value_repr,
    })
}

fn external_sif_for_forward<'a>(
    external_sif: &OmenaQueryExternalSifInputV0,
    forward: &omena_sif::OmenaSifForwardExportV1,
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    external_sif_forward_canonical_url_candidates(
        external_sif.sif.canonical_url.as_str(),
        forward.canonical_url.as_str(),
    )
    .into_iter()
    .find_map(|candidate| {
        external_sifs.iter().find(|input| {
            external_sif_canonical_urls_match(input.canonical_url.as_str(), candidate.as_str())
                || external_sif_canonical_urls_match(
                    input.sif.canonical_url.as_str(),
                    candidate.as_str(),
                )
        })
    })
}

pub(crate) fn external_sif_sass_symbol_definition_location(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Option<Value> {
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind)?;
    let sources =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())?;
    let mut visiting = BTreeSet::new();
    let mut target = None;
    for source in resolve_omena_query_sass_module_use_sources_for_candidate(
        &sources,
        candidate.namespace.as_deref(),
    ) {
        target = external_sif_sass_symbol_target_for_module_source(
            state,
            document,
            source.as_str(),
            family,
            candidate.name.as_str(),
            &mut visiting,
        );
        if target.is_some() {
            break;
        }
    }
    let target = target?;
    let range = external_sif_sass_symbol_definition_range(state, &target).or_else(|| {
        style_text_for_uri(state, target.canonical_url.as_str()).map(|_| {
            let start = ParserPositionV0 {
                line: 0,
                character: 0,
            };
            ParserRangeV0 { start, end: start }
        })
    })?;
    Some(json!({
        "uri": target.canonical_url,
        "range": range,
    }))
}

fn external_sif_sass_symbol_definition_range(
    state: &LspShellState,
    target: &ExternalSifSassSymbolTarget,
) -> Option<ParserRangeV0> {
    let (_, candidates) = style_hover_candidates_for_uri(state, target.canonical_url.as_str())?;
    candidates
        .into_iter()
        .find(|candidate| {
            is_sass_symbol_declaration_kind(candidate.kind)
                && sass_symbol_kind_from_candidate_kind(candidate.kind) == Some(target.family)
                && sass_symbol_names_match(candidate.name.as_str(), target.name.as_str())
        })
        .map(|candidate| candidate.range)
}

fn push_external_sif_file_uri_candidates(candidates: &mut BTreeSet<String>, path: &Path) {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some()
    {
        candidates.insert(format!(
            "file://{}",
            normalize_path(path.to_path_buf()).to_string_lossy()
        ));
        return;
    }
    for extension in ["scss", "sass", "css"] {
        let with_extension = path.with_extension(extension);
        candidates.insert(format!(
            "file://{}",
            normalize_path(with_extension).to_string_lossy()
        ));
        if let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) {
            let partial = path
                .with_file_name(format!("_{file_name}"))
                .with_extension(extension);
            candidates.insert(format!(
                "file://{}",
                normalize_path(partial).to_string_lossy()
            ));
        }
    }
}

fn apply_sass_forward_prefix(prefix: Option<&str>, name: &str) -> String {
    match prefix {
        Some(prefix) if prefix.contains('*') => prefix.replace('*', name),
        Some(prefix) => format!("{prefix}{name}"),
        None => name.to_string(),
    }
}

fn external_sif_canonical_url_path(canonical_url: &str) -> Option<String> {
    if let Some(path) = canonical_url.strip_prefix("file://") {
        return Some(path.to_string());
    }
    Path::new(canonical_url)
        .is_absolute()
        .then(|| canonical_url.to_string())
}

fn fold_sass_symbol_name(name: &str) -> String {
    name.replace('_', "-")
}

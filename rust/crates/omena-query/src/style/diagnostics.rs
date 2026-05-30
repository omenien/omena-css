use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedSassModuleEdgeFactKind,
    ParsedSelectorFactKind, ParsedVariableFactKind,
};

use super::cascade_checker::summarize_query_cascade_checker_diagnostics;
use super::diagnostic_suppressions::apply_omena_query_style_diagnostic_suppressions;
use super::parser_facade::collect_omena_query_omena_parser_style_facts_raw;
use super::*;

const LSP_DIAGNOSTIC_TAG_UNNECESSARY: u8 = 1;
const LSP_DIAGNOSTIC_TAG_DEPRECATED: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaQueryExternalModuleModeV0 {
    Ignored,
    Sif,
}

pub fn summarize_omena_query_missing_custom_property_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let declaration_names = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| candidate.name.as_str())
        .collect::<BTreeSet<_>>();
    if declaration_names.is_empty() {
        return Vec::new();
    }

    // `var(--x, fallback)` references cannot be "missing" in any observable way — the
    // fallback guarantees a value — so suppress the lint per-reference. The fallback fact
    // range and the candidate range both derive from the same parser byte span via
    // `parser_range_for_byte_span`, so matching on the rendered range scopes the suppression
    // to the exact `var()` argument (a nested fallback-less `var(--b)` in
    // `var(--a, var(--b))` stays a live candidate).
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let fallback_ranges = facts
        .variables
        .iter()
        .filter(|fact| {
            fact.kind == ParsedVariableFactKind::CustomPropertyReference && fact.has_fallback
        })
        .map(|fact| {
            let byte_span = ParserByteSpanV0 {
                start: u32::from(fact.range.start()) as usize,
                end: u32::from(fact.range.end()) as usize,
            };
            (
                fact.name.clone(),
                parser_range_for_byte_span(source, byte_span),
            )
        })
        .collect::<BTreeSet<_>>();

    let insertion_range = end_of_source_range(source);
    candidates
        .iter()
        .filter(|candidate| {
            candidate.kind == "customPropertyReference"
                && !declaration_names.contains(candidate.name.as_str())
                && !fallback_ranges.contains(&(candidate.name.clone(), candidate.range))
        })
        .map(|candidate| OmenaQueryStyleDiagnosticV0 {
            code: "missingCustomProperty",
            severity: "warning",
            provenance: vec![
                "omena-parser.custom-property-facts",
                "omena-query.style-diagnostics",
            ],
            range: candidate.range,
            message: format!(
                "CSS custom property '{}' not found in indexed style tokens.",
                candidate.name
            ),
            tags: Vec::new(),
            create_custom_property: Some(OmenaQueryCreateCustomPropertyActionV0 {
                uri: style_uri.to_string(),
                range: insertion_range,
                new_text: format!("\n\n:root {{\n  {}: ;\n}}\n", candidate.name),
                property_name: candidate.name.clone(),
            }),
        })
        .collect()
}

pub fn summarize_omena_query_cascade_aware_style_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let declarations_by_name = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| (candidate.name.as_str(), candidate.range))
        .collect::<BTreeMap<_, _>>();

    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let mut diagnostics =
        summarize_static_css_custom_property_fixed_point_from_source(source, dialect)
            .entries
            .into_iter()
            .filter(|entry| entry.guaranteed_invalid)
            .filter_map(|entry| {
                declarations_by_name
                    .get(entry.name.as_str())
                    .copied()
                    .map(|range| OmenaQueryStyleDiagnosticV0 {
                        code: "guaranteedInvalidCustomProperty",
                        severity: "warning",
                        provenance: vec![
                            "omena-transform-passes.custom-property-lfp",
                            "omena-query.cascade-aware-diagnostics",
                        ],
                        range,
                        message: format!(
                            "CSS custom property '{}' resolves to the guaranteed-invalid value.",
                            entry.name
                        ),
                        tags: Vec::new(),
                        create_custom_property: None,
                    })
            })
            .collect::<Vec<_>>();

    diagnostics.extend(summarize_query_cascade_checker_diagnostics(
        style_uri, source,
    ));

    diagnostics
}

pub fn summarize_omena_query_missing_keyframes_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let declared_keyframes = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
        .map(|animation| animation.name.clone())
        .collect::<BTreeSet<_>>();
    let mut emitted = BTreeSet::new();

    facts
        .animations
        .into_iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .filter(|animation| !declared_keyframes.contains(animation.name.as_str()))
        .filter_map(|animation| {
            let start: u32 = animation.range.start().into();
            let end: u32 = animation.range.end().into();
            let byte_span = ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            };
            if !emitted.insert((animation.name.clone(), byte_span.start, byte_span.end)) {
                return None;
            }
            Some((animation, parser_range_for_byte_span(source, byte_span)))
        })
        .map(|(animation, range)| OmenaQueryStyleDiagnosticV0 {
            code: "missingKeyframes",
            severity: "warning",
            provenance: vec![
                "omena-parser.animation-facts",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!("@keyframes '{}' not found in this file.", animation.name),
            tags: Vec::new(),
            create_custom_property: None,
        })
        .collect()
}

pub fn summarize_omena_query_missing_sass_symbol_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let mut declarations = BTreeSet::<SassSymbolKey>::new();
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for symbol in facts.sass_symbols {
        // Route the single-file key tuple through the `sass_symbol_key` chokepoint so the
        // hyphen/underscore fold (Sass treats `$a-b` and `$a_b` as the same identifier) is
        // applied here too, not only on the cross-file/workspace path. (#48)
        let key = sass_symbol_key(
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
        );
        if omena_query_sass_symbol_fact_kind_is_declaration(symbol.kind) {
            declarations.insert(key);
            continue;
        }
        if !omena_query_sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }
        if declarations.contains(&key) {
            continue;
        }
        if is_omena_query_sass_builtin_symbol_reference_resolved(
            &facts.sass_module_edges,
            symbol.symbol_kind,
            symbol.namespace.as_deref(),
            symbol.name.as_str(),
        ) {
            continue;
        }

        let start: u32 = symbol.range.start().into();
        let end: u32 = symbol.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
            byte_span.start,
            byte_span.end,
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingSassSymbol",
            severity: "warning",
            provenance: vec![
                "omena-parser.sass-symbol-facts",
                "omena-query.style-diagnostics",
            ],
            range: parser_range_for_byte_span(source, byte_span),
            message: format!(
                "{} not found in this file.",
                format_query_sass_symbol_label(symbol.symbol_kind, symbol.name.as_str())
            ),
            tags: Vec::new(),
            create_custom_property: None,
        });
    }

    diagnostics
}

pub fn summarize_omena_query_sass_import_deprecation_hints(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    if !matches!(
        dialect,
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    facts
        .sass_module_edges
        .into_iter()
        .filter(|edge| edge.kind == ParsedSassModuleEdgeFactKind::Import)
        // Sass deprecated `@import` only for Sass partials. CSS-form imports
        // (`url(...)`, `.css` targets, protocol/`//` URLs) are explicitly kept and
        // must NOT be flagged. Classify per-edge (each comma-peer target is its own
        // Import edge), so a partial that shares a multi-target statement with a CSS
        // import still warns. (RFC-0007 D1, #44)
        .filter(|edge| !sass_import_is_plain_css(edge.source.as_str()))
        .map(|edge| {
            let start: u32 = edge.range.start().into();
            let end: u32 = edge.range.end().into();
            OmenaQueryStyleDiagnosticV0 {
                code: "deprecatedSassImport",
                severity: "information",
                provenance: vec![
                    "omena-parser.sass-module-edges",
                    "omena-query.sass-import-deprecation-hints",
                ],
                range: parser_range_for_byte_span(
                    source,
                    ParserByteSpanV0 {
                        start: start as usize,
                        end: end as usize,
                    },
                ),
                message: "Sass @import is deprecated; prefer @use or @forward.".to_string(),
                tags: vec![LSP_DIAGNOSTIC_TAG_DEPRECATED],
                create_custom_property: None,
            }
        })
        .collect()
}

/// Classify an `@import` target as plain CSS, which Sass explicitly keeps (NOT
/// deprecated). Operates on the `source` already captured in the Import edge fact,
/// so cross-file resolution (the edge collector) is unaffected.
///
/// Detects the CSS-form imports that are recoverable from the edge fact alone:
/// - `url(...)` (unquoted url form, source retains the `url(` wrapper),
/// - a `.css` extension target,
/// - protocol (`scheme://`) or scheme-relative (`//host/...`) URLs.
///
/// Necessary-not-sufficient: the media-qualified form (`@import "foo" screen`) and
/// the quoted-url-without-`.css` form (`@import url("foo")`, whose `url(...)`
/// wrapper is lost during tokenization) are NOT distinguishable from a Sass partial
/// at the edge-fact level, so they are still treated as Sass-form imports here. Both
/// are noted as remaining in #44 (media-qualifier Ident not captured).
fn sass_import_is_plain_css(source: &str) -> bool {
    let trimmed = source.trim();
    let lower = trimmed.to_ascii_lowercase();
    // Unquoted `url(...)` form: the source still carries the `url(` prefix.
    if lower.starts_with("url(") {
        return true;
    }
    // Protocol (`https://`, `http://`, `data:`-less `scheme://`) and scheme-relative
    // (`//cdn.example/...`) URLs are always plain CSS.
    if lower.starts_with("//") || lower.contains("://") {
        return true;
    }
    // Explicit `.css` extension target.
    if lower.ends_with(".css") {
        return true;
    }
    false
}

pub fn summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs(
        target_style_path,
        style_sources,
        package_manifests,
        &[],
    )
}

fn summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), &entry.facts))
        .collect::<BTreeMap<_, _>>();
    let resolution =
        summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    let visible_symbols = collect_visible_sass_symbol_keys(
        target_style_path,
        &facts_by_path,
        &resolution,
        external_sifs,
    );
    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for symbol in facts.sass_symbols {
        if !omena_query_sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }
        let key = sass_symbol_key(
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
        );
        if visible_symbols.contains(&key) {
            continue;
        }
        if is_omena_query_sass_builtin_symbol_reference_resolved(
            &facts.sass_module_edges,
            symbol.symbol_kind,
            symbol.namespace.as_deref(),
            symbol.name.as_str(),
        ) {
            continue;
        }

        let start: u32 = symbol.range.start().into();
        let end: u32 = symbol.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
            byte_span.start,
            byte_span.end,
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingSassSymbol",
            severity: "warning",
            provenance: vec![
                "omena-parser.sass-symbol-facts",
                "omena-query.graph-aware-sass-diagnostics",
            ],
            range: parser_range_for_byte_span(target.style_source.as_str(), byte_span),
            message: format!(
                "{} not found in the visible Sass module graph.",
                format_query_sass_symbol_label(symbol.symbol_kind, symbol.name.as_str())
            ),
            tags: Vec::new(),
            create_custom_property: None,
        });
    }

    diagnostics
}

/// RFC-0007-E2 (#45): `@use`/`@forward` module cycles. dart-sass hard-errors on a module loop
/// (`a.scss: @use 'b'`; `b.scss: @use 'a'`) or a self-loop (`@use './self'`); omena was silent.
///
/// The cycle facts are ALREADY computed — `summarize_sass_module_cross_file_resolution` fills
/// `resolution.cycles` (with `cycle_detection_ready: true`), but no diagnostic ever read them.
/// This is pure last-mile consumer wiring: read the existing `cycles`, keep the ones whose path
/// includes the target file, and anchor one diagnostic per such cycle to the outgoing
/// `@use`/`@forward`/`@import` statement in the target that closes the loop.
///
/// Anchoring: each cycle `path` is a node list `[A, B, …, A]`; for the target `A` the next node is
/// the module it loads (`B`). We map back to the resolved edge `from == target && resolved == B`,
/// then to the parser fact carrying its source range, so the squiggle lands on the actual
/// `@use 'b'` statement rather than the whole file. A cycle where the target is not a participant
/// (only reachable *through* a cycle) emits nothing here — it is reported on the file that owns the
/// looping statement, so each cycle is surfaced exactly once per participating edge.
fn summarize_omena_query_sass_use_cycle_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution =
        summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    if resolution.cycles.is_empty() {
        return Vec::new();
    }

    // Parser facts for the target file: the resolution edges carry the loop topology but not source
    // ranges, so we re-derive the `@use`/`@forward`/`@import` statement span by matching the edge's
    // `source` text back to the fact that produced it.
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );

    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for cycle in &resolution.cycles {
        // The target participates iff it appears in the loop node list.
        if !cycle.path.iter().any(|node| node == target_style_path) {
            continue;
        }
        // `RawAllPaths` emits every rotation of the same loop (`[a, b, a]` and `[b, a, b]`), so
        // dedupe on a rotation-invariant key before emitting, otherwise `a <-> b` would surface
        // twice on `a.scss`. The repeated closing node is dropped first, then we key on the
        // lexicographically-smallest rotation of the node ring.
        let canonical_cycle = canonical_sass_module_cycle(&cycle.path);
        // The next node after the target in the loop is the module the target loads to close it.
        // A self-loop (`@use './self'`) has the target as both the current and next node.
        let Some(next_module) = cycle
            .path
            .windows(2)
            .find(|window| window[0] == target_style_path)
            .map(|window| window[1].clone())
        else {
            continue;
        };
        // Find the resolved edge target -> next_module to recover the `@use`/`@forward` source text.
        let Some(loop_edge) = resolution.edges.iter().find(|edge| {
            edge.from_style_path == target_style_path
                && edge.resolved_style_path.as_deref() == Some(next_module.as_str())
        }) else {
            continue;
        };
        // Map back to the parser fact carrying the statement range (match on source text + kind).
        let Some(fact) = target_facts.sass_module_edges.iter().find(|fact| {
            fact.source == loop_edge.source
                && parsed_sass_module_edge_fact_kind_matches(fact.kind, loop_edge.edge_kind)
        }) else {
            continue;
        };
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((byte_span.start, byte_span.end, canonical_cycle.clone())) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassUseCycle",
            severity: "error",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.sass-use-cycle-diagnostics",
            ],
            range: parser_range_for_byte_span(target.style_source.as_str(), byte_span),
            message: format!(
                "Sass module loop: {}. dart-sass rejects this as a hard error.",
                render_sass_module_cycle_from(&canonical_cycle, target_style_path)
            ),
            tags: Vec::new(),
            create_custom_property: None,
        });
    }

    diagnostics
}

fn parsed_sass_module_edge_fact_kind_matches(
    fact_kind: ParsedSassModuleEdgeFactKind,
    edge_kind: &str,
) -> bool {
    matches!(
        (fact_kind, edge_kind),
        (ParsedSassModuleEdgeFactKind::Use, "sassUse")
            | (ParsedSassModuleEdgeFactKind::Forward, "sassForward")
            | (ParsedSassModuleEdgeFactKind::Import, "sassImport")
    )
}

/// Reduce a cycle `path` (a node ring whose first and last entries repeat, e.g. `[a, b, a]`) to a
/// rotation-invariant key: drop the repeated closing node, then return the lexicographically
/// smallest rotation. Two rotations of the same loop (`[a, b, a]` / `[b, a, b]`) collapse to one
/// key, so each distinct loop is surfaced exactly once per anchoring edge. A self-loop `[a, a]`
/// reduces to `[a]`.
fn canonical_sass_module_cycle(path: &[String]) -> Vec<String> {
    let ring: &[String] = match path.split_last() {
        Some((last, head)) if Some(last) == path.first() && !head.is_empty() => head,
        _ => path,
    };
    if ring.is_empty() {
        return path.to_vec();
    }
    let len = ring.len();
    (0..len)
        .map(|offset| {
            (0..len)
                .map(|index| ring[(offset + index) % len].clone())
                .collect::<Vec<_>>()
        })
        .min()
        .unwrap_or_else(|| ring.to_vec())
}

/// Render a canonical cycle ring as a closed `start -> … -> start` path beginning at `start`, so
/// each participating file describes the loop from its own perspective. `start` is guaranteed to be
/// in the ring by the caller (the target participates in the cycle).
fn render_sass_module_cycle_from(canonical_cycle: &[String], start: &str) -> String {
    let len = canonical_cycle.len();
    let begin = canonical_cycle
        .iter()
        .position(|node| node == start)
        .unwrap_or(0);
    let mut ordered = (0..len)
        .map(|index| canonical_cycle[(begin + index) % len].clone())
        .collect::<Vec<_>>();
    // Re-close the ring so the loop reads `a -> b -> a` (or `a -> a` for a self-loop).
    ordered.push(canonical_cycle[begin].clone());
    ordered.join(" -> ")
}

pub fn summarize_omena_query_style_diagnostics_for_file(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    let mut diagnostics =
        summarize_omena_query_missing_custom_property_diagnostics(style_uri, source, candidates);
    diagnostics.extend(summarize_omena_query_cascade_aware_style_diagnostics(
        style_uri, source, candidates,
    ));
    diagnostics.extend(summarize_omena_query_missing_keyframes_diagnostics(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_sass_import_deprecation_hints(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_missing_sass_symbol_diagnostics(
        style_uri, source,
    ));
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut diagnostics);
    let mut summary = OmenaQueryStyleDiagnosticsForFileV0 {
        schema_version: "0",
        product: "omena-query.diagnostics-for-file",
        file_uri: style_uri.to_string(),
        file_kind: "style",
        diagnostic_count: diagnostics.len(),
        diagnostics,
        ready_surfaces: vec![
            "missingCustomPropertyDiagnostics",
            "cascadeAwareDiagnostics",
            "missingKeyframesDiagnostics",
            "sassImportDeprecationHints",
            "missingSassSymbolDiagnostics",
            "checkerProductDiagnosticGate",
        ],
    };
    apply_omena_query_style_diagnostic_suppressions(source, &mut summary);
    summary
}

/// RFC-0007-F (#46): single-file `style-diagnostics` (no `--source`) used to skip composes-target
/// validation entirely, so a bare invocation and one with any unrelated `--source` produced
/// different diagnostics for the same file. This variant augments the single-file summary with the
/// composes outcomes that are fully resolvable without cross-file context — only `composes: x`
/// (Local edges) against the file's own selectors. Global edges produce nothing and External edges
/// (`composes: x from './other'`) are deliberately left to the `--source`-backed workspace path, so
/// no false `missingComposedSelector`/`missingComposedModule` is invented for an unseen sibling.
pub fn summarize_omena_query_style_diagnostics_for_file_with_local_composes(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    let mut summary =
        summarize_omena_query_style_diagnostics_for_file(style_uri, source, candidates);
    let mut local_composes =
        summarize_omena_query_css_modules_local_composes_style_diagnostics(style_uri, source);
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut local_composes);
    if !local_composes.is_empty() {
        summary.diagnostics.extend(local_composes);
        push_omena_query_ready_surface(
            &mut summary.ready_surfaces,
            "cssModulesComposesResolutionDiagnostics",
        );
        // Re-run suppressions so the appended composes diagnostics honour the same inline directives.
        apply_omena_query_style_diagnostic_suppressions(source, &mut summary);
        summary.diagnostic_count = summary.diagnostics.len();
    }
    summary
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        OmenaQueryExternalModuleModeV0::Ignored,
    )
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        external_mode,
        &[],
    )
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    let target = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)?;
    let candidates =
        summarize_omena_query_style_hover_candidates(target_style_path, &target.style_source)?;
    let mut summary = summarize_omena_query_style_diagnostics_for_file(
        target_style_path,
        &target.style_source,
        candidates.candidates.as_slice(),
    );
    summary
        .diagnostics
        .retain(|diagnostic| diagnostic.code != "missingSassSymbol");
    summary.diagnostics.extend(
        summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs(
            target_style_path,
            style_sources,
            package_manifests,
            external_sifs,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_css_modules_resolution_style_diagnostics(
            target_style_path,
            &target.style_source,
            style_sources,
            package_manifests,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_sass_use_cycle_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            package_manifests,
        ),
    );
    summary
        .diagnostics
        .extend(summarize_omena_query_unused_selector_style_diagnostics(
            target_style_path,
            &target.style_source,
            style_sources,
            source_documents,
            package_manifests,
            classname_transform,
        ));
    summary.diagnostic_count = summary.diagnostics.len();
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "cssModulesComposesResolutionDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "cssModulesValueResolutionDiagnostics",
    );
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "unusedSelectorDiagnostics");
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "sassUseCycleDiagnostics");
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "graphAwareSassSymbolDiagnostics",
    );
    if external_mode == OmenaQueryExternalModuleModeV0::Sif {
        let top_any_external_symbol_ranges =
            collect_omena_query_external_top_any_sass_symbol_ranges(
                target_style_path,
                style_sources,
                package_manifests,
                external_sifs,
            );
        summary.diagnostics.retain(|diagnostic| {
            diagnostic.code != "missingSassSymbol"
                || !top_any_external_symbol_ranges.contains(&diagnostic.range)
        });
        summary
            .diagnostics
            .extend(summarize_omena_query_external_sif_boundary_diagnostics(
                target_style_path,
                style_sources,
                package_manifests,
                external_sifs,
            ));
        push_omena_query_ready_surface(
            &mut summary.ready_surfaces,
            "externalSifBoundaryDiagnostics",
        );
    }
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut summary.diagnostics);
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "checkerProductDiagnosticGate");
    apply_omena_query_style_diagnostic_suppressions(&target.style_source, &mut summary);
    summary.diagnostic_count = summary.diagnostics.len();
    Some(summary)
}

fn collect_omena_query_external_top_any_sass_symbol_ranges(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> BTreeSet<ParserRangeV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return BTreeSet::new();
    };
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution =
        summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    let top_any_namespaces = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "external")
        .filter(|edge| find_omena_query_external_sif(edge.source.as_str(), external_sifs).is_none())
        .filter_map(|edge| match edge.edge_kind {
            "sassUse"
                if edge.namespace_kind == Some("default")
                    || edge.namespace_kind == Some("alias") =>
            {
                edge.namespace.clone().map(Some)
            }
            "sassUse" if edge.namespace_kind == Some("wildcard") => Some(None),
            "sassImport" => Some(None),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if top_any_namespaces.is_empty() {
        return BTreeSet::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    facts
        .sass_symbols
        .into_iter()
        .filter(|symbol| omena_query_sass_symbol_fact_kind_is_reference(symbol.kind))
        .filter(|symbol| top_any_namespaces.contains(&symbol.namespace))
        .map(|symbol| {
            let start: u32 = symbol.range.start().into();
            let end: u32 = symbol.range.end().into();
            parser_range_for_byte_span(
                target.style_source.as_str(),
                ParserByteSpanV0 {
                    start: start as usize,
                    end: end as usize,
                },
            )
        })
        .collect()
}

fn summarize_omena_query_external_sif_boundary_diagnostics(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution =
        summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    let external_sources = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "external")
        .filter(|edge| find_omena_query_external_sif(edge.source.as_str(), external_sifs).is_none())
        .map(|edge| edge.source.as_str())
        .collect::<BTreeSet<_>>();
    if external_sources.is_empty() {
        return Vec::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let mut emitted = BTreeSet::new();
    facts
        .sass_module_edges
        .into_iter()
        .filter(|edge| external_sources.contains(edge.source.as_str()))
        .filter_map(|edge| {
            if !emitted.insert((edge.kind, edge.source.clone())) {
                return None;
            }
            let state = OmenaResolverBoundaryStateV0::missing(
                None,
                "SIF mode requires a local SIF artifact for this external Sass module",
            );
            let start: u32 = edge.range.start().into();
            let end: u32 = edge.range.end().into();
            Some(OmenaQueryStyleDiagnosticV0 {
                code: "missingExternalSif",
                severity: "warning",
                provenance: vec![
                    "omena-resolver.boundary-state",
                    "omena-query.external-sif-boundary-diagnostics",
                ],
                range: parser_range_for_byte_span(
                    target.style_source.as_str(),
                    ParserByteSpanV0 {
                        start: start as usize,
                        end: end as usize,
                    },
                ),
                message: format!(
                    "External Sass module '{}' is {} ({}); generate or provide a SIF artifact, or use --external ignored.",
                    edge.source, state.state_name, state.top_name
                ),
                tags: Vec::new(),
                create_custom_property: None,
            })
        })
        .collect()
}

type SassSymbolKey = (&'static str, Option<String>, String);

/// Fold the Sass-identifier name component so `_` and `-` compare equal.
///
/// Sass treats `$a-b` and `$a_b` (and likewise mixin/function names) as the *same*
/// identifier, so the symbol key must canonicalize the name before lookup; otherwise
/// a reference spelled `$ns-token` is flagged missing against a `$ns_token` definition
/// (and vice versa). Only the name is folded — the namespace (`@use` alias) is matched
/// elsewhere, and CSS custom properties (`--a-b` ≠ `--a_b`) never flow through this key
/// space, so they are untouched. (#48)
fn fold_sass_symbol_name(name: &str) -> String {
    name.replace('_', "-")
}

fn sass_symbol_key(
    symbol_kind: &'static str,
    namespace: Option<String>,
    name: String,
) -> SassSymbolKey {
    let folded = fold_sass_symbol_name(&name);
    (symbol_kind, namespace, folded)
}

fn collect_visible_sass_symbol_keys(
    target_style_path: &str,
    facts_by_path: &BTreeMap<&str, &OmenaQueryOmenaParserStyleFactsV0>,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> BTreeSet<SassSymbolKey> {
    let mut visible = BTreeSet::new();
    if let Some(facts) = facts_by_path.get(target_style_path) {
        visible.extend(
            own_sass_symbol_declaration_keys(facts)
                .into_iter()
                .map(|(symbol_kind, name)| sass_symbol_key(symbol_kind, None, name)),
        );
    }

    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
    {
        let exported = if let Some(module_name) = sass_builtin_module_name(edge.source.as_str()) {
            builtin_sass_symbol_exports(module_name)
        } else if edge.status == "resolved" {
            let mut visiting = BTreeSet::new();
            edge.resolved_style_path
                .as_deref()
                .map(|path| {
                    collect_exported_sass_symbol_keys(
                        path,
                        facts_by_path,
                        resolution,
                        external_sifs,
                        &mut visiting,
                    )
                })
                .unwrap_or_default()
        } else if edge.status == "external" {
            find_omena_query_external_sif(edge.source.as_str(), external_sifs)
                .map(|sif| collect_sif_exported_sass_symbol_keys(&sif.sif))
                .unwrap_or_default()
        } else {
            BTreeSet::new()
        };

        match edge.edge_kind {
            "sassUse"
                if edge.namespace_kind == Some("default")
                    || edge.namespace_kind == Some("alias") =>
            {
                if let Some(namespace) = edge.namespace.clone() {
                    visible.extend(exported.into_iter().map(|(symbol_kind, name)| {
                        sass_symbol_key(symbol_kind, Some(namespace.clone()), name)
                    }));
                }
            }
            "sassUse" if edge.namespace_kind == Some("wildcard") => {
                visible.extend(
                    exported
                        .into_iter()
                        .map(|(symbol_kind, name)| sass_symbol_key(symbol_kind, None, name)),
                );
            }
            "sassImport" => {
                visible.extend(
                    exported
                        .into_iter()
                        .map(|(symbol_kind, name)| sass_symbol_key(symbol_kind, None, name)),
                );
            }
            _ => {}
        }
    }

    visible
}

fn collect_exported_sass_symbol_keys(
    style_path: &str,
    facts_by_path: &BTreeMap<&str, &OmenaQueryOmenaParserStyleFactsV0>,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    visiting: &mut BTreeSet<String>,
) -> BTreeSet<(&'static str, String)> {
    if !visiting.insert(style_path.to_string()) {
        return BTreeSet::new();
    }

    let mut exported = facts_by_path
        .get(style_path)
        .map(|facts| own_sass_symbol_declaration_keys(facts))
        .unwrap_or_default();

    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == style_path)
        .filter(|edge| edge.edge_kind == "sassForward" || edge.edge_kind == "sassImport")
    {
        let module_exports =
            if let Some(module_name) = sass_builtin_module_name(edge.source.as_str()) {
                builtin_sass_symbol_exports(module_name)
            } else if edge.status == "resolved" {
                edge.resolved_style_path
                    .as_deref()
                    .map(|path| {
                        collect_exported_sass_symbol_keys(
                            path,
                            facts_by_path,
                            resolution,
                            external_sifs,
                            visiting,
                        )
                    })
                    .unwrap_or_default()
            } else if edge.status == "external" {
                find_omena_query_external_sif(edge.source.as_str(), external_sifs)
                    .map(|sif| collect_sif_exported_sass_symbol_keys(&sif.sif))
                    .unwrap_or_default()
            } else {
                BTreeSet::new()
            };

        for (symbol_kind, name) in module_exports {
            if !sass_forward_visibility_allows(edge, symbol_kind, name.as_str()) {
                continue;
            }
            let exported_name = if edge.edge_kind == "sassForward" {
                apply_sass_forward_prefix(edge.forward_prefix.as_deref(), name.as_str())
            } else {
                name
            };
            exported.insert((symbol_kind, exported_name));
        }
    }

    visiting.remove(style_path);
    exported
}

fn own_sass_symbol_declaration_keys(
    facts: &OmenaQueryOmenaParserStyleFactsV0,
) -> BTreeSet<(&'static str, String)> {
    facts
        .sass_symbol_facts
        .iter()
        .filter(|fact| is_omena_query_sass_symbol_declaration_kind(fact.kind))
        .map(|fact| (fact.symbol_kind, fact.name.clone()))
        .collect()
}

fn collect_sif_exported_sass_symbol_keys(
    sif: &omena_sif::OmenaSifV1,
) -> BTreeSet<(&'static str, String)> {
    let mut exported = BTreeSet::new();
    exported.extend(sif.exports.variables.iter().map(|variable| {
        (
            "variable",
            variable.name.trim_start_matches('$').to_string(),
        )
    }));
    exported.extend(
        sif.exports
            .mixins
            .iter()
            .map(|mixin| ("mixin", mixin.name.clone())),
    );
    exported.extend(
        sif.exports
            .functions
            .iter()
            .map(|function| ("function", function.name.clone())),
    );
    exported
}

fn find_omena_query_external_sif<'a>(
    canonical_url: &str,
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    external_sifs.iter().find(|input| {
        input.canonical_url == canonical_url || input.sif.canonical_url == canonical_url
    })
}

fn sass_forward_visibility_allows(
    edge: &OmenaQuerySassModuleEdgeResolutionV0,
    symbol_kind: &'static str,
    name: &str,
) -> bool {
    let prefixed = apply_sass_forward_prefix(edge.forward_prefix.as_deref(), name);
    let matches_filter = |filter_name: &String| {
        filter_name == name
            || filter_name == prefixed.as_str()
            || filter_name.trim_start_matches('$') == name
            || filter_name.trim_start_matches('$') == prefixed.as_str()
            || (symbol_kind != "variable" && filter_name.trim_start_matches('@') == name)
    };
    match edge.visibility_filter_kind {
        Some("show") => edge.visibility_filter_names.iter().any(matches_filter),
        Some("hide") => !edge.visibility_filter_names.iter().any(matches_filter),
        _ => true,
    }
}

fn apply_sass_forward_prefix(prefix: Option<&str>, name: &str) -> String {
    match prefix {
        Some(prefix) if prefix.contains('*') => prefix.replace('*', name),
        Some(prefix) => format!("{prefix}{name}"),
        None => name.to_string(),
    }
}

fn is_omena_query_sass_builtin_symbol_reference_resolved(
    edges: &[omena_parser::ParsedSassModuleEdgeFact],
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

fn sass_builtin_module_name(source: &str) -> Option<&str> {
    source.strip_prefix("sass:")
}

fn builtin_sass_symbol_exports(module: &str) -> BTreeSet<(&'static str, String)> {
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

fn sass_builtin_module_function_names(module: &str) -> &'static [&'static str] {
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
            // `meta.get-mixin` is a real `sass:meta` function added in Sass 1.77. (#44 D2)
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

fn sass_builtin_module_mixin_names(module: &str) -> &'static [&'static str] {
    match module {
        // `meta.apply` is a real `sass:meta` mixin added in Sass 1.77. (#44 D2)
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

/// RFC-0007-F (#46): composes-target validation restricted to outcomes that are fully resolvable
/// from a single file with no cross-file `--source` context. Only `composes: x` / `composes: x, y`
/// (Local edges) target the file's own selectors and can be checked here; `composes: x from global`
/// (Global edges) reference no concrete selector and produce nothing; `composes: x from './other'`
/// (External edges) require the sibling module's facts, which a single-file invocation does not have,
/// so they are deliberately skipped to avoid a false `missingComposedSelector`/`missingComposedModule`.
/// @value imports are always cross-file and are likewise excluded.
pub fn summarize_omena_query_css_modules_local_composes_style_diagnostics(
    target_style_path: &str,
    target_source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(target_source, dialect);
    let target_class_names = target_facts
        .selectors
        .iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .map(|selector| selector.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut diagnostics = Vec::new();

    for edge in target_facts.css_module_composes_edges {
        // Global edges resolve to no concrete selector; External edges need cross-file facts.
        // Both are outside the single-file-resolvable surface, so only Local edges are validated.
        if edge.kind != ParsedCssModuleComposesEdgeKind::Local {
            continue;
        }
        let start: u32 = edge.range.start().into();
        let end: u32 = edge.range.end().into();
        let range = parser_range_for_byte_span(
            target_source,
            ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            },
        );
        for target_name in edge.target_names {
            if target_class_names.contains(target_name.as_str()) {
                continue;
            }
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "missingComposedSelector",
                severity: "warning",
                provenance: vec![
                    "omena-parser.css-modules-composes-facts",
                    "omena-query.css-modules-resolution-diagnostics",
                ],
                range,
                message: format!(
                    "Selector '.{}' not found in this file for composes.",
                    target_name
                ),
                tags: Vec::new(),
                create_custom_property: None,
            });
        }
    }

    diagnostics
}

pub fn summarize_omena_query_css_modules_resolution_style_diagnostics(
    target_style_path: &str,
    target_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(target_source, dialect);
    let mut diagnostics = Vec::new();

    for edge in target_facts.css_module_composes_edges {
        if edge.kind == ParsedCssModuleComposesEdgeKind::Global {
            continue;
        }
        let start: u32 = edge.range.start().into();
        let end: u32 = edge.range.end().into();
        let range = parser_range_for_byte_span(
            target_source,
            ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            },
        );
        let target_style = if edge.kind == ParsedCssModuleComposesEdgeKind::External {
            let Some(source) = edge.import_source.as_deref() else {
                continue;
            };
            let Some(resolved_style_path) = resolve_style_module_source(
                target_style_path,
                source,
                &available_style_paths,
                package_manifests,
            ) else {
                diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                    code: "missingComposedModule",
                    severity: "warning",
                    provenance: vec![
                        "omena-parser.css-modules-composes-facts",
                        "omena-resolver.style-module-resolution",
                    ],
                    range,
                    message: format!("Cannot resolve composed CSS Module '{}'.", source),
                    tags: Vec::new(),
                    create_custom_property: None,
                });
                continue;
            };
            resolved_style_path
        } else {
            target_style_path.to_string()
        };
        let target_class_names = facts_by_path
            .get(target_style.as_str())
            .map(|facts| facts.class_selector_names.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();

        for target_name in edge.target_names {
            if target_class_names.contains(target_name.as_str()) {
                continue;
            }
            let message = if let Some(source) = edge.import_source.as_deref() {
                format!(
                    "Selector '.{}' not found in composed module '{}'.",
                    target_name, source
                )
            } else {
                format!(
                    "Selector '.{}' not found in this file for composes.",
                    target_name
                )
            };
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "missingComposedSelector",
                severity: "warning",
                provenance: vec![
                    "omena-parser.css-modules-composes-facts",
                    "omena-query.css-modules-resolution-diagnostics",
                ],
                range,
                message,
                tags: Vec::new(),
                create_custom_property: None,
            });
        }
    }

    let mut reported_missing_value_modules = BTreeSet::new();
    for edge in target_facts.css_module_value_import_edges {
        let start: u32 = edge.range.start().into();
        let end: u32 = edge.range.end().into();
        let range = parser_range_for_byte_span(
            target_source,
            ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            },
        );
        let Some(resolved_style_path) = resolve_style_module_source(
            target_style_path,
            &edge.import_source,
            &available_style_paths,
            package_manifests,
        ) else {
            if reported_missing_value_modules.insert(edge.import_source.clone()) {
                diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                    code: "missingValueModule",
                    severity: "warning",
                    provenance: vec![
                        "omena-parser.css-modules-value-facts",
                        "omena-resolver.style-module-resolution",
                    ],
                    range,
                    message: format!(
                        "Cannot resolve imported @value module '{}'.",
                        edge.import_source
                    ),
                    tags: Vec::new(),
                    create_custom_property: None,
                });
            }
            continue;
        };
        let target_value_names = facts_by_path
            .get(resolved_style_path.as_str())
            .map(|facts| facts.css_module_value_definition_names.as_slice())
            .unwrap_or(&[])
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if target_value_names.contains(edge.remote_name.as_str()) {
            continue;
        }
        let message = if edge.local_name == edge.remote_name {
            format!(
                "@value '{}' not found in '{}'.",
                edge.remote_name, edge.import_source
            )
        } else {
            format!(
                "@value '{}' not found in '{}' for local binding '{}'.",
                edge.remote_name, edge.import_source, edge.local_name
            )
        };
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingImportedValue",
            severity: "warning",
            provenance: vec![
                "omena-parser.css-modules-value-facts",
                "omena-query.css-modules-resolution-diagnostics",
            ],
            range,
            message,
            tags: Vec::new(),
            create_custom_property: None,
        });
    }

    diagnostics
}

pub fn summarize_omena_query_unused_selector_style_diagnostics(
    target_style_path: &str,
    target_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if source_documents.is_empty() {
        return Vec::new();
    }

    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let aliases_by_path = collect_classname_transform_aliases(&facts_by_path, classname_transform);
    let (mut used_selectors, unresolved_dynamic_usage, has_unresolved_style_import) =
        collect_omena_query_source_selector_usage_by_style(
            &available_style_paths,
            source_documents,
            package_manifests,
            &aliases_by_path,
        );
    if unresolved_dynamic_usage.contains(target_style_path) {
        return Vec::new();
    }
    // RFC-0007-J (#50): when a source document imports a style module via a specifier we cannot
    // resolve (e.g. a workspace alias `@/styles/a.module.scss` with no tsconfig/bundler path
    // mapping wired in), we do not know which module its `cx('foo')`/`styles.foo` references point
    // at — so we cannot prove any selector is unused. References/goto stay lenient with that
    // ambiguity; the negative assertion (`unusedSelector`) must be conservative to match, instead
    // of dimming every selector in the file. Treat such documents as "possibly using" and skip the
    // lint for this target rather than emitting a wall of false positives.
    if has_unresolved_style_import {
        return Vec::new();
    }

    let composes_graph = collect_css_modules_composes_adjacency(
        &facts_by_path,
        &available_style_paths,
        package_manifests,
    );
    propagate_omena_query_composes_usage(&composes_graph, &mut used_selectors);

    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(target_source, dialect);
    let used_in_target = used_selectors
        .get(target_style_path)
        .cloned()
        .unwrap_or_default();
    let mut emitted = BTreeSet::new();

    target_facts
        .selectors
        .into_iter()
        .filter(|selector| selector.kind == ParsedSelectorFactKind::Class)
        .filter(|selector| !used_in_target.contains(selector.name.as_str()))
        .filter_map(|selector| {
            let start: u32 = selector.range.start().into();
            let end: u32 = selector.range.end().into();
            if !emitted.insert(selector.name.clone()) {
                return None;
            }
            Some(OmenaQueryStyleDiagnosticV0 {
                code: "unusedSelector",
                severity: "hint",
                provenance: vec![
                    "omena-parser.selector-facts",
                    "omena-query.source-selector-usage",
                ],
                range: parser_range_for_byte_span(
                    target_source,
                    ParserByteSpanV0 {
                        start: start as usize,
                        end: end as usize,
                    },
                ),
                message: format!("Selector '.{}' is declared but never used.", selector.name),
                tags: vec![LSP_DIAGNOSTIC_TAG_UNNECESSARY],
                create_custom_property: None,
            })
        })
        .collect()
}

fn collect_omena_query_source_selector_usage_by_style(
    available_style_paths: &BTreeSet<&str>,
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    aliases_by_path: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
) -> (BTreeMap<String, BTreeSet<String>>, BTreeSet<String>, bool) {
    let mut used_selectors = BTreeMap::<String, BTreeSet<String>>::new();
    let mut unresolved_dynamic_usage = BTreeSet::<String>::new();
    // RFC-0007-J (#50): tracks whether any document imports a style-like specifier we failed to
    // resolve (an unwired workspace alias). Such a document's selector usages cannot be attributed
    // to a concrete module, so the caller treats the file as "possibly used" instead of dimming
    // every selector.
    let mut has_unresolved_style_import = false;

    for document in source_documents {
        let imports = summarize_omena_query_source_import_declarations_for_source_language(
            document.source_path.as_str(),
            &document.source_source,
            None,
        );
        let mut imported_style_bindings = Vec::new();
        let mut classnames_bind_bindings = Vec::new();
        for import in imports.imports {
            if import.specifier == "classnames/bind" {
                classnames_bind_bindings.push(import.binding);
                continue;
            }
            let Some(style_path) = resolve_style_module_source(
                &document.source_path,
                &import.specifier,
                available_style_paths,
                package_manifests,
            ) else {
                if specifier_targets_style_module(&import.specifier) {
                    has_unresolved_style_import = true;
                }
                continue;
            };
            imported_style_bindings.push(OmenaQuerySourceImportedStyleBindingV0 {
                binding: import.binding,
                style_uri: style_path,
            });
        }
        if imported_style_bindings.is_empty() {
            continue;
        }

        let index = summarize_omena_query_source_syntax_index_for_source_language(
            document.source_path.as_str(),
            &document.source_source,
            None,
            imported_style_bindings,
            classnames_bind_bindings,
        );
        for reference in index.selector_references {
            let Some(target_style_path) = reference.target_style_uri else {
                continue;
            };
            let Some(selector_name) = reference.selector_name.or_else(|| {
                source_reference_text_selector_name(&document.source_source, reference.byte_span)
            }) else {
                unresolved_dynamic_usage.insert(target_style_path);
                continue;
            };
            let used_for_style = used_selectors.entry(target_style_path.clone()).or_default();
            if let Some(canonical_names) = aliases_by_path
                .get(target_style_path.as_str())
                .and_then(|aliases| aliases.get(selector_name.as_str()))
            {
                used_for_style.extend(canonical_names.iter().cloned());
            } else {
                used_for_style.insert(selector_name);
            }
        }
    }

    (
        used_selectors,
        unresolved_dynamic_usage,
        has_unresolved_style_import,
    )
}

/// Whether an import specifier names a CSS-family style module (so failing to resolve it is a
/// style-resolution gap worth treating conservatively, RFC-0007-J #50) rather than an ordinary
/// JS/TS dependency. A query string or hash on the specifier (e.g. `?inline`) is ignored.
fn specifier_targets_style_module(specifier: &str) -> bool {
    let path = specifier
        .split(['?', '#'])
        .next()
        .unwrap_or(specifier)
        .to_ascii_lowercase();
    path.ends_with(".css")
        || path.ends_with(".scss")
        || path.ends_with(".sass")
        || path.ends_with(".less")
}

fn collect_classname_transform_aliases(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    classname_transform: Option<&str>,
) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
    let mut aliases_by_path = BTreeMap::<String, BTreeMap<String, BTreeSet<String>>>::new();
    for (style_path, facts) in facts_by_path {
        let aliases = aliases_by_path
            .entry((*style_path).to_string())
            .or_default();
        for selector_name in &facts.class_selector_names {
            for alias in classname_transform_aliases(selector_name.as_str(), classname_transform) {
                aliases
                    .entry(alias)
                    .or_default()
                    .insert(selector_name.clone());
            }
        }
    }
    aliases_by_path
}

fn classname_transform_aliases(name: &str, classname_transform: Option<&str>) -> Vec<String> {
    match classname_transform.unwrap_or("asIs") {
        "camelCase" => keep_original_plus_transformed(name, to_ascii_camel_case(name)),
        "camelCaseOnly" => vec![to_ascii_camel_case(name)],
        "dashes" => keep_original_plus_transformed(name, dashes_to_ascii_camel(name)),
        "dashesOnly" => vec![dashes_to_ascii_camel(name)],
        _ => vec![name.to_string()],
    }
}

fn keep_original_plus_transformed(name: &str, transformed: String) -> Vec<String> {
    if transformed == name {
        vec![name.to_string()]
    } else {
        vec![name.to_string(), transformed]
    }
}

fn dashes_to_ascii_camel(name: &str) -> String {
    transform_ascii_separated_name(name, |byte| byte == b'-')
}

fn to_ascii_camel_case(name: &str) -> String {
    transform_ascii_separated_name(name, |byte| byte == b'-' || byte == b'_' || byte == b' ')
}

fn transform_ascii_separated_name(name: &str, is_separator: impl Fn(u8) -> bool) -> String {
    let mut output = String::with_capacity(name.len());
    let mut capitalize_next = false;
    for byte in name.bytes() {
        if is_separator(byte) {
            capitalize_next = true;
            continue;
        }
        if capitalize_next {
            output.push((byte as char).to_ascii_uppercase());
            capitalize_next = false;
            continue;
        }
        output.push(byte as char);
    }
    output
}

fn propagate_omena_query_composes_usage(
    composes_graph: &BTreeMap<CssModulesComposesNode, BTreeSet<CssModulesComposesNode>>,
    used_selectors: &mut BTreeMap<String, BTreeSet<String>>,
) {
    let mut used_nodes = used_selectors
        .iter()
        .flat_map(|(style_path, selectors)| {
            selectors
                .iter()
                .map(|selector_name| CssModulesComposesNode {
                    style_path: style_path.clone(),
                    selector_name: selector_name.clone(),
                })
        })
        .collect::<BTreeSet<_>>();

    let mut changed = true;
    while changed {
        changed = false;
        for (owner, targets) in composes_graph {
            if !used_nodes.contains(owner) {
                continue;
            }
            for target in targets {
                if used_nodes.insert(target.clone()) {
                    used_selectors
                        .entry(target.style_path.clone())
                        .or_default()
                        .insert(target.selector_name.clone());
                    changed = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sass_import_is_plain_css_classifies_css_forms() {
        // CSS-form imports Sass explicitly KEEPS (must NOT be flagged).
        assert!(sass_import_is_plain_css("url(theme.css)"));
        assert!(sass_import_is_plain_css("url(theme.scss)")); // unquoted url is always CSS
        assert!(sass_import_is_plain_css("vendor.css"));
        assert!(sass_import_is_plain_css("VENDOR.CSS")); // case-insensitive
        assert!(sass_import_is_plain_css("//cdn.example/x.css"));
        assert!(sass_import_is_plain_css("https://x.com/y.css"));
        assert!(sass_import_is_plain_css("http://x.com/y")); // protocol URL, no .css
    }

    #[test]
    fn sass_import_is_plain_css_keeps_partials_flaggable() {
        // Over-correction guard: genuine Sass-form partials must STILL be classified
        // as Sass imports (i.e. NOT plain CSS), so the deprecation hint still fires.
        assert!(!sass_import_is_plain_css("partial"));
        assert!(!sass_import_is_plain_css("./legacy"));
        assert!(!sass_import_is_plain_css("foundation/buttons"));
        assert!(!sass_import_is_plain_css("legacy")); // bare partial name
    }

    /// Durable CI drift check: pin the full `sass:meta` module surface against a
    /// known-good Sass 1.77 member set. If a future edit adds or removes a member
    /// (or the upstream module surface changes and we update one site but not the
    /// pinned set), this fails loudly instead of silently rotting. (#44 D2)
    ///
    /// Functions and mixins are tracked separately because Sass distinguishes them
    /// (`meta.get-mixin` is a function returning a mixin reference; `meta.apply` is a
    /// mixin invoked via `@include`).
    #[test]
    fn sass_meta_allowlist_matches_pinned_1_77_surface() {
        let pinned_functions: BTreeSet<&str> = [
            "accepts-content",
            "calc-args",
            "calc-name",
            "call",
            "content-exists",
            "feature-exists",
            "function-exists",
            "get-function",
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
        ]
        .into_iter()
        .collect();
        let pinned_mixins: BTreeSet<&str> = ["apply", "load-css"].into_iter().collect();

        let actual_functions: BTreeSet<&str> = sass_builtin_module_function_names("meta")
            .iter()
            .copied()
            .collect();
        let actual_mixins: BTreeSet<&str> = sass_builtin_module_mixin_names("meta")
            .iter()
            .copied()
            .collect();

        assert_eq!(
            actual_functions, pinned_functions,
            "sass:meta function allowlist drifted from pinned Sass 1.77 surface; \
             update both the allowlist and this pinned set together"
        );
        assert_eq!(
            actual_mixins, pinned_mixins,
            "sass:meta mixin allowlist drifted from pinned Sass 1.77 surface; \
             update both the allowlist and this pinned set together"
        );
    }

    // RFC-0007-F (#46): single-file local-composes validation.

    #[test]
    fn local_composes_flags_real_same_file_typo() {
        // True positive: `composes: missing` references a class that does not exist in this
        // file. With no cross-file context, this is fully resolvable and MUST be flagged.
        let source = ".base { color: red; }\n.button { composes: missing; }\n";
        let diagnostics = summarize_omena_query_css_modules_local_composes_style_diagnostics(
            "/tmp/foo.module.scss",
            source,
        );
        assert_eq!(diagnostics.len(), 1, "expected one missingComposedSelector");
        assert_eq!(diagnostics[0].code, "missingComposedSelector");
        assert!(
            diagnostics[0].message.contains("not found in this file"),
            "message should reference same-file resolution, got: {}",
            diagnostics[0].message
        );
    }

    #[test]
    fn local_composes_keeps_resolvable_target_silent() {
        // A local composes target that DOES exist in the file must NOT be flagged.
        let source = ".base { color: red; }\n.button { composes: base; }\n";
        let diagnostics = summarize_omena_query_css_modules_local_composes_style_diagnostics(
            "/tmp/foo.module.scss",
            source,
        );
        assert!(
            diagnostics.is_empty(),
            "resolvable local composes target should not be flagged, got: {diagnostics:?}"
        );
    }

    #[test]
    fn local_composes_does_not_flag_external_target_without_source() {
        // Over-correction guard: `composes: x from './other'` is an External edge that needs the
        // sibling module's facts. In single-file mode we have no access to `./other`, so we must
        // NOT invent a missingComposedSelector/missingComposedModule for it.
        let source = ".button { composes: shared from './other.module.scss'; color: blue; }\n";
        let diagnostics = summarize_omena_query_css_modules_local_composes_style_diagnostics(
            "/tmp/foo.module.scss",
            source,
        );
        assert!(
            diagnostics.is_empty(),
            "external composes target must not be flagged without cross-file source, got: {diagnostics:?}"
        );
    }

    #[test]
    fn local_composes_does_not_flag_global_target() {
        // `composes: x from global` references no concrete selector and must produce nothing.
        let source = ".button { composes: someGlobal from global; color: blue; }\n";
        let diagnostics = summarize_omena_query_css_modules_local_composes_style_diagnostics(
            "/tmp/foo.module.scss",
            source,
        );
        assert!(
            diagnostics.is_empty(),
            "global composes target must not be flagged, got: {diagnostics:?}"
        );
    }

    #[test]
    fn single_file_summary_includes_local_composes_typo() -> Result<(), String> {
        // The CLI bare path (`style-diagnostics foo` with no --source) routes through this
        // wrapper. A real same-file composes typo must surface even without --source, closing
        // the invocation-mode inconsistency the issue describes.
        let style_uri = "/tmp/foo.module.scss";
        let source = ".base { color: red; }\n.button { composes: missing; }\n";
        let candidates = summarize_omena_query_style_hover_candidates(style_uri, source)
            .ok_or("hover candidates")?;
        let summary = summarize_omena_query_style_diagnostics_for_file_with_local_composes(
            style_uri,
            source,
            candidates.candidates.as_slice(),
        );
        assert!(
            summary
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missingComposedSelector"),
            "bare single-file summary should surface the local composes typo, got: {:?}",
            summary.diagnostics
        );
        assert_eq!(summary.diagnostic_count, summary.diagnostics.len());
        Ok(())
    }

    #[test]
    fn single_file_summary_does_not_flag_external_composes() -> Result<(), String> {
        // Over-correction guard at the wrapper level: a clean file whose only composes target is
        // cross-file must stay free of composes diagnostics in single-file mode.
        let style_uri = "/tmp/foo.module.scss";
        let source = ".button { composes: shared from './other.module.scss'; color: blue; }\n";
        let candidates = summarize_omena_query_style_hover_candidates(style_uri, source)
            .ok_or("hover candidates")?;
        let summary = summarize_omena_query_style_diagnostics_for_file_with_local_composes(
            style_uri,
            source,
            candidates.candidates.as_slice(),
        );
        assert!(
            !summary.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "missingComposedSelector"
                    || diagnostic.code == "missingComposedModule"
            }),
            "external composes target must not be flagged in single-file mode, got: {:?}",
            summary.diagnostics
        );
        Ok(())
    }
}

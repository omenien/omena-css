use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedExtendTargetFactKind,
    ParsedSassModuleEdgeFact, ParsedSassModuleEdgeFactKind, ParsedSelectorFactKind,
    ParsedVariableFactKind,
};
use omena_query_checker_orchestrator::{
    ModuleGraphEdgeV0, ModuleGraphV0, OutcomeMode,
    REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0,
    REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0, ReplicaSnapshotV0,
    ReportOptionsV0, ReportRecommendation, build_cross_file_inconsistency_report,
};
use omena_query_checker_orchestrator::{
    OmenaCheckerReplicaEnsembleInputV0, OmenaCheckerReplicaEnsembleReportInputV0,
    run_omena_query_checker_replica_ensemble_gate_v0,
};

use super::cascade_checker::collect_query_replica_ensemble_site_outcomes;
use super::cascade_checker::query_runtime_state_confidence_tier;
use super::cascade_checker::summarize_query_cascade_checker_diagnostics_with_deep_analysis;
use super::diagnostic_suppressions::OmenaStrictnessLevelV0;
use super::diagnostic_suppressions::apply_omena_query_style_diagnostic_suppressions;
use super::diagnostic_suppressions::parse_omena_query_style_strictness_level;
use super::diagnostic_suppressions::report_omena_query_style_diagnostic_suppressions;
use super::parser_facade::collect_omena_query_omena_parser_style_facts_raw;
use super::*;
use omena_resolver::{
    canonicalize_omena_resolver_style_identity_path,
    collect_omena_resolver_style_module_source_candidates_with_load_path_roots,
};

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
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        })
        .collect()
}

pub fn summarize_omena_query_cascade_aware_style_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    summarize_omena_query_cascade_aware_style_diagnostics_with_deep_analysis(
        style_uri, source, candidates, false,
    )
}

/// Cascade-aware diagnostics with an explicit opt-in deep-analysis switch. With
/// `deep_analysis == false` (the default surface) only the product cascade gate
/// diagnostics are emitted; `deep_analysis == true` additionally surfaces the
/// rg-flow / categorical theory hints, deduplicated against `circularVar`.
pub fn summarize_omena_query_cascade_aware_style_diagnostics_with_deep_analysis(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
    deep_analysis: bool,
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
                        cascade_narrowing: None,
                        cascade_confidence: None,
                        polynomial_provenance: None,
                        cross_file_scc: None,
                    })
            })
            .collect::<Vec<_>>();

    diagnostics.extend(
        summarize_query_cascade_checker_diagnostics_with_deep_analysis(
            style_uri,
            source,
            deep_analysis,
        ),
    );

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
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
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
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    diagnostics
}

/// RFC-0007-E1 (#45): `@extend` target validation. dart-sass hard-errors on `@extend %nonexistent`
/// / `@extend .missing` (`"%nonexistent" does not exist`); omena was silent because the
/// `ScssExtendRule` target was parsed and discarded. The parser now captures each target as a
/// `ParsedExtendTargetFact` (kind + name + `!optional` flag + range); this rule mirrors
/// `missingSassSymbol`'s file-local structure: an `@extend` target that does not resolve to a
/// declared placeholder/class **in this file** is flagged.
///
/// Scope and non-over-correction:
/// - `!optional` targets are NEVER flagged — dart-sass permits a missing optional extend, and
///   omena already (correctly) emitted nothing for them, so the flag is honored here.
/// - A placeholder target is checked only against declared placeholders; a class target only
///   against declared classes (Sass keeps the two namespaces distinct).
/// - This is file-local (single-file surface), like the `missingSassSymbol` companion. A target
///   declared in another file reachable via `@use`/`@forward`/`@import` is NOT yet validated here,
///   so cross-file `@extend` resolution is out of scope for v0 (recorded as remaining) — that keeps
///   the rule from inventing a false positive for a placeholder defined in an imported partial.
pub fn summarize_omena_query_missing_extend_target_diagnostics(
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
    if facts.extend_targets.is_empty() {
        return Vec::new();
    }

    let mut declared_placeholders = BTreeSet::new();
    let mut declared_classes = BTreeSet::new();
    for selector in &facts.selectors {
        match selector.kind {
            ParsedSelectorFactKind::Placeholder => {
                declared_placeholders.insert(selector.name.clone());
            }
            ParsedSelectorFactKind::Class => {
                declared_classes.insert(selector.name.clone());
            }
            ParsedSelectorFactKind::Id => {}
        }
    }

    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for target in &facts.extend_targets {
        // An optional extend (`@extend %x !optional`) is allowed to miss — never flag it.
        if target.optional {
            continue;
        }
        let (resolved, label) = match target.kind {
            ParsedExtendTargetFactKind::Placeholder => (
                declared_placeholders.contains(&target.name),
                format!("%{}", target.name),
            ),
            ParsedExtendTargetFactKind::Class => (
                declared_classes.contains(&target.name),
                format!(".{}", target.name),
            ),
        };
        if resolved {
            continue;
        }
        let start: u32 = target.range.start().into();
        let end: u32 = target.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((byte_span.start, byte_span.end)) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingExtendTarget",
            severity: "error",
            provenance: vec![
                "omena-parser.extend-target-facts",
                "omena-query.missing-extend-target-diagnostics",
            ],
            range: parser_range_for_byte_span(source, byte_span),
            message: format!(
                "@extend target '{label}' does not exist in this file. dart-sass rejects this as a hard error."
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    diagnostics
}

/// RFC-0007-E1 (#45) workspace variant: like the file-local rule, but a target is only flagged when
/// it is absent from the placeholders/classes declared in files **reachable from the target's
/// `@use`/`@forward`/`@import` import graph** (the target file plus its transitive module-graph
/// closure), so a cross-file `@extend` of a placeholder defined in an imported partial is never a
/// false positive — while an `@extend` of a placeholder that only exists in an UNRELATED,
/// non-imported file still fires (dart-sass only sees declarations the loaded modules bring into
/// scope, never the whole corpus). Optional extends are skipped.
fn summarize_omena_query_missing_extend_target_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let dialect = omena_parser_dialect_for_style_path(target_style_path);
    if !matches!(
        dialect,
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    let target_facts =
        collect_omena_query_omena_parser_style_facts_raw(target.style_source.as_str(), dialect);
    if target_facts.extend_targets.is_empty() {
        return Vec::new();
    }

    // Resolve the import graph so visibility tracks only the modules the target actually loads,
    // not the whole corpus. The substrate's RES-B slot (EMPTY manifests + bundler/tsconfig
    // mappings) is exactly the resolution this pass computed for itself;
    // `collect_sass_module_graph_reachable_style_paths` walks its edges from the target.
    // Bundler/tsconfig path mappings are threaded through (rfcs#59) so an `@use`/`@forward`/
    // `@import` behind an alias keeps its edge resolved — without them the aliased edge drops out
    // of the reachable set and a placeholder declared in the aliased partial false-positives.
    let resolution = &substrate.sass_resolution_without_manifests;
    let reachable_paths =
        collect_sass_module_graph_reachable_style_paths(target_style_path, resolution);

    // Declared placeholders/classes from the target plus every file reachable through its
    // `@use`/`@forward`/`@import` graph. A placeholder declared only in an unrelated, non-imported
    // file is NOT in this set, so an `@extend` of it correctly fires (matching dart-sass scope).
    let mut declared_placeholders = BTreeSet::new();
    let mut declared_classes = BTreeSet::new();
    for source in style_sources {
        if !reachable_paths.contains(source.style_path.as_str()) {
            continue;
        }
        let facts = collect_omena_query_omena_parser_style_facts_raw(
            source.style_source.as_str(),
            omena_parser_dialect_for_style_path(source.style_path.as_str()),
        );
        for selector in facts.selectors {
            match selector.kind {
                ParsedSelectorFactKind::Placeholder => {
                    declared_placeholders.insert(selector.name);
                }
                ParsedSelectorFactKind::Class => {
                    declared_classes.insert(selector.name);
                }
                ParsedSelectorFactKind::Id => {}
            }
        }
    }

    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for extend_target in &target_facts.extend_targets {
        if extend_target.optional {
            continue;
        }
        let (resolved, label) = match extend_target.kind {
            ParsedExtendTargetFactKind::Placeholder => (
                declared_placeholders.contains(&extend_target.name),
                format!("%{}", extend_target.name),
            ),
            ParsedExtendTargetFactKind::Class => (
                declared_classes.contains(&extend_target.name),
                format!(".{}", extend_target.name),
            ),
        };
        if resolved {
            continue;
        }
        let start: u32 = extend_target.range.start().into();
        let end: u32 = extend_target.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((byte_span.start, byte_span.end)) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingExtendTarget",
            severity: "error",
            provenance: vec![
                "omena-parser.extend-target-facts",
                "omena-query.missing-extend-target-diagnostics",
            ],
            range: parser_range_for_byte_span(target.style_source.as_str(), byte_span),
            message: format!(
                "@extend target '{label}' does not exist in the visible Sass module graph. dart-sass rejects this as a hard error."
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    diagnostics
}

/// RFC-0007-E1 (#45): the set of in-graph style paths reachable from `target_style_path` through
/// the resolved `@use`/`@forward`/`@import` edges (the target itself plus its transitive module-graph
/// closure). Used to scope cross-file `@extend` visibility to the modules the target actually loads
/// rather than the whole corpus, so a placeholder declared only in an unrelated file is not
/// (wrongly) treated as visible. Cycle-safe: each path is visited at most once.
pub(super) fn collect_sass_module_graph_reachable_style_paths<'a>(
    target_style_path: &'a str,
    resolution: &'a OmenaQuerySassModuleCrossFileResolutionV0,
) -> BTreeSet<&'a str> {
    let mut reachable = BTreeSet::new();
    let mut stack = vec![target_style_path];
    while let Some(current) = stack.pop() {
        if !reachable.insert(current) {
            continue;
        }
        for edge in resolution
            .edges
            .iter()
            .filter(|edge| edge.from_style_path == current && edge.status == "resolved")
        {
            if let Some(next) = edge.resolved_style_path.as_deref() {
                stack.push(next);
            }
        }
    }
    reachable
}

/// Surface the real cross-file replica-ensemble inconsistency diagnostic in the
/// workspace style path (#33 / L0 / L2).
///
/// When the target file's resolved `@use`/`@forward`/`@import` graph closure spans
/// two or more in-graph CSS modules, each module is treated as one *replica* of the
/// shared design surface and its REAL per-`(selector, property)` cascade winners are
/// extracted via `collect_query_replica_ensemble_site_outcomes` (genuine
/// `cascade_property` ranking over the parsed declarations — no fabricated
/// snapshots). `omena-ensemble`'s `build_cross_file_inconsistency_report` then
/// computes the replica overlap-Q distribution over the modules' winners and the
/// SBM detectability over the resolved module graph; the report's overlap statistics
/// (recommendation, `meanQ`, genuine disagreement-pair count) drive the registered
/// `replicaEnsembleInconsistency` checker rule through
/// `run_omena_query_checker_replica_ensemble_gate_v0`.
///
/// The diagnostic depends entirely on the overlap statistics over the real winners:
/// a workspace whose modules agree on every shared `(selector, property)` outcome
/// has `meanQ == 1.0`, zero disagreement pairs, and a `noActionNeeded`
/// recommendation, so the checker rule filters the report out and nothing is
/// surfaced; a workspace where two modules resolve a shared site to different
/// winning values drops `meanQ` below one and surfaces the diagnostic. The report is
/// whole-graph, so the single emitted diagnostic is anchored on the target file's
/// whole-file span.
fn summarize_omena_query_replica_ensemble_inconsistency_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };

    // Substrate RES-A slot: plain resolution with (package_manifests, bundler, tsconfig).
    let resolution = &substrate.sass_resolution;
    let reachable_paths =
        collect_sass_module_graph_reachable_style_paths(target_style_path, resolution);

    // A single-module closure is not an ensemble: there is no second replica to
    // overlap against, so there is no cross-file inconsistency to surface.
    if reachable_paths.len() < 2 {
        return Vec::new();
    }

    // Build one replica per in-graph module from its REAL cascade winners. A module
    // that declares no comparable definite cascade site contributes an empty replica
    // (it cannot agree or disagree with anything), so drop it from the ensemble.
    let replicas = style_sources
        .iter()
        .filter(|source| reachable_paths.contains(source.style_path.as_str()))
        .filter_map(|source| {
            let sites = collect_query_replica_ensemble_site_outcomes(source.style_source.as_str());
            if sites.is_empty() {
                return None;
            }
            Some(ReplicaSnapshotV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.replica-snapshot",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                path: source.style_path.clone(),
                sites,
            })
        })
        .collect::<Vec<_>>();

    // Fewer than two non-empty replicas => no shared cascade surface to compare.
    if replicas.len() < 2 {
        return Vec::new();
    }

    let module_graph = replica_ensemble_module_graph_from_resolution(
        target_style_path,
        resolution,
        &reachable_paths,
        &replicas,
    );
    let report = build_cross_file_inconsistency_report(
        target_style_path,
        replicas.clone(),
        &module_graph,
        OutcomeMode::DefiniteOnly,
        ReportOptionsV0::default(),
        None,
    );
    debug_assert_eq!(report.mechanism_scope, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0);
    debug_assert_eq!(report.product_surface, REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0);
    debug_assert_eq!(
        report.default_product_decision_mechanism,
        REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0
    );

    // Genuine disagreement count: only replica pairs whose computed overlap-Q is
    // strictly below 1.0 actually disagree on a shared cascade outcome. (The report's
    // `top_disagreement_pairs` keeps the lowest-Q pairs even when every pair fully
    // agrees, so counting that list directly would fire on a consistent ensemble.)
    let genuine_disagreement_pair_count = report
        .top_disagreement_pairs
        .iter()
        .filter(|pair| pair.shared_site_count > 0 && pair.overlap_q < 1.0)
        .count();
    let recommendation = replica_ensemble_recommendation_name(report.recommendation);

    let gate =
        run_omena_query_checker_replica_ensemble_gate_v0(OmenaCheckerReplicaEnsembleInputV0 {
            reports: vec![OmenaCheckerReplicaEnsembleReportInputV0 {
                workspace_root: target_style_path.to_string(),
                recommendation: recommendation.to_string(),
                mean_q: report.distribution.mean_q,
                variance_q: report.distribution.variance_q,
                top_disagreement_pair_count: genuine_disagreement_pair_count,
                mechanism_scope: report.mechanism_scope.to_string(),
                product_surface: report.product_surface.to_string(),
                default_product_decision_mechanism: report.default_product_decision_mechanism,
            }],
        });
    if !gate.enforcement_passed {
        return Vec::new();
    }

    let whole_file_range = parser_range_for_byte_span(
        target.style_source.as_str(),
        ParserByteSpanV0 {
            start: 0,
            end: target.style_source.len(),
        },
    );

    gate.evaluations
        .into_iter()
        .map(|evaluation| {
            let mut provenance = vec![
                "omena-query-checker-orchestrator.replica-ensemble-gate",
                "omena-checker.replica-ensemble-rules",
                "omena-ensemble.cross-file-inconsistency-report",
                "omena-query.cross-file-replica-ensemble",
            ];
            provenance.extend(evaluation.mechanism_products.iter().copied());
            OmenaQueryStyleDiagnosticV0 {
                code: "replicaEnsembleInconsistency",
                severity: "hint",
                provenance,
                range: whole_file_range,
                message: evaluation.message,
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            }
        })
        .collect()
}

/// Build the replica-ensemble module graph from the target's resolved import graph:
/// nodes are the in-graph modules that contributed a non-empty replica, and edges
/// are the resolved `@use`/`@forward`/`@import` edges between those modules. This is
/// the real dependency structure the SBM detectability reasons over — not a
/// synthesized clique.
fn replica_ensemble_module_graph_from_resolution(
    workspace_root: &str,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    reachable_paths: &BTreeSet<&str>,
    replicas: &[ReplicaSnapshotV0],
) -> ModuleGraphV0 {
    let nodes = replicas
        .iter()
        .map(|replica| replica.path.clone())
        .collect::<Vec<_>>();
    let node_set = nodes.iter().map(String::as_str).collect::<BTreeSet<_>>();

    let edges = resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "resolved")
        .filter(|edge| reachable_paths.contains(edge.from_style_path.as_str()))
        .filter_map(|edge| {
            let to = edge.resolved_style_path.as_deref()?;
            if node_set.contains(edge.from_style_path.as_str()) && node_set.contains(to) {
                Some(ModuleGraphEdgeV0 {
                    schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                    product: "omena-ensemble.module-graph-edge",
                    layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                    feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                    from_module: edge.from_style_path.clone(),
                    to_module: to.to_string(),
                    edge_kind: "resolvedModuleEdge",
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    ModuleGraphV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.module-graph",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        workspace_root: workspace_root.to_string(),
        nodes,
        edges,
    }
}

fn replica_ensemble_recommendation_name(recommendation: ReportRecommendation) -> &'static str {
    match recommendation {
        ReportRecommendation::NoActionNeeded => "noActionNeeded",
        ReportRecommendation::InvestigateRsbBroken => "investigateRsbBroken",
        ReportRecommendation::UndetectablePhase => "undetectablePhase",
    }
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
        // (`url(...)`, `.css` targets, protocol/`//` URLs, media-qualified targets)
        // are explicitly kept and must NOT be flagged. Classify per-edge (each
        // comma-peer target is its own Import edge), so a partial that shares a
        // multi-target statement with a CSS import still warns. (RFC-0007 D1, #44)
        .filter(|edge| !edge.media_qualified && !sass_import_is_plain_css(edge.source.as_str()))
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
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
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
/// The media-qualified form (`@import "foo" screen`) is now handled upstream via the
/// `media_qualified` flag on the Import edge (captured in the parser, where the
/// qualifier token is still available), so it is filtered out before this predicate
/// runs and does not need detecting here. (RFC-0007 D1, #44)
///
/// Necessary-not-sufficient: the quoted-url-without-`.css` form (`@import url("foo")`,
/// whose `url(...)` wrapper is lost during tokenization) remains NOT distinguishable
/// from a Sass partial at the edge-fact level, so it is still treated as a Sass-form
/// import here.
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
        &[],
        &[],
    )
}

fn summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if !style_sources
        .iter()
        .any(|source| source.style_path == target_style_path)
    {
        return Vec::new();
    }
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution = summarize_sass_module_cross_file_resolution_with_external_sifs(
        &style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        external_sifs,
    );
    summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs_from_substrate(
        target_style_path,
        style_sources,
        package_manifests,
        external_sifs,
        bundler_path_mappings,
        tsconfig_path_mappings,
        &style_fact_entries,
        &resolution,
    )
}

/// Substrate-threaded core of the graph-aware `missingSassSymbol` pass (RFC 0009 Pillar
/// B stage-2, #65). `style_fact_entries` is the ENTRIES slot and `resolution` the RES-D
/// slot (`summarize_sass_module_cross_file_resolution_with_external_sifs` over the same
/// arguments) — RES-D, not RES-A, because the slot keys on the actual `external_sifs`
/// argument: the monolith calls this pass unconditionally (even in `Ignored` mode), and
/// the two resolutions coincide only when `external_sifs` is empty.
#[allow(clippy::too_many_arguments)]
fn summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs_from_substrate(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), &entry.facts))
        .collect::<BTreeMap<_, _>>();
    let external_sif_context = OmenaQueryExternalSifResolutionContext {
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        external_sifs,
    };
    let visible_symbols = collect_visible_sass_symbol_keys(
        target_style_path,
        &facts_by_path,
        resolution,
        external_sif_context,
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
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
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
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    // Substrate RES-A slot: plain resolution with (package_manifests, bundler, tsconfig).
    let resolution = &substrate.sass_resolution;
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
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    diagnostics
}

#[cfg(feature = "hypergraph-ifds")]
fn summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace(
    target_style_path: &str,
    target_style_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    // Substrate slots ENTRIES + RES-E (css-modules resolution) + RES-C (plain Sass
    // resolution with EMPTY path mappings) — exactly what the workspace cross-file
    // summary collected for itself. The summary's source-selector leg still re-parses
    // `source_documents` internally (not covered by the substrate).
    let summary = super::cross_file_summary::summarize_omena_query_workspace_cross_file_summary_with_substrate(
        style_sources,
        source_documents,
        package_manifests,
        &substrate.style_fact_entries,
        &substrate.css_modules_resolution,
        &substrate.sass_resolution_without_path_mappings,
    );
    let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);
    let report = summarize_omena_query_unified_cross_file_scc_report(&hypergraph);
    if report.sccs.is_empty() {
        return Vec::new();
    }

    let whole_file_range = parser_range_for_byte_span(
        target_style_source,
        ParserByteSpanV0 {
            start: 0,
            end: target_style_source.len(),
        },
    );
    let mut emitted = BTreeSet::new();
    report
        .sccs
        .into_iter()
        .filter(|scc| scc.cross_file)
        .filter(|scc| scc.style_paths.iter().any(|path| path == target_style_path))
        .filter(|scc| {
            scc.edge_kinds
                .iter()
                .any(|edge_kind| edge_kind.starts_with("composes"))
        })
        .filter_map(|scc| {
            if !emitted.insert(scc.scc_id.clone()) {
                return None;
            }
            let style_path_count = scc.style_paths.len();
            let edge_kinds = scc.edge_kinds.join(", ");
            let cycle_paths = scc.style_paths.join(" -> ");
            Some(OmenaQueryStyleDiagnosticV0 {
                code: "crossFileStyleCycle",
                severity: "warning",
                provenance: vec![
                    "omena-query.unified-cross-file-scc-report",
                    "omena-query.unified-cross-file-hypergraph",
                    "omena-query.cross-file-summary",
                ],
                range: whole_file_range,
                message: format!(
                    "Cross-file style dependency cycle across {style_path_count} files via {edge_kinds}: {cycle_paths}"
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: Some(scc),
            })
        })
        .collect()
}

#[cfg(not(feature = "hypergraph-ifds"))]
fn summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace(
    _target_style_path: &str,
    _target_style_source: &str,
    _style_sources: &[OmenaQueryStyleSourceInputV0],
    _source_documents: &[OmenaQuerySourceDocumentInputV0],
    _package_manifests: &[OmenaQueryStylePackageManifestV0],
    _substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    Vec::new()
}

/// RFC-0007-E3 (#45): an unresolved Sass module reference to a **workspace-local** path.
/// dart-sass hard-errors on `@import './missing'` / `@use '../gone'` (file not found); omena was
/// silent — only `deprecatedSassImport` ever surfaced, and the `missingModule` rule existed only
/// for the JS/TS-imports-CSS-Modules direction.
///
/// The resolution facts are ALREADY computed: `summarize_sass_module_cross_file_resolution` marks
/// each edge `status == "unresolved"` (resolver kind `unresolved`), `"external"` (the resolver
/// kind `externalIgnored` for `sass:`/`http(s)://`), or `"resolved"`. We read the existing
/// `unresolved` edges and emit a `missingModule` diagnostic, but ONLY for relative/absolute
/// specifiers (`./`, `../`, `/`):
///
/// - A relative/absolute specifier is unambiguously a workspace-local file reference — it can never
///   be an `npm` package or a `sass:` builtin — so an unresolved one is a genuine file-not-found
///   error, matching dart-sass.
/// - A *bare* specifier (`'no-such-file'`, `'bootstrap'`) is left untouched: it is indistinguishable
///   at this layer from an external bare-package import that has no SIF in scope (the #32/#34
///   external-wiring known limitation, NOT an error in `Ignored` mode). Flagging it would regress
///   the external case, so bare unresolved partials stay deferred (reported as a remaining item).
/// - `status == "external"` edges (`sass:`/`http(s)://`) are never flagged.
///
/// Anchoring mirrors the use-cycle rule: re-derive the statement span from the parser
/// `sass_module_edges` fact whose `source` + kind match the resolution edge.
fn summarize_omena_query_unresolved_sass_import_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    // Substrate RES-C slot: plain resolution with (package_manifests) + EMPTY path
    // mappings — this pass never threaded bundler/tsconfig mappings, and substituting
    // the full-args RES-A would change the unresolved-edge set behind an alias.
    let resolution = &substrate.sass_resolution_without_path_mappings;

    let target_facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );

    unresolved_sass_import_diagnostics_from_edges(
        target_style_path,
        target.style_source.as_str(),
        &target_facts,
        resolution.edges.iter().filter_map(|edge| {
            (edge.from_style_path == target_style_path).then_some(
                ResolvedSassImportEdgeForDiagnostic {
                    source: edge.source.as_str(),
                    edge_kind: edge.edge_kind,
                    status: edge.status,
                },
            )
        }),
    )
}

/// RFC 0009 Pillar D (rfcs#70): target-only Baseline `missingModule`
/// derivation for the LSP fast path. It parses only the target document, but it
/// runs the same resolver policy as RES-C over the in-hand workspace style path
/// set and package manifests, then renders diagnostics through the shared
/// workspace emission helper above. That keeps the immediate Baseline publish
/// byte-aligned with the full workspace pass for the code family that cannot be
/// produced by the single-file diagnostics summary.
pub fn summarize_omena_query_target_unresolved_sass_import_diagnostics_for_workspace_paths(
    target_style_path: &str,
    target_style_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let load_path_roots = collect_load_path_roots(&available_style_paths);
    let load_path_root_refs = load_path_roots
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    let target_facts = collect_omena_query_omena_parser_style_facts_raw(
        target_style_source,
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let edge_resolutions = target_facts
        .sass_module_edges
        .iter()
        .map(|fact| {
            let edge_kind = parsed_sass_module_edge_fact_kind_label(fact.kind);
            let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
                target_style_path,
                fact.source.as_str(),
                &available_style_paths,
                &resolver_package_manifests,
                &[],
                &[],
                &load_path_root_refs,
            );
            let status = if resolution.resolution_kind == "externalIgnored" {
                "external"
            } else if resolution.resolved_style_path.is_some() {
                "resolved"
            } else {
                "unresolved"
            };
            ResolvedSassImportEdgeForDiagnostic {
                source: fact.source.as_str(),
                edge_kind,
                status,
            }
        })
        .collect::<Vec<_>>();

    let mut diagnostics = unresolved_sass_import_diagnostics_from_edges(
        target_style_path,
        target_style_source,
        &target_facts,
        edge_resolutions,
    );
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut diagnostics);
    diagnostics
}

#[derive(Debug, Clone, Copy)]
struct ResolvedSassImportEdgeForDiagnostic<'a> {
    source: &'a str,
    edge_kind: &'a str,
    status: &'a str,
}

fn unresolved_sass_import_diagnostics_from_edges<'a>(
    _target_style_path: &str,
    target_style_source: &str,
    target_facts: &omena_parser::ParsedStyleFacts,
    edges: impl IntoIterator<Item = ResolvedSassImportEdgeForDiagnostic<'a>>,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for edge in edges {
        if edge.status != "unresolved" || !sass_module_source_is_workspace_local(edge.source) {
            continue;
        }
        let Some(fact) = target_facts.sass_module_edges.iter().find(|fact| {
            fact.source == edge.source
                && parsed_sass_module_edge_fact_kind_matches(fact.kind, edge.edge_kind)
        }) else {
            continue;
        };
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if !emitted.insert((byte_span.start, byte_span.end)) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "missingModule",
            severity: "error",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.unresolved-sass-import-diagnostics",
            ],
            range: parser_range_for_byte_span(target_style_source, byte_span),
            message: format!(
                "Cannot resolve Sass module '{}'. dart-sass rejects this as a hard error.",
                edge.source
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    diagnostics
}

/// A Sass module specifier is workspace-local — and so a genuine file-not-found error when it does
/// not resolve — iff it is relative (`./`, `../`) or root-absolute (`/`). Bare specifiers
/// (`'partial'`, `'pkg'`) are excluded: they cannot be distinguished here from an external
/// bare-package import with no SIF in scope (RFC-0007-E3, #45).
fn sass_module_source_is_workspace_local(source: &str) -> bool {
    let trimmed = source.trim();
    trimmed.starts_with("./") || trimmed.starts_with("../") || trimmed.starts_with('/')
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

fn parsed_sass_module_edge_fact_kind_label(
    fact_kind: ParsedSassModuleEdgeFactKind,
) -> &'static str {
    match fact_kind {
        ParsedSassModuleEdgeFactKind::Use => "sassUse",
        ParsedSassModuleEdgeFactKind::Forward => "sassForward",
        ParsedSassModuleEdgeFactKind::Import => "sassImport",
    }
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
    summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
        style_uri, source, candidates, false,
    )
}

/// File-level diagnostics summary with an explicit opt-in deep-analysis switch.
/// `deep_analysis == false` (the default LSP/CLI surface) keeps only the product
/// cascade diagnostics; `deep_analysis == true` surfaces the rg-flow / categorical
/// theory hints, deduplicated against `circularVar`.
pub fn summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
    deep_analysis: bool,
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    let mut diagnostics =
        summarize_omena_query_missing_custom_property_diagnostics(style_uri, source, candidates);
    diagnostics.extend(
        summarize_omena_query_cascade_aware_style_diagnostics_with_deep_analysis(
            style_uri,
            source,
            candidates,
            deep_analysis,
        ),
    );
    diagnostics.extend(summarize_omena_query_missing_keyframes_diagnostics(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_sass_import_deprecation_hints(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_missing_sass_symbol_diagnostics(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_missing_extend_target_diagnostics(
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
            "missingExtendTargetDiagnostics",
            "checkerProductDiagnosticGate",
            "runtimeStateScenarioEvidence",
        ],
        suppression_summary: None,
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
    summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis(
        style_uri, source, candidates, false,
    )
}

/// Single-file (local composes) diagnostics summary with an explicit opt-in
/// deep-analysis switch. `deep_analysis == false` (the default surface) keeps only
/// the product cascade diagnostics; `deep_analysis == true` surfaces the rg-flow /
/// categorical theory hints, deduplicated against `circularVar`.
pub fn summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
    deep_analysis: bool,
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    let mut summary = summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
        style_uri,
        source,
        candidates,
        deep_analysis,
    );
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
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_suppression_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        external_mode,
        external_sifs,
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_suppression_mode(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        external_mode,
        external_sifs,
        &OmenaQueryStyleResolutionInputsV0 {
            package_manifests: package_manifests.to_vec(),
            ..Default::default()
        },
        suppression_mode,
    )
}

/// Workspace-file style diagnostics variant that additionally carries the workspace's
/// tsconfig/bundler path mappings. RFC-0007-J (#50): the unused-selector usage collector resolves
/// source-document style imports through these mappings so an alias import (`@/styles/a.module.scss`)
/// is attributed to its real module — matching the reference/goto path — instead of leaving every
/// selector dimmed `unusedSelector`. Path mappings only affect alias resolution; with empty mappings
/// the behaviour is byte-for-byte the no-mappings entry above.
#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        external_mode,
        external_sifs,
        resolution_inputs,
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
    )
}

/// RFC 0009 Pillar B stage-2 (#65): the per-call workspace diagnostics substrate.
///
/// Each workspace sub-pass used to rebuild the style fact entries (one full parse per
/// in-graph style source) and its own Sass cross-file resolution from the same corpus on
/// every monolith call. The substrate hoists those rebuilds to ONE collection per call.
///
/// The resolution variants are intentionally NOT collapsed into one slot: the sub-passes
/// call `summarize_sass_module_cross_file_resolution` with *different* argument shapes
/// (empty manifests, empty path mappings, SIF promotion), and substituting one variant
/// for another would change reachability / unresolved-edge sets — package manifests
/// enable bare-package and pkg-export edges, path mappings enable alias edges. Every
/// distinct `(entries, resolution-arguments)` value the monolith's sub-passes actually
/// compute is carried as its own slot, byte-identical to what the sub-pass would have
/// computed itself. The slots are built from the monolith's separate `package_manifests`
/// parameter (NOT `resolution_inputs.package_manifests`, which no sub-pass reads).
struct OmenaQueryWorkspaceDiagnosticsSubstrateV0 {
    /// ENTRIES: `collect_omena_query_style_fact_entries` over ALL
    /// `(style_path, style_source)` pairs in input order. Order-sensitive: the
    /// resolution derives per-kind `rule_ordinal`s from iteration order, so the corpus
    /// is never sorted/deduped/keyed here.
    style_fact_entries: Vec<OmenaQueryStyleFactEntry>,
    /// RES-A: plain resolution with `(package_manifests, bundler, tsconfig)`.
    /// Consumers: sass-use-cycle, resolution-identity, replica-ensemble, module-graph
    /// property-value narrowing.
    sass_resolution: OmenaQuerySassModuleCrossFileResolutionV0,
    /// RES-B: plain resolution with EMPTY manifests + `(bundler, tsconfig)`. Sole
    /// consumer: missing-extend-target. Equals RES-A only when `package_manifests` is
    /// empty, so it stays a separate slot.
    sass_resolution_without_manifests: OmenaQuerySassModuleCrossFileResolutionV0,
    /// RES-C: plain resolution with `(package_manifests)` + EMPTY path mappings.
    /// Consumers: unresolved-sass-import (always) and the unified-SCC pass's Sass leg
    /// (`hypergraph-ifds` builds only). Equals RES-A only when both mapping vecs are
    /// empty, so it stays a separate slot.
    sass_resolution_without_path_mappings: OmenaQuerySassModuleCrossFileResolutionV0,
    /// RES-D: SIF-promoted resolution (== RES-A + `promote_sif_backed_external_edges`;
    /// byte-identical to RES-A when `external_sifs` is empty since the promotion
    /// early-returns). Consumers: missing-sass-symbol (unconditional, even in `Ignored`
    /// mode), external top-any ranges + SIF boundary (Sif mode).
    sass_resolution_with_external_sifs: OmenaQuerySassModuleCrossFileResolutionV0,
    /// RES-E (`hypergraph-ifds` only): css-modules resolution with
    /// `(package_manifests)`, consumed by the unified-SCC pass via the workspace
    /// cross-file summary. Default builds run the empty SCC stub and never compute it.
    #[cfg(feature = "hypergraph-ifds")]
    css_modules_resolution: OmenaQueryCssModulesCrossFileResolutionV0,
}

fn collect_omena_query_workspace_diagnostics_substrate(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaQueryWorkspaceDiagnosticsSubstrateV0 {
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let sass_resolution = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    // RES-D is what `summarize_sass_module_cross_file_resolution_with_external_sifs`
    // returns for the same arguments — and that function is exactly "plain resolution
    // (RES-A's arguments) + in-place `promote_sif_backed_external_edges`" — so it is
    // derived from a clone of RES-A instead of re-running the plain resolution again.
    let mut sass_resolution_with_external_sifs = sass_resolution.clone();
    promote_sif_backed_external_edges(
        &mut sass_resolution_with_external_sifs,
        OmenaQueryExternalSifResolutionContext {
            package_manifests,
            bundler_path_mappings,
            tsconfig_path_mappings,
            external_sifs,
        },
    );
    let sass_resolution_without_manifests = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        &[],
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    let sass_resolution_without_path_mappings = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        &[],
        &[],
    );
    #[cfg(feature = "hypergraph-ifds")]
    let css_modules_resolution =
        summarize_css_modules_cross_file_resolution(&style_fact_entries, package_manifests);
    OmenaQueryWorkspaceDiagnosticsSubstrateV0 {
        style_fact_entries,
        sass_resolution,
        sass_resolution_without_manifests,
        sass_resolution_without_path_mappings,
        sass_resolution_with_external_sifs,
        #[cfg(feature = "hypergraph-ifds")]
        css_modules_resolution,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
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
    // RFC-0007-E1 (#45): the file-local `missingExtendTarget` rule cannot see a placeholder/class
    // declared in another in-graph file reachable via `@use`/`@forward`/`@import`, so it would
    // false-positive on a cross-file `@extend`. In workspace mode we drop the file-local result and
    // re-emit only those whose target is also absent from EVERY other in-graph style source — a
    // conservative cross-file-aware pass that keeps a genuinely-missing target flagged while never
    // inventing a false positive for a target defined in an imported partial. (Single-file surface
    // keeps the file-local rule unchanged.)
    summary
        .diagnostics
        .retain(|diagnostic| diagnostic.code != "missingExtendTarget");
    // RFC 0009 Pillar B stage-2 (#65): one shared substrate per monolith call. Every
    // workspace sub-pass below receives the precomputed (entries, resolution-variant)
    // slot it would otherwise have rebuilt from the same corpus itself.
    let substrate = collect_omena_query_workspace_diagnostics_substrate(
        style_sources,
        package_manifests,
        external_sifs,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    summary.diagnostics.extend(
        summarize_omena_query_missing_extend_target_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            &substrate,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs_from_substrate(
            target_style_path,
            style_sources,
            package_manifests,
            external_sifs,
            resolution_inputs.bundler_path_mappings.as_slice(),
            resolution_inputs.tsconfig_path_mappings.as_slice(),
            &substrate.style_fact_entries,
            &substrate.sass_resolution_with_external_sifs,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_css_modules_resolution_style_diagnostics_from_entries(
            target_style_path,
            &target.style_source,
            &substrate.style_fact_entries,
            package_manifests,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_sass_use_cycle_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            &substrate,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace(
            target_style_path,
            &target.style_source,
            style_sources,
            source_documents,
            package_manifests,
            &substrate,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_unresolved_sass_import_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            &substrate,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
            target_style_path,
            style_sources,
            &substrate.sass_resolution,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings_from_entries(
            target_style_path,
            &target.style_source,
            &substrate.style_fact_entries,
            source_documents,
            package_manifests,
            classname_transform,
            resolution_inputs.bundler_path_mappings.as_slice(),
            resolution_inputs.tsconfig_path_mappings.as_slice(),
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_replica_ensemble_inconsistency_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            &substrate,
        ),
    );
    attach_omena_query_module_graph_property_value_narrowing_for_workspace(
        target_style_path,
        &mut summary,
        style_sources,
        &substrate,
    );
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
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "crossFileSccDiagnostics");
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "unresolvedSassImportDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "sassModuleResolutionIdentityDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "missingExtendTargetDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "graphAwareSassSymbolDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "crossFileReplicaEnsembleDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "crossFileReplicaEnsembleHintScope",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "moduleGraphPropertyValueNarrowing",
    );
    if external_mode == OmenaQueryExternalModuleModeV0::Sif {
        // RFC 0004 #28 / #35: the file-scoped `@omena-strict: <level>` sigil dials the
        // external-boundary lattice behaviour. Absent/malformed sigil => `Standard`, which
        // keeps every branch below a no-op (byte-for-byte identical to the un-sigiled flow).
        let strictness = parse_omena_query_style_strictness_level(&target.style_source);
        let top_any_external_symbol_ranges =
            collect_omena_query_external_top_any_sass_symbol_ranges(
                target_style_path,
                style_sources,
                package_manifests,
                resolution_inputs.bundler_path_mappings.as_slice(),
                resolution_inputs.tsconfig_path_mappings.as_slice(),
                external_sifs,
                &substrate,
            );
        if strictness.suppresses_top_any_external_symbols() {
            summary.diagnostics.retain(|diagnostic| {
                diagnostic.code != "missingSassSymbol"
                    || !top_any_external_symbol_ranges.contains(&diagnostic.range)
            });
        } else {
            // `Closed` (#35): `TopOpaque` everywhere — genuinely-unknown external symbols are no
            // longer suppressed and are escalated to `error` rather than left as warnings.
            for diagnostic in summary.diagnostics.iter_mut() {
                if diagnostic.code == "missingSassSymbol"
                    && top_any_external_symbol_ranges.contains(&diagnostic.range)
                {
                    diagnostic.severity = "error";
                }
            }
        }
        if strictness.emits_external_boundary_diagnostics() {
            summary
                .diagnostics
                .extend(summarize_omena_query_external_sif_boundary_diagnostics(
                    target_style_path,
                    style_sources,
                    package_manifests,
                    resolution_inputs.bundler_path_mappings.as_slice(),
                    resolution_inputs.tsconfig_path_mappings.as_slice(),
                    external_sifs,
                    strictness,
                    &substrate,
                ));
        }
        push_omena_query_ready_surface(
            &mut summary.ready_surfaces,
            "externalSifBoundaryDiagnostics",
        );
        push_omena_query_ready_surface(&mut summary.ready_surfaces, "strictnessSigilGating");
    }
    attach_omena_query_runtime_state_inline_overrides_for_workspace(
        target_style_path,
        &mut summary,
        style_sources,
        source_documents,
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "runtimeStateInlineStyleOverrides",
    );
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut summary.diagnostics);
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "checkerProductDiagnosticGate");
    match suppression_mode {
        OmenaQueryDiagnosticSuppressionModeV0::Apply => {
            apply_omena_query_style_diagnostic_suppressions(&target.style_source, &mut summary);
        }
        OmenaQueryDiagnosticSuppressionModeV0::ReportOnly => {
            report_omena_query_style_diagnostic_suppressions(&target.style_source, &mut summary);
        }
    }
    summary.diagnostic_count = summary.diagnostics.len();
    Some(summary)
}

pub fn summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if !workspace_sources
        .iter()
        .any(|source| source.style_path == target_style_path)
    {
        return Vec::new();
    }
    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        workspace_sources,
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
        target_style_path,
        workspace_sources,
        &resolution,
    )
}

/// Substrate-threaded core of the resolution-identity pass (RFC 0009 Pillar B stage-2,
/// #65). `resolution` is the substrate's RES-A slot — plain resolution with
/// (package_manifests, bundler, tsconfig), identical to what the pub wrapper above
/// computes via `summarize_omena_query_sass_module_cross_file_resolution_for_workspace`.
fn summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = workspace_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let range = whole_file_omena_query_style_range(target.style_source.as_str());
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
    {
        let visible_symlink_links = edge
            .symlink_chain_links
            .iter()
            .filter(|link| !is_platform_alias_omena_query_symlink_link(link))
            .collect::<Vec<_>>();
        if !visible_symlink_links.is_empty()
            && emitted.insert((
                "sassModuleSymlinkResolution",
                edge.source.clone(),
                edge.resolved_style_path.clone(),
            ))
        {
            let target_path = edge
                .resolved_style_path
                .as_deref()
                .unwrap_or(edge.source.as_str());
            let link_summary = visible_symlink_links
                .first()
                .map(|link| format!("; first link {} -> {}", link.link_path, link.target_path))
                .unwrap_or_default();
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleSymlinkResolution",
                severity: "hint",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-resolver.symlink-chain-metadata",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' resolves to '{}' through {} symlink link(s){}.",
                    edge.source,
                    target_path,
                    visible_symlink_links.len(),
                    link_summary
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }

        if edge.configuration_variable_count > 0
            && let Some(identity_key) = edge.module_instance_identity_key.as_ref()
            && emitted.insert((
                "sassModuleInstanceIdentity",
                edge.source.clone(),
                Some(identity_key.clone()),
            ))
        {
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleInstanceIdentity",
                severity: "hint",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-query.module-instance-identity",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' uses {} configured variable(s); module instance identity is {}.",
                    edge.source, edge.configuration_variable_count, identity_key
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }

        if !edge.invalid_configuration_variable_names.is_empty()
            && emitted.insert((
                "sassModuleInvalidConfiguration",
                edge.source.clone(),
                edge.resolved_style_path.clone(),
            ))
        {
            let target_path = edge
                .resolved_style_path
                .as_deref()
                .unwrap_or(edge.source.as_str());
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleInvalidConfiguration",
                severity: "error",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-query.module-instance-identity",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' configures {} on '{}', but Sass @use/@forward with(...) can configure only public !default variables.",
                    edge.source,
                    format_omena_query_sass_configuration_variable_names(
                        edge.invalid_configuration_variable_names.as_slice()
                    ),
                    target_path
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }
    }

    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.configuration_variable_count > 0)
    {
        let Some(identity_key) = edge.module_instance_identity_key.as_ref() else {
            continue;
        };
        if !emitted.insert((
            "sassModuleInstanceIdentity",
            edge.target_style_path.clone(),
            Some(identity_key.clone()),
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleInstanceIdentity",
            severity: "hint",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module graph reaches configured module instance '{}' in {} hop(s); module instance identity is {}.",
                edge.target_style_path, edge.depth, identity_key
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| !edge.invalid_configuration_variable_names.is_empty())
    {
        if !emitted.insert((
            "sassModuleInvalidConfiguration",
            edge.target_style_path.clone(),
            Some(edge.configuration_signature.clone()),
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleInvalidConfiguration",
            severity: "error",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module graph reaches invalid configuration for '{}': {} are not public !default variables.",
                edge.target_style_path,
                format_omena_query_sass_configuration_variable_names(
                    edge.invalid_configuration_variable_names.as_slice()
                )
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    diagnostics.extend(
        summarize_omena_query_sass_module_configuration_conflict_diagnostics(
            target_style_path,
            workspace_sources,
            resolution,
            range,
        ),
    );

    diagnostics
}

fn summarize_omena_query_sass_module_configuration_conflict_diagnostics(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    range: ParserRangeV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let mut signatures_by_target = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.configuration_variable_count > 0)
    {
        signatures_by_target
            .entry(edge.target_style_path.clone())
            .or_default()
            .insert(edge.configuration_signature.clone());
    }
    for (target, signatures) in collect_omena_query_sass_module_load_order_configuration_conflicts(
        target_style_path,
        workspace_sources,
        resolution,
    ) {
        signatures_by_target
            .entry(target)
            .or_default()
            .extend(signatures);
    }

    signatures_by_target
        .into_iter()
        .filter(|(_, signatures)| signatures.len() > 1)
        .map(|(target, signatures)| OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleConfigurationConflict",
            severity: "error",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module '{target}' is reached with {} different configurations ({}); Sass modules can be configured only once per compilation.",
                signatures.len(),
                signatures.into_iter().collect::<Vec<_>>().join(", ")
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        })
        .collect()
}

fn collect_omena_query_sass_module_load_order_configuration_conflicts(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> BTreeMap<String, BTreeSet<String>> {
    let source_by_path = workspace_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut edges_by_from = BTreeMap::<&str, Vec<&OmenaQuerySassModuleEdgeResolutionV0>>::new();
    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "resolved" && edge.resolved_style_path.is_some())
    {
        edges_by_from
            .entry(edge.from_style_path.as_str())
            .or_default()
            .push(edge);
    }
    for (style_path, edges) in &mut edges_by_from {
        let style_source = source_by_path.get(style_path).copied().unwrap_or_default();
        edges.sort_by_key(|edge| {
            (
                omena_query_sass_module_edge_source_offset(
                    style_source,
                    edge.edge_kind,
                    edge.source.as_str(),
                ),
                edge.edge_kind,
                edge.rule_ordinal,
                edge.source.clone(),
            )
        });
    }

    let mut loaded_signatures_by_target = BTreeMap::new();
    let mut active_stack = BTreeSet::new();
    let mut conflicts_by_target = BTreeMap::new();
    collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
        target_style_path,
        &edges_by_from,
        &mut loaded_signatures_by_target,
        &mut active_stack,
        &mut conflicts_by_target,
    );
    conflicts_by_target
}

fn collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
    style_path: &str,
    edges_by_from: &BTreeMap<&str, Vec<&OmenaQuerySassModuleEdgeResolutionV0>>,
    loaded_signatures_by_target: &mut BTreeMap<String, String>,
    active_stack: &mut BTreeSet<String>,
    conflicts_by_target: &mut BTreeMap<String, BTreeSet<String>>,
) {
    if !active_stack.insert(style_path.to_string()) {
        return;
    }
    if let Some(edges) = edges_by_from.get(style_path) {
        for edge in edges {
            let Some(target_style_path) = edge.resolved_style_path.as_ref() else {
                continue;
            };
            let requested_signature = edge.configuration_signature.clone();
            let should_visit_target =
                match loaded_signatures_by_target.get(target_style_path.as_str()) {
                    Some(existing_signature)
                        if is_unconfigured_omena_query_sass_module_signature(
                            requested_signature.as_str(),
                        ) || existing_signature == &requested_signature =>
                    {
                        false
                    }
                    Some(existing_signature) => {
                        let signatures = conflicts_by_target
                            .entry(target_style_path.clone())
                            .or_default();
                        signatures.insert(existing_signature.clone());
                        signatures.insert(requested_signature);
                        false
                    }
                    None => {
                        loaded_signatures_by_target
                            .insert(target_style_path.clone(), requested_signature);
                        true
                    }
                };
            if should_visit_target {
                collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
                    target_style_path.as_str(),
                    edges_by_from,
                    loaded_signatures_by_target,
                    active_stack,
                    conflicts_by_target,
                );
            }
        }
    }
    active_stack.remove(style_path);
}

fn is_unconfigured_omena_query_sass_module_signature(signature: &str) -> bool {
    signature == "with:none"
}

fn format_omena_query_sass_configuration_variable_names(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("${name}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn omena_query_sass_module_edge_source_offset(
    style_source: &str,
    edge_kind: &str,
    source: &str,
) -> usize {
    let keyword = match edge_kind {
        "sassUse" => "@use",
        "sassForward" => "@forward",
        _ => return usize::MAX,
    };
    let mut search_start = 0usize;
    while let Some(relative_keyword_start) = style_source[search_start..].find(keyword) {
        let keyword_start = search_start + relative_keyword_start;
        let after_keyword = &style_source[keyword_start + keyword.len()..];
        let Some(relative_source_start) = after_keyword.find(source) else {
            search_start = keyword_start + keyword.len();
            continue;
        };
        let between_keyword_and_source = &after_keyword[..relative_source_start];
        if !between_keyword_and_source.contains(';') && !between_keyword_and_source.contains('{') {
            return keyword_start;
        }
        search_start = keyword_start + keyword.len();
    }
    usize::MAX
}

fn is_platform_alias_omena_query_symlink_link(link: &OmenaQuerySymlinkChainLinkV0) -> bool {
    matches!(
        (link.link_path.as_str(), link.target_path.as_str()),
        ("/var", "/private/var") | ("/tmp", "/private/tmp") | ("/etc", "/private/etc")
    )
}

fn whole_file_omena_query_style_range(source: &str) -> ParserRangeV0 {
    let mut line = 0usize;
    let mut character = 0usize;
    for ch in source.chars() {
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }
    ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 { line, character },
    }
}

fn attach_omena_query_module_graph_property_value_narrowing_for_workspace(
    target_style_path: &str,
    summary: &mut OmenaQueryStyleDiagnosticsForFileV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) {
    // Substrate RES-A slot: plain resolution with (package_manifests, bundler, tsconfig).
    let resolution = &substrate.sass_resolution;
    let reachable_style_paths =
        collect_sass_module_graph_reachable_style_paths(target_style_path, resolution);
    if reachable_style_paths.len() <= 1 {
        return;
    }

    let graph_candidates = style_sources
        .iter()
        .filter(|source| reachable_style_paths.contains(source.style_path.as_str()))
        .flat_map(|source| {
            super::cascade_checker::collect_query_checker_cascade_declarations(
                source.style_source.as_str(),
            )
            .into_iter()
            .map(|declaration| {
                (
                    declaration.input.selector.as_str().to_string(),
                    AbstractPropertyValueCandidateV0 {
                        property_name: declaration.input.property,
                        value: declaration.input.value,
                        pseudo_state: None,
                        condition_context: declaration.input.condition_context,
                        layer_name: declaration.input.layer_name,
                        layer_order: declaration.input.layer_order,
                        source_order: Some(declaration.input.source_order),
                        important: declaration.input.important,
                        same_selector_ordering: false,
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    if graph_candidates.is_empty() {
        return;
    }

    for diagnostic in &mut summary.diagnostics {
        let Some(cascade_narrowing) = diagnostic.cascade_narrowing.as_mut() else {
            continue;
        };
        let property_value_narrowing = &cascade_narrowing.property_value_narrowing;
        let mut static_reachability_by_context = BTreeMap::new();
        let property_candidates = graph_candidates
            .iter()
            .filter(|(selector, candidate)| {
                selector == &cascade_narrowing.selector
                    && candidate.property_name == cascade_narrowing.property_name
                    && *static_reachability_by_context
                        .entry(candidate.condition_context.clone())
                        .or_insert_with(|| {
                            super::cascade_checker::query_condition_context_static_supports_pruning_evidence(
                                candidate.condition_context.as_slice(),
                                Some(
                                    property_value_narrowing
                                        .requested_condition_context
                                        .as_slice(),
                                ),
                            )
                            .is_none_or(|evidence| !evidence.pruned)
                        })
            })
            .map(|(_, candidate)| candidate.clone())
            .collect::<Vec<_>>();
        if property_candidates.is_empty() {
            continue;
        }
        let mut narrowed = narrow_abstract_property_value_for_cascade_branch(
            cascade_narrowing.property_name.as_str(),
            property_value_narrowing.requested_pseudo_state.as_deref(),
            property_value_narrowing
                .requested_condition_context
                .as_slice(),
            property_value_narrowing.requested_layer_name.as_deref(),
            property_value_narrowing.requested_layer_order,
            property_value_narrowing.requested_layer_scope == "exactLayer",
            property_candidates.as_slice(),
        );
        narrowed.stylesheet_scope = "moduleGraph";
        cascade_narrowing.property_value_narrowing = narrowed;
    }
}

fn attach_omena_query_runtime_state_inline_overrides_for_workspace(
    target_style_path: &str,
    summary: &mut OmenaQueryStyleDiagnosticsForFileV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) {
    let inline_overrides = collect_omena_query_inline_style_runtime_overrides_for_style(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    if inline_overrides.is_empty() {
        return;
    }

    for diagnostic in &mut summary.diagnostics {
        let Some(runtime_state) = diagnostic
            .cascade_narrowing
            .as_mut()
            .and_then(|narrowing| narrowing.runtime_state.as_mut())
        else {
            continue;
        };
        runtime_state.inline_style_overrides = inline_overrides.clone();
        let property_name = runtime_state.property_name.clone();
        runtime_state.scenarios.extend(
            inline_overrides
                .iter()
                .filter(|override_fact| override_fact.property_name == property_name)
                .map(|override_fact| {
                    omena_query_inline_style_runtime_override_scenario(
                        property_name.as_str(),
                        override_fact,
                    )
                }),
        );
        for driver in &mut runtime_state.driver_summaries {
            if driver.driver == "inlineStyleHighestSpecificityTier" {
                driver.status = "sourceFactsJoined";
                driver.scenario_count = runtime_state
                    .inline_style_overrides
                    .iter()
                    .filter(|override_fact| override_fact.property_name == property_name)
                    .count();
            }
        }
        runtime_state.confidence_tier = query_runtime_state_confidence_tier(
            runtime_state.scenarios.as_slice(),
            runtime_state.inline_style_overrides.as_slice(),
        );
    }
}

fn omena_query_inline_style_runtime_override_scenario(
    property_name: &str,
    override_fact: &OmenaQueryInlineStyleRuntimeOverrideV0,
) -> OmenaQueryRuntimeStateScenarioV0 {
    let value = override_fact
        .value
        .clone()
        .unwrap_or_else(|| "<dynamic>".to_string());
    let property_value_narrowing = narrow_abstract_property_value_for_pseudo_state(
        property_name,
        None,
        &[AbstractPropertyValueCandidateV0 {
            property_name: property_name.to_string(),
            value: value.clone(),
            pseudo_state: None,
            condition_context: Vec::new(),
            layer_name: None,
            layer_order: None,
            source_order: Some(0),
            important: true,
            same_selector_ordering: true,
        }],
    );

    OmenaQueryRuntimeStateScenarioV0 {
        scenario_kind: "inlineStyleOverride",
        pseudo_state: None,
        condition_context: Vec::new(),
        declaration_ids: vec![format!(
            "inline-style:{}:{}:{}",
            override_fact.source_path,
            override_fact.range.start.line,
            override_fact.range.start.character
        )],
        winner_declaration_id: Some("inline-style-author-tier".to_string()),
        winner_value: Some(value),
        property_value_narrowing,
    }
}

fn collect_omena_query_inline_style_runtime_overrides_for_style(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<OmenaQueryInlineStyleRuntimeOverrideV0> {
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut overrides = Vec::new();

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
            let Some(style_path) = resolve_style_module_source_with_path_mappings(
                &document.source_path,
                &import.specifier,
                &available_style_paths,
                package_manifests,
                bundler_path_mappings,
                tsconfig_path_mappings,
            ) else {
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
        for declaration in index.inline_style_declarations {
            if declaration.target_style_uri.as_deref() != Some(target_style_path) {
                continue;
            }
            overrides.push(OmenaQueryInlineStyleRuntimeOverrideV0 {
                source_path: document.source_path.clone(),
                range: parser_range_for_byte_span(&document.source_source, declaration.byte_span),
                property_name: declaration.property_name,
                value: declaration.value,
                cascade_tier: declaration.cascade_tier,
                static_value: declaration.static_value,
            });
        }
    }

    overrides.sort_by(|left, right| {
        left.source_path
            .cmp(&right.source_path)
            .then_with(|| left.range.start.line.cmp(&right.range.start.line))
            .then_with(|| left.range.start.character.cmp(&right.range.start.character))
            .then_with(|| left.property_name.cmp(&right.property_name))
    });
    overrides.dedup();
    overrides
}

#[allow(clippy::too_many_arguments)]
fn collect_omena_query_external_top_any_sass_symbol_ranges(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> BTreeSet<ParserRangeV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return BTreeSet::new();
    };
    // Substrate RES-D slot: SIF-promoted resolution over the same
    // (package_manifests, bundler, tsconfig, external_sifs) arguments.
    let resolution = &substrate.sass_resolution_with_external_sifs;
    let external_sif_context = OmenaQueryExternalSifResolutionContext {
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        external_sifs,
    };
    let external_sources = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "external")
        .map(|edge| edge.source.as_str())
        .collect::<BTreeSet<_>>();
    if external_sources.is_empty() {
        return BTreeSet::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    // The protocol lattice is the single source of truth: a namespace is TopAny iff its
    // external edge classifies to a `top == TopAny` state (Missing/Partial/Stale). A
    // Resolved (TopOpaque) edge — i.e. one backed by a complete SIF — is *not* TopAny, so
    // its symbols stay subject to ordinary missing-symbol checking. (#34)
    let top_any_namespaces = facts
        .sass_module_edges
        .iter()
        .filter(|edge| external_sources.contains(edge.source.as_str()))
        .filter(|edge| {
            let sif = resolution
                .edges
                .iter()
                .find(|resolution_edge| {
                    resolution_edge.from_style_path == target_style_path
                        && resolution_edge.source == edge.source
                        && resolution_edge.status == "external"
                })
                .and_then(|resolution_edge| {
                    find_omena_query_external_sif_for_edge(resolution_edge, external_sif_context)
                });
            classify_external_boundary_state(edge, sif, &facts, external_sifs).top
                == OmenaResolverBoundaryTopV0::TopAny
        })
        .filter_map(|edge| match edge.kind {
            ParsedSassModuleEdgeFactKind::Use
                if edge.namespace_kind == Some("default")
                    || edge.namespace_kind == Some("alias") =>
            {
                edge.namespace.clone().map(Some)
            }
            ParsedSassModuleEdgeFactKind::Use if edge.namespace_kind == Some("wildcard") => {
                Some(None)
            }
            ParsedSassModuleEdgeFactKind::Import => Some(None),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if top_any_namespaces.is_empty() {
        return BTreeSet::new();
    }

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

/// Classify a single external (`status == "external"`) Sass module edge onto the
/// resolver's five-state boundary lattice (#34).
///
/// Four of the five states are derivable today, with no new transport:
/// - **Missing** — no local SIF artifact is in scope for the edge's canonical URL.
/// - **Stale** — a SIF is present but one of its declared dependency interface
///   hashes no longer matches the SIF actually in scope for that dependency.
/// - **Partial** — a SIF is present but only some of the symbols referenced through
///   this edge's namespace appear in its exported interface.
/// - **Resolved** — a SIF is present and every referenced symbol (or no symbol at
///   all) is covered by its exported interface.
///
/// The fifth state (`Unresolved`) is classified by the caller, not here: an unresolved edge
/// has no SIF lattice to reason over, so it folds through the resolver-error channel via
/// `omena_resolver_boundary_state_for_unresolved_reference_v0` (#34).
fn classify_external_boundary_state(
    edge: &ParsedSassModuleEdgeFact,
    sif: Option<&OmenaQueryExternalSifInputV0>,
    target_facts: &omena_parser::ParsedStyleFacts,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> OmenaResolverBoundaryStateV0 {
    let Some(sif) = sif else {
        return OmenaResolverBoundaryStateV0::missing(
            None,
            "SIF mode requires a local SIF artifact for this external Sass module",
        );
    };

    let canonical_url = OmenaResolverCanonicalUrlV0 {
        url: edge.source.clone(),
    };

    // Stale: a declared dependency's recorded interface hash no longer agrees with the
    // SIF currently in scope for that dependency canonical URL.
    if let Some(dependency) = sif.sif.dependencies.iter().find(|dependency| {
        find_omena_query_external_sif(dependency.canonical_url.as_str(), external_sifs)
            .map(|dependency_sif| {
                dependency_sif.sif.fingerprints.interface_hash != dependency.interface_hash
            })
            .unwrap_or(false)
    }) {
        return OmenaResolverBoundaryStateV0::stale(
            canonical_url,
            format!(
                "external SIF dependency '{}' interface hash drifted from the lockfile-recorded hash",
                dependency.canonical_url
            ),
        );
    }

    // Partial vs Resolved: do all symbols referenced through this edge's namespace
    // appear in the SIF's exported interface?
    let exported = collect_sif_exported_sass_symbol_keys(&sif.sif, external_sifs);
    let mut referenced = 0usize;
    let mut covered = 0usize;
    for symbol in &target_facts.sass_symbols {
        if !omena_query_sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }
        if !sass_symbol_reference_belongs_to_edge(edge, symbol.namespace.as_deref()) {
            continue;
        }
        referenced += 1;
        if exported.contains(&(symbol.symbol_kind, fold_sass_symbol_name(&symbol.name))) {
            covered += 1;
        }
    }

    if referenced > 0 && covered < referenced {
        return OmenaResolverBoundaryStateV0::partial(format!(
            "external SIF for '{}' exports only {}/{} referenced symbol(s)",
            edge.source, covered, referenced
        ));
    }

    OmenaResolverBoundaryStateV0::resolved(canonical_url)
}

/// Does a Sass symbol reference (with the given `@use` namespace) flow through `edge`?
///
/// Mirrors the namespace-binding rules already used by the visible-symbol collector:
/// a default/alias `@use` binds references under its namespace, while a wildcard
/// `@use` or an `@import`/`@forward` binds bare (namespace-less) references.
fn sass_symbol_reference_belongs_to_edge(
    edge: &ParsedSassModuleEdgeFact,
    reference_namespace: Option<&str>,
) -> bool {
    match edge.kind {
        ParsedSassModuleEdgeFactKind::Use
            if edge.namespace_kind == Some("default") || edge.namespace_kind == Some("alias") =>
        {
            edge.namespace.as_deref() == reference_namespace
        }
        ParsedSassModuleEdgeFactKind::Use if edge.namespace_kind == Some("wildcard") => {
            reference_namespace.is_none()
        }
        ParsedSassModuleEdgeFactKind::Import | ParsedSassModuleEdgeFactKind::Forward => {
            reference_namespace.is_none()
        }
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
fn summarize_omena_query_external_sif_boundary_diagnostics(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    strictness: OmenaStrictnessLevelV0,
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    // Substrate RES-D slot: SIF-promoted resolution over the same
    // (package_manifests, bundler, tsconfig, external_sifs) arguments.
    let resolution = &substrate.sass_resolution_with_external_sifs;
    let external_sif_context = OmenaQueryExternalSifResolutionContext {
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        external_sifs,
    };
    // Every external edge is now classified on the real lattice — including ones that
    // *do* have a SIF in scope (the `.is_none()` pre-filter is gone, #34). Missing edges
    // warn; Stale/Partial edges warn with distinct codes; Resolved edges emit nothing.
    let external_sources = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "external")
        .map(|edge| edge.source.as_str())
        .collect::<BTreeSet<_>>();
    // The fifth state (`Unresolved`, #34): an edge whose canonical URL the resolver could
    // not canonicalize at all (`status == "unresolved"`). The resolver-error channel now
    // reaches this layer through `omena_resolver_boundary_state_for_unresolved_reference_v0`.
    // We only adopt the *bare* unresolved edges here: workspace-local unresolved specifiers
    // (`./`, `../`, `/`) are already a hard `missingModule` error elsewhere, so re-flagging
    // them as a boundary state would double-emit. Bare unresolved edges (`'bootstrap'` with
    // no SIF in scope) are the ones the boundary diagnostic previously left silent.
    let unresolved_sources = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "unresolved")
        .filter(|edge| !sass_module_source_is_workspace_local(edge.source.as_str()))
        .map(|edge| edge.source.as_str())
        .collect::<BTreeSet<_>>();
    if external_sources.is_empty() && unresolved_sources.is_empty() {
        return Vec::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for edge in &facts.sass_module_edges {
        let is_external = external_sources.contains(edge.source.as_str());
        let is_unresolved = unresolved_sources.contains(edge.source.as_str());
        if !is_external && !is_unresolved {
            continue;
        }
        if !emitted.insert((edge.kind, edge.source.clone())) {
            continue;
        }
        // An unresolved edge folds through the resolver-error channel onto the `Unresolved`
        // boundary state; an external edge is classified against the SIF lattice (#34).
        let state = if is_unresolved {
            omena_resolver_boundary_state_for_unresolved_reference_v0(edge.source.as_str())
        } else {
            let sif = resolution
                .edges
                .iter()
                .find(|resolution_edge| {
                    resolution_edge.from_style_path == target_style_path
                        && resolution_edge.source == edge.source
                        && resolution_edge.status == "external"
                })
                .and_then(|resolution_edge| {
                    find_omena_query_external_sif_for_edge(resolution_edge, external_sif_context)
                });
            classify_external_boundary_state(edge, sif, &facts, external_sifs)
        };
        let (code, default_severity) = match state.state {
            // A fully-resolved boundary has no diagnostic to emit.
            OmenaResolverBoundaryStateKindV0::Resolved => continue,
            OmenaResolverBoundaryStateKindV0::Stale => ("staleExternalSif", "warning"),
            OmenaResolverBoundaryStateKindV0::Partial => ("partialExternalSif", "information"),
            OmenaResolverBoundaryStateKindV0::Missing => ("missingExternalSif", "warning"),
            OmenaResolverBoundaryStateKindV0::Unresolved => {
                ("unresolvedExternalReference", "warning")
            }
        };
        // The strictness sigil (#35) multiplies into the severity decision: `Strict`/`Closed`
        // escalate the boundary to `error`; `Standard`/`Relaxed` pass the default through.
        let severity = strictness.boundary_severity(default_severity);
        let start: u32 = edge.range.start().into();
        let end: u32 = edge.range.end().into();
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code,
            severity,
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
                "External Sass module '{}' is {} ({}); {}",
                edge.source,
                state.state_name,
                state.top_name,
                external_boundary_remediation_hint(state.state)
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    diagnostics
}

/// Per-state remediation hint appended to the boundary diagnostic message.
fn external_boundary_remediation_hint(state: OmenaResolverBoundaryStateKindV0) -> &'static str {
    match state {
        OmenaResolverBoundaryStateKindV0::Missing => {
            "generate or provide a SIF artifact, or use --external ignored."
        }
        OmenaResolverBoundaryStateKindV0::Stale => {
            "regenerate the SIF/lockfile so its dependency interface hashes match."
        }
        OmenaResolverBoundaryStateKindV0::Partial => {
            "some referenced symbols are absent from the SIF interface; regenerate the SIF or fix the reference."
        }
        OmenaResolverBoundaryStateKindV0::Unresolved => {
            "the resolver cannot canonicalize this reference; fix the specifier or add it to the workspace."
        }
        OmenaResolverBoundaryStateKindV0::Resolved => "",
    }
}

type SassSymbolKey = (&'static str, Option<String>, String);

#[derive(Clone, Copy)]
struct OmenaQueryExternalSifResolutionContext<'a> {
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &'a [OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &'a [OmenaResolverTsconfigPathMappingV0],
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
}

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
    external_sif_context: OmenaQueryExternalSifResolutionContext<'_>,
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
                        external_sif_context,
                        &mut visiting,
                    )
                })
                .unwrap_or_default()
        } else if edge.status == "external" {
            find_omena_query_external_sif_for_edge(edge, external_sif_context)
                .map(|sif| {
                    collect_sif_exported_sass_symbol_keys(
                        &sif.sif,
                        external_sif_context.external_sifs,
                    )
                })
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
    external_sif_context: OmenaQueryExternalSifResolutionContext<'_>,
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
                            external_sif_context,
                            visiting,
                        )
                    })
                    .unwrap_or_default()
            } else if edge.status == "external" {
                find_omena_query_external_sif_for_edge(edge, external_sif_context)
                    .map(|sif| {
                        collect_sif_exported_sass_symbol_keys(
                            &sif.sif,
                            external_sif_context.external_sifs,
                        )
                    })
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

fn summarize_sass_module_cross_file_resolution_with_external_sifs(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    let mut resolution = summarize_sass_module_cross_file_resolution(
        style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    promote_sif_backed_external_edges(
        &mut resolution,
        OmenaQueryExternalSifResolutionContext {
            package_manifests,
            bundler_path_mappings,
            tsconfig_path_mappings,
            external_sifs,
        },
    );
    resolution
}

fn promote_sif_backed_external_edges(
    resolution: &mut OmenaQuerySassModuleCrossFileResolutionV0,
    external_sif_context: OmenaQueryExternalSifResolutionContext<'_>,
) {
    let external_sifs = external_sif_context.external_sifs;
    if external_sifs.is_empty() {
        return;
    }
    for edge in &mut resolution.edges {
        if edge.status == "unresolved"
            && !sass_module_source_is_workspace_local(edge.source.as_str())
            && find_omena_query_external_sif_for_edge(edge, external_sif_context).is_some()
        {
            edge.status = "external";
            edge.resolution_kind = "externalSifCanonicalUrl";
        }
    }
    resolution.resolved_module_edge_count = resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "resolved")
        .count();
    resolution.external_module_edge_count = resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "external")
        .count();
    resolution.unresolved_module_edge_count = resolution.module_edge_count.saturating_sub(
        resolution.resolved_module_edge_count + resolution.external_module_edge_count,
    );
}

fn find_omena_query_external_sif_for_edge<'a>(
    edge: &OmenaQuerySassModuleEdgeResolutionV0,
    external_sif_context: OmenaQueryExternalSifResolutionContext<'a>,
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    let external_sifs = external_sif_context.external_sifs;
    if let Some(sif) = find_omena_query_external_sif(edge.source.as_str(), external_sifs) {
        return Some(sif);
    }

    let resolver_package_manifests = external_sif_context
        .package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    collect_omena_resolver_style_module_source_candidates_with_load_path_roots(
        edge.from_style_path.as_str(),
        edge.source.as_str(),
        resolver_package_manifests.as_slice(),
        external_sif_context.bundler_path_mappings,
        external_sif_context.tsconfig_path_mappings,
        &[],
    )
    .into_iter()
    .find_map(|candidate| find_omena_query_external_sif(candidate.as_str(), external_sifs))
}

fn collect_sif_exported_sass_symbol_keys(
    sif: &omena_sif::OmenaSifV1,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> BTreeSet<(&'static str, String)> {
    let mut visiting = BTreeSet::new();
    collect_sif_exported_sass_symbol_keys_inner(sif, external_sifs, &mut visiting)
}

fn collect_sif_exported_sass_symbol_keys_inner(
    sif: &omena_sif::OmenaSifV1,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    visiting: &mut BTreeSet<String>,
) -> BTreeSet<(&'static str, String)> {
    if !visiting.insert(sif.canonical_url.clone()) {
        return BTreeSet::new();
    }
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
    exported.extend(sif.exports.placeholders.iter().map(|placeholder| {
        (
            "placeholder",
            placeholder.name.trim_start_matches('%').to_string(),
        )
    }));
    for forward in &sif.exports.forwards {
        let Some(forwarded_sif) =
            find_omena_query_external_sif_for_forward(sif, forward, external_sifs)
        else {
            continue;
        };
        let forwarded_exports = collect_sif_exported_sass_symbol_keys_inner(
            &forwarded_sif.sif,
            external_sifs,
            visiting,
        );
        for (symbol_kind, name) in forwarded_exports {
            if !sif_forward_visibility_allows(forward, symbol_kind, name.as_str()) {
                continue;
            }
            exported.insert((
                symbol_kind,
                apply_sass_forward_prefix(forward.prefix.as_deref(), name.as_str()),
            ));
        }
    }
    visiting.remove(sif.canonical_url.as_str());
    exported
}

fn find_omena_query_external_sif_for_forward<'a>(
    sif: &omena_sif::OmenaSifV1,
    forward: &omena_sif::OmenaSifForwardExportV1,
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    let candidates = collect_sif_forward_canonical_url_candidates(
        sif.canonical_url.as_str(),
        forward.canonical_url.as_str(),
    );
    candidates
        .iter()
        .find_map(|candidate| find_omena_query_external_sif(candidate.as_str(), external_sifs))
}

fn collect_sif_forward_canonical_url_candidates(
    base_canonical_url: &str,
    source: &str,
) -> Vec<String> {
    let mut candidates = BTreeSet::new();
    candidates.insert(source.to_string());
    if let Some(base_file_path) = base_canonical_url.strip_prefix("file://") {
        push_file_uri_forward_candidates(&mut candidates, base_file_path, source);
    }
    candidates.into_iter().collect()
}

fn push_file_uri_forward_candidates(
    candidates: &mut BTreeSet<String>,
    base_file_path: &str,
    source: &str,
) {
    if source.starts_with("sass:")
        || source.starts_with("http://")
        || source.starts_with("https://")
        || source.starts_with("file://")
        || source.starts_with("pkg:")
    {
        return;
    }
    let base_path = Path::new(base_file_path);
    let joined = if source.starts_with('/') {
        PathBuf::from(source)
    } else {
        base_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(source)
    };
    push_file_uri_style_path_candidates(candidates, joined.as_path());
}

fn push_file_uri_style_path_candidates(candidates: &mut BTreeSet<String>, path: &Path) {
    let has_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some();
    if has_extension {
        push_file_uri_candidate(candidates, path);
        return;
    }
    for extension in ["scss", "sass", "css"] {
        let with_extension = path.with_extension(extension);
        push_file_uri_candidate(candidates, with_extension.as_path());
        if let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) {
            let partial = path
                .with_file_name(format!("_{file_name}"))
                .with_extension(extension);
            push_file_uri_candidate(candidates, partial.as_path());
        }
    }
}

fn push_file_uri_candidate(candidates: &mut BTreeSet<String>, path: &Path) {
    candidates.insert(format!("file://{}", normalize_sif_file_uri_path(path)));
}

fn normalize_sif_file_uri_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn find_omena_query_external_sif<'a>(
    canonical_url: &str,
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    external_sifs.iter().find(|input| {
        omena_query_external_sif_canonical_urls_match(input.canonical_url.as_str(), canonical_url)
            || omena_query_external_sif_canonical_urls_match(
                input.sif.canonical_url.as_str(),
                canonical_url,
            )
    })
}

fn omena_query_external_sif_canonical_urls_match(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let Some(left_path) = omena_query_external_sif_canonical_url_path(left) else {
        return false;
    };
    let Some(right_path) = omena_query_external_sif_canonical_url_path(right) else {
        return false;
    };
    canonicalize_omena_resolver_style_identity_path(left_path.as_str())
        == canonicalize_omena_resolver_style_identity_path(right_path.as_str())
}

fn omena_query_external_sif_canonical_url_path(canonical_url: &str) -> Option<String> {
    if let Some(path) = canonical_url.strip_prefix("file://") {
        return Some(path.to_string());
    }
    Path::new(canonical_url)
        .is_absolute()
        .then(|| canonical_url.to_string())
}

fn sif_forward_visibility_allows(
    forward: &omena_sif::OmenaSifForwardExportV1,
    symbol_kind: &'static str,
    name: &str,
) -> bool {
    let matches_filter = |filter_name: &String| {
        sass_forward_filter_name_matches_symbol(
            filter_name,
            forward.prefix.as_deref(),
            symbol_kind,
            name,
        )
    };
    if !forward.show.is_empty() {
        return forward.show.iter().any(matches_filter);
    }
    !forward.hide.iter().any(matches_filter)
}

fn sass_forward_visibility_allows(
    edge: &OmenaQuerySassModuleEdgeResolutionV0,
    symbol_kind: &'static str,
    name: &str,
) -> bool {
    let matches_filter = |filter_name: &String| {
        sass_forward_filter_name_matches_symbol(
            filter_name,
            edge.forward_prefix.as_deref(),
            symbol_kind,
            name,
        )
    };
    match edge.visibility_filter_kind {
        Some("show") => edge.visibility_filter_names.iter().any(matches_filter),
        Some("hide") => !edge.visibility_filter_names.iter().any(matches_filter),
        _ => true,
    }
}

fn sass_forward_filter_name_matches_symbol(
    filter_name: &str,
    prefix: Option<&str>,
    symbol_kind: &'static str,
    name: &str,
) -> bool {
    let exposed_name = apply_sass_forward_prefix(prefix, name);
    filter_name == exposed_name
        || filter_name.trim_start_matches('$') == exposed_name
        || (symbol_kind != "variable" && filter_name.trim_start_matches('@') == exposed_name)
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
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
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
    summarize_omena_query_css_modules_resolution_style_diagnostics_from_entries(
        target_style_path,
        target_source,
        &style_fact_entries,
        package_manifests,
    )
}

/// Substrate-threaded core of the css-modules resolution pass (RFC 0009 Pillar B
/// stage-2, #65). `style_fact_entries` is the substrate's ENTRIES slot; this pass never
/// computed a Sass cross-file resolution (composes targets resolve per-edge via
/// `resolve_style_module_source`).
fn summarize_omena_query_css_modules_resolution_style_diagnostics_from_entries(
    target_style_path: &str,
    target_source: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
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
                    cascade_narrowing: None,
                    cascade_confidence: None,
                    polynomial_provenance: None,
                    cross_file_scc: None,
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
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
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
                    cascade_narrowing: None,
                    cascade_confidence: None,
                    polynomial_provenance: None,
                    cross_file_scc: None,
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
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
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
    summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings(
        target_style_path,
        target_source,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        &[],
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings(
    target_style_path: &str,
    target_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if source_documents.is_empty() {
        return Vec::new();
    }

    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings_from_entries(
        target_style_path,
        target_source,
        &style_fact_entries,
        source_documents,
        package_manifests,
        classname_transform,
        bundler_path_mappings,
        tsconfig_path_mappings,
    )
}

/// Substrate-threaded core of the unused-selector pass (RFC 0009 Pillar B stage-2,
/// #65). `style_fact_entries` is the substrate's ENTRIES slot; this pass never computed
/// a Sass cross-file resolution (source usage resolves per-document imports via
/// `collect_omena_query_source_selector_usage_by_style`, which still re-runs its own
/// source import/syntax indexing — not covered by the substrate).
#[allow(clippy::too_many_arguments)]
fn summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings_from_entries(
    target_style_path: &str,
    target_source: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if source_documents.is_empty() {
        return Vec::new();
    }

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
            bundler_path_mappings,
            tsconfig_path_mappings,
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
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            })
        })
        .collect()
}

fn collect_omena_query_source_selector_usage_by_style(
    available_style_paths: &BTreeSet<&str>,
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    aliases_by_path: &BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
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
            let Some(style_path) = resolve_style_module_source_with_path_mappings(
                &document.source_path,
                &import.specifier,
                available_style_paths,
                package_manifests,
                bundler_path_mappings,
                tsconfig_path_mappings,
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
    fn external_sif_canonical_url_match_accepts_raw_path_and_file_uri() {
        assert!(omena_query_external_sif_canonical_urls_match(
            "/tmp/tokens.scss",
            "file:///tmp/tokens.scss",
        ));
    }

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

    #[test]
    fn media_qualified_import_is_not_deprecated() {
        // RFC-0007-D1 (#44): `@import "foo" screen` and `@import "foo" (min-width: ...)`
        // are kept as plain CSS by Sass; the media qualifier must suppress the hint.
        let screen = summarize_omena_query_sass_import_deprecation_hints(
            "theme.scss",
            "@import \"foo\" screen;",
        );
        assert!(
            screen.is_empty(),
            "media-qualified (Ident) @import must NOT warn deprecatedSassImport, got {screen:?}"
        );
        let feature = summarize_omena_query_sass_import_deprecation_hints(
            "theme.scss",
            "@import \"foo\" (min-width: 100px);",
        );
        assert!(
            feature.is_empty(),
            "media-feature-qualified @import must NOT warn deprecatedSassImport, got {feature:?}"
        );
    }

    #[test]
    fn bare_partial_import_still_deprecated() {
        // Over-correction guard: a genuine Sass-form `@import 'partial'` (no media, no
        // url, no `.css`) MUST still warn deprecatedSassImport.
        let diagnostics =
            summarize_omena_query_sass_import_deprecation_hints("theme.scss", "@import 'partial';");
        assert_eq!(
            diagnostics.len(),
            1,
            "bare Sass partial @import must still warn, got {diagnostics:?}"
        );
        assert_eq!(diagnostics[0].code, "deprecatedSassImport");
    }

    #[test]
    fn media_qualified_comma_peer_classifies_per_target() {
        // `@import "a", "b" screen`: only `"b"` is media-qualified; `"a"` stays a Sass
        // partial and must still warn. Per-target classification, not per-statement.
        let diagnostics = summarize_omena_query_sass_import_deprecation_hints(
            "theme.scss",
            "@import \"a\", \"b\" screen;",
        );
        assert_eq!(
            diagnostics.len(),
            1,
            "exactly the bare partial peer must warn, got {diagnostics:?}"
        );
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

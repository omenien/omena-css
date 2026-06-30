//! Transform pass execution over source text and explicit workspace context.
//!
//! The executor is the mutation boundary for ordered transform plans. It applies
//! registered pass kinds, records provenance outcomes, and preserves semantic
//! removal evidence for downstream query and consumer surfaces.

use omena_parser::StyleDialect;
use omena_transform_cst::{
    IrNodeKindV0, StableTransformIrNodeV0, TransformIrV0, TransformPassClassV0, TransformPassKind,
    build_stable_transform_ir_from_source, lower_transform_ir_from_source,
};

use super::{
    cascade_proof::{
        collect_cascade_proof_obligations_for_ir_pass_input,
        collect_cascade_proof_obligations_for_pass_input, summarize_cascade_proof_obligations,
    },
    outcome::{mutation_outcome, no_change_outcome, planned_only_outcome},
    planner::{
        default_transform_pass_registry, plan_transform_passes, transform_pass_kind_from_id,
    },
    provenance::{derive_transform_mutation_spans, provenance_derivation_forest_from_outcomes},
};
use crate::helpers::ir_transaction::{
    reset_structural_ir_transaction_mutation_span_batches,
    reset_structural_ir_transaction_telemetry, structural_ir_transaction_telemetry_snapshot,
    take_structural_ir_transaction_mutation_span_batches,
};
use crate::model::{
    TransformCssModuleComposesResolutionV0, TransformDesignTokenRouteV0,
    TransformExecutionContextV0, TransformExecutionSummaryV0, TransformImportInlineV0,
    TransformModuleEvaluationNativeEditV0, TransformModuleEvaluationV0,
    TransformPassDispatchKindV0, TransformPassExecutionOutcomeV0, TransformPassRegistryEntryV0,
    TransformPassRuntimeStatus, TransformProvenanceMutationSpanV0, TransformSemanticRemovalV0,
    TransformVendorPrefixPolicyV0,
};
use crate::registry::{
    add_css_vendor_prefixes, combine_css_shorthands, compress_css_colors,
    compress_css_is_where_selectors, compress_css_numbers, css_module_composes_resolutions_for_ir,
    dedupe_exact_css_rules_in_ir, evaluate_dead_media_branch_rules_in_ir,
    evaluate_native_css_static_values_in_ir, evaluate_static_container_rules_in_ir,
    evaluate_static_media_rules_in_ir, evaluate_static_supports_rules_in_ir,
    flatten_css_layers_in_ir, flatten_css_scopes_in_ir, inline_css_imports_in_ir,
    lower_css_color_function, lower_css_color_mix, lower_css_light_dark,
    lower_css_logical_to_physical, lower_css_oklab_oklch, lower_relative_color,
    merge_adjacent_same_block_css_selectors_in_ir, merge_adjacent_same_selector_css_rules_in_ir,
    normalize_css_string_quotes, normalize_css_units, normalize_css_whitespace,
    reachable_class_names_with_composes_exports, reduce_css_calc, remove_empty_css_rules_in_ir,
    remove_stale_css_vendor_prefixes, resolve_css_module_composes_in_ir,
    resolve_static_css_modules_values, rewrite_css_module_class_names_in_ir,
    route_design_token_values_in_ir, strip_css_comments, strip_css_url_quotes,
    substitute_static_css_custom_properties, tree_shake_css_class_rules_in_ir,
    tree_shake_css_custom_properties_in_ir, tree_shake_css_keyframes_in_ir,
    tree_shake_css_modules_values_in_ir, unwrap_css_nesting_in_ir,
};

type TransformTextLocalRunnerV0 =
    for<'a> fn(TransformTextLocalPassInputV0<'a>) -> TransformTextLocalPassOutputV0<'a>;

#[derive(Clone, Copy)]
struct TransformTextLocalPassHandlerV0 {
    kind: TransformPassKind,
    window_scope: TransformTextLocalWindowScopeV0,
    execution_mode: TransformTextLocalExecutionModeV0,
    detail: &'static str,
    run: TransformTextLocalRunnerV0,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TransformTextLocalWindowScopeV0 {
    DocumentTokenStream,
    Selector,
    DeclarationBlock,
    DeclarationValue,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TransformTextLocalExecutionModeV0 {
    FullDocument,
    WindowBatch,
}

struct TransformPassDispatchResultV0 {
    next_textual_css: Option<String>,
    document_ir_updated: bool,
    outcome: TransformPassExecutionOutcomeV0,
    css_module_evaluation: Option<TransformModuleEvaluationV0>,
    css_import_inlines: Vec<TransformImportInlineV0>,
    css_module_composes_exports: Vec<TransformCssModuleComposesResolutionV0>,
    design_token_routes: Vec<TransformDesignTokenRouteV0>,
    semantic_removals: Vec<TransformSemanticRemovalV0>,
    provenance_mutation_spans: Option<Vec<TransformProvenanceMutationSpanV0>>,
}

#[derive(Clone, Copy)]
enum TransformRuntimePassImplementationV0 {
    TextLocal(TransformTextLocalPassHandlerV0),
    Structural(TransformStructuralPassHandlerV0),
    ModuleEvaluation(TransformPassKind),
    Emission(TransformPassKind),
}

struct TransformRuntimePassEntryV0<'a> {
    registry_entry: &'a TransformPassRegistryEntryV0,
    implementation: TransformRuntimePassImplementationV0,
}

fn runtime_pass_entry_for_kind<'a>(
    kind: TransformPassKind,
    registry_entries: &'a [TransformPassRegistryEntryV0],
) -> Option<TransformRuntimePassEntryV0<'a>> {
    let registry_entry = registry_entries
        .iter()
        .find(|entry| entry.contract.kind == kind)?;
    let implementation = runtime_pass_implementation_for_entry(registry_entry)?;
    Some(TransformRuntimePassEntryV0 {
        registry_entry,
        implementation,
    })
}

fn runtime_pass_implementation_for_entry(
    entry: &TransformPassRegistryEntryV0,
) -> Option<TransformRuntimePassImplementationV0> {
    match entry.dispatch_kind {
        TransformPassDispatchKindV0::TextLocalSliceRewrite => text_local_pass_handlers()
            .iter()
            .find(|handler| handler.kind == entry.contract.kind)
            .copied()
            .map(TransformRuntimePassImplementationV0::TextLocal),
        TransformPassDispatchKindV0::StructuralIrTransaction => structural_pass_handlers()
            .iter()
            .find(|handler| handler.kind == entry.contract.kind)
            .copied()
            .map(TransformRuntimePassImplementationV0::Structural),
        TransformPassDispatchKindV0::ModuleEvaluationHandler => Some(
            TransformRuntimePassImplementationV0::ModuleEvaluation(entry.contract.kind),
        ),
        TransformPassDispatchKindV0::EmissionBoundary => Some(
            TransformRuntimePassImplementationV0::Emission(entry.contract.kind),
        ),
    }
}

#[derive(Default)]
struct TransformTextualBridgeSnapshotV0 {
    css: Option<String>,
}

impl TransformTextualBridgeSnapshotV0 {
    fn materialize_current_css<'a>(
        &'a mut self,
        document: &TransformExecutionDocumentV0,
    ) -> &'a str {
        self.css
            .get_or_insert_with(|| document.current_css().to_string())
            .as_str()
    }
}

impl TransformPassDispatchResultV0 {
    fn from_pair(
        next_textual_css: Option<String>,
        outcome: TransformPassExecutionOutcomeV0,
    ) -> Self {
        Self {
            next_textual_css,
            document_ir_updated: false,
            outcome,
            css_module_evaluation: None,
            css_import_inlines: Vec::new(),
            css_module_composes_exports: Vec::new(),
            design_token_routes: Vec::new(),
            semantic_removals: Vec::new(),
            provenance_mutation_spans: None,
        }
    }

    fn textual_mutation(
        pass_id: &'static str,
        input_byte_len: usize,
        next_css: String,
        mutation_count: usize,
        detail: &'static str,
    ) -> Self {
        let outcome = mutation_outcome(
            pass_id,
            input_byte_len,
            next_css.len(),
            mutation_count,
            detail,
        );
        Self::from_pair(Some(next_css), outcome)
    }

    fn ir_mutation(
        pass_id: &'static str,
        input_byte_len: usize,
        output_byte_len: usize,
        mutation_count: usize,
        detail: &'static str,
    ) -> Self {
        let outcome = mutation_outcome(
            pass_id,
            input_byte_len,
            output_byte_len,
            mutation_count,
            detail,
        );
        let mut result = Self::from_pair(None, outcome);
        result.document_ir_updated = true;
        result
    }

    fn planned_only(pass_id: &'static str, input_byte_len: usize, detail: &'static str) -> Self {
        Self::from_pair(
            None,
            planned_only_outcome(pass_id, input_byte_len, input_byte_len, detail),
        )
    }

    fn no_change(pass_id: &'static str, input_byte_len: usize, detail: &'static str) -> Self {
        Self::from_pair(
            None,
            no_change_outcome(pass_id, input_byte_len, input_byte_len, detail),
        )
    }
}

fn text_local_pass_handlers() -> &'static [TransformTextLocalPassHandlerV0] {
    &TEXT_LOCAL_PASS_HANDLERS
}

fn text_local_execution_mode_for_kind(
    kind: TransformPassKind,
) -> Option<TransformTextLocalExecutionModeV0> {
    text_local_pass_handlers()
        .iter()
        .find(|handler| handler.kind == kind)
        .map(|handler| handler.execution_mode)
}

#[derive(Clone, Copy)]
struct TransformTextLocalIrWindowV0<'a> {
    source: &'a str,
    source_span_start: usize,
    source_span_end: usize,
}

impl<'a> TransformTextLocalIrWindowV0<'a> {
    fn full_document(ir: &'a TransformIrV0) -> Vec<Self> {
        vec![Self {
            source: ir.source_text(),
            source_span_start: 0,
            source_span_end: ir.source_text().len(),
        }]
    }

    fn for_scope(ir: &'a TransformIrV0, scope: TransformTextLocalWindowScopeV0) -> Vec<Self> {
        match scope {
            TransformTextLocalWindowScopeV0::DocumentTokenStream
            | TransformTextLocalWindowScopeV0::Selector => Self::full_document(ir),
            TransformTextLocalWindowScopeV0::DeclarationBlock
            | TransformTextLocalWindowScopeV0::DeclarationValue => {
                Self::root_rule_windows(ir).unwrap_or_else(|| Self::full_document(ir))
            }
        }
    }

    fn root_rule_windows(ir: &'a TransformIrV0) -> Option<Vec<Self>> {
        let mut windows = ir
            .root_nodes
            .iter()
            .filter_map(|node_id| {
                let node = ir.nodes.get(node_id.index())?;
                matches!(node.kind, IrNodeKindV0::StyleRule | IrNodeKindV0::AtRule).then(|| Self {
                    source: ir.source_text(),
                    source_span_start: node.source_span_start,
                    source_span_end: node.source_span_end,
                })
            })
            .collect::<Vec<_>>();
        windows.sort_by_key(|window| (window.source_span_start, window.source_span_end));
        (!windows.is_empty() && windows_are_non_overlapping(windows.as_slice())).then_some(windows)
    }

    fn source_text(self) -> &'a str {
        self.source
            .get(self.source_span_start..self.source_span_end)
            .unwrap_or_default()
    }
}

fn windows_are_non_overlapping(windows: &[TransformTextLocalIrWindowV0<'_>]) -> bool {
    windows
        .windows(2)
        .all(|pair| pair[0].source_span_end <= pair[1].source_span_start)
}

struct TransformTextLocalPassInputV0<'a> {
    source: &'a str,
    source_windows: Vec<TransformTextLocalIrWindowV0<'a>>,
    execution_mode: TransformTextLocalExecutionModeV0,
    dialect: StyleDialect,
    context: &'a TransformExecutionContextV0,
}

impl<'a> TransformTextLocalPassInputV0<'a> {
    fn from_ir(
        ir: &'a TransformIrV0,
        scope: TransformTextLocalWindowScopeV0,
        execution_mode: TransformTextLocalExecutionModeV0,
        dialect: StyleDialect,
        context: &'a TransformExecutionContextV0,
    ) -> Self {
        Self {
            source: ir.source_text(),
            source_windows: TransformTextLocalIrWindowV0::for_scope(ir, scope),
            execution_mode,
            dialect,
            context,
        }
    }

    fn rewrite_text_local(
        self,
        rewrite: impl FnMut(&str, StyleDialect, &TransformExecutionContextV0) -> (String, usize),
    ) -> TransformTextLocalPassOutputV0<'a> {
        match self.execution_mode {
            TransformTextLocalExecutionModeV0::FullDocument => self.rewrite_full_document(rewrite),
            TransformTextLocalExecutionModeV0::WindowBatch => self.rewrite_windows(rewrite),
        }
    }

    fn rewrite_windows(
        self,
        mut rewrite: impl FnMut(&str, StyleDialect, &TransformExecutionContextV0) -> (String, usize),
    ) -> TransformTextLocalPassOutputV0<'a> {
        let mut rewrites = Vec::new();
        let mut mutation_count = 0usize;
        for window in &self.source_windows {
            let window_source = window.source_text();
            let (rewritten_css, window_mutation_count) =
                rewrite(window_source, self.dialect, self.context);
            mutation_count += window_mutation_count;
            if window_mutation_count > 0 || rewritten_css != window_source {
                rewrites.push(TransformTextLocalWindowRewriteV0 {
                    source_span_start: window.source_span_start,
                    source_span_end: window.source_span_end,
                    rewritten_css,
                });
            }
        }

        TransformTextLocalPassOutputV0 {
            input_byte_len: self.source.len(),
            rewritten_css: apply_text_local_window_rewrites(self.source, rewrites.as_slice()),
            provenance_mutation_spans: derive_text_local_window_mutation_spans(
                self.source,
                rewrites.as_slice(),
            ),
            mutation_count,
            _lifetime: std::marker::PhantomData,
        }
    }

    fn rewrite_full_document(
        self,
        mut rewrite: impl FnMut(&str, StyleDialect, &TransformExecutionContextV0) -> (String, usize),
    ) -> TransformTextLocalPassOutputV0<'a> {
        let (rewritten_css, mutation_count) = rewrite(self.source, self.dialect, self.context);
        let provenance_mutation_spans =
            derive_transform_mutation_spans(self.source, rewritten_css.as_str());
        TransformTextLocalPassOutputV0 {
            input_byte_len: self.source.len(),
            rewritten_css,
            provenance_mutation_spans,
            mutation_count,
            _lifetime: std::marker::PhantomData,
        }
    }
}

struct TransformTextLocalWindowRewriteV0 {
    source_span_start: usize,
    source_span_end: usize,
    rewritten_css: String,
}

struct TransformTextLocalPassOutputV0<'a> {
    input_byte_len: usize,
    rewritten_css: String,
    provenance_mutation_spans: Vec<TransformProvenanceMutationSpanV0>,
    mutation_count: usize,
    _lifetime: std::marker::PhantomData<&'a str>,
}

impl TransformTextLocalPassOutputV0<'_> {
    fn input_byte_len(&self) -> usize {
        self.input_byte_len
    }

    fn provenance_mutation_spans(&self) -> &[TransformProvenanceMutationSpanV0] {
        self.provenance_mutation_spans.as_slice()
    }

    fn into_document_css(self) -> String {
        self.rewritten_css
    }
}

fn apply_text_local_window_rewrites(
    source: &str,
    rewrites: &[TransformTextLocalWindowRewriteV0],
) -> String {
    if rewrites.is_empty() {
        return source.to_string();
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0usize;
    for rewrite in rewrites {
        if rewrite.source_span_start < cursor {
            continue;
        }
        if rewrite.source_span_start > cursor {
            output.push_str(&source[cursor..rewrite.source_span_start]);
        }
        output.push_str(&rewrite.rewritten_css);
        cursor = rewrite.source_span_end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }
    output
}

fn derive_text_local_window_mutation_spans(
    source: &str,
    rewrites: &[TransformTextLocalWindowRewriteV0],
) -> Vec<TransformProvenanceMutationSpanV0> {
    let mut spans = Vec::new();
    let mut generated_delta = 0isize;
    for rewrite in rewrites {
        let Some(source_window) = source.get(rewrite.source_span_start..rewrite.source_span_end)
        else {
            let rewritten_css = apply_text_local_window_rewrites(source, rewrites);
            return derive_transform_mutation_spans(source, rewritten_css.as_str());
        };
        let Some(generated_window_start) =
            add_signed_offset(rewrite.source_span_start, generated_delta)
        else {
            let rewritten_css = apply_text_local_window_rewrites(source, rewrites);
            return derive_transform_mutation_spans(source, rewritten_css.as_str());
        };
        spans.extend(
            derive_transform_mutation_spans(source_window, &rewrite.rewritten_css)
                .into_iter()
                .map(|span| TransformProvenanceMutationSpanV0 {
                    source_span_start: rewrite.source_span_start + span.source_span_start,
                    source_span_end: rewrite.source_span_start + span.source_span_end,
                    generated_span_start: generated_window_start + span.generated_span_start,
                    generated_span_end: generated_window_start + span.generated_span_end,
                    node_key: None,
                }),
        );
        generated_delta += rewrite.rewritten_css.len() as isize
            - rewrite
                .source_span_end
                .saturating_sub(rewrite.source_span_start) as isize;
    }
    spans
}

fn add_signed_offset(value: usize, offset: isize) -> Option<usize> {
    if offset >= 0 {
        value.checked_add(offset as usize)
    } else {
        value.checked_sub((-offset) as usize)
    }
}

type TransformStructuralRunnerV0 =
    for<'a> fn(TransformStructuralPassInputV0<'a>) -> TransformPassDispatchResultV0;

#[derive(Clone, Copy)]
struct TransformStructuralPassHandlerV0 {
    kind: TransformPassKind,
    run: TransformStructuralRunnerV0,
}

struct TransformStructuralPassInputV0<'a> {
    pass_id: &'static str,
    current_ir: &'a mut TransformIrV0,
    input_byte_len: usize,
    dialect: StyleDialect,
    context: &'a TransformExecutionContextV0,
    reachable_class_names: &'a [String],
}

impl TransformStructuralPassInputV0<'_> {
    fn current_ir_mut(&mut self) -> &mut TransformIrV0 {
        self.current_ir
    }

    fn ir_mutation_result(
        &self,
        mutation_count: usize,
        detail: &'static str,
    ) -> TransformPassDispatchResultV0 {
        TransformPassDispatchResultV0::ir_mutation(
            self.pass_id,
            self.input_byte_len,
            self.current_ir.source_text().len(),
            mutation_count,
            detail,
        )
    }
}

fn structural_pass_handlers() -> &'static [TransformStructuralPassHandlerV0] {
    &STRUCTURAL_PASS_HANDLERS
}

struct TransformExecutionDocumentV0 {
    current_ir: TransformIrV0,
    dialect: StyleDialect,
}

impl TransformExecutionDocumentV0 {
    fn new(source: &str, dialect: StyleDialect) -> Self {
        Self {
            current_ir: lower_transform_ir_from_source(
                source,
                dialect,
                "omena-transform-passes.execution.current",
            ),
            dialect,
        }
    }

    fn current_ir_mut(&mut self) -> &mut TransformIrV0 {
        &mut self.current_ir
    }

    fn current_css(&self) -> &str {
        self.current_ir.source_text()
    }

    fn current_byte_len(&self) -> usize {
        self.current_css().len()
    }

    fn replace_with_css(&mut self, css: String) {
        self.current_ir = lower_transform_ir_from_source(
            css.as_str(),
            self.dialect,
            "omena-transform-passes.execution.current",
        );
    }

    fn output_css(&self) -> String {
        self.current_css().to_string()
    }
}

static TEXT_LOCAL_PASS_HANDLERS: [TransformTextLocalPassHandlerV0; 20] = [
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::WhitespaceStrip,
        window_scope: TransformTextLocalWindowScopeV0::DocumentTokenStream,
        execution_mode: TransformTextLocalExecutionModeV0::FullDocument,
        detail: "normalized lexer trivia where adjacent token boundaries remain unambiguous",
        run: run_whitespace_strip_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::CommentStrip,
        window_scope: TransformTextLocalWindowScopeV0::DocumentTokenStream,
        execution_mode: TransformTextLocalExecutionModeV0::FullDocument,
        detail: "removed CSS block comments outside string literals",
        run: run_comment_strip_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::NumberCompression,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "compressed lexer numeric tokens without touching identifiers or strings",
        run: run_number_compression_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::UnitNormalization,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "normalized zero length units and known CSS unit casing inside declaration contexts",
        run: run_unit_normalization_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::ColorCompression,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "compressed static declaration color values and hex color tokens",
        run: run_color_compression_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::UrlQuoteStrip,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "stripped quotes from safe url() string arguments",
        run: run_url_quote_strip_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::StringQuoteNormalize,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "normalized safe CSS string tokens, declaration-scoped font family strings, and static font keyword aliases",
        run: run_string_quote_normalize_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::SelectorIsWhereCompression,
        window_scope: TransformTextLocalWindowScopeV0::Selector,
        execution_mode: TransformTextLocalExecutionModeV0::FullDocument,
        detail: "compressed :is/:where selector functions and keyframe selector aliases only when matching semantics are preserved",
        run: run_selector_is_where_compression_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::ShorthandCombining,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationBlock,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "combined safe shorthand declarations and adjacent longhands only with cascade-preserving proofs",
        run: run_shorthand_combining_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::VendorPrefixing,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationBlock,
        execution_mode: TransformTextLocalExecutionModeV0::FullDocument,
        detail: "inserted target-aware vendor-prefixed declaration synonyms when absent",
        run: run_vendor_prefixing_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::StalePrefixRemoval,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationBlock,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "removed explicit stale prefixed declarations only when an exact unprefixed peer proves equivalence",
        run: run_stale_prefix_removal_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::LightDarkLowering,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "lowered light-dark() color references into dark media branches",
        run: run_light_dark_lowering_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::ColorMixLowering,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "lowered static srgb color-mix() references with static color operands",
        run: run_color_mix_lowering_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::OklchOklabLowering,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "lowered in-gamut oklab()/oklch() color references to srgb",
        run: run_oklch_oklab_lowering_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::ColorFunctionLowering,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "lowered static color(...) references with static channels",
        run: run_color_function_lowering_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::RelativeColorLowering,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "lowered static rgb(from ...) relative-color references to absolute srgb",
        run: run_relative_color_lowering_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::LogicalToPhysical,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationBlock,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "lowered logical properties only under static horizontal writing direction",
        run: run_logical_to_physical_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::ValueResolution,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::FullDocument,
        detail: "resolved whole-value references from unique local literal CSS Modules @value declarations",
        run: run_value_resolution_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::StaticVarSubstitution,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::FullDocument,
        detail: "resolved whole-value var() references from unique static :root custom properties",
        run: run_static_var_substitution_text_local,
    },
    TransformTextLocalPassHandlerV0 {
        kind: TransformPassKind::CalcReduction,
        window_scope: TransformTextLocalWindowScopeV0::DeclarationValue,
        execution_mode: TransformTextLocalExecutionModeV0::WindowBatch,
        detail: "reduced whole-value CSS math functions with static same-unit arithmetic and identity operations",
        run: run_calc_reduction_text_local,
    },
];

static STRUCTURAL_PASS_HANDLERS: [TransformStructuralPassHandlerV0; 21] = [
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::ImportInline,
        run: run_import_inline_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::ResolveCssModulesComposes,
        run: run_resolve_css_modules_composes_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::DesignTokenRouting,
        run: run_design_token_routing_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::HashCssModuleClassNames,
        run: run_hash_css_module_class_names_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::RuleDeduplication,
        run: run_rule_deduplication_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::RuleMerging,
        run: run_rule_merging_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::SelectorMerging,
        run: run_selector_merging_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::NestingUnwrap,
        run: run_nesting_unwrap_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::ScopeFlatten,
        run: run_scope_flatten_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::LayerFlatten,
        run: run_layer_flatten_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::SupportsStaticEval,
        run: run_supports_static_eval_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::MediaStaticEval,
        run: run_media_static_eval_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::ContainerStaticEval,
        run: run_container_static_eval_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::NativeCssStaticEval,
        run: run_native_css_static_eval_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::DeadMediaBranchRemoval,
        run: run_dead_media_branch_removal_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::DeadSupportsBranchRemoval,
        run: run_dead_supports_branch_removal_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::TreeShakeClass,
        run: run_tree_shake_class_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::TreeShakeKeyframes,
        run: run_tree_shake_keyframes_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::TreeShakeValue,
        run: run_tree_shake_value_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::TreeShakeCustomProperty,
        run: run_tree_shake_custom_property_structural,
    },
    TransformStructuralPassHandlerV0 {
        kind: TransformPassKind::EmptyRuleRemoval,
        run: run_empty_rule_removal_structural,
    },
];

pub fn execute_transform_passes_on_source(
    source: &str,
    requested: &[TransformPassKind],
) -> TransformExecutionSummaryV0 {
    execute_transform_passes_on_source_with_dialect(source, StyleDialect::Css, requested)
}

pub fn execute_transform_passes_on_source_with_dialect(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
) -> TransformExecutionSummaryV0 {
    let context = TransformExecutionContextV0::default();
    execute_transform_passes_on_source_with_dialect_and_context(
        source, dialect, requested, &context,
    )
}

#[cfg(feature = "lawvere-trace")]
pub fn execute_transform_passes_on_source_with_lawvere_trace(
    source: &str,
    requested: &[TransformPassKind],
) -> (
    TransformExecutionSummaryV0,
    omena_lawvere::LawvereModelTraceV0,
) {
    execute_transform_passes_on_source_with_lawvere_trace_and_dialect(
        source,
        StyleDialect::Css,
        requested,
    )
}

#[cfg(feature = "lawvere-trace")]
pub fn execute_transform_passes_on_source_with_lawvere_trace_and_dialect(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
) -> (
    TransformExecutionSummaryV0,
    omena_lawvere::LawvereModelTraceV0,
) {
    let summary = execute_transform_passes_on_source_with_dialect(source, dialect, requested);
    let trace = omena_lawvere::trace_lawvere_model_v0(requested, summary.ordered_pass_ids.clone());
    (summary, trace)
}

#[cfg(feature = "lawvere-trace")]
pub fn evaluate_lawvere_reorderability_with_differential_corpus(
    left: TransformPassKind,
    right: TransformPassKind,
    fixtures: &[&str],
) -> (
    omena_lawvere::ReorderabilityCertificateV0,
    omena_lawvere::LawvereDifferentialCommutativityWitnessV0,
) {
    let cases = fixtures
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let left_first = execute_transform_passes_on_source(source, &[left]);
            let left_then_right =
                execute_transform_passes_on_source(&left_first.output_css, &[right]);
            let right_first = execute_transform_passes_on_source(source, &[right]);
            let right_then_left =
                execute_transform_passes_on_source(&right_first.output_css, &[left]);
            let left_then_right_mutation_count =
                left_first.mutation_count + left_then_right.mutation_count;
            let right_then_left_mutation_count =
                right_first.mutation_count + right_then_left.mutation_count;

            omena_lawvere::LawvereDifferentialCommutativityCaseV0 {
                label: format!("fixture-{index}"),
                input_css: (*source).to_string(),
                left_then_right_css: left_then_right.output_css.clone(),
                right_then_left_css: right_then_left.output_css.clone(),
                left_then_right_mutation_count,
                right_then_left_mutation_count,
                equal_output: left_then_right.output_css == right_then_left.output_css,
            }
        })
        .collect::<Vec<_>>();
    let witness = omena_lawvere::lawvere_differential_commutativity_witness_v0(left, right, cases);
    let certificate =
        omena_lawvere::reorderability_certificate_from_differential_v0(left, right, &witness);
    (certificate, witness)
}

pub fn execute_transform_passes_on_source_with_dialect_and_context(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
) -> TransformExecutionSummaryV0 {
    super::lex_cache::with_transform_lex_cache(|| {
        execute_transform_passes_on_source_with_active_lex_cache(
            source, dialect, requested, context,
        )
    })
}

#[doc(hidden)]
pub fn execute_transform_passes_on_source_with_dialect_and_context_without_lex_cache_for_measurement(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
) -> TransformExecutionSummaryV0 {
    execute_transform_passes_on_source_with_active_lex_cache(source, dialect, requested, context)
}

fn dispatch_text_local_pass(
    pass_id: &'static str,
    handler: TransformTextLocalPassHandlerV0,
    current_ir: &TransformIrV0,
    dialect: StyleDialect,
    context: &TransformExecutionContextV0,
) -> Option<TransformPassDispatchResultV0> {
    debug_assert_eq!(
        omena_transform_cst::transform_pass_class(handler.kind),
        TransformPassClassV0::TextLocal
    );
    let input = TransformTextLocalPassInputV0::from_ir(
        current_ir,
        handler.window_scope,
        handler.execution_mode,
        dialect,
        context,
    );
    let output = (handler.run)(input);
    let input_byte_len = output.input_byte_len();
    let mutation_count = output.mutation_count;
    let provenance_mutation_spans = output.provenance_mutation_spans().to_vec();
    let next_css = output.into_document_css();
    let mut result = TransformPassDispatchResultV0::textual_mutation(
        pass_id,
        input_byte_len,
        next_css,
        mutation_count,
        handler.detail,
    );
    if !provenance_mutation_spans.is_empty() {
        result.provenance_mutation_spans = Some(provenance_mutation_spans);
    }
    Some(result)
}

fn run_whitespace_strip_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| normalize_css_whitespace(source, dialect))
}

fn run_comment_strip_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| strip_css_comments(source, dialect))
}

fn run_number_compression_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| compress_css_numbers(source, dialect))
}

fn run_unit_normalization_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| normalize_css_units(source, dialect))
}

fn run_color_compression_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| compress_css_colors(source, dialect))
}

fn run_url_quote_strip_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| strip_css_url_quotes(source, dialect))
}

fn run_string_quote_normalize_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| {
        normalize_css_string_quotes(source, dialect)
    })
}

fn run_selector_is_where_compression_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| {
        compress_css_is_where_selectors(source, dialect)
    })
}

fn run_shorthand_combining_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| combine_css_shorthands(source, dialect))
}

fn run_vendor_prefixing_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, context| {
        let vendor_prefix_policy = context
            .vendor_prefix_policy
            .unwrap_or_else(TransformVendorPrefixPolicyV0::conservative);
        add_css_vendor_prefixes(source, dialect, vendor_prefix_policy)
    })
}

fn run_stale_prefix_removal_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| {
        remove_stale_css_vendor_prefixes(source, dialect)
    })
}

fn run_light_dark_lowering_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| lower_css_light_dark(source, dialect))
}

fn run_color_mix_lowering_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| lower_css_color_mix(source, dialect))
}

fn run_oklch_oklab_lowering_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| lower_css_oklab_oklch(source, dialect))
}

fn run_color_function_lowering_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| lower_css_color_function(source, dialect))
}

fn run_relative_color_lowering_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| lower_relative_color(source, dialect))
}

fn run_logical_to_physical_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| {
        lower_css_logical_to_physical(source, dialect)
    })
}

fn run_value_resolution_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, context| {
        resolve_static_css_modules_values(source, dialect, &context.css_module_value_resolutions)
    })
}

fn run_static_var_substitution_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| {
        substitute_static_css_custom_properties(source, dialect)
    })
}

fn run_calc_reduction_text_local(
    input: TransformTextLocalPassInputV0<'_>,
) -> TransformTextLocalPassOutputV0<'_> {
    input.rewrite_text_local(|source, dialect, _context| reduce_css_calc(source, dialect))
}

fn dispatch_module_evaluation_pass(
    pass_id: &'static str,
    pass: TransformPassKind,
    input_css: &str,
    dialect: StyleDialect,
    context: &TransformExecutionContextV0,
) -> Option<TransformPassDispatchResultV0> {
    let input_byte_len = input_css.len();
    match pass {
        TransformPassKind::ScssModuleEvaluate
            if matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) =>
        {
            if let Some(evaluation) = context.scss_module_evaluation.as_ref() {
                let materialized = materialize_transform_module_evaluation_output(
                    input_css,
                    evaluation,
                    "applied explicit SCSS module evaluation native edit output from the evaluator boundary",
                    "preserved SCSS source because native evaluator edits did not match the oracle boundary",
                );
                let mutation_count = usize::from(input_css != materialized.css);
                let mut result = TransformPassDispatchResultV0::textual_mutation(
                    pass_id,
                    input_byte_len,
                    materialized.css,
                    mutation_count,
                    materialized.detail,
                );
                result.css_module_evaluation = Some(evaluation.clone());
                Some(result)
            } else {
                Some(TransformPassDispatchResultV0::planned_only(
                    pass_id,
                    input_byte_len,
                    "requires explicit SCSS evaluator output before mutation",
                ))
            }
        }
        TransformPassKind::ScssModuleEvaluate => Some(TransformPassDispatchResultV0::planned_only(
            pass_id,
            input_byte_len,
            "requires explicit SCSS evaluator output before mutation",
        )),
        TransformPassKind::LessModuleEvaluate if dialect == StyleDialect::Less => {
            if let Some(evaluation) = context.less_module_evaluation.as_ref() {
                let materialized = materialize_transform_module_evaluation_output(
                    input_css,
                    evaluation,
                    "applied explicit Less module evaluation native edit output from the evaluator boundary",
                    "preserved Less source because native evaluator edits did not match the oracle boundary",
                );
                let mutation_count = usize::from(input_css != materialized.css);
                let mut result = TransformPassDispatchResultV0::textual_mutation(
                    pass_id,
                    input_byte_len,
                    materialized.css,
                    mutation_count,
                    materialized.detail,
                );
                result.css_module_evaluation = Some(evaluation.clone());
                Some(result)
            } else {
                Some(TransformPassDispatchResultV0::planned_only(
                    pass_id,
                    input_byte_len,
                    "requires explicit Less evaluator output before mutation",
                ))
            }
        }
        TransformPassKind::LessModuleEvaluate => Some(TransformPassDispatchResultV0::planned_only(
            pass_id,
            input_byte_len,
            "requires explicit Less evaluator output before mutation",
        )),
        _ => None,
    }
}

fn dispatch_structural_pass(
    pass_id: &'static str,
    handler: TransformStructuralPassHandlerV0,
    current_ir: &mut TransformIrV0,
    dialect: StyleDialect,
    context: &TransformExecutionContextV0,
    reachable_class_names: &[String],
) -> Option<TransformPassDispatchResultV0> {
    debug_assert_eq!(
        omena_transform_cst::transform_pass_class(handler.kind),
        TransformPassClassV0::Structural
    );
    let input_byte_len = current_ir.source_text().len();
    reset_structural_ir_transaction_mutation_span_batches();
    let mut result = (handler.run)(TransformStructuralPassInputV0 {
        pass_id,
        current_ir,
        input_byte_len,
        dialect,
        context,
        reachable_class_names,
    });
    let span_batches = take_structural_ir_transaction_mutation_span_batches();
    if let Some(mutation_spans) = compose_ir_transaction_mutation_span_batches(
        input_byte_len,
        current_ir.source_text().len(),
        span_batches.as_slice(),
    ) {
        result.provenance_mutation_spans = Some(mutation_spans);
    }
    Some(result)
}

fn dispatch_emission_pass(
    pass_id: &'static str,
    pass: TransformPassKind,
    input_byte_len: usize,
) -> Option<TransformPassDispatchResultV0> {
    match pass {
        TransformPassKind::PrintCss => Some(TransformPassDispatchResultV0::no_change(
            pass_id,
            input_byte_len,
            "observed final emission boundary",
        )),
        _ => None,
    }
}

fn run_import_inline_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    if input.dialect != StyleDialect::Less && input.context.import_inlines.is_empty() {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires explicit resolved import replacements before mutation",
        );
    }
    let dialect = input.dialect;
    let import_inlines = input.context.import_inlines.clone();
    let Ok(mutation_count) =
        inline_css_imports_in_ir(input.current_ir_mut(), dialect, &import_inlines)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the import-inline structural rewrite",
        );
    };
    let mut result = input.ir_mutation_result(
        mutation_count,
        "replaced resolved @import directives and optional Less imports",
    );
    result.css_import_inlines = import_inlines;
    result
}

fn run_resolve_css_modules_composes_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let explicit_resolutions = input.context.css_module_composes_resolutions.clone();
    let resolutions =
        css_module_composes_resolutions_for_ir(input.current_ir, dialect, &explicit_resolutions);
    if resolutions.is_empty() {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires CSS Modules composes declarations or an explicit export set before mutation",
        );
    }
    let Ok(mutation_count) =
        resolve_css_module_composes_in_ir(input.current_ir_mut(), dialect, &resolutions)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the CSS Modules composes structural rewrite",
        );
    };
    let mut result = input.ir_mutation_result(
        mutation_count,
        "removed resolved CSS Modules composes declarations using an explicit export set",
    );
    result.css_module_composes_exports = resolutions;
    result
}

fn run_design_token_routing_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    if input.context.design_token_routes.is_empty() {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires explicit bridge design-token routes before mutation",
        );
    }
    let dialect = input.dialect;
    let design_token_routes = input.context.design_token_routes.clone();
    let Ok(mutation_count) =
        route_design_token_values_in_ir(input.current_ir_mut(), dialect, &design_token_routes)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the design-token structural rewrite",
        );
    };
    let mut result = input.ir_mutation_result(
        mutation_count,
        "routed whole-value design-token references through explicit bridge token routes",
    );
    result.design_token_routes = design_token_routes;
    result
}

fn run_hash_css_module_class_names_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let class_name_rewrites = input.context.class_name_rewrites.clone();
    let Ok(mutation_count) =
        rewrite_css_module_class_names_in_ir(input.current_ir_mut(), dialect, &class_name_rewrites)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the CSS Modules class hashing structural rewrite",
        );
    };
    if mutation_count == 0 && class_name_rewrites.is_empty() {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires an explicit selector identity map or CSS Modules scope markers before mutation",
        );
    }
    if mutation_count == 0 {
        return TransformPassDispatchResultV0::no_change(
            input.pass_id,
            input.input_byte_len,
            "observed CSS Modules class hashing boundary without matching selector rewrites",
        );
    }
    input.ir_mutation_result(
        mutation_count,
        "rewrote CSS Modules class selectors and scope markers through the structural IR boundary",
    )
}

fn run_rule_deduplication_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let mutation_count = match dedupe_exact_css_rules_in_ir(input.current_ir_mut(), dialect) {
        Ok(result) => result,
        Err(_) => {
            return TransformPassDispatchResultV0::planned_only(
                input.pass_id,
                input.input_byte_len,
                "typed IR transaction rejected the rule deduplication rewrite",
            );
        }
    };
    input.ir_mutation_result(
        mutation_count,
        "removed cascade-safe duplicate ordinary rules while preserving the final occurrence",
    )
}

fn run_rule_merging_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) =
        merge_adjacent_same_selector_css_rules_in_ir(input.current_ir_mut(), dialect)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the rule merging rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "merged adjacent same-selector ordinary rule runs without reordering declarations",
    )
}

fn run_selector_merging_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) =
        merge_adjacent_same_block_css_selectors_in_ir(input.current_ir_mut(), dialect)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the selector merging rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "merged adjacent ordinary rule runs with identical declaration blocks",
    )
}

fn run_nesting_unwrap_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) = unwrap_css_nesting_in_ir(input.current_ir_mut(), dialect) else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the nesting structural rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "unwrapped nested ordinary rules and conditional group rules",
    )
}

fn run_scope_flatten_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) = flatten_css_scopes_in_ir(input.current_ir_mut(), dialect) else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the scope structural rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "flattened only @scope candidates accepted by the cascade scope-flatten proof",
    )
}

fn run_layer_flatten_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    if input.context.closed_style_world {
        let dialect = input.dialect;
        let Ok(mutation_count) = flatten_css_layers_in_ir(input.current_ir_mut(), dialect, true)
        else {
            return TransformPassDispatchResultV0::planned_only(
                input.pass_id,
                input.input_byte_len,
                "typed IR transaction rejected the layer structural rewrite",
            );
        };
        input.ir_mutation_result(
            mutation_count,
            "flattened only @layer candidates accepted by the closed-bundle cascade proof",
        )
    } else {
        TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires an explicit closed-style-world bundle witness before mutation",
        )
    }
}

fn run_supports_static_eval_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) = evaluate_static_supports_rules_in_ir(input.current_ir_mut(), dialect)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the supports static structural rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "evaluated simple @supports branches with cascade supports-static witness",
    )
}

fn run_media_static_eval_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) = evaluate_static_media_rules_in_ir(input.current_ir_mut(), dialect)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the media static structural rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "evaluated literal @media all/not all branches and normalized simple min/max media ranges",
    )
}

fn run_container_static_eval_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) = evaluate_static_container_rules_in_ir(input.current_ir_mut(), dialect)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the container static structural rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "removed @container branches whose size condition is provably unsatisfiable",
    )
}

fn run_native_css_static_eval_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    if input.dialect == StyleDialect::Css {
        let dialect = input.dialect;
        let Ok(mutation_count) =
            evaluate_native_css_static_values_in_ir(input.current_ir_mut(), dialect)
        else {
            return TransformPassDispatchResultV0::planned_only(
                input.pass_id,
                input.input_byte_len,
                "typed IR transaction rejected the native CSS static structural rewrite",
            );
        };
        input.ir_mutation_result(
            mutation_count,
            "folded fully static native CSS if() values and native CSS function calls while preserving runtime-dependent constructs",
        )
    } else {
        TransformPassDispatchResultV0::no_change(
            input.pass_id,
            input.input_byte_len,
            "preserved non-CSS dialect because native CSS static evaluation is CSS-only",
        )
    }
}

fn run_dead_media_branch_removal_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let drop_dark_mode_media_queries = input.context.drop_dark_mode_media_queries;
    let Ok(mutation_count) = evaluate_dead_media_branch_rules_in_ir(
        input.current_ir_mut(),
        dialect,
        drop_dark_mode_media_queries,
    ) else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the dead media structural rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "removed dead @media branches through the static cascade witness evaluator",
    )
}

fn run_dead_supports_branch_removal_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) = evaluate_static_supports_rules_in_ir(input.current_ir_mut(), dialect)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the dead supports structural rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "removed dead @supports branches through the static cascade witness evaluator",
    )
}

fn run_tree_shake_class_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    if !input.context.closed_style_world {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires an explicit closed-style-world reachability context before mutation",
        );
    }
    let dialect = input.dialect;
    let reachable_class_names = input.reachable_class_names.to_vec();
    let Ok(removals) =
        tree_shake_css_class_rules_in_ir(input.current_ir_mut(), dialect, &reachable_class_names)
    else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the class tree-shake structural rewrite",
        );
    };
    let mutation_count = removals.len();
    let mut result = input.ir_mutation_result(
        mutation_count,
        "removed unreachable class-owned selector rules under an explicit closed-style-world reachability context",
    );
    result.semantic_removals = removals
        .into_iter()
        .map(|removal| removal.into_public(input.pass_id))
        .collect();
    result
}

fn run_tree_shake_keyframes_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    if !input.context.closed_style_world {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires an explicit closed-style-world reachability context before mutation",
        );
    }
    let dialect = input.dialect;
    let reachable_keyframe_names = input.context.reachable_keyframe_names.clone();
    let reachable_class_names = input.reachable_class_names.to_vec();
    let Ok(removals) = tree_shake_css_keyframes_in_ir(
        input.current_ir_mut(),
        dialect,
        &reachable_keyframe_names,
        &reachable_class_names,
    ) else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the keyframes tree-shake structural rewrite",
        );
    };
    let mutation_count = removals.len();
    let mut result = input.ir_mutation_result(
        mutation_count,
        "removed unreferenced @keyframes under an explicit closed-style-world reachability context",
    );
    result.semantic_removals = removals
        .into_iter()
        .map(|removal| removal.into_public(input.pass_id))
        .collect();
    result
}

fn run_tree_shake_value_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    if !input.context.closed_style_world {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires an explicit closed-style-world reachability context before mutation",
        );
    }
    let dialect = input.dialect;
    let reachable_value_names = input.context.reachable_value_names.clone();
    let reachable_keyframe_names = input.context.reachable_keyframe_names.clone();
    let reachable_class_names = input.reachable_class_names.to_vec();
    let Ok(removals) = tree_shake_css_modules_values_in_ir(
        input.current_ir_mut(),
        dialect,
        &reachable_value_names,
        &reachable_keyframe_names,
        &reachable_class_names,
    ) else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the CSS Modules value tree-shake structural rewrite",
        );
    };
    let mutation_count = removals.len();
    let mut result = input.ir_mutation_result(
        mutation_count,
        "removed unreachable local CSS Modules @value declarations under an explicit closed-style-world reachability context",
    );
    result.semantic_removals = removals
        .into_iter()
        .map(|removal| removal.into_public(input.pass_id))
        .collect();
    result
}

fn run_tree_shake_custom_property_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    if !input.context.closed_style_world {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "requires an explicit closed-style-world reachability context before mutation",
        );
    }
    let dialect = input.dialect;
    let reachable_custom_property_names = input.context.reachable_custom_property_names.clone();
    let reachable_keyframe_names = input.context.reachable_keyframe_names.clone();
    let reachable_class_names = input.reachable_class_names.to_vec();
    let Ok(removals) = tree_shake_css_custom_properties_in_ir(
        input.current_ir_mut(),
        dialect,
        &reachable_custom_property_names,
        &reachable_keyframe_names,
        &reachable_class_names,
    ) else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the custom-property tree-shake structural rewrite",
        );
    };
    let mutation_count = removals.len();
    let mut result = input.ir_mutation_result(
        mutation_count,
        "removed unreachable custom-property declarations under an explicit closed-style-world reachability context",
    );
    result.semantic_removals = removals
        .into_iter()
        .map(|removal| removal.into_public(input.pass_id))
        .collect();
    result
}

fn run_empty_rule_removal_structural(
    mut input: TransformStructuralPassInputV0<'_>,
) -> TransformPassDispatchResultV0 {
    let dialect = input.dialect;
    let Ok(mutation_count) = remove_empty_css_rules_in_ir(input.current_ir_mut(), dialect) else {
        return TransformPassDispatchResultV0::planned_only(
            input.pass_id,
            input.input_byte_len,
            "typed IR transaction rejected the empty-rule structural rewrite",
        );
    };
    input.ir_mutation_result(
        mutation_count,
        "removed ordinary empty rules with no comments or at-rule semantics",
    )
}

fn execute_transform_passes_on_source_with_active_lex_cache(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
) -> TransformExecutionSummaryV0 {
    reset_structural_ir_transaction_telemetry();
    let pass_plan = plan_transform_passes(requested);
    let pass_registry = default_transform_pass_registry();
    let stable_ir =
        build_stable_transform_ir_from_source(source, dialect, "omena-transform-passes.execution");
    let stable_ir_nodes = stable_ir.nodes;
    let mut coordinate_map = TransformSpanCoordinateMapV0::new(source.len());
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let ordered_pass_ids = pass_plan.ordered_pass_ids.clone();
    let mut document = TransformExecutionDocumentV0::new(source, dialect);
    let reachable_class_names = reachable_class_names_with_composes_exports(
        &document.current_ir,
        &context.reachable_class_names,
        &context.css_module_composes_resolutions,
    );
    let mut outcomes = Vec::new();
    let mut css_module_evaluation = None;
    let mut css_import_inlines = Vec::new();
    let mut css_module_composes_exports = Vec::new();
    let mut design_token_routes = Vec::new();
    let mut semantic_removals = Vec::new();
    let mut outcome_mutation_spans = Vec::new();
    let mut cascade_proof_obligations = Vec::new();

    for (pass_index, pass_id) in ordered_pass_ids.iter().enumerate() {
        let should_maintain_document_lex_cache =
            next_document_lex_cache_consumer(ordered_pass_ids.as_slice(), pass_index)
                == Some(TransformTextLocalExecutionModeV0::FullDocument);
        let pass = transform_pass_kind_from_id(pass_id);
        let runtime_entry = pass
            .and_then(|kind| runtime_pass_entry_for_kind(kind, pass_registry.entries.as_slice()));
        let dispatch_kind = runtime_entry
            .as_ref()
            .map(|entry| entry.registry_entry.dispatch_kind);
        let input_byte_len = document.current_byte_len();
        let mut textual_bridge = TransformTextualBridgeSnapshotV0::default();
        if dispatch_kind == Some(TransformPassDispatchKindV0::StructuralIrTransaction) {
            cascade_proof_obligations.extend(collect_cascade_proof_obligations_for_ir_pass_input(
                pass_id,
                pass,
                &document.current_ir,
                context,
            ));
        } else {
            let pass_input_css = textual_bridge.materialize_current_css(&document);
            cascade_proof_obligations.extend(collect_cascade_proof_obligations_for_pass_input(
                pass_id,
                pass,
                pass_input_css,
                dialect,
                context,
            ));
        }
        let dispatch_result = match runtime_entry.as_ref().map(|entry| entry.implementation) {
            Some(TransformRuntimePassImplementationV0::TextLocal(handler)) => {
                dispatch_text_local_pass(pass_id, handler, &document.current_ir, dialect, context)
            }
            Some(TransformRuntimePassImplementationV0::ModuleEvaluation(pass)) => {
                let pass_input_css = textual_bridge.materialize_current_css(&document);
                dispatch_module_evaluation_pass(pass_id, pass, pass_input_css, dialect, context)
            }
            Some(TransformRuntimePassImplementationV0::Structural(handler)) => {
                if should_maintain_document_lex_cache {
                    textual_bridge.materialize_current_css(&document);
                }
                dispatch_structural_pass(
                    pass_id,
                    handler,
                    document.current_ir_mut(),
                    dialect,
                    context,
                    &reachable_class_names,
                )
            }
            Some(TransformRuntimePassImplementationV0::Emission(pass)) => {
                dispatch_emission_pass(pass_id, pass, input_byte_len)
            }
            None => None,
        }
        .unwrap_or_else(|| {
            TransformPassDispatchResultV0::planned_only(
                pass_id,
                input_byte_len,
                "unknown pass id in execution plan",
            )
        });
        let TransformPassDispatchResultV0 {
            next_textual_css,
            document_ir_updated,
            outcome,
            css_module_evaluation: dispatched_css_module_evaluation,
            css_import_inlines: dispatched_css_import_inlines,
            css_module_composes_exports: dispatched_css_module_composes_exports,
            design_token_routes: dispatched_design_token_routes,
            semantic_removals: dispatched_semantic_removals,
            provenance_mutation_spans,
        } = dispatch_result;
        if let Some(evaluation) = dispatched_css_module_evaluation {
            css_module_evaluation = Some(evaluation);
        }
        if !dispatched_css_import_inlines.is_empty() {
            css_import_inlines = dispatched_css_import_inlines;
        }
        if !dispatched_css_module_composes_exports.is_empty() {
            css_module_composes_exports = dispatched_css_module_composes_exports;
        }
        if !dispatched_design_token_routes.is_empty() {
            design_token_routes = dispatched_design_token_routes;
        }
        semantic_removals.extend(dispatched_semantic_removals);
        if document_ir_updated {
            let output_byte_len = document.current_byte_len();
            let mut mutation_spans = match provenance_mutation_spans {
                Some(mutation_spans) => mutation_spans,
                None => {
                    vec![TransformProvenanceMutationSpanV0 {
                        source_span_start: 0,
                        source_span_end: input_byte_len,
                        generated_span_start: 0,
                        generated_span_end: output_byte_len,
                        node_key: None,
                    }]
                }
            };
            stamp_mutation_span_node_keys(
                mutation_spans.as_mut_slice(),
                &coordinate_map,
                stable_ir_nodes.as_slice(),
            );
            if should_maintain_document_lex_cache {
                let next_css = document.current_css().to_string();
                let pass_input_css = textual_bridge.materialize_current_css(&document);
                super::lex_cache::update_cached_lex_from_splice(
                    pass_input_css,
                    &next_css,
                    dialect,
                    mutation_spans.as_slice(),
                );
            }
            coordinate_map.apply_mutation_spans(mutation_spans.as_slice());
            outcome_mutation_spans.push(mutation_spans);
        } else {
            match next_textual_css {
                Some(next_css) => {
                    let mut mutation_spans = match provenance_mutation_spans {
                        Some(mutation_spans) => mutation_spans,
                        _ => {
                            let pass_input_css = textual_bridge.materialize_current_css(&document);
                            derive_transform_mutation_spans(pass_input_css, &next_css)
                        }
                    };
                    stamp_mutation_span_node_keys(
                        mutation_spans.as_mut_slice(),
                        &coordinate_map,
                        stable_ir_nodes.as_slice(),
                    );
                    if should_maintain_document_lex_cache {
                        let pass_input_css = textual_bridge.materialize_current_css(&document);
                        super::lex_cache::update_cached_lex_from_splice(
                            pass_input_css,
                            &next_css,
                            dialect,
                            mutation_spans.as_slice(),
                        );
                    }
                    coordinate_map.apply_mutation_spans(mutation_spans.as_slice());
                    outcome_mutation_spans.push(mutation_spans);
                    document.replace_with_css(next_css);
                }
                None => {
                    outcome_mutation_spans.push(Vec::new());
                }
            }
        }
        outcomes.push(outcome);
    }

    let executed_pass_ids = outcomes
        .iter()
        .filter(|outcome| outcome.status != TransformPassRuntimeStatus::PlannedOnly)
        .map(|outcome| outcome.pass_id)
        .collect::<Vec<_>>();
    let planned_only_pass_ids = outcomes
        .iter()
        .filter(|outcome| outcome.status == TransformPassRuntimeStatus::PlannedOnly)
        .map(|outcome| outcome.pass_id)
        .collect::<Vec<_>>();
    let mutation_count = outcomes
        .iter()
        .map(|outcome| outcome.mutation_count)
        .sum::<usize>();
    let provenance_preserved = outcomes.iter().all(|outcome| outcome.provenance_preserved);
    let provenance_derivation_forest =
        provenance_derivation_forest_from_outcomes(&outcomes, &outcome_mutation_spans);
    let cascade_proof_obligations = summarize_cascade_proof_obligations(cascade_proof_obligations);
    let structural_ir_transaction_telemetry = structural_ir_transaction_telemetry_snapshot();
    let output_byte_len = document.current_byte_len();
    let output_css = document.output_css();

    TransformExecutionSummaryV0 {
        schema_version: "0",
        product: "omena-transform-passes.execution",
        input_byte_len: source.len(),
        output_byte_len,
        requested_pass_ids,
        ordered_pass_ids,
        executed_pass_ids,
        planned_only_pass_ids,
        mutation_count,
        provenance_preserved,
        output_css,
        css_module_evaluation,
        css_import_inlines,
        css_module_composes_exports,
        design_token_routes,
        semantic_removals,
        cascade_proof_obligations,
        provenance_derivation_forest,
        structural_ir_transaction_telemetry,
        outcomes,
        pass_plan,
    }
}

fn transform_pass_may_consume_lex_cache(pass: TransformPassKind) -> bool {
    omena_transform_cst::transform_pass_class(pass) == TransformPassClassV0::TextLocal
}

fn next_document_lex_cache_consumer(
    ordered_pass_ids: &[&'static str],
    pass_index: usize,
) -> Option<TransformTextLocalExecutionModeV0> {
    ordered_pass_ids
        .iter()
        .skip(pass_index + 1)
        .filter_map(|pass_id| transform_pass_kind_from_id(pass_id))
        .filter(|kind| transform_pass_may_consume_lex_cache(*kind))
        .find_map(text_local_execution_mode_for_kind)
}

fn compose_ir_transaction_mutation_span_batches(
    input_byte_len: usize,
    output_byte_len: usize,
    span_batches: &[Vec<TransformProvenanceMutationSpanV0>],
) -> Option<Vec<TransformProvenanceMutationSpanV0>> {
    if span_batches.is_empty() {
        return None;
    }

    let mut coordinate_map = TransformSpanCoordinateMapV0::new(input_byte_len);
    let mut composed_spans = Vec::new();
    for batch in span_batches {
        if batch.is_empty() {
            return None;
        }
        remap_composed_generated_spans(composed_spans.as_mut_slice(), batch.as_slice())?;
        for span in batch {
            let (source_span_start, source_span_end) = coordinate_map
                .map_current_span_to_original(span.source_span_start, span.source_span_end)?;
            composed_spans.push(TransformProvenanceMutationSpanV0 {
                source_span_start,
                source_span_end,
                generated_span_start: span.generated_span_start,
                generated_span_end: span.generated_span_end,
                node_key: None,
            });
        }
        coordinate_map.apply_mutation_spans(batch.as_slice());
    }

    if composed_spans
        .iter()
        .all(|span| span.generated_span_end <= output_byte_len)
    {
        Some(composed_spans)
    } else {
        None
    }
}

fn remap_composed_generated_spans(
    composed_spans: &mut [TransformProvenanceMutationSpanV0],
    next_batch: &[TransformProvenanceMutationSpanV0],
) -> Option<()> {
    for span in composed_spans {
        span.generated_span_start =
            map_current_position_through_mutations(span.generated_span_start, next_batch)?;
        span.generated_span_end =
            map_current_position_through_mutations(span.generated_span_end, next_batch)?;
        if span.generated_span_start > span.generated_span_end {
            return None;
        }
    }
    Some(())
}

struct TransformModuleEvaluationMaterializedOutput {
    css: String,
    detail: &'static str,
}

fn materialize_transform_module_evaluation_output(
    input_css: &str,
    evaluation: &TransformModuleEvaluationV0,
    native_detail: &'static str,
    preserve_detail: &'static str,
) -> TransformModuleEvaluationMaterializedOutput {
    if !evaluation.may_consume_native_product_output() {
        return TransformModuleEvaluationMaterializedOutput {
            css: input_css.to_string(),
            detail: preserve_detail,
        };
    }

    if let Some(native_edit_output) = evaluation.native_edit_output.as_ref() {
        if evaluation.native_output_matches_retained_oracle(native_edit_output) {
            return TransformModuleEvaluationMaterializedOutput {
                css: native_edit_output.clone(),
                detail: native_detail,
            };
        }
        return TransformModuleEvaluationMaterializedOutput {
            css: input_css.to_string(),
            detail: preserve_detail,
        };
    }

    if let Some(native_css) =
        apply_transform_module_evaluation_native_edits(input_css, &evaluation.native_edits)
        && native_css == evaluation.evaluated_css
        && evaluation.native_output_matches_retained_oracle(native_css.as_str())
    {
        return TransformModuleEvaluationMaterializedOutput {
            css: native_css,
            detail: native_detail,
        };
    }

    TransformModuleEvaluationMaterializedOutput {
        css: input_css.to_string(),
        detail: preserve_detail,
    }
}

fn apply_transform_module_evaluation_native_edits(
    input_css: &str,
    native_edits: &[TransformModuleEvaluationNativeEditV0],
) -> Option<String> {
    if native_edits.is_empty() {
        return None;
    }

    let mut edits = native_edits.to_vec();
    edits.sort_by_key(|edit| edit.start);

    let mut previous_end = 0usize;
    for edit in &edits {
        if edit.start < previous_end
            || edit.start > edit.end
            || edit.end > input_css.len()
            || !input_css.is_char_boundary(edit.start)
            || !input_css.is_char_boundary(edit.end)
        {
            return None;
        }
        previous_end = edit.end;
    }

    let mut output = input_css.to_string();
    for edit in edits.iter().rev() {
        output.replace_range(edit.start..edit.end, edit.replacement.as_str());
    }
    Some(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TransformSpanMapSegmentV0 {
    current_start: usize,
    current_end: usize,
    original_start: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransformSpanCoordinateMapV0 {
    segments: Vec<TransformSpanMapSegmentV0>,
}

impl TransformSpanCoordinateMapV0 {
    fn new(source_len: usize) -> Self {
        Self {
            segments: vec![TransformSpanMapSegmentV0 {
                current_start: 0,
                current_end: source_len,
                original_start: 0,
            }],
        }
    }

    fn map_current_span_to_original(
        &self,
        current_start: usize,
        current_end: usize,
    ) -> Option<(usize, usize)> {
        let segment = self.segments.iter().find(|segment| {
            segment.current_start <= current_start && current_end <= segment.current_end
        })?;
        let original_start = segment.original_start + current_start - segment.current_start;
        let original_end = original_start + current_end.saturating_sub(current_start);
        Some((original_start, original_end))
    }

    fn apply_mutation_spans(&mut self, mutation_spans: &[TransformProvenanceMutationSpanV0]) {
        if mutation_spans.is_empty() {
            return;
        }

        let mut sorted_spans = mutation_spans.to_vec();
        sorted_spans.sort_by(|left, right| {
            left.source_span_start
                .cmp(&right.source_span_start)
                .then_with(|| left.source_span_end.cmp(&right.source_span_end))
        });

        let mut next_segments = Vec::new();
        for segment in &self.segments {
            let mut cursor = segment.current_start;
            for span in &sorted_spans {
                if span.source_span_end <= cursor {
                    continue;
                }
                if span.source_span_start >= segment.current_end {
                    break;
                }
                let unchanged_end = span.source_span_start.min(segment.current_end);
                self.push_mapped_piece(
                    segment,
                    cursor,
                    unchanged_end,
                    &sorted_spans,
                    &mut next_segments,
                );
                cursor = cursor.max(span.source_span_end.min(segment.current_end));
            }
            self.push_mapped_piece(
                segment,
                cursor,
                segment.current_end,
                &sorted_spans,
                &mut next_segments,
            );
        }
        self.segments = next_segments;
    }

    fn push_mapped_piece(
        &self,
        segment: &TransformSpanMapSegmentV0,
        current_start: usize,
        current_end: usize,
        mutation_spans: &[TransformProvenanceMutationSpanV0],
        next_segments: &mut Vec<TransformSpanMapSegmentV0>,
    ) {
        if current_start >= current_end {
            return;
        }
        let Some(next_start) =
            map_current_position_through_mutations(current_start, mutation_spans)
        else {
            return;
        };
        let Some(next_end) = map_current_position_through_mutations(current_end, mutation_spans)
        else {
            return;
        };
        if next_start >= next_end {
            return;
        }
        next_segments.push(TransformSpanMapSegmentV0 {
            current_start: next_start,
            current_end: next_end,
            original_start: segment.original_start + current_start - segment.current_start,
        });
    }
}

fn map_current_position_through_mutations(
    position: usize,
    mutation_spans: &[TransformProvenanceMutationSpanV0],
) -> Option<usize> {
    let mut delta = 0isize;
    for span in mutation_spans {
        if position < span.source_span_start {
            return apply_position_delta(position, delta);
        }
        if position <= span.source_span_end {
            return (position == span.source_span_start)
                .then(|| apply_position_delta(span.generated_span_start, 0))
                .flatten()
                .or_else(|| {
                    (position == span.source_span_end)
                        .then(|| apply_position_delta(span.generated_span_end, 0))
                        .flatten()
                });
        }
        delta = span.generated_span_end as isize - span.source_span_end as isize;
    }
    apply_position_delta(position, delta)
}

fn apply_position_delta(position: usize, delta: isize) -> Option<usize> {
    if delta >= 0 {
        position.checked_add(delta as usize)
    } else {
        position.checked_sub((-delta) as usize)
    }
}

/// Stamp each mutation span with the stable node key of the innermost original-source
/// node it maps back to. `node_key` is **best-effort, additive metadata** (it never affects
/// emitted CSS): a span in a later pass whose current coordinates straddle a prior-pass
/// mutation boundary matches no single surviving segment, so it maps to `None` and the key
/// is omitted rather than mis-attributed. The common case (a span fully inside one surviving
/// region) maps through the composed coordinate map to the correct original interval.
fn stamp_mutation_span_node_keys(
    mutation_spans: &mut [TransformProvenanceMutationSpanV0],
    coordinate_map: &TransformSpanCoordinateMapV0,
    stable_ir_nodes: &[StableTransformIrNodeV0],
) {
    for span in mutation_spans {
        let Some((original_start, original_end)) = coordinate_map
            .map_current_span_to_original(span.source_span_start, span.source_span_end)
        else {
            continue;
        };
        span.node_key =
            innermost_stable_node_key_for_span(original_start, original_end, stable_ir_nodes);
    }
}

fn innermost_stable_node_key_for_span(
    original_start: usize,
    original_end: usize,
    stable_ir_nodes: &[StableTransformIrNodeV0],
) -> Option<omena_transform_cst::StableNodeKeyV0> {
    stable_ir_nodes
        .iter()
        .filter(|node| {
            let overlap_start = node.source_span_start.max(original_start);
            let overlap_end = node.source_span_end.min(original_end);
            overlap_start < overlap_end
        })
        .min_by_key(|node| {
            let contains =
                node.source_span_start <= original_start && original_end <= node.source_span_end;
            (
                usize::from(!contains),
                node.source_span_end.saturating_sub(node.source_span_start),
            )
        })
        .and_then(|node| node.node_key.clone())
}

#[cfg(test)]
mod dispatch_table_tests {
    use super::*;
    use omena_transform_cst::{TransformPassClassV0, default_transform_pass_descriptors};

    fn mutation_span(
        source_span_start: usize,
        source_span_end: usize,
        generated_span_start: usize,
        generated_span_end: usize,
    ) -> TransformProvenanceMutationSpanV0 {
        TransformProvenanceMutationSpanV0 {
            source_span_start,
            source_span_end,
            generated_span_start,
            generated_span_end,
            node_key: None,
        }
    }

    #[test]
    fn text_local_dispatch_handlers_match_pass_descriptors() {
        let mut descriptor_pass_ids = default_transform_pass_descriptors()
            .into_iter()
            .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::TextLocal)
            .map(|descriptor| descriptor.id)
            .collect::<Vec<_>>();
        let mut handler_pass_ids = text_local_pass_handlers()
            .iter()
            .map(|handler| handler.kind.id())
            .collect::<Vec<_>>();

        descriptor_pass_ids.sort_unstable();
        handler_pass_ids.sort_unstable();

        assert_eq!(handler_pass_ids.len(), 20);
        assert_eq!(handler_pass_ids, descriptor_pass_ids);
    }

    #[test]
    fn runtime_dispatch_entries_cover_public_registry_entries() {
        let registry = default_transform_pass_registry();

        assert!(
            registry
                .entries
                .iter()
                .all(|entry| { runtime_pass_implementation_for_entry(entry).is_some() })
        );
        assert!(registry.entries.iter().all(|entry| {
            matches!(
                (
                    entry.dispatch_kind,
                    runtime_pass_implementation_for_entry(entry)
                ),
                (
                    TransformPassDispatchKindV0::TextLocalSliceRewrite,
                    Some(TransformRuntimePassImplementationV0::TextLocal(_))
                ) | (
                    TransformPassDispatchKindV0::StructuralIrTransaction,
                    Some(TransformRuntimePassImplementationV0::Structural(_))
                ) | (
                    TransformPassDispatchKindV0::ModuleEvaluationHandler,
                    Some(TransformRuntimePassImplementationV0::ModuleEvaluation(_))
                ) | (
                    TransformPassDispatchKindV0::EmissionBoundary,
                    Some(TransformRuntimePassImplementationV0::Emission(_))
                )
            )
        }));
    }

    #[test]
    fn text_local_dispatch_handlers_declare_ir_window_scopes() {
        let handlers = text_local_pass_handlers();

        assert_eq!(handlers.len(), 20);
        assert_eq!(
            handlers
                .iter()
                .filter(|handler| handler.window_scope
                    == TransformTextLocalWindowScopeV0::DocumentTokenStream)
                .count(),
            2
        );
        assert_eq!(
            handlers
                .iter()
                .filter(|handler| handler.window_scope == TransformTextLocalWindowScopeV0::Selector)
                .count(),
            1
        );
        assert_eq!(
            handlers
                .iter()
                .filter(|handler| handler.window_scope
                    == TransformTextLocalWindowScopeV0::DeclarationBlock)
                .count(),
            4
        );
        assert_eq!(
            handlers
                .iter()
                .filter(|handler| handler.window_scope
                    == TransformTextLocalWindowScopeV0::DeclarationValue)
                .count(),
            13
        );

        assert!(handlers.iter().any(|handler| {
            handler.kind == TransformPassKind::WhitespaceStrip
                && handler.window_scope == TransformTextLocalWindowScopeV0::DocumentTokenStream
        }));
        assert!(handlers.iter().any(|handler| {
            handler.kind == TransformPassKind::SelectorIsWhereCompression
                && handler.window_scope == TransformTextLocalWindowScopeV0::Selector
        }));
        assert!(handlers.iter().any(|handler| {
            handler.kind == TransformPassKind::ShorthandCombining
                && handler.window_scope == TransformTextLocalWindowScopeV0::DeclarationBlock
        }));
        assert!(handlers.iter().any(|handler| {
            handler.kind == TransformPassKind::CalcReduction
                && handler.window_scope == TransformTextLocalWindowScopeV0::DeclarationValue
        }));
    }

    #[test]
    fn text_local_dispatch_handlers_declare_execution_modes() {
        let handlers = text_local_pass_handlers();

        assert_eq!(handlers.len(), 20);
        assert_eq!(
            handlers
                .iter()
                .filter(|handler| handler.execution_mode
                    == TransformTextLocalExecutionModeV0::FullDocument)
                .count(),
            6
        );
        assert_eq!(
            handlers
                .iter()
                .filter(|handler| handler.execution_mode
                    == TransformTextLocalExecutionModeV0::WindowBatch)
                .count(),
            14
        );

        assert!(handlers.iter().any(|handler| {
            handler.kind == TransformPassKind::ValueResolution
                && handler.execution_mode == TransformTextLocalExecutionModeV0::FullDocument
        }));
        assert!(handlers.iter().any(|handler| {
            handler.kind == TransformPassKind::CalcReduction
                && handler.execution_mode == TransformTextLocalExecutionModeV0::WindowBatch
        }));
    }

    #[test]
    fn structural_dispatch_handlers_match_remaining_structural_descriptors() {
        let mut descriptor_pass_ids = default_transform_pass_descriptors()
            .into_iter()
            .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::Structural)
            .map(|descriptor| descriptor.id)
            .collect::<Vec<_>>();
        let mut handler_pass_ids = structural_pass_handlers()
            .iter()
            .map(|handler| handler.kind.id())
            .collect::<Vec<_>>();

        descriptor_pass_ids.sort_unstable();
        handler_pass_ids.sort_unstable();

        assert_eq!(descriptor_pass_ids.len(), 21);
        assert_eq!(handler_pass_ids.len(), 21);
        assert_eq!(handler_pass_ids, descriptor_pass_ids);
    }

    #[test]
    fn structural_dispatch_input_carries_ir_not_raw_css() -> Result<(), String> {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("runtime")
                .join("executor.rs"),
        )
        .map_err(|err| format!("executor source should be readable: {err:?}"))?;
        let input_anchor = source
            .find("struct TransformStructuralPassInputV0")
            .ok_or_else(|| "structural input should exist".to_string())?;
        let handler_anchor = source[input_anchor..]
            .find("fn structural_pass_handlers")
            .ok_or_else(|| "structural handler boundary should exist".to_string())?;
        let input_body = &source[input_anchor..input_anchor + handler_anchor];

        assert!(input_body.contains("current_ir: &'a mut TransformIrV0"));
        assert!(!input_body.contains("input_css:"));
        assert!(!input_body.contains("fn source_text(&self) -> &str"));
        assert!(input_body.contains("fn current_ir_mut(&mut self) -> &mut TransformIrV0"));
        Ok(())
    }

    #[test]
    fn text_local_dispatch_uses_ir_window_input() -> Result<(), String> {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("runtime")
                .join("executor.rs"),
        )
        .map_err(|err| format!("executor source should be readable: {err:?}"))?;
        let runner_anchor = source
            .find("type TransformTextLocalRunnerV0")
            .ok_or_else(|| "text-local runner type should exist".to_string())?;
        let structural_runner_anchor = source[runner_anchor..]
            .find("type TransformStructuralRunnerV0")
            .ok_or_else(|| "structural runner type should delimit text-local input".to_string())?;
        let text_local_input_body =
            &source[runner_anchor..runner_anchor + structural_runner_anchor];

        assert!(text_local_input_body.contains("TransformTextLocalPassInputV0<'a>"));
        assert!(text_local_input_body.contains("TransformTextLocalPassOutputV0<'a>"));
        assert!(text_local_input_body.contains("TransformTextLocalIrWindowV0"));
        assert!(!text_local_input_body.contains("fn(&str, StyleDialect"));
        assert!(text_local_input_body.contains("fn rewrite_windows("));
        assert!(text_local_input_body.contains("fn rewrite_full_document("));

        let dispatch_anchor = source
            .find("fn dispatch_text_local_pass")
            .ok_or_else(|| "text-local dispatch should exist".to_string())?;
        let whitespace_anchor = source[dispatch_anchor..]
            .find("fn run_whitespace_strip_text_local")
            .ok_or_else(|| "first text-local runner should delimit dispatch".to_string())?;
        let dispatch_body = &source[dispatch_anchor..dispatch_anchor + whitespace_anchor];

        assert!(dispatch_body.contains("current_ir: &TransformIrV0"));
        assert!(dispatch_body.contains("TransformTextLocalPassInputV0::from_ir("));
        assert!(dispatch_body.contains("handler.window_scope"));
        assert!(dispatch_body.contains("handler.execution_mode"));
        assert!(!dispatch_body.contains("input_css: &str"));

        let loop_anchor = source
            .find("Some(TransformRuntimePassImplementationV0::TextLocal(handler))")
            .ok_or_else(|| "text-local executor dispatch branch should exist".to_string())?;
        let module_anchor = source[loop_anchor..]
            .find("Some(TransformRuntimePassImplementationV0::ModuleEvaluation(pass))")
            .ok_or_else(|| "module branch should delimit text-local branch".to_string())?;
        let loop_body = &source[loop_anchor..loop_anchor + module_anchor];

        assert!(loop_body.contains("dispatch_text_local_pass(pass_id, handler"));
        assert!(!loop_body.contains("pass_input_css, dialect, context"));
        Ok(())
    }

    #[test]
    fn text_local_declaration_scopes_rewrite_multiple_ir_windows() {
        let source = ".a { color: RED; } .b { color: BLUE; }";
        let ir = lower_transform_ir_from_source(source, StyleDialect::Css, "window-batch-test");
        let context = TransformExecutionContextV0::default();
        let input = TransformTextLocalPassInputV0::from_ir(
            &ir,
            TransformTextLocalWindowScopeV0::DeclarationValue,
            TransformTextLocalExecutionModeV0::WindowBatch,
            StyleDialect::Css,
            &context,
        );

        assert_eq!(input.source_windows.len(), 2);

        let output = input.rewrite_text_local(|window_source, _dialect, _context| {
            (window_source.to_ascii_lowercase(), 1)
        });

        assert_eq!(output.input_byte_len(), source.len());
        assert_eq!(output.mutation_count, 2);
        assert_eq!(output.provenance_mutation_spans().len(), 2);
        assert_eq!(
            &source[output.provenance_mutation_spans()[0].source_span_start
                ..output.provenance_mutation_spans()[0].source_span_end],
            "RED"
        );
        assert_eq!(
            &output.rewritten_css[output.provenance_mutation_spans()[0].generated_span_start
                ..output.provenance_mutation_spans()[0].generated_span_end],
            "red"
        );
        assert_eq!(
            &source[output.provenance_mutation_spans()[1].source_span_start
                ..output.provenance_mutation_spans()[1].source_span_end],
            "BLUE"
        );
        assert_eq!(
            &output.rewritten_css[output.provenance_mutation_spans()[1].generated_span_start
                ..output.provenance_mutation_spans()[1].generated_span_end],
            "blue"
        );
        assert_eq!(
            output.into_document_css(),
            ".a { color: red; } .b { color: blue; }"
        );
    }

    #[test]
    fn structural_dispatch_handlers_commit_through_ir_mutation_only() -> Result<(), String> {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("runtime")
                .join("executor.rs"),
        )
        .map_err(|err| format!("executor source should be readable: {err:?}"))?;
        let first_structural_handler = source
            .find("fn run_import_inline_structural")
            .ok_or_else(|| "first structural handler should exist".to_string())?;
        let executor_loop_anchor = source[first_structural_handler..]
            .find("fn execute_transform_passes_on_source_with_active_lex_cache")
            .ok_or_else(|| "executor loop should delimit structural handlers".to_string())?;
        let structural_handler_body =
            &source[first_structural_handler..first_structural_handler + executor_loop_anchor];
        let ir_mutation_count = structural_handler_body
            .matches("input.ir_mutation_result(")
            .count();

        assert_eq!(ir_mutation_count, structural_pass_handlers().len());
        assert!(
            !structural_handler_body.contains("TransformPassDispatchResultV0::textual_mutation(")
        );
        assert!(!structural_handler_body.contains("TransformPassDispatchResultV0::ir_mutation("));
        assert!(!structural_handler_body.contains("_rendered_css"));
        assert!(!structural_handler_body.contains("input.source_text("));
        Ok(())
    }

    #[test]
    fn structural_ir_mutations_do_not_relower_document_from_css() -> Result<(), String> {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("runtime")
                .join("executor.rs"),
        )
        .map_err(|err| format!("executor source should be readable: {err:?}"))?;
        let ir_mutation_anchor = source
            .find("fn ir_mutation(")
            .ok_or_else(|| "IR mutation dispatch result constructor should exist".to_string())?;
        let planned_only_anchor = source[ir_mutation_anchor..]
            .find("fn planned_only(")
            .ok_or_else(|| "planned-only constructor should delimit IR mutation".to_string())?;
        let ir_mutation_body =
            &source[ir_mutation_anchor..ir_mutation_anchor + planned_only_anchor];

        assert!(ir_mutation_body.contains("result.document_ir_updated = true;"));

        let update_anchor = source
            .find("if document_ir_updated")
            .ok_or_else(|| "executor should branch structural IR updates".to_string())?;
        let outcomes_anchor = source[update_anchor..]
            .find("outcomes.push(outcome);")
            .ok_or_else(|| "outcome push should delimit document update".to_string())?;
        let update_body = &source[update_anchor..update_anchor + outcomes_anchor];
        let text_branch_anchor = update_body
            .find("match next_textual_css")
            .ok_or_else(|| "text-local/module output branch should exist".to_string())?;
        let relower_anchor = update_body
            .find("document.replace_with_css(next_css);")
            .ok_or_else(|| {
                "text-local/module document re-lowering path should exist".to_string()
            })?;
        let ir_branch_body = &update_body[..text_branch_anchor];

        assert!(relower_anchor > text_branch_anchor);
        assert!(!ir_branch_body.contains("document.replace_with_css(next_css);"));
        Ok(())
    }

    #[test]
    fn executor_loop_materializes_textual_bridge_lazily() -> Result<(), String> {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("runtime")
                .join("executor.rs"),
        )
        .map_err(|err| format!("executor source should be readable: {err:?}"))?;
        let loop_anchor = source
            .find("for (pass_index, pass_id) in ordered_pass_ids.iter().enumerate()")
            .ok_or_else(|| "executor pass loop should exist".to_string())?;
        let outcomes_anchor = source[loop_anchor..]
            .find("outcomes.push(outcome);")
            .ok_or_else(|| "outcome push should delimit pass loop body".to_string())?;
        let loop_body = &source[loop_anchor..loop_anchor + outcomes_anchor];

        assert!(loop_body.contains("TransformTextualBridgeSnapshotV0::default()"));
        assert!(loop_body.contains("textual_bridge.materialize_current_css(&document);"));
        assert!(loop_body.contains("if should_maintain_document_lex_cache"));
        assert!(loop_body.contains("collect_cascade_proof_obligations_for_ir_pass_input("));
        assert!(loop_body.contains("dispatch_structural_pass("));
        assert!(loop_body.contains("Some(mutation_spans) => mutation_spans"));
        assert!(loop_body.contains("if document_ir_updated"));
        assert!(loop_body.contains("let output_byte_len = document.current_byte_len();"));
        assert!(loop_body.contains("let next_css = document.current_css().to_string();"));
        assert!(loop_body.contains("match next_textual_css"));
        assert!(loop_body.contains("derive_transform_mutation_spans(pass_input_css, &next_css)"));
        assert!(loop_body.contains("outcome_mutation_spans.push(Vec::new());"));
        Ok(())
    }

    #[test]
    fn structural_single_transaction_uses_ir_mutation_span_envelope() -> Result<(), String> {
        let source = ".button { color: red; }";
        let selector_end = source
            .find('{')
            .ok_or_else(|| "fixture should contain a block".to_string())?;
        let context = TransformExecutionContextV0 {
            class_name_rewrites: vec![crate::TransformClassNameRewriteV0 {
                original_name: "button".to_string(),
                rewritten_name: "_button_x".to_string(),
            }],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[TransformPassKind::HashCssModuleClassNames],
            &context,
        );
        let mutation_spans = &execution.provenance_derivation_forest.nodes[0].mutation_spans;

        assert_eq!(execution.output_css, "._button_x{ color: red; }");
        assert_eq!(mutation_spans.len(), 1);
        assert_eq!(mutation_spans[0].source_span_start, 0);
        assert_eq!(mutation_spans[0].source_span_end, selector_end);
        assert!(mutation_spans[0].source_span_end < source.len());
        Ok(())
    }

    #[test]
    fn structural_multi_transaction_batches_compose_mutation_span_coordinates() {
        let spans = compose_ir_transaction_mutation_span_batches(
            6,
            10,
            &[
                vec![mutation_span(1, 3, 1, 5)],
                vec![mutation_span(6, 7, 6, 9)],
            ],
        );

        assert_eq!(
            spans,
            Some(vec![mutation_span(1, 3, 1, 5), mutation_span(4, 5, 6, 9)])
        );
    }

    #[test]
    fn structural_multi_transaction_batches_fall_back_when_coordinates_do_not_project() {
        let spans = compose_ir_transaction_mutation_span_batches(
            6,
            8,
            &[
                vec![mutation_span(1, 3, 1, 5)],
                vec![mutation_span(4, 6, 4, 6)],
            ],
        );

        assert_eq!(spans, None);
    }

    #[test]
    fn structural_multi_transaction_pass_uses_composed_ir_mutation_spans() {
        let source = ".a { color: red; color: blue; }.dup { margin: 0; }.dup { margin: 0; }";
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[TransformPassKind::RuleDeduplication],
            &TransformExecutionContextV0::default(),
        );
        let mutation_spans = &execution.provenance_derivation_forest.nodes[0].mutation_spans;

        assert_eq!(
            execution
                .structural_ir_transaction_telemetry
                .transaction_commit_count,
            2
        );
        assert_eq!(execution.mutation_count, 2);
        assert_eq!(mutation_spans.len(), 2);
        assert!(mutation_spans.iter().all(|span| {
            span.source_span_start <= span.source_span_end
                && span.generated_span_start <= span.generated_span_end
                && span.generated_span_end <= execution.output_css.len()
        }));
    }

    #[test]
    fn lex_cache_consumer_classification_stays_text_local() {
        for descriptor in default_transform_pass_descriptors() {
            assert_eq!(
                transform_pass_may_consume_lex_cache(descriptor.kind),
                descriptor.pass_class == TransformPassClassV0::TextLocal,
                "unexpected lex-cache consumer classification for {}",
                descriptor.id
            );
        }
    }

    #[test]
    fn structural_only_execution_never_records_lex_cache_full_relex_fallback() {
        let structural_passes = default_transform_pass_descriptors()
            .into_iter()
            .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::Structural)
            .map(|descriptor| descriptor.kind)
            .collect::<Vec<_>>();
        let source =
            ".dup { color: red; }.dup { color: red; }.empty { } @media screen { .media { } }";

        super::super::lex_cache::reset_transform_lex_cache_splice_telemetry();
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            structural_passes.as_slice(),
            &TransformExecutionContextV0::default(),
        );
        let telemetry = super::super::lex_cache::transform_lex_cache_splice_telemetry_snapshot();

        assert_eq!(structural_passes.len(), 21);
        assert!(execution.mutation_count > 0);
        assert!(
            execution
                .structural_ir_transaction_telemetry
                .transaction_commit_count
                > 0
        );
        assert_eq!(telemetry.full_relex_fallback_count, 0);
        assert_eq!(telemetry.window_derivation_fallback_count, 0);
        assert_eq!(telemetry.full_output_window_fallback_count, 0);
        assert_eq!(telemetry.token_offset_fallback_count, 0);
    }

    #[test]
    fn structural_execution_hashes_less_class_names_through_ir_transactions() {
        let context = TransformExecutionContextV0 {
            class_name_rewrites: vec![crate::TransformClassNameRewriteV0 {
                original_name: "button".to_string(),
                rewritten_name: "_button_x".to_string(),
            }],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            ".button { color: red; }",
            StyleDialect::Less,
            &[
                TransformPassKind::HashCssModuleClassNames,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.output_css, "._button_x{ color: red; }");
        assert!(
            execution
                .structural_ir_transaction_telemetry
                .transaction_commit_count
                > 0
        );
    }

    #[test]
    fn executor_loop_dispatches_without_pass_kind_match() -> Result<(), String> {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("runtime")
                .join("executor.rs"),
        )
        .map_err(|err| format!("executor source should be readable: {err:?}"))?;
        let loop_anchor = source
            .find("let dispatch_result =")
            .ok_or_else(|| "executor should keep a dispatch result boundary".to_string())?;
        let loop_match_tail = &source[loop_anchor..];
        let destructure_anchor = loop_match_tail
            .find("let TransformPassDispatchResultV0")
            .ok_or_else(|| "executor should destructure the dispatch result".to_string())?;
        let loop_dispatch_body = &loop_match_tail[..destructure_anchor];

        assert!(loop_dispatch_body.contains("dispatch_text_local_pass"));
        assert!(loop_dispatch_body.contains("dispatch_module_evaluation_pass"));
        assert!(loop_dispatch_body.contains("dispatch_structural_pass"));
        assert!(loop_dispatch_body.contains("dispatch_emission_pass"));
        assert!(loop_dispatch_body.contains("TransformRuntimePassImplementationV0::Structural"));
        assert!(
            source.contains("runtime_pass_entry_for_kind(kind, pass_registry.entries.as_slice())")
        );
        assert!(!loop_dispatch_body.contains("ModuleEvaluationOrEgressHandler"));
        assert!(!loop_dispatch_body.contains("StructuralHandler"));
        assert!(!loop_dispatch_body.contains("match pass"));
        assert!(!loop_dispatch_body.contains("Some(TransformPassKind::"));
        Ok(())
    }
}

#[cfg(test)]
mod module_evaluation_materialization_tests {
    use super::*;
    use crate::model::TransformModuleEvaluationOracleV0;

    fn oracle_allowing_native_output() -> TransformModuleEvaluationOracleV0 {
        TransformModuleEvaluationOracleV0 {
            mode: "oracleOnly".to_string(),
            product_output_source: "legacyEvaluatedCss".to_string(),
            divergence_count: 0,
            all_legacy_declaration_values_preserved: true,
            ..TransformModuleEvaluationOracleV0::default()
        }
    }

    fn module_evaluation(
        evaluated_css: &str,
        native_edit_output: Option<&str>,
        oracle: Option<TransformModuleEvaluationOracleV0>,
    ) -> TransformModuleEvaluationV0 {
        TransformModuleEvaluationV0 {
            evaluator: "test".to_string(),
            product_output_source: Some("nativeEditOutput".to_string()),
            evaluated_css: evaluated_css.to_string(),
            native_edit_output: native_edit_output.map(str::to_string),
            native_replacements: Vec::new(),
            native_edits: Vec::new(),
            oracle,
        }
    }

    #[test]
    fn module_evaluation_consumes_oracle_backed_matching_native_output() {
        let input_css = ".input { color: red; }";
        let evaluation = module_evaluation(
            ".native { color: red; }",
            Some(".native { color: red; }"),
            Some(oracle_allowing_native_output()),
        );

        let output = materialize_transform_module_evaluation_output(
            input_css,
            &evaluation,
            "native",
            "preserve",
        );

        assert_eq!(output.css, ".native { color: red; }");
        assert_eq!(output.detail, "native");
    }

    #[test]
    fn module_evaluation_preserves_input_when_oracle_backed_native_output_mismatches() {
        let input_css = ".input { color: red; }";
        let evaluation = module_evaluation(
            ".legacy { color: red; }",
            Some(".native { color: red; }"),
            Some(oracle_allowing_native_output()),
        );

        let output = materialize_transform_module_evaluation_output(
            input_css,
            &evaluation,
            "native",
            "preserve",
        );

        assert_eq!(output.css, input_css);
        assert_eq!(output.detail, "preserve");
    }
}

#[cfg(test)]
mod coordinate_map_tests {
    use super::*;

    fn mutation_span(
        source_span_start: usize,
        source_span_end: usize,
        generated_span_start: usize,
        generated_span_end: usize,
    ) -> TransformProvenanceMutationSpanV0 {
        TransformProvenanceMutationSpanV0 {
            source_span_start,
            source_span_end,
            generated_span_start,
            generated_span_end,
            node_key: None,
        }
    }

    // After a length-changing pass, a span in the drifted (post-pass) coordinate space must
    // map back to the correct ORIGINAL-source interval. This is the multi-pass remap the
    // node_key coordinate caveat exists to solve; it is RED if `apply_mutation_spans` were
    // identity-only (it would return the un-remapped (5,7) instead of the original (3,5)).
    #[test]
    fn coordinate_map_remaps_post_mutation_span_to_original_after_one_pass() {
        // original "abcdef" (len 6); pass replaces current [1,3) ("bc") with 4 bytes -> generated [1,5).
        let mut map = TransformSpanCoordinateMapV0::new(6);
        map.apply_mutation_spans(&[mutation_span(1, 3, 1, 5)]);
        // post-pass output "a????def" (len 8); current [5,7) ("de") -> original [3,5) ("de").
        assert_eq!(map.map_current_span_to_original(5, 7), Some((3, 5)));
        // the surviving prefix still maps to itself.
        assert_eq!(map.map_current_span_to_original(0, 1), Some((0, 1)));
    }

    // The remap composes across two stacked mutating passes (the case with zero prior coverage).
    #[test]
    fn coordinate_map_composes_across_two_mutating_passes() {
        let mut map = TransformSpanCoordinateMapV0::new(6); // "abcdef"
        // pass 1: current [1,3) -> 4 bytes (generated [1,5)); output len 8.
        map.apply_mutation_spans(&[mutation_span(1, 3, 1, 5)]);
        // pass 2 (coords in pass-1 output space, len 8): current [6,7) -> 3 bytes (generated [6,9)); output len 10.
        map.apply_mutation_spans(&[mutation_span(6, 7, 6, 9)]);
        // pass-2 output current [9,10) is the trailing original "f" at original [5,6).
        assert_eq!(map.map_current_span_to_original(9, 10), Some((5, 6)));
        assert_eq!(map.map_current_span_to_original(0, 1), Some((0, 1)));
    }

    // node_key is best-effort: a post-mutation span straddling a prior mutation boundary
    // matches no single surviving segment and maps to None (omitted, never mis-attributed).
    #[test]
    fn coordinate_map_returns_none_for_post_mutation_straddling_span() {
        let mut map = TransformSpanCoordinateMapV0::new(6);
        map.apply_mutation_spans(&[mutation_span(1, 3, 1, 5)]);
        // current [0,7) straddles the surviving prefix [0,1) and the shifted suffix [5,8).
        assert_eq!(map.map_current_span_to_original(0, 7), None);
    }
}

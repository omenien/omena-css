//! Optional e-graph rewrite boundary for Omena CSS transforms.
//!
//! Selector, shorthand, and computed-value rewrites are the current e-graph candidates.
//! This crate keeps their proof requirements explicit without forcing an
//! e-graph dependency into the core transform path.

use std::fmt::Write as _;

use egg::{
    Analysis, Applier, EGraph, Extractor, Id, Pattern, PatternAst, RecExpr, Rewrite, Runner, Subst,
    Symbol, Var, define_language, rewrite as egg_rewrite,
};
use omena_evidence_graph::ObligationFamilyIdV0;
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{
    TransformPassPlanV0, collect_stale_vendor_prefix_removal_proof_candidates_from_source,
    plan_transform_passes,
};
use serde::Serialize;

mod mdl_cost;
pub use mdl_cost::*;
#[cfg(feature = "lawvere-saturation")]
mod lawvere_analysis;
#[cfg(feature = "lawvere-saturation")]
pub use lawvere_analysis::*;

define_language! {
    enum CssRewriteLanguage {
        Num(i64),
        Symbol(Symbol),
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        "calc" = Calc(Id),
        "unit" = Unit([Id; 2]),
        "is" = Is(Id),
        "where" = Where(Id),
        "list" = List([Id; 2]),
        "decl" = Declaration([Id; 3]),
        "stale-prefix-decl" = StalePrefixDeclaration([Id; 4]),
        "box1" = Box1(Id),
        "box2" = Box2([Id; 2]),
        "box3" = Box3([Id; 3]),
        "box4" = Box4([Id; 4]),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteProofV0 {
    pub specificity_preserved: bool,
    #[serde(skip_serializing)]
    obligation_family: ObligationFamilyIdV0,
    pub computed_value_preserved: bool,
    pub provenance_preserved: bool,
    pub cascade_safe_witness: String,
}

impl EggRewriteProofV0 {
    pub fn new(
        specificity_preserved: bool,
        obligation_family: ObligationFamilyIdV0,
        provenance_preserved: bool,
        cascade_safe_witness: impl Into<String>,
    ) -> Self {
        Self {
            specificity_preserved,
            obligation_family,
            computed_value_preserved: obligation_family.preserves_computed_value(),
            provenance_preserved,
            cascade_safe_witness: cascade_safe_witness.into(),
        }
    }

    pub const fn obligation_family(&self) -> ObligationFamilyIdV0 {
        self.obligation_family
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteCandidateV0 {
    pub pass_id: &'static str,
    pub before: String,
    pub after: String,
    pub proof: EggRewriteProofV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteDecisionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub pass_id: &'static str,
    pub accepted: bool,
    pub blocked_reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteExecutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub pass_id: &'static str,
    pub accepted: bool,
    pub blocked_reason: Option<&'static str>,
    pub before: String,
    pub after: String,
    pub expected_after: String,
    pub after_matches_candidate: bool,
    pub engine: &'static str,
    pub iteration_limit: usize,
    pub iteration_count: usize,
    pub eclass_count: usize,
    pub enode_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdl_bits: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdl_residual_bits: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdl_unit: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EggRewriteSourceWitnessV0 {
    pub pass_id: &'static str,
    pub source_kind: &'static str,
    pub byte_offset: usize,
    pub css_before: String,
    pub css_after: String,
    pub execution: EggRewriteExecutionV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformEggBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub managed_pass_ids: Vec<&'static str>,
    pub optional_engine: &'static str,
    pub proof_obligations: Vec<&'static str>,
    pub planner_surface: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformEggPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub requested_pass_ids: Vec<&'static str>,
    pub planned_pass_ids: Vec<&'static str>,
    pub pass_plan: TransformPassPlanV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextualEqSatScaffoldV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub claim_level: &'static str,
    pub scaffold_kind: &'static str,
    pub execution_view: &'static str,
    pub current_engine: &'static str,
    pub egg_engine_ready: bool,
    pub egglog_binding_ready: bool,
    pub external_datalog_host_ready: bool,
    pub three_view_fusion_ready: bool,
    pub theorem_claimed: bool,
    pub public_safety_claim_ready: bool,
    pub modal_witness_product: &'static str,
    pub modal_bridge_claim_level: &'static str,
    pub paper_substrate_claim_level: &'static str,
    pub managed_pass_ids: Vec<&'static str>,
    pub substrate_products: Vec<&'static str>,
    pub supported_claims: Vec<&'static str>,
    pub deferred_claims: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy)]
enum CalcFoldOperator {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
struct ConstFoldSameUnitApplier {
    left_var: Var,
    right_var: Var,
    unit_var: Option<Var>,
    operator: CalcFoldOperator,
}

impl ConstFoldSameUnitApplier {
    fn new(operator: CalcFoldOperator, unit_var: Option<Var>) -> Option<Self> {
        Some(Self {
            left_var: "?a".parse().ok()?,
            right_var: "?b".parse().ok()?,
            unit_var,
            operator,
        })
    }
}

impl<N> Applier<CssRewriteLanguage, N> for ConstFoldSameUnitApplier
where
    N: Analysis<CssRewriteLanguage>,
{
    fn apply_one(
        &self,
        egraph: &mut EGraph<CssRewriteLanguage, N>,
        eclass: Id,
        subst: &Subst,
        _searcher_ast: Option<&PatternAst<CssRewriteLanguage>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        let Some(left) = numeric_value_from_eclass(egraph, subst[self.left_var]) else {
            return Vec::new();
        };
        let Some(right) = numeric_value_from_eclass(egraph, subst[self.right_var]) else {
            return Vec::new();
        };
        let value = match self.operator {
            CalcFoldOperator::Add => left + right,
            CalcFoldOperator::Sub => left - right,
        };
        let value_id = egraph.add(CssRewriteLanguage::Num(value));
        let result_id = if let Some(unit_var) = self.unit_var {
            egraph.add(CssRewriteLanguage::Unit([value_id, subst[unit_var]]))
        } else {
            value_id
        };
        egraph.union(eclass, result_id);
        vec![eclass]
    }

    fn vars(&self) -> Vec<Var> {
        let mut vars = vec![self.left_var, self.right_var];
        if let Some(unit_var) = self.unit_var {
            vars.push(unit_var);
        }
        vars
    }
}

pub fn summarize_omena_transform_egg_boundary() -> TransformEggBoundarySummaryV0 {
    TransformEggBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-egg.boundary",
        managed_pass_ids: managed_egg_passes().iter().map(|pass| pass.id()).collect(),
        optional_engine: "egg-compatible equality saturation engine",
        proof_obligations: vec![
            "selector rewrites preserve specificity",
            "calc rewrites preserve computed value",
            "shorthand rewrites preserve computed value",
            "stale-prefix removals preserve an exact unprefixed declaration peer",
            "all rewrites preserve provenance",
            "all accepted rewrites carry a cascade-safe witness",
        ],
        planner_surface: "omena-transform-passes.plan",
    }
}

pub fn plan_egg_rewrite_passes(include_selector: bool, include_calc: bool) -> TransformEggPlanV0 {
    let mut requested_passes = Vec::new();
    if include_selector {
        requested_passes.push(TransformPassKind::SelectorIsWhereCompression);
    }
    if include_calc {
        requested_passes.push(TransformPassKind::CalcReduction);
    }
    let pass_plan = plan_transform_passes(&requested_passes);

    TransformEggPlanV0 {
        schema_version: "0",
        product: "omena-transform-egg.plan",
        requested_pass_ids: requested_passes.iter().map(|pass| pass.id()).collect(),
        planned_pass_ids: pass_plan.ordered_pass_ids.clone(),
        pass_plan,
    }
}

pub fn plan_egg_rewrite_passes_for_source(source: &str) -> TransformEggPlanV0 {
    plan_egg_rewrite_passes(
        source.contains(":is(") || source.contains(":where("),
        source.contains("calc("),
    )
}

pub fn summarize_contextual_eqsat_scaffold_v0() -> ContextualEqSatScaffoldV0 {
    let boundary = summarize_omena_transform_egg_boundary();

    ContextualEqSatScaffoldV0 {
        schema_version: "0",
        product: "omena-transform-egg.contextual-eqsat-scaffold",
        claim_level: "m6ScaffoldOnlyNoEgglogBinding",
        scaffold_kind: "contextualEqualitySaturationExecutionView",
        execution_view: "m6BridgeNodeExecutionView",
        current_engine: "egg",
        egg_engine_ready: true,
        egglog_binding_ready: false,
        external_datalog_host_ready: false,
        three_view_fusion_ready: false,
        theorem_claimed: false,
        public_safety_claim_ready: false,
        modal_witness_product: "omena-cascade.modal-check-witness",
        modal_bridge_claim_level: "dependencyDeclaredOnly",
        paper_substrate_claim_level: "draftScaffoldOnly",
        managed_pass_ids: boundary.managed_pass_ids,
        substrate_products: vec![
            "omena-transform-egg.boundary",
            "omena-transform-egg.plan",
            "omena-transform-egg.execution",
            "omena-cascade.modal-check-witness",
        ],
        supported_claims: vec![
            "optional egg equality-saturation rewrite boundary",
            "selector, calc, and shorthand rewrite proof obligations",
            "contextual equality-saturation scaffold for M6 positioning",
            "modal witness dependency declaration for #66/#73 paper substrate",
        ],
        deferred_claims: vec![
            "egglog Rust binding",
            "external Datalog host execution",
            "full three-view fusion",
            "Contextual EqSat theorem",
            "production research-tier execution view",
        ],
    }
}

pub fn decide_egg_rewrite(candidate: EggRewriteCandidateV0) -> EggRewriteDecisionV0 {
    let blocked_reason = if !is_managed_egg_pass_id(candidate.pass_id) {
        Some("pass is not managed by omena-transform-egg")
    } else if candidate.proof.cascade_safe_witness.is_empty() {
        Some("missing cascade-safe witness")
    } else if !candidate.proof.provenance_preserved {
        Some("rewrite does not preserve provenance")
    } else if candidate.pass_id == TransformPassKind::SelectorIsWhereCompression.id()
        && !candidate.proof.specificity_preserved
    {
        Some("selector rewrite does not preserve specificity")
    } else if candidate.pass_id == TransformPassKind::CalcReduction.id()
        && !candidate.proof.computed_value_preserved
    {
        Some("calc rewrite does not preserve computed value")
    } else if candidate.pass_id == TransformPassKind::ShorthandCombining.id()
        && !candidate.proof.computed_value_preserved
    {
        Some("shorthand rewrite does not preserve computed value")
    } else if candidate.pass_id == TransformPassKind::StalePrefixRemoval.id()
        && !candidate.proof.computed_value_preserved
    {
        Some("stale-prefix removal does not preserve computed value")
    } else {
        None
    };

    EggRewriteDecisionV0 {
        schema_version: "0",
        product: "omena-transform-egg.decision",
        pass_id: candidate.pass_id,
        accepted: blocked_reason.is_none(),
        blocked_reason,
    }
}

pub fn execute_egg_rewrite(candidate: EggRewriteCandidateV0) -> EggRewriteExecutionV0 {
    let decision = decide_egg_rewrite(candidate.clone());
    if !decision.accepted {
        return blocked_execution(candidate, decision.blocked_reason);
    }

    let expression = match candidate.before.parse::<RecExpr<CssRewriteLanguage>>() {
        Ok(expression) => expression,
        Err(_) => return blocked_execution(candidate, Some("rewrite expression could not parse")),
    };
    let Some(rules) = rewrite_rules_for_pass::<()>(candidate.pass_id) else {
        return blocked_execution(
            candidate,
            Some("pass is not managed by omena-transform-egg"),
        );
    };

    let iteration_limit = 8;
    let runner = Runner::default()
        .with_expr(&expression)
        .with_iter_limit(iteration_limit)
        .run(rules.as_slice());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, MdlExtractionCostV0::default_ast_size());
    let (_, extracted) = extractor.find_best(root);
    let after = extracted.to_string();
    let after_matches_candidate = after == candidate.after;

    EggRewriteExecutionV0 {
        schema_version: "0",
        product: "omena-transform-egg.execution",
        pass_id: candidate.pass_id,
        accepted: after_matches_candidate,
        blocked_reason: (!after_matches_candidate)
            .then_some("egg extraction did not match candidate output"),
        before: candidate.before,
        after,
        expected_after: candidate.after,
        after_matches_candidate,
        engine: "egg",
        iteration_limit,
        iteration_count: runner.iterations.len(),
        eclass_count: runner.egraph.number_of_classes(),
        enode_count: runner.egraph.total_size(),
        mdl_bits: None,
        mdl_residual_bits: None,
        mdl_unit: None,
    }
}

pub fn execute_egg_rewrite_witnesses_for_css_source(
    source: &str,
    dialect: StyleDialect,
    transformed_source: &str,
    planned_pass_ids: &[&'static str],
) -> Vec<EggRewriteSourceWitnessV0> {
    let mut witnesses = Vec::new();
    if planned_pass_ids.contains(&TransformPassKind::SelectorIsWhereCompression.id()) {
        witnesses.extend(selector_rewrite_witnesses(source, transformed_source));
    }
    if planned_pass_ids.contains(&TransformPassKind::CalcReduction.id()) {
        witnesses.extend(calc_rewrite_witnesses(source, transformed_source));
    }
    if planned_pass_ids.contains(&TransformPassKind::StalePrefixRemoval.id()) {
        witnesses.extend(stale_prefix_removal_witnesses(
            source,
            dialect,
            transformed_source,
        ));
    }
    witnesses
}

fn managed_egg_passes() -> [TransformPassKind; 4] {
    [
        TransformPassKind::SelectorIsWhereCompression,
        TransformPassKind::CalcReduction,
        TransformPassKind::ShorthandCombining,
        TransformPassKind::StalePrefixRemoval,
    ]
}

fn is_managed_egg_pass_id(pass_id: &str) -> bool {
    managed_egg_passes().iter().any(|pass| pass.id() == pass_id)
}

fn numeric_value_from_eclass<N>(egraph: &EGraph<CssRewriteLanguage, N>, id: Id) -> Option<i64>
where
    N: Analysis<CssRewriteLanguage>,
{
    egraph[id].nodes.iter().find_map(|node| match node {
        CssRewriteLanguage::Num(value) => Some(*value),
        _ => None,
    })
}

fn selector_rewrite_witnesses(
    source: &str,
    transformed_source: &str,
) -> Vec<EggRewriteSourceWitnessV0> {
    let mut witnesses = Vec::new();
    for (prefix, source_kind) in [(":is(", "selectorIs"), (":where(", "selectorWhere")] {
        let mut cursor = 0usize;
        while let Some(relative_start) = source[cursor..].find(prefix) {
            let start = cursor + relative_start;
            let inner_start = start + prefix.len();
            let Some(relative_end) = source[inner_start..].find(')') else {
                break;
            };
            let end = inner_start + relative_end;
            let inner = source[inner_start..end].trim();
            let css_before = source[start..=end].to_string();
            let pseudo_name = prefix.trim_start_matches(':').trim_end_matches('(');
            if let Some((source_kind, css_after, before, after, witness)) =
                selector_witness_candidate(pseudo_name, source_kind, inner)
                && transformed_source.contains(&css_after)
                && !transformed_source.contains(&css_before)
            {
                let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
                    pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
                    before,
                    after,
                    proof: EggRewriteProofV0::new(
                        true,
                        ObligationFamilyIdV0::CascadeSafetyFloor,
                        true,
                        witness,
                    ),
                });
                witnesses.push(EggRewriteSourceWitnessV0 {
                    pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
                    source_kind,
                    byte_offset: start,
                    css_before,
                    css_after,
                    execution,
                });
            }
            cursor = end + 1;
        }
    }
    witnesses
}

fn calc_rewrite_witnesses(
    source: &str,
    transformed_source: &str,
) -> Vec<EggRewriteSourceWitnessV0> {
    let mut witnesses = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative_start) = source[cursor..].find("calc(") {
        let start = cursor + relative_start;
        let inner_start = start + "calc(".len();
        let Some(relative_end) = source[inner_start..].find(')') else {
            break;
        };
        let end = inner_start + relative_end;
        let inner = source[inner_start..end].trim();
        let css_before = source[start..=end].to_string();
        if let Some(candidate) = calc_rewrite_candidate(inner)
            && transformed_source.contains(candidate.css_after.as_str())
            && !transformed_source.contains(&css_before)
        {
            let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::CalcReduction.id(),
                before: format!("(calc {})", candidate.before),
                after: candidate.after,
                proof: EggRewriteProofV0::new(
                    false,
                    ObligationFamilyIdV0::ComputedValuePreservation,
                    true,
                    candidate.witness,
                ),
            });
            witnesses.push(EggRewriteSourceWitnessV0 {
                pass_id: TransformPassKind::CalcReduction.id(),
                source_kind: candidate.source_kind,
                byte_offset: start,
                css_before,
                css_after: candidate.css_after,
                execution,
            });
        }
        cursor = end + 1;
    }
    witnesses
}

fn stale_prefix_removal_witnesses(
    source: &str,
    dialect: StyleDialect,
    transformed_source: &str,
) -> Vec<EggRewriteSourceWitnessV0> {
    collect_stale_vendor_prefix_removal_proof_candidates_from_source(source, dialect)
        .into_iter()
        .filter_map(|candidate| {
            let css_before =
                source[candidate.source_span_start..candidate.source_span_end].to_string();
            if transformed_source.contains(&css_before) {
                return None;
            }
            let css_after = source
                [candidate.unprefixed_peer_span_start..candidate.unprefixed_peer_span_end]
                .to_string();
            if !transformed_source.contains(&css_after) {
                return None;
            }

            let prefixed_property = egg_safe_symbol(candidate.prefixed_property.as_str());
            let unprefixed_property = egg_safe_symbol(candidate.unprefixed_property);
            let value = egg_safe_symbol(candidate.value.as_str());
            let importance = if candidate.important {
                "important"
            } else {
                "normal"
            };
            let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::StalePrefixRemoval.id(),
                before: format!(
                    "(stale-prefix-decl {prefixed_property} {unprefixed_property} {value} {importance})"
                ),
                after: format!("(decl {unprefixed_property} {value} {importance})"),
                proof: EggRewriteProofV0::new(
                    false,
                    ObligationFamilyIdV0::ComputedValuePreservation,
                    true,
                    format!(
                        "{} has exact unprefixed declaration peer {} with the same value and importance",
                        candidate.prefixed_property, candidate.unprefixed_property
                    ),
                ),
            });
            Some(EggRewriteSourceWitnessV0 {
                pass_id: TransformPassKind::StalePrefixRemoval.id(),
                source_kind: "stalePrefixExactPeer",
                byte_offset: candidate.source_span_start,
                css_before,
                css_after,
                execution,
            })
        })
        .collect()
}

fn selector_witness_candidate(
    pseudo_name: &str,
    source_kind: &'static str,
    inner: &str,
) -> Option<(&'static str, String, String, String, String)> {
    if pseudo_name == "is"
        && let Some((symbol, css_ident)) = selector_single_argument_parts(inner)
    {
        return Some((
            source_kind,
            format!(".{css_ident}"),
            format!("(is {symbol})"),
            symbol,
            "actual CSS selectorIs single-argument rewrite".to_string(),
        ));
    }

    let args = split_simple_selector_arguments(inner)?;
    let [left, right] = args.as_slice() else {
        return None;
    };
    if left != right {
        return None;
    }
    let (symbol, css_ident) = selector_single_argument_parts(left)?;
    match pseudo_name {
        "is" => Some((
            "selectorIsDedup",
            format!(".{css_ident}"),
            format!("(is (list {symbol} {symbol}))"),
            symbol,
            "actual CSS selectorIs duplicate-argument rewrite".to_string(),
        )),
        "where" => Some((
            "selectorWhereDedup",
            format!(":where(.{css_ident})"),
            format!("(where (list {symbol} {symbol}))"),
            format!("(where {symbol})"),
            "actual CSS selectorWhere duplicate-argument rewrite".to_string(),
        )),
        _ => None,
    }
}

fn egg_safe_symbol(value: &str) -> String {
    let mut symbol = String::with_capacity(value.len().max(1));
    for byte in value.bytes() {
        let character = byte as char;
        if character.is_ascii_alphanumeric() {
            symbol.push(character.to_ascii_lowercase());
        } else {
            let _ = write!(&mut symbol, "_{byte:02x}");
        }
    }
    if symbol.is_empty() {
        "empty".to_string()
    } else if symbol
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_digit())
    {
        format!("v_{symbol}")
    } else {
        symbol
    }
}

fn split_simple_selector_arguments(inner: &str) -> Option<Vec<String>> {
    let args = inner
        .split(',')
        .map(str::trim)
        .map(str::to_string)
        .collect::<Vec<_>>();
    (!args.is_empty() && args.iter().all(|arg| !arg.is_empty())).then_some(args)
}

fn selector_single_argument_parts(inner: &str) -> Option<(String, String)> {
    let class_name = inner.trim().strip_prefix('.')?;
    if class_name.is_empty()
        || !class_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
    {
        return None;
    }
    Some((symbol_for_css_ident(class_name), class_name.to_string()))
}

fn symbol_for_css_ident(value: &str) -> String {
    value.replace('-', "_")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CalcRewriteCandidate {
    before: String,
    after: String,
    css_after: String,
    source_kind: &'static str,
    witness: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CalcNumericValue {
    value: i64,
    unit: String,
}

fn calc_rewrite_candidate(inner: &str) -> Option<CalcRewriteCandidate> {
    let parts = inner.split_whitespace().collect::<Vec<_>>();
    let [left, operator, right] = parts.as_slice() else {
        return None;
    };
    let left_value = parse_calc_numeric_value(left)?;
    let right_value = parse_calc_numeric_value(right)?;
    if left_value.unit != right_value.unit {
        return None;
    }
    let term_left = calc_numeric_term(&left_value);
    let term_right = calc_numeric_term(&right_value);
    match *operator {
        "+" => Some(calc_fold_candidate(
            format!("(+ {term_left} {term_right})"),
            left_value.value + right_value.value,
            &left_value.unit,
            "calcSameUnitAdd",
            "actual CSS calc same-unit addition rewrite",
        )),
        "-" => Some(calc_fold_candidate(
            format!("(- {term_left} {term_right})"),
            left_value.value - right_value.value,
            &left_value.unit,
            "calcSameUnitSub",
            "actual CSS calc same-unit subtraction rewrite",
        )),
        "*" if right_value.value == 1 && right_value.unit.is_empty() => {
            Some(calc_passthrough_candidate(
                format!("(* {term_left} 1)"),
                &left_value,
                "calcIdentity",
                "actual CSS calc multiplicative identity rewrite",
            ))
        }
        "*" if left_value.value == 1 && left_value.unit.is_empty() => {
            Some(calc_passthrough_candidate(
                format!("(* 1 {term_right})"),
                &right_value,
                "calcIdentity",
                "actual CSS calc multiplicative identity rewrite",
            ))
        }
        "*" if right_value.value == 0 && right_value.unit.is_empty() => Some(calc_fold_candidate(
            format!("(* {term_left} 0)"),
            0,
            "",
            "calcZero",
            "actual CSS calc safe zero multiplication rewrite",
        )),
        "*" if left_value.value == 0 && left_value.unit.is_empty() => Some(calc_fold_candidate(
            format!("(* 0 {term_right})"),
            0,
            "",
            "calcZero",
            "actual CSS calc safe zero multiplication rewrite",
        )),
        "/" if right_value.value == 1 && right_value.unit.is_empty() => {
            Some(calc_passthrough_candidate(
                format!("(/ {term_left} 1)"),
                &left_value,
                "calcIdentity",
                "actual CSS calc division identity rewrite",
            ))
        }
        _ => None,
    }
}

fn parse_calc_numeric_value(text: &str) -> Option<CalcNumericValue> {
    let split = text
        .char_indices()
        .find_map(|(index, ch)| (!matches!(ch, '-' | '+') && !ch.is_ascii_digit()).then_some(index))
        .unwrap_or(text.len());
    let (value, unit) = text.split_at(split);
    let value = value.parse::<i64>().ok()?;
    unit.chars()
        .all(|ch| ch.is_ascii_alphabetic() || ch == '%')
        .then_some(CalcNumericValue {
            value,
            unit: unit.to_string(),
        })
}

fn calc_numeric_term(value: &CalcNumericValue) -> String {
    if value.unit.is_empty() {
        value.value.to_string()
    } else {
        format!("(unit {} {})", value.value, value.unit)
    }
}

fn calc_fold_candidate(
    before: String,
    value: i64,
    unit: &str,
    source_kind: &'static str,
    witness: &'static str,
) -> CalcRewriteCandidate {
    let result = CalcNumericValue {
        value,
        unit: unit.to_string(),
    };
    CalcRewriteCandidate {
        before,
        after: calc_numeric_term(&result),
        css_after: format!("{}{}", result.value, result.unit),
        source_kind,
        witness: witness.to_string(),
    }
}

fn calc_passthrough_candidate(
    before: String,
    value: &CalcNumericValue,
    source_kind: &'static str,
    witness: &'static str,
) -> CalcRewriteCandidate {
    CalcRewriteCandidate {
        before,
        after: calc_numeric_term(value),
        css_after: format!("{}{}", value.value, value.unit),
        source_kind,
        witness: witness.to_string(),
    }
}

fn rewrite_pattern(text: &str) -> Option<Pattern<CssRewriteLanguage>> {
    text.parse().ok()
}

pub(crate) fn calc_const_fold_rule<N>(
    name: &'static str,
    search: &'static str,
    operator: CalcFoldOperator,
    unit_var: Option<Var>,
) -> Option<Rewrite<CssRewriteLanguage, N>>
where
    N: Analysis<CssRewriteLanguage>,
{
    Rewrite::new(
        name,
        rewrite_pattern(search)?,
        ConstFoldSameUnitApplier::new(operator, unit_var)?,
    )
    .ok()
}

fn egg_var(name: &str) -> Option<Var> {
    name.parse().ok()
}

pub(crate) fn rewrite_rules_for_pass<N>(
    pass_id: &'static str,
) -> Option<Vec<Rewrite<CssRewriteLanguage, N>>>
where
    N: Analysis<CssRewriteLanguage>,
{
    if pass_id == TransformPassKind::SelectorIsWhereCompression.id() {
        return Some(vec![
            egg_rewrite!("single-is-selector"; "(is ?a)" => "?a"),
            egg_rewrite!("nested-is-selector"; "(is (is ?a))" => "?a"),
            egg_rewrite!("duplicate-is-selector"; "(is (list ?a ?a))" => "?a"),
            egg_rewrite!("duplicate-where-selector"; "(where (list ?a ?a))" => "(where ?a)"),
        ]);
    }
    if pass_id == TransformPassKind::CalcReduction.id() {
        let mut rules = vec![
            egg_rewrite!("unwrap-calc"; "(calc ?a)" => "?a"),
            egg_rewrite!("add-zero-right"; "(+ ?a 0)" => "?a"),
            egg_rewrite!("add-zero-left"; "(+ 0 ?a)" => "?a"),
            egg_rewrite!("sub-zero-right"; "(- ?a 0)" => "?a"),
            egg_rewrite!("self-sub"; "(- ?a ?a)" => "0"),
            egg_rewrite!("mul-one-right"; "(* ?a 1)" => "?a"),
            egg_rewrite!("mul-one-left"; "(* 1 ?a)" => "?a"),
            egg_rewrite!("mul-zero-right"; "(* ?a 0)" => "0"),
            egg_rewrite!("mul-zero-left"; "(* 0 ?a)" => "0"),
            egg_rewrite!("div-one-right"; "(/ ?a 1)" => "?a"),
        ];
        if let Some(rule) = calc_const_fold_rule(
            "constfold-add-number",
            "(+ ?a ?b)",
            CalcFoldOperator::Add,
            None,
        ) {
            rules.push(rule);
        }
        if let Some(unit_var) = egg_var("?u")
            && let Some(rule) = calc_const_fold_rule(
                "constfold-add-same-unit",
                "(+ (unit ?a ?u) (unit ?b ?u))",
                CalcFoldOperator::Add,
                Some(unit_var),
            )
        {
            rules.push(rule);
        }
        if let Some(rule) = calc_const_fold_rule(
            "constfold-sub-number",
            "(- ?a ?b)",
            CalcFoldOperator::Sub,
            None,
        ) {
            rules.push(rule);
        }
        if let Some(unit_var) = egg_var("?u")
            && let Some(rule) = calc_const_fold_rule(
                "constfold-sub-same-unit",
                "(- (unit ?a ?u) (unit ?b ?u))",
                CalcFoldOperator::Sub,
                Some(unit_var),
            )
        {
            rules.push(rule);
        }
        return Some(rules);
    }
    if pass_id == TransformPassKind::ShorthandCombining.id() {
        return Some(vec![
            egg_rewrite!("box4-all-equal"; "(box4 ?a ?a ?a ?a)" => "(box1 ?a)"),
            egg_rewrite!("box4-vertical-horizontal"; "(box4 ?a ?b ?a ?b)" => "(box2 ?a ?b)"),
            egg_rewrite!("box4-horizontal-pair"; "(box4 ?a ?b ?c ?b)" => "(box3 ?a ?b ?c)"),
        ]);
    }
    if pass_id == TransformPassKind::StalePrefixRemoval.id() {
        return Some(vec![
            egg_rewrite!("stale-prefix-exact-peer"; "(stale-prefix-decl ?p ?u ?v ?i)" => "(decl ?u ?v ?i)"),
        ]);
    }
    None
}

fn blocked_execution(
    candidate: EggRewriteCandidateV0,
    blocked_reason: Option<&'static str>,
) -> EggRewriteExecutionV0 {
    EggRewriteExecutionV0 {
        schema_version: "0",
        product: "omena-transform-egg.execution",
        pass_id: candidate.pass_id,
        accepted: false,
        blocked_reason,
        before: candidate.before.clone(),
        after: candidate.before,
        expected_after: candidate.after,
        after_matches_candidate: false,
        engine: "egg",
        iteration_limit: 0,
        iteration_count: 0,
        eclass_count: 0,
        enode_count: 0,
        mdl_bits: None,
        mdl_residual_bits: None,
        mdl_unit: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EggRewriteCandidateV0, EggRewriteProofV0, decide_egg_rewrite, execute_egg_rewrite,
        execute_egg_rewrite_witnesses_for_css_source, plan_egg_rewrite_passes,
        plan_egg_rewrite_passes_for_source, rewrite_rules_for_pass,
        summarize_contextual_eqsat_scaffold_v0, summarize_mdl_extraction_mode,
        summarize_omena_transform_egg_boundary,
    };
    use omena_evidence_graph::ObligationFamilyIdV0;
    use omena_parser::StyleDialect;
    use omena_transform_cst::TransformPassKind;

    #[test]
    fn exposes_selector_calc_and_shorthand_optional_egg_boundary() {
        let boundary = summarize_omena_transform_egg_boundary();

        assert_eq!(boundary.product, "omena-transform-egg.boundary");
        assert_eq!(
            boundary.managed_pass_ids,
            vec![
                "selector-is-where-compression",
                "calc-reduction",
                "shorthand-combining",
                "stale-prefix-removal"
            ]
        );
        assert_eq!(boundary.proof_obligations.len(), 6);
    }

    #[test]
    fn mdl_extraction_default_preserves_ast_size() {
        let summary = summarize_mdl_extraction_mode();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.product, "omena-transform-egg.mdl-extraction");
        assert!(summary.default_preserves_ast_size);
        assert_eq!(summary.layer_marker, "mdl-bits");
        assert_eq!(summary.unit, "bit");
        assert_eq!(summary.feature_gate, "mdl");
    }

    #[test]
    fn plans_requested_egg_passes_through_transform_pass_planner() {
        let plan = plan_egg_rewrite_passes(true, true);

        assert_eq!(
            plan.planned_pass_ids,
            vec!["selector-is-where-compression", "calc-reduction"]
        );
        assert_eq!(plan.pass_plan.violated_dag_edge_count, 0);
    }

    #[test]
    fn plans_egg_passes_from_css_source() {
        let plan = plan_egg_rewrite_passes_for_source(".a:is(.ready) { width: calc(7 + 0); }");

        assert_eq!(
            plan.planned_pass_ids,
            vec!["selector-is-where-compression", "calc-reduction"]
        );
        assert_eq!(plan.pass_plan.violated_dag_edge_count, 0);
    }

    #[test]
    fn contextual_eqsat_scaffold_stays_no_egglog_binding() {
        let scaffold = summarize_contextual_eqsat_scaffold_v0();

        assert_eq!(scaffold.schema_version, "0");
        assert_eq!(
            scaffold.product,
            "omena-transform-egg.contextual-eqsat-scaffold"
        );
        assert_eq!(scaffold.claim_level, "m6ScaffoldOnlyNoEgglogBinding");
        assert_eq!(scaffold.current_engine, "egg");
        assert!(scaffold.egg_engine_ready);
        assert!(!scaffold.egglog_binding_ready);
        assert!(!scaffold.external_datalog_host_ready);
        assert!(!scaffold.three_view_fusion_ready);
        assert!(!scaffold.theorem_claimed);
        assert!(!scaffold.public_safety_claim_ready);
        assert_eq!(
            scaffold.modal_witness_product,
            "omena-cascade.modal-check-witness"
        );
        assert_eq!(scaffold.modal_bridge_claim_level, "dependencyDeclaredOnly");
        assert_eq!(scaffold.paper_substrate_claim_level, "draftScaffoldOnly");
        assert_eq!(
            scaffold.managed_pass_ids,
            vec![
                "selector-is-where-compression",
                "calc-reduction",
                "shorthand-combining",
                "stale-prefix-removal"
            ]
        );
        assert!(
            scaffold
                .supported_claims
                .contains(&"contextual equality-saturation scaffold for M6 positioning")
        );
        assert!(scaffold.deferred_claims.contains(&"egglog Rust binding"));
        assert!(scaffold.deferred_claims.contains(&"full three-view fusion"));
    }

    #[test]
    fn accepts_selector_rewrite_only_with_specificity_and_provenance_witnesses() {
        let decision = decide_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
            before: ":is(.a, .b)".to_string(),
            after: ".a,.b".to_string(),
            proof: EggRewriteProofV0::new(
                true,
                ObligationFamilyIdV0::CascadeSafetyFloor,
                true,
                "specificity tuple preserved",
            ),
        });

        assert!(decision.accepted);
        assert_eq!(decision.blocked_reason, None);
    }

    #[test]
    fn rejects_calc_rewrite_without_computed_value_witness() {
        let decision = decide_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::CalcReduction.id(),
            before: "calc(1rem + 2px)".to_string(),
            after: "1rem".to_string(),
            proof: EggRewriteProofV0::new(
                false,
                ObligationFamilyIdV0::CascadeSafetyFloor,
                true,
                "candidate generated",
            ),
        });

        assert!(!decision.accepted);
        assert_eq!(
            decision.blocked_reason,
            Some("calc rewrite does not preserve computed value")
        );
    }

    #[test]
    fn rejects_shorthand_rewrite_without_computed_value_witness() {
        let decision = decide_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::ShorthandCombining.id(),
            before: "(box4 0 0 0 0)".to_string(),
            after: "(box1 0)".to_string(),
            proof: EggRewriteProofV0::new(
                false,
                ObligationFamilyIdV0::CascadeSafetyFloor,
                true,
                "candidate generated",
            ),
        });

        assert!(!decision.accepted);
        assert_eq!(
            decision.blocked_reason,
            Some("shorthand rewrite does not preserve computed value")
        );
    }

    #[test]
    fn executes_selector_rewrite_through_egg_engine() {
        let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
            before: "(is buttonPrimary)".to_string(),
            after: "buttonPrimary".to_string(),
            proof: EggRewriteProofV0::new(
                true,
                ObligationFamilyIdV0::CascadeSafetyFloor,
                true,
                "single :is() argument keeps specificity",
            ),
        });

        assert!(execution.accepted);
        assert_eq!(execution.product, "omena-transform-egg.execution");
        assert_eq!(execution.engine, "egg");
        assert_eq!(execution.after, "buttonPrimary");
        assert_eq!(execution.iteration_limit, 8);
        assert!(execution.iteration_count > 0);
        assert!(execution.eclass_count > 0);
        assert!(execution.enode_count > 0);
    }

    #[test]
    fn executes_selector_dedup_rewrites_through_egg_engine() {
        let is_execution = execute_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
            before: "(is (list ready ready))".to_string(),
            after: "ready".to_string(),
            proof: EggRewriteProofV0::new(
                true,
                ObligationFamilyIdV0::CascadeSafetyFloor,
                true,
                "duplicate :is() argument keeps specificity",
            ),
        });
        let where_execution = execute_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::SelectorIsWhereCompression.id(),
            before: "(where (list ready ready))".to_string(),
            after: "(where ready)".to_string(),
            proof: EggRewriteProofV0::new(
                true,
                ObligationFamilyIdV0::CascadeSafetyFloor,
                true,
                "duplicate :where() argument keeps zero specificity",
            ),
        });

        assert!(is_execution.accepted);
        assert_eq!(is_execution.after, "ready");
        assert!(where_execution.accepted);
        assert_eq!(where_execution.after, "(where ready)");
    }

    #[test]
    fn executes_calc_rewrite_through_egg_engine() {
        let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
            pass_id: TransformPassKind::CalcReduction.id(),
            before: "(calc (+ width 0))".to_string(),
            after: "width".to_string(),
            proof: EggRewriteProofV0::new(
                false,
                ObligationFamilyIdV0::ComputedValuePreservation,
                true,
                "additive identity preserves computed value",
            ),
        });

        assert!(execution.accepted);
        assert_eq!(execution.after, "width");
        assert!(execution.after_matches_candidate);
    }

    #[test]
    fn executes_extended_calc_identity_rewrites_through_egg_engine() {
        for (before, after) in [
            ("(calc (- width 0))", "width"),
            ("(calc (/ width 1))", "width"),
            ("(calc (* width 0))", "0"),
            ("(calc (- width width))", "0"),
        ] {
            let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::CalcReduction.id(),
                before: before.to_string(),
                after: after.to_string(),
                proof: EggRewriteProofV0::new(
                    false,
                    ObligationFamilyIdV0::ComputedValuePreservation,
                    true,
                    "calc algebra identity preserves computed value",
                ),
            });

            assert!(execution.accepted, "{before} -> {after}");
            assert_eq!(execution.after, after);
        }
    }

    #[test]
    fn executes_same_unit_calc_const_folding_through_egg_engine() {
        for (before, after) in [
            ("(calc (+ (unit 1 px) (unit 2 px)))", "(unit 3 px)"),
            ("(calc (- (unit 10 rem) (unit 2 rem)))", "(unit 8 rem)"),
            ("(calc (+ 1 2))", "3"),
        ] {
            let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::CalcReduction.id(),
                before: before.to_string(),
                after: after.to_string(),
                proof: EggRewriteProofV0::new(
                    false,
                    ObligationFamilyIdV0::ComputedValuePreservation,
                    true,
                    "same-unit calc arithmetic preserves computed value",
                ),
            });

            assert!(execution.accepted, "{before} -> {after}");
            assert_eq!(execution.after, after);
        }
    }

    #[test]
    fn executes_box_shorthand_rewrites_through_egg_engine() {
        for (before, after) in [
            ("(box4 0 0 0 0)", "(box1 0)"),
            ("(box4 1 2 1 2)", "(box2 1 2)"),
            ("(box4 1 2 3 2)", "(box3 1 2 3)"),
        ] {
            let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
                pass_id: TransformPassKind::ShorthandCombining.id(),
                before: before.to_string(),
                after: after.to_string(),
                proof: EggRewriteProofV0::new(
                    false,
                    ObligationFamilyIdV0::ComputedValuePreservation,
                    true,
                    "box shorthand expansion preserves computed value",
                ),
            });

            assert!(execution.accepted, "{before} -> {after}");
            assert_eq!(execution.after, after);
        }
    }

    #[test]
    fn exposes_shorthand_rewrite_rules_for_managed_pass() {
        let rules = rewrite_rules_for_pass::<()>(TransformPassKind::ShorthandCombining.id());

        assert!(rules.is_some_and(|rules| rules.len() == 3));
    }

    #[test]
    fn executes_css_source_witnesses_through_egg_engine() {
        let source = ".a:is(.ready) { width: calc(1px + 2px); } .b:is(.x, .x) { color: red; } .c:where(.y, .y) { color: blue; }";
        let transformed =
            ".a.ready { width: 3px; } .b.x { color: red; } .c:where(.y) { color: blue; }";
        let plan = plan_egg_rewrite_passes_for_source(source);
        let witnesses = execute_egg_rewrite_witnesses_for_css_source(
            source,
            StyleDialect::Css,
            transformed,
            &plan.planned_pass_ids,
        );

        assert_eq!(witnesses.len(), 4);
        assert!(witnesses.iter().all(|witness| witness.execution.accepted));
        assert!(
            witnesses
                .iter()
                .any(|witness| witness.pass_id == "selector-is-where-compression")
        );
        assert!(
            witnesses
                .iter()
                .any(|witness| witness.pass_id == "calc-reduction")
        );
        assert!(witnesses.iter().any(|witness| {
            witness.source_kind == "selectorIsDedup" && witness.css_after == ".x"
        }));
        assert!(witnesses.iter().any(|witness| {
            witness.source_kind == "selectorWhereDedup" && witness.css_after == ":where(.y)"
        }));
        assert!(witnesses.iter().any(|witness| {
            witness.source_kind == "calcSameUnitAdd"
                && witness.css_after == "3px"
                && witness.execution.after == "(unit 3 px)"
        }));
    }

    #[test]
    fn executes_stale_prefix_removal_source_witness_through_egg_engine() {
        let source = ".a { -webkit-user-select: none; user-select: none; }";
        let transformed = ".a {  user-select: none; }";
        let witnesses = execute_egg_rewrite_witnesses_for_css_source(
            source,
            StyleDialect::Css,
            transformed,
            &[TransformPassKind::StalePrefixRemoval.id()],
        );

        assert_eq!(witnesses.len(), 1);
        let witness = &witnesses[0];
        assert_eq!(witness.pass_id, "stale-prefix-removal");
        assert_eq!(witness.source_kind, "stalePrefixExactPeer");
        assert_eq!(witness.css_before, "-webkit-user-select: none;");
        assert_eq!(witness.css_after, "user-select: none;");
        assert!(witness.execution.accepted);
        assert_eq!(witness.execution.engine, "egg");
        assert_eq!(witness.execution.after, witness.execution.expected_after);
    }

    #[test]
    fn mdl_default_ast_size_matches_100_fixture_differential_corpus() {
        let selector_cases = (0..50).map(|index| {
            (
                TransformPassKind::SelectorIsWhereCompression.id(),
                format!("(is token{index})"),
                format!("token{index}"),
                true,
                false,
                "single :is() argument keeps specificity",
            )
        });
        let calc_cases = (0..50).map(|index| {
            let left = index + 1;
            let right = 50 - index;
            (
                TransformPassKind::CalcReduction.id(),
                format!("(calc (+ (unit {left} px) (unit {right} px)))"),
                format!("(unit {} px)", left + right),
                false,
                true,
                "same-unit calc arithmetic preserves computed value",
            )
        });
        let cases = selector_cases.chain(calc_cases).collect::<Vec<_>>();

        assert_eq!(cases.len(), 100);
        for (
            pass_id,
            before,
            expected_after,
            specificity_preserved,
            computed_value_preserved,
            witness,
        ) in cases
        {
            let execution = execute_egg_rewrite(EggRewriteCandidateV0 {
                pass_id,
                before: before.clone(),
                after: expected_after.clone(),
                proof: EggRewriteProofV0::new(
                    specificity_preserved,
                    ObligationFamilyIdV0::from_computed_value_preservation(
                        computed_value_preserved,
                    ),
                    true,
                    witness,
                ),
            });

            assert!(execution.accepted, "{before} -> {expected_after}");
            assert_eq!(execution.after, expected_after);
            assert!(execution.after_matches_candidate);
            assert_eq!(execution.mdl_bits, None);
            assert_eq!(execution.mdl_unit, None);
        }
    }
}

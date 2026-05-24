use egg::{CostFunction, Id, Language};
use serde::Serialize;

use crate::CssRewriteLanguage;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MdlExtractionModeV0 {
    #[default]
    AstSize,
    #[cfg(feature = "mdl")]
    TwoPartUniform,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MdlExtractionModeSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub default_mode: MdlExtractionModeV0,
    pub alternative_modes: Vec<MdlExtractionModeV0>,
    pub unit: &'static str,
    pub feature_gate: &'static str,
    pub default_preserves_ast_size: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MdlExtractionCostV0 {
    mode: MdlExtractionModeV0,
}

impl MdlExtractionCostV0 {
    pub const fn new(mode: MdlExtractionModeV0) -> Self {
        Self { mode }
    }

    pub const fn default_ast_size() -> Self {
        Self::new(MdlExtractionModeV0::AstSize)
    }
}

impl Default for MdlExtractionCostV0 {
    fn default() -> Self {
        Self::default_ast_size()
    }
}

impl CostFunction<CssRewriteLanguage> for MdlExtractionCostV0 {
    type Cost = usize;

    fn cost<C>(&mut self, enode: &CssRewriteLanguage, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        let ast_size = 1 + enode
            .children()
            .iter()
            .copied()
            .map(&mut costs)
            .sum::<usize>();

        match self.mode {
            MdlExtractionModeV0::AstSize => ast_size,
            #[cfg(feature = "mdl")]
            MdlExtractionModeV0::TwoPartUniform => ast_size + two_part_uniform_model_penalty(enode),
        }
    }
}

#[cfg(feature = "mdl")]
fn two_part_uniform_model_penalty(enode: &CssRewriteLanguage) -> usize {
    match enode {
        CssRewriteLanguage::Num(_) | CssRewriteLanguage::Symbol(_) => 1,
        CssRewriteLanguage::Add(_)
        | CssRewriteLanguage::Sub(_)
        | CssRewriteLanguage::Mul(_)
        | CssRewriteLanguage::Div(_)
        | CssRewriteLanguage::Unit(_)
        | CssRewriteLanguage::List(_) => 2,
        CssRewriteLanguage::Calc(_) | CssRewriteLanguage::Is(_) | CssRewriteLanguage::Where(_) => 3,
    }
}

pub fn summarize_mdl_extraction_mode() -> MdlExtractionModeSummaryV0 {
    #[cfg(feature = "mdl")]
    let alternative_modes = vec![MdlExtractionModeV0::TwoPartUniform];
    #[cfg(not(feature = "mdl"))]
    let alternative_modes = Vec::new();

    MdlExtractionModeSummaryV0 {
        schema_version: "0",
        product: "omena-transform-egg.mdl-extraction",
        default_mode: MdlExtractionModeV0::AstSize,
        alternative_modes,
        unit: "bit",
        feature_gate: "mdl",
        default_preserves_ast_size: true,
    }
}

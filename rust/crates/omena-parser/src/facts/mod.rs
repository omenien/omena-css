mod animations;
mod at_rules;
mod css_modules;
mod icss;
mod sass;
mod selectors;
mod variables;

use omena_syntax::StyleDialect;

pub use animations::{ParsedAnimationFact, ParsedAnimationFactKind};
pub use at_rules::ParsedAtRuleFact;
pub use css_modules::{
    ParsedCssModuleComposesEdgeFact, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFact,
    ParsedCssModuleComposesFactKind, ParsedCssModuleValueDefinitionEdgeFact,
    ParsedCssModuleValueFact, ParsedCssModuleValueFactKind, ParsedCssModuleValueImportEdgeFact,
};
pub use icss::{
    ParsedIcssExportEdgeFact, ParsedIcssFact, ParsedIcssFactKind, ParsedIcssImportEdgeFact,
};
pub use sass::{
    ParsedExtendTargetFact, ParsedExtendTargetFactKind, ParsedSassIncludeFact,
    ParsedSassModuleEdgeFact, ParsedSassModuleEdgeFactKind, ParsedSassSymbolFact,
    ParsedSassSymbolFactKind,
};
pub use selectors::{ParsedSelectorFact, ParsedSelectorFactKind};
pub use variables::{ParsedVariableFact, ParsedVariableFactKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStyleFacts {
    pub product: &'static str,
    pub dialect: StyleDialect,
    pub selector_count: usize,
    pub selectors: Vec<ParsedSelectorFact>,
    pub variable_count: usize,
    pub variables: Vec<ParsedVariableFact>,
    pub sass_symbol_count: usize,
    pub sass_symbols: Vec<ParsedSassSymbolFact>,
    pub sass_include_count: usize,
    pub sass_includes: Vec<ParsedSassIncludeFact>,
    pub sass_module_edge_count: usize,
    pub sass_module_edges: Vec<ParsedSassModuleEdgeFact>,
    pub extend_target_count: usize,
    pub extend_targets: Vec<ParsedExtendTargetFact>,
    pub animation_count: usize,
    pub animations: Vec<ParsedAnimationFact>,
    pub css_module_value_count: usize,
    pub css_module_values: Vec<ParsedCssModuleValueFact>,
    pub css_module_value_import_edge_count: usize,
    pub css_module_value_import_edges: Vec<ParsedCssModuleValueImportEdgeFact>,
    pub css_module_value_definition_edge_count: usize,
    pub css_module_value_definition_edges: Vec<ParsedCssModuleValueDefinitionEdgeFact>,
    pub css_module_composes_count: usize,
    pub css_module_composes: Vec<ParsedCssModuleComposesFact>,
    pub css_module_composes_edge_count: usize,
    pub css_module_composes_edges: Vec<ParsedCssModuleComposesEdgeFact>,
    pub icss_count: usize,
    pub icss: Vec<ParsedIcssFact>,
    pub icss_import_edge_count: usize,
    pub icss_import_edges: Vec<ParsedIcssImportEdgeFact>,
    pub icss_export_edge_count: usize,
    pub icss_export_edges: Vec<ParsedIcssExportEdgeFact>,
    pub at_rule_count: usize,
    pub at_rules: Vec<ParsedAtRuleFact>,
    pub error_count: usize,
}

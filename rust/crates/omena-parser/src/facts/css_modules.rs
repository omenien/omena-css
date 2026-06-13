use cstree::text::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueFact {
    pub kind: ParsedCssModuleValueFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleValueFactKind {
    Definition,
    Reference,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueImportEdgeFact {
    pub remote_name: String,
    pub local_name: String,
    pub import_source: String,
    pub local_range: TextRange,
    pub remote_range: TextRange,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleValueDefinitionEdgeFact {
    pub definition_name: String,
    pub reference_names: Vec<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleComposesFact {
    pub kind: ParsedCssModuleComposesFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleComposesFactKind {
    Target,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCssModuleComposesEdgeFact {
    pub kind: ParsedCssModuleComposesEdgeKind,
    pub owner_selector_names: Vec<String>,
    pub target_names: Vec<String>,
    pub import_source: Option<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedCssModuleComposesEdgeKind {
    Local,
    Global,
    External,
}

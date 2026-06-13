use cstree::text::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassSymbolFact {
    pub kind: ParsedSassSymbolFactKind,
    pub symbol_kind: &'static str,
    pub name: String,
    pub role: &'static str,
    pub namespace: Option<String>,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedSassSymbolFactKind {
    VariableDeclaration,
    VariableReference,
    MixinDeclaration,
    MixinInclude,
    FunctionDeclaration,
    FunctionCall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassIncludeFact {
    pub name: String,
    pub namespace: Option<String>,
    pub params: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSassModuleEdgeFact {
    pub kind: ParsedSassModuleEdgeFactKind,
    pub source: String,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    /// RFC-0007-D1 (#44): whether this `@import` target carries a trailing media
    /// qualifier (`@import "foo" screen`, `@import "foo" (min-width: 100px)`). Sass
    /// keeps media-qualified imports as plain CSS (NOT deprecated). Recoverable only
    /// in the parser, where the target's comma-peer segment is still tokenized: a
    /// non-`Comma` significant token after the target String marks the qualifier.
    /// Always `false` for `Use`/`Forward` edges (media qualifiers are `@import`-only).
    pub media_qualified: bool,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedSassModuleEdgeFactKind {
    Use,
    Forward,
    Import,
}

/// RFC-0007-E1 (#45): the target of an `@extend` rule. The `ScssExtendRule` node previously
/// parsed and then discarded its target, so an `@extend %nonexistent` / `@extend .missing`
/// (a dart-sass hard error) went unreported. This fact captures the simple target selector,
/// whether it carries the `!optional` flag, and its source range for diagnostic anchoring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedExtendTargetFact {
    pub kind: ParsedExtendTargetFactKind,
    pub name: String,
    pub optional: bool,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedExtendTargetFactKind {
    Class,
    Placeholder,
}

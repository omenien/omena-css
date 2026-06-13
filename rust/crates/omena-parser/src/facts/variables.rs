use cstree::text::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVariableFact {
    pub kind: ParsedVariableFactKind,
    pub name: String,
    pub range: TextRange,
    /// For a `CustomPropertyReference` written as `var(--x, fallback)`, records that a
    /// top-level fallback argument is present. The reference cannot be "missing" in any
    /// observable way — the fallback guarantees a value — so the `missingCustomProperty`
    /// lint must skip it. `false` for declarations and fallback-less references.
    pub has_fallback: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedVariableFactKind {
    ScssDeclaration,
    ScssReference,
    LessDeclaration,
    LessReference,
    CustomPropertyDeclaration,
    CustomPropertyReference,
}

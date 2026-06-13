use cstree::text::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSelectorFact {
    pub kind: ParsedSelectorFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedSelectorFactKind {
    Class,
    Id,
    Placeholder,
}

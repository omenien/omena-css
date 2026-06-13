use cstree::text::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnimationFact {
    pub kind: ParsedAnimationFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedAnimationFactKind {
    KeyframesDeclaration,
    AnimationNameReference,
}

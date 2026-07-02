//! Typed CST wrappers over the raw parser syntax tree.
//!
//! These wrappers give downstream parser consumers stable node categories
//! without exposing the full cstree traversal contract at every call site.

use cstree::{syntax::SyntaxNode, text::TextRange};
use omena_syntax::SyntaxKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCst {
    root: SyntaxNode<SyntaxKind>,
}

impl ParsedCst {
    pub fn new(root: SyntaxNode<SyntaxKind>) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &SyntaxNode<SyntaxKind> {
        &self.root
    }

    pub fn stylesheet(&self) -> Option<StylesheetCstNode> {
        self.first_node(StylesheetCstNode::cast)
    }

    pub fn rules(&self) -> Vec<RuleCstNode> {
        self.nodes(RuleCstNode::cast)
    }

    pub fn selectors(&self) -> Vec<SelectorCstNode> {
        self.nodes(SelectorCstNode::cast)
    }

    pub fn declarations(&self) -> Vec<DeclarationCstNode> {
        self.nodes(DeclarationCstNode::cast)
    }

    pub fn declaration_lists(&self) -> Vec<DeclarationListCstNode> {
        self.nodes(DeclarationListCstNode::cast)
    }

    pub fn values(&self) -> Vec<ValueCstNode> {
        self.nodes(ValueCstNode::cast)
    }

    pub fn url_values(&self) -> Vec<UrlValueCstNode> {
        self.nodes(UrlValueCstNode::cast)
    }

    pub fn component_values(&self) -> Vec<ComponentValueCstNode> {
        self.nodes(ComponentValueCstNode::cast)
    }

    pub fn simple_blocks(&self) -> Vec<SimpleBlockCstNode> {
        self.nodes(SimpleBlockCstNode::cast)
    }

    pub fn component_value_lists(&self) -> Vec<ComponentValueListCstNode> {
        self.nodes(ComponentValueListCstNode::cast)
    }

    pub fn comma_separated_component_value_lists(
        &self,
    ) -> Vec<CommaSeparatedComponentValueListCstNode> {
        self.nodes(CommaSeparatedComponentValueListCstNode::cast)
    }

    pub fn custom_property_values(&self) -> Vec<CustomPropertyValueCstNode> {
        self.nodes(CustomPropertyValueCstNode::cast)
    }

    pub fn at_rules(&self) -> Vec<AtRuleCstNode> {
        self.nodes(AtRuleCstNode::cast)
    }

    pub fn bogus_nodes(&self) -> Vec<BogusCstNode> {
        self.nodes(BogusCstNode::cast)
    }

    pub fn has_bogus_nodes(&self) -> bool {
        self.first_node(BogusCstNode::cast).is_some()
    }

    fn first_node<T>(&self, cast: impl Fn(SyntaxNode<SyntaxKind>) -> Option<T>) -> Option<T> {
        let mut nodes = Vec::new();
        collect_typed_nodes(&self.root, &cast, &mut nodes);
        nodes.into_iter().next()
    }

    fn nodes<T>(&self, cast: impl Fn(SyntaxNode<SyntaxKind>) -> Option<T>) -> Vec<T> {
        let mut nodes = Vec::new();
        collect_typed_nodes(&self.root, &cast, &mut nodes);
        nodes
    }
}

pub trait TypedCstNode: Sized {
    fn cast(syntax: SyntaxNode<SyntaxKind>) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode<SyntaxKind>;

    fn kind(&self) -> SyntaxKind {
        self.syntax().kind()
    }

    fn text_range(&self) -> TextRange {
        self.syntax().text_range()
    }

    fn into_syntax(self) -> SyntaxNode<SyntaxKind>;
}

macro_rules! typed_cst_node {
    ($name:ident, $kind:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            syntax: SyntaxNode<SyntaxKind>,
        }

        impl $name {
            pub const KIND: SyntaxKind = $kind;
        }

        impl TypedCstNode for $name {
            fn cast(syntax: SyntaxNode<SyntaxKind>) -> Option<Self> {
                (syntax.kind() == Self::KIND).then_some(Self { syntax })
            }

            fn syntax(&self) -> &SyntaxNode<SyntaxKind> {
                &self.syntax
            }

            fn into_syntax(self) -> SyntaxNode<SyntaxKind> {
                self.syntax
            }
        }
    };
}

typed_cst_node!(StylesheetCstNode, SyntaxKind::Stylesheet);
typed_cst_node!(RuleCstNode, SyntaxKind::Rule);
typed_cst_node!(SelectorCstNode, SyntaxKind::Selector);
typed_cst_node!(DeclarationCstNode, SyntaxKind::Declaration);
typed_cst_node!(DeclarationListCstNode, SyntaxKind::DeclarationList);
typed_cst_node!(ValueCstNode, SyntaxKind::Value);
typed_cst_node!(UrlValueCstNode, SyntaxKind::UrlValue);
typed_cst_node!(ComponentValueCstNode, SyntaxKind::ComponentValue);
typed_cst_node!(SimpleBlockCstNode, SyntaxKind::SimpleBlock);
typed_cst_node!(ComponentValueListCstNode, SyntaxKind::ComponentValueList);
typed_cst_node!(
    CommaSeparatedComponentValueListCstNode,
    SyntaxKind::CommaSeparatedComponentValueList
);
typed_cst_node!(CustomPropertyValueCstNode, SyntaxKind::CustomPropertyValue);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtRuleCstNode {
    syntax: SyntaxNode<SyntaxKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BogusCstNode {
    syntax: SyntaxNode<SyntaxKind>,
}

impl TypedCstNode for AtRuleCstNode {
    fn cast(syntax: SyntaxNode<SyntaxKind>) -> Option<Self> {
        is_at_rule_node_kind(syntax.kind()).then_some(Self { syntax })
    }

    fn syntax(&self) -> &SyntaxNode<SyntaxKind> {
        &self.syntax
    }

    fn into_syntax(self) -> SyntaxNode<SyntaxKind> {
        self.syntax
    }
}

impl TypedCstNode for BogusCstNode {
    fn cast(syntax: SyntaxNode<SyntaxKind>) -> Option<Self> {
        syntax.kind().is_bogus().then_some(Self { syntax })
    }

    fn syntax(&self) -> &SyntaxNode<SyntaxKind> {
        &self.syntax
    }

    fn into_syntax(self) -> SyntaxNode<SyntaxKind> {
        self.syntax
    }
}

pub fn is_at_rule_node_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::AtRule
            | SyntaxKind::MediaRule
            | SyntaxKind::SupportsRule
            | SyntaxKind::ContainerRule
            | SyntaxKind::LayerRule
            | SyntaxKind::ScopeRule
            | SyntaxKind::KeyframesRule
            | SyntaxKind::FontFaceRule
            | SyntaxKind::PageRule
            | SyntaxKind::NamespaceRule
            | SyntaxKind::ImportRule
            | SyntaxKind::CharsetRule
            | SyntaxKind::PropertyRule
            | SyntaxKind::FunctionRule
            | SyntaxKind::StartingStyleRule
            | SyntaxKind::PageMarginRule
            | SyntaxKind::WhenRule
            | SyntaxKind::ElseRule
            | SyntaxKind::IfRule
            | SyntaxKind::CounterStyleRule
            | SyntaxKind::FontPaletteValuesRule
            | SyntaxKind::ColorProfileRule
            | SyntaxKind::PositionTryRule
            | SyntaxKind::FontFeatureValuesRule
            | SyntaxKind::FontFeatureValuesStylisticRule
            | SyntaxKind::FontFeatureValuesStylesetRule
            | SyntaxKind::FontFeatureValuesCharacterVariantRule
            | SyntaxKind::FontFeatureValuesSwashRule
            | SyntaxKind::FontFeatureValuesOrnamentsRule
            | SyntaxKind::FontFeatureValuesAnnotationRule
            | SyntaxKind::FontFeatureValuesHistoricalFormsRule
            | SyntaxKind::ViewTransitionRule
            | SyntaxKind::NestRule
            | SyntaxKind::CustomMediaRule
            | SyntaxKind::ScssUseRule
            | SyntaxKind::ScssForwardRule
            | SyntaxKind::ScssMixinDeclaration
            | SyntaxKind::ScssIncludeRule
            | SyntaxKind::ScssFunctionDeclaration
            | SyntaxKind::ScssReturnRule
            | SyntaxKind::ScssAtRootRule
            | SyntaxKind::ScssErrorRule
            | SyntaxKind::ScssWarnRule
            | SyntaxKind::ScssDebugRule
            | SyntaxKind::ScssContentRule
    )
}

fn collect_typed_nodes<T>(
    node: &SyntaxNode<SyntaxKind>,
    cast: &impl Fn(SyntaxNode<SyntaxKind>) -> Option<T>,
    nodes: &mut Vec<T>,
) {
    if let Some(typed) = cast(node.clone()) {
        nodes.push(typed);
    }
    for child in node.children() {
        collect_typed_nodes(child, cast, nodes);
    }
}

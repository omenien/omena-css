use omena_syntax::{StyleDialect, SyntaxKind};

pub trait DialectExtension {
    fn dialect(&self) -> StyleDialect;

    fn classify_variable_token(&self, text: &str) -> Option<SyntaxKind> {
        match self.dialect() {
            StyleDialect::Css => None,
            StyleDialect::Scss | StyleDialect::Sass if text.starts_with('$') => {
                Some(SyntaxKind::ScssVariable)
            }
            StyleDialect::Less if text.starts_with('@') => Some(SyntaxKind::LessVariable),
            StyleDialect::Scss | StyleDialect::Sass | StyleDialect::Less => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinDialectExtension {
    dialect: StyleDialect,
}

impl BuiltinDialectExtension {
    pub const fn new(dialect: StyleDialect) -> Self {
        Self { dialect }
    }
}

impl DialectExtension for BuiltinDialectExtension {
    fn dialect(&self) -> StyleDialect {
        self.dialect
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AtRuleSpec {
    pub(crate) node_kind: SyntaxKind,
    pub(crate) block_kind: AtRuleBlockKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AtRuleBlockKind {
    GroupRuleList,
    DeclarationList,
    Keyframes,
    Raw,
}

pub(crate) fn at_rule_spec(text: &str) -> Option<AtRuleSpec> {
    let lowered = text.to_ascii_lowercase();
    let (node_kind, block_kind) = match lowered.as_str() {
        "@media" => (SyntaxKind::MediaRule, AtRuleBlockKind::GroupRuleList),
        "@supports" => (SyntaxKind::SupportsRule, AtRuleBlockKind::GroupRuleList),
        "@when" => (SyntaxKind::WhenRule, AtRuleBlockKind::GroupRuleList),
        "@else" => (SyntaxKind::ElseRule, AtRuleBlockKind::GroupRuleList),
        "@container" => (SyntaxKind::ContainerRule, AtRuleBlockKind::GroupRuleList),
        "@layer" => (SyntaxKind::LayerRule, AtRuleBlockKind::GroupRuleList),
        "@scope" => (SyntaxKind::ScopeRule, AtRuleBlockKind::GroupRuleList),
        "@starting-style" => (
            SyntaxKind::StartingStyleRule,
            AtRuleBlockKind::GroupRuleList,
        ),
        "@nest" => (SyntaxKind::NestRule, AtRuleBlockKind::DeclarationList),
        "@keyframes" => (SyntaxKind::KeyframesRule, AtRuleBlockKind::Keyframes),
        "@font-face" => (SyntaxKind::FontFaceRule, AtRuleBlockKind::DeclarationList),
        "@page" => (SyntaxKind::PageRule, AtRuleBlockKind::DeclarationList),
        "@property" => (SyntaxKind::PropertyRule, AtRuleBlockKind::DeclarationList),
        "@counter-style" => (
            SyntaxKind::CounterStyleRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@font-palette-values" => (
            SyntaxKind::FontPaletteValuesRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@color-profile" => (
            SyntaxKind::ColorProfileRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@position-try" => (
            SyntaxKind::PositionTryRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@font-feature-values" => (
            SyntaxKind::FontFeatureValuesRule,
            AtRuleBlockKind::GroupRuleList,
        ),
        "@stylistic" => (
            SyntaxKind::FontFeatureValuesStylisticRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@styleset" => (
            SyntaxKind::FontFeatureValuesStylesetRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@character-variant" => (
            SyntaxKind::FontFeatureValuesCharacterVariantRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@swash" => (
            SyntaxKind::FontFeatureValuesSwashRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@ornaments" => (
            SyntaxKind::FontFeatureValuesOrnamentsRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@annotation" => (
            SyntaxKind::FontFeatureValuesAnnotationRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@historical-forms" => (
            SyntaxKind::FontFeatureValuesHistoricalFormsRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@view-transition" => (
            SyntaxKind::ViewTransitionRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@charset" => (SyntaxKind::CharsetRule, AtRuleBlockKind::Raw),
        "@import" => (SyntaxKind::ImportRule, AtRuleBlockKind::Raw),
        "@namespace" => (SyntaxKind::NamespaceRule, AtRuleBlockKind::Raw),
        "@custom-media" => (SyntaxKind::CustomMediaRule, AtRuleBlockKind::Raw),
        text if is_page_margin_at_rule(text) => {
            (SyntaxKind::PageMarginRule, AtRuleBlockKind::DeclarationList)
        }
        _ => return None,
    };
    Some(AtRuleSpec {
        node_kind,
        block_kind,
    })
}

pub(crate) fn scss_at_rule_spec(text: &str) -> Option<AtRuleSpec> {
    let (node_kind, block_kind) = match text {
        "@use" => (SyntaxKind::ScssUseRule, AtRuleBlockKind::Raw),
        "@forward" => (SyntaxKind::ScssForwardRule, AtRuleBlockKind::Raw),
        "@mixin" => (
            SyntaxKind::ScssMixinDeclaration,
            AtRuleBlockKind::DeclarationList,
        ),
        "@include" => (
            SyntaxKind::ScssIncludeRule,
            AtRuleBlockKind::DeclarationList,
        ),
        "@function" => (
            SyntaxKind::ScssFunctionDeclaration,
            AtRuleBlockKind::DeclarationList,
        ),
        "@return" => (SyntaxKind::ScssReturnRule, AtRuleBlockKind::Raw),
        "@extend" => (SyntaxKind::ScssExtendRule, AtRuleBlockKind::Raw),
        "@if" => (SyntaxKind::ScssControlIf, AtRuleBlockKind::DeclarationList),
        "@else" => (
            SyntaxKind::ScssControlElse,
            AtRuleBlockKind::DeclarationList,
        ),
        "@each" => (
            SyntaxKind::ScssControlEach,
            AtRuleBlockKind::DeclarationList,
        ),
        "@for" => (SyntaxKind::ScssControlFor, AtRuleBlockKind::DeclarationList),
        "@while" => (
            SyntaxKind::ScssControlWhile,
            AtRuleBlockKind::DeclarationList,
        ),
        "@at-root" => (SyntaxKind::ScssAtRootRule, AtRuleBlockKind::DeclarationList),
        "@error" => (SyntaxKind::ScssErrorRule, AtRuleBlockKind::Raw),
        "@warn" => (SyntaxKind::ScssWarnRule, AtRuleBlockKind::Raw),
        "@debug" => (SyntaxKind::ScssDebugRule, AtRuleBlockKind::Raw),
        "@content" => (SyntaxKind::ScssContentRule, AtRuleBlockKind::Raw),
        _ => return None,
    };
    Some(AtRuleSpec {
        node_kind,
        block_kind,
    })
}

fn is_page_margin_at_rule(text: &str) -> bool {
    matches!(
        text,
        "@top-left-corner"
            | "@top-left"
            | "@top-center"
            | "@top-right"
            | "@top-right-corner"
            | "@bottom-left-corner"
            | "@bottom-left"
            | "@bottom-center"
            | "@bottom-right"
            | "@bottom-right-corner"
            | "@left-top"
            | "@left-middle"
            | "@left-bottom"
            | "@right-top"
            | "@right-middle"
            | "@right-bottom"
    )
}

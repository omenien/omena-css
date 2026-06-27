//! Shared syntax vocabulary for the Omena CSS-family parser stack.
//!
//! This crate is intentionally substrate-only: it defines stable syntax kind
//! ranges and CST integration without parsing source text yet.
//!
//! syntax_kind_extraction_decision: keep `SyntaxKind` extracted in
//! `omena-syntax`; parser, semantic, resolver, LSP, and checker layers consume
//! this crate instead of re-declaring local node/token taxonomies.

use cstree::{RawSyntaxKind, Syntax};

pub const TOKEN_START: u32 = 0x0000;
pub const TOKEN_END: u32 = 0x03ff;
pub const DIALECT_TOKEN_START: u32 = 0x0400;
pub const DIALECT_TOKEN_END: u32 = 0x04ff;
pub const NODE_START: u32 = 0x1000;
pub const NODE_END: u32 = 0x13ff;
pub const DIALECT_NODE_START: u32 = 0x1400;
pub const DIALECT_NODE_END: u32 = 0x14ff;
pub const BOGUS_START: u32 = 0x2000;
pub const BOGUS_END: u32 = 0x20ff;
pub const MARKER_START: u32 = 0x2100;
pub const MARKER_END: u32 = 0x21ff;

const _: () = {
    assert!(TOKEN_END < DIALECT_TOKEN_START);
    assert!(DIALECT_TOKEN_END < NODE_START);
    assert!(NODE_END < DIALECT_NODE_START);
    assert!(DIALECT_NODE_END < BOGUS_START);
    assert!(BOGUS_END < MARKER_START);
};

pub type SyntaxNode<D = ()> = cstree::syntax::SyntaxNode<SyntaxKind, D>;
pub type SyntaxToken<D = ()> = cstree::syntax::SyntaxToken<SyntaxKind, D>;
pub type SyntaxElement<D = ()> = cstree::syntax::SyntaxElement<SyntaxKind, D>;
pub type SyntaxElementRef<'a, D = ()> = cstree::syntax::SyntaxElementRef<'a, SyntaxKind, D>;

macro_rules! syntax_kinds {
    ($($name:ident = $value:expr,)+) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u32)]
        pub enum SyntaxKind {
            $($name = $value,)+
        }

        impl SyntaxKind {
            pub const ALL: &'static [Self] = &[$(Self::$name,)+];

            pub const fn as_u32(self) -> u32 {
                self as u32
            }

            pub fn from_raw_kind(raw: u32) -> Option<Self> {
                let mut index = 0;
                while index < Self::ALL.len() {
                    let kind = Self::ALL[index];
                    if kind.as_u32() == raw {
                        return Some(kind);
                    }
                    index += 1;
                }
                None
            }

            pub const fn is_token(self) -> bool {
                let raw = self.as_u32();
                (raw >= TOKEN_START && raw <= TOKEN_END)
                    || (raw >= DIALECT_TOKEN_START && raw <= DIALECT_TOKEN_END)
            }

            pub const fn is_node(self) -> bool {
                let raw = self.as_u32();
                (raw >= NODE_START && raw <= NODE_END)
                    || (raw >= DIALECT_NODE_START && raw <= DIALECT_NODE_END)
            }

            pub const fn is_bogus(self) -> bool {
                let raw = self.as_u32();
                raw >= BOGUS_START && raw <= BOGUS_END
            }

            pub const fn is_marker(self) -> bool {
                let raw = self.as_u32();
                raw >= MARKER_START && raw <= MARKER_END
            }

            pub const fn is_dialect_specific(self) -> bool {
                let raw = self.as_u32();
                (raw >= DIALECT_TOKEN_START && raw <= DIALECT_TOKEN_END)
                    || (raw >= DIALECT_NODE_START && raw <= DIALECT_NODE_END)
            }

            pub const fn is_dialect(self) -> bool {
                self.is_dialect_specific()
            }

            pub const fn is_trivia(self) -> bool {
                matches!(
                    self,
                    Self::Whitespace
                        | Self::LineComment
                        | Self::BlockComment
                        | Self::SassIndentedNewline
                )
            }
        }
    };
}

syntax_kinds! {
    Whitespace = 0x0000,
    LineComment = 0x0001,
    BlockComment = 0x0002,
    Ident = 0x0003,
    Hash = 0x0004,
    String = 0x0005,
    BadString = 0x0006,
    Url = 0x0007,
    BadUrl = 0x0008,
    Number = 0x0009,
    Percentage = 0x000a,
    Dimension = 0x000b,
    UnicodeRange = 0x000c,
    AtKeyword = 0x000d,
    Delim = 0x000e,
    Important = 0x000f,
    Dot = 0x0010,
    Comma = 0x0011,
    Colon = 0x0012,
    Semicolon = 0x0013,
    LeftBrace = 0x0014,
    RightBrace = 0x0015,
    LeftParen = 0x0016,
    RightParen = 0x0017,
    LeftBracket = 0x0018,
    RightBracket = 0x0019,
    Plus = 0x001a,
    Minus = 0x001b,
    Star = 0x001c,
    Slash = 0x001d,
    Percent = 0x001e,
    Equals = 0x001f,
    Tilde = 0x0020,
    Pipe = 0x0021,
    Caret = 0x0022,
    Dollar = 0x0023,
    Ampersand = 0x0024,
    GreaterThan = 0x0025,
    LessThan = 0x0026,
    PlusEquals = 0x0027,
    MinusEquals = 0x0028,
    StarEquals = 0x0029,
    SlashEquals = 0x002a,
    PipeEquals = 0x002b,
    TildeEquals = 0x002c,
    CaretEquals = 0x002d,
    DollarEquals = 0x002e,
    DoubleColon = 0x002f,
    DoublePipe = 0x0030,
    DoubleAmpersand = 0x0031,
    Arrow = 0x0032,
    IncludesMatch = 0x0033,
    DashMatch = 0x0034,
    PrefixMatch = 0x0035,
    SuffixMatch = 0x0036,
    SubstringMatch = 0x0037,
    ColumnCombinator = 0x0038,
    NestingSelector = 0x0039,
    CustomPropertyName = 0x003a,
    ClassName = 0x003b,
    IdName = 0x003c,
    KeywordAnd = 0x003d,
    KeywordOr = 0x003e,
    KeywordNot = 0x003f,
    KeywordOnly = 0x0040,
    KeywordFrom = 0x0041,
    KeywordTo = 0x0042,
    KeywordThrough = 0x0043,
    KeywordImportant = 0x0044,
    KeywordGlobal = 0x0045,
    KeywordLocal = 0x0046,
    KeywordExport = 0x0047,
    KeywordImport = 0x0048,
    KeywordComposes = 0x0049,
    KeywordAs = 0x004a,
    KeywordWith = 0x004b,
    KeywordLayer = 0x004c,
    KeywordSupports = 0x004d,
    KeywordContainer = 0x004e,
    KeywordScope = 0x004f,
    KeywordMedia = 0x0050,
    KeywordKeyframes = 0x0051,
    KeywordCharset = 0x0052,
    KeywordNamespace = 0x0053,
    KeywordPage = 0x0054,
    KeywordFontFace = 0x0055,
    KeywordProperty = 0x0056,
    KeywordStartingStyle = 0x0057,
    KeywordWhen = 0x0058,
    KeywordElse = 0x0059,
    KeywordUse = 0x005a,
    KeywordForward = 0x005b,
    KeywordMixin = 0x005c,
    KeywordInclude = 0x005d,
    KeywordFunction = 0x005e,
    KeywordReturn = 0x005f,
    KeywordIf = 0x0060,
    KeywordEach = 0x0061,
    KeywordFor = 0x0062,
    KeywordWhile = 0x0063,
    KeywordIn = 0x0064,
    Cdo = 0x0065,
    Cdc = 0x0066,

    ScssVariable = 0x0400,
    ScssInterpolationStart = 0x0401,
    ScssInterpolationEnd = 0x0402,
    ScssSilentComment = 0x0403,
    ScssPlaceholder = 0x0404,
    ScssModuleNamespace = 0x0405,
    SassIndentedNewline = 0x0406,
    SassIndent = 0x0407,
    SassDedent = 0x0408,
    SassOptionalSemicolon = 0x0409,
    LessVariable = 0x0410,
    LessEscapedString = 0x0411,
    LessDetachedRuleset = 0x0412,
    LessMixinGuardWhen = 0x0413,
    LessExtendKeyword = 0x0414,
    LessNamespaceSeparator = 0x0415,
    LessInterpolationStart = 0x0416,
    LessInterpolationEnd = 0x0417,
    LessPropertyVariableToken = 0x0418,
    TemplateInterpolationStart = 0x0419,
    TemplateInterpolationEnd = 0x041a,
    TemplatePlaceholder = 0x041b,

    Stylesheet = 0x1000,
    Rule = 0x1001,
    QualifiedRule = 0x1002,
    Declaration = 0x1003,
    DeclarationList = 0x1004,
    RuleList = 0x1005,
    SelectorList = 0x1006,
    Selector = 0x1007,
    ComplexSelector = 0x1008,
    CompoundSelector = 0x1009,
    ClassSelector = 0x100a,
    IdSelector = 0x100b,
    TypeSelector = 0x100c,
    UniversalSelector = 0x100d,
    AttributeSelector = 0x100e,
    AttributeMatcher = 0x100f,
    PseudoClassSelector = 0x1010,
    PseudoElementSelector = 0x1011,
    PseudoSelectorArgument = 0x1012,
    NestingSelectorNode = 0x1013,
    Combinator = 0x1014,
    SelectorValue = 0x1015,
    PropertyName = 0x1016,
    CustomPropertyDeclaration = 0x1017,
    Value = 0x1018,
    ValueList = 0x1019,
    FunctionCall = 0x101a,
    FunctionArguments = 0x101b,
    BinaryExpression = 0x101c,
    UnaryExpression = 0x101d,
    ParenthesizedExpression = 0x101e,
    Interpolation = 0x101f,
    DimensionValue = 0x1020,
    ColorValue = 0x1021,
    UrlValue = 0x1022,
    VarFunction = 0x1023,
    CalcFunction = 0x1024,
    AtRule = 0x1025,
    MediaRule = 0x1026,
    SupportsRule = 0x1027,
    ContainerRule = 0x1028,
    LayerRule = 0x1029,
    ScopeRule = 0x102a,
    KeyframesRule = 0x102b,
    KeyframeBlock = 0x102c,
    FontFaceRule = 0x102d,
    PageRule = 0x102e,
    NamespaceRule = 0x102f,
    ImportRule = 0x1030,
    CharsetRule = 0x1031,
    PropertyRule = 0x1032,
    StartingStyleRule = 0x1033,
    MediaQueryList = 0x1034,
    MediaQuery = 0x1035,
    MediaFeature = 0x1036,
    SupportsCondition = 0x1037,
    ContainerCondition = 0x1038,
    LayerName = 0x1039,
    ScopeRange = 0x103a,
    CssModuleLocalBlock = 0x103b,
    CssModuleGlobalBlock = 0x103c,
    CssModuleExportBlock = 0x103d,
    CssModuleImportBlock = 0x103e,
    CssModuleComposesDeclaration = 0x103f,
    CssModuleComposesTarget = 0x1040,
    CssModuleFromClause = 0x1041,
    TokenDefinition = 0x1042,
    TokenReference = 0x1043,
    Comment = 0x1044,
    ErrorNode = 0x1045,
    EnvFunction = 0x1046,
    AttrFunction = 0x1047,
    MathFunction = 0x1048,
    PageMarginRule = 0x1049,
    WhenRule = 0x104a,
    ElseRule = 0x104b,
    CounterStyleRule = 0x104c,
    FontPaletteValuesRule = 0x104d,
    ColorProfileRule = 0x104e,
    PositionTryRule = 0x104f,
    FontFeatureValuesRule = 0x1050,
    FontFeatureValuesStylisticRule = 0x1051,
    FontFeatureValuesStylesetRule = 0x1052,
    FontFeatureValuesCharacterVariantRule = 0x1053,
    FontFeatureValuesSwashRule = 0x1054,
    FontFeatureValuesOrnamentsRule = 0x1055,
    FontFeatureValuesAnnotationRule = 0x1056,
    FontFeatureValuesHistoricalFormsRule = 0x1057,
    ViewTransitionRule = 0x1058,
    GradientFunction = 0x1059,
    TransformFunction = 0x105a,
    FilterFunction = 0x105b,
    ImageFunction = 0x105c,
    ShapeFunction = 0x105d,
    AtRulePrelude = 0x105e,
    NestRule = 0x105f,
    CustomMediaRule = 0x1060,
    IdentifierValue = 0x1061,
    StringValue = 0x1062,
    UnicodeRangeValue = 0x1063,
    NumberValue = 0x1064,
    PercentageValue = 0x1065,
    BracketedValue = 0x1066,
    ImportantAnnotation = 0x1067,
    ComponentValue = 0x1068,
    SimpleBlock = 0x1069,
    ComponentValueList = 0x106a,
    CommaSeparatedComponentValueList = 0x106b,
    CustomPropertyValue = 0x106c,
    AttributeName = 0x106d,
    AttributeValue = 0x106e,
    AttributeModifier = 0x106f,
    NthSelectorArgument = 0x1070,
    NthSelectorFormula = 0x1071,
    NthSelectorOfSelectorList = 0x1072,
    RelativeSelectorList = 0x1073,
    RelativeSelector = 0x1074,
    LanguageSelectorArgument = 0x1075,
    LanguageTag = 0x1076,
    DirectionalitySelectorArgument = 0x1077,
    NamespacePrefix = 0x1078,
    FunctionRule = 0x1079,
    IfFunction = 0x107a,

    ScssStylesheet = 0x1400,
    ScssUseRule = 0x1401,
    ScssForwardRule = 0x1402,
    ScssMixinDeclaration = 0x1403,
    ScssIncludeRule = 0x1404,
    ScssFunctionDeclaration = 0x1405,
    ScssReturnRule = 0x1406,
    ScssVariableDeclaration = 0x1407,
    ScssVariableReference = 0x1408,
    ScssPlaceholderSelector = 0x1409,
    ScssExtendRule = 0x140a,
    ScssControlIf = 0x140b,
    ScssControlElse = 0x140c,
    ScssControlEach = 0x140d,
    ScssControlFor = 0x140e,
    ScssControlWhile = 0x140f,
    ScssNestedProperty = 0x1410,
    ScssModuleConfig = 0x1411,
    SassIndentedBlock = 0x1412,
    SassIndentedRule = 0x1413,
    ScssAtRootRule = 0x1414,
    ScssErrorRule = 0x1415,
    ScssWarnRule = 0x1416,
    ScssDebugRule = 0x1417,
    ScssContentRule = 0x1418,
    ScssVariableFlag = 0x1419,
    LessStylesheet = 0x1420,
    LessVariableDeclaration = 0x1421,
    LessVariableReference = 0x1422,
    LessMixinDeclaration = 0x1423,
    LessMixinCall = 0x1424,
    LessMixinGuard = 0x1425,
    LessDetachedRulesetNode = 0x1426,
    LessExtendRule = 0x1427,
    LessNamespaceAccess = 0x1428,
    LessPropertyVariable = 0x1429,
    ScssMap = 0x142a,
    ScssMapEntry = 0x142b,
    ScssList = 0x142c,
    ScssCondition = 0x142d,
    LessCondition = 0x142e,

    BogusToken = 0x2000,
    BogusTrivia = 0x2001,
    BogusRule = 0x2002,
    BogusSelector = 0x2003,
    BogusSelectorList = 0x2004,
    BogusCompoundSelector = 0x2005,
    BogusCombinator = 0x2006,
    BogusDeclaration = 0x2007,
    BogusDeclarationList = 0x2008,
    BogusPropertyName = 0x2009,
    BogusValue = 0x200a,
    BogusValueList = 0x200b,
    BogusFunctionCall = 0x200c,
    BogusFunctionArguments = 0x200d,
    BogusAtRule = 0x200e,
    BogusMediaQuery = 0x200f,
    BogusSupportsCondition = 0x2010,
    BogusContainerCondition = 0x2011,
    BogusLayerName = 0x2012,
    BogusScopeRange = 0x2013,
    BogusKeyframeBlock = 0x2014,
    BogusCssModuleBlock = 0x2015,
    BogusComposesDeclaration = 0x2016,
    BogusComposesTarget = 0x2017,
    BogusFromClause = 0x2018,
    BogusInterpolation = 0x2019,
    BogusScssVariable = 0x201a,
    BogusScssMixin = 0x201b,
    BogusScssFunction = 0x201c,
    BogusScssControl = 0x201d,
    BogusSassIndentation = 0x201e,
    BogusLessVariable = 0x201f,
    BogusLessMixin = 0x2020,
    BogusLessGuard = 0x2021,
    BogusLessDetachedRuleset = 0x2022,
    BogusRecovery = 0x2023,
    BogusScssModuleConfig = 0x2024,
    BogusAtRulePrelude = 0x2025,
    BogusBracketedValue = 0x2026,
    BogusSimpleBlock = 0x2027,
    BogusScssMap = 0x2028,
    BogusScssMapEntry = 0x2029,
    BogusScssList = 0x202a,
    BogusScssCondition = 0x202b,
    BogusLessCondition = 0x202c,

    Root = 0x2100,
    Eof = 0x2101,
    Unknown = 0x21fe,
    Tombstone = 0x21ff,
}

impl Syntax for SyntaxKind {
    fn from_raw(raw: RawSyntaxKind) -> Self {
        match Self::from_raw_kind(raw.0) {
            Some(kind) => kind,
            None => Self::Unknown,
        }
    }

    fn into_raw(self) -> RawSyntaxKind {
        RawSyntaxKind(self.as_u32())
    }

    fn static_text(self) -> Option<&'static str> {
        match self {
            Self::Dot => Some("."),
            Self::Comma => Some(","),
            Self::Colon => Some(":"),
            Self::Semicolon => Some(";"),
            Self::LeftBrace => Some("{"),
            Self::RightBrace => Some("}"),
            Self::LeftParen => Some("("),
            Self::RightParen => Some(")"),
            Self::LeftBracket => Some("["),
            Self::RightBracket => Some("]"),
            Self::Plus => Some("+"),
            Self::Minus => Some("-"),
            Self::Star => Some("*"),
            Self::Slash => Some("/"),
            Self::Percent => Some("%"),
            Self::Equals => Some("="),
            Self::Tilde => Some("~"),
            Self::Pipe => Some("|"),
            Self::Caret => Some("^"),
            Self::Dollar => Some("$"),
            Self::Ampersand => Some("&"),
            Self::GreaterThan => Some(">"),
            Self::LessThan => Some("<"),
            Self::PlusEquals => Some("+="),
            Self::MinusEquals => Some("-="),
            Self::StarEquals => Some("*="),
            Self::SlashEquals => Some("/="),
            Self::PipeEquals => Some("|="),
            Self::TildeEquals => Some("~="),
            Self::CaretEquals => Some("^="),
            Self::DollarEquals => Some("$="),
            Self::DoubleColon => Some("::"),
            Self::DoublePipe => Some("||"),
            Self::DoubleAmpersand => Some("&&"),
            Self::Arrow => Some("=>"),
            Self::IncludesMatch => Some("~="),
            Self::DashMatch => Some("|="),
            Self::PrefixMatch => Some("^="),
            Self::SuffixMatch => Some("$="),
            Self::SubstringMatch => Some("*="),
            Self::ColumnCombinator => Some("||"),
            Self::KeywordAnd => Some("and"),
            Self::KeywordOr => Some("or"),
            Self::KeywordNot => Some("not"),
            Self::KeywordOnly => Some("only"),
            Self::KeywordFrom => Some("from"),
            Self::KeywordTo => Some("to"),
            Self::KeywordThrough => Some("through"),
            Self::KeywordImportant => Some("important"),
            Self::Cdo => Some("<!--"),
            Self::Cdc => Some("-->"),
            Self::KeywordGlobal => Some("global"),
            Self::KeywordLocal => Some("local"),
            Self::KeywordExport => Some("export"),
            Self::KeywordImport => Some("import"),
            Self::KeywordComposes => Some("composes"),
            Self::KeywordAs => Some("as"),
            Self::KeywordWith => Some("with"),
            Self::KeywordLayer => Some("layer"),
            Self::KeywordSupports => Some("supports"),
            Self::KeywordContainer => Some("container"),
            Self::KeywordScope => Some("scope"),
            Self::KeywordMedia => Some("media"),
            Self::KeywordKeyframes => Some("keyframes"),
            Self::KeywordCharset => Some("charset"),
            Self::KeywordNamespace => Some("namespace"),
            Self::KeywordPage => Some("page"),
            Self::KeywordFontFace => Some("font-face"),
            Self::KeywordProperty => Some("property"),
            Self::KeywordStartingStyle => Some("starting-style"),
            Self::KeywordWhen => Some("when"),
            Self::KeywordElse => Some("else"),
            Self::KeywordUse => Some("use"),
            Self::KeywordForward => Some("forward"),
            Self::KeywordMixin => Some("mixin"),
            Self::KeywordInclude => Some("include"),
            Self::KeywordFunction => Some("function"),
            Self::KeywordReturn => Some("return"),
            Self::KeywordIf => Some("if"),
            Self::KeywordEach => Some("each"),
            Self::KeywordFor => Some("for"),
            Self::KeywordWhile => Some("while"),
            Self::KeywordIn => Some("in"),
            Self::Eof => Some(""),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StyleDialect {
    Css,
    Scss,
    Sass,
    Less,
}

impl StyleDialect {
    pub const ALL: &'static [Self] = &[Self::Css, Self::Scss, Self::Sass, Self::Less];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleMode {
    Plain,
    CssModules,
}

impl ModuleMode {
    pub const ALL: &'static [Self] = &[Self::Plain, Self::CssModules];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolKind {
    Class,
    Id,
    TypeSelector,
    PlaceholderSelector,
    Keyframes,
    CustomProperty,
    ScssVariable,
    LessVariable,
    Mixin,
    Function,
    ValueDeclaration,
    ComposesTarget,
    Namespace,
    Layer,
    Container,
    Scope,
    Import,
    Export,
    ModuleLocal,
    ModuleGlobal,
}

impl SymbolKind {
    pub const ALL: &'static [Self] = &[
        Self::Class,
        Self::Id,
        Self::TypeSelector,
        Self::PlaceholderSelector,
        Self::Keyframes,
        Self::CustomProperty,
        Self::ScssVariable,
        Self::LessVariable,
        Self::Mixin,
        Self::Function,
        Self::ValueDeclaration,
        Self::ComposesTarget,
        Self::Namespace,
        Self::Layer,
        Self::Container,
        Self::Scope,
        Self::Import,
        Self::Export,
        Self::ModuleLocal,
        Self::ModuleGlobal,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScopeKind {
    File,
    LocalBlock,
    GlobalBlock,
    SelectorBlock,
    MixinBody,
    FunctionBody,
    AtRuleScope,
    NestedRule,
    ScopeAtRule,
    MediaQuery,
    SupportsQuery,
    ContainerQuery,
    CascadeLayer,
    ModuleNamespace,
    LessMixin,
    SassControlFlow,
    CssModuleExport,
    CssModuleImport,
}

impl ScopeKind {
    pub const ALL: &'static [Self] = &[
        Self::File,
        Self::LocalBlock,
        Self::GlobalBlock,
        Self::SelectorBlock,
        Self::MixinBody,
        Self::FunctionBody,
        Self::AtRuleScope,
        Self::NestedRule,
        Self::ScopeAtRule,
        Self::MediaQuery,
        Self::SupportsQuery,
        Self::ContainerQuery,
        Self::CascadeLayer,
        Self::ModuleNamespace,
        Self::LessMixin,
        Self::SassControlFlow,
        Self::CssModuleExport,
        Self::CssModuleImport,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReferenceKind {
    Class,
    Id,
    TypeSelector,
    PlaceholderSelector,
    Keyframes,
    ComposesTarget,
    ComposesFrom,
    CustomPropertyRead,
    VarRead,
    ValueRead,
    Import,
    Export,
    MixinInclude,
    FunctionCall,
    NamespaceMember,
    Layer,
    Container,
    SelectorExtends,
    CssModuleAccess,
    CssModuleToken,
}

impl ReferenceKind {
    pub const ALL: &'static [Self] = &[
        Self::Class,
        Self::Id,
        Self::TypeSelector,
        Self::PlaceholderSelector,
        Self::Keyframes,
        Self::ComposesTarget,
        Self::ComposesFrom,
        Self::CustomPropertyRead,
        Self::VarRead,
        Self::ValueRead,
        Self::Import,
        Self::Export,
        Self::MixinInclude,
        Self::FunctionCall,
        Self::NamespaceMember,
        Self::Layer,
        Self::Container,
        Self::SelectorExtends,
        Self::CssModuleAccess,
        Self::CssModuleToken,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaSyntaxBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub phase: &'static str,
    pub syntax_kind_owner_crate: &'static str,
    pub parser_consumer_policy: &'static str,
    pub syntax_kind_count: usize,
    pub token_kind_count: usize,
    pub node_kind_count: usize,
    pub bogus_kind_count: usize,
    pub marker_kind_count: usize,
    pub dialect_kind_count: usize,
    pub style_dialect_count: usize,
    pub module_mode_count: usize,
    pub symbol_kind_count: usize,
    pub scope_kind_count: usize,
    pub reference_kind_count: usize,
    pub cstree_integration_ready: bool,
    pub ready_surfaces: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

pub fn summarize_omena_syntax_boundary() -> OmenaSyntaxBoundarySummaryV0 {
    OmenaSyntaxBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-syntax.boundary",
        phase: "h1-alpha-syntax-substrate",
        syntax_kind_owner_crate: "omena-syntax",
        parser_consumer_policy: "parserConsumesOmenaSyntaxKindNoLocalTaxonomy",
        syntax_kind_count: SyntaxKind::ALL.len(),
        token_kind_count: SyntaxKind::ALL
            .iter()
            .filter(|kind| kind.is_token())
            .count(),
        node_kind_count: SyntaxKind::ALL.iter().filter(|kind| kind.is_node()).count(),
        bogus_kind_count: SyntaxKind::ALL
            .iter()
            .filter(|kind| kind.is_bogus())
            .count(),
        marker_kind_count: SyntaxKind::ALL
            .iter()
            .filter(|kind| kind.is_marker())
            .count(),
        dialect_kind_count: SyntaxKind::ALL
            .iter()
            .filter(|kind| kind.is_dialect())
            .count(),
        style_dialect_count: StyleDialect::ALL.len(),
        module_mode_count: ModuleMode::ALL.len(),
        symbol_kind_count: SymbolKind::ALL.len(),
        scope_kind_count: ScopeKind::ALL.len(),
        reference_kind_count: ReferenceKind::ALL.len(),
        cstree_integration_ready: SyntaxKind::Ident.into_raw()
            == RawSyntaxKind(SyntaxKind::Ident.as_u32())
            && SyntaxKind::from_raw(RawSyntaxKind(SyntaxKind::Ident.as_u32())) == SyntaxKind::Ident,
        ready_surfaces: vec![
            "rangeDividedSyntaxKind",
            "symbolScopeReferenceVocabulary",
            "styleDialectAndModuleMode",
            "cstreeRawKindBridge",
            "bogusRecoveryKindSuperset",
            "semanticSoaTables",
            "parserCstEquivalence",
        ],
        next_surfaces: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_kind_ranges_are_disjoint() {
        let mut raws: Vec<u32> = SyntaxKind::ALL.iter().map(|kind| kind.as_u32()).collect();
        raws.sort_unstable();

        for pair in raws.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }

    #[test]
    fn classifies_token_node_bogus_marker_ranges() {
        assert!(SyntaxKind::Ident.is_token());
        assert!(SyntaxKind::ScssVariable.is_token());
        assert!(SyntaxKind::Selector.is_node());
        assert!(SyntaxKind::LessMixinCall.is_node());
        assert!(SyntaxKind::BogusSelector.is_bogus());
        assert!(SyntaxKind::Root.is_marker());
        assert!(SyntaxKind::Whitespace.is_trivia());
        assert!(SyntaxKind::ScssUseRule.is_dialect_specific());
    }

    #[test]
    fn declares_four_style_dialects_and_module_modes() {
        assert_eq!(StyleDialect::ALL.len(), 4);
        assert_eq!(ModuleMode::ALL.len(), 2);
    }

    #[test]
    fn cstree_round_trip_preserves_known_kinds() {
        for kind in [
            SyntaxKind::Ident,
            SyntaxKind::Selector,
            SyntaxKind::ScssUseRule,
            SyntaxKind::BogusLessGuard,
            SyntaxKind::Root,
        ] {
            let raw = kind.into_raw();
            assert_eq!(SyntaxKind::from_raw(raw), kind);
        }
    }

    #[test]
    fn declares_bogus_superset_contract() {
        let bogus_count = SyntaxKind::ALL
            .iter()
            .filter(|kind| kind.is_bogus())
            .count();

        assert!(bogus_count >= 33);
    }

    #[test]
    fn syntax_kind_count_tracks_phase_alpha_contract() {
        let token_count = SyntaxKind::ALL
            .iter()
            .filter(|kind| kind.is_token())
            .count();
        let node_count = SyntaxKind::ALL.iter().filter(|kind| kind.is_node()).count();

        assert!(SyntaxKind::ALL.len() >= 160);
        assert!(token_count >= 80);
        assert!(node_count >= 80);
    }

    #[test]
    fn summarizes_phase_alpha_boundary_contract() {
        let summary = summarize_omena_syntax_boundary();

        assert_eq!(summary.product, "omena-syntax.boundary");
        assert_eq!(summary.phase, "h1-alpha-syntax-substrate");
        assert_eq!(summary.syntax_kind_owner_crate, "omena-syntax");
        assert_eq!(
            summary.parser_consumer_policy,
            "parserConsumesOmenaSyntaxKindNoLocalTaxonomy"
        );
        assert!(summary.syntax_kind_count >= 160);
        assert!(summary.bogus_kind_count >= 33);
        assert_eq!(summary.style_dialect_count, 4);
        assert_eq!(summary.module_mode_count, 2);
        assert_eq!(summary.symbol_kind_count, SymbolKind::ALL.len());
        assert_eq!(summary.scope_kind_count, ScopeKind::ALL.len());
        assert_eq!(summary.reference_kind_count, ReferenceKind::ALL.len());
        assert!(summary.cstree_integration_ready);
        assert!(SyntaxKind::ScssUseRule.is_dialect());
        assert!(
            summary
                .ready_surfaces
                .contains(&"symbolScopeReferenceVocabulary")
        );
        assert!(summary.ready_surfaces.contains(&"semanticSoaTables"));
        assert!(summary.ready_surfaces.contains(&"parserCstEquivalence"));
        assert!(!summary.next_surfaces.contains(&"semanticSoaTables"));
        assert!(!summary.next_surfaces.contains(&"parserCstEquivalence"));
    }
}

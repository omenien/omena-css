//! Workspace-level interned name identities for omena-css.
//!
//! `cstree` owns token storage inside green trees. This crate owns semantic
//! string identity above the CST layer so hot-path equality can use typed
//! interned IDs instead of repeated string comparison.

use std::{error::Error, fmt};

use omena_syntax::SymbolKind;
use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NameKind {
    ClassName,
    CssIdent,
    PropertyName,
    SelectorKey,
    CustomPropertyName,
    KeyframesName,
    MixinName,
    FilePath,
}

impl NameKind {
    pub const ALL: &'static [Self] = &[
        Self::ClassName,
        Self::CssIdent,
        Self::PropertyName,
        Self::SelectorKey,
        Self::CustomPropertyName,
        Self::KeyframesName,
        Self::MixinName,
        Self::FilePath,
    ];

    pub const fn canonical_symbol_kind(self) -> Option<SymbolKind> {
        match self {
            Self::ClassName => Some(SymbolKind::Class),
            Self::CustomPropertyName => Some(SymbolKind::CustomProperty),
            Self::KeyframesName => Some(SymbolKind::Keyframes),
            Self::MixinName => Some(SymbolKind::Mixin),
            Self::CssIdent | Self::PropertyName | Self::SelectorKey | Self::FilePath => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternError {
    EmptyText { kind: NameKind },
}

impl fmt::Display for InternError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyText { kind } => {
                write!(formatter, "cannot intern empty text for {kind:?}")
            }
        }
    }
}

impl Error for InternError {}

#[salsa::interned(debug)]
pub struct ClassName<'db> {
    #[returns(ref)]
    pub text: SmolStr,
}

#[salsa::interned(debug)]
pub struct CssIdent<'db> {
    #[returns(ref)]
    pub text: SmolStr,
}

#[salsa::interned(debug)]
pub struct PropertyName<'db> {
    #[returns(ref)]
    pub text: SmolStr,
}

#[salsa::interned(debug)]
pub struct SelectorKey<'db> {
    #[returns(ref)]
    pub text: SmolStr,
}

#[salsa::interned(debug)]
pub struct CustomPropertyName<'db> {
    #[returns(ref)]
    pub text: SmolStr,
}

#[salsa::interned(debug)]
pub struct KeyframesName<'db> {
    #[returns(ref)]
    pub text: SmolStr,
}

#[salsa::interned(debug)]
pub struct MixinName<'db> {
    #[returns(ref)]
    pub text: SmolStr,
}

#[salsa::interned(debug)]
pub struct FilePath<'db> {
    #[returns(ref)]
    pub text: SmolStr,
}

pub fn intern_class_name<'db>(
    db: &'db dyn salsa::Database,
    text: impl Into<SmolStr>,
) -> Result<ClassName<'db>, InternError> {
    checked_text(NameKind::ClassName, text).map(|text| ClassName::new(db, text))
}

pub fn intern_css_ident<'db>(
    db: &'db dyn salsa::Database,
    text: impl Into<SmolStr>,
) -> Result<CssIdent<'db>, InternError> {
    checked_text(NameKind::CssIdent, text).map(|text| CssIdent::new(db, text))
}

pub fn intern_property_name<'db>(
    db: &'db dyn salsa::Database,
    text: impl Into<SmolStr>,
) -> Result<PropertyName<'db>, InternError> {
    checked_text(NameKind::PropertyName, text).map(|text| PropertyName::new(db, text))
}

pub fn intern_selector_key<'db>(
    db: &'db dyn salsa::Database,
    text: impl Into<SmolStr>,
) -> Result<SelectorKey<'db>, InternError> {
    checked_text(NameKind::SelectorKey, text).map(|text| SelectorKey::new(db, text))
}

pub fn intern_custom_property_name<'db>(
    db: &'db dyn salsa::Database,
    text: impl Into<SmolStr>,
) -> Result<CustomPropertyName<'db>, InternError> {
    checked_text(NameKind::CustomPropertyName, text).map(|text| CustomPropertyName::new(db, text))
}

pub fn intern_keyframes_name<'db>(
    db: &'db dyn salsa::Database,
    text: impl Into<SmolStr>,
) -> Result<KeyframesName<'db>, InternError> {
    checked_text(NameKind::KeyframesName, text).map(|text| KeyframesName::new(db, text))
}

pub fn intern_mixin_name<'db>(
    db: &'db dyn salsa::Database,
    text: impl Into<SmolStr>,
) -> Result<MixinName<'db>, InternError> {
    checked_text(NameKind::MixinName, text).map(|text| MixinName::new(db, text))
}

pub fn intern_file_path<'db>(
    db: &'db dyn salsa::Database,
    text: impl Into<SmolStr>,
) -> Result<FilePath<'db>, InternError> {
    checked_text(NameKind::FilePath, text).map(|text| FilePath::new(db, text))
}

fn checked_text(kind: NameKind, text: impl Into<SmolStr>) -> Result<SmolStr, InternError> {
    let text = text.into();
    if text.is_empty() {
        Err(InternError::EmptyText { kind })
    } else {
        Ok(text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaInternerBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub phase: &'static str,
    pub name_kind_count: usize,
    pub salsa_interned_type_count: usize,
    pub validated_helper_count: usize,
    pub symbol_mapped_name_kind_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

pub fn summarize_omena_interner_boundary() -> OmenaInternerBoundarySummaryV0 {
    OmenaInternerBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-interner.boundary",
        phase: "h1-beta-name-identity-substrate",
        name_kind_count: NameKind::ALL.len(),
        salsa_interned_type_count: 8,
        validated_helper_count: 8,
        symbol_mapped_name_kind_count: NameKind::ALL
            .iter()
            .filter(|kind| kind.canonical_symbol_kind().is_some())
            .count(),
        ready_surfaces: vec![
            "typedSalsaInternedNames",
            "validatedNameHelpers",
            "syntaxSymbolKindMapping",
            "workspaceFilePathIdentity",
            "parserSemanticNameConsumption",
            "semanticSoaNameTables",
        ],
        next_surfaces: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declares_eight_workspace_name_kinds() {
        assert_eq!(NameKind::ALL.len(), 8);
        assert_eq!(
            NameKind::CustomPropertyName.canonical_symbol_kind(),
            Some(SymbolKind::CustomProperty),
        );
        assert_eq!(NameKind::FilePath.canonical_symbol_kind(), None);
    }

    #[test]
    fn interns_equal_text_to_equal_typed_ids() {
        let db = salsa::DatabaseImpl::default();

        let first = intern_class_name(&db, "button");
        let second = intern_class_name(&db, SmolStr::new("button"));

        assert!(matches!((&first, &second), (Ok(_), Ok(_))));
        if let (Ok(first), Ok(second)) = (first, second) {
            assert_eq!(first, second);
            assert_eq!(first.text(&db).as_str(), "button");
        }
    }

    #[test]
    fn keeps_name_kinds_type_separated() {
        let db = salsa::DatabaseImpl::default();

        let class_name = intern_class_name(&db, "primary");
        let property_name = intern_property_name(&db, "primary");

        assert!(matches!((&class_name, &property_name), (Ok(_), Ok(_))));
        if let (Ok(class_name), Ok(property_name)) = (class_name, property_name) {
            assert_eq!(class_name.text(&db), property_name.text(&db));
        }
    }

    #[test]
    fn rejects_empty_names_through_validated_helpers() {
        let db = salsa::DatabaseImpl::default();

        assert_eq!(
            intern_mixin_name(&db, ""),
            Err(InternError::EmptyText {
                kind: NameKind::MixinName,
            }),
        );
    }

    #[test]
    fn interns_all_declared_name_surfaces() {
        let db = salsa::DatabaseImpl::default();

        assert!(intern_css_ident(&db, "display").is_ok());
        assert!(intern_selector_key(&db, ".button").is_ok());
        assert!(intern_custom_property_name(&db, "--space").is_ok());
        assert!(intern_keyframes_name(&db, "fade-in").is_ok());
        assert!(intern_file_path(&db, "/workspace/Button.module.scss").is_ok());
    }

    #[test]
    fn summarizes_phase_beta_name_identity_boundary() {
        let summary = summarize_omena_interner_boundary();

        assert_eq!(summary.product, "omena-interner.boundary");
        assert_eq!(summary.phase, "h1-beta-name-identity-substrate");
        assert_eq!(summary.name_kind_count, 8);
        assert_eq!(summary.salsa_interned_type_count, 8);
        assert_eq!(summary.validated_helper_count, 8);
        assert_eq!(summary.symbol_mapped_name_kind_count, 4);
        assert!(summary.ready_surfaces.contains(&"typedSalsaInternedNames"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"parserSemanticNameConsumption")
        );
        assert!(summary.ready_surfaces.contains(&"semanticSoaNameTables"));
        assert!(
            !summary
                .next_surfaces
                .contains(&"parserSemanticNameConsumption")
        );
        assert!(!summary.next_surfaces.contains(&"semanticSoaNameTables"));
    }
}

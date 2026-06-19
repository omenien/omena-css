use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};

use crate::abstract_css_value_kind;

use super::{
    OmenaScssEvalStaticValueResolutionV0, static_scss_function_value_contains_any_callable,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StaticStylesheetResolutionOutcome {
    Resolved,
    Raw,
    Top,
}

impl StaticStylesheetResolutionOutcome {
    fn label(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Raw => "raw",
            Self::Top => "top",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StaticStylesheetResolutionReason {
    Resolved,
    Cycle,
    FuelExhausted,
    UnresolvedReference,
    UnsupportedDynamic,
}

impl StaticStylesheetResolutionReason {
    fn label(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Cycle => "cycle",
            Self::FuelExhausted => "fuelExhausted",
            Self::UnresolvedReference => "unresolvedReference",
            Self::UnsupportedDynamic => "unsupportedDynamic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticStylesheetAbstractResolution {
    pub(super) rendered_value: Option<String>,
    pub(super) abstract_value: AbstractCssValueV0,
    pub(super) outcome: StaticStylesheetResolutionOutcome,
    pub(super) reason: StaticStylesheetResolutionReason,
}

pub(super) fn static_value_resolution_record(
    name: &str,
    start: usize,
    end: usize,
    source_text: &str,
    resolution: StaticStylesheetAbstractResolution,
) -> OmenaScssEvalStaticValueResolutionV0 {
    OmenaScssEvalStaticValueResolutionV0 {
        name: name.to_string(),
        start,
        end,
        source_text: source_text.to_string(),
        rendered_value: resolution.rendered_value,
        abstract_value_kind: abstract_css_value_kind(&resolution.abstract_value),
        abstract_value: resolution.abstract_value,
        outcome: resolution.outcome.label(),
        reason: resolution.reason.label(),
    }
}

pub(super) fn resolved_static_abstract_value(text: &str) -> StaticStylesheetAbstractResolution {
    let abstract_value = abstract_css_value_from_text(text);
    let rendered_value = render_static_abstract_value(&abstract_value);
    let outcome = if matches!(abstract_value, AbstractCssValueV0::Raw { .. }) {
        StaticStylesheetResolutionOutcome::Raw
    } else {
        StaticStylesheetResolutionOutcome::Resolved
    };
    let reason = if outcome == StaticStylesheetResolutionOutcome::Raw {
        StaticStylesheetResolutionReason::UnsupportedDynamic
    } else {
        StaticStylesheetResolutionReason::Resolved
    };
    StaticStylesheetAbstractResolution {
        rendered_value,
        abstract_value,
        outcome,
        reason,
    }
}

pub(super) fn resolved_static_abstract_value_preserving_callable_raw(
    original_text: &str,
    reduced_text: &str,
) -> StaticStylesheetAbstractResolution {
    let abstract_value = abstract_css_value_from_text(reduced_text);
    if matches!(abstract_value, AbstractCssValueV0::Raw { .. })
        && static_scss_function_value_contains_any_callable(reduced_text)
    {
        return raw_static_abstract_value(
            original_text,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    resolved_static_abstract_value(reduced_text)
}

pub(super) fn raw_static_abstract_value(
    text: &str,
    reason: StaticStylesheetResolutionReason,
) -> StaticStylesheetAbstractResolution {
    StaticStylesheetAbstractResolution {
        rendered_value: Some(text.to_string()),
        abstract_value: AbstractCssValueV0::Raw {
            value: text.to_string(),
        },
        outcome: StaticStylesheetResolutionOutcome::Raw,
        reason,
    }
}

pub(super) fn top_static_abstract_value(
    reason: StaticStylesheetResolutionReason,
) -> StaticStylesheetAbstractResolution {
    StaticStylesheetAbstractResolution {
        rendered_value: None,
        abstract_value: AbstractCssValueV0::Top,
        outcome: StaticStylesheetResolutionOutcome::Top,
        reason,
    }
}

pub(super) fn render_static_abstract_value(value: &AbstractCssValueV0) -> Option<String> {
    match value {
        AbstractCssValueV0::Bottom => Some(String::new()),
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => {
            Some(value.clone())
        }
        AbstractCssValueV0::FiniteSet { values } => values.first().cloned(),
        AbstractCssValueV0::Top => None,
    }
}

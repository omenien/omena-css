use std::cmp::Ordering;

use crate::{CascadeKey, CascadeLevel, Specificity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CascadeKeyAxisV0 {
    Level,
    LayerRank,
    ScopeProximity,
    SpecificityIds,
    SpecificityClasses,
    SpecificityElements,
    SourceOrder,
}

pub(crate) const CASCADE_KEY_AXIS_ORDER_V0: [CascadeKeyAxisV0; 7] = [
    CascadeKeyAxisV0::Level,
    CascadeKeyAxisV0::LayerRank,
    CascadeKeyAxisV0::ScopeProximity,
    CascadeKeyAxisV0::SpecificityIds,
    CascadeKeyAxisV0::SpecificityClasses,
    CascadeKeyAxisV0::SpecificityElements,
    CascadeKeyAxisV0::SourceOrder,
];

pub(crate) const fn cascade_key_axis_order_v0() -> &'static [CascadeKeyAxisV0] {
    &CASCADE_KEY_AXIS_ORDER_V0
}

pub(crate) const fn cascade_key_axis_name_v0(axis: CascadeKeyAxisV0) -> &'static str {
    match axis {
        CascadeKeyAxisV0::Level => "level",
        CascadeKeyAxisV0::LayerRank => "layerRank",
        CascadeKeyAxisV0::ScopeProximity => "scopeProximity",
        CascadeKeyAxisV0::SpecificityIds => "specificityIds",
        CascadeKeyAxisV0::SpecificityClasses => "specificityClasses",
        CascadeKeyAxisV0::SpecificityElements => "specificityElements",
        CascadeKeyAxisV0::SourceOrder => "sourceOrder",
    }
}

pub(crate) fn compare_cascade_key_axes_v0(left: &CascadeKey, right: &CascadeKey) -> Ordering {
    compare_axes(left, right, cascade_key_axis_order_v0().iter().copied())
}

pub(crate) fn compare_cascade_axis_prefix_v0(left: &CascadeKey, right: &CascadeKey) -> Ordering {
    compare_axes(
        left,
        right,
        cascade_key_axis_order_v0()
            .iter()
            .copied()
            .take_while(|axis| !is_specificity_axis(*axis)),
    )
}

pub(crate) fn compare_specificity_axes_v0(left: &Specificity, right: &Specificity) -> Ordering {
    for axis in cascade_key_axis_order_v0()
        .iter()
        .copied()
        .filter(|axis| is_specificity_axis(*axis))
    {
        let ordering = match axis {
            CascadeKeyAxisV0::SpecificityIds => left.ids.cmp(&right.ids),
            CascadeKeyAxisV0::SpecificityClasses => left.classes.cmp(&right.classes),
            CascadeKeyAxisV0::SpecificityElements => left.elements.cmp(&right.elements),
            _ => unreachable!("specificity-axis filter admitted a non-specificity axis"),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

pub(crate) fn first_deciding_cascade_key_axis_v0(
    winner: &CascadeKey,
    challenger: &CascadeKey,
) -> Option<CascadeKeyAxisV0> {
    cascade_key_axis_order_v0()
        .iter()
        .copied()
        .find(|axis| compare_axis(*axis, winner, challenger) != Ordering::Equal)
}

pub(crate) fn cascade_key_axis_signed_distance_v0(
    axis: CascadeKeyAxisV0,
    winner: &CascadeKey,
    challenger: &CascadeKey,
) -> i64 {
    match axis {
        CascadeKeyAxisV0::Level => {
            cascade_level_rank(winner.level) - cascade_level_rank(challenger.level)
        }
        CascadeKeyAxisV0::LayerRank => {
            i64::from(winner.layer_rank.get()) - i64::from(challenger.layer_rank.get())
        }
        CascadeKeyAxisV0::ScopeProximity => {
            i64::from(challenger.scope_proximity) - i64::from(winner.scope_proximity)
        }
        CascadeKeyAxisV0::SpecificityIds => {
            i64::from(winner.specificity.ids) - i64::from(challenger.specificity.ids)
        }
        CascadeKeyAxisV0::SpecificityClasses => {
            i64::from(winner.specificity.classes) - i64::from(challenger.specificity.classes)
        }
        CascadeKeyAxisV0::SpecificityElements => {
            i64::from(winner.specificity.elements) - i64::from(challenger.specificity.elements)
        }
        CascadeKeyAxisV0::SourceOrder => {
            i64::from(winner.source_order) - i64::from(challenger.source_order)
        }
    }
}

fn compare_axes(
    left: &CascadeKey,
    right: &CascadeKey,
    axes: impl IntoIterator<Item = CascadeKeyAxisV0>,
) -> Ordering {
    for axis in axes {
        let ordering = compare_axis(axis, left, right);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

fn compare_axis(axis: CascadeKeyAxisV0, left: &CascadeKey, right: &CascadeKey) -> Ordering {
    match axis {
        CascadeKeyAxisV0::Level => left.level.cmp(&right.level),
        CascadeKeyAxisV0::LayerRank => left.layer_rank.cmp(&right.layer_rank),
        CascadeKeyAxisV0::ScopeProximity => right.scope_proximity.cmp(&left.scope_proximity),
        CascadeKeyAxisV0::SpecificityIds => left.specificity.ids.cmp(&right.specificity.ids),
        CascadeKeyAxisV0::SpecificityClasses => {
            left.specificity.classes.cmp(&right.specificity.classes)
        }
        CascadeKeyAxisV0::SpecificityElements => {
            left.specificity.elements.cmp(&right.specificity.elements)
        }
        CascadeKeyAxisV0::SourceOrder => left.source_order.cmp(&right.source_order),
    }
}

const fn is_specificity_axis(axis: CascadeKeyAxisV0) -> bool {
    matches!(
        axis,
        CascadeKeyAxisV0::SpecificityIds
            | CascadeKeyAxisV0::SpecificityClasses
            | CascadeKeyAxisV0::SpecificityElements
    )
}

const fn cascade_level_rank(level: CascadeLevel) -> i64 {
    match level {
        CascadeLevel::UserAgentNormal => 0,
        CascadeLevel::UserNormal => 1,
        CascadeLevel::AuthorNormal => 2,
        CascadeLevel::InlineNormal => 3,
        CascadeLevel::Animation => 4,
        CascadeLevel::AuthorImportant => 5,
        CascadeLevel::InlineImportant => 6,
        CascadeLevel::UserImportant => 7,
        CascadeLevel::UserAgentImportant => 8,
        CascadeLevel::Transition => 9,
    }
}

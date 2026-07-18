//! Cascade-origin inputs and their mapping onto the existing priority ladder.

use serde::{Deserialize, Serialize};

use crate::CascadeLevel;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
)]
#[serde(rename_all = "camelCase")]
pub enum CascadeOriginV0 {
    UserAgent,
    User,
    #[default]
    Author,
    Inline,
}

impl CascadeOriginV0 {
    pub const fn is_author(&self) -> bool {
        matches!(self, Self::Author)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeOriginDriverV0 {
    pub origin: CascadeOriginV0,
    pub important: bool,
    pub level: CascadeLevel,
}

pub const fn cascade_level_for_origin(origin: CascadeOriginV0, important: bool) -> CascadeLevel {
    match (origin, important) {
        (CascadeOriginV0::UserAgent, false) => CascadeLevel::UserAgentNormal,
        (CascadeOriginV0::User, false) => CascadeLevel::UserNormal,
        (CascadeOriginV0::Author, false) => CascadeLevel::AuthorNormal,
        (CascadeOriginV0::Inline, false) => CascadeLevel::InlineNormal,
        (CascadeOriginV0::UserAgent, true) => CascadeLevel::UserAgentImportant,
        (CascadeOriginV0::User, true) => CascadeLevel::UserImportant,
        (CascadeOriginV0::Author | CascadeOriginV0::Inline, true) => CascadeLevel::AuthorImportant,
    }
}

pub const fn cascade_level_catalog_v0() -> [CascadeLevel; 9] {
    [
        CascadeLevel::UserAgentNormal,
        CascadeLevel::UserNormal,
        CascadeLevel::AuthorNormal,
        CascadeLevel::InlineNormal,
        CascadeLevel::Animation,
        CascadeLevel::AuthorImportant,
        CascadeLevel::UserImportant,
        CascadeLevel::UserAgentImportant,
        CascadeLevel::Transition,
    ]
}

pub const fn cascade_level_name_v0(level: CascadeLevel) -> &'static str {
    match level {
        CascadeLevel::UserAgentNormal => "userAgentNormal",
        CascadeLevel::UserNormal => "userNormal",
        CascadeLevel::AuthorNormal => "authorNormal",
        CascadeLevel::InlineNormal => "inlineNormal",
        CascadeLevel::Animation => "animation",
        CascadeLevel::AuthorImportant => "authorImportant",
        CascadeLevel::UserImportant => "userImportant",
        CascadeLevel::UserAgentImportant => "userAgentImportant",
        CascadeLevel::Transition => "transition",
    }
}

pub const fn cascade_origin_driver_catalog_v0() -> [CascadeOriginDriverV0; 8] {
    [
        origin_driver(CascadeOriginV0::UserAgent, false),
        origin_driver(CascadeOriginV0::User, false),
        origin_driver(CascadeOriginV0::Author, false),
        origin_driver(CascadeOriginV0::Inline, false),
        origin_driver(CascadeOriginV0::Author, true),
        origin_driver(CascadeOriginV0::Inline, true),
        origin_driver(CascadeOriginV0::User, true),
        origin_driver(CascadeOriginV0::UserAgent, true),
    ]
}

const fn origin_driver(origin: CascadeOriginV0, important: bool) -> CascadeOriginDriverV0 {
    CascadeOriginDriverV0 {
        origin,
        important,
        level: cascade_level_for_origin(origin, important),
    }
}

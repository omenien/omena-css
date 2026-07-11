use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum OmenaConfigReportKind {
    UnknownKey,
    NotYetConsumed,
    ShadowedConfig,
}

impl OmenaConfigReportKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownKey => "unknownKey",
            Self::NotYetConsumed => "notYetConsumed",
            Self::ShadowedConfig => "shadowedConfig",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OmenaConfigReport {
    pub(crate) kind: OmenaConfigReportKind,
    pub(crate) path: String,
    pub(crate) detail: String,
}

impl OmenaConfigReport {
    pub(crate) fn unknown(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            kind: OmenaConfigReportKind::UnknownKey,
            detail: format!(
                "unrecognized configuration key `{path}` was observed as a reported gap"
            ),
            path,
        }
    }

    pub(crate) fn not_yet_consumed(path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind: OmenaConfigReportKind::NotYetConsumed,
            path: path.into(),
            detail: detail.into(),
        }
    }

    pub(crate) fn shadowed(path: impl Into<String>, selected: impl Into<String>) -> Self {
        let path = path.into();
        let selected = selected.into();
        Self {
            kind: OmenaConfigReportKind::ShadowedConfig,
            detail: format!(
                "configuration `{path}` is shadowed by canonical candidate `{selected}`"
            ),
            path,
        }
    }

    pub(crate) fn render_warning(&self) -> String {
        format!(
            "omena config [{}] {}: {}",
            self.kind.as_str(),
            self.path,
            self.detail
        )
    }
}

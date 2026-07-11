use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductVerb {
    Check,
    #[allow(
        dead_code,
        reason = "wired verbs remain part of the complete product command contract"
    )]
    Lint,
    Fmt,
    Minify,
    Bundle,
    Modules,
    Sass,
    Intel,
    Migrate,
    Verify,
    Ci,
    Explain,
}

impl ProductVerb {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Lint => "lint",
            Self::Fmt => "fmt",
            Self::Minify => "minify",
            Self::Bundle => "bundle",
            Self::Modules => "modules",
            Self::Sass => "sass",
            Self::Intel => "intel",
            Self::Migrate => "migrate",
            Self::Verify => "verify",
            Self::Ci => "ci",
            Self::Explain => "explain",
        }
    }
}

#[derive(Debug)]
pub(crate) enum CliExit {
    Failure(String),
    NotYetWired { verb: ProductVerb },
}

impl CliExit {
    pub(crate) fn failure(message: String) -> Self {
        Self::Failure(message)
    }

    pub(crate) const fn not_yet_wired(verb: ProductVerb) -> Self {
        Self::NotYetWired { verb }
    }

    pub(crate) const fn code(&self) -> u8 {
        match self {
            Self::Failure(_) => 1,
            Self::NotYetWired { .. } => 2,
        }
    }
}

impl fmt::Display for CliExit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failure(message) => formatter.write_str(message),
            Self::NotYetWired { verb } => write!(
                formatter,
                "omena {} is reserved but not yet wired; run `omena {} --help` to inspect its command contract",
                verb.as_str(),
                verb.as_str()
            ),
        }
    }
}

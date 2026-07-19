/// Borrowed CSS keyword text compared under the ASCII case-insensitive rules
/// used by CSS keyword grammars.
#[derive(Debug, Clone, Copy)]
pub struct CssKeywordText<'a> {
    text: &'a str,
}

/// Wraps borrowed text for allocation-free CSS keyword comparisons.
pub const fn css_keyword(text: &str) -> CssKeywordText<'_> {
    CssKeywordText { text }
}

impl<'a> CssKeywordText<'a> {
    /// Returns whether the complete text equals `expected` ignoring ASCII case.
    pub fn equals(self, expected: &str) -> bool {
        self.text.eq_ignore_ascii_case(expected)
    }

    /// Removes an ASCII case-insensitive prefix and returns the borrowed remainder.
    pub fn strip_prefix(self, expected: &str) -> Option<&'a str> {
        let prefix = self.text.get(..expected.len())?;
        prefix
            .eq_ignore_ascii_case(expected)
            .then(|| &self.text[expected.len()..])
    }

    /// Removes an ASCII case-insensitive suffix and returns the borrowed remainder.
    pub fn strip_suffix(self, expected: &str) -> Option<&'a str> {
        let suffix_start = self.text.len().checked_sub(expected.len())?;
        let suffix = self.text.get(suffix_start..)?;
        suffix
            .eq_ignore_ascii_case(expected)
            .then(|| &self.text[..suffix_start])
    }

    /// Finds the first ASCII case-insensitive match and returns its byte offset.
    pub fn find(self, expected: &str) -> Option<usize> {
        if expected.is_empty() {
            return Some(0);
        }
        self.text
            .as_bytes()
            .windows(expected.len())
            .position(|candidate| candidate.eq_ignore_ascii_case(expected.as_bytes()))
    }

    /// Returns whether the text contains an ASCII case-insensitive match.
    pub fn contains(self, expected: &str) -> bool {
        self.find(expected).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::css_keyword;

    #[test]
    fn compares_css_keywords_without_allocating_lowercase_copies() {
        assert!(css_keyword("@LaYeR").equals("@layer"));
        assert!(!css_keyword("@layered").equals("@layer"));
    }

    #[test]
    fn strips_only_complete_ascii_case_insensitive_affixes() {
        assert_eq!(
            css_keyword("@KEYFRAMES fade").strip_prefix("@keyframes"),
            Some(" fade")
        );
        assert_eq!(
            css_keyword("red !IMPORTANT").strip_suffix("!important"),
            Some("red ")
        );
        assert_eq!(css_keyword("@lay").strip_prefix("@layer"), None);
        assert_eq!(css_keyword("@FORWARD 'x' AS ui-*").find(" as "), Some(12));
        assert!(css_keyword("media:@SuPpOrTs").contains("@supports"));
    }
}

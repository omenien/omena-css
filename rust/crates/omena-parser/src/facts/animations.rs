//! Parser facts for animation and keyframes references.
//!
//! These facts are intentionally token-derived so diagnostics can consume
//! animation names without depending on later semantic graph construction.

use cstree::text::TextRange;
use omena_syntax::SyntaxKind;
use std::collections::BTreeSet;

use crate::{Token, next_non_trivia_token_index_until};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnimationFact {
    pub kind: ParsedAnimationFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedAnimationFactKind {
    KeyframesDeclaration,
    AnimationNameReference,
}

pub(crate) fn collect_animation_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedAnimationFact> {
    let mut animations = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind == SyntaxKind::AtKeyword && at_keyword_is_keyframes_rule(token.text) {
            if let Some(name_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
                && let Some(name) = animation_name_from_token(tokens[name_index])
            {
                push_animation_fact(
                    &mut animations,
                    &mut seen,
                    ParsedAnimationFactKind::KeyframesDeclaration,
                    name,
                    tokens[name_index].range,
                );
            }
            continue;
        }

        if token.kind == SyntaxKind::Ident
            && token.text.eq_ignore_ascii_case("animation-name")
            && let Some(colon_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            collect_animation_name_references_until(
                tokens,
                colon_index + 1,
                &mut animations,
                &mut seen,
            );
        }

        if token.kind == SyntaxKind::Ident
            && token.text.eq_ignore_ascii_case("animation")
            && let Some(colon_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            collect_animation_shorthand_references_until(
                tokens,
                colon_index + 1,
                &mut animations,
                &mut seen,
            );
        }
    }
    animations
}

fn collect_animation_name_references_until(
    tokens: &[Token<'_>],
    start: usize,
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                break;
            }
            _ => {}
        }

        if paren_depth == 0
            && bracket_depth == 0
            && !animation_name_token_is_interpolation_adjacent(tokens, index)
            && let Some(name) = animation_name_from_token(tokens[index])
        {
            push_animation_fact(
                animations,
                seen,
                ParsedAnimationFactKind::AnimationNameReference,
                name,
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn collect_animation_shorthand_references_until(
    tokens: &[Token<'_>],
    start: usize,
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                break;
            }
            _ => {}
        }

        if paren_depth == 0
            && bracket_depth == 0
            && animation_shorthand_token_can_be_name(tokens, index)
            && let Some(name) = animation_name_from_token(tokens[index])
        {
            push_animation_fact(
                animations,
                seen,
                ParsedAnimationFactKind::AnimationNameReference,
                name,
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn animation_shorthand_token_can_be_name(tokens: &[Token<'_>], index: usize) -> bool {
    let token = tokens[index];
    if token.kind == SyntaxKind::String {
        return true;
    }
    if token.kind != SyntaxKind::Ident {
        return false;
    }
    // A literal fragment that is *immediately* adjacent to an interpolation boundary is part
    // of a statically-unknown name (`#{$dur}s` unit suffix, `#{$p}-spin` / `spin-#{$p}`
    // interpolated keyframes name), not a standalone animation name. Reject it so neither the
    // unit nor the literal fragment is misread as a missing `@keyframes` reference.
    if animation_name_token_is_interpolation_adjacent(tokens, index) {
        return false;
    }
    // Standalone CSS time-unit idents (`s` / `ms`) are durations, never animation names.
    if animation_shorthand_ident_is_time_unit(token.text) {
        return false;
    }
    if let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        && tokens[next_index].kind == SyntaxKind::LeftParen
    {
        return false;
    }
    !animation_shorthand_ident_is_non_name(token.text)
}

fn animation_shorthand_ident_is_time_unit(name: &str) -> bool {
    name.eq_ignore_ascii_case("s") || name.eq_ignore_ascii_case("ms")
}

/// An ident is part of an interpolated (statically-unknown) animation name when it is
/// *immediately* adjacent to an interpolation boundary — `#{$p}-spin` (post-interpolation
/// literal fragment) or `spin-#{$p}` (pre-interpolation literal fragment). The post-`#{...}`
/// text is the trailing fragment of a dynamic name, not a real keyframes reference, so it
/// must not be flagged as `missingKeyframes`.
///
/// Adjacency is checked against the *immediate* neighbor token (no trivia skipping): a
/// fully-static name separated from an interpolation by whitespace (`#{$p} spin`, a real
/// space-delimited keyframes reference) is NOT suppressed.
fn animation_name_token_is_interpolation_adjacent(tokens: &[Token<'_>], index: usize) -> bool {
    if index > 0
        && matches!(
            tokens[index - 1].kind,
            SyntaxKind::ScssInterpolationEnd | SyntaxKind::LessInterpolationEnd
        )
    {
        return true;
    }
    if let Some(next) = tokens.get(index + 1)
        && matches!(
            next.kind,
            SyntaxKind::ScssInterpolationStart | SyntaxKind::LessInterpolationStart
        )
    {
        return true;
    }
    false
}

fn animation_shorthand_ident_is_non_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "ease"
            | "ease-in"
            | "ease-out"
            | "ease-in-out"
            | "linear"
            | "step-start"
            | "step-end"
            | "infinite"
            | "normal"
            | "reverse"
            | "alternate"
            | "alternate-reverse"
            | "running"
            | "paused"
            | "forwards"
            | "backwards"
            | "both"
            | "replace"
            | "add"
            | "accumulate"
            | "auto"
    )
}

fn push_animation_fact(
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
    kind: ParsedAnimationFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        animations.push(ParsedAnimationFact { kind, name, range });
    }
}

fn animation_name_from_token(token: Token<'_>) -> Option<String> {
    if !matches!(token.kind, SyntaxKind::Ident | SyntaxKind::String) {
        return None;
    }
    let name = token
        .text
        .trim_matches(|character| character == '"' || character == '\'')
        .to_string();
    if name.is_empty() || animation_name_is_reserved(&name) {
        return None;
    }
    Some(name)
}

fn animation_name_is_reserved(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "none" | "initial" | "inherit" | "unset" | "revert" | "revert-layer"
    )
}

/// Recognize an `@keyframes` at-rule prefix-insensitively.
///
/// Per CSS, `animation-name` resolves against any `@keyframes`/`@-webkit-keyframes`
/// (and other vendor prefixes) with a matching name, so a vendor-prefixed at-rule
/// must register the same bare keyframes-name fact as the unprefixed form. Strips a
/// leading `@`, then an optional `-vendor-` prefix, and compares the remainder to
/// `keyframes`.
fn at_keyword_is_keyframes_rule(text: &str) -> bool {
    let Some(rule) = text.strip_prefix('@') else {
        return false;
    };
    if rule.eq_ignore_ascii_case("keyframes") {
        return true;
    }
    // Accept a single `-vendor-` prefix (`-webkit-`, `-moz-`, `-o-`, `-ms-`, ...).
    if let Some(rest) = rule.strip_prefix('-')
        && let Some((vendor, remainder)) = rest.split_once('-')
        && !vendor.is_empty()
        && remainder.eq_ignore_ascii_case("keyframes")
    {
        return true;
    }
    false
}

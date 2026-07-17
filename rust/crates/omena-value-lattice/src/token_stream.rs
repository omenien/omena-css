use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::ValueByteSpanV0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssValueComponentV0 {
    pub kind: CssValueComponentKindV0,
    pub text: String,
    pub span: ValueByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssValueComponentKindV0 {
    Ident,
    Number,
    Percentage,
    Dimension,
    Hash,
    String,
    Url,
    Function {
        name: String,
        arguments: Vec<CssValueComponentV0>,
    },
    Parenthesized {
        values: Vec<CssValueComponentV0>,
    },
    Bracketed {
        values: Vec<CssValueComponentV0>,
    },
    Braced {
        values: Vec<CssValueComponentV0>,
    },
    Comma,
    Slash,
    Delimiter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssValueTokenStreamErrorV0 {
    pub message: String,
    pub span: ValueByteSpanV0,
}

/// Builds a component-value stream from the parser-owned CSS lexer.
///
/// Whitespace and comments separate components but are not emitted as values.
/// Balanced functions and blocks retain their nested streams, so semantic
/// consumers never need an ad-hoc whitespace or comma parser.
pub fn css_value_component_stream(
    source: &str,
    base_offset: usize,
) -> Result<Vec<CssValueComponentV0>, CssValueTokenStreamErrorV0> {
    let lexed = lex(source, StyleDialect::Css);
    if let Some(error) = lexed.errors().first() {
        return Err(CssValueTokenStreamErrorV0 {
            message: error.message.to_string(),
            span: ValueByteSpanV0::new(base_offset, base_offset + source.len()),
        });
    }
    let mut parser = ComponentStreamParser {
        source,
        base_offset,
        tokens: lexed.tokens(),
        cursor: 0,
    };
    let values = parser.parse_until(None)?;
    if parser.cursor != parser.tokens.len() {
        let token = &parser.tokens[parser.cursor];
        return Err(parser.error_at(token, "unexpected closing token in CSS value"));
    }
    Ok(values)
}

struct ComponentStreamParser<'a> {
    source: &'a str,
    base_offset: usize,
    tokens: &'a [LexedToken],
    cursor: usize,
}

impl ComponentStreamParser<'_> {
    fn parse_until(
        &mut self,
        closing: Option<SyntaxKind>,
    ) -> Result<Vec<CssValueComponentV0>, CssValueTokenStreamErrorV0> {
        let mut values = Vec::new();
        loop {
            self.skip_trivia();
            let Some(token) = self.tokens.get(self.cursor) else {
                if let Some(kind) = closing {
                    return Err(CssValueTokenStreamErrorV0 {
                        message: format!("missing closing token {kind:?} in CSS value"),
                        span: ValueByteSpanV0::new(
                            self.base_offset + self.source.len(),
                            self.base_offset + self.source.len(),
                        ),
                    });
                }
                return Ok(values);
            };
            if closing == Some(token.kind) {
                self.cursor += 1;
                return Ok(values);
            }
            if is_closing_kind(token.kind) {
                return Err(self.error_at(token, "mismatched closing token in CSS value"));
            }
            values.push(self.parse_one()?);
        }
    }

    fn parse_one(&mut self) -> Result<CssValueComponentV0, CssValueTokenStreamErrorV0> {
        let token = self.tokens[self.cursor].clone();
        let start = token_start(&token);
        if let Some(open_index) = is_ident_text(token.text.as_str())
            .then(|| self.next_non_trivia(self.cursor + 1))
            .flatten()
            .filter(|(_, next)| {
                next.kind == SyntaxKind::LeftParen && token_end(&token) == token_start(next)
            })
            .map(|(open_index, _)| open_index)
        {
            self.cursor = open_index + 1;
            let arguments = self.parse_until(Some(SyntaxKind::RightParen))?;
            let end = token_end(&self.tokens[self.cursor - 1]);
            return Ok(self.component(
                start,
                end,
                CssValueComponentKindV0::Function {
                    name: token.text.to_ascii_lowercase(),
                    arguments,
                },
            ));
        }

        self.cursor += 1;
        let kind = match token.kind {
            SyntaxKind::Number => CssValueComponentKindV0::Number,
            SyntaxKind::Percentage => CssValueComponentKindV0::Percentage,
            SyntaxKind::Dimension => CssValueComponentKindV0::Dimension,
            SyntaxKind::Hash => CssValueComponentKindV0::Hash,
            SyntaxKind::String => CssValueComponentKindV0::String,
            SyntaxKind::Url => CssValueComponentKindV0::Url,
            SyntaxKind::Comma => CssValueComponentKindV0::Comma,
            SyntaxKind::Slash => CssValueComponentKindV0::Slash,
            SyntaxKind::LeftParen => {
                let values = self.parse_until(Some(SyntaxKind::RightParen))?;
                let end = token_end(&self.tokens[self.cursor - 1]);
                return Ok(self.component(
                    start,
                    end,
                    CssValueComponentKindV0::Parenthesized { values },
                ));
            }
            SyntaxKind::LeftBracket => {
                let values = self.parse_until(Some(SyntaxKind::RightBracket))?;
                let end = token_end(&self.tokens[self.cursor - 1]);
                return Ok(self.component(
                    start,
                    end,
                    CssValueComponentKindV0::Bracketed { values },
                ));
            }
            SyntaxKind::LeftBrace => {
                let values = self.parse_until(Some(SyntaxKind::RightBrace))?;
                let end = token_end(&self.tokens[self.cursor - 1]);
                return Ok(self.component(start, end, CssValueComponentKindV0::Braced { values }));
            }
            _ if is_ident_text(token.text.as_str()) => CssValueComponentKindV0::Ident,
            _ => CssValueComponentKindV0::Delimiter,
        };
        Ok(self.component(start, token_end(&token), kind))
    }

    fn component(
        &self,
        start: usize,
        end: usize,
        kind: CssValueComponentKindV0,
    ) -> CssValueComponentV0 {
        CssValueComponentV0 {
            kind,
            text: self.source[start..end].to_string(),
            span: ValueByteSpanV0::new(self.base_offset + start, self.base_offset + end),
        }
    }

    fn skip_trivia(&mut self) {
        while self
            .tokens
            .get(self.cursor)
            .is_some_and(|token| token.kind.is_trivia())
        {
            self.cursor += 1;
        }
    }

    fn next_non_trivia(&self, mut cursor: usize) -> Option<(usize, &LexedToken)> {
        while self
            .tokens
            .get(cursor)
            .is_some_and(|token| token.kind.is_trivia())
        {
            cursor += 1;
        }
        self.tokens.get(cursor).map(|token| (cursor, token))
    }

    fn error_at(&self, token: &LexedToken, message: &str) -> CssValueTokenStreamErrorV0 {
        CssValueTokenStreamErrorV0 {
            message: message.to_string(),
            span: ValueByteSpanV0::new(
                self.base_offset + token_start(token),
                self.base_offset + token_end(token),
            ),
        }
    }
}

fn token_start(token: &LexedToken) -> usize {
    u32::from(token.range.start()) as usize
}

fn token_end(token: &LexedToken) -> usize {
    u32::from(token.range.end()) as usize
}

fn is_closing_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::RightParen | SyntaxKind::RightBracket | SyntaxKind::RightBrace
    )
}

fn is_ident_text(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '-' || first == '_' || first.is_alphabetic() || !first.is_ascii())
        && chars.all(|character| {
            character == '-'
                || character == '_'
                || character.is_alphanumeric()
                || !character.is_ascii()
                || character == '\\'
        })
}

#[cfg(test)]
mod tests {
    use super::{CssValueComponentKindV0, css_value_component_stream};

    #[test]
    fn component_stream_preserves_compounds_and_nested_functions() -> Result<(), String> {
        let values =
            css_value_component_stream("1px solid color-mix(in srgb, red 20%, rgb(0 0 0)) / 2", 12)
                .map_err(|error| error.message)?;
        assert_eq!(values.len(), 5);
        assert!(matches!(values[0].kind, CssValueComponentKindV0::Dimension));
        assert!(matches!(values[1].kind, CssValueComponentKindV0::Ident));
        let CssValueComponentKindV0::Function { name, arguments } = &values[2].kind else {
            return Err("expected a function component".to_string());
        };
        assert_eq!(name, "color-mix");
        assert!(arguments.iter().any(|argument| {
            matches!(
                &argument.kind,
                CssValueComponentKindV0::Function { name, .. } if name == "rgb"
            )
        }));
        assert!(matches!(values[3].kind, CssValueComponentKindV0::Slash));
        assert_eq!(values[0].span.start, 12);
        Ok(())
    }

    #[test]
    fn component_stream_rejects_unbalanced_values_with_a_span() -> Result<(), String> {
        let Err(error) = css_value_component_stream("calc(1px + 2px", 30) else {
            return Err("unbalanced function should fail".to_string());
        };
        assert!(error.message.contains("missing closing token"));
        assert_eq!(error.span.start, 44);
        Ok(())
    }
}

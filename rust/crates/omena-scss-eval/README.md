# omena-scss-eval

Oracle-first SCSS and Less value evaluator rail for Omena CSS.

This crate is intentionally a narrow evaluator boundary over parser facts and
the shared abstract value vocabulary. Product output still consumes the legacy
`evaluated_css` string until the oracle rail proves native parity.

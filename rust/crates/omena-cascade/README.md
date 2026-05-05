# `omena-cascade`

Cascade-formal substrate for the Omena CSS track.

This crate owns the public model for cascade ordering, specificity, static
cascade outcomes, cascade proofs, selector-match witnesses, generic winner
selection, and custom-property substitution. `omena-semantic` consumes this
crate for design-token cascade ranking and selector-context witnesses so
cascade order does not stay duplicated in semantic consumers.

Selector matching is intentionally three-valued. The current witness supports
selector lists and simple compound selectors directly, reports exact misses for
concrete signatures, and returns `Maybe` for unsupported selectors or inexact
abstract element signatures instead of pretending to be a full browser selector
engine.

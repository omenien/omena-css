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

The crate also exposes a seed conformance corpus for the cascade ordering model.
That corpus covers source order, specificity, origin/importance level, layer
rank, scope proximity, and missing-property inheritance. It is not a replacement
for the full WPT `css/css-cascade` corpus; the full WPT mapping remains a later
conformance target.

Transform proof helpers live here as well. `@supports` static evaluation uses an
explicit modern-browser assumption witness, and `@scope` / `@layer` flattening
helpers reject candidates unless the caller can provide the closed-world
cascade conditions needed to erase scope proximity or layer rank.

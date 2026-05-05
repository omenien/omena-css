# `omena-cascade`

Cascade-formal substrate for the Omena CSS track.

This crate owns the public model for cascade ordering, specificity, static
cascade outcomes, cascade proofs, generic winner selection, and custom-property
substitution. `omena-semantic` consumes this crate for design-token cascade
ranking so cascade order does not stay duplicated in semantic consumers.

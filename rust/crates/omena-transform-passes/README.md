# omena-transform-passes

`omena-transform-passes` owns the transform pass registry and DAG planner for
the post-v5 omena-css track. It consumes `omena-transform-cst` contracts instead
of redefining pass metadata. Concrete mutation engines plug in behind this
registry so transform execution cannot drift from the semantic/cascade proof
obligations.

The first execution runtime surface is intentionally conservative. It executes
lexer-backed commodity mutations, context-gated bundle/module rewrites, and the
emission boundary only when the required resolver, evaluator, bridge, cascade,
or source-map evidence is present.

Current safe mutations include whitespace normalization, comment stripping,
numeric and color literal compression, zero-length unit normalization, URL and
string quote normalization, specificity-preserving selector compression,
adjacent exact rule deduplication, adjacent same-selector rule merging,
adjacent selector merging with identical declaration blocks, top-level empty
rule removal, conservative vendor prefixing, guarded color-function lowering,
simple nesting unwrap, static media/supports evaluation, and same-unit `calc()`
reduction.

Context-gated transforms include box-shorthand combining through the
`omena-cascade` proof, scope/layer flattening through cascade witnesses, import
inlining through explicit resolved CSS replacements, SCSS/Less module
evaluation through evaluator output, CSS Modules class hashing and `composes`
resolution through selector identity/export evidence, local `@value` resolution,
custom-property substitution, simple reachability-based tree shaking, and
design-token routing through bridge token routes.

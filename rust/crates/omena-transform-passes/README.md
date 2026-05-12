# omena-transform-passes

`omena-transform-passes` owns the P01-P40 transform pass registry and DAG
planner for the post-v5 omena-css track. It consumes
`omena-transform-cst` contracts instead of redefining pass metadata. Concrete
mutation engines will land behind this registry so transform execution cannot
drift from the semantic/cascade proof obligations.

The first execution runtime surface is intentionally conservative: it executes
lexer-backed safe commodity mutations for P01 through P20, P23-P25, P31-P38,
and observes the P40 emission boundary. P33-P36 tree shaking is context-gated
and mutates only under an explicit closed-style-world reachability context.
P04 unit normalization is limited to zero length dimensions
inside declaration properties that accept unitless zero; broader unit/value
rewrites remain planned until property/value semantics can prove them legal. P08
selector compression is limited to specificity-preserving `:is()` unwrapping and
duplicate argument removal for `:is()`/`:where()`. P09 shorthand combining
consumes the `omena-cascade` box-shorthand proof and only combines adjacent,
non-important margin/padding longhand quartets. P10 rule deduplication is
limited to adjacent exact duplicate ordinary rules. P11 rule merging is limited
to adjacent same-selector ordinary rules and preserves declaration order. P12
selector merging is limited to adjacent ordinary rules with identical
declaration blocks. P13 empty rule removal is limited to top-level ordinary
rules whose blocks contain only whitespace. P14 vendor prefixing currently
inserts conservative `-webkit-` synonyms for known prefix-sensitive properties
when absent. P15 `light-dark()` lowering only rewrites whole-value color
declarations into light defaults plus dark-mode media branches. P16
`color-mix()` lowering currently supports whole-value `in srgb` declarations
with static hex/basic named color operands. P17 `oklab()`/`oklch()` lowering
rewrites only whole-value in-gamut static colors to sRGB. P18 `color()`
lowering currently supports whole-value static `color(srgb ...)` declarations.
P19 logical-to-physical lowering requires static horizontal `direction` context
inside the same rule before rewriting inline logical properties. P20 nesting
unwrap is limited to simple single-depth ordinary nested rules without comma
selectors, at-rules, comments, or deeper nested blocks. P24 media static eval
only unwraps literal `@media all` and removes literal `@media not all`. P23
supports static eval consumes `omena-cascade` supports witnesses and currently
handles simple declaration feature queries under the modern-browser assumption.
P25 `calc()` reduction currently handles whole-value same-unit
addition/subtraction. P32 custom-property static resolve consumes
`omena-cascade` substitution and only resolves whole-value `var()` references
from unique literal `:root` custom properties. P31 value resolution only resolves
unique local literal CSS Modules `@value` declarations and whole-value
references; imports, aliases, strings, duplicates, and composite values remain
planned until the semantic graph can prove the full workspace closure. P33
removes only simple class-rule selector lists whose classes are absent from the
reachability context. P34 removes unreferenced top-level keyframes. P35 removes
unreferenced local literal `@value` declarations. P36 removes unreferenced
custom-property declarations. P37-P38 reuse the same static media/supports
witness evaluators as P24/P23 under their semantic-aware dead-branch pass
surfaces.

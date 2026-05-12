# omena-parser

`omena-parser` is the green-field parser track for the future Omena CSS engine.

It lives next to the current `engine-style-parser` and does not replace product behavior until parity gates are met. The crate starts with the stable public parser surface, tokenizer, CST builder, recovery vocabulary, and dialect-extension seams that later full grammar work will fill in.

Current scope:

- `ParseResult` over a `cstree` green root.
- `ParserCstEquivalenceSummaryV0` verifies that parser CST nodes and tokens use
  the shared `omena-syntax` `SyntaxKind` contract with source-text round-trip
  and typed-wrapper evidence.
- `LexResult` exposes token kind, original range, and token text for
  differential token checks.
- `omena-parser-summary` exposes the parser-owned parity-lite summary used by
  the public parser lane for selectors, CSS Modules values, keyframes, and
  structural CSS-family counts.
- `omena-parser-css-modules-intermediate` exposes the parser-owned
  CSS Modules intermediate index summary used by the parser index-bridge gate,
  including values, custom properties, Sass symbols, wrappers, keyframes,
  composes, and nested BEM selector metadata.
- Recursive-descent parser core coverage is explicit: stylesheet/rule/
  declaration entry points, Selectors L4 CST slices, registered at-rule
  preludes, CSS nesting, SCSS/Sass/Less dialect statements, Bogus recovery, and
  style-fact extraction are ready. The complete external CSS-family spec mirror
  remains a separate conformance target.
- Typed CST wrapper accessors for the current stylesheet, rule, selector,
  declaration, value, component-value, component-value-list, simple-block,
  custom-property-value, and at-rule nodes.
- Typed Bogus-node wrapper accessors for recovery-aware consumers.
- CSS Syntax entry points for rule lists, component values, component-value
  lists, comma-separated component-value lists, and simple blocks.
- CSS custom property declarations parse values as arbitrary component-value lists.
- Pratt value parser core coverage is explicit: unary `+`/`-`, additive and
  multiplicative precedence, parenthesized expressions, function argument
  lists, specialized CSS value function families, and value-level recovery are
  ready. The full CSS property-value grammar registry remains a separate
  product-cutover target.
- Functional pseudo-classes that carry selector lists (`:is`, `:where`,
  `:has`, `:not`, `:local`, `:global`) now surface nested selector-list CST
  nodes and isolate malformed selector-list items as Bogus selectors.
- `:nth-child()`, `:nth-last-child()`, `:nth-of-type()`, and
  `:nth-last-of-type()` expose formula arguments, including `of
<selector-list>`, as structured CST nodes.
- `:has()` arguments are represented as relative selector lists, preserving
  leading combinators for downstream selector semantics.
- `:lang()` and `:dir()` expose dedicated argument CST nodes for linguistic
  selector semantics.
- Attribute selectors expose dedicated name, matcher, value, and modifier
  nodes, including Selectors L4 case-sensitivity flags.
- Missing block-close recovery markers represented as `BogusTrivia`.
- Panic-free tokenizer for CSS-family source slices using char-boundary-safe cursor movement.
- Initial dialect classification for CSS, SCSS, Sass, and Less tokens.
- `TokenSet` recovery scaffolding and parser boundary summary.

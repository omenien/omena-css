# omena-parser

`omena-parser` is the green-field parser track for the future Omena CSS engine.

It lives next to the current `engine-style-parser` and does not replace product behavior until parity gates are met. The crate starts with the stable public parser surface, tokenizer, CST builder, recovery vocabulary, and dialect-extension seams that later full grammar work will fill in.

Current scope:

- `ParseResult` over a `cstree` green root.
- Typed CST wrapper accessors for the current stylesheet, rule, selector,
  declaration, value, component-value, component-value-list, simple-block,
  and at-rule nodes.
- Typed Bogus-node wrapper accessors for recovery-aware consumers.
- CSS Syntax entry points for rule lists, component values, component-value
  lists, comma-separated component-value lists, and simple blocks.
- CSS custom property declarations parse values as arbitrary component-value lists.
- Functional pseudo-classes that carry selector lists (`:is`, `:where`,
  `:has`, `:not`, `:local`, `:global`) now surface nested selector-list CST
  nodes and isolate malformed selector-list items as Bogus selectors.
- Missing block-close recovery markers represented as `BogusTrivia`.
- Panic-free tokenizer for CSS-family source slices using char-boundary-safe cursor movement.
- Initial dialect classification for CSS, SCSS, Sass, and Less tokens.
- `TokenSet` recovery scaffolding and parser boundary summary.

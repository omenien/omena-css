# omena-parser

`omena-parser` is the green-field parser track for the future Omena CSS engine.

It lives next to the current `engine-style-parser` and does not replace product behavior until parity gates are met. The crate starts with the stable public parser surface, tokenizer, CST builder, recovery vocabulary, and dialect-extension seams that later full grammar work will fill in.

Current scope:

- `ParseResult` over a `cstree` green root.
- Panic-free tokenizer for CSS-family source slices using char-boundary-safe cursor movement.
- Initial dialect classification for CSS, SCSS, Sass, and Less tokens.
- `TokenSet` recovery scaffolding and parser boundary summary.

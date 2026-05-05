# omena-syntax

`omena-syntax` is the phase-alpha syntax substrate for the Omena CSS parser stack.

It intentionally does not parse source text yet. The crate defines the shared syntax-kind ranges, CST bridge, and semantic vocabulary that later parser, semantic, resolver, LSP, and checker layers must consume instead of inventing their own local node/token taxonomies.

Current scope:

- Range-divided `SyntaxKind` values for CSS, SCSS, Sass, and Less.
- `cstree` raw-kind conversion and typed node/token aliases.
- Shared `SymbolKind`, `ScopeKind`, and `ReferenceKind` enums.
- Bogus-node superset for lossless error recovery.

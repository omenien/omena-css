# omena-interner

`omena-interner` owns the workspace-level string identity substrate for the Omena CSS parser stack.

The crate deliberately sits above `cstree` token interning. `cstree` keeps green-tree token storage compact; this crate gives semantic layers stable, typed Salsa interned IDs for names that participate in selector, symbol, scope, resolver, and query equality.

Current scope:

- Salsa interned IDs for class names, CSS identifiers, property names, selector keys, custom properties, keyframes, mixins, and file paths.
- A small validated helper API for constructing non-empty interned names.
- A `NameKind` inventory that maps interned names back to `omena-syntax` symbol categories where the mapping is unambiguous.

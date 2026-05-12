# omena-transform-bundle

`omena-transform-bundle` turns parser facts into a transform bundle plan for
`@import` inlining, SCSS/Less module evaluation, CSS Modules hashing,
`composes` resolution, and `@value` resolution. It does not inline text yet; it
establishes the source-fact boundary that later mutation engines must consume.

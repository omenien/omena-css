# omena-transform-bundle

`omena-transform-bundle` is the historical compatibility path for Omena's
bundle planning API. New consumers may depend on `omena-bundler` directly.

The crate re-exports the same `...V0` Rust surface from `omena-bundler` so
existing imports keep compiling during the 0.x line. The standalone
`omena-bundler` crate is still pre-1.0, and its surface may change.

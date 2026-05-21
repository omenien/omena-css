# omena-transform-target

`omena-transform-target` maps a target feature matrix to target-sensitive
transform passes. It keeps target lowering explicit: opt-in flattening passes
stay blocked unless the caller enables them, and final ordering is delegated to
`omena-transform-passes`.

Browser thresholds that are not fully covered by the embedded browserslist
snapshot are reviewed data, not inline code. The crate reads
`data/browser-thresholds.toml` and validates `data/pass-feature-bindings.toml`
so target-sensitive passes stay tied to explicit source quorum and pass IDs.

# omena-transform-target

`omena-transform-target` maps a target feature matrix to the P14-P24 transform
passes. It keeps target lowering explicit: opt-in flattening passes stay blocked
unless the caller enables them, and final ordering is delegated to
`omena-transform-passes`.

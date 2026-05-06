# omena-transform-print

`omena-transform-print` owns the P40 emission boundary. The first surface is a
provenance-preserving identity printer: it appends P40 to an upstream transform
plan, emits CSS bytes, and records source-map segments. Later formatting and
minified printers must keep this source-map contract.

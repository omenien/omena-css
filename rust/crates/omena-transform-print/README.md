# omena-transform-print

`omena-transform-print` owns the CSS emission boundary. The current supported
surface is a provenance-preserving identity printer: it attaches emission
provenance to an upstream transform plan, emits CSS bytes, and records
source-map segments. Query-owned transform plans now compose those source-map
segments from the execution provenance chain, so emitted artifacts preserve
pass-level source-map lineage. Segments expose byte offsets plus explicit
zero-based line/column points for both UTF-8 byte columns and UTF-16 columns so
downstream query, LSP, and source-map consumers do not need to recompute
position data from raw CSS text. Future formatting and minified printers must
preserve this source-map contract and tighten it from pass-level full-range
segments to narrower mutation spans where available before they are reported as
supported modes.

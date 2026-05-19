# omena-transform-print

`omena-transform-print` owns the CSS emission boundary. The supported surface
now covers provenance-preserving identity emission and minified emission that
reuses the transform pass runtime for comment/trivia deletion. Query-owned
transform plans compose source-map segments from the execution provenance
chain, so emitted artifacts preserve pass-level source-map lineage. Segments
expose byte offsets plus explicit zero-based line/column points for both UTF-8
byte columns and UTF-16 columns so downstream query, LSP, and source-map
consumers do not need to recompute position data from raw CSS text. Future
pretty formatting must preserve this source-map contract and tighten it from
pass-level full-range segments to narrower mutation spans where available
before it is reported as a supported mode.

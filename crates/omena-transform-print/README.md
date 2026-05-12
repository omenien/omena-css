# omena-transform-print

`omena-transform-print` owns the CSS emission boundary. The first surface is a
provenance-preserving identity printer: it attaches emission provenance to an
upstream transform plan, emits CSS bytes, and records source-map segments.
Query-owned transform plans now compose those source-map segments from the
execution provenance chain, so emitted artifacts preserve pass-level source-map
lineage. Later formatting and minified printers must keep this source-map
contract and tighten it from pass-level full-range segments to byte-precise
source spans.

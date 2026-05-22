# omena-spec-audit

Pinned CSS specification source audit substrate for Omena CSS.

This crate owns the Stage 1 advisory manifest shape for CSS spec source pins
and Omena coverage entries. It does not claim full spec parity; it only gates
source provenance, source freshness metadata, generated-data human-review
policy, manifest-to-source cross references, spec URLs, webref IDs, and P0
missing-coverage rationale policy. The manifest also declares cross-source
coverage for the pinned webref, browser-specs, web-features, and MDN browser
compat data sources so source pins are not merely listed without product
coverage evidence.

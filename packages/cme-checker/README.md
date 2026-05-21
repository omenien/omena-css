# `@css-module-explainer/cme-checker`

TS-side checker substrate for the M4 testing-toolkit lane.

This package owns reusable checker archetype helpers that are shared by
`scripts/check-rust-checker-*` gates. It is layered above the existing
shadow-runner and contract-parity fixtures; it does not replace the Rust
`omena-checker` crate.

The M4 surface includes per-field canonical-candidate diff reports. Checker
gates should use `assertCheckerCanonicalCandidateEqual` instead of raw object
equality so drift reports identify the exact summary/finding field that
changed.

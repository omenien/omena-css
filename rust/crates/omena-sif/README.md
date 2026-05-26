# omena-sif

`omena-sif` owns the Sass Interface File v1 contract used by the M7
foreign-reference resolution track.

The crate intentionally does not evaluate Sass, execute package code, or access
the network. It provides deterministic JSON writing and BLAKE3-tagged
fingerprints that higher layers can use for local `omena.lock` verification.

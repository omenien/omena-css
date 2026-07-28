# `omena-benchmarks`

Benchmark and measurement harnesses for Omena parser, query, reachability, and
transform paths. The crate owns committed performance baselines, representative
corpora, emitted-CSS fixtures, Criterion benchmarks, and Linux-only
instruction-count measurements.

It is an internal measurement crate and is not published to crates.io. Product
correctness remains covered by the corresponding library and differential
tests; benchmark thresholds are regression signals rather than semantic
authorities.

# omena-zk-audit

## Role

`omena-zk-audit` defines the opt-in zero-knowledge cascade audit protocol used
by the feature-gated CLI command.

## Contract

The crate carries typed audit inputs, public outputs, and backend-independent
protocol records.

## Consumers

`omena-cli` exposes this surface only when built with the `zk-audit` feature.

## Boundaries

Heavy proving systems remain optional. A protocol record is not evidence that a
proof backend ran unless the returned backend and verification fields say so.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-zk-audit
```

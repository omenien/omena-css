# Engine V2 Contract IDL Decisions

This document records the pre-codegen contract decisions for the Engine V2 JSON
wire surface. The IDL must describe the serde JSON shape that crosses the
TypeScript and Rust boundary. It must not describe napi-derive bindings directly,
because the native bridge accepts and returns JSON strings.

The current implementation still has hand-written contract mirrors. Codegen must
not start until the decisions below are reflected in the generated schemas,
TypeScript declarations, Rust serde structs, and round-trip tests.

## Contract Owner

The IDL owns the Engine V2 JSON wire contract.

Owned surfaces:

- `server/engine-core-ts/src/contracts/engine-v2.ts`
- `server/engine-host-node/src/engine-output-v2.ts`
- Engine-query result DTOs returned by `server/engine-host-node/src/engine-query-v2.ts`
- Rust input structs currently in `rust/crates/omena-engine-input-producers`
- Rust output mirrors currently local to `rust/crates/engine-shadow-runner`
- Rust query/code-action DTOs consumed through JSON-string bridge paths

Not owned by the first IDL cut:

- Runtime-only host dependencies such as `DocumentAnalysisCache`, `TypeResolver`,
  `StyleDocumentHIR` lookup callbacks, environment values, and workspace cache
  objects.
- The full source/style document HIR shape. Until the parser-owned fact authority
  is settled, HIR payloads remain opaque passthrough slots at the IDL boundary and
  are projected by product adapters where needed.

## Input Decisions

`EngineInputV2` remains the canonical input envelope. Its JSON shape is:

- `version`: required literal `"2"`.
- `workspace`: required object with `root`, `classnameTransform`, and `settingsKey`.
- `sources`: required array.
- `styles`: required array.
- `typeFacts`: required array.

Current drift to resolve:

- TypeScript requires `workspace`; Rust currently deserializes `EngineInputV2`
  without a `workspace` field and therefore ignores the TypeScript field.
- TypeScript names the type fact row `TypeFactTableEntryV2`; Rust names the same
  wire row `TypeFactEntryV2`.
- TypeScript `StringTypeFactsV2` carries optional `provenance`; Rust currently
  omits it and ignores the field during deserialization.
- TypeScript uses string literal unions for fact kind and constraint kind; Rust
  currently uses raw `String` fields.

Canonical decisions:

- `workspace` is part of the canonical full `EngineInputV2` contract. Native
  optional-input helpers that synthesize an empty input must either populate a
  valid default workspace or use a separate optional helper type before the
  generated contract is enforced.
- The IDL model name is `TypeFactEntryV2`. TypeScript may keep
  `TypeFactTableEntryV2` as a compatibility alias while consumers migrate.
- `StringTypeFactsV2.provenance` is part of the canonical contract and is
  optional with skip-on-absence serialization on Rust.
- String kind fields are closed IDL enums for the currently emitted values. New
  values require schema and round-trip fixture updates.
- Source/style document HIR fields remain opaque in the IDL. Narrow Rust
  projection structs can continue to exist as adapters, but they are not the
  cross-language source of truth.

## Output Decisions

`EngineOutputV2` remains the canonical output envelope. Its JSON shape is:

- `version`: required literal `"2"`.
- `queryResults`: required array.
- `rewritePlans`: required array.
- `checkerReport`: required object.

Current drift to resolve:

- `server/engine-core-ts/src/contracts/engine-v2.ts` defines the canonical
  `EngineOutputV2` and `QueryResultV2` union.
- `server/engine-host-node/src/engine-output-v2.ts` no longer defines a second
  `EngineOutputV2`, but it still defines a hand-written builder options DTO and
  defaulting behavior for `queryResults` and `rewritePlans`.
- `rust/crates/engine-shadow-runner/src/main.rs` defines a local Rust
  `EngineOutputV2` and local `QueryResultV2` union for shadow payloads.
- `server/engine-host-node/src/engine-query-v2.ts` constructs the same
  `QueryResultV2` variants but also contains runtime-only options that are not
  wire DTOs.
- `server/engine-host-node/src/code-action-query.ts` contains a hand-written
  Rust query JSON DTO for code-action plans. The runtime `CodeActionPlan` union
  is host-internal, but the JSON payload from Rust is a contract surface.

Canonical decisions:

- `EngineOutputV2` is generated from the IDL and exported from the core contract
  package.
- `QueryResultV2` is a tagged union on `kind` with these variants:
  `expression-semantics`, `source-expression-resolution`, and `selector-usage`.
- `rewritePlans` stays a required array on the wire. Builder helpers may accept
  optional input and normalize to an empty array, but generated output DTOs must
  not make the wire field optional.
- `CheckerReportV1` and `TextRewritePlan<unknown>` are referenced contract
  dependencies. The IDL must either import their schema fragments or explicitly
  model them as versioned referenced schemas, not duplicate ad-hoc shapes.
- Host builder input DTOs can be generated helper input types, but runtime-only
  dependency bags in `engine-query-v2.ts` stay manual.
- The Rust shadow-runner output mirror must be generated or replaced by a shared
  generated Rust output module before the drift gate can pass.
- The Rust code-action JSON plan shape must be IDL-owned; the host-internal
  `CodeActionPlan` workspace-edit union stays TypeScript-only.

## Codegen Requirements

The generator must produce all of the following from one authoritative IDL:

- TypeScript contract declarations for Engine V2 input and output.
- Rust serde structs for Engine V2 input, re-used by the bridge and query crates.
- Rust serde structs for Engine V2 output, re-used by the shadow runner.
- JSON Schema artifacts for input, output, query result variants, and the
  code-action query JSON surface.
- A drift gate that regenerates the files and fails on `git diff --exit-code`.

## Toolchain Decision

The current prototype uses:

- TypeSpec 1.13 with `@typespec/json-schema` 1.13 as the IDL front-end and JSON
  Schema emitter.
- Separate JSON Schema files, not a bundled schema file. The bundled emitter mode
  currently leaves `$ref` values that downstream TypeScript generation treats as
  external files; separate files plus an explicit schema root are deterministic.
- `json-schema-to-typescript` 15.0 for generated TypeScript declaration smoke
  checks.
- A repo-owned Rust serde emitter path for generated Rust structs. The first
  smoke gate emits a Rust proof crate from the IDL decisions and checks serde
  round-trips for `workspace`, `provenance`, tagged query results, required
  `rewritePlans`, and code-action query JSON.

The prototype command is `pnpm check:engine-v2-contract-idl-toolchain`. It does
not replace the production drift gate yet. It proves the selected toolchain can
represent the required wire decisions without manual patches before the generated
files are wired into product code.

The generator must preserve:

- camelCase serde names.
- Optional field omission semantics.
- Required array fields on canonical envelopes.
- Tagged-union discriminants.
- Byte-canonical fixture stability for both TypeScript-produced and
  Rust-produced JSON.

## Verification Requirements

Completion requires tests that cover both directions:

- TypeScript-produced `EngineInputV2` parses in Rust and serializes to the same
  canonical JSON after normalization.
- Rust-produced or Rust-consumed `EngineInputV2` validates against the schema and
  includes the canonical `workspace` and optional `provenance` behavior.
- TypeScript-produced `EngineOutputV2` validates against the schema and
  round-trips through the Rust output DTO.
- Rust shadow payload output validates against the same `EngineOutputV2` schema.
- Query result fixtures cover all `QueryResultV2.kind` variants.
- Code-action query JSON fixtures cover the Rust-produced action plan shape.

Structural or key-set-only comparison is not sufficient. The gate must compare
canonical serialized bytes after one normalization pass so that rename drift,
missing optional omission rules, and optional-vs-required array drift are visible.

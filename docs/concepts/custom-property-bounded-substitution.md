---
title: Custom-property dependency resolution
description: The shipped dependency-graph and strongly connected component algorithm, its independent witness, and the remaining value-domain boundary.
kind: explanation
status: stable
products: [cascade, diagnostics]
owner: cascade
sourceOfTruth: authored
---

# Custom-property dependency resolution

Omena resolves custom properties by the dependency structure required by CSS
Variables. The file name and several public Rust symbols retain earlier
fixed-point wording for compatibility; the shipped algorithm no longer returns
a bounded approximation.

## Shipped algorithm

The implementation builds a directed graph over the canonical keys in
`CustomPropertyEnv`. `collect_custom_property_reference_indices` visits every
`CascadeValue::Var`, including references that appear in a fallback, so a
fallback reference is a dependency edge rather than an escape from a cycle.
Graph nodes are `CanonicalCustomPropertyNameV0` values produced by
`PropertyNameV0`, not independently normalized strings.

`strongly_connected_components` partitions that graph. A component is cyclic
when it has more than one member or its only member has a self-edge. Every
member of a cyclic component becomes `CascadeValue::GuaranteedInvalid` before
any non-member is evaluated. A fallback therefore cannot rescue a declaration
that belongs to the cycle. A non-member that later references an invalid cycle
member may use its own fallback, which is the distinct outer-reference rule.

`dependency_ordered_components` schedules the acyclic remainder after its
dependencies. Each binding is evaluated once against the memoized resolved
environment by `substitute_custom_properties_against_resolved_env`. The
schedule covers every component, and the implementation asserts both complete
key coverage and the absence of remaining `var()` references. There is no
non-converged-value return path. Both Kosaraju passes and the condensation-graph
schedule are iterative, so a long alias chain does not consume the Rust call
stack. Product value resolution omits the compatibility trace; summaries build
each trace row from incremental counters instead of rescanning the environment.

## Code correspondence

| Semantic object       | Shipped symbol                                            | Correspondence                                                                                |
| --------------------- | --------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Canonical graph keys  | `CanonicalCustomPropertyNameV0` / `PropertyNameV0`        | Shared standard/custom property identity authority.                                           |
| Input environment     | `CustomPropertyEnv`                                       | Fixed `BTreeMap` of canonical custom-property keys.                                           |
| Value syntax          | `CascadeValue`                                            | Literal, composite, variable, CSS-wide states, indeterminate, guaranteed-invalid, and unset.  |
| Dependency graph      | `custom_property_dependency_graph`                        | Collects references from primary values and fallbacks.                                        |
| Reference collector   | `collect_custom_property_reference_indices`               | Maps primary and fallback references to canonical graph indices.                              |
| SCC partition         | `strongly_connected_components`                           | Computes the complete strongly connected component partition.                                 |
| Component schedule    | `dependency_ordered_components`                           | Orders distinct components after their dependencies.                                          |
| Cycle predicate       | `component_is_cyclic`                                     | Detects multi-member components and self-loops.                                               |
| Resolved substitution | `substitute_custom_properties_against_resolved_env`       | Applies fallbacks only while evaluating an outer, non-cycle value.                            |
| Public value query    | `substitute_custom_properties`                            | Resolves the environment structurally, then substitutes the requested outer value.            |
| Public environment    | `resolve_custom_property_env_least_fixed_point`           | Compatibility name returning the complete SCC-scheduled environment.                          |
| Public summary        | `summarize_custom_property_least_fixed_point`             | Compatibility shape exposing component-schedule observations and results.                     |
| Schedule observation  | `CustomPropertyLeastFixedPointIterationV0`                | One compatibility trace row per scheduled component.                                          |
| Structural witness    | `custom_property_bounded_fixed_point_computation_witness` | Compatibility name describing the graph, SCC, and no-approximation invariants.                |
| Completion bit        | `reached_fixed_point`                                     | Always true for a completed structural schedule; no false-valued approximation branch exists. |

The trace fields retain their public compatibility names, but each row is a
component-schedule observation rather than a Kleene iteration. Downstream
RG-flow projections therefore keep `monotoneKleeneCertificate` false for
non-empty structural summaries; a changing declaration count across component
rows is not presented as convergence evidence.

## Frozen independent witness

The committed independent oracle's five-field `oracle` object defines an
all-bottom status iteration. The complete evaluator kernel—from
`evaluate_from_all_bottom` through `evaluate_fixture_value` and
`finalize_oracle_environment`—and every `expectedEvaluator` projection are
protected by separate SHA-256 checks.

The current eight-case corpus reports eight agreements and zero findings. Its
named non-degenerate cycle-shape allowlist is exactly
`mutuallyRecursiveFallbackChain`, `cycleThroughFallback`, and
`threeNodeFallbackCycleEnteredMidChain`. The plain two-cycle remains a baseline
case without a novel-shape label. An outer-reference case separately proves
that a non-member may use its fallback after its dependency is invalid.

The `reordered-in-place` mutation still weakens the independent simultaneous
transfer and must fail. Product mutations that remove SCC classification or
reintroduce a cycle-member fallback rescue must also fail.

## Standard-property validation after substitution

A cascade winner for a standard property can contain `var()`, so its grammar
cannot always be decided before custom-property substitution. The cascade
crate exposes `CascadeStandardValueValidatorV0` as a port; the product adapter
`SpecStandardPropertyValueValidatorV0` delegates to the spec-derived
`validate_standard_property_value_v0` authority.

`compute_cascade_computed_value_with_standard_value_validator_v0` selects the
winner, substitutes custom properties, and then validates the resulting
standard-property text. A definite mismatch becomes invalid at computed-value
time. A validator result that remains unknown becomes typed indeterminate. If
the caller supplies no standard-property verdict, a literal or composite value
also becomes typed indeterminate rather than resolved. CSS-wide keywords and a
definite guaranteed-invalid substitution retain their independently known
computed-value behavior.

The Salsa source-element path carries inline custom-property bindings into the
same computation. Its regression case distinguishes `--tone: red`, which keeps
`color: var(--tone)` resolved, from `--tone: 12px`, which is invalid for
`color`. An always-valid validator or the former literal-only verdict filter
makes that product-path test fail.

A definite grammar mismatch is promoted to invalid only when the matcher has a
complete implementation-coverage certificate for every referenced grammar
shape. Otherwise the result stays indeterminate. The same pinned registry now
supplies the reviewed `<paint>` color inclusion and the modeled
`calc()`/`min()`/`max()`/`clamp()` and grid `repeat()`/`minmax()` paths; no
parallel property grammar table is consulted.

That path resolves the custom-property environment at every ancestor boundary
before applying child declarations. An inherited computed value therefore does
not rebind to a child override of one of its former dependencies. A dynamic
custom-property declaration is represented as indeterminate for that key, so
it blocks a value that references it without blocking an unrelated static
standard-property declaration.

## Remaining rfcs#10 boundary

The committed register is
`rust/omena-custom-property-fixed-point-gap-register.json`.

<!-- gap-register:start -->

| Gap id                     | Shipped state                                     | Claims-under-test state                                                               | Observable consequence                                                     | Upgrade cost                                                                                                       |
| -------------------------- | ------------------------------------------------- | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `conditional-value-domain` | `CascadeValue` has no conditional `if()` variant. | Conditional custom-property values participate in the dependency and transfer domain. | Branch-sensitive conditional dependencies cannot be represented or proved. | Extend parsing and `CascadeValue`, define branch joins, and update every substitution and computed-value consumer. |

<!-- gap-register:end -->

The earlier cycle decomposition, clone-start approximation, and bound-
exhaustion gaps are closed by the structural algorithm. The residual register
does not claim a conditional-value theorem that the product value domain cannot
express.

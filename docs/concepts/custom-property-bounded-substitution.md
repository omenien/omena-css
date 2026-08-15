---
title: Custom-property bounded substitution
description: The shipped custom-property substitution algorithm, its independent all-bottom witness, and the explicit research gaps.
kind: explanation
status: stable
products: [cascade, diagnostics]
owner: cascade
sourceOfTruth: authored
---

# Custom-property bounded substitution

Omena resolves the custom-property environment with a finite, deterministic
substitution loop. This page describes the code that ships. Historical public
symbols retain proof-oriented names for compatibility, but those names do not
upgrade the implementation into a theorem.

## Shipped algorithm

The implementation receives a fixed-key `CustomPropertyEnv` and clones the
original environment into `current`. Its iteration bound is
`max(1, env.len() + 1)`. On every step it walks the original bindings and
substitutes each value against the previous `current` environment, producing a
simultaneous `next` environment.

Substitution follows `CascadeValue::Var` references recursively. A fresh
`BTreeSet<String>` records names currently visited by one substitution walk. A
revisit yields `CascadeValue::GuaranteedInvalid`; the surrounding variable may
then use its own fallback. Missing or `Unset` references use a fallback when
present and otherwise yield the same guaranteed-invalid value. A composite
containing that value also becomes guaranteed-invalid.

The loop returns when `next == current`. If equality is not observed before the
explicit bound, it returns the final bounded approximation with
`reached_fixed_point: false`. The iteration trace counts changed, settled, and
guaranteed-invalid entries. `custom_property_iteration_trace_is_monotone`
checks only that the settled-entry count does not decrease; it is not a proof
that the substitution operator is monotone on `CascadeValue`.

## Code correspondence

| Formal object                       | Shipped symbol                                            | Correspondence                                                                               |
| ----------------------------------- | --------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| Environment keys and input bindings | `CustomPropertyEnv`                                       | Fixed `BTreeMap` supplied by the caller.                                                     |
| Value syntax                        | `CascadeValue`                                            | Literal, composite, variable, CSS-wide states, indeterminate, guaranteed-invalid, and unset. |
| One substitution query              | `substitute_custom_properties`                            | Creates one visiting set and evaluates one value against an environment.                     |
| Recursive transfer                  | `substitute_custom_properties_inner`                      | Walks values and references, applies fallbacks, and detects a name revisit.                  |
| Clone-start iteration               | `compute_custom_property_env_least_fixed_point`           | Initializes `current` with `env.clone()` and computes simultaneous `next` environments.      |
| Public resolved environment         | `resolve_custom_property_env_least_fixed_point`           | Returns the bounded computation's environment.                                               |
| Equality-or-bound summary           | `summarize_custom_property_least_fixed_point`             | Exposes counts, trace, equality disposition, and per-entry results.                          |
| Cycle detector                      | `visiting`                                                | Per-substitution `BTreeSet<String>`; it is not an SCC decomposition.                         |
| Cycle/invalid value                 | `CascadeValue::GuaranteedInvalid`                         | Result of a revisited, missing-without-fallback, or invalid dependency.                      |
| Iteration observation               | `CustomPropertyLeastFixedPointIterationV0`                | Records changed, settled, and guaranteed-invalid counts.                                     |
| Settled-count check                 | `custom_property_iteration_trace_is_monotone`             | Compares adjacent settled counts only.                                                       |
| Bounded-computation disclosure      | `custom_property_bounded_fixed_point_computation_witness` | States the fixed-key, equality, bound, and cycle policy used by the product.                 |
| Bound-exhaustion bit                | `reached_fixed_point`                                     | False when equality was not observed within the explicit bound.                              |

## Frozen independent witness

The committed witness corpus does not reuse the clone-start transfer. Its
per-variable status order is `Unresolved(bottom) <= Resolved(value)`, its
environment order is pointwise, and its initial environment is all-bottom. A
step evaluates every original binding simultaneously against the previous
status environment. Reading an unresolved variable remains unresolved; a
missing or guaranteed-invalid variable evaluates its fallback when present.
At a stable point, remaining unresolved variables are projected to
`CascadeValue::GuaranteedInvalid`.

`CascadeValue` itself has no order relation in the product. The independent
witness therefore orders statuses, not values. The corpus compares the shipped
`F^n(original environment)` trajectory with the witness
`F^n(all-bottom)` trajectory and records disagreement as a finding. The frozen
six-case corpus currently produces four agreements and two findings.

| Corpus case                         | Shipped result                                          | All-bottom witness                 | Disposition |
| ----------------------------------- | ------------------------------------------------------- | ---------------------------------- | ----------- |
| `direct-literal`                    | literal `red`                                           | literal `red`                      | agreement   |
| `acyclic-alias-chain`               | both bindings become literal `red`                      | both bindings become literal `red` | agreement   |
| `missing-reference-fallback`        | literal `blue`                                          | literal `blue`                     | agreement   |
| `plain-two-cycle`                   | both guaranteed-invalid                                 | both guaranteed-invalid            | agreement   |
| `mutually-recursive-fallback-chain` | bounded output `a=blue`, `b=red`; equality not observed | both guaranteed-invalid            | **finding** |
| `cycle-through-fallback`            | both literal `green`                                    | both guaranteed-invalid            | **finding** |

The two findings are wrong-definite-species candidates for independent review.
They are not silently resolved by changing the witness toward the implementation.
The `reordered-in-place` mutation intentionally replaces the
simultaneous evaluator with an order-sensitive update and must make the harness
RED.

## rfcs#10 gap register

The committed JSON register is
`rust/omena-custom-property-fixed-point-gap-register.json`. The research track,
not the product implementation, owns any upgrade decision.

<!-- gap-register:start -->

| Gap id                     | Shipped state                                              | Claims-under-test state                                | Observable consequence                                                                                             | Upgrade cost                                                                                                             |
| -------------------------- | ---------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| `cycle-decomposition`      | Per-substitution DFS visiting set.                         | Tarjan SCC decomposition over a dependency graph.      | No component-level schedule or SCC certificate is produced.                                                        | Build and retain a dependency graph, add SCC scheduling and certificates, and differential-test the new cycle semantics. |
| `conditional-value-domain` | `CascadeValue` has no conditional `if()` variant.          | Conditional values participate in the transfer domain. | Branch-sensitive conditional dependencies cannot be represented or proved.                                         | Extend parsing and `CascadeValue`, define branch joins, and update all consumers.                                        |
| `initial-approximation`    | Iteration starts from a clone of the original environment. | Kleene iteration starts from all-bottom.               | Fallback cycles may produce definite values or exhaust the bound while the independent witness remains unresolved. | Introduce an explicit status lattice and all-bottom environment, then revalidate fallback and computed-value behavior.   |
| `termination-claim`        | Equality check or `max(1, env.len() + 1)` bound.           | A proven least solution is reached.                    | `reached_fixed_point` can be false, so the output is a bounded computation result rather than a theorem.           | Define an ordered value domain, prove convergence under an explicit bound, and validate the result independently.        |

<!-- gap-register:end -->

The register disposition for rfcs#10 is `claims-under-test-not-shipped`. It
records research gaps; it does not assert that the current crate implements
Tarjan SCCs, conditional custom-property values, all-bottom product iteration,
or a convergence proof.

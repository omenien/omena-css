## Public API evolution contract

The public enums are non-exhaustive so downstream consumers must retain a
future-variant branch. This protects variant-set growth; it does not make
variant order, payload types, method signatures, serialized labels, or
behavioral meaning compatible by itself.

| Public enum                       | Protected evolution                                                                             | Residual contract                                                                                                             |
| --------------------------------- | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `ReactiveValueV0`                 | New variants can be appended without breaking exhaustive downstream matches.                    | Derived `Ord` makes declaration order observable; variants must be appended, and payload or ordering changes remain breaking. |
| `ReactiveStateV0`                 | New availability states can be introduced.                                                      | Existing payload types and propagation semantics remain contracts.                                                            |
| `StabilizeStatusV0`               | New stabilization outcomes can be introduced; its named-field variants are also non-exhaustive. | Existing field types and the meaning of settled versus pending remain contracts.                                              |
| `ReactiveEngineErrorV0`           | New engine failures can be introduced; named-field variants are also non-exhaustive.            | Existing error meaning, field types, and display text are not protected by the enum attribute.                                |
| `DeltaFoldParityErrorV0`          | New parity failures can be introduced; named-field variants are also non-exhaustive.            | Digest meaning, field types, and display text remain contracts.                                                               |
| `ReactiveNodeKindV0`              | New static node kinds can be introduced.                                                        | Existing discriminant order and operation meaning remain contracts.                                                           |
| `ReactiveGraphBuildErrorV0`       | New construction failures can be introduced; named-field variants are also non-exhaustive.      | Existing field types and error meaning remain contracts.                                                                      |
| `ReactiveDivergenceClassV0`       | New reviewed mismatch classes can be introduced.                                                | Machine IDs, taxonomy meaning, and ordering remain contracts.                                                                 |
| `ReactiveObservationPhaseV0`      | New observation phases can be introduced.                                                       | Existing phase meaning remains a behavioral contract.                                                                         |
| `ReactiveDivergenceDispositionV0` | New review dispositions can be introduced.                                                      | Existing disposition meaning remains a behavioral contract.                                                                   |

`ReactiveDivergenceClassV0::all` returns a static slice rather than a
fixed-size array. This keeps the const, allocation-free API while removing
array length from the public signature when the taxonomy grows.

The six public structs have these construction dispositions:

| Public struct            | Construction disposition                                                                                                   |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| `ReactiveUnavailableV0`  | Public fields remain readable, but the non-exhaustive struct prevents external literals so fields may be added compatibly. |
| `EffectReceiptV0`        | Public fields remain readable, but the non-exhaustive struct prevents external literals so receipt metadata may grow.      |
| `ReactiveNodeIdV0`       | All fields are crate-private; external construction is already impossible, so no struct attribute is needed.               |
| `ReactiveGraphBuilderV0` | All fields are private; callers construct it with `new` or `Default`.                                                      |
| `ChangePolicyV0`         | All fields are private; callers use the explicit policy constructors, and no `Default` is intentionally provided.          |
| `ReactiveEngineV0`       | All fields are private; callers obtain an engine by building a graph.                                                      |

Each example below is an external-consumer contract probe. It must remain a
compile failure because an exhaustive downstream match would make adding a
variant a breaking change.

```compile_fail
use omena_reactive::ReactiveValueV0;

fn exhaustive(value: ReactiveValueV0) {
    match value {
        ReactiveValueV0::Unit => {}
        ReactiveValueV0::Bool(_) => {}
        ReactiveValueV0::Counter(_) => {}
        ReactiveValueV0::Text(_) => {}
        ReactiveValueV0::StringSet(_) => {}
        ReactiveValueV0::TextMap(_) => {}
        ReactiveValueV0::Tuple(_) => {}
        ReactiveValueV0::Digest(_) => {}
    }
}
```

```compile_fail
use omena_reactive::ReactiveStateV0;

fn exhaustive(value: ReactiveStateV0) {
    match value {
        ReactiveStateV0::Available(_) => {}
        ReactiveStateV0::Unavailable(_) => {}
    }
}
```

```compile_fail
use omena_reactive::StabilizeStatusV0;

fn exhaustive(value: StabilizeStatusV0) {
    match value {
        StabilizeStatusV0::Settled { .. } => {}
        StabilizeStatusV0::Pending { .. } => {}
    }
}
```

```compile_fail
use omena_reactive::ReactiveEngineErrorV0;

fn exhaustive(value: ReactiveEngineErrorV0) {
    match value {
        ReactiveEngineErrorV0::InvalidNode { .. } => {}
        ReactiveEngineErrorV0::NodeDoesNotAcceptDeposits { .. } => {}
        ReactiveEngineErrorV0::ObserverMutationDuringWave => {}
        ReactiveEngineErrorV0::ZeroStepBudget => {}
    }
}
```

```compile_fail
use omena_reactive::DeltaFoldParityErrorV0;

fn exhaustive(value: DeltaFoldParityErrorV0) {
    match value {
        DeltaFoldParityErrorV0::InvalidNode { .. } => {}
        DeltaFoldParityErrorV0::NotDeltaFold { .. } => {}
        DeltaFoldParityErrorV0::Diverged { .. } => {}
    }
}
```

```compile_fail
use omena_reactive::ReactiveNodeKindV0;

fn exhaustive(value: ReactiveNodeKindV0) {
    match value {
        ReactiveNodeKindV0::Input => {}
        ReactiveNodeKindV0::Map => {}
        ReactiveNodeKindV0::Zip => {}
        ReactiveNodeKindV0::Switch => {}
        ReactiveNodeKindV0::DeltaFold => {}
        ReactiveNodeKindV0::AsyncResult => {}
        ReactiveNodeKindV0::EffectBoundary => {}
    }
}
```

```compile_fail
use omena_reactive::ReactiveGraphBuildErrorV0;

fn exhaustive(value: ReactiveGraphBuildErrorV0) {
    match value {
        ReactiveGraphBuildErrorV0::EmptyChangePolicyName { .. } => {}
        ReactiveGraphBuildErrorV0::DuplicateDeltaKey { .. } => {}
        ReactiveGraphBuildErrorV0::ForeignNodeId { .. } => {}
    }
}
```

```compile_fail
use omena_reactive::ReactiveDivergenceClassV0;

fn exhaustive(value: ReactiveDivergenceClassV0) {
    match value {
        ReactiveDivergenceClassV0::FlushConeClosureTiming => {}
        ReactiveDivergenceClassV0::MidWaveReadTiming => {}
    }
}
```

```compile_fail
use omena_reactive::ReactiveObservationPhaseV0;

fn exhaustive(value: ReactiveObservationPhaseV0) {
    match value {
        ReactiveObservationPhaseV0::DuringWave => {}
        ReactiveObservationPhaseV0::Flush => {}
    }
}
```

```compile_fail
use omena_reactive::ReactiveDivergenceDispositionV0;

fn exhaustive(value: ReactiveDivergenceDispositionV0) {
    match value {
        ReactiveDivergenceDispositionV0::BenignUntilFlush => {}
        ReactiveDivergenceDispositionV0::Blocker => {}
    }
}
```

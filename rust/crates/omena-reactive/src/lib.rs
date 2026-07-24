//! Static reactive graph primitives for read-only control-plane observation.
//!
//! The graph records values and effect receipts but performs no external
//! effects. Callers retain authority over scheduling, persistence, and output.
//! Input deposits made during a wave are deferred until the next wave.
//!
//! Every node requires an explicit [`ChangePolicyV0`]. Omitting the policy is a
//! compile-time error:
//!
//! ```compile_fail
//! use omena_reactive::{ReactiveGraphBuilderV0, ReactiveStateV0, ReactiveValueV0};
//!
//! let mut graph = ReactiveGraphBuilderV0::new();
//! graph.add_input(ReactiveStateV0::available(ReactiveValueV0::Counter(1)));
//! ```

mod divergence;
mod engine;
mod graph;
mod policy;
mod value;

pub use divergence::{
    ReactiveDivergenceClassV0, ReactiveDivergenceDispositionV0, ReactiveObservationPhaseV0,
};
pub use engine::{
    DeltaFoldParityErrorV0, EffectReceiptV0, ReactiveEngineErrorV0, ReactiveEngineV0,
    StabilizeStatusV0,
};
pub use graph::{
    MapOperationV0, ReactiveGraphBuildErrorV0, ReactiveGraphBuilderV0, ReactiveNodeIdV0,
    ReactiveNodeKindV0, ZipOperationV0,
};
pub use policy::ChangePolicyV0;
pub use value::{ReactiveStateV0, ReactiveUnavailableV0, ReactiveValueV0};

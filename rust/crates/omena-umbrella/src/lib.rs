//! # omena-umbrella
//!
//! Umbrella facade for the omena reusable building blocks (R1) and composed engine
//! libraries (R2). Depend on this single crate to pull the omena public surface; each
//! member crate is re-exported under its own name, e.g. `omena_umbrella::omena_query`.
//!
//! This is a pure re-export layer (role = U): it carries NO logic of its own. Membership
//! is the R1+R2 set of the role manifest; `rust/role-boundaries` keeps it in sync.

pub use engine_input_producers;
pub use omena_abstract_value;
pub use omena_bridge;
pub use omena_cascade;
pub use omena_categorical;
pub use omena_checker;
pub use omena_ensemble;
pub use omena_incremental;
pub use omena_interner;
pub use omena_lawvere;
pub use omena_parser;
pub use omena_query;
pub use omena_query_checker_orchestrator;
pub use omena_query_core;
pub use omena_query_transform_runner;
pub use omena_refinement;
pub use omena_refinement_trait;
pub use omena_resolver;
pub use omena_rg_flow;
pub use omena_semantic;
pub use omena_sif;
pub use omena_smt;
pub use omena_spec_audit;
pub use omena_streaming_ifds;
pub use omena_syntax;
pub use omena_transform_bundle;
pub use omena_transform_cst;
pub use omena_transform_egg;
pub use omena_transform_passes;
pub use omena_transform_print;
pub use omena_transform_target;
pub use omena_variational;
pub use omena_zk_audit;
pub use omena_zk_circuit;

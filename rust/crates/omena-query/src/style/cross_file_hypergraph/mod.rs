#![cfg_attr(not(feature = "hypergraph-ifds"), allow(unused_imports))]

pub(in crate::style) use omena_cross_file_summary::{
    HypergraphClosurePath, collect_directed_graph_cycles,
    collect_hypergraph_transitive_closure_paths,
};
#[cfg(feature = "hypergraph-ifds")]
pub use omena_cross_file_summary::{
    summarize_omena_query_unified_cross_file_hypergraph,
    summarize_omena_query_unified_cross_file_scc_report,
};

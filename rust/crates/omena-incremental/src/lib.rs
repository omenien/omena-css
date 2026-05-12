//! Salsa-backed incremental computation substrate for omena-css.
//!
//! The crate owns graph snapshots, dirty-set planning, cancellation state, and
//! fuzzable consistency cases. Downstream parser, semantic, and transform
//! crates depend on these stable V0 payloads instead of reaching into Salsa
//! internals directly.

use std::collections::{BTreeMap, BTreeSet};

use salsa::Setter;
use serde::Serialize;

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(test)]
static SALSA_DIGEST_QUERY_RUNS: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
static SALSA_DEPENDENCY_QUERY_RUNS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaIncrementalBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub engine_name: &'static str,
    pub invalidation_model: &'static str,
    pub query_model: &'static str,
    pub node_identity: Vec<&'static str>,
    pub dirty_reasons: Vec<&'static str>,
    pub ready_surfaces: Vec<&'static str>,
}

pub const DEFAULT_INCREMENTAL_CANCELLATION_LIMIT: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalRevisionV0 {
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalGraphInputV0 {
    pub revision: IncrementalRevisionV0,
    pub nodes: Vec<IncrementalNodeInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalNodeInputV0 {
    pub id: String,
    pub digest: String,
    pub dependency_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub revision: IncrementalRevisionV0,
    pub nodes: Vec<IncrementalSnapshotNodeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalSnapshotNodeV0 {
    pub id: String,
    pub digest: String,
    pub dependency_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalComputationPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub revision: IncrementalRevisionV0,
    pub node_count: usize,
    pub dirty_node_count: usize,
    pub changed_input_count: usize,
    pub new_node_count: usize,
    pub removed_node_count: usize,
    pub dependency_dirty_count: usize,
    pub nodes: Vec<IncrementalComputationNodeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalComputationNodeV0 {
    pub id: String,
    pub digest: String,
    pub dependency_ids: Vec<String>,
    pub dirty: bool,
    pub reasons: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalCancellationSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub cancelled_request_count: usize,
    pub cancelled_request_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalDatabaseUpdateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub incremental_plan: IncrementalComputationPlanV0,
    pub next_snapshot: IncrementalSnapshotV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalConsistencyFuzzCaseV0 {
    pub seed: u64,
    pub node_count: usize,
    pub changed_node_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalConsistencyFuzzResultV0 {
    pub seed: u64,
    pub node_count: usize,
    pub changed_node_id: Option<String>,
    pub dirty_node_count: usize,
    pub expected_dirty_node_count: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalFuzzSeedReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub case_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<IncrementalConsistencyFuzzResultV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalCancellationRegistryV0 {
    limit: usize,
    cancelled_request_ids: BTreeSet<String>,
}

#[salsa::input(debug)]
pub struct SalsaIncrementalNodeInputV0 {
    #[returns(ref)]
    id: String,
    #[returns(ref)]
    digest: String,
    #[returns(ref)]
    dependency_ids: Vec<String>,
}

#[derive(Default)]
pub struct OmenaIncrementalDatabaseV0 {
    db: salsa::DatabaseImpl,
    node_inputs_by_id: BTreeMap<String, SalsaIncrementalNodeInputV0>,
    current_snapshot: Option<IncrementalSnapshotV0>,
}

pub fn summarize_omena_incremental_boundary() -> OmenaIncrementalBoundarySummaryV0 {
    OmenaIncrementalBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-incremental.boundary",
        engine_name: "omena-incremental",
        invalidation_model: "stableNodeId+inputDigest+dependencyPropagation",
        query_model: "salsaInput+trackedQueryFieldGranularReuse",
        node_identity: vec!["id", "digest", "dependencyIds"],
        dirty_reasons: vec![
            "newNode",
            "inputDigestChanged",
            "dependencySetChanged",
            "dependencyDirty",
        ],
        ready_surfaces: vec![
            "incrementalGraphInput",
            "incrementalSnapshot",
            "incrementalComputationPlan",
            "incrementalCancellationRegistry",
            "salsaPersistentDatabase",
            "salsaTrackedNodeSnapshotQuery",
            "salsaFieldGranularReuse",
            "salsaPlanAndSnapshotUpdate",
        ],
    }
}

pub fn snapshot_from_graph_input(input: &IncrementalGraphInputV0) -> IncrementalSnapshotV0 {
    IncrementalSnapshotV0 {
        schema_version: "0",
        product: "omena-incremental.snapshot",
        revision: input.revision,
        nodes: normalized_snapshot_nodes(input),
    }
}

pub fn plan_incremental_computation(
    input: &IncrementalGraphInputV0,
    previous: Option<&IncrementalSnapshotV0>,
) -> IncrementalComputationPlanV0 {
    let normalized_nodes = normalized_snapshot_nodes(input);
    let previous_by_id = previous
        .map(|snapshot| {
            snapshot
                .nodes
                .iter()
                .map(|node| (node.id.as_str(), node))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let current_ids = normalized_nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let removed_node_count = previous_by_id
        .keys()
        .filter(|id| !current_ids.contains(**id))
        .count();
    let mut dirty_ids = BTreeSet::<String>::new();
    let mut nodes = normalized_nodes
        .into_iter()
        .map(|node| {
            let mut reasons = Vec::new();
            match previous_by_id.get(node.id.as_str()) {
                None => reasons.push("newNode"),
                Some(previous_node) => {
                    if previous_node.digest != node.digest {
                        reasons.push("inputDigestChanged");
                    }
                    if previous_node.dependency_ids != node.dependency_ids {
                        reasons.push("dependencySetChanged");
                    }
                }
            }
            if !reasons.is_empty() {
                dirty_ids.insert(node.id.clone());
            }

            IncrementalComputationNodeV0 {
                id: node.id,
                digest: node.digest,
                dependency_ids: node.dependency_ids,
                dirty: !reasons.is_empty(),
                reasons,
            }
        })
        .collect::<Vec<_>>();

    propagate_dependency_dirty(&mut nodes, &mut dirty_ids);

    IncrementalComputationPlanV0 {
        schema_version: "0",
        product: "omena-incremental.computation-plan",
        revision: input.revision,
        node_count: nodes.len(),
        dirty_node_count: nodes.iter().filter(|node| node.dirty).count(),
        changed_input_count: nodes
            .iter()
            .filter(|node| node.reasons.contains(&"inputDigestChanged"))
            .count(),
        new_node_count: nodes
            .iter()
            .filter(|node| node.reasons.contains(&"newNode"))
            .count(),
        removed_node_count,
        dependency_dirty_count: nodes
            .iter()
            .filter(|node| node.reasons.contains(&"dependencyDirty"))
            .count(),
        nodes,
    }
}

pub fn run_incremental_consistency_fuzz_case(
    case: IncrementalConsistencyFuzzCaseV0,
) -> IncrementalConsistencyFuzzResultV0 {
    let node_count = case.node_count.clamp(1, 64);
    let previous_input = generated_incremental_fuzz_graph(case.seed, node_count, None);
    let previous_snapshot = snapshot_from_graph_input(&previous_input);
    let changed_index = case
        .changed_node_index
        .map(|index| index.min(node_count.saturating_sub(1)));
    let next_input = generated_incremental_fuzz_graph(case.seed, node_count, changed_index);
    let plan = plan_incremental_computation(&next_input, Some(&previous_snapshot));
    let changed_node_id = changed_index.map(fuzz_node_id);
    let expected_dirty_ids = changed_node_id
        .as_ref()
        .map(|changed_id| transitive_dependents(&next_input, changed_id))
        .unwrap_or_default();
    let actual_dirty_ids = plan
        .nodes
        .iter()
        .filter(|node| node.dirty)
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    let expected_dirty_node_count = expected_dirty_ids.len();
    let passed = actual_dirty_ids == expected_dirty_ids
        && plan.dirty_node_count == expected_dirty_node_count
        && plan.changed_input_count == usize::from(changed_node_id.is_some());

    IncrementalConsistencyFuzzResultV0 {
        seed: case.seed,
        node_count,
        changed_node_id,
        dirty_node_count: plan.dirty_node_count,
        expected_dirty_node_count,
        passed,
    }
}

pub fn run_incremental_fuzz_seed_corpus() -> IncrementalFuzzSeedReportV0 {
    let seeds = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233];
    let results = seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| {
            run_incremental_consistency_fuzz_case(IncrementalConsistencyFuzzCaseV0 {
                seed,
                node_count: index + 1,
                changed_node_index: if index % 4 == 0 {
                    None
                } else {
                    Some(index / 2)
                },
            })
        })
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    IncrementalFuzzSeedReportV0 {
        schema_version: "0",
        product: "omena-incremental.fuzz-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count - passed_count,
        results,
    }
}

#[salsa::tracked(returns(clone))]
pub fn summarize_salsa_incremental_node_snapshot(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> IncrementalSnapshotNodeV0 {
    IncrementalSnapshotNodeV0 {
        id: node.id(db).clone(),
        digest: node.digest(db).clone(),
        dependency_ids: normalized_ids(node.dependency_ids(db)),
    }
}

#[salsa::tracked(returns(clone))]
pub fn read_salsa_incremental_node_digest(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> String {
    #[cfg(test)]
    SALSA_DIGEST_QUERY_RUNS.fetch_add(1, Ordering::Relaxed);

    node.digest(db).clone()
}

#[salsa::tracked(returns(clone))]
pub fn read_salsa_incremental_node_dependency_ids(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> Vec<String> {
    #[cfg(test)]
    SALSA_DEPENDENCY_QUERY_RUNS.fetch_add(1, Ordering::Relaxed);

    normalized_ids(node.dependency_ids(db))
}

fn normalized_snapshot_nodes(input: &IncrementalGraphInputV0) -> Vec<IncrementalSnapshotNodeV0> {
    let mut nodes = input
        .nodes
        .iter()
        .map(|node| IncrementalSnapshotNodeV0 {
            id: node.id.clone(),
            digest: node.digest.clone(),
            dependency_ids: normalized_ids(&node.dependency_ids),
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    nodes
}

fn normalized_ids(ids: &[String]) -> Vec<String> {
    ids.iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn propagate_dependency_dirty(
    nodes: &mut [IncrementalComputationNodeV0],
    dirty_ids: &mut BTreeSet<String>,
) {
    loop {
        let mut changed = false;
        for node in nodes.iter_mut() {
            if node.dirty {
                continue;
            }
            if node
                .dependency_ids
                .iter()
                .any(|dependency_id| dirty_ids.contains(dependency_id))
            {
                node.dirty = true;
                node.reasons.push("dependencyDirty");
                dirty_ids.insert(node.id.clone());
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }
}

fn generated_incremental_fuzz_graph(
    seed: u64,
    node_count: usize,
    changed_index: Option<usize>,
) -> IncrementalGraphInputV0 {
    let mut state = seed ^ 0xa076_1d64_78bd_642f;
    let nodes = (0..node_count)
        .map(|index| {
            let id = fuzz_node_id(index);
            let mut digest_seed = fuzz_next(&mut state);
            if changed_index == Some(index) {
                digest_seed ^= 0xffff_ffff_ffff_ffff;
            }
            let dependency_ids = (0..index)
                .filter(|candidate| {
                    let divisor = ((*candidate + 2) as u64).max(2);
                    (seed + index as u64).is_multiple_of(divisor)
                })
                .map(fuzz_node_id)
                .collect::<Vec<_>>();
            IncrementalNodeInputV0 {
                id,
                digest: format!("digest-{index}-{digest_seed:016x}"),
                dependency_ids,
            }
        })
        .collect();

    IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 {
            value: 1 + if changed_index.is_some() { 1 } else { 0 },
        },
        nodes,
    }
}

fn transitive_dependents(input: &IncrementalGraphInputV0, changed_id: &str) -> BTreeSet<String> {
    let mut dirty_ids = BTreeSet::from([changed_id.to_string()]);
    let mut changed = true;
    while changed {
        changed = false;
        for node in &input.nodes {
            if dirty_ids.contains(&node.id) {
                continue;
            }
            if node
                .dependency_ids
                .iter()
                .any(|dependency_id| dirty_ids.contains(dependency_id))
            {
                changed = dirty_ids.insert(node.id.clone()) || changed;
            }
        }
    }
    dirty_ids
}

fn fuzz_node_id(index: usize) -> String {
    format!("node-{index}")
}

fn fuzz_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

impl OmenaIncrementalDatabaseV0 {
    pub fn salsa_database(&self) -> &salsa::DatabaseImpl {
        &self.db
    }

    pub fn node_input(&self, id: &str) -> Option<SalsaIncrementalNodeInputV0> {
        self.node_inputs_by_id.get(id).copied()
    }

    pub fn current_snapshot(&self) -> Option<&IncrementalSnapshotV0> {
        self.current_snapshot.as_ref()
    }

    pub fn plan_and_upsert_graph_input(
        &mut self,
        input: &IncrementalGraphInputV0,
    ) -> IncrementalDatabaseUpdateV0 {
        let incremental_plan = plan_incremental_computation(input, self.current_snapshot.as_ref());
        let next_snapshot = self.upsert_graph_input(input);
        self.current_snapshot = Some(next_snapshot.clone());

        IncrementalDatabaseUpdateV0 {
            schema_version: "0",
            product: "omena-incremental.salsa-database-update",
            incremental_plan,
            next_snapshot,
        }
    }

    pub fn upsert_graph_input(&mut self, input: &IncrementalGraphInputV0) -> IncrementalSnapshotV0 {
        let normalized_nodes = normalized_snapshot_nodes(input);
        let current_ids = normalized_nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<BTreeSet<_>>();
        self.node_inputs_by_id
            .retain(|id, _node| current_ids.contains(id.as_str()));

        for node in &normalized_nodes {
            self.upsert_node_input(node);
        }

        let nodes = self
            .node_inputs_by_id
            .values()
            .copied()
            .map(|node| summarize_salsa_incremental_node_snapshot(&self.db, node))
            .collect::<Vec<_>>();

        IncrementalSnapshotV0 {
            schema_version: "0",
            product: "omena-incremental.salsa-snapshot",
            revision: input.revision,
            nodes,
        }
    }

    fn upsert_node_input(&mut self, node: &IncrementalSnapshotNodeV0) {
        let Some(node_input) = self.node_inputs_by_id.get(node.id.as_str()).copied() else {
            let node_input = SalsaIncrementalNodeInputV0::new(
                &self.db,
                node.id.clone(),
                node.digest.clone(),
                node.dependency_ids.clone(),
            );
            self.node_inputs_by_id.insert(node.id.clone(), node_input);
            return;
        };

        if node_input.digest(&self.db).as_str() != node.digest.as_str() {
            node_input.set_digest(&mut self.db).to(node.digest.clone());
        }
        if node_input.dependency_ids(&self.db).as_slice() != node.dependency_ids.as_slice() {
            node_input
                .set_dependency_ids(&mut self.db)
                .to(node.dependency_ids.clone());
        }
    }
}

impl Default for IncrementalCancellationRegistryV0 {
    fn default() -> Self {
        Self::with_limit(DEFAULT_INCREMENTAL_CANCELLATION_LIMIT)
    }
}

impl IncrementalCancellationRegistryV0 {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            cancelled_request_ids: BTreeSet::new(),
        }
    }

    pub fn cancel(&mut self, request_id: impl Into<String>) {
        if self.cancelled_request_ids.len() >= self.limit {
            self.cancelled_request_ids.clear();
        }
        self.cancelled_request_ids.insert(request_id.into());
    }

    pub fn take_cancelled(&mut self, request_id: &str) -> bool {
        self.cancelled_request_ids.remove(request_id)
    }

    pub fn len(&self) -> usize {
        self.cancelled_request_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cancelled_request_ids.is_empty()
    }

    pub fn snapshot(&self) -> IncrementalCancellationSnapshotV0 {
        IncrementalCancellationSnapshotV0 {
            schema_version: "0",
            product: "omena-incremental.cancellation-registry",
            cancelled_request_count: self.cancelled_request_ids.len(),
            cancelled_request_ids: self.cancelled_request_ids.iter().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IncrementalCancellationRegistryV0, IncrementalGraphInputV0, IncrementalNodeInputV0,
        IncrementalRevisionV0, OmenaIncrementalDatabaseV0, SALSA_DEPENDENCY_QUERY_RUNS,
        SALSA_DIGEST_QUERY_RUNS, plan_incremental_computation,
        read_salsa_incremental_node_dependency_ids, read_salsa_incremental_node_digest,
        snapshot_from_graph_input, summarize_omena_incremental_boundary,
    };
    use std::sync::atomic::Ordering;

    #[test]
    fn summarizes_incremental_boundary() {
        let summary = summarize_omena_incremental_boundary();

        assert_eq!(summary.product, "omena-incremental.boundary");
        assert_eq!(
            summary.query_model,
            "salsaInput+trackedQueryFieldGranularReuse"
        );
        assert!(summary.dirty_reasons.contains(&"dependencyDirty"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"incrementalCancellationRegistry")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"salsaTrackedNodeSnapshotQuery")
        );
    }

    #[test]
    fn first_plan_marks_all_nodes_dirty() {
        let input = sample_input("a:v1", "b:v1", 1);
        let plan = plan_incremental_computation(&input, None);

        assert_eq!(plan.product, "omena-incremental.computation-plan");
        assert_eq!(plan.node_count, 2);
        assert_eq!(plan.dirty_node_count, 2);
        assert_eq!(plan.new_node_count, 2);
    }

    #[test]
    fn unchanged_second_plan_marks_nodes_clean() {
        let input = sample_input("a:v1", "b:v1", 1);
        let snapshot = snapshot_from_graph_input(&input);
        let next_input = sample_input("a:v1", "b:v1", 2);
        let plan = plan_incremental_computation(&next_input, Some(&snapshot));

        assert_eq!(plan.dirty_node_count, 0);
        assert_eq!(plan.changed_input_count, 0);
    }

    #[test]
    fn changed_dependency_marks_dependent_dirty() {
        let input = sample_input("a:v1", "b:v1", 1);
        let snapshot = snapshot_from_graph_input(&input);
        let next_input = sample_input("a:v2", "b:v1", 2);
        let plan = plan_incremental_computation(&next_input, Some(&snapshot));

        assert_eq!(plan.changed_input_count, 1);
        assert_eq!(plan.dependency_dirty_count, 1);
        assert_eq!(node_reasons(&plan, "a"), vec!["inputDigestChanged"]);
        assert_eq!(node_reasons(&plan, "b"), vec!["dependencyDirty"]);
    }

    #[test]
    fn fuzz_seed_corpus_preserves_incremental_dirty_set_invariants() {
        let report = super::run_incremental_fuzz_seed_corpus();

        assert_eq!(report.product, "omena-incremental.fuzz-seed-corpus");
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.passed_count, report.case_count);
        assert!(
            report
                .results
                .iter()
                .any(|result| result.changed_node_id.is_none())
        );
        assert!(
            report
                .results
                .iter()
                .any(|result| result.expected_dirty_node_count > 1)
        );
    }

    #[test]
    fn salsa_database_reuses_digest_query_when_only_dependencies_change() {
        SALSA_DIGEST_QUERY_RUNS.store(0, Ordering::Relaxed);
        SALSA_DEPENDENCY_QUERY_RUNS.store(0, Ordering::Relaxed);

        let mut db = OmenaIncrementalDatabaseV0::default();
        let input = IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: 1 },
            nodes: vec![IncrementalNodeInputV0 {
                id: "a".to_string(),
                digest: "a:v1".to_string(),
                dependency_ids: Vec::new(),
            }],
        };
        let snapshot = db.upsert_graph_input(&input);
        assert_eq!(snapshot.product, "omena-incremental.salsa-snapshot");

        let Some(node) = db.node_input("a") else {
            return;
        };
        assert_eq!(
            read_salsa_incremental_node_digest(db.salsa_database(), node),
            "a:v1"
        );
        assert_eq!(
            read_salsa_incremental_node_dependency_ids(db.salsa_database(), node),
            Vec::<String>::new()
        );
        assert_eq!(SALSA_DIGEST_QUERY_RUNS.load(Ordering::Relaxed), 1);
        assert_eq!(SALSA_DEPENDENCY_QUERY_RUNS.load(Ordering::Relaxed), 1);

        let next_input = IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: 2 },
            nodes: vec![IncrementalNodeInputV0 {
                id: "a".to_string(),
                digest: "a:v1".to_string(),
                dependency_ids: vec!["root".to_string()],
            }],
        };
        db.upsert_graph_input(&next_input);

        let Some(node) = db.node_input("a") else {
            return;
        };
        assert_eq!(
            read_salsa_incremental_node_digest(db.salsa_database(), node),
            "a:v1"
        );
        assert_eq!(
            read_salsa_incremental_node_dependency_ids(db.salsa_database(), node),
            vec!["root".to_string()]
        );
        assert_eq!(SALSA_DIGEST_QUERY_RUNS.load(Ordering::Relaxed), 1);
        assert_eq!(SALSA_DEPENDENCY_QUERY_RUNS.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn salsa_database_update_owns_plan_and_snapshot_progression() {
        let mut db = OmenaIncrementalDatabaseV0::default();
        let input = sample_input("a:v1", "b:v1", 1);
        let first = db.plan_and_upsert_graph_input(&input);

        assert_eq!(first.product, "omena-incremental.salsa-database-update");
        assert_eq!(first.incremental_plan.dirty_node_count, 2);
        assert_eq!(
            first.next_snapshot.product,
            "omena-incremental.salsa-snapshot"
        );
        assert!(db.current_snapshot().is_some());

        let unchanged = db.plan_and_upsert_graph_input(&sample_input("a:v1", "b:v1", 2));
        assert_eq!(unchanged.incremental_plan.dirty_node_count, 0);

        let changed = db.plan_and_upsert_graph_input(&sample_input("a:v2", "b:v1", 3));
        assert_eq!(changed.incremental_plan.changed_input_count, 1);
        assert_eq!(changed.incremental_plan.dependency_dirty_count, 1);
    }

    #[test]
    fn cancellation_registry_tracks_and_consumes_request_ids() {
        let mut registry = IncrementalCancellationRegistryV0::with_limit(4);

        registry.cancel("s:hover-1");

        assert_eq!(registry.len(), 1);
        assert!(registry.take_cancelled("s:hover-1"));
        assert!(!registry.take_cancelled("s:hover-1"));
        assert!(registry.is_empty());
    }

    #[test]
    fn cancellation_registry_bounds_stale_cancelled_requests() {
        let mut registry = IncrementalCancellationRegistryV0::with_limit(2);

        registry.cancel("n:1");
        registry.cancel("n:2");
        registry.cancel("n:3");

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.product, "omena-incremental.cancellation-registry");
        assert_eq!(snapshot.cancelled_request_ids, vec!["n:3"]);
    }

    fn sample_input(a_digest: &str, b_digest: &str, revision: u64) -> IncrementalGraphInputV0 {
        IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: revision },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "b".to_string(),
                    digest: b_digest.to_string(),
                    dependency_ids: vec!["a".to_string()],
                },
                IncrementalNodeInputV0 {
                    id: "a".to_string(),
                    digest: a_digest.to_string(),
                    dependency_ids: Vec::new(),
                },
            ],
        }
    }

    fn node_reasons(plan: &super::IncrementalComputationPlanV0, id: &str) -> Vec<&'static str> {
        plan.nodes
            .iter()
            .find(|node| node.id == id)
            .map(|node| node.reasons.clone())
            .unwrap_or_default()
    }
}

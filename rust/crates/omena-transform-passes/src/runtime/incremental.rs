use omena_incremental::{
    IncrementalGraphInputV0, IncrementalNodeInputV0, IncrementalRevisionV0,
    OmenaIncrementalDatabaseV0,
};
use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

use crate::{
    TransformExecutionContextV0, TransformExecutionSummaryV0,
    TransformIncrementalExecutionSummaryV0,
    execute_transform_passes_on_source_with_dialect_and_context, plan_transform_passes,
};

pub fn execute_transform_passes_incremental_with_database(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
    incremental_database: &mut OmenaIncrementalDatabaseV0,
    previous_execution: Option<&TransformExecutionSummaryV0>,
    revision: IncrementalRevisionV0,
) -> TransformIncrementalExecutionSummaryV0 {
    let incremental_input =
        transform_pass_incremental_graph_input(source, dialect, requested, context, revision);
    let update = incremental_database.plan_and_upsert_graph_input(&incremental_input);
    let reused_previous_execution =
        update.incremental_plan.dirty_node_count == 0 && previous_execution.is_some();
    let execution = match (reused_previous_execution, previous_execution) {
        (true, Some(previous_execution)) => previous_execution.clone(),
        _ => execute_transform_passes_on_source_with_dialect_and_context(
            source, dialect, requested, context,
        ),
    };

    TransformIncrementalExecutionSummaryV0 {
        schema_version: "0",
        product: "omena-transform-passes.incremental-execution",
        incremental_engine: "omena-incremental",
        query_model: "persistentSalsaDatabase+transformPassDependencyGraph",
        reuse_policy: "reuse previous transform execution when the omena-incremental plan is clean",
        reused_previous_execution,
        incremental_plan: update.incremental_plan,
        next_snapshot: update.next_snapshot,
        execution,
        ready_surfaces: vec![
            "transformSalsaQueries",
            "transformPassIncrementalGraph",
            "cleanTransformExecutionReuse",
        ],
    }
}

pub fn transform_pass_incremental_graph_input(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
    revision: IncrementalRevisionV0,
) -> IncrementalGraphInputV0 {
    let pass_plan = plan_transform_passes(requested);
    let dialect_label = transform_style_dialect_label(dialect);
    let context_digest = transform_execution_context_digest(context);
    let ordered_pass_ids = pass_plan.ordered_pass_ids.join("|");
    let mut nodes = vec![
        IncrementalNodeInputV0 {
            id: "transform:source".to_string(),
            digest: stable_transform_digest(&["source", dialect_label, source]),
            dependency_ids: Vec::new(),
        },
        IncrementalNodeInputV0 {
            id: "transform:context".to_string(),
            digest: stable_transform_digest(&["context", context_digest.as_str()]),
            dependency_ids: Vec::new(),
        },
        IncrementalNodeInputV0 {
            id: "transform:plan".to_string(),
            digest: stable_transform_digest(&["plan", ordered_pass_ids.as_str()]),
            dependency_ids: Vec::new(),
        },
    ];

    let mut previous_pass_node_id = None;
    for pass_id in pass_plan.ordered_pass_ids {
        let node_id = format!("transform:pass:{pass_id}");
        let mut dependency_ids = vec![
            "transform:source".to_string(),
            "transform:context".to_string(),
            "transform:plan".to_string(),
        ];
        if let Some(previous_pass_node_id) = previous_pass_node_id {
            dependency_ids.push(previous_pass_node_id);
        }

        nodes.push(IncrementalNodeInputV0 {
            id: node_id.clone(),
            digest: stable_transform_digest(&["pass", pass_id]),
            dependency_ids,
        });
        previous_pass_node_id = Some(node_id);
    }

    let mut execution_dependency_ids = vec![
        "transform:source".to_string(),
        "transform:context".to_string(),
        "transform:plan".to_string(),
    ];
    if let Some(previous_pass_node_id) = previous_pass_node_id {
        execution_dependency_ids.push(previous_pass_node_id);
    }
    nodes.push(IncrementalNodeInputV0 {
        id: "transform:execution".to_string(),
        digest: stable_transform_digest(&["execution", ordered_pass_ids.as_str()]),
        dependency_ids: execution_dependency_ids,
    });

    IncrementalGraphInputV0 { revision, nodes }
}

fn transform_style_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

fn transform_execution_context_digest(context: &TransformExecutionContextV0) -> String {
    let serialized = match serde_json::to_string(context) {
        Ok(serialized) => serialized,
        Err(error) => format!("serialization-error:{error}"),
    };
    stable_transform_digest(&["transform-context", serialized.as_str()])
}

fn stable_transform_digest(parts: &[&str]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

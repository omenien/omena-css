use crate::{
    io::{read_engine_input_json, read_source},
    output::print_json,
    paths::path_string,
};
use omena_query::{
    OmenaQueryEngineInputV2, OmenaQueryExpressionDomainFlowRuntimeV0, ParserPositionV0,
    read_omena_query_cascade_at_position,
    read_omena_query_cascade_at_position_with_categorical_evidence,
    read_omena_query_style_context_index,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_style_completion_at_position,
    summarize_omena_query_style_hover_candidates,
    summarize_omena_query_transform_context_from_engine_input,
};
use std::path::PathBuf;

pub(crate) fn context_from_engine_input(
    path: PathBuf,
    engine_input_json: PathBuf,
    closed_style_world: bool,
    json: bool,
) -> Result<(), String> {
    let style_path = path_string(&path);
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let summary = summarize_omena_query_transform_context_from_engine_input(
        &engine_input,
        &style_path,
        closed_style_world,
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("target: {}", summary.target_style_path);
    println!("closed style world: {}", summary.closed_style_world);
    println!("projections: {}", summary.projection_count);
    println!(
        "selected projections: {}",
        summary.selected_projection_count
    );
    println!("reachable classes: {}", summary.reachable_class_name_count);
    for class_name in &summary.context.reachable_class_names {
        println!("  {class_name}");
    }
    Ok(())
}

pub(crate) fn expression_flow(engine_input_json: PathBuf, json: bool) -> Result<(), String> {
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let summary = summarize_omena_query_expression_domain_incremental_flow_analysis(
        &engine_input,
        &mut runtime,
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("product: {}", summary.product);
    println!("revision: {}", summary.revision);
    println!("graphs: {}", summary.graph_count);
    println!("dirty graphs: {}", summary.dirty_graph_count);
    println!("reused graphs: {}", summary.reused_graph_count);
    for entry in &summary.analyses {
        println!(
            "{}\tnodes={}\tdirty={}\treused={}",
            entry.graph_id,
            entry.analysis.analysis.nodes.len(),
            entry.analysis.incremental_plan.dirty_node_count,
            entry.analysis.reused_previous_analysis
        );
    }
    Ok(())
}

pub(crate) fn selector_projection(engine_input_json: PathBuf, json: bool) -> Result<(), String> {
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let summary = summarize_omena_query_expression_domain_selector_projection(&engine_input);

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("product: {}", summary.product);
    println!("projections: {}", summary.projection_count);
    for projection in &summary.projections {
        println!(
            "{}\t{}\t{:?}\t{}",
            projection.graph_id,
            projection.node_id,
            projection.certainty,
            projection.selector_names.join(",")
        );
    }
    Ok(())
}

pub(crate) fn cascade_at_position(
    path: PathBuf,
    line: usize,
    character: usize,
    engine_input_json: Option<PathBuf>,
    categorical_evidence: bool,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let engine_input = if let Some(engine_input_path) = engine_input_json.as_deref() {
        read_engine_input_json(engine_input_path)?
    } else {
        empty_engine_input()
    };
    let position = ParserPositionV0 { line, character };
    let summary = if categorical_evidence {
        read_omena_query_cascade_at_position_with_categorical_evidence(
            &style_path,
            &source,
            &engine_input,
            position,
            true,
        )
    } else {
        read_omena_query_cascade_at_position(&style_path, &source, &engine_input, position)
    };
    let Some(summary) = summary else {
        return Err(format!(
            "failed to read cascade information for {style_path}:{line}:{character}",
        ));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.style_path);
    println!("status: {}", summary.status);
    println!(
        "reference: {}",
        summary.reference_name.as_deref().unwrap_or("-")
    );
    println!(
        "computed status: {}",
        summary
            .referenced_declaration_computed_value_status
            .unwrap_or("-")
    );
    println!(
        "computed value: {}",
        summary
            .referenced_declaration_computed_value
            .as_deref()
            .unwrap_or("-")
    );
    println!(
        "lfp status: {}",
        summary
            .reference_custom_property_fixed_point_status
            .unwrap_or("-")
    );
    println!(
        "lfp value: {}",
        summary
            .reference_custom_property_fixed_point_value
            .as_deref()
            .unwrap_or("-")
    );
    println!(
        "lfp iterations: {}",
        summary.custom_property_fixed_point_iteration_count
    );
    println!(
        "lfp guaranteed-invalid count: {}",
        summary.custom_property_fixed_point_guaranteed_invalid_count
    );
    if let Some(evidence) = summary.categorical_evidence.as_ref() {
        println!("categorical evidence: attached");
        println!("categorical endpoints: {}", evidence.endpoint_count);
        println!(
            "categorical functor accepted: {}",
            evidence
                .functor_applications
                .first()
                .map(|functor| functor.accepted)
                .unwrap_or(false)
        );
    }
    Ok(())
}

pub(crate) fn context_index(
    path: PathBuf,
    engine_input_json: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let engine_input = if let Some(engine_input_path) = engine_input_json.as_deref() {
        read_engine_input_json(engine_input_path)?
    } else {
        empty_engine_input()
    };
    let Some(summary) = read_omena_query_style_context_index(&style_path, &source, &engine_input)
    else {
        return Err(format!("failed to read context index for {style_path}"));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.style_path);
    println!("source: {}", summary.context_index_source);
    println!(
        "layer blocks: {}",
        summary.context_index.layer_index.block_layers.len()
    );
    println!(
        "layer statements: {}",
        summary.context_index.layer_index.statement_layers.len()
    );
    println!(
        "containers: {}",
        summary.context_index.container_index.containers.len()
    );
    println!("scopes: {}", summary.context_index.scope_index.scopes.len());
    println!(
        "selector context memberships: {}",
        summary.context_index.selector_context_count
    );
    Ok(())
}

pub(crate) fn style_hover_candidates(path: PathBuf, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let Some(summary) = summarize_omena_query_style_hover_candidates(&style_path, &source) else {
        return Err(format!(
            "failed to read style candidates for {}",
            path_string(&path)
        ));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {style_path}");
    println!("language: {}", summary.language);
    println!("candidates: {}", summary.candidates.len());
    for candidate in &summary.candidates {
        println!(
            "{}\t{}\t{}",
            candidate.kind, candidate.name, candidate.source
        );
    }
    Ok(())
}

pub(crate) fn style_completion(
    path: PathBuf,
    line: usize,
    character: usize,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let Some(candidates) = summarize_omena_query_style_hover_candidates(&style_path, &source)
    else {
        return Err(format!(
            "failed to read style candidates for {}",
            path_string(&path)
        ));
    };
    let summary = summarize_omena_query_style_completion_at_position(
        &style_path,
        &source,
        ParserPositionV0 { line, character },
        candidates.candidates.as_slice(),
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("context: {}", summary.context_kind);
    println!("items: {}", summary.item_count);
    for item in &summary.items {
        println!("{}\t{}\t{}", item.label, item.detail, item.source);
    }
    Ok(())
}

fn empty_engine_input() -> OmenaQueryEngineInputV2 {
    OmenaQueryEngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: Vec::new(),
    }
}

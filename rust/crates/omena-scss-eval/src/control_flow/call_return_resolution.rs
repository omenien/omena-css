use std::collections::BTreeMap;

use omena_abstract_value::{AbstractCssValueV0, join_abstract_css_values};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    abstract_css_value_kind,
    scss_metadata::reduce_static_scss_metadata_with_context,
    value_eval::{reduce_static_scss_value, static_scss_literal_truthiness},
};

use super::{
    SCSS_CALL_RETURN_RECURSION_LIMIT,
    analysis_model::{
        ScssCallBoundReturnActivity, ScssCallReturnResolutionContext,
        ScssGlobalVariableDeclaration, ScssLoopReturnResolution,
    },
    arguments::split_scss_call_arguments,
    call_resolution::{
        call_stack_depth_from, canonical_scss_callable_name, declaration_call_graph,
        scss_visible_function_declaration_exists, scss_visible_mixin_declaration_exists,
        static_scss_value_contains_function_call,
    },
    call_return_nodes::call_return_node_is_declaration,
    header_values::{
        single_static_scss_header_value_text, substitute_static_scss_header_variables,
    },
    lexical::{
        scss_global_variable_metadata_exists, static_scss_metadata_exists_call_may_need_resolution,
    },
    loop_values::{ScssControlFlowLoopContext, loop_carried_binding_frames_for_contexts},
    model::{
        OmenaScssEvalCallArgumentValueV0, OmenaScssEvalCallReturnEdgeV0,
        OmenaScssEvalCallReturnNodeV0,
    },
    return_candidates::{
        static_scss_return_abstract_value, static_scss_return_abstract_value_with_context,
    },
    symbol_candidates::scss_call_argument_value_from_text,
    tokens::{
        matching_right_paren_token_index, next_non_trivia_token_index, token_range_end,
        token_range_start,
    },
    variables::{
        canonical_scss_variable_name, insert_static_scss_binding, static_scss_binding_value,
        variable_names_in_text,
    },
};

pub(super) fn stamp_contextual_return_values(
    nodes: &mut [OmenaScssEvalCallReturnNodeV0],
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
) {
    let values = nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            (node.kind == "functionReturn").then_some((
                index,
                static_scss_return_abstract_value_with_context(
                    node.return_text.as_deref()?,
                    node.source_span_start,
                    nodes,
                    None,
                    global_variable_declarations,
                ),
            ))
        })
        .collect::<Vec<_>>();

    for (index, value) in values {
        if let Some(node) = nodes.get_mut(index) {
            node.return_value_kind = Some(abstract_css_value_kind(&value));
            node.return_value = Some(value);
        }
    }
}

pub(super) fn build_call_return_edges(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
) -> Vec<OmenaScssEvalCallReturnEdgeV0> {
    let declarations = nodes
        .iter()
        .filter(|node| call_return_node_is_declaration(node))
        .filter_map(|node| {
            Some((
                (
                    node.symbol_kind,
                    canonical_scss_callable_name(node.name.as_deref()?),
                ),
                node.node_key.clone(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();

    for node in nodes {
        match node.kind {
            "mixinInclude" | "functionCall" if node.namespace.is_none() => {
                if let Some(name) = node.name.as_deref()
                    && let Some(target_node_key) =
                        declarations.get(&(node.symbol_kind, canonical_scss_callable_name(name)))
                {
                    let recursive =
                        node.containing_declaration_node_key.as_ref() == Some(target_node_key);
                    edges.push(OmenaScssEvalCallReturnEdgeV0 {
                        source_node_key: node.node_key.clone(),
                        target_node_key: target_node_key.clone(),
                        kind: if node.kind == "mixinInclude" {
                            "mixinCall"
                        } else {
                            "functionCall"
                        },
                        recursive,
                        capped_by_recursion_cap: recursive,
                    });
                }
            }
            "functionReturn" => {
                if let Some(target_node_key) = node.containing_declaration_node_key.clone()
                    && nodes.iter().any(|candidate| {
                        candidate.node_key == target_node_key
                            && candidate.kind == "functionDeclaration"
                    })
                {
                    edges.push(OmenaScssEvalCallReturnEdgeV0 {
                        source_node_key: target_node_key,
                        target_node_key: node.node_key.clone(),
                        kind: "functionReturn",
                        recursive: false,
                        capped_by_recursion_cap: false,
                    });
                }
            }
            _ => {}
        }
    }

    edges
}

pub(super) fn stamp_call_resolved_return_values(
    nodes: &mut [OmenaScssEvalCallReturnNodeV0],
    edges: &[OmenaScssEvalCallReturnEdgeV0],
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
) {
    let call_graph = declaration_call_graph(nodes, edges);
    let resolutions = edges
        .iter()
        .filter(|edge| edge.kind == "functionCall")
        .filter_map(|edge| {
            let call_index = nodes
                .iter()
                .position(|node| node.node_key == edge.source_node_key)?;
            let value = call_resolved_return_value_for_edge(
                nodes,
                &call_graph,
                edge,
                global_variable_declarations,
            )?;
            Some((call_index, value))
        })
        .collect::<Vec<_>>();

    for (call_index, value) in resolutions {
        if let Some(node) = nodes.get_mut(call_index) {
            node.call_resolved_return_value_kind = Some(abstract_css_value_kind(&value));
            node.call_resolved_return_value = Some(value);
        }
    }
}

fn call_resolved_return_value_for_edge(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
    call_graph: &BTreeMap<String, Vec<String>>,
    edge: &OmenaScssEvalCallReturnEdgeV0,
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
) -> Option<AbstractCssValueV0> {
    if edge.capped_by_recursion_cap {
        return Some(AbstractCssValueV0::Top);
    }
    let call_node = nodes
        .iter()
        .find(|node| node.node_key == edge.source_node_key)?;
    let declaration_node = nodes
        .iter()
        .find(|node| node.node_key == edge.target_node_key)?;
    if declaration_node.kind != "functionDeclaration" || call_node.kind != "functionCall" {
        return None;
    }
    call_resolved_return_value_for_call(
        nodes,
        call_graph,
        declaration_node,
        &call_node.argument_values,
        &[],
        global_variable_declarations,
    )
}

fn call_resolved_return_value_for_call(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
    call_graph: &BTreeMap<String, Vec<String>>,
    declaration_node: &OmenaScssEvalCallReturnNodeV0,
    argument_values: &[OmenaScssEvalCallArgumentValueV0],
    active_stack: &[String],
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
) -> Option<AbstractCssValueV0> {
    if declaration_node.kind != "functionDeclaration" {
        return None;
    }
    if active_stack
        .iter()
        .any(|entry| entry == declaration_node.node_key.as_str())
    {
        return Some(AbstractCssValueV0::Top);
    }
    if call_stack_depth_from(&declaration_node.node_key.0, call_graph, &mut Vec::new())
        >= SCSS_CALL_RETURN_RECURSION_LIMIT
    {
        return Some(AbstractCssValueV0::Top);
    }
    let mut next_stack = active_stack.to_vec();
    next_stack.push(declaration_node.node_key.0.clone());
    let context = ScssCallReturnResolutionContext {
        nodes,
        call_graph,
        active_stack: &next_stack,
        global_variable_declarations,
    };
    let Some(argument_bindings) = call_bound_argument_bindings(
        declaration_node,
        argument_values,
        declaration_node.name.as_deref(),
        Some(&context),
    ) else {
        return Some(AbstractCssValueV0::Top);
    };
    let return_nodes = nodes
        .iter()
        .filter(|node| {
            node.kind == "functionReturn"
                && node.containing_declaration_node_key.as_ref() == Some(&declaration_node.node_key)
        })
        .collect::<Vec<_>>();
    if return_nodes.is_empty() {
        return None;
    }

    for node in return_nodes {
        let mut bindings = argument_bindings.clone();
        apply_call_bound_local_bindings_before(
            &mut bindings,
            declaration_node,
            node.source_span_start,
            declaration_node.name.as_deref(),
            Some(&context),
        );
        if node.return_inside_loop_control_flow {
            match call_bound_loop_return_resolution(
                node,
                bindings,
                declaration_node.name.as_deref(),
                Some(&context),
            ) {
                ScssLoopReturnResolution::Active(value) => return Some(value),
                ScssLoopReturnResolution::Inactive => continue,
                ScssLoopReturnResolution::Unknown => return Some(AbstractCssValueV0::Top),
            }
        }
        match call_bound_return_activity(node, &bindings, Some(&context)) {
            ScssCallBoundReturnActivity::Active => {}
            ScssCallBoundReturnActivity::Inactive => continue,
            ScssCallBoundReturnActivity::Unknown => return Some(AbstractCssValueV0::Top),
        }
        return Some(
            node.return_text
                .as_deref()
                .map(|text| {
                    call_bound_return_value(
                        text,
                        &bindings,
                        declaration_node.name.as_deref(),
                        Some(&context),
                        Some(node.source_span_start),
                    )
                })
                .unwrap_or(AbstractCssValueV0::Top),
        );
    }
    Some(AbstractCssValueV0::Top)
}

fn call_bound_loop_return_resolution(
    node: &OmenaScssEvalCallReturnNodeV0,
    bindings: BTreeMap<String, AbstractCssValueV0>,
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> ScssLoopReturnResolution {
    let loop_contexts = if node.return_loop_header_texts.is_empty() {
        match node.return_loop_header_text.as_deref() {
            Some(header) => vec![ScssControlFlowLoopContext {
                header_text: header.to_string(),
                body_text: None,
            }],
            None => return ScssLoopReturnResolution::Unknown,
        }
    } else {
        node.return_loop_header_texts
            .iter()
            .enumerate()
            .map(|(index, header_text)| ScssControlFlowLoopContext {
                header_text: header_text.clone(),
                body_text: node
                    .return_loop_body_texts
                    .get(index)
                    .filter(|body| !body.is_empty())
                    .cloned(),
            })
            .collect::<Vec<_>>()
    };
    let Some(return_text) = node.return_text.as_deref() else {
        return ScssLoopReturnResolution::Unknown;
    };
    let Some(frames) =
        loop_carried_binding_frames_for_contexts(loop_contexts.as_slice(), &bindings)
    else {
        return ScssLoopReturnResolution::Unknown;
    };
    if frames.is_empty() {
        return ScssLoopReturnResolution::Inactive;
    }

    let mut resolved = AbstractCssValueV0::Bottom;
    let mut active = false;
    for frame in frames {
        let mut frame_bindings = bindings.clone();
        for binding in frame {
            insert_static_scss_binding(&mut frame_bindings, binding.name.as_str(), binding.value);
        }
        match call_bound_return_activity(node, &frame_bindings, context) {
            ScssCallBoundReturnActivity::Active => {}
            ScssCallBoundReturnActivity::Inactive => continue,
            ScssCallBoundReturnActivity::Unknown => return ScssLoopReturnResolution::Unknown,
        }
        let value =
            single_variable_return_value(return_text, &frame_bindings).unwrap_or_else(|| {
                call_bound_return_value(
                    return_text,
                    &frame_bindings,
                    function_name,
                    context,
                    Some(node.source_span_start),
                )
            });
        if matches!(value, AbstractCssValueV0::Top) {
            return ScssLoopReturnResolution::Unknown;
        }
        resolved = join_abstract_css_values(&resolved, &value);
        active = true;
    }
    if active {
        ScssLoopReturnResolution::Active(resolved)
    } else {
        ScssLoopReturnResolution::Inactive
    }
}

fn single_variable_return_value(
    return_text: &str,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let return_text = return_text.trim();
    let names = variable_names_in_text(return_text);
    if names.len() != 1 || names.first().is_none_or(|name| name != return_text) {
        return None;
    }
    static_scss_binding_value(bindings, return_text).cloned()
}

fn call_bound_argument_bindings(
    declaration_node: &OmenaScssEvalCallReturnNodeV0,
    argument_values: &[OmenaScssEvalCallArgumentValueV0],
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> Option<BTreeMap<String, AbstractCssValueV0>> {
    let mut argument_texts = BTreeMap::<String, String>::new();
    let mut positional_index = 0usize;
    let mut saw_named_argument = false;
    for argument in argument_values {
        if let Some(argument_name) = argument.name.as_ref() {
            let argument_key = canonical_scss_variable_name(argument_name);
            saw_named_argument = true;
            if !declaration_node.parameter_values.iter().any(|parameter| {
                canonical_scss_variable_name(parameter.name.as_str()) == argument_key
            }) || argument_texts
                .insert(argument_key, argument.text.clone())
                .is_some()
            {
                return None;
            }
            continue;
        }

        if saw_named_argument {
            return None;
        }
        let parameter = declaration_node.parameter_values.get(positional_index)?;
        if argument_texts
            .insert(
                canonical_scss_variable_name(parameter.name.as_str()),
                argument.text.clone(),
            )
            .is_some()
        {
            return None;
        }
        positional_index += 1;
    }

    let mut bindings = BTreeMap::new();
    for parameter in &declaration_node.parameter_values {
        let value_text = argument_texts
            .remove(canonical_scss_variable_name(parameter.name.as_str()).as_str())
            .or_else(|| parameter.default_value_text.clone())?;
        let value =
            call_bound_return_value(value_text.as_str(), &bindings, function_name, context, None);
        insert_static_scss_binding(&mut bindings, parameter.name.as_str(), value);
    }
    if argument_texts.is_empty() {
        Some(bindings)
    } else {
        None
    }
}

fn apply_call_bound_local_bindings_before(
    bindings: &mut BTreeMap<String, AbstractCssValueV0>,
    declaration_node: &OmenaScssEvalCallReturnNodeV0,
    position: usize,
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) {
    for local_binding in &declaration_node.local_binding_values {
        if local_binding.source_span_start >= position
            || position < local_binding.scope_span_start
            || position >= local_binding.scope_span_end
        {
            continue;
        }
        let value = call_bound_return_value(
            local_binding.value_text.as_str(),
            bindings,
            function_name,
            context,
            Some(local_binding.source_span_start),
        );
        insert_static_scss_binding(bindings, local_binding.name.as_str(), value);
    }
}

fn call_bound_return_activity(
    node: &OmenaScssEvalCallReturnNodeV0,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> ScssCallBoundReturnActivity {
    for condition in &node.return_negated_condition_texts {
        match call_bound_condition_truthiness(condition, bindings, node.source_span_start, context)
        {
            Some(true) => return ScssCallBoundReturnActivity::Inactive,
            Some(false) => {}
            None => return ScssCallBoundReturnActivity::Unknown,
        }
    }
    match node.return_condition_text.as_deref() {
        Some(condition) => {
            match call_bound_condition_truthiness(
                condition,
                bindings,
                node.source_span_start,
                context,
            ) {
                Some(true) => ScssCallBoundReturnActivity::Active,
                Some(false) => ScssCallBoundReturnActivity::Inactive,
                None => ScssCallBoundReturnActivity::Unknown,
            }
        }
        None => ScssCallBoundReturnActivity::Active,
    }
}

fn call_bound_condition_truthiness(
    condition: &str,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
    position: usize,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> Option<bool> {
    let condition = if variable_names_in_text(condition).is_empty() {
        condition.to_string()
    } else {
        substitute_static_scss_header_variables(condition, bindings)?
    };
    let condition = match context {
        Some(context) => {
            let reduced = reduce_static_scss_metadata_with_context(
                condition.as_str(),
                |name| {
                    scss_visible_function_declaration_exists(name, position, context.nodes)
                        .then_some(true)
                },
                |name| {
                    scss_visible_mixin_declaration_exists(name, position, context.nodes)
                        .then_some(true)
                },
                |name| Some(bindings.contains_key(canonical_scss_variable_name(name).as_str())),
                |name| {
                    scss_global_variable_metadata_exists(
                        name,
                        position,
                        context.global_variable_declarations,
                    )
                },
            );
            match reduced {
                Some(reduced) => reduced,
                None if static_scss_metadata_exists_call_may_need_resolution(
                    condition.as_str(),
                ) =>
                {
                    return None;
                }
                None => condition,
            }
        }
        None => condition,
    };
    let reduced = reduce_static_scss_value(condition);
    static_scss_literal_truthiness(reduced.as_str())
}

fn call_bound_return_value(
    return_text: &str,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
    position: Option<usize>,
) -> AbstractCssValueV0 {
    if function_name.is_some_and(|name| static_scss_value_contains_function_call(return_text, name))
    {
        return AbstractCssValueV0::Top;
    }
    let value_text = if variable_names_in_text(return_text).is_empty() {
        return_text.to_string()
    } else {
        let Some(substituted) = substitute_static_scss_header_variables(return_text, bindings)
        else {
            return AbstractCssValueV0::Top;
        };
        substituted
    };
    if function_name
        .is_some_and(|name| static_scss_value_contains_function_call(value_text.as_str(), name))
    {
        return AbstractCssValueV0::Top;
    }
    let value_text = if let Some(context) = context {
        let Some(substituted) = substitute_call_bound_function_calls(value_text.as_str(), context)
        else {
            return AbstractCssValueV0::Top;
        };
        substituted
    } else {
        value_text
    };
    match (context, position) {
        (Some(context), Some(position)) => static_scss_return_abstract_value_with_context(
            value_text.as_str(),
            position,
            context.nodes,
            Some(bindings),
            context.global_variable_declarations,
        ),
        _ => static_scss_return_abstract_value(value_text.as_str()),
    }
}

fn substitute_call_bound_function_calls(
    value: &str,
    context: &ScssCallReturnResolutionContext<'_>,
) -> Option<String> {
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut replaced = false;

    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::Ident {
            index += 1;
            continue;
        }
        let canonical_call_name = canonical_scss_callable_name(token.text.as_str());
        let Some(declaration_node) = context.nodes.iter().find(|node| {
            node.kind == "functionDeclaration"
                && node
                    .name
                    .as_deref()
                    .is_some_and(|name| canonical_scss_callable_name(name) == canonical_call_name)
        }) else {
            index += 1;
            continue;
        };
        let Some(left_paren_index) = next_non_trivia_token_index(tokens, index + 1) else {
            index += 1;
            continue;
        };
        if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
            index += 1;
            continue;
        }
        let right_paren_index = matching_right_paren_token_index(tokens, left_paren_index)?;
        let call_start = token_range_start(token);
        let call_end = token_range_end(&tokens[right_paren_index]);
        let argument_source = value.get(
            token_range_end(&tokens[left_paren_index])
                ..token_range_start(&tokens[right_paren_index]),
        )?;
        let argument_values = split_scss_call_arguments(argument_source)?
            .into_iter()
            .map(|argument| scss_call_argument_value_from_text(argument.as_str()))
            .collect::<Option<Vec<_>>>()?;
        let resolved = call_resolved_return_value_for_call(
            context.nodes,
            context.call_graph,
            declaration_node,
            &argument_values,
            context.active_stack,
            context.global_variable_declarations,
        )?;
        let replacement = single_static_scss_header_value_text(&resolved)?;
        output.push_str(value.get(cursor..call_start)?);
        output.push_str(replacement);
        cursor = call_end;
        index = right_paren_index + 1;
        replaced = true;
    }

    if !replaced {
        return Some(value.to_string());
    }
    output.push_str(value.get(cursor..)?);
    Some(output)
}

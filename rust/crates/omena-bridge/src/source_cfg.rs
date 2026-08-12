use oxc_ast::ast::{
    AssignmentOperator, AssignmentTarget, BinaryOperator, BindingIdentifier, BindingPattern,
    Declaration, ExportDefaultDeclarationKind, Expression, FunctionBody, IdentifierReference,
    IfStatement, LabeledStatement, LogicalExpression, Program, Statement, SwitchCase,
    VariableDeclaration,
};
use oxc_ast_visit::Visit;
use oxc_semantic::{Scoping, SymbolId};
use oxc_span::{GetSpan, Span};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use engine_input_producers::{
    StringTypeFactsV2, TypeFactControlFlowBlockV2, TypeFactControlFlowGraphV2,
};
use omena_abstract_value::{
    ClassBoundaryEffectV0, DomClassTokenizationV0, OrderedTokenWordV0,
    external_string_type_facts_from_abstract_class_value, prefix_suffix_class_value,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceControlFlowGraphCaptureV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub binding: SourceFlowBindingRefV0,
    pub variable_name: String,
    pub reference_byte_offset: usize,
    pub snapshot: SourceFlowBlockGraphSnapshotV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFlowBindingRefV0 {
    pub symbol_ordinal: usize,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFlowBlockGraphSnapshotV0 {
    pub entry_block_id: String,
    pub blocks: Vec<SourceFlowBlockSnapshotV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFlowBlockSnapshotV0 {
    pub id: String,
    pub kind: &'static str,
    pub transfer_kind: &'static str,
    pub successor_block_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding: Option<SourceFlowBindingRefV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_ordinal: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_kind: Option<&'static str>,
    pub boundary_effect: ClassBoundaryEffectV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered_word: Option<OrderedTokenWordV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facts: Option<StringTypeFactsV2>,
}

enum SourceFlowNode<'a> {
    Assignment {
        binding: Option<SourceFlowBindingRefV0>,
        expression: Option<&'a Expression<'a>>,
    },
    Branch {
        then_nodes: Vec<SourceFlowNode<'a>>,
        else_nodes: Vec<SourceFlowNode<'a>>,
    },
    Loop {
        body_nodes: Vec<SourceFlowNode<'a>>,
    },
    Break,
    Terminate,
}

type SourceFlowBlockMetadataV0 = (
    Option<SourceFlowBindingRefV0>,
    Option<&'static str>,
    Option<StringTypeFactsV2>,
    ClassBoundaryEffectV0,
);

pub fn summarize_omena_bridge_source_control_flow_graph_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<SourceControlFlowGraphCaptureV0> {
    crate::source_syntax::summarize_source_control_flow_graph_with_semantic(
        source_path,
        source,
        source_language,
        variable_name,
        reference_byte_offset,
    )
}

pub(crate) fn summarize_source_control_flow_graph_from_program<'a>(
    program: &'a Program<'a>,
    scoping: &'a Scoping,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<SourceControlFlowGraphCaptureV0> {
    if variable_name.contains('.') {
        return None;
    }

    let reference_binding =
        reference_binding_for_offset(program, scoping, variable_name, reference_byte_offset)?;

    let container = statement_container_for_reference(&program.body, reference_byte_offset);
    let nodes = build_flow_nodes(container, scoping, reference_byte_offset);
    Some(SourceControlFlowGraphCaptureV0 {
        schema_version: "0",
        product: "omena-bridge.source-control-flow-graph",
        variable_name: reference_binding.name.clone(),
        binding: reference_binding,
        reference_byte_offset,
        snapshot: SourceFlowBlockGraphSnapshotBuilder::new(&program.body, scoping)
            .build(nodes.as_slice()),
    })
}

pub fn summarize_omena_bridge_source_type_fact_control_flow_graph_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<TypeFactControlFlowGraphV2> {
    summarize_omena_bridge_source_control_flow_graph_for_source_language(
        source_path,
        source,
        source_language,
        variable_name,
        reference_byte_offset,
    )
    .map(|capture| source_type_fact_control_flow_graph_from_snapshot(&capture.snapshot))
}

pub fn source_type_fact_control_flow_graph_from_snapshot(
    snapshot: &SourceFlowBlockGraphSnapshotV0,
) -> TypeFactControlFlowGraphV2 {
    TypeFactControlFlowGraphV2 {
        entry_block_id: snapshot.entry_block_id.clone(),
        blocks: snapshot
            .blocks
            .iter()
            .map(source_type_fact_control_flow_block_from_snapshot)
            .collect(),
    }
}

fn source_type_fact_control_flow_block_from_snapshot(
    block: &SourceFlowBlockSnapshotV0,
) -> TypeFactControlFlowBlockV2 {
    TypeFactControlFlowBlockV2 {
        id: block.id.clone(),
        kind: block.kind.to_string(),
        transfer_kind: block.transfer_kind.to_string(),
        successor_block_ids: block.successor_block_ids.clone(),
        symbol_ordinal: block.symbol_ordinal,
        variable_name: block.variable_name.clone(),
        expression_kind: block.expression_kind.map(str::to_string),
        boundary_effect: boundary_effect_wire_value(block.boundary_effect).to_string(),
        facts: block.facts.clone(),
    }
}

fn statement_container_for_reference<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    reference_byte_offset: usize,
) -> &'a oxc_allocator::Vec<'a, Statement<'a>> {
    find_function_body_statements_containing_reference(statements, reference_byte_offset)
        .unwrap_or(statements)
}

fn find_function_body_statements_containing_reference<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    reference_byte_offset: usize,
) -> Option<&'a oxc_allocator::Vec<'a, Statement<'a>>> {
    for statement in statements {
        if let Some(body) = function_body_for_statement(statement)
            && span_contains(body.span, reference_byte_offset)
        {
            return find_function_body_statements_containing_reference(
                &body.statements,
                reference_byte_offset,
            )
            .or(Some(&body.statements));
        }
    }
    None
}

fn function_body_for_statement<'a>(statement: &'a Statement<'a>) -> Option<&'a FunctionBody<'a>> {
    match statement {
        Statement::FunctionDeclaration(function) => function.body.as_deref(),
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::FunctionDeclaration(function)) = &export.declaration {
                function.body.as_deref()
            } else {
                None
            }
        }
        Statement::ExportDefaultDeclaration(export) => {
            if let ExportDefaultDeclarationKind::FunctionDeclaration(function) = &export.declaration
            {
                function.body.as_deref()
            } else {
                None
            }
        }
        _ => None,
    }
}

fn build_flow_nodes<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    scoping: &Scoping,
    reference_byte_offset: usize,
) -> Vec<SourceFlowNode<'a>> {
    let mut nodes = Vec::new();

    for statement in statements {
        if span_start(statement.span()) >= reference_byte_offset {
            break;
        }
        if matches!(statement, Statement::FunctionDeclaration(_)) {
            continue;
        }

        match statement {
            Statement::IfStatement(if_statement) => {
                let reference_location =
                    locate_reference_in_if(if_statement, reference_byte_offset);
                nodes.push(SourceFlowNode::Branch {
                    then_nodes: build_flow_nodes_for_statement(
                        &if_statement.consequent,
                        scoping,
                        branch_reference_offset(reference_location, "then", reference_byte_offset),
                    ),
                    else_nodes: if_statement
                        .alternate
                        .as_ref()
                        .map(|alternate| {
                            build_flow_nodes_for_statement(
                                alternate,
                                scoping,
                                branch_reference_offset(
                                    reference_location,
                                    "else",
                                    reference_byte_offset,
                                ),
                            )
                        })
                        .unwrap_or_default(),
                });
                if reference_location != "after" {
                    break;
                }
            }
            Statement::WhileStatement(while_statement) => {
                nodes.push(SourceFlowNode::Loop {
                    body_nodes: build_loop_body_nodes(
                        &while_statement.body,
                        scoping,
                        reference_byte_offset,
                    ),
                });
                if span_contains(while_statement.body.span(), reference_byte_offset) {
                    break;
                }
            }
            Statement::ForStatement(for_statement) => {
                nodes.push(SourceFlowNode::Loop {
                    body_nodes: build_loop_body_nodes(
                        &for_statement.body,
                        scoping,
                        reference_byte_offset,
                    ),
                });
                if span_contains(for_statement.body.span(), reference_byte_offset) {
                    break;
                }
            }
            Statement::DoWhileStatement(do_statement) => {
                nodes.push(SourceFlowNode::Loop {
                    body_nodes: build_loop_body_nodes(
                        &do_statement.body,
                        scoping,
                        reference_byte_offset,
                    ),
                });
                if span_contains(do_statement.body.span(), reference_byte_offset) {
                    break;
                }
            }
            Statement::LabeledStatement(labeled) => {
                nodes.extend(build_flow_nodes_for_labeled(
                    labeled,
                    scoping,
                    reference_byte_offset,
                ));
                if span_contains(labeled.body.span(), reference_byte_offset) {
                    break;
                }
            }
            _ if span_contains(statement.span(), reference_byte_offset) => break,
            Statement::BreakStatement(_) => {
                nodes.push(SourceFlowNode::Break);
                break;
            }
            Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => {
                nodes.push(SourceFlowNode::Terminate);
                break;
            }
            _ => nodes.extend(assignment_nodes_for_statement(statement, scoping)),
        }
    }

    nodes
}

fn build_flow_nodes_for_statement<'a>(
    statement: &'a Statement<'a>,
    scoping: &Scoping,
    reference_byte_offset: usize,
) -> Vec<SourceFlowNode<'a>> {
    match statement {
        Statement::BlockStatement(block) => {
            build_flow_nodes(&block.body, scoping, reference_byte_offset)
        }
        _ => build_flow_nodes_from_slice(
            std::slice::from_ref(statement),
            scoping,
            reference_byte_offset,
        ),
    }
}

fn build_flow_nodes_from_slice<'a>(
    statements: &'a [Statement<'a>],
    scoping: &Scoping,
    reference_byte_offset: usize,
) -> Vec<SourceFlowNode<'a>> {
    let mut nodes = Vec::new();
    for statement in statements {
        if span_start(statement.span()) >= reference_byte_offset {
            break;
        }
        if span_contains(statement.span(), reference_byte_offset) {
            break;
        }
        nodes.extend(assignment_nodes_for_statement(statement, scoping));
    }
    nodes
}

fn build_loop_body_nodes<'a>(
    body: &'a Statement<'a>,
    scoping: &Scoping,
    reference_byte_offset: usize,
) -> Vec<SourceFlowNode<'a>> {
    if span_contains(body.span(), reference_byte_offset) {
        build_flow_nodes_for_statement(body, scoping, reference_byte_offset)
    } else {
        build_flow_nodes_for_statement(body, scoping, usize::MAX)
    }
}

fn build_flow_nodes_for_labeled<'a>(
    labeled: &'a LabeledStatement<'a>,
    scoping: &Scoping,
    reference_byte_offset: usize,
) -> Vec<SourceFlowNode<'a>> {
    build_flow_nodes_for_statement(&labeled.body, scoping, reference_byte_offset)
}

fn locate_reference_in_if(
    statement: &IfStatement<'_>,
    reference_byte_offset: usize,
) -> &'static str {
    if span_contains(statement.consequent.span(), reference_byte_offset) {
        return "then";
    }
    if statement
        .alternate
        .as_ref()
        .is_some_and(|alternate| span_contains(alternate.span(), reference_byte_offset))
    {
        return "else";
    }
    "after"
}

fn branch_reference_offset(
    reference_location: &'static str,
    branch: &'static str,
    reference_byte_offset: usize,
) -> usize {
    if reference_location == branch {
        reference_byte_offset
    } else {
        usize::MAX
    }
}

fn assignment_nodes_for_statement<'a>(
    statement: &'a Statement<'a>,
    scoping: &Scoping,
) -> Vec<SourceFlowNode<'a>> {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            assignment_nodes_for_variable_declaration(declaration)
        }
        Statement::ExpressionStatement(statement) => {
            if let Expression::AssignmentExpression(assignment) = &statement.expression
                && assignment.operator == AssignmentOperator::Assign
                && let AssignmentTarget::AssignmentTargetIdentifier(identifier) = &assignment.left
            {
                return vec![SourceFlowNode::Assignment {
                    binding: binding_ref_from_reference(scoping, identifier),
                    expression: Some(&assignment.right),
                }];
            }
            Vec::new()
        }
        Statement::BlockStatement(block) => build_flow_nodes(&block.body, scoping, usize::MAX)
            .into_iter()
            .filter(|node| matches!(node, SourceFlowNode::Assignment { .. }))
            .collect(),
        _ => Vec::new(),
    }
}

fn assignment_nodes_for_variable_declaration<'a>(
    declaration: &'a VariableDeclaration<'a>,
) -> Vec<SourceFlowNode<'a>> {
    declaration
        .declarations
        .iter()
        .filter_map(|declarator| {
            binding_pattern_identifier(&declarator.id).map(|identifier| {
                SourceFlowNode::Assignment {
                    binding: binding_ref_from_binding_identifier(identifier),
                    expression: declarator.init.as_ref(),
                }
            })
        })
        .collect()
}

fn reference_binding_for_offset(
    program: &Program<'_>,
    scoping: &Scoping,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<SourceFlowBindingRefV0> {
    let mut visitor = SourceReferenceBindingFinder {
        scoping,
        variable_name,
        reference_byte_offset,
        binding: None,
    };
    visitor.visit_program(program);
    visitor.binding
}

struct SourceReferenceBindingFinder<'a> {
    scoping: &'a Scoping,
    variable_name: &'a str,
    reference_byte_offset: usize,
    binding: Option<SourceFlowBindingRefV0>,
}

impl<'a, 'ast> Visit<'ast> for SourceReferenceBindingFinder<'a> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'ast>) {
        if self.binding.is_none()
            && identifier.name.as_str() == self.variable_name
            && span_contains(identifier.span, self.reference_byte_offset)
        {
            self.binding = binding_ref_from_reference(self.scoping, identifier);
        }
    }
}

fn binding_ref_from_reference(
    scoping: &Scoping,
    identifier: &IdentifierReference<'_>,
) -> Option<SourceFlowBindingRefV0> {
    identifier
        .reference_id
        .get()
        .and_then(|reference_id| scoping.get_reference(reference_id).symbol_id())
        .map(|symbol_id| binding_ref_from_symbol(symbol_id, identifier.name.as_str()))
}

fn binding_ref_from_binding_identifier(
    identifier: &BindingIdentifier<'_>,
) -> Option<SourceFlowBindingRefV0> {
    binding_identifier_symbol_id(identifier)
        .map(|symbol_id| binding_ref_from_symbol(symbol_id, identifier.name.as_str()))
}

fn binding_identifier_symbol_id(identifier: &BindingIdentifier<'_>) -> Option<SymbolId> {
    identifier.symbol_id.get()
}

fn binding_ref_from_symbol(symbol_id: SymbolId, name: &str) -> SourceFlowBindingRefV0 {
    SourceFlowBindingRefV0 {
        symbol_ordinal: symbol_id.index(),
        name: name.to_string(),
    }
}

fn binding_pattern_identifier<'a>(
    pattern: &'a BindingPattern<'a>,
) -> Option<&'a BindingIdentifier<'a>> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier),
        _ => None,
    }
}

struct SourceFlowBlockGraphSnapshotBuilder<'a> {
    blocks: Vec<SourceFlowBlockSnapshotV0>,
    counters: BTreeMap<&'static str, usize>,
    root_statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
    scoping: &'a Scoping,
}

impl<'a> SourceFlowBlockGraphSnapshotBuilder<'a> {
    fn new(
        root_statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
        scoping: &'a Scoping,
    ) -> Self {
        Self {
            blocks: Vec::new(),
            counters: BTreeMap::new(),
            root_statements,
            scoping,
        }
    }

    fn build(mut self, nodes: &[SourceFlowNode<'_>]) -> SourceFlowBlockGraphSnapshotV0 {
        let entry_block_id = self.add_block("entry", Some("entry"), None, None);
        let tails = self.append_nodes(nodes, vec![entry_block_id], None);
        let exit_block_id = self.add_block("exit", Some("exit"), None, None);
        self.connect(tails.as_slice(), exit_block_id.as_str());
        SourceFlowBlockGraphSnapshotV0 {
            entry_block_id: "entry".to_string(),
            blocks: self.blocks,
        }
    }

    fn append_nodes(
        &mut self,
        nodes: &[SourceFlowNode<'_>],
        incoming_block_ids: Vec<String>,
        break_target_block_id: Option<&str>,
    ) -> Vec<String> {
        let mut tails = incoming_block_ids;
        for node in nodes {
            if tails.is_empty() {
                return Vec::new();
            }
            tails = self.append_node(node, tails, break_target_block_id);
        }
        tails
    }

    fn append_node(
        &mut self,
        node: &SourceFlowNode<'_>,
        incoming_block_ids: Vec<String>,
        break_target_block_id: Option<&str>,
    ) -> Vec<String> {
        match node {
            SourceFlowNode::Assignment {
                binding,
                expression,
            } => self.append_assignment(binding.as_ref(), *expression, incoming_block_ids),
            SourceFlowNode::Branch {
                then_nodes,
                else_nodes,
            } => self.append_branch(
                then_nodes,
                else_nodes,
                incoming_block_ids,
                break_target_block_id,
            ),
            SourceFlowNode::Loop { body_nodes } => self.append_loop(body_nodes, incoming_block_ids),
            SourceFlowNode::Break => {
                let break_block_id = self.add_block("break", None, None, None);
                self.connect(incoming_block_ids.as_slice(), break_block_id.as_str());
                if let Some(target) = break_target_block_id {
                    self.connect(std::slice::from_ref(&break_block_id), target);
                }
                Vec::new()
            }
            SourceFlowNode::Terminate => {
                let terminate_block_id = self.add_block("terminate", None, None, None);
                self.connect(incoming_block_ids.as_slice(), terminate_block_id.as_str());
                Vec::new()
            }
        }
    }

    fn append_assignment(
        &mut self,
        binding: Option<&SourceFlowBindingRefV0>,
        expression: Option<&Expression<'_>>,
        incoming_block_ids: Vec<String>,
    ) -> Vec<String> {
        let boundary_effect = expression
            .map(class_boundary_effect_for_expression)
            .unwrap_or_default();
        let transfer_kind = if expression.is_some_and(is_concat_expression) {
            "concatFacts"
        } else {
            "assignFacts"
        };
        let facts = expression.and_then(|expression| {
            expression_type_facts(expression, self.root_statements, self.scoping, &self.blocks)
        });
        let assignment_block_id = self.add_block(
            "assignment",
            None,
            Some(transfer_kind),
            Some((binding.cloned(), None, facts, boundary_effect)),
        );
        self.connect(incoming_block_ids.as_slice(), assignment_block_id.as_str());

        if let Some(Expression::LogicalExpression(expression)) = expression {
            return self.append_short_circuit_expression(expression, vec![assignment_block_id]);
        }

        vec![assignment_block_id]
    }

    fn append_short_circuit_expression(
        &mut self,
        expression: &LogicalExpression<'_>,
        incoming_block_ids: Vec<String>,
    ) -> Vec<String> {
        let expression_kind = logical_expression_kind(expression);
        let operand_block_id = self.add_block(
            "logicalOperand",
            None,
            None,
            Some((
                None,
                expression_kind,
                None,
                ClassBoundaryEffectV0::UnknownBoundary,
            )),
        );
        let rhs_block_id = self.add_block(
            "logicalRhs",
            None,
            None,
            Some((
                None,
                expression_kind,
                None,
                ClassBoundaryEffectV0::UnknownBoundary,
            )),
        );
        let join_block_id = self.add_block(
            "logicalJoin",
            None,
            None,
            Some((
                None,
                expression_kind,
                None,
                ClassBoundaryEffectV0::UnknownBoundary,
            )),
        );
        self.connect(incoming_block_ids.as_slice(), operand_block_id.as_str());
        self.connect(
            std::slice::from_ref(&operand_block_id),
            join_block_id.as_str(),
        );
        self.connect(
            std::slice::from_ref(&operand_block_id),
            rhs_block_id.as_str(),
        );
        self.connect(std::slice::from_ref(&rhs_block_id), join_block_id.as_str());
        vec![join_block_id]
    }

    fn append_branch(
        &mut self,
        then_nodes: &[SourceFlowNode<'_>],
        else_nodes: &[SourceFlowNode<'_>],
        incoming_block_ids: Vec<String>,
        break_target_block_id: Option<&str>,
    ) -> Vec<String> {
        let branch_block_id = self.add_block("branch", None, None, None);
        let join_block_id = self.add_block("join", None, None, None);
        self.connect(incoming_block_ids.as_slice(), branch_block_id.as_str());
        let then_tails = self.append_nodes(
            then_nodes,
            vec![branch_block_id.clone()],
            break_target_block_id,
        );
        let else_tails = if else_nodes.is_empty() {
            vec![branch_block_id]
        } else {
            self.append_nodes(else_nodes, vec![branch_block_id], break_target_block_id)
        };
        self.connect(then_tails.as_slice(), join_block_id.as_str());
        self.connect(else_tails.as_slice(), join_block_id.as_str());
        vec![join_block_id]
    }

    fn append_loop(
        &mut self,
        body_nodes: &[SourceFlowNode<'_>],
        incoming_block_ids: Vec<String>,
    ) -> Vec<String> {
        let loop_index = self.next_index("loop");
        let header_block_id = format!("loop:{loop_index}:header");
        let body_block_id = format!("loop:{loop_index}:body");
        let exit_block_id = format!("loop:{loop_index}:exit");
        self.add_block("loopHeader", Some(header_block_id.as_str()), None, None);
        self.add_block("loopBody", Some(body_block_id.as_str()), None, None);
        self.add_block("loopExit", Some(exit_block_id.as_str()), None, None);
        self.connect(incoming_block_ids.as_slice(), header_block_id.as_str());
        self.connect(
            std::slice::from_ref(&header_block_id),
            body_block_id.as_str(),
        );
        self.connect(
            std::slice::from_ref(&header_block_id),
            exit_block_id.as_str(),
        );
        let body_tails = self.append_nodes(
            body_nodes,
            vec![body_block_id],
            Some(exit_block_id.as_str()),
        );
        self.connect(body_tails.as_slice(), header_block_id.as_str());
        vec![exit_block_id]
    }

    fn add_block(
        &mut self,
        kind: &'static str,
        explicit_id: Option<&str>,
        transfer_kind: Option<&'static str>,
        metadata: Option<SourceFlowBlockMetadataV0>,
    ) -> String {
        let id = explicit_id
            .map(str::to_string)
            .unwrap_or_else(|| format!("{kind}:{}", self.next_index(kind)));
        let (binding, expression_kind, facts, boundary_effect) = metadata.unwrap_or_default();
        let ordered_word = facts.as_ref().and_then(ordered_word_for_type_facts);
        self.blocks.push(SourceFlowBlockSnapshotV0 {
            id: id.clone(),
            kind,
            transfer_kind: transfer_kind.unwrap_or_else(|| transfer_kind_for_block_kind(kind)),
            successor_block_ids: Vec::new(),
            symbol_ordinal: binding.as_ref().map(|binding| binding.symbol_ordinal),
            variable_name: binding.as_ref().map(|binding| binding.name.clone()),
            binding,
            expression_kind,
            boundary_effect,
            ordered_word,
            facts,
        });
        id
    }

    fn connect(&mut self, from_block_ids: &[String], to_block_id: &str) {
        for from_block_id in from_block_ids {
            if let Some(block) = self
                .blocks
                .iter_mut()
                .find(|candidate| candidate.id == *from_block_id)
                && !block
                    .successor_block_ids
                    .iter()
                    .any(|candidate| candidate == to_block_id)
            {
                block.successor_block_ids.push(to_block_id.to_string());
            }
        }
    }

    fn next_index(&mut self, kind: &'static str) -> usize {
        let next = self.counters.get(kind).copied().unwrap_or_default();
        self.counters.insert(kind, next + 1);
        next
    }
}

fn is_concat_expression(expression: &Expression<'_>) -> bool {
    match transparent_expression(expression) {
        Expression::BinaryExpression(expression) => expression.operator == BinaryOperator::Addition,
        Expression::TemplateLiteral(template) => !template.expressions.is_empty(),
        Expression::CallExpression(call) => {
            class_boundary_effect_for_call(call) != ClassBoundaryEffectV0::UnknownBoundary
        }
        _ => false,
    }
}

fn class_boundary_effect_for_expression(expression: &Expression<'_>) -> ClassBoundaryEffectV0 {
    match transparent_expression(expression) {
        Expression::BinaryExpression(expression)
            if expression.operator == BinaryOperator::Addition =>
        {
            boundary_effect_for_binary_concat(&expression.left, &expression.right)
        }
        Expression::TemplateLiteral(template) if !template.expressions.is_empty() => {
            let mut saw_literal_delimiter = false;
            for (index, interpolation) in template.expressions.iter().enumerate() {
                let left = template
                    .quasis
                    .get(index)
                    .and_then(|quasi| quasi.value.cooked.as_ref())
                    .map(|value| value.as_str())
                    .unwrap_or_default();
                let right = template
                    .quasis
                    .get(index + 1)
                    .and_then(|quasi| quasi.value.cooked.as_ref())
                    .map(|value| value.as_str())
                    .unwrap_or_default();
                if left
                    .chars()
                    .next_back()
                    .is_some_and(omena_abstract_value::is_dom_class_ascii_whitespace_v0)
                    || right
                        .chars()
                        .next()
                        .is_some_and(omena_abstract_value::is_dom_class_ascii_whitespace_v0)
                {
                    return ClassBoundaryEffectV0::ConcatAtTokenBoundary;
                }
                saw_literal_delimiter |= !left.is_empty() || !right.is_empty();
                if class_boundary_effect_for_expression(interpolation)
                    == ClassBoundaryEffectV0::ConcatAtTokenBoundary
                {
                    return ClassBoundaryEffectV0::ConcatAtTokenBoundary;
                }
            }
            if saw_literal_delimiter {
                ClassBoundaryEffectV0::ConcatInsideToken
            } else {
                ClassBoundaryEffectV0::UnknownBoundary
            }
        }
        Expression::CallExpression(call) => class_boundary_effect_for_call(call),
        _ => ClassBoundaryEffectV0::UnknownBoundary,
    }
}

fn boundary_effect_for_binary_concat(
    left: &Expression<'_>,
    right: &Expression<'_>,
) -> ClassBoundaryEffectV0 {
    let left_edge = expression_literal_edge(left, LiteralEdgeSide::Trailing);
    let right_edge = expression_literal_edge(right, LiteralEdgeSide::Leading);
    if matches!(left_edge, LiteralEdgeV0::Whitespace)
        || matches!(right_edge, LiteralEdgeV0::Whitespace)
    {
        ClassBoundaryEffectV0::ConcatAtTokenBoundary
    } else if matches!(left_edge, LiteralEdgeV0::NonWhitespace)
        || matches!(right_edge, LiteralEdgeV0::NonWhitespace)
    {
        ClassBoundaryEffectV0::ConcatInsideToken
    } else {
        ClassBoundaryEffectV0::UnknownBoundary
    }
}

fn class_boundary_effect_for_call(
    call: &oxc_ast::ast::CallExpression<'_>,
) -> ClassBoundaryEffectV0 {
    if matches!(
        transparent_expression(&call.callee),
        Expression::Identifier(identifier)
            if matches!(identifier.name.as_str(), "clsx" | "classnames" | "classNames")
    ) {
        return ClassBoundaryEffectV0::ConcatAtTokenBoundary;
    }
    let Expression::StaticMemberExpression(member) = transparent_expression(&call.callee) else {
        return ClassBoundaryEffectV0::UnknownBoundary;
    };
    if member.property.name.as_str() != "join" {
        return ClassBoundaryEffectV0::UnknownBoundary;
    }
    let Some(separator) = call
        .arguments
        .first()
        .and_then(argument_expression)
        .and_then(static_string_value)
    else {
        return ClassBoundaryEffectV0::UnknownBoundary;
    };
    if separator
        .chars()
        .any(omena_abstract_value::is_dom_class_ascii_whitespace_v0)
    {
        ClassBoundaryEffectV0::ConcatAtTokenBoundary
    } else {
        ClassBoundaryEffectV0::ConcatInsideToken
    }
}

#[derive(Clone, Copy)]
enum LiteralEdgeSide {
    Leading,
    Trailing,
}

#[derive(Clone, Copy)]
enum LiteralEdgeV0 {
    Empty,
    Whitespace,
    NonWhitespace,
    Unknown,
}

fn expression_literal_edge(expression: &Expression<'_>, side: LiteralEdgeSide) -> LiteralEdgeV0 {
    match transparent_expression(expression) {
        Expression::StringLiteral(literal) => literal_edge(literal.value.as_str(), side),
        Expression::TemplateLiteral(template) => {
            let value = match side {
                LiteralEdgeSide::Leading => template.quasis.first(),
                LiteralEdgeSide::Trailing => template.quasis.last(),
            }
            .and_then(|quasi| quasi.value.cooked.as_ref())
            .map(|value| value.as_str())
            .unwrap_or_default();
            literal_edge(value, side)
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            let primary = match side {
                LiteralEdgeSide::Leading => &binary.left,
                LiteralEdgeSide::Trailing => &binary.right,
            };
            let fallback = match side {
                LiteralEdgeSide::Leading => &binary.right,
                LiteralEdgeSide::Trailing => &binary.left,
            };
            match expression_literal_edge(primary, side) {
                LiteralEdgeV0::Empty => expression_literal_edge(fallback, side),
                edge => edge,
            }
        }
        _ => LiteralEdgeV0::Unknown,
    }
}

fn literal_edge(value: &str, side: LiteralEdgeSide) -> LiteralEdgeV0 {
    let character = match side {
        LiteralEdgeSide::Leading => value.chars().next(),
        LiteralEdgeSide::Trailing => value.chars().next_back(),
    };
    match character {
        None => LiteralEdgeV0::Empty,
        Some(character) if omena_abstract_value::is_dom_class_ascii_whitespace_v0(character) => {
            LiteralEdgeV0::Whitespace
        }
        Some(_) => LiteralEdgeV0::NonWhitespace,
    }
}

fn transparent_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(expression) => {
            transparent_expression(&expression.expression)
        }
        Expression::TSAsExpression(expression) => transparent_expression(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => {
            transparent_expression(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => transparent_expression(&expression.expression),
        Expression::TSNonNullExpression(expression) => {
            transparent_expression(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            transparent_expression(&expression.expression)
        }
        _ => expression,
    }
}

fn static_string_value<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match transparent_expression(expression) {
        Expression::StringLiteral(literal) => Some(literal.value.as_str()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
            .quasis
            .first()
            .and_then(|quasi| quasi.value.cooked.as_ref())
            .map(|value| value.as_str()),
        _ => None,
    }
}

fn argument_expression<'a>(argument: &'a oxc_ast::ast::Argument<'a>) -> Option<&'a Expression<'a>> {
    match argument {
        oxc_ast::ast::Argument::SpreadElement(spread) => Some(&spread.argument),
        _ => argument.as_expression(),
    }
}

fn ordered_word_for_type_facts(facts: &StringTypeFactsV2) -> Option<OrderedTokenWordV0> {
    let mut values = facts.values.as_ref()?.iter();
    let first = values.next()?;
    let DomClassTokenizationV0::Known { word, .. } =
        omena_abstract_value::tokenize_dom_class_attribute_v0(Some(first))
    else {
        return None;
    };
    values
        .all(|value| {
            matches!(
                omena_abstract_value::tokenize_dom_class_attribute_v0(Some(value)),
                DomClassTokenizationV0::Known { word: candidate, .. } if candidate == word
            )
        })
        .then_some(word)
}

fn boundary_effect_wire_value(effect: ClassBoundaryEffectV0) -> &'static str {
    match effect {
        ClassBoundaryEffectV0::ConcatInsideToken => "concatInsideToken",
        ClassBoundaryEffectV0::ConcatAtTokenBoundary => "concatAtTokenBoundary",
        ClassBoundaryEffectV0::UnknownBoundary => "unknownBoundary",
    }
}

fn logical_expression_kind(expression: &LogicalExpression<'_>) -> Option<&'static str> {
    if expression.operator.is_and() {
        Some("logicalAnd")
    } else if expression.operator.is_or() {
        Some("logicalOr")
    } else if expression.operator.is_coalesce() {
        Some("nullishCoalesce")
    } else {
        None
    }
}

struct ExpressionFactContext<'ast, 'context> {
    root_statements: &'context oxc_allocator::Vec<'ast, Statement<'ast>>,
    scoping: &'context Scoping,
    binding_facts: &'context BTreeMap<usize, Vec<StringTypeFactsV2>>,
}

fn expression_type_facts<'ast>(
    expression: &Expression<'ast>,
    root_statements: &oxc_allocator::Vec<'ast, Statement<'ast>>,
    scoping: &Scoping,
    prior_blocks: &[SourceFlowBlockSnapshotV0],
) -> Option<StringTypeFactsV2> {
    let mut binding_facts = BTreeMap::<usize, Vec<StringTypeFactsV2>>::new();
    for block in prior_blocks {
        if let (Some(symbol_ordinal), Some(facts)) = (block.symbol_ordinal, block.facts.clone()) {
            binding_facts.entry(symbol_ordinal).or_default().push(facts);
        }
    }
    let context = ExpressionFactContext {
        root_statements,
        scoping,
        binding_facts: &binding_facts,
    };
    expression_type_facts_inner(
        expression,
        &context,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
    )
}

fn expression_type_facts_inner(
    expression: &Expression<'_>,
    context: &ExpressionFactContext<'_, '_>,
    seen_functions: &mut BTreeSet<String>,
    seen_bindings: &mut BTreeSet<usize>,
) -> Option<StringTypeFactsV2> {
    match expression {
        Expression::StringLiteral(literal) => Some(exact_type_facts(literal.value.as_str())),
        Expression::Identifier(identifier) => {
            let symbol_ordinal =
                binding_ref_from_reference(context.scoping, identifier)?.symbol_ordinal;
            if !seen_bindings.insert(symbol_ordinal) {
                return None;
            }
            let facts = merge_type_facts(
                context
                    .binding_facts
                    .get(&symbol_ordinal)?
                    .iter()
                    .cloned()
                    .map(Some),
            );
            seen_bindings.remove(&symbol_ordinal);
            facts
        }
        Expression::TemplateLiteral(template) => {
            let first = template.quasis.first()?.value.cooked.as_ref()?.as_str();
            let mut facts = Some(exact_type_facts(first));
            for (index, expression) in template.expressions.iter().enumerate() {
                facts = concatenate_type_facts(
                    facts,
                    expression_type_facts_inner(expression, context, seen_functions, seen_bindings),
                );
                let suffix = template
                    .quasis
                    .get(index + 1)?
                    .value
                    .cooked
                    .as_ref()?
                    .as_str();
                facts = concatenate_type_facts(facts, Some(exact_type_facts(suffix)));
            }
            facts
        }
        Expression::ParenthesizedExpression(expression) => expression_type_facts_inner(
            &expression.expression,
            context,
            seen_functions,
            seen_bindings,
        ),
        Expression::TSAsExpression(expression) => expression_type_facts_inner(
            &expression.expression,
            context,
            seen_functions,
            seen_bindings,
        ),
        Expression::TSSatisfiesExpression(expression) => expression_type_facts_inner(
            &expression.expression,
            context,
            seen_functions,
            seen_bindings,
        ),
        Expression::TSTypeAssertion(expression) => expression_type_facts_inner(
            &expression.expression,
            context,
            seen_functions,
            seen_bindings,
        ),
        Expression::TSNonNullExpression(expression) => expression_type_facts_inner(
            &expression.expression,
            context,
            seen_functions,
            seen_bindings,
        ),
        Expression::TSInstantiationExpression(expression) => expression_type_facts_inner(
            &expression.expression,
            context,
            seen_functions,
            seen_bindings,
        ),
        Expression::ConditionalExpression(expression) => merge_type_facts([
            expression_type_facts_inner(
                &expression.consequent,
                context,
                seen_functions,
                seen_bindings,
            ),
            expression_type_facts_inner(
                &expression.alternate,
                context,
                seen_functions,
                seen_bindings,
            ),
        ]),
        Expression::LogicalExpression(expression) => {
            if expression.operator.is_and() {
                return expression_type_facts_inner(
                    &expression.right,
                    context,
                    seen_functions,
                    seen_bindings,
                );
            }
            merge_type_facts([
                expression_type_facts_inner(
                    &expression.left,
                    context,
                    seen_functions,
                    seen_bindings,
                ),
                expression_type_facts_inner(
                    &expression.right,
                    context,
                    seen_functions,
                    seen_bindings,
                ),
            ])
        }
        Expression::BinaryExpression(expression)
            if expression.operator == BinaryOperator::Addition =>
        {
            concatenate_type_facts(
                expression_type_facts_inner(
                    &expression.left,
                    context,
                    seen_functions,
                    seen_bindings,
                ),
                expression_type_facts_inner(
                    &expression.right,
                    context,
                    seen_functions,
                    seen_bindings,
                ),
            )
        }
        Expression::CallExpression(call) => {
            if let Some(facts) =
                class_boundary_call_type_facts(call, context, seen_functions, seen_bindings)
            {
                return Some(facts);
            }
            let Expression::Identifier(callee) = &call.callee else {
                return None;
            };
            function_return_type_facts(callee.name.as_str(), context, seen_functions, seen_bindings)
        }
        _ => None,
    }
}

fn class_boundary_call_type_facts(
    call: &oxc_ast::ast::CallExpression<'_>,
    context: &ExpressionFactContext<'_, '_>,
    seen_functions: &mut BTreeSet<String>,
    seen_bindings: &mut BTreeSet<usize>,
) -> Option<StringTypeFactsV2> {
    if matches!(
        transparent_expression(&call.callee),
        Expression::Identifier(identifier)
            if matches!(identifier.name.as_str(), "clsx" | "classnames" | "classNames")
    ) {
        let facts = call
            .arguments
            .iter()
            .map(|argument| {
                argument_expression(argument).and_then(|expression| {
                    expression_type_facts_inner(expression, context, seen_functions, seen_bindings)
                })
            })
            .collect::<Option<Vec<_>>>()?;
        return concatenate_type_fact_sequence(facts, " ");
    }

    let Expression::StaticMemberExpression(member) = transparent_expression(&call.callee) else {
        return None;
    };
    if member.property.name.as_str() != "join" {
        return None;
    }
    let Expression::ArrayExpression(array) = transparent_expression(&member.object) else {
        return None;
    };
    let separator = call
        .arguments
        .first()
        .and_then(argument_expression)
        .and_then(static_string_value)
        .unwrap_or(",");
    let facts = array
        .elements
        .iter()
        .map(|element| {
            let expression = match element {
                oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => &spread.argument,
                oxc_ast::ast::ArrayExpressionElement::Elision(_) => return None,
                _ => element.as_expression()?,
            };
            expression_type_facts_inner(expression, context, seen_functions, seen_bindings)
        })
        .collect::<Option<Vec<_>>>()?;
    concatenate_type_fact_sequence(facts, separator)
}

fn concatenate_type_fact_sequence(
    facts: Vec<StringTypeFactsV2>,
    separator: &str,
) -> Option<StringTypeFactsV2> {
    let mut facts = facts.into_iter();
    let mut combined = Some(facts.next()?);
    for fact in facts {
        combined = concatenate_type_facts(combined, Some(exact_type_facts(separator)));
        combined = concatenate_type_facts(combined, Some(fact));
    }
    combined
}

fn function_return_type_facts(
    function_name: &str,
    context: &ExpressionFactContext<'_, '_>,
    seen_functions: &mut BTreeSet<String>,
    seen_bindings: &mut BTreeSet<usize>,
) -> Option<StringTypeFactsV2> {
    if !seen_functions.insert(function_name.to_string()) {
        return None;
    }
    let body = context
        .root_statements
        .iter()
        .find_map(|statement| function_body_for_named_statement(statement, function_name))?;
    let facts = merge_type_facts(
        body.statements
            .iter()
            .flat_map(|statement| {
                return_type_facts_for_statement(statement, context, seen_functions, seen_bindings)
            })
            .map(Some),
    );
    seen_functions.remove(function_name);
    facts
}

fn function_body_for_named_statement<'a>(
    statement: &'a Statement<'a>,
    function_name: &str,
) -> Option<&'a FunctionBody<'a>> {
    match statement {
        Statement::FunctionDeclaration(function)
            if function
                .id
                .as_ref()
                .is_some_and(|id| id.name.as_str() == function_name) =>
        {
            function.body.as_deref()
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::FunctionDeclaration(function)) = &export.declaration
                && function
                    .id
                    .as_ref()
                    .is_some_and(|id| id.name.as_str() == function_name)
            {
                return function.body.as_deref();
            }
            None
        }
        _ => None,
    }
}

fn return_type_facts_for_statement(
    statement: &Statement<'_>,
    context: &ExpressionFactContext<'_, '_>,
    seen_functions: &mut BTreeSet<String>,
    seen_bindings: &mut BTreeSet<usize>,
) -> Vec<StringTypeFactsV2> {
    match statement {
        Statement::ReturnStatement(statement) => statement
            .argument
            .as_ref()
            .and_then(|expression| {
                expression_type_facts_inner(expression, context, seen_functions, seen_bindings)
            })
            .into_iter()
            .collect(),
        Statement::BlockStatement(block) => block
            .body
            .iter()
            .flat_map(|statement| {
                return_type_facts_for_statement(statement, context, seen_functions, seen_bindings)
            })
            .collect(),
        Statement::IfStatement(statement) => {
            let mut facts = return_type_facts_for_statement(
                &statement.consequent,
                context,
                seen_functions,
                seen_bindings,
            );
            if let Some(alternate) = &statement.alternate {
                facts.extend(return_type_facts_for_statement(
                    alternate,
                    context,
                    seen_functions,
                    seen_bindings,
                ));
            }
            facts
        }
        Statement::SwitchStatement(statement) => statement
            .cases
            .iter()
            .flat_map(|case| {
                return_type_facts_for_switch_case(case, context, seen_functions, seen_bindings)
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn return_type_facts_for_switch_case(
    case: &SwitchCase<'_>,
    context: &ExpressionFactContext<'_, '_>,
    seen_functions: &mut BTreeSet<String>,
    seen_bindings: &mut BTreeSet<usize>,
) -> Vec<StringTypeFactsV2> {
    case.consequent
        .iter()
        .flat_map(|statement| {
            return_type_facts_for_statement(statement, context, seen_functions, seen_bindings)
        })
        .collect()
}

fn merge_type_facts(
    facts: impl IntoIterator<Item = Option<StringTypeFactsV2>>,
) -> Option<StringTypeFactsV2> {
    let mut values = BTreeSet::new();
    for fact in facts {
        let fact = fact?;
        for value in finite_values_for_type_facts(&fact)? {
            values.insert(value);
        }
    }
    finite_type_facts(values)
}

fn concatenate_type_facts(
    left: Option<StringTypeFactsV2>,
    right: Option<StringTypeFactsV2>,
) -> Option<StringTypeFactsV2> {
    if left.as_ref().and_then(single_finite_value).as_deref() == Some("") {
        return right;
    }
    if right.as_ref().and_then(single_finite_value).as_deref() == Some("") {
        return left;
    }
    match (left, right) {
        (Some(left), Some(right)) => {
            if let (Some(left_values), Some(right_values)) = (
                finite_values_for_type_facts(&left),
                finite_values_for_type_facts(&right),
            ) {
                return finite_type_facts(left_values.iter().flat_map(|left| {
                    right_values
                        .iter()
                        .map(move |right| format!("{left}{right}"))
                }));
            }
            if left.constraint_kind.as_deref() == Some("prefix")
                && let Some(suffix) = single_finite_value(&right)
            {
                return Some(prefix_suffix_type_facts(
                    left.prefix.as_deref().unwrap_or_default(),
                    suffix.as_str(),
                ));
            }
            if right.constraint_kind.as_deref() == Some("suffix")
                && let Some(prefix) = single_finite_value(&left)
            {
                return Some(prefix_suffix_type_facts(
                    prefix.as_str(),
                    right.suffix.as_deref().unwrap_or_default(),
                ));
            }
            None
        }
        (Some(left), None) => finite_values_for_type_facts(&left)
            .and_then(|values| longest_common_prefix(values.as_slice()))
            .map(|prefix| {
                constrained_type_facts("prefix", Some(prefix), None, "concatUnknownRight")
            }),
        (None, Some(right)) => finite_values_for_type_facts(&right)
            .and_then(|values| longest_common_suffix(values.as_slice()))
            .map(|suffix| {
                constrained_type_facts("suffix", None, Some(suffix), "concatUnknownLeft")
            }),
        (None, None) => None,
    }
}

fn exact_type_facts(value: &str) -> StringTypeFactsV2 {
    let mut facts = empty_type_facts("exact");
    facts.values = Some(vec![value.to_string()]);
    facts
}

fn finite_type_facts(values: impl IntoIterator<Item = String>) -> Option<StringTypeFactsV2> {
    let values = values.into_iter().collect::<BTreeSet<_>>();
    if values.is_empty() {
        return None;
    }
    if values.len() == 1 {
        return values.iter().next().map(|value| exact_type_facts(value));
    }
    let mut facts = empty_type_facts("finiteSet");
    facts.values = Some(values.into_iter().collect());
    Some(facts)
}

fn prefix_suffix_type_facts(prefix: &str, suffix: &str) -> StringTypeFactsV2 {
    let mut facts = constrained_type_facts(
        "prefixSuffix",
        Some(prefix.to_string()),
        Some(suffix.to_string()),
        "concatKnownEdges",
    );
    let value = prefix_suffix_class_value(
        prefix,
        suffix,
        Some(prefix.len().saturating_add(suffix.len())),
        None,
    );
    facts.min_len = external_string_type_facts_from_abstract_class_value(&value).min_len;
    facts
}

fn constrained_type_facts(
    constraint_kind: &str,
    prefix: Option<String>,
    suffix: Option<String>,
    provenance: &str,
) -> StringTypeFactsV2 {
    let mut facts = empty_type_facts("constrained");
    facts.constraint_kind = Some(constraint_kind.to_string());
    facts.prefix = prefix;
    facts.suffix = suffix;
    facts.provenance = Some(provenance.to_string());
    facts
}

fn empty_type_facts(kind: &str) -> StringTypeFactsV2 {
    StringTypeFactsV2 {
        kind: kind.to_string(),
        values: None,
        constraint_kind: None,
        prefix: None,
        suffix: None,
        min_len: None,
        max_len: None,
        char_must: None,
        char_may: None,
        may_include_other_chars: None,
        provenance: None,
    }
}

fn finite_values_for_type_facts(facts: &StringTypeFactsV2) -> Option<Vec<String>> {
    match facts.kind.as_str() {
        "exact" | "finiteSet" => facts.values.clone(),
        _ => None,
    }
}

fn single_finite_value(facts: &StringTypeFactsV2) -> Option<String> {
    let values = finite_values_for_type_facts(facts)?;
    (values.len() == 1).then(|| values[0].clone())
}

fn longest_common_prefix(values: &[String]) -> Option<String> {
    let first = values.first()?;
    let mut prefix = first.clone();
    for value in values.iter().skip(1) {
        while !value.starts_with(prefix.as_str()) {
            prefix.pop()?;
        }
    }
    (!prefix.is_empty()).then_some(prefix)
}

fn longest_common_suffix(values: &[String]) -> Option<String> {
    let first = values.first()?;
    let mut suffix = first.clone();
    for value in values.iter().skip(1) {
        while !value.ends_with(suffix.as_str()) {
            let mut chars = suffix.chars();
            chars.next()?;
            suffix = chars.collect();
        }
    }
    (!suffix.is_empty()).then_some(suffix)
}

fn transfer_kind_for_block_kind(kind: &str) -> &'static str {
    match kind {
        "entry" => "entry",
        "assignment" => "assignFacts",
        "branch" | "logicalOperand" => "branch",
        "join" | "logicalJoin" => "join",
        "loopHeader" | "loopBody" | "loopExit" => "loop",
        "break" => "break",
        "terminate" => "terminate",
        "logicalRhs" => "assignFacts",
        "exit" => "exit",
        _ => "exit",
    }
}

fn span_contains(span: Span, byte_offset: usize) -> bool {
    span_start(span) <= byte_offset && byte_offset < span_end(span)
}

fn span_start(span: Span) -> usize {
    span.start as usize
}

fn span_end(span: Span) -> usize {
    span.end as usize
}

#[cfg(test)]
mod tests {
    use super::{
        ClassBoundaryEffectV0, prefix_suffix_type_facts,
        source_type_fact_control_flow_graph_from_snapshot,
        summarize_omena_bridge_source_control_flow_graph_for_source_language,
    };

    #[test]
    fn captures_branchy_css_module_source_cfg_shape() -> Result<(), String> {
        let source = [
            "export function Card({ enabled }: { enabled: boolean }) {",
            "  let size = \"card\";",
            "  if (enabled) {",
            "    size = \"card--active\";",
            "  }",
            "  return <div className={size} />;",
            "}",
            "",
        ]
        .join("\n");
        let Some(reference) = source.rfind("size") else {
            return Err("fixture contains size reference".to_string());
        };
        let Some(graph) = summarize_omena_bridge_source_control_flow_graph_for_source_language(
            "/fake/ws/src/Card.tsx",
            source.as_str(),
            Some("typescriptreact"),
            "size",
            reference,
        ) else {
            return Err("fixture should produce CFG".to_string());
        };

        assert_eq!(graph.product, "omena-bridge.source-control-flow-graph");
        assert_eq!(graph.snapshot.entry_block_id, "entry");
        assert_eq!(
            graph
                .snapshot
                .blocks
                .iter()
                .map(|block| block.kind)
                .collect::<Vec<_>>(),
            vec![
                "entry",
                "assignment",
                "branch",
                "join",
                "assignment",
                "exit"
            ]
        );
        assert!(
            graph
                .snapshot
                .blocks
                .iter()
                .any(|block| block.variable_name.as_deref() == Some("size"))
        );
        let symbol_ordinals = graph
            .snapshot
            .blocks
            .iter()
            .filter(|block| block.variable_name.as_deref() == Some("size"))
            .map(|block| block.symbol_ordinal)
            .collect::<Vec<_>>();
        assert!(symbol_ordinals.iter().all(Option::is_some));
        assert!(symbol_ordinals.contains(&Some(graph.binding.symbol_ordinal)));
        let type_fact_graph = source_type_fact_control_flow_graph_from_snapshot(&graph.snapshot);
        assert_eq!(
            type_fact_graph.entry_block_id,
            graph.snapshot.entry_block_id
        );
        assert_eq!(
            type_fact_graph
                .blocks
                .iter()
                .map(|block| block.kind.as_str())
                .collect::<Vec<_>>(),
            graph
                .snapshot
                .blocks
                .iter()
                .map(|block| block.kind)
                .collect::<Vec<_>>()
        );
        assert!(type_fact_graph.blocks.iter().any(|block| {
            block.symbol_ordinal == Some(graph.binding.symbol_ordinal)
                && block.variable_name.as_deref() == Some("size")
        }));
        Ok(())
    }

    #[test]
    fn captures_assignment_value_facts_for_concatenated_source_cfg() -> Result<(), String> {
        let source = [
            "export function Card(variant: string) {",
            "  const size = \"btn-\" + variant + \"-chip\";",
            "  return cx(size);",
            "}",
            "",
        ]
        .join("\n");
        let Some(reference) = source.rfind("size") else {
            return Err("fixture contains size reference".to_string());
        };
        let Some(graph) = summarize_omena_bridge_source_control_flow_graph_for_source_language(
            "/fake/ws/src/Card.tsx",
            source.as_str(),
            Some("typescriptreact"),
            "size",
            reference,
        ) else {
            return Err("fixture should produce CFG".to_string());
        };

        let Some(block) = graph
            .snapshot
            .blocks
            .iter()
            .find(|block| block.variable_name.as_deref() == Some("size"))
        else {
            return Err("size assignment block should be present".to_string());
        };
        let Some(facts) = &block.facts else {
            return Err("size assignment should carry value facts".to_string());
        };
        assert_eq!(facts.kind, "constrained");
        assert_eq!(facts.constraint_kind.as_deref(), Some("prefixSuffix"));
        assert_eq!(facts.prefix.as_deref(), Some("btn-"));
        assert_eq!(facts.suffix.as_deref(), Some("-chip"));
        assert_eq!(facts.min_len, Some(9));

        let type_fact_graph = source_type_fact_control_flow_graph_from_snapshot(&graph.snapshot);
        assert!(type_fact_graph.blocks.iter().any(|block| {
            block.variable_name.as_deref() == Some("size")
                && block
                    .facts
                    .as_ref()
                    .is_some_and(|facts| facts.constraint_kind.as_deref() == Some("prefixSuffix"))
        }));
        Ok(())
    }

    #[test]
    fn delimiter_content_drives_boundary_effects_and_ordered_words() -> Result<(), String> {
        let source = [
            "export function Card(flag: boolean) {",
            "  const token = flag ? \"large\" : \"small\";",
            "  const binary = \"btn-\" + token;",
            "  const template = `btn-${token}`;",
            "  const explicitBoundary = \"btn-\" + \" \" + token;",
            "  const joinedInside = [\"a\", \"b\"].join(\"\");",
            "  const joinedBoundary = [\"a\", \"b\"].join(\" \" );",
            "  const listed = clsx(\"a\", \"b\");",
            "  const guarded = clsx({ active: flag });",
            "  const opaque = left() + right();",
            "  return opaque;",
            "}",
            "",
        ]
        .join("\n");
        let reference = source
            .rfind("opaque")
            .ok_or_else(|| "fixture contains opaque reference".to_string())?;
        let graph = summarize_omena_bridge_source_control_flow_graph_for_source_language(
            "/fake/ws/src/Card.tsx",
            source.as_str(),
            Some("typescriptreact"),
            "opaque",
            reference,
        )
        .ok_or_else(|| "fixture should produce CFG".to_string())?;
        let block = |name: &str| {
            graph
                .snapshot
                .blocks
                .iter()
                .find(|block| block.variable_name.as_deref() == Some(name))
                .ok_or_else(|| format!("missing {name} assignment"))
        };

        let binary = block("binary")?;
        let template = block("template")?;
        assert_eq!(
            binary.boundary_effect,
            ClassBoundaryEffectV0::ConcatInsideToken
        );
        assert_eq!(template.boundary_effect, binary.boundary_effect);
        assert_eq!(template.facts, binary.facts);
        assert_eq!(
            binary.facts.as_ref().and_then(|facts| facts.values.clone()),
            Some(vec!["btn-large".to_string(), "btn-small".to_string()])
        );

        assert_eq!(
            block("explicitBoundary")?.boundary_effect,
            ClassBoundaryEffectV0::ConcatAtTokenBoundary
        );
        assert_eq!(
            block("joinedInside")?.boundary_effect,
            ClassBoundaryEffectV0::ConcatInsideToken
        );
        assert_eq!(
            block("joinedBoundary")?.boundary_effect,
            ClassBoundaryEffectV0::ConcatAtTokenBoundary
        );
        assert_eq!(
            block("listed")?.boundary_effect,
            ClassBoundaryEffectV0::ConcatAtTokenBoundary
        );
        assert_eq!(
            block("listed")?.ordered_word.as_ref().map(|word| word
                .tokens()
                .iter()
                .map(|token| token.as_str())
                .collect::<Vec<_>>()),
            Some(vec!["a", "b"])
        );
        assert_eq!(
            block("guarded")?.boundary_effect,
            ClassBoundaryEffectV0::ConcatAtTokenBoundary
        );
        assert!(block("guarded")?.ordered_word.is_none());
        assert_eq!(
            block("opaque")?.boundary_effect,
            ClassBoundaryEffectV0::UnknownBoundary
        );
        assert!(block("opaque")?.ordered_word.is_none());
        Ok(())
    }

    #[test]
    fn class_boundary_transfer_table_names_each_delimiter_authority()
    -> Result<(), Box<dyn std::error::Error>> {
        let rows: serde_json::Value =
            serde_json::from_str(include_str!("../data/class-boundary-transfer-v0.json"))?;
        let rows = rows
            .as_array()
            .ok_or_else(|| std::io::Error::other("boundary transfer table must be an array"))?;
        assert_eq!(rows.len(), 6);
        assert!(rows.iter().all(|row| row["delimiterFact"].is_string()));
        assert_eq!(rows[0]["construct"], "binaryAddition");
        assert_eq!(rows[1]["construct"], "templateInterpolation");
        assert_eq!(rows[2]["construct"], "clsxOrClassnamesArguments");
        assert_eq!(rows[3]["construct"], "arrayJoin");
        assert_eq!(rows[4]["construct"], "objectMapEntry");
        assert_eq!(rows[5]["construct"], "opaqueOperands");
        Ok(())
    }

    #[test]
    fn source_cfg_prefix_suffix_fact_preserves_explicit_concat_utf16_length() {
        let facts = prefix_suffix_type_facts("카드-", "-활성");

        assert_eq!(facts.min_len, Some(6));
        assert_eq!(facts.prefix.as_deref(), Some("카드-"));
        assert_eq!(facts.suffix.as_deref(), Some("-활성"));
    }

    #[test]
    fn captures_same_file_helper_return_facts_for_source_cfg() -> Result<(), String> {
        let source = [
            "type Status = \"idle\" | \"busy\" | \"error\";",
            "function resolveStatusClass(status: Status): string {",
            "  switch (status) {",
            "    case \"idle\": return \"state-idle\";",
            "    case \"busy\": return \"state-busy\";",
            "    case \"error\": return \"state-error\";",
            "    default: return \"state-idle\";",
            "  }",
            "}",
            "export function Card(status: Status) {",
            "  const size = resolveStatusClass(status);",
            "  return cx(size);",
            "}",
            "",
        ]
        .join("\n");
        let Some(reference) = source.rfind("size") else {
            return Err("fixture contains size reference".to_string());
        };
        let Some(graph) = summarize_omena_bridge_source_control_flow_graph_for_source_language(
            "/fake/ws/src/Card.tsx",
            source.as_str(),
            Some("typescriptreact"),
            "size",
            reference,
        ) else {
            return Err("fixture should produce CFG".to_string());
        };

        let values = graph
            .snapshot
            .blocks
            .iter()
            .find(|block| block.variable_name.as_deref() == Some("size"))
            .and_then(|block| block.facts.as_ref())
            .and_then(|facts| facts.values.clone())
            .unwrap_or_default();
        assert_eq!(values, vec!["state-busy", "state-error", "state-idle"]);
        Ok(())
    }

    #[test]
    fn source_cfg_serializes_symbol_ordinals_without_raw_symbol_ids() -> Result<(), String> {
        let source = [
            "export function Card() {",
            "  const size = \"card\";",
            "  return cx(size);",
            "}",
            "",
        ]
        .join("\n");
        let Some(reference) = source.rfind("size") else {
            return Err("fixture contains size reference".to_string());
        };
        let Some(graph) = summarize_omena_bridge_source_control_flow_graph_for_source_language(
            "/fake/ws/src/Card.tsx",
            source.as_str(),
            Some("typescriptreact"),
            "size",
            reference,
        ) else {
            return Err("fixture should produce CFG".to_string());
        };

        let value = serde_json::to_value(&graph).map_err(|error| error.to_string())?;
        assert!(value.pointer("/binding/symbolOrdinal").is_some());
        assert!(value.pointer("/snapshot/blocks/1/symbolOrdinal").is_some());
        let serialized = serde_json::to_string(&value).map_err(|error| error.to_string())?;
        assert!(!serialized.contains("SymbolId"));
        Ok(())
    }
}

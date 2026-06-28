use oxc_allocator::Allocator;
use oxc_ast::ast::{
    AssignmentOperator, AssignmentTarget, BinaryOperator, BindingPattern, Declaration,
    ExportDefaultDeclarationKind, Expression, ForStatement, FunctionBody, IfStatement,
    LabeledStatement, LogicalExpression, Statement, SwitchCase, VariableDeclaration,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::{GetSpan, Span};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::source_language::{project_source_for_language, source_type_for_language};
use engine_input_producers::{
    StringTypeFactsV2, TypeFactControlFlowBlockV2, TypeFactControlFlowGraphV2,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceControlFlowGraphCaptureV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub variable_name: String,
    pub reference_byte_offset: usize,
    pub snapshot: SourceFlowBlockGraphSnapshotV0,
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
    pub variable_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facts: Option<StringTypeFactsV2>,
}

enum SourceFlowNode<'a> {
    Assignment {
        variable_name: String,
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

pub fn summarize_omena_bridge_source_control_flow_graph_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<SourceControlFlowGraphCaptureV0> {
    if variable_name.contains('.') {
        return None;
    }

    let projected_source = project_source_for_language(source_path, source, source_language);
    let allocator = Allocator::default();
    let ParserReturn {
        program, panicked, ..
    } = Parser::new(
        &allocator,
        projected_source.as_ref(),
        source_type_for_language(source_path, source_language),
    )
    .parse();
    if panicked {
        return None;
    }

    let container = statement_container_for_reference(&program.body, reference_byte_offset);
    if has_ambiguous_declarations(container, variable_name, reference_byte_offset) {
        return None;
    }

    let nodes = build_flow_nodes(container, reference_byte_offset);
    Some(SourceControlFlowGraphCaptureV0 {
        schema_version: "0",
        product: "omena-bridge.source-control-flow-graph",
        variable_name: variable_name.to_string(),
        reference_byte_offset,
        snapshot: SourceFlowBlockGraphSnapshotBuilder::new(&program.body).build(nodes.as_slice()),
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
        variable_name: block.variable_name.clone(),
        expression_kind: block.expression_kind.map(str::to_string),
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
                        branch_reference_offset(reference_location, "then", reference_byte_offset),
                    ),
                    else_nodes: if_statement
                        .alternate
                        .as_ref()
                        .map(|alternate| {
                            build_flow_nodes_for_statement(
                                alternate,
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
                    body_nodes: build_loop_body_nodes(&while_statement.body, reference_byte_offset),
                });
                if span_contains(while_statement.body.span(), reference_byte_offset) {
                    break;
                }
            }
            Statement::ForStatement(for_statement) => {
                nodes.push(SourceFlowNode::Loop {
                    body_nodes: build_loop_body_nodes(&for_statement.body, reference_byte_offset),
                });
                if span_contains(for_statement.body.span(), reference_byte_offset) {
                    break;
                }
            }
            Statement::DoWhileStatement(do_statement) => {
                nodes.push(SourceFlowNode::Loop {
                    body_nodes: build_loop_body_nodes(&do_statement.body, reference_byte_offset),
                });
                if span_contains(do_statement.body.span(), reference_byte_offset) {
                    break;
                }
            }
            Statement::LabeledStatement(labeled) => {
                nodes.extend(build_flow_nodes_for_labeled(labeled, reference_byte_offset));
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
            _ => nodes.extend(assignment_nodes_for_statement(statement)),
        }
    }

    nodes
}

fn build_flow_nodes_for_statement<'a>(
    statement: &'a Statement<'a>,
    reference_byte_offset: usize,
) -> Vec<SourceFlowNode<'a>> {
    match statement {
        Statement::BlockStatement(block) => build_flow_nodes(&block.body, reference_byte_offset),
        _ => build_flow_nodes_from_slice(std::slice::from_ref(statement), reference_byte_offset),
    }
}

fn build_flow_nodes_from_slice<'a>(
    statements: &'a [Statement<'a>],
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
        nodes.extend(assignment_nodes_for_statement(statement));
    }
    nodes
}

fn build_loop_body_nodes<'a>(
    body: &'a Statement<'a>,
    reference_byte_offset: usize,
) -> Vec<SourceFlowNode<'a>> {
    if span_contains(body.span(), reference_byte_offset) {
        build_flow_nodes_for_statement(body, reference_byte_offset)
    } else {
        build_flow_nodes_for_statement(body, usize::MAX)
    }
}

fn build_flow_nodes_for_labeled<'a>(
    labeled: &'a LabeledStatement<'a>,
    reference_byte_offset: usize,
) -> Vec<SourceFlowNode<'a>> {
    build_flow_nodes_for_statement(&labeled.body, reference_byte_offset)
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

fn assignment_nodes_for_statement<'a>(statement: &'a Statement<'a>) -> Vec<SourceFlowNode<'a>> {
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
                    variable_name: identifier.name.as_str().to_string(),
                    expression: Some(&assignment.right),
                }];
            }
            Vec::new()
        }
        Statement::BlockStatement(block) => build_flow_nodes(&block.body, usize::MAX)
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
            binding_pattern_identifier_name(&declarator.id).map(|name| SourceFlowNode::Assignment {
                variable_name: name.to_string(),
                expression: declarator.init.as_ref(),
            })
        })
        .collect()
}

fn binding_pattern_identifier_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn has_ambiguous_declarations(
    statements: &oxc_allocator::Vec<'_, Statement<'_>>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> bool {
    let mut declarations = 0usize;
    for statement in statements {
        declarations +=
            count_declarations_before_reference(statement, variable_name, reference_byte_offset);
        if declarations > 1 {
            return true;
        }
    }
    false
}

fn count_declarations_before_reference(
    statement: &Statement<'_>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> usize {
    if span_start(statement.span()) >= reference_byte_offset {
        return 0;
    }
    match statement {
        Statement::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .filter(|declarator| {
                binding_pattern_identifier_name(&declarator.id) == Some(variable_name)
            })
            .count(),
        Statement::BlockStatement(block) => block
            .body
            .iter()
            .map(|statement| {
                count_declarations_before_reference(statement, variable_name, reference_byte_offset)
            })
            .sum(),
        Statement::IfStatement(statement) => {
            count_declarations_before_reference(
                &statement.consequent,
                variable_name,
                reference_byte_offset,
            ) + statement.alternate.as_ref().map_or(0, |alternate| {
                count_declarations_before_reference(alternate, variable_name, reference_byte_offset)
            })
        }
        Statement::WhileStatement(statement) => count_declarations_before_reference(
            &statement.body,
            variable_name,
            reference_byte_offset,
        ),
        Statement::ForStatement(statement) => {
            count_for_statement_init_declarations(statement, variable_name)
                + count_declarations_before_reference(
                    &statement.body,
                    variable_name,
                    reference_byte_offset,
                )
        }
        _ => 0,
    }
}

fn count_for_statement_init_declarations(
    statement: &ForStatement<'_>,
    variable_name: &str,
) -> usize {
    let Some(oxc_ast::ast::ForStatementInit::VariableDeclaration(declaration)) =
        statement.init.as_ref()
    else {
        return 0;
    };
    declaration
        .declarations
        .iter()
        .filter(|declarator| binding_pattern_identifier_name(&declarator.id) == Some(variable_name))
        .count()
}

struct SourceFlowBlockGraphSnapshotBuilder<'a> {
    blocks: Vec<SourceFlowBlockSnapshotV0>,
    counters: BTreeMap<&'static str, usize>,
    root_statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
}

impl<'a> SourceFlowBlockGraphSnapshotBuilder<'a> {
    fn new(root_statements: &'a oxc_allocator::Vec<'a, Statement<'a>>) -> Self {
        Self {
            blocks: Vec::new(),
            counters: BTreeMap::new(),
            root_statements,
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
                variable_name,
                expression,
            } => self.append_assignment(variable_name, *expression, incoming_block_ids),
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
        variable_name: &str,
        expression: Option<&Expression<'_>>,
        incoming_block_ids: Vec<String>,
    ) -> Vec<String> {
        let transfer_kind = if expression.is_some_and(is_concat_expression) {
            "concatFacts"
        } else {
            "assignFacts"
        };
        let assignment_block_id = self.add_block(
            "assignment",
            None,
            Some(transfer_kind),
            Some((
                variable_name.to_string(),
                None,
                expression
                    .and_then(|expression| expression_type_facts(expression, self.root_statements)),
            )),
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
            Some((String::new(), expression_kind, None)),
        );
        let rhs_block_id = self.add_block(
            "logicalRhs",
            None,
            None,
            Some((String::new(), expression_kind, None)),
        );
        let join_block_id = self.add_block(
            "logicalJoin",
            None,
            None,
            Some((String::new(), expression_kind, None)),
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
        metadata: Option<(String, Option<&'static str>, Option<StringTypeFactsV2>)>,
    ) -> String {
        let id = explicit_id
            .map(str::to_string)
            .unwrap_or_else(|| format!("{kind}:{}", self.next_index(kind)));
        let (variable_name, expression_kind, facts) = metadata.unwrap_or_default();
        self.blocks.push(SourceFlowBlockSnapshotV0 {
            id: id.clone(),
            kind,
            transfer_kind: transfer_kind.unwrap_or_else(|| transfer_kind_for_block_kind(kind)),
            successor_block_ids: Vec::new(),
            variable_name: (!variable_name.is_empty()).then_some(variable_name),
            expression_kind,
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
    matches!(
        expression,
        Expression::BinaryExpression(expression) if expression.operator == BinaryOperator::Addition
    )
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

fn expression_type_facts(
    expression: &Expression<'_>,
    root_statements: &oxc_allocator::Vec<'_, Statement<'_>>,
) -> Option<StringTypeFactsV2> {
    expression_type_facts_inner(expression, root_statements, &mut BTreeSet::new())
}

fn expression_type_facts_inner(
    expression: &Expression<'_>,
    root_statements: &oxc_allocator::Vec<'_, Statement<'_>>,
    seen_functions: &mut BTreeSet<String>,
) -> Option<StringTypeFactsV2> {
    match expression {
        Expression::StringLiteral(literal) => Some(exact_type_facts(literal.value.as_str())),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            let value = template.quasis.first()?.value.cooked.as_ref()?.as_str();
            Some(exact_type_facts(value))
        }
        Expression::ParenthesizedExpression(expression) => {
            expression_type_facts_inner(&expression.expression, root_statements, seen_functions)
        }
        Expression::TSAsExpression(expression) => {
            expression_type_facts_inner(&expression.expression, root_statements, seen_functions)
        }
        Expression::TSSatisfiesExpression(expression) => {
            expression_type_facts_inner(&expression.expression, root_statements, seen_functions)
        }
        Expression::TSTypeAssertion(expression) => {
            expression_type_facts_inner(&expression.expression, root_statements, seen_functions)
        }
        Expression::TSNonNullExpression(expression) => {
            expression_type_facts_inner(&expression.expression, root_statements, seen_functions)
        }
        Expression::TSInstantiationExpression(expression) => {
            expression_type_facts_inner(&expression.expression, root_statements, seen_functions)
        }
        Expression::ConditionalExpression(expression) => merge_type_facts([
            expression_type_facts_inner(&expression.consequent, root_statements, seen_functions),
            expression_type_facts_inner(&expression.alternate, root_statements, seen_functions),
        ]),
        Expression::LogicalExpression(expression) => {
            if expression.operator.is_and() {
                return expression_type_facts_inner(
                    &expression.right,
                    root_statements,
                    seen_functions,
                );
            }
            merge_type_facts([
                expression_type_facts_inner(&expression.left, root_statements, seen_functions),
                expression_type_facts_inner(&expression.right, root_statements, seen_functions),
            ])
        }
        Expression::BinaryExpression(expression)
            if expression.operator == BinaryOperator::Addition =>
        {
            concatenate_type_facts(
                expression_type_facts_inner(&expression.left, root_statements, seen_functions),
                expression_type_facts_inner(&expression.right, root_statements, seen_functions),
            )
        }
        Expression::CallExpression(call) => {
            let Expression::Identifier(callee) = &call.callee else {
                return None;
            };
            function_return_type_facts(callee.name.as_str(), root_statements, seen_functions)
        }
        _ => None,
    }
}

fn function_return_type_facts(
    function_name: &str,
    root_statements: &oxc_allocator::Vec<'_, Statement<'_>>,
    seen_functions: &mut BTreeSet<String>,
) -> Option<StringTypeFactsV2> {
    if !seen_functions.insert(function_name.to_string()) {
        return None;
    }
    let body = root_statements
        .iter()
        .find_map(|statement| function_body_for_named_statement(statement, function_name))?;
    let facts = merge_type_facts(
        body.statements
            .iter()
            .flat_map(|statement| {
                return_type_facts_for_statement(statement, root_statements, seen_functions)
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
    root_statements: &oxc_allocator::Vec<'_, Statement<'_>>,
    seen_functions: &mut BTreeSet<String>,
) -> Vec<StringTypeFactsV2> {
    match statement {
        Statement::ReturnStatement(statement) => statement
            .argument
            .as_ref()
            .and_then(|expression| {
                expression_type_facts_inner(expression, root_statements, seen_functions)
            })
            .into_iter()
            .collect(),
        Statement::BlockStatement(block) => block
            .body
            .iter()
            .flat_map(|statement| {
                return_type_facts_for_statement(statement, root_statements, seen_functions)
            })
            .collect(),
        Statement::IfStatement(statement) => {
            let mut facts = return_type_facts_for_statement(
                &statement.consequent,
                root_statements,
                seen_functions,
            );
            if let Some(alternate) = &statement.alternate {
                facts.extend(return_type_facts_for_statement(
                    alternate,
                    root_statements,
                    seen_functions,
                ));
            }
            facts
        }
        Statement::SwitchStatement(statement) => statement
            .cases
            .iter()
            .flat_map(|case| {
                return_type_facts_for_switch_case(case, root_statements, seen_functions)
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn return_type_facts_for_switch_case(
    case: &SwitchCase<'_>,
    root_statements: &oxc_allocator::Vec<'_, Statement<'_>>,
    seen_functions: &mut BTreeSet<String>,
) -> Vec<StringTypeFactsV2> {
    case.consequent
        .iter()
        .flat_map(|statement| {
            return_type_facts_for_statement(statement, root_statements, seen_functions)
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
    facts.min_len = Some(prefix.len() + suffix.len());
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
        assert_eq!(facts.min_len, Some("btn-".len() + "-chip".len()));

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
}

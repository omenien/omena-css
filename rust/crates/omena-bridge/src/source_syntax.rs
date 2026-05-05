use engine_style_parser::ParserByteSpanV0;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, CallExpression, ChainElement, Class,
    ClassElement, ComputedMemberExpression, ConditionalExpression, Declaration, Expression,
    JSXAttributeValue, JSXChild, JSXExpression, LogicalExpression, ObjectExpression,
    ObjectPropertyKind, ParenthesizedExpression, Program, Statement, StaticMemberExpression,
    TSAsExpression, TSNonNullExpression, TSSatisfiesExpression,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::{SourceType, Span};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSyntaxIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub imported_style_bindings: Vec<SourceImportedStyleBindingV0>,
    pub class_string_literals: Vec<ParserByteSpanV0>,
    pub style_property_accesses: Vec<SourceStylePropertyAccessFactV0>,
    pub selector_references: Vec<SourceSelectorReferenceFactV0>,
    pub type_fact_targets: Vec<SourceTypeFactTargetV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImportedStyleBindingV0 {
    pub binding: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStylePropertyAccessFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub target_style_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSelectorReferenceFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub selector_name: Option<String>,
    pub match_kind: SourceSelectorReferenceMatchKindV0,
    pub target_style_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTypeFactTargetV0 {
    pub byte_span: ParserByteSpanV0,
    pub expression_id: String,
    pub target_style_uri: Option<String>,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceSelectorReferenceMatchKindV0 {
    Exact,
    Prefix,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceStyleBindingTarget {
    binding: String,
    target_style_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassnamesBindUtilityBinding {
    binding: String,
    style_uri: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct SourceClassValue {
    exact: Vec<String>,
    prefixes: Vec<String>,
}

impl SourceClassValue {
    fn is_empty(&self) -> bool {
        self.exact.is_empty() && self.prefixes.is_empty()
    }

    fn merge(&mut self, other: SourceClassValue) {
        self.exact.extend(other.exact);
        self.prefixes.extend(other.prefixes);
        self.canonicalize();
    }

    fn canonicalize(&mut self) {
        self.exact.sort();
        self.exact.dedup();
        self.prefixes.sort();
        self.prefixes.dedup();
    }
}

type SourceReferenceDedupeKey = (
    usize,
    usize,
    Option<String>,
    SourceSelectorReferenceMatchKindV0,
);
type SourceReferenceTargetMap = BTreeMap<SourceReferenceDedupeKey, BTreeSet<Option<String>>>;

pub fn summarize_omena_bridge_source_syntax_index(
    source: &str,
    imported_style_bindings: Vec<SourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> SourceSyntaxIndexV0 {
    let imported_style_targets = imported_style_targets(imported_style_bindings.as_slice());
    let property_access_targets = property_access_style_targets(imported_style_bindings.as_slice());
    let classnames_bind_targets = collect_classnames_bind_utility_bindings(
        source,
        imported_style_targets.as_slice(),
        classnames_bind_bindings.as_slice(),
    );
    let local_class_values = collect_local_class_value_bindings(source);

    let mut index = SourceSyntaxIndexV0 {
        schema_version: "0",
        product: "omena-bridge.source-syntax-index",
        imported_style_bindings,
        class_string_literals: collect_class_name_string_literal_spans(source),
        style_property_accesses: collect_style_property_access_facts(
            source,
            property_access_targets.as_slice(),
        ),
        selector_references: Vec::new(),
        type_fact_targets: Vec::new(),
    };

    for span in &index.class_string_literals {
        push_string_literal_selector_references(
            source,
            *span,
            None,
            &mut index.selector_references,
        );
    }
    collect_class_name_expression_reference_facts(
        source,
        &local_class_values,
        &mut index.selector_references,
        &mut index.type_fact_targets,
    );
    for access in &index.style_property_accesses {
        index
            .selector_references
            .push(SourceSelectorReferenceFactV0 {
                byte_span: access.byte_span,
                selector_name: None,
                match_kind: SourceSelectorReferenceMatchKindV0::Exact,
                target_style_uri: access.target_style_uri.clone(),
            });
    }
    for binding in classnames_bind_targets {
        collect_classnames_bind_call_reference_facts(
            source,
            binding.binding.as_str(),
            Some(binding.style_uri.as_str()),
            &local_class_values,
            &mut index.selector_references,
            &mut index.type_fact_targets,
        );
    }
    canonicalize_source_selector_references(&mut index.selector_references);

    index
}

pub fn canonicalize_source_selector_references(
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    let mut targets_by_reference: SourceReferenceTargetMap = BTreeMap::new();
    for reference in references.iter() {
        targets_by_reference
            .entry((
                reference.byte_span.start,
                reference.byte_span.end,
                reference.selector_name.clone(),
                reference.match_kind,
            ))
            .or_default()
            .insert(reference.target_style_uri.clone());
    }

    let mut canonical = Vec::new();
    for ((start, end, selector_name, match_kind), targets) in targets_by_reference {
        let has_targeted_reference = targets.iter().any(Option::is_some);
        for target_style_uri in targets {
            if has_targeted_reference && target_style_uri.is_none() {
                continue;
            }
            canonical.push(SourceSelectorReferenceFactV0 {
                byte_span: ParserByteSpanV0 { start, end },
                selector_name: selector_name.clone(),
                match_kind,
                target_style_uri,
            });
        }
    }
    *references = canonical;
}

fn imported_style_targets(
    bindings: &[SourceImportedStyleBindingV0],
) -> Vec<SourceStyleBindingTarget> {
    bindings
        .iter()
        .map(|binding| SourceStyleBindingTarget {
            binding: binding.binding.clone(),
            target_style_uri: Some(binding.style_uri.clone()),
        })
        .collect()
}

fn property_access_style_targets(
    bindings: &[SourceImportedStyleBindingV0],
) -> Vec<SourceStyleBindingTarget> {
    let imported = imported_style_targets(bindings);
    if imported.is_empty() {
        vec![SourceStyleBindingTarget {
            binding: "styles".to_string(),
            target_style_uri: None,
        }]
    } else {
        imported
    }
}

fn collect_class_name_string_literal_spans(source: &str) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        if identifier.text != "className" {
            continue;
        }
        let equals_offset = skip_js_trivia(source, identifier.end);
        if source.as_bytes().get(equals_offset) != Some(&b'=') {
            continue;
        }
        let value_offset = skip_js_trivia(source, equals_offset + 1);
        match source.as_bytes().get(value_offset).copied() {
            Some(b'\'' | b'"' | b'`') => {
                if let Some((literal_start, literal_end, next_offset)) =
                    js_string_literal_span(source, value_offset, source.len())
                {
                    spans.push(ParserByteSpanV0 {
                        start: literal_start,
                        end: literal_end,
                    });
                    cursor = next_offset;
                }
            }
            Some(b'{') => {}
            _ => {}
        }
    }
    spans
}

fn collect_style_property_access_facts(
    source: &str,
    targets: &[SourceStyleBindingTarget],
) -> Vec<SourceStylePropertyAccessFactV0> {
    let allocator = Allocator::default();
    let ParserReturn {
        program, panicked, ..
    } = Parser::new(&allocator, source, SourceType::tsx()).parse();
    if panicked {
        return Vec::new();
    }

    let mut collector = StylePropertyAccessAstCollector {
        source,
        targets,
        facts: Vec::new(),
    };
    collector.collect_program(&program);
    collector.canonicalize();
    collector.facts
}

struct StylePropertyAccessAstCollector<'a> {
    source: &'a str,
    targets: &'a [SourceStyleBindingTarget],
    facts: Vec<SourceStylePropertyAccessFactV0>,
}

impl<'a> StylePropertyAccessAstCollector<'a> {
    fn collect_program(&mut self, program: &Program<'a>) {
        for statement in &program.body {
            self.collect_statement(statement);
        }
    }

    fn collect_statement(&mut self, statement: &Statement<'a>) {
        match statement {
            Statement::BlockStatement(statement) => {
                for statement in &statement.body {
                    self.collect_statement(statement);
                }
            }
            Statement::ExpressionStatement(statement) => {
                self.collect_expression(&statement.expression);
            }
            Statement::ReturnStatement(statement) => {
                if let Some(argument) = &statement.argument {
                    self.collect_expression(argument);
                }
            }
            Statement::IfStatement(statement) => {
                self.collect_expression(&statement.test);
                self.collect_statement(&statement.consequent);
                if let Some(alternate) = &statement.alternate {
                    self.collect_statement(alternate);
                }
            }
            Statement::ForStatement(statement) => {
                if let Some(init) = &statement.init {
                    self.collect_for_statement_init(init);
                }
                if let Some(test) = &statement.test {
                    self.collect_expression(test);
                }
                if let Some(update) = &statement.update {
                    self.collect_expression(update);
                }
                self.collect_statement(&statement.body);
            }
            Statement::ForInStatement(statement) => {
                self.collect_expression(&statement.right);
                self.collect_statement(&statement.body);
            }
            Statement::ForOfStatement(statement) => {
                self.collect_expression(&statement.right);
                self.collect_statement(&statement.body);
            }
            Statement::WhileStatement(statement) => {
                self.collect_expression(&statement.test);
                self.collect_statement(&statement.body);
            }
            Statement::DoWhileStatement(statement) => {
                self.collect_statement(&statement.body);
                self.collect_expression(&statement.test);
            }
            Statement::SwitchStatement(statement) => {
                self.collect_expression(&statement.discriminant);
                for switch_case in &statement.cases {
                    if let Some(test) = &switch_case.test {
                        self.collect_expression(test);
                    }
                    for consequent in &switch_case.consequent {
                        self.collect_statement(consequent);
                    }
                }
            }
            Statement::ThrowStatement(statement) => {
                self.collect_expression(&statement.argument);
            }
            Statement::TryStatement(statement) => {
                for statement in &statement.block.body {
                    self.collect_statement(statement);
                }
                if let Some(handler) = &statement.handler {
                    for statement in &handler.body.body {
                        self.collect_statement(statement);
                    }
                }
                if let Some(finalizer) = &statement.finalizer {
                    for statement in &finalizer.body {
                        self.collect_statement(statement);
                    }
                }
            }
            Statement::VariableDeclaration(declaration) => {
                self.collect_variable_declaration(declaration);
            }
            Statement::FunctionDeclaration(function) => {
                self.collect_function_body(function.body.as_deref());
            }
            Statement::ClassDeclaration(class) => {
                self.collect_class(class);
            }
            Statement::ExportNamedDeclaration(declaration) => {
                if let Some(declaration) = &declaration.declaration {
                    self.collect_declaration(declaration);
                }
            }
            Statement::ExportDefaultDeclaration(declaration) => {
                self.collect_export_default_declaration(&declaration.declaration);
            }
            Statement::TSExportAssignment(declaration) => {
                self.collect_expression(&declaration.expression);
            }
            _ => {}
        }
    }

    fn collect_declaration(&mut self, declaration: &Declaration<'a>) {
        match declaration {
            Declaration::VariableDeclaration(declaration) => {
                self.collect_variable_declaration(declaration);
            }
            Declaration::FunctionDeclaration(function) => {
                self.collect_function_body(function.body.as_deref());
            }
            Declaration::ClassDeclaration(class) => {
                self.collect_class(class);
            }
            _ => {}
        }
    }

    fn collect_export_default_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportDefaultDeclarationKind<'a>,
    ) {
        match declaration {
            oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                self.collect_function_body(function.body.as_deref());
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                self.collect_class(class);
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::StaticMemberExpression(member) => {
                self.collect_static_member_expression(member);
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::ComputedMemberExpression(member) => {
                self.collect_computed_member_expression(member);
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::CallExpression(expression) => {
                self.collect_call_expression(expression);
            }
            _ => {}
        }
    }

    fn collect_for_statement_init(&mut self, init: &oxc_ast::ast::ForStatementInit<'a>) {
        match init {
            oxc_ast::ast::ForStatementInit::VariableDeclaration(declaration) => {
                self.collect_variable_declaration(declaration);
            }
            oxc_ast::ast::ForStatementInit::StaticMemberExpression(member) => {
                self.collect_static_member_expression(member);
            }
            oxc_ast::ast::ForStatementInit::ComputedMemberExpression(member) => {
                self.collect_computed_member_expression(member);
            }
            oxc_ast::ast::ForStatementInit::CallExpression(expression) => {
                self.collect_call_expression(expression);
            }
            _ => {}
        }
    }

    fn collect_variable_declaration(
        &mut self,
        declaration: &oxc_ast::ast::VariableDeclaration<'a>,
    ) {
        for declarator in &declaration.declarations {
            if let Some(init) = &declarator.init {
                self.collect_expression(init);
            }
        }
    }

    fn collect_function_body(&mut self, body: Option<&oxc_ast::ast::FunctionBody<'a>>) {
        let Some(body) = body else {
            return;
        };
        for statement in &body.statements {
            self.collect_statement(statement);
        }
    }

    fn collect_class(&mut self, class: &Class<'a>) {
        if let Some(super_class) = &class.super_class {
            self.collect_expression(super_class);
        }
        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) => {
                    self.collect_function_body(method.value.body.as_deref());
                }
                ClassElement::PropertyDefinition(property) => {
                    if property.computed {
                        self.collect_property_key(&property.key);
                    }
                    if let Some(value) = &property.value {
                        self.collect_expression(value);
                    }
                }
                ClassElement::AccessorProperty(property) => {
                    if property.computed {
                        self.collect_property_key(&property.key);
                    }
                    if let Some(value) = &property.value {
                        self.collect_expression(value);
                    }
                }
                ClassElement::StaticBlock(block) => {
                    for statement in &block.body {
                        self.collect_statement(statement);
                    }
                }
                ClassElement::TSIndexSignature(_) => {}
            }
        }
    }

    fn collect_expression(&mut self, expression: &Expression<'a>) {
        match expression {
            Expression::StaticMemberExpression(member) => {
                self.collect_static_member_expression(member);
            }
            Expression::ComputedMemberExpression(member) => {
                self.collect_computed_member_expression(member);
            }
            Expression::PrivateFieldExpression(member) => {
                self.collect_expression(&member.object);
            }
            Expression::ArrayExpression(expression) => {
                self.collect_array_expression(expression);
            }
            Expression::ObjectExpression(expression) => {
                self.collect_object_expression(expression);
            }
            Expression::CallExpression(expression) => {
                self.collect_call_expression(expression);
            }
            Expression::NewExpression(expression) => {
                self.collect_expression(&expression.callee);
                for argument in &expression.arguments {
                    self.collect_argument(argument);
                }
            }
            Expression::ChainExpression(expression) => {
                self.collect_chain_element(&expression.expression);
            }
            Expression::ConditionalExpression(expression) => {
                self.collect_conditional_expression(expression);
            }
            Expression::BinaryExpression(expression) => {
                self.collect_expression(&expression.left);
                self.collect_expression(&expression.right);
            }
            Expression::LogicalExpression(expression) => {
                self.collect_logical_expression(expression);
            }
            Expression::AssignmentExpression(expression) => {
                self.collect_expression(&expression.right);
            }
            Expression::SequenceExpression(expression) => {
                for expression in &expression.expressions {
                    self.collect_expression(expression);
                }
            }
            Expression::ParenthesizedExpression(expression) => {
                self.collect_parenthesized_expression(expression);
            }
            Expression::UnaryExpression(expression) => {
                self.collect_expression(&expression.argument);
            }
            Expression::AwaitExpression(expression) => {
                self.collect_expression(&expression.argument);
            }
            Expression::TemplateLiteral(expression) => {
                for expression in &expression.expressions {
                    self.collect_expression(expression);
                }
            }
            Expression::TaggedTemplateExpression(expression) => {
                self.collect_expression(&expression.tag);
                for expression in &expression.quasi.expressions {
                    self.collect_expression(expression);
                }
            }
            Expression::ArrowFunctionExpression(expression) => {
                self.collect_function_body(Some(&expression.body));
            }
            Expression::FunctionExpression(expression) => {
                self.collect_function_body(expression.body.as_deref());
            }
            Expression::ClassExpression(class) => {
                self.collect_class(class);
            }
            Expression::ImportExpression(expression) => {
                self.collect_expression(&expression.source);
                if let Some(options) = &expression.options {
                    self.collect_expression(options);
                }
            }
            Expression::JSXElement(element) => {
                self.collect_jsx_element(element);
            }
            Expression::JSXFragment(fragment) => {
                for child in &fragment.children {
                    self.collect_jsx_child(child);
                }
            }
            Expression::TSAsExpression(expression) => {
                self.collect_ts_as_expression(expression);
            }
            Expression::TSSatisfiesExpression(expression) => {
                self.collect_ts_satisfies_expression(expression);
            }
            Expression::TSTypeAssertion(expression) => {
                self.collect_expression(&expression.expression);
            }
            Expression::TSNonNullExpression(expression) => {
                self.collect_ts_non_null_expression(expression);
            }
            Expression::TSInstantiationExpression(expression) => {
                self.collect_expression(&expression.expression);
            }
            _ => {}
        }
    }

    fn collect_array_expression_element(&mut self, element: &ArrayExpressionElement<'a>) {
        match element {
            ArrayExpressionElement::SpreadElement(spread) => {
                self.collect_expression(&spread.argument);
            }
            ArrayExpressionElement::StaticMemberExpression(member) => {
                self.collect_static_member_expression(member);
            }
            ArrayExpressionElement::ComputedMemberExpression(member) => {
                self.collect_computed_member_expression(member);
            }
            ArrayExpressionElement::CallExpression(expression) => {
                self.collect_call_expression(expression);
            }
            _ => {}
        }
    }

    fn collect_argument(&mut self, argument: &Argument<'a>) {
        match argument {
            Argument::SpreadElement(spread) => {
                self.collect_expression(&spread.argument);
            }
            Argument::StaticMemberExpression(member) => {
                self.collect_static_member_expression(member);
            }
            Argument::ComputedMemberExpression(member) => {
                self.collect_computed_member_expression(member);
            }
            Argument::CallExpression(expression) => {
                self.collect_call_expression(expression);
            }
            Argument::ConditionalExpression(expression) => {
                self.collect_conditional_expression(expression);
            }
            Argument::LogicalExpression(expression) => {
                self.collect_logical_expression(expression);
            }
            Argument::ArrayExpression(expression) => {
                self.collect_array_expression(expression);
            }
            Argument::ObjectExpression(expression) => {
                self.collect_object_expression(expression);
            }
            Argument::ParenthesizedExpression(expression) => {
                self.collect_parenthesized_expression(expression);
            }
            Argument::TSAsExpression(expression) => {
                self.collect_ts_as_expression(expression);
            }
            Argument::TSSatisfiesExpression(expression) => {
                self.collect_ts_satisfies_expression(expression);
            }
            Argument::TSNonNullExpression(expression) => {
                self.collect_ts_non_null_expression(expression);
            }
            _ => {}
        }
    }

    fn collect_chain_element(&mut self, element: &ChainElement<'a>) {
        match element {
            ChainElement::CallExpression(expression) => {
                self.collect_expression(&expression.callee);
                for argument in &expression.arguments {
                    self.collect_argument(argument);
                }
            }
            ChainElement::StaticMemberExpression(member) => {
                self.collect_static_member_expression(member);
            }
            ChainElement::ComputedMemberExpression(member) => {
                self.collect_computed_member_expression(member);
            }
            ChainElement::PrivateFieldExpression(member) => {
                self.collect_expression(&member.object);
            }
            ChainElement::TSNonNullExpression(expression) => {
                self.collect_expression(&expression.expression);
            }
        }
    }

    fn collect_property_key(&mut self, key: &oxc_ast::ast::PropertyKey<'a>) {
        match key {
            oxc_ast::ast::PropertyKey::StaticMemberExpression(member) => {
                self.collect_static_member_expression(member);
            }
            oxc_ast::ast::PropertyKey::ComputedMemberExpression(member) => {
                self.collect_computed_member_expression(member);
            }
            oxc_ast::ast::PropertyKey::CallExpression(expression) => {
                self.collect_call_expression(expression);
            }
            _ => {}
        }
    }

    fn collect_jsx_element(&mut self, element: &oxc_ast::ast::JSXElement<'a>) {
        for attribute in &element.opening_element.attributes {
            match attribute {
                oxc_ast::ast::JSXAttributeItem::Attribute(attribute) => {
                    if let Some(value) = &attribute.value {
                        self.collect_jsx_attribute_value(value);
                    }
                }
                oxc_ast::ast::JSXAttributeItem::SpreadAttribute(attribute) => {
                    self.collect_expression(&attribute.argument);
                }
            }
        }
        for child in &element.children {
            self.collect_jsx_child(child);
        }
    }

    fn collect_jsx_attribute_value(&mut self, value: &JSXAttributeValue<'a>) {
        match value {
            JSXAttributeValue::ExpressionContainer(container) => {
                self.collect_jsx_expression(&container.expression);
            }
            JSXAttributeValue::Element(element) => {
                self.collect_jsx_element(element);
            }
            JSXAttributeValue::Fragment(fragment) => {
                for child in &fragment.children {
                    self.collect_jsx_child(child);
                }
            }
            JSXAttributeValue::StringLiteral(_) => {}
        }
    }

    fn collect_jsx_child(&mut self, child: &JSXChild<'a>) {
        match child {
            JSXChild::Element(element) => {
                self.collect_jsx_element(element);
            }
            JSXChild::Fragment(fragment) => {
                for child in &fragment.children {
                    self.collect_jsx_child(child);
                }
            }
            JSXChild::ExpressionContainer(container) => {
                self.collect_jsx_expression(&container.expression);
            }
            JSXChild::Spread(spread) => {
                self.collect_expression(&spread.expression);
            }
            JSXChild::Text(_) => {}
        }
    }

    fn collect_jsx_expression(&mut self, expression: &JSXExpression<'a>) {
        match expression {
            JSXExpression::StaticMemberExpression(member) => {
                self.collect_static_member_expression(member);
            }
            JSXExpression::ComputedMemberExpression(member) => {
                self.collect_computed_member_expression(member);
            }
            JSXExpression::CallExpression(expression) => {
                self.collect_call_expression(expression);
            }
            JSXExpression::ConditionalExpression(expression) => {
                self.collect_conditional_expression(expression);
            }
            JSXExpression::LogicalExpression(expression) => {
                self.collect_logical_expression(expression);
            }
            JSXExpression::ArrayExpression(expression) => {
                self.collect_array_expression(expression);
            }
            JSXExpression::ObjectExpression(expression) => {
                self.collect_object_expression(expression);
            }
            JSXExpression::ParenthesizedExpression(expression) => {
                self.collect_parenthesized_expression(expression);
            }
            JSXExpression::TSAsExpression(expression) => {
                self.collect_ts_as_expression(expression);
            }
            JSXExpression::TSSatisfiesExpression(expression) => {
                self.collect_ts_satisfies_expression(expression);
            }
            JSXExpression::TSNonNullExpression(expression) => {
                self.collect_ts_non_null_expression(expression);
            }
            JSXExpression::JSXElement(element) => {
                self.collect_jsx_element(element);
            }
            JSXExpression::JSXFragment(fragment) => {
                for child in &fragment.children {
                    self.collect_jsx_child(child);
                }
            }
            _ => {}
        }
    }

    fn collect_array_expression(&mut self, expression: &ArrayExpression<'a>) {
        for element in &expression.elements {
            self.collect_array_expression_element(element);
        }
    }

    fn collect_object_expression(&mut self, expression: &ObjectExpression<'a>) {
        for property in &expression.properties {
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    if property.computed {
                        self.collect_property_key(&property.key);
                    }
                    self.collect_expression(&property.value);
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    self.collect_expression(&spread.argument);
                }
            }
        }
    }

    fn collect_call_expression(&mut self, expression: &CallExpression<'a>) {
        self.collect_expression(&expression.callee);
        for argument in &expression.arguments {
            self.collect_argument(argument);
        }
    }

    fn collect_conditional_expression(&mut self, expression: &ConditionalExpression<'a>) {
        self.collect_expression(&expression.test);
        self.collect_expression(&expression.consequent);
        self.collect_expression(&expression.alternate);
    }

    fn collect_logical_expression(&mut self, expression: &LogicalExpression<'a>) {
        self.collect_expression(&expression.left);
        self.collect_expression(&expression.right);
    }

    fn collect_parenthesized_expression(&mut self, expression: &ParenthesizedExpression<'a>) {
        self.collect_expression(&expression.expression);
    }

    fn collect_ts_as_expression(&mut self, expression: &TSAsExpression<'a>) {
        self.collect_expression(&expression.expression);
    }

    fn collect_ts_satisfies_expression(&mut self, expression: &TSSatisfiesExpression<'a>) {
        self.collect_expression(&expression.expression);
    }

    fn collect_ts_non_null_expression(&mut self, expression: &TSNonNullExpression<'a>) {
        self.collect_expression(&expression.expression);
    }

    fn collect_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if let Some(target) = self.target_for_object(&member.object)
            && let Some(byte_span) = self.css_identifier_span(member.property.span)
        {
            self.facts.push(SourceStylePropertyAccessFactV0 {
                byte_span,
                target_style_uri: target.target_style_uri.clone(),
            });
        }
        self.collect_expression(&member.object);
    }

    fn collect_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if let Some(target) = self.target_for_object(&member.object)
            && let Some(byte_span) = self.static_string_expression_content_span(&member.expression)
        {
            self.facts.push(SourceStylePropertyAccessFactV0 {
                byte_span,
                target_style_uri: target.target_style_uri.clone(),
            });
        }
        self.collect_expression(&member.object);
        self.collect_expression(&member.expression);
    }

    fn target_for_object(&self, expression: &Expression<'a>) -> Option<&SourceStyleBindingTarget> {
        match expression {
            Expression::Identifier(identifier) => self
                .targets
                .iter()
                .find(|target| target.binding == identifier.name.as_str()),
            Expression::ParenthesizedExpression(expression) => {
                self.target_for_object(&expression.expression)
            }
            Expression::TSAsExpression(expression) => {
                self.target_for_object(&expression.expression)
            }
            Expression::TSSatisfiesExpression(expression) => {
                self.target_for_object(&expression.expression)
            }
            Expression::TSTypeAssertion(expression) => {
                self.target_for_object(&expression.expression)
            }
            Expression::TSNonNullExpression(expression) => {
                self.target_for_object(&expression.expression)
            }
            Expression::TSInstantiationExpression(expression) => {
                self.target_for_object(&expression.expression)
            }
            _ => None,
        }
    }

    fn static_string_expression_content_span(
        &self,
        expression: &Expression<'a>,
    ) -> Option<ParserByteSpanV0> {
        match expression {
            Expression::StringLiteral(literal) => self.css_identifier_content_span(literal.span),
            Expression::TemplateLiteral(literal) if literal.expressions.is_empty() => {
                self.css_identifier_content_span(literal.span)
            }
            _ => None,
        }
    }

    fn css_identifier_span(&self, span: Span) -> Option<ParserByteSpanV0> {
        let span = parser_byte_span(span);
        let text = self.source.get(span.start..span.end)?;
        (!text.is_empty() && text.chars().all(is_css_identifier_continue)).then_some(span)
    }

    fn css_identifier_content_span(&self, span: Span) -> Option<ParserByteSpanV0> {
        let span = parser_byte_span(span);
        if span.end <= span.start + 1 {
            return None;
        }
        let content = ParserByteSpanV0 {
            start: span.start + 1,
            end: span.end - 1,
        };
        let text = self.source.get(content.start..content.end)?;
        (!text.is_empty() && text.chars().all(is_css_identifier_continue)).then_some(content)
    }

    fn canonicalize(&mut self) {
        self.facts.sort_by(|left, right| {
            left.byte_span
                .start
                .cmp(&right.byte_span.start)
                .then_with(|| left.byte_span.end.cmp(&right.byte_span.end))
                .then_with(|| left.target_style_uri.cmp(&right.target_style_uri))
        });
        self.facts.dedup();
    }
}

fn parser_byte_span(span: Span) -> ParserByteSpanV0 {
    ParserByteSpanV0 {
        start: span.start as usize,
        end: span.end as usize,
    }
}

fn collect_classnames_bind_utility_bindings(
    source: &str,
    style_targets: &[SourceStyleBindingTarget],
    classnames_bind_imports: &[String],
) -> Vec<ClassnamesBindUtilityBinding> {
    if style_targets.is_empty() || classnames_bind_imports.is_empty() {
        return Vec::new();
    }
    let mut bindings = Vec::new();
    let mut cursor = 0usize;
    while let Some(keyword) = next_code_identifier(source, cursor) {
        cursor = keyword.end;
        if !matches!(keyword.text, "const" | "let" | "var") {
            continue;
        }
        if let Some((binding, next_offset)) = parse_classnames_bind_utility_binding(
            source,
            keyword.end,
            style_targets,
            classnames_bind_imports,
        ) {
            bindings.push(binding);
            cursor = next_offset;
        }
    }
    bindings.sort_by(|left, right| {
        left.binding
            .cmp(&right.binding)
            .then_with(|| left.style_uri.cmp(&right.style_uri))
    });
    bindings
        .dedup_by(|left, right| left.binding == right.binding && left.style_uri == right.style_uri);
    bindings
}

fn parse_classnames_bind_utility_binding(
    source: &str,
    after_keyword: usize,
    style_targets: &[SourceStyleBindingTarget],
    classnames_bind_imports: &[String],
) -> Option<(ClassnamesBindUtilityBinding, usize)> {
    let binding_start = skip_js_trivia(source, after_keyword);
    let (binding, binding_end) = read_js_identifier(source, binding_start)?;
    let equals_offset = skip_js_trivia(source, binding_end);
    if source.as_bytes().get(equals_offset) != Some(&b'=') {
        return None;
    }
    let callee_start = skip_js_trivia(source, equals_offset + 1);
    let (callee, callee_end) = read_js_identifier(source, callee_start)?;
    if !classnames_bind_imports
        .iter()
        .any(|import_binding| import_binding == callee)
    {
        return None;
    }
    let dot_offset = skip_js_trivia(source, callee_end);
    if source.as_bytes().get(dot_offset) != Some(&b'.') {
        return None;
    }
    let (property, property_end) = read_js_identifier(source, dot_offset + 1)?;
    if property != "bind" {
        return None;
    }
    let open_paren = skip_js_trivia(source, property_end);
    if source.as_bytes().get(open_paren) != Some(&b'(') {
        return None;
    }
    let style_arg_start = skip_js_trivia(source, open_paren + 1);
    let (style_binding_name, style_binding_end) = read_js_identifier(source, style_arg_start)?;
    let style_uri = style_targets
        .iter()
        .find(|style_binding| style_binding.binding == style_binding_name)?
        .target_style_uri
        .clone();
    let style_uri = style_uri?;

    Some((
        ClassnamesBindUtilityBinding {
            binding: binding.to_string(),
            style_uri,
        },
        style_binding_end,
    ))
}

fn collect_class_name_expression_reference_facts(
    source: &str,
    local_class_values: &BTreeMap<String, SourceClassValue>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
    type_fact_targets: &mut Vec<SourceTypeFactTargetV0>,
) {
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        if identifier.text != "className" {
            continue;
        }
        let equals_offset = skip_js_trivia(source, identifier.end);
        if source.as_bytes().get(equals_offset) != Some(&b'=') {
            continue;
        }
        let value_offset = skip_js_trivia(source, equals_offset + 1);
        if source.as_bytes().get(value_offset) == Some(&b'{') {
            let expression_start = value_offset + 1;
            if let Some(expression_end) = jsx_expression_end(source, expression_start) {
                collect_selector_references_from_js_expression(
                    source,
                    expression_start,
                    expression_end,
                    None,
                    local_class_values,
                    references,
                    type_fact_targets,
                );
                cursor = advance_js_scan_cursor(source, expression_end, source.len());
            }
        }
    }
}

fn collect_classnames_bind_call_reference_facts(
    source: &str,
    binding: &str,
    target_style_uri: Option<&str>,
    local_class_values: &BTreeMap<String, SourceClassValue>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
    type_fact_targets: &mut Vec<SourceTypeFactTargetV0>,
) {
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        if identifier.text != binding {
            continue;
        }
        let open_paren = skip_js_trivia(source, identifier.end);
        if source.as_bytes().get(open_paren) != Some(&b'(') {
            continue;
        }
        let call_end = js_call_end(source, open_paren).unwrap_or(source.len());
        for (argument_start, argument_end) in
            split_top_level_js_segments(source, open_paren + 1, call_end, b',')
        {
            collect_selector_references_from_js_expression(
                source,
                argument_start,
                argument_end,
                target_style_uri,
                local_class_values,
                references,
                type_fact_targets,
            );
        }
        cursor = call_end.saturating_add(1).min(source.len());
    }
}

fn collect_selector_references_from_js_expression(
    source: &str,
    start: usize,
    end: usize,
    target_style_uri: Option<&str>,
    local_class_values: &BTreeMap<String, SourceClassValue>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
    type_fact_targets: &mut Vec<SourceTypeFactTargetV0>,
) {
    let (start, end) = trim_js_expression(source, start, end);
    let (start, end) = unwrap_js_parenthesized_expression(source, start, end);
    if start >= end {
        return;
    }

    if let Some((literal_start, literal_end, next_offset)) =
        js_string_literal_span(source, start, end)
        && trim_js_expression(source, next_offset, end).0 >= end
    {
        push_js_literal_selector_references(
            source,
            literal_start,
            literal_end,
            source.as_bytes().get(start).copied() == Some(b'`'),
            target_style_uri,
            references,
        );
        if source.as_bytes().get(start).copied() == Some(b'`') {
            collect_template_type_fact_targets(
                source,
                literal_start,
                literal_end,
                target_style_uri,
                type_fact_targets,
            );
        }
        return;
    }

    if source.as_bytes().get(start) == Some(&b'{')
        && matching_js_block_end(source, start, b'{', b'}') == Some(end - 1)
    {
        collect_object_literal_selector_references(
            source,
            start,
            end,
            target_style_uri,
            local_class_values,
            references,
            type_fact_targets,
        );
        return;
    }

    if source.as_bytes().get(start) == Some(&b'[')
        && matching_js_block_end(source, start, b'[', b']') == Some(end - 1)
    {
        for (element_start, element_end) in
            split_top_level_js_segments(source, start + 1, end - 1, b',')
        {
            let element_start = skip_js_trivia_until(source, element_start, element_end);
            let element_start = if source[element_start..element_end].starts_with("...") {
                element_start + 3
            } else {
                element_start
            };
            collect_selector_references_from_js_expression(
                source,
                element_start,
                element_end,
                target_style_uri,
                local_class_values,
                references,
                type_fact_targets,
            );
        }
        return;
    }

    if let Some((arguments_start, arguments_end)) = class_utility_call_arguments(source, start, end)
    {
        for (argument_start, argument_end) in
            split_top_level_js_segments(source, arguments_start, arguments_end, b',')
        {
            collect_selector_references_from_js_expression(
                source,
                argument_start,
                argument_end,
                target_style_uri,
                local_class_values,
                references,
                type_fact_targets,
            );
        }
        return;
    }

    if let Some((_, true_start, true_end, false_start, false_end)) =
        top_level_conditional_parts(source, start, end)
    {
        collect_selector_references_from_js_expression(
            source,
            true_start,
            true_end,
            target_style_uri,
            local_class_values,
            references,
            type_fact_targets,
        );
        collect_selector_references_from_js_expression(
            source,
            false_start,
            false_end,
            target_style_uri,
            local_class_values,
            references,
            type_fact_targets,
        );
        return;
    }

    if let Some(operator_offset) = find_top_level_js_operator(source, start, end, "&&")
        .or_else(|| find_top_level_js_operator(source, start, end, "||"))
    {
        collect_selector_references_from_js_expression(
            source,
            operator_offset + 2,
            end,
            target_style_uri,
            local_class_values,
            references,
            type_fact_targets,
        );
        return;
    }

    if let Some(value) =
        source_class_value_from_js_expression(source, start, end, local_class_values)
        && !value.is_empty()
    {
        push_source_class_value_reference(
            ParserByteSpanV0 { start, end },
            value,
            target_style_uri,
            references,
        );
        return;
    }

    if let Some(prefix) =
        static_string_prefix_for_js_expression(source, start, end, local_class_values)
        && !prefix.is_empty()
    {
        push_selector_reference(
            ParserByteSpanV0 { start, end },
            Some(prefix),
            SourceSelectorReferenceMatchKindV0::Prefix,
            target_style_uri,
            references,
        );
        return;
    }

    if let Some(path) = js_expression_path(source, start, end) {
        push_source_type_fact_target(
            ParserByteSpanV0 { start, end },
            path.as_str(),
            target_style_uri,
            "",
            "",
            type_fact_targets,
        );
    }
}

fn collect_local_class_value_bindings(source: &str) -> BTreeMap<String, SourceClassValue> {
    let mut values = BTreeMap::new();
    let mut cursor = 0usize;
    while let Some(keyword) = next_code_identifier(source, cursor) {
        cursor = keyword.end;
        if !matches!(keyword.text, "const" | "let" | "var") {
            continue;
        }
        let binding_start = skip_js_trivia(source, keyword.end);
        let Some((binding, binding_end)) = read_js_identifier(source, binding_start) else {
            continue;
        };
        let equals_offset = skip_js_trivia(source, binding_end);
        if source.as_bytes().get(equals_offset) != Some(&b'=') {
            continue;
        }
        let expression_start = skip_js_trivia(source, equals_offset + 1);
        let expression_end = js_statement_expression_end(source, expression_start);
        if let Some(value) =
            source_class_value_from_js_expression(source, expression_start, expression_end, &values)
            && !value.is_empty()
        {
            values.insert(binding.to_string(), value);
        }
        let (_, property_values) = source_class_value_from_object_literal(
            source,
            expression_start,
            expression_end,
            &values,
        );
        for (property, value) in property_values {
            if !value.is_empty() {
                values.insert(format!("{binding}.{property}"), value);
            }
        }
        cursor = expression_end.min(source.len());
    }
    values
}

fn source_class_value_from_js_expression(
    source: &str,
    start: usize,
    end: usize,
    local_class_values: &BTreeMap<String, SourceClassValue>,
) -> Option<SourceClassValue> {
    let (start, end) = trim_js_expression(source, start, end);
    let (start, end) = unwrap_js_parenthesized_expression(source, start, end);
    if start >= end {
        return None;
    }

    if let Some((literal_start, literal_end, next_offset)) =
        js_string_literal_span(source, start, end)
        && trim_js_expression(source, next_offset, end).0 >= end
    {
        return Some(source_class_value_from_js_literal(
            source,
            literal_start,
            literal_end,
            source.as_bytes().get(start).copied() == Some(b'`'),
        ));
    }

    if source.as_bytes().get(start) == Some(&b'{')
        && matching_js_block_end(source, start, b'{', b'}') == Some(end - 1)
    {
        let (value, _) =
            source_class_value_from_object_literal(source, start, end, local_class_values);
        return Some(value);
    }

    if source.as_bytes().get(start) == Some(&b'[')
        && matching_js_block_end(source, start, b'[', b']') == Some(end - 1)
    {
        let mut value = SourceClassValue::default();
        for (element_start, element_end) in
            split_top_level_js_segments(source, start + 1, end - 1, b',')
        {
            let element_start = skip_js_trivia_until(source, element_start, element_end);
            let element_start = if source[element_start..element_end].starts_with("...") {
                element_start + 3
            } else {
                element_start
            };
            if let Some(element_value) = source_class_value_from_js_expression(
                source,
                element_start,
                element_end,
                local_class_values,
            ) {
                value.merge(element_value);
            }
        }
        return Some(value);
    }

    if let Some((arguments_start, arguments_end)) = class_utility_call_arguments(source, start, end)
    {
        let mut value = SourceClassValue::default();
        for (argument_start, argument_end) in
            split_top_level_js_segments(source, arguments_start, arguments_end, b',')
        {
            if let Some(argument_value) = source_class_value_from_js_expression(
                source,
                argument_start,
                argument_end,
                local_class_values,
            ) {
                value.merge(argument_value);
            }
        }
        return Some(value);
    }

    if let Some((_, true_start, true_end, false_start, false_end)) =
        top_level_conditional_parts(source, start, end)
    {
        let mut value = SourceClassValue::default();
        if let Some(true_value) =
            source_class_value_from_js_expression(source, true_start, true_end, local_class_values)
        {
            value.merge(true_value);
        }
        if let Some(false_value) = source_class_value_from_js_expression(
            source,
            false_start,
            false_end,
            local_class_values,
        ) {
            value.merge(false_value);
        }
        return Some(value);
    }

    if let Some(operator_offset) = find_top_level_js_operator(source, start, end, "&&")
        .or_else(|| find_top_level_js_operator(source, start, end, "||"))
    {
        return source_class_value_from_js_expression(
            source,
            operator_offset + 2,
            end,
            local_class_values,
        );
    }

    if let Some(path) = js_expression_path(source, start, end)
        && let Some(value) = local_class_values.get(path.as_str())
    {
        return Some(value.clone());
    }

    static_string_prefix_for_js_expression(source, start, end, local_class_values).map(|prefix| {
        let mut value = SourceClassValue::default();
        if !prefix.is_empty() {
            value.prefixes.push(prefix);
        }
        value
    })
}

fn source_class_value_from_js_literal(
    source: &str,
    literal_start: usize,
    literal_end: usize,
    is_template: bool,
) -> SourceClassValue {
    let mut value = SourceClassValue::default();
    if is_template
        && let Some(relative_interpolation) = source[literal_start..literal_end].find("${")
    {
        let prefix_end = literal_start + relative_interpolation;
        push_template_prefix_value(source, literal_start, prefix_end, &mut value);
    } else {
        value
            .exact
            .extend(class_token_strings(source, literal_start, literal_end));
    }
    value.canonicalize();
    value
}

fn source_class_value_from_object_literal(
    source: &str,
    start: usize,
    end: usize,
    local_class_values: &BTreeMap<String, SourceClassValue>,
) -> (SourceClassValue, BTreeMap<String, SourceClassValue>) {
    let (start, end) = trim_js_expression(source, start, end);
    let (start, end) = unwrap_js_parenthesized_expression(source, start, end);
    let mut object_value = SourceClassValue::default();
    let mut property_values = BTreeMap::new();
    if source.as_bytes().get(start) != Some(&b'{')
        || matching_js_block_end(source, start, b'{', b'}') != Some(end.saturating_sub(1))
    {
        return (object_value, property_values);
    }

    for (property_start, property_end) in
        split_top_level_js_segments(source, start + 1, end - 1, b',')
    {
        let (property_start, property_end) =
            trim_js_expression(source, property_start, property_end);
        if property_start >= property_end {
            continue;
        }
        if source[property_start..property_end].starts_with("...") {
            if let Some(spread_value) = source_class_value_from_js_expression(
                source,
                property_start + 3,
                property_end,
                local_class_values,
            ) {
                object_value.merge(spread_value);
            }
            continue;
        }
        let colon = find_top_level_js_byte(source, property_start, property_end, b':');
        let key_end = colon.unwrap_or(property_end);
        let key_value =
            source_class_value_from_object_key(source, property_start, key_end, local_class_values);
        object_value.merge(key_value.clone());
        if let Some(property_name) = object_property_name(source, property_start, key_end) {
            let property_value = colon
                .and_then(|colon| {
                    source_class_value_from_js_expression(
                        source,
                        colon + 1,
                        property_end,
                        local_class_values,
                    )
                })
                .filter(|value| !value.is_empty())
                .unwrap_or(key_value);
            property_values.insert(property_name, property_value);
        }
    }
    object_value.canonicalize();
    (object_value, property_values)
}

fn collect_object_literal_selector_references(
    source: &str,
    start: usize,
    end: usize,
    target_style_uri: Option<&str>,
    local_class_values: &BTreeMap<String, SourceClassValue>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
    type_fact_targets: &mut Vec<SourceTypeFactTargetV0>,
) {
    for (property_start, property_end) in
        split_top_level_js_segments(source, start + 1, end - 1, b',')
    {
        let (property_start, property_end) =
            trim_js_expression(source, property_start, property_end);
        if property_start >= property_end {
            continue;
        }
        if source[property_start..property_end].starts_with("...") {
            collect_selector_references_from_js_expression(
                source,
                property_start + 3,
                property_end,
                target_style_uri,
                local_class_values,
                references,
                type_fact_targets,
            );
            continue;
        }
        let colon = find_top_level_js_byte(source, property_start, property_end, b':');
        let key_end = colon.unwrap_or(property_end);
        collect_selector_references_from_object_key(
            source,
            property_start,
            key_end,
            target_style_uri,
            local_class_values,
            references,
            type_fact_targets,
        );
    }
}

fn class_utility_call_arguments(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let (callee, callee_end) = read_js_identifier(source, start)?;
    if !is_class_utility_callee(callee) {
        return None;
    }
    let open_paren = skip_js_trivia_until(source, callee_end, end);
    if source.as_bytes().get(open_paren) != Some(&b'(') {
        return None;
    }
    let call_end = js_call_end(source, open_paren)?;
    if call_end > end || trim_js_expression(source, call_end + 1, end).0 < end {
        return None;
    }
    Some((open_paren + 1, call_end))
}

fn is_class_utility_callee(callee: &str) -> bool {
    matches!(callee, "classnames" | "classNames" | "clsx" | "cn")
}

fn collect_selector_references_from_object_key(
    source: &str,
    start: usize,
    end: usize,
    target_style_uri: Option<&str>,
    local_class_values: &BTreeMap<String, SourceClassValue>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
    type_fact_targets: &mut Vec<SourceTypeFactTargetV0>,
) {
    let (start, end) = trim_js_expression(source, start, end);
    if start >= end {
        return;
    }
    if source.as_bytes().get(start) == Some(&b'[')
        && matching_js_block_end(source, start, b'[', b']') == Some(end - 1)
    {
        collect_selector_references_from_js_expression(
            source,
            start + 1,
            end - 1,
            target_style_uri,
            local_class_values,
            references,
            type_fact_targets,
        );
        return;
    }
    if let Some((literal_start, literal_end, next_offset)) =
        js_string_literal_span(source, start, end)
        && trim_js_expression(source, next_offset, end).0 >= end
    {
        push_js_literal_selector_references(
            source,
            literal_start,
            literal_end,
            source.as_bytes().get(start).copied() == Some(b'`'),
            target_style_uri,
            references,
        );
        if source.as_bytes().get(start).copied() == Some(b'`') {
            collect_template_type_fact_targets(
                source,
                literal_start,
                literal_end,
                target_style_uri,
                type_fact_targets,
            );
        }
        return;
    }
    if let Some((identifier, identifier_end)) = read_js_identifier(source, start)
        && trim_js_expression(source, identifier_end, end).0 >= end
    {
        push_selector_reference(
            ParserByteSpanV0 { start, end },
            Some(identifier.to_string()),
            SourceSelectorReferenceMatchKindV0::Exact,
            target_style_uri,
            references,
        );
    }
}

fn source_class_value_from_object_key(
    source: &str,
    start: usize,
    end: usize,
    local_class_values: &BTreeMap<String, SourceClassValue>,
) -> SourceClassValue {
    let (start, end) = trim_js_expression(source, start, end);
    if start >= end {
        return SourceClassValue::default();
    }
    if source.as_bytes().get(start) == Some(&b'[')
        && matching_js_block_end(source, start, b'[', b']') == Some(end - 1)
    {
        return source_class_value_from_js_expression(
            source,
            start + 1,
            end - 1,
            local_class_values,
        )
        .unwrap_or_default();
    }
    if let Some((literal_start, literal_end, next_offset)) =
        js_string_literal_span(source, start, end)
        && trim_js_expression(source, next_offset, end).0 >= end
    {
        return source_class_value_from_js_literal(
            source,
            literal_start,
            literal_end,
            source.as_bytes().get(start).copied() == Some(b'`'),
        );
    }
    if let Some((identifier, identifier_end)) = read_js_identifier(source, start)
        && trim_js_expression(source, identifier_end, end).0 >= end
    {
        let mut value = SourceClassValue::default();
        value.exact.push(identifier.to_string());
        return value;
    }
    SourceClassValue::default()
}

fn object_property_name(source: &str, start: usize, end: usize) -> Option<String> {
    let (start, end) = trim_js_expression(source, start, end);
    if let Some((literal_start, literal_end, next_offset)) =
        js_string_literal_span(source, start, end)
        && trim_js_expression(source, next_offset, end).0 >= end
    {
        return source.get(literal_start..literal_end).map(str::to_string);
    }
    let (identifier, identifier_end) = read_js_identifier(source, start)?;
    (trim_js_expression(source, identifier_end, end).0 >= end).then(|| identifier.to_string())
}

fn push_source_class_value_reference(
    byte_span: ParserByteSpanV0,
    value: SourceClassValue,
    target_style_uri: Option<&str>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    for selector_name in value.exact {
        push_selector_reference(
            byte_span,
            Some(selector_name),
            SourceSelectorReferenceMatchKindV0::Exact,
            target_style_uri,
            references,
        );
    }
    for prefix in value.prefixes {
        push_selector_reference(
            byte_span,
            Some(prefix),
            SourceSelectorReferenceMatchKindV0::Prefix,
            target_style_uri,
            references,
        );
    }
}

fn collect_template_type_fact_targets(
    source: &str,
    literal_start: usize,
    literal_end: usize,
    target_style_uri: Option<&str>,
    type_fact_targets: &mut Vec<SourceTypeFactTargetV0>,
) {
    let Some((prefix, expression_span, suffix)) =
        single_template_interpolation_projection(source, literal_start, literal_end)
    else {
        return;
    };
    let Some(path) = js_expression_path(source, expression_span.start, expression_span.end) else {
        return;
    };
    push_source_type_fact_target(
        expression_span,
        path.as_str(),
        target_style_uri,
        prefix.as_str(),
        suffix.as_str(),
        type_fact_targets,
    );
}

fn single_template_interpolation_projection(
    source: &str,
    literal_start: usize,
    literal_end: usize,
) -> Option<(String, ParserByteSpanV0, String)> {
    let relative_open = source.get(literal_start..literal_end)?.find("${")?;
    let open = literal_start + relative_open;
    if source.get(open + 2..literal_end)?.contains("${") {
        return None;
    }
    let expression_start = open + 2;
    let close = matching_js_block_end(source, open + 1, b'{', b'}')?;
    if close > literal_end {
        return None;
    }
    let (expression_start, expression_end) = trim_js_expression(source, expression_start, close);
    if expression_start >= expression_end {
        return None;
    }
    let prefix_start = template_token_start(source, literal_start, open);
    let suffix_end = template_token_end(source, close + 1, literal_end);
    let prefix = source.get(prefix_start..open)?.to_string();
    let suffix = source.get(close + 1..suffix_end)?.to_string();
    if !prefix.chars().all(is_css_identifier_continue)
        || !suffix.chars().all(is_css_identifier_continue)
    {
        return None;
    }
    Some((
        prefix,
        ParserByteSpanV0 {
            start: expression_start,
            end: expression_end,
        },
        suffix,
    ))
}

fn template_token_start(source: &str, literal_start: usize, prefix_end: usize) -> usize {
    source
        .get(literal_start..prefix_end)
        .and_then(|value| {
            value
                .char_indices()
                .rev()
                .find(|(_, ch)| ch.is_ascii_whitespace())
                .map(|(index, ch)| literal_start + index + ch.len_utf8())
        })
        .unwrap_or(literal_start)
}

fn template_token_end(source: &str, suffix_start: usize, literal_end: usize) -> usize {
    source
        .get(suffix_start..literal_end)
        .and_then(|value| {
            value
                .char_indices()
                .find(|(_, ch)| ch.is_ascii_whitespace())
                .map(|(index, _)| suffix_start + index)
        })
        .unwrap_or(literal_end)
}

fn push_source_type_fact_target(
    byte_span: ParserByteSpanV0,
    expression_path: &str,
    target_style_uri: Option<&str>,
    prefix: &str,
    suffix: &str,
    type_fact_targets: &mut Vec<SourceTypeFactTargetV0>,
) {
    type_fact_targets.push(SourceTypeFactTargetV0 {
        byte_span,
        expression_id: source_type_fact_expression_id(expression_path, byte_span),
        target_style_uri: target_style_uri.map(ToString::to_string),
        prefix: prefix.to_string(),
        suffix: suffix.to_string(),
    });
}

fn source_type_fact_expression_id(expression_path: &str, byte_span: ParserByteSpanV0) -> String {
    format!(
        "omena-bridge-source-type-fact:{expression_path}:{}:{}",
        byte_span.start, byte_span.end
    )
}

fn push_selector_reference(
    byte_span: ParserByteSpanV0,
    selector_name: Option<String>,
    match_kind: SourceSelectorReferenceMatchKindV0,
    target_style_uri: Option<&str>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    references.push(SourceSelectorReferenceFactV0 {
        byte_span,
        selector_name,
        match_kind,
        target_style_uri: target_style_uri.map(ToString::to_string),
    });
}

fn push_js_literal_selector_references(
    source: &str,
    literal_start: usize,
    literal_end: usize,
    is_template: bool,
    target_style_uri: Option<&str>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    if is_template
        && let Some(relative_interpolation) = source[literal_start..literal_end].find("${")
    {
        push_template_prefix_selector_references(
            source,
            literal_start,
            literal_start + relative_interpolation,
            target_style_uri,
            references,
        );
        return;
    }

    push_string_literal_selector_references(
        source,
        ParserByteSpanV0 {
            start: literal_start,
            end: literal_end,
        },
        target_style_uri.map(ToString::to_string),
        references,
    );
}

fn push_template_prefix_selector_references(
    source: &str,
    literal_start: usize,
    prefix_end: usize,
    target_style_uri: Option<&str>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    let spans = class_token_byte_spans(source, literal_start, prefix_end);
    let prefix_ends_with_space = source[..prefix_end]
        .chars()
        .last()
        .is_none_or(char::is_whitespace);
    for (index, span) in spans.iter().enumerate() {
        let is_open_prefix = index + 1 == spans.len() && !prefix_ends_with_space;
        push_selector_reference(
            *span,
            Some(source[span.start..span.end].to_string()),
            if is_open_prefix {
                SourceSelectorReferenceMatchKindV0::Prefix
            } else {
                SourceSelectorReferenceMatchKindV0::Exact
            },
            target_style_uri,
            references,
        );
    }
}

fn push_template_prefix_value(
    source: &str,
    literal_start: usize,
    prefix_end: usize,
    value: &mut SourceClassValue,
) {
    let spans = class_token_byte_spans(source, literal_start, prefix_end);
    let prefix_ends_with_space = source[..prefix_end]
        .chars()
        .last()
        .is_none_or(char::is_whitespace);
    for (index, span) in spans.iter().enumerate() {
        let token = source[span.start..span.end].to_string();
        if index + 1 == spans.len() && !prefix_ends_with_space {
            value.prefixes.push(token);
        } else {
            value.exact.push(token);
        }
    }
}

fn class_token_strings(source: &str, literal_start: usize, literal_end: usize) -> Vec<String> {
    class_token_byte_spans(source, literal_start, literal_end)
        .into_iter()
        .map(|span| source[span.start..span.end].to_string())
        .collect()
}

fn push_string_literal_selector_references(
    source: &str,
    literal_span: ParserByteSpanV0,
    target_style_uri: Option<String>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    for span in class_token_byte_spans(source, literal_span.start, literal_span.end) {
        references.push(SourceSelectorReferenceFactV0 {
            byte_span: span,
            selector_name: None,
            match_kind: SourceSelectorReferenceMatchKindV0::Exact,
            target_style_uri: target_style_uri.clone(),
        });
    }
}

fn trim_js_expression(source: &str, start: usize, end: usize) -> (usize, usize) {
    let mut start = char_boundary_ceil(source, start);
    let mut end = char_boundary_floor(source, end);
    start = skip_js_trivia_until(source, start, end);
    while end > start
        && source
            .as_bytes()
            .get(end - 1)
            .is_some_and(u8::is_ascii_whitespace)
    {
        end -= 1;
    }
    (start, end)
}

fn char_boundary_floor(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while index > 0 && !source.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn char_boundary_ceil(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while index < source.len() && !source.is_char_boundary(index) {
        index += 1;
    }
    index
}

fn advance_js_scan_cursor(source: &str, cursor: usize, limit: usize) -> usize {
    let cursor = char_boundary_ceil(source, cursor);
    let limit = char_boundary_floor(source, limit);
    if cursor >= limit {
        return limit;
    }
    char_boundary_ceil(source, cursor + 1).min(limit)
}

fn advance_js_escaped_char(source: &str, slash_offset: usize, limit: usize) -> usize {
    let after_slash = advance_js_scan_cursor(source, slash_offset, limit);
    advance_js_scan_cursor(source, after_slash, limit)
}

fn unwrap_js_parenthesized_expression(source: &str, start: usize, end: usize) -> (usize, usize) {
    let mut current_start = start;
    let mut current_end = end;
    loop {
        let (trimmed_start, trimmed_end) = trim_js_expression(source, current_start, current_end);
        if source.as_bytes().get(trimmed_start) == Some(&b'(')
            && matching_js_block_end(source, trimmed_start, b'(', b')')
                == Some(trimmed_end.saturating_sub(1))
        {
            current_start = trimmed_start + 1;
            current_end = trimmed_end - 1;
            continue;
        }
        return (trimmed_start, trimmed_end);
    }
}

fn js_statement_expression_end(source: &str, start: usize) -> usize {
    let mut cursor = char_boundary_ceil(source, start);
    let mut depth = 0usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied() {
            Some(b'\'' | b'"' | b'`') => {
                cursor =
                    skip_js_string_literal(source, cursor, source.len()).unwrap_or(source.len());
            }
            Some(b'(' | b'[' | b'{') => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            Some(b')' | b']' | b'}') => {
                depth = depth.saturating_sub(1);
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            Some(b';') if depth == 0 => return cursor,
            Some(b'\n') if depth == 0 => return cursor,
            Some(_) => cursor = advance_js_scan_cursor(source, cursor, source.len()),
            None => break,
        }
    }
    source.len()
}

fn matching_js_block_end(source: &str, open_offset: usize, open: u8, close: u8) -> Option<usize> {
    if source.as_bytes().get(open_offset) != Some(&open) {
        return None;
    }
    let mut cursor = advance_js_scan_cursor(source, open_offset, source.len());
    let mut depth = 1usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, source.len())?;
            }
            byte if byte == open => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            byte if byte == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor);
                }
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            _ => cursor = advance_js_scan_cursor(source, cursor, source.len()),
        }
    }
    None
}

fn split_top_level_js_segments(
    source: &str,
    start: usize,
    end: usize,
    delimiter: u8,
) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let end = char_boundary_floor(source, end);
    let mut segment_start = char_boundary_ceil(source, start).min(end);
    let mut cursor = segment_start;
    let mut depth = 0usize;
    while cursor < end {
        match source.as_bytes().get(cursor).copied() {
            Some(b'\'' | b'"' | b'`') => {
                cursor = skip_js_string_literal(source, cursor, end).unwrap_or(end);
            }
            Some(b'(' | b'[' | b'{') => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            Some(b')' | b']' | b'}') => {
                depth = depth.saturating_sub(1);
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            Some(byte) if byte == delimiter && depth == 0 => {
                segments.push((segment_start, cursor));
                cursor = advance_js_scan_cursor(source, cursor, end);
                segment_start = cursor;
            }
            Some(_) => cursor = advance_js_scan_cursor(source, cursor, end),
            None => break,
        }
    }
    if segment_start <= end {
        segments.push((segment_start, end));
    }
    segments
}

fn find_top_level_js_byte(source: &str, start: usize, end: usize, needle: u8) -> Option<usize> {
    let end = char_boundary_floor(source, end);
    let mut cursor = char_boundary_ceil(source, start).min(end);
    let mut depth = 0usize;
    while cursor < end {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, end).unwrap_or(end);
            }
            b'(' | b'[' | b'{' => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            byte if byte == needle && depth == 0 => return Some(cursor),
            _ => cursor = advance_js_scan_cursor(source, cursor, end),
        }
    }
    None
}

fn find_top_level_js_operator(
    source: &str,
    start: usize,
    end: usize,
    operator: &str,
) -> Option<usize> {
    let end = char_boundary_floor(source, end);
    let mut cursor = char_boundary_ceil(source, start).min(end);
    let mut depth = 0usize;
    while cursor < end {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, end).unwrap_or(end);
            }
            b'(' | b'[' | b'{' => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            _ if depth == 0
                && source
                    .get(cursor..end)
                    .is_some_and(|rest| rest.starts_with(operator)) =>
            {
                return Some(cursor);
            }
            _ => cursor = advance_js_scan_cursor(source, cursor, end),
        }
    }
    None
}

fn top_level_conditional_parts(
    source: &str,
    start: usize,
    end: usize,
) -> Option<(usize, usize, usize, usize, usize)> {
    let question = find_top_level_js_byte(source, start, end, b'?')?;
    let end = char_boundary_floor(source, end);
    let mut cursor = advance_js_scan_cursor(source, question, end);
    let mut depth = 0usize;
    let mut nested_conditional_depth = 0usize;
    while cursor < end {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, end).unwrap_or(end);
            }
            b'(' | b'[' | b'{' => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            b'?' if depth == 0 => {
                nested_conditional_depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            b':' if depth == 0 && nested_conditional_depth == 0 => {
                return Some((
                    question,
                    advance_js_scan_cursor(source, question, end),
                    cursor,
                    advance_js_scan_cursor(source, cursor, end),
                    end,
                ));
            }
            b':' if depth == 0 => {
                nested_conditional_depth = nested_conditional_depth.saturating_sub(1);
                cursor = advance_js_scan_cursor(source, cursor, end);
            }
            _ => cursor = advance_js_scan_cursor(source, cursor, end),
        }
    }
    None
}

fn js_expression_path(source: &str, start: usize, end: usize) -> Option<String> {
    let (start, end) = trim_js_expression(source, start, end);
    let (first, mut cursor) = read_js_identifier(source, start)?;
    let mut path = vec![first.to_string()];
    loop {
        cursor = skip_js_trivia_until(source, cursor, end);
        match source.as_bytes().get(cursor).copied() {
            Some(b'.') => {
                let member_start = skip_js_trivia_until(source, cursor + 1, end);
                let (member, member_end) = read_js_identifier(source, member_start)?;
                path.push(member.to_string());
                cursor = member_end;
            }
            Some(b'[') => {
                if let Some((literal_start, literal_end, bracket_end)) =
                    bracket_string_literal_access(source, cursor)
                    && bracket_end <= end
                {
                    path.push(source[literal_start..literal_end].to_string());
                    cursor = bracket_end;
                } else {
                    return None;
                }
            }
            _ => break,
        }
    }
    (trim_js_expression(source, cursor, end).0 >= end).then(|| path.join("."))
}

fn static_string_prefix_for_js_expression(
    source: &str,
    start: usize,
    end: usize,
    local_class_values: &BTreeMap<String, SourceClassValue>,
) -> Option<String> {
    let (start, end) = trim_js_expression(source, start, end);
    let (start, end) = unwrap_js_parenthesized_expression(source, start, end);
    if let Some((literal_start, literal_end, next_offset)) =
        js_string_literal_span(source, start, end)
        && trim_js_expression(source, next_offset, end).0 >= end
    {
        if source.as_bytes().get(start).copied() == Some(b'`')
            && let Some(relative_interpolation) = source[literal_start..literal_end].find("${")
        {
            return Some(source[literal_start..literal_start + relative_interpolation].to_string());
        }
        return Some(source[literal_start..literal_end].to_string());
    }
    if let Some(path) = js_expression_path(source, start, end)
        && let Some(value) = local_class_values.get(path.as_str())
    {
        if value.exact.len() == 1 && value.prefixes.is_empty() {
            return value.exact.first().cloned();
        }
        if value.prefixes.len() == 1 && value.exact.is_empty() {
            return value.prefixes.first().cloned();
        }
    }
    if let Some(plus_offset) = find_top_level_js_operator(source, start, end, "+") {
        let left =
            static_string_prefix_for_js_expression(source, start, plus_offset, local_class_values)?;
        let right = static_string_prefix_for_js_expression(
            source,
            plus_offset + 1,
            end,
            local_class_values,
        )
        .unwrap_or_default();
        return Some(format!("{left}{right}"));
    }
    None
}

fn js_call_end(source: &str, open_paren: usize) -> Option<usize> {
    if source.as_bytes().get(open_paren) != Some(&b'(') {
        return None;
    }
    let mut cursor = advance_js_scan_cursor(source, open_paren, source.len());
    let mut depth = 1usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, source.len())?;
            }
            b'(' => {
                depth += 1;
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor);
                }
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            _ => {
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
        }
    }
    None
}

fn class_token_byte_spans(
    source: &str,
    literal_start: usize,
    literal_end: usize,
) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut token_start: Option<usize> = None;
    for (relative_index, ch) in source[literal_start..literal_end].char_indices() {
        let index = literal_start + relative_index;
        if ch.is_ascii_whitespace() {
            if let Some(start) = token_start.take() {
                push_class_token_span(source, start, index, &mut spans);
            }
        } else if token_start.is_none() {
            token_start = Some(index);
        }
    }
    if let Some(start) = token_start {
        push_class_token_span(source, start, literal_end, &mut spans);
    }
    spans
}

fn push_class_token_span(
    source: &str,
    start: usize,
    end: usize,
    spans: &mut Vec<ParserByteSpanV0>,
) {
    if start < end && source[start..end].chars().all(is_css_identifier_continue) {
        spans.push(ParserByteSpanV0 { start, end });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CodeIdentifier<'a> {
    text: &'a str,
    end: usize,
}

fn next_code_identifier(source: &str, mut cursor: usize) -> Option<CodeIdentifier<'_>> {
    while cursor < source.len() {
        cursor = skip_js_trivia(source, cursor);
        let byte = source.as_bytes().get(cursor).copied()?;
        if matches!(byte, b'\'' | b'"' | b'`') {
            cursor = skip_js_string_literal(source, cursor, source.len()).unwrap_or(source.len());
            continue;
        }
        if byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$') {
            let (text, end) = read_js_identifier(source, cursor)?;
            return Some(CodeIdentifier { text, end });
        }
        cursor = advance_js_scan_cursor(source, cursor, source.len());
    }
    None
}

fn skip_js_trivia(source: &str, cursor: usize) -> usize {
    skip_js_trivia_until(source, cursor, source.len())
}

fn skip_js_trivia_until(source: &str, mut cursor: usize, limit: usize) -> usize {
    loop {
        cursor = skip_ascii_whitespace_until(source, cursor, limit);
        if source.as_bytes().get(cursor) == Some(&b'/') {
            match source.as_bytes().get(cursor + 1).copied() {
                Some(b'/') => {
                    cursor = skip_js_line_comment(source, cursor + 2, limit);
                    continue;
                }
                Some(b'*') => {
                    cursor = skip_js_block_comment(source, cursor + 2, limit);
                    continue;
                }
                _ => {}
            }
        }
        return cursor;
    }
}

fn skip_ascii_whitespace_until(source: &str, mut offset: usize, limit: usize) -> usize {
    while offset < limit
        && source
            .as_bytes()
            .get(offset)
            .is_some_and(u8::is_ascii_whitespace)
    {
        offset += 1;
    }
    offset
}

fn skip_ascii_whitespace(source: &str, mut offset: usize) -> usize {
    while source
        .as_bytes()
        .get(offset)
        .is_some_and(u8::is_ascii_whitespace)
    {
        offset += 1;
    }
    offset
}

fn skip_js_line_comment(source: &str, mut cursor: usize, limit: usize) -> usize {
    let limit = char_boundary_floor(source, limit);
    while cursor < limit {
        if source.as_bytes().get(cursor) == Some(&b'\n') {
            return advance_js_scan_cursor(source, cursor, limit);
        }
        cursor = advance_js_scan_cursor(source, cursor, limit);
    }
    limit
}

fn skip_js_block_comment(source: &str, mut cursor: usize, limit: usize) -> usize {
    let limit = char_boundary_floor(source, limit);
    while cursor + 1 < limit {
        if source.as_bytes().get(cursor) == Some(&b'*')
            && source.as_bytes().get(cursor + 1) == Some(&b'/')
        {
            return cursor + 2;
        }
        cursor = advance_js_scan_cursor(source, cursor, limit);
    }
    limit
}

fn jsx_expression_end(source: &str, start: usize) -> Option<usize> {
    let mut cursor = char_boundary_ceil(source, start);
    let mut nested_braces = 0usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_js_string_literal(source, cursor, source.len())?;
            }
            b'{' => {
                nested_braces += 1;
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            b'}' => {
                if nested_braces == 0 {
                    return Some(cursor);
                }
                nested_braces -= 1;
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
            _ => {
                cursor = advance_js_scan_cursor(source, cursor, source.len());
            }
        }
    }
    None
}

fn js_string_literal_span(
    source: &str,
    quote_offset: usize,
    limit: usize,
) -> Option<(usize, usize, usize)> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    if !matches!(quote, b'\'' | b'"' | b'`') {
        return None;
    }
    let literal_start = quote_offset + 1;
    let next_offset = skip_js_string_literal(source, quote_offset, limit)?;
    Some((literal_start, next_offset - 1, next_offset))
}

fn skip_js_string_literal(source: &str, quote_offset: usize, limit: usize) -> Option<usize> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    let limit = char_boundary_floor(source, limit);
    let mut cursor = quote_offset + 1;
    while cursor < limit {
        let byte = source.as_bytes().get(cursor).copied()?;
        if byte == b'\\' {
            cursor = advance_js_escaped_char(source, cursor, limit);
            continue;
        }
        if byte == quote {
            return Some(cursor + 1);
        }
        cursor = advance_js_scan_cursor(source, cursor, limit);
    }
    None
}

fn bracket_string_literal_access(
    source: &str,
    bracket_offset: usize,
) -> Option<(usize, usize, usize)> {
    if source.as_bytes().get(bracket_offset) != Some(&b'[') {
        return None;
    }
    let quote_offset = skip_ascii_whitespace(source, bracket_offset + 1);
    let quote = source.as_bytes().get(quote_offset).copied()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let (literal_start, literal_end, literal_next) =
        js_string_literal_span(source, quote_offset, source.len())?;
    if literal_next > source.len() {
        return None;
    }
    let closing_bracket = skip_ascii_whitespace(source, literal_end + 1);
    if source.as_bytes().get(closing_bracket) != Some(&b']') {
        return None;
    }
    Some((literal_start, literal_end, closing_bracket + 1))
}

fn read_js_identifier(source: &str, start: usize) -> Option<(&str, usize)> {
    let start = char_boundary_ceil(source, start);
    let first = source.get(start..)?.chars().next()?;
    if !is_js_identifier_start(first) {
        return None;
    }
    let mut end = start + first.len_utf8();
    let scan_start = end;
    for (relative_index, ch) in source.get(scan_start..)?.char_indices() {
        if !is_js_identifier_continue(ch) {
            break;
        }
        end = scan_start + relative_index + ch.len_utf8();
    }
    Some((&source[start..end], end))
}

fn is_js_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

fn is_js_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$')
}

fn is_css_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_target_aware_source_syntax_index_for_css_modules_binding_inputs() {
        let source = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const variants = { primary: "item--primary", icon: "item__icon" };
export function App({ tone }: { tone: "warm" | "cool" }) {
  return <div className={clsx("alert", cx("wrapper", variants.primary, `tone-${tone}`))} />;
}"#;

        let index = summarize_omena_bridge_source_syntax_index(
            source,
            vec![SourceImportedStyleBindingV0 {
                binding: "styles".to_string(),
                style_uri: "file:///workspace/App.module.scss".to_string(),
            }],
            vec!["bind".to_string()],
        );

        assert_eq!(index.product, "omena-bridge.source-syntax-index");
        assert!(index.class_string_literals.is_empty());
        assert!(index.selector_references.iter().any(|reference| {
            selector_reference_name(source, reference) == "wrapper"
                && reference.target_style_uri.as_deref()
                    == Some("file:///workspace/App.module.scss")
        }));
        assert!(index.selector_references.iter().any(|reference| {
            reference.selector_name.as_deref() == Some("item--primary")
                && reference.target_style_uri.as_deref()
                    == Some("file:///workspace/App.module.scss")
        }));
        assert!(index.selector_references.iter().any(|reference| {
            selector_reference_name(source, reference) == "alert"
                && reference.target_style_uri.as_deref().is_none()
        }));
        assert!(index.type_fact_targets.iter().any(|target| {
            &source[target.byte_span.start..target.byte_span.end] == "tone"
                && target.prefix == "tone-"
                && target.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
        }));
    }

    #[test]
    fn collects_style_property_accesses_from_oxc_ast() {
        let source = r#"import styles from "./App.module.scss";
const text = "styles.fake";
export function View() {
  return <div className={styles.root} data-token={styles["item--primary"]} />;
}"#;

        let index = summarize_omena_bridge_source_syntax_index(
            source,
            vec![SourceImportedStyleBindingV0 {
                binding: "styles".to_string(),
                style_uri: "file:///workspace/App.module.scss".to_string(),
            }],
            Vec::new(),
        );

        let access_names = index
            .style_property_accesses
            .iter()
            .map(|access| &source[access.byte_span.start..access.byte_span.end])
            .collect::<Vec<_>>();

        assert_eq!(access_names, vec!["root", "item--primary"]);
        assert!(index.style_property_accesses.iter().all(|access| {
            access.target_style_uri.as_deref() == Some("file:///workspace/App.module.scss")
        }));
        assert!(
            !index
                .selector_references
                .iter()
                .any(|reference| selector_reference_name(source, reference) == "fake")
        );
    }

    #[test]
    fn source_recovery_scanners_keep_multibyte_escape_boundaries()
    -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"const escaped = "\비";
const view = <div className={cx("root", active && `상태-${tone}`)} />;"#;
        let escaped_quote = source
            .find(r#""\비""#)
            .ok_or_else(|| std::io::Error::other("escaped fixture exists"))?;
        let escaped_end = skip_js_string_literal(source, escaped_quote, source.len())
            .ok_or_else(|| std::io::Error::other("escaped string should be skipped"))?;
        assert!(source.is_char_boundary(escaped_end));

        let expression_start = source
            .find("cx(")
            .ok_or_else(|| std::io::Error::other("cx call exists"))?
            + "cx(".len();
        let expression_end = js_call_end(source, expression_start - 1)
            .ok_or_else(|| std::io::Error::other("cx call ends"))?;
        let segments = split_top_level_js_segments(source, expression_start, expression_end, b',');
        assert_eq!(segments.len(), 2);
        for (start, end) in segments {
            assert!(source.is_char_boundary(start));
            assert!(source.is_char_boundary(end));
        }

        let operator = find_top_level_js_operator(source, expression_start, expression_end, "&&")
            .ok_or_else(|| {
            std::io::Error::other(
                "conditional operator should be found without slicing inside UTF-8",
            )
        })?;
        assert!(source.is_char_boundary(operator));
        Ok(())
    }

    fn selector_reference_name<'a>(
        source: &'a str,
        reference: &'a SourceSelectorReferenceFactV0,
    ) -> &'a str {
        reference
            .selector_name
            .as_deref()
            .unwrap_or(&source[reference.byte_span.start..reference.byte_span.end])
    }
}

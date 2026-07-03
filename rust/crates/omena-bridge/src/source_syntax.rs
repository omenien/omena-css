use omena_parser::ParserByteSpanV0;
use oxc_allocator::Allocator;
use oxc_ast::ast::TSModuleReference;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, BindingIdentifier, BindingPattern,
    CallExpression, ChainElement, Class, ClassElement, ComputedMemberExpression,
    ConditionalExpression, Declaration, Expression, IdentifierReference, ImportDeclaration,
    ImportDeclarationSpecifier, ImportOrExportKind, JSXAttributeName, JSXAttributeValue, JSXChild,
    JSXExpression, LogicalExpression, ObjectExpression, ObjectPropertyKind,
    ParenthesizedExpression, Program, Statement, StaticMemberExpression, TSAsExpression,
    TSNonNullExpression, TSSatisfiesExpression, VariableDeclarator,
};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{Scoping, SemanticBuilder, SymbolId};
use oxc_span::{GetSpan, SourceType, Span};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::source_language::{
    ServerTemplateDelimiterFamilyV0, is_astro_source, is_html_source, is_markdown_source,
    is_server_template_source, is_svelte_source, is_vue_source, project_source_for_language,
    server_template_delimiter_family, source_type_for_language, tag_content_ranges,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSyntaxIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub imported_style_bindings: Vec<SourceImportedStyleBindingV0>,
    pub class_string_literals: Vec<ParserByteSpanV0>,
    pub style_property_accesses: Vec<SourceStylePropertyAccessFactV0>,
    pub inline_style_declarations: Vec<SourceInlineStyleDeclarationFactV0>,
    pub selector_references: Vec<SourceSelectorReferenceFactV0>,
    pub type_fact_targets: Vec<SourceTypeFactTargetV0>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub type_fact_provider_unavailable: Vec<SourceTypeFactProviderUnavailableFactV0>,
    pub class_value_universes: Vec<SourceClassValueUniverseEntryV0>,
    pub domain_class_references: Vec<SourceDomainClassReferenceFactV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImportedStyleBindingV0 {
    pub binding: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBindingIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub binding_scopes: Vec<SourceBindingScopeFactV0>,
    pub scope_parent_edges: Vec<SourceScopeParentFactV0>,
    pub binding_decls: Vec<SourceBindingDeclFactV0>,
    pub scope_contains_decls: Vec<SourceScopeContainsDeclFactV0>,
    pub style_import_bindings: Vec<SourceBindingStyleImportFactV0>,
    pub declares_style_imports: Vec<SourceDeclaresStyleImportFactV0>,
    pub style_import_resolves_modules: Vec<SourceStyleImportResolvesModuleFactV0>,
    pub class_expression_nodes: Vec<SourceClassExpressionNodeFactV0>,
    pub expression_targets_modules: Vec<SourceExpressionTargetsModuleFactV0>,
    pub classnames_bind_utility_bindings: Vec<SourceClassnamesBindUtilityBindingFactV0>,
    pub class_util_bindings: Vec<SourceClassUtilityBindingFactV0>,
    pub declares_utility_bindings: Vec<SourceDeclaresUtilityBindingFactV0>,
    pub utility_uses_style_imports: Vec<SourceUtilityUsesStyleImportFactV0>,
    pub style_access_uses_style_imports: Vec<SourceStyleAccessUsesStyleImportFactV0>,
    pub symbol_ref_uses_decls: Vec<SourceSymbolRefUsesDeclFactV0>,
    pub module_specifiers: Vec<SourceModuleSpecifierFactV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBindingScopeFactV0 {
    pub kind: &'static str,
    pub byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceScopeParentFactV0 {
    pub child_kind: &'static str,
    pub child_byte_span: ParserByteSpanV0,
    pub parent_kind: &'static str,
    pub parent_byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBindingDeclFactV0 {
    pub kind: &'static str,
    pub name: String,
    pub byte_span: ParserByteSpanV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceScopeContainsDeclFactV0 {
    pub scope_kind: &'static str,
    pub scope_byte_span: ParserByteSpanV0,
    pub decl_kind: &'static str,
    pub decl_name: String,
    pub decl_byte_span: ParserByteSpanV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBindingStyleImportFactV0 {
    pub local_name: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDeclaresStyleImportFactV0 {
    pub decl_name: String,
    pub styles_local_name: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStyleImportResolvesModuleFactV0 {
    pub styles_local_name: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceClassExpressionNodeFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub kind: &'static str,
    pub target_style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceExpressionTargetsModuleFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub target_style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceClassnamesBindUtilityBindingFactV0 {
    pub local_name: String,
    pub styles_local_name: String,
    pub style_uri: String,
    pub classnames_import_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceClassUtilityBindingFactV0 {
    pub local_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDeclaresUtilityBindingFactV0 {
    pub decl_name: String,
    pub utility_local_name: String,
    pub utility_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUtilityUsesStyleImportFactV0 {
    pub utility_local_name: String,
    pub styles_local_name: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStyleAccessUsesStyleImportFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub decl_name: String,
    pub styles_local_name: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSymbolRefUsesDeclFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub raw_reference: String,
    pub root_name: String,
    pub decl_name: String,
    pub style_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceModuleSpecifierFactV0 {
    pub kind: &'static str,
    pub specifier: String,
    pub byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStylePropertyAccessFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub target_style_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInlineStyleDeclarationFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub value_byte_span: Option<ParserByteSpanV0>,
    pub property_name: String,
    pub value: Option<String>,
    pub target_style_uri: Option<String>,
    pub cascade_tier: &'static str,
    pub static_value: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceTypeFactProviderUnavailableFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub expression_id: String,
    pub target_style_uri: Option<String>,
    pub provider_id: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceClassValueUniverseEntryV0 {
    pub plugin_id: &'static str,
    pub domain: &'static str,
    pub owner_name: String,
    pub class_names: Vec<String>,
    pub axes: Vec<SourceClassValueUniverseAxisV0>,
    pub byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceClassValueUniverseAxisV0 {
    pub axis_name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDomainClassReferenceFactV0 {
    pub byte_span: ParserByteSpanV0,
    pub plugin_id: &'static str,
    pub domain: &'static str,
    pub owner_name: String,
    pub axis_name: String,
    pub option_name: Option<String>,
    pub prefix: Option<String>,
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
    binding_symbol_id: Option<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassnamesBindUtilityBinding {
    binding: String,
    binding_symbol_id: SymbolId,
    styles_binding: String,
    style_uri: String,
    classnames_import_binding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassnamesBindCallArgument {
    binding: String,
    binding_symbol_id: SymbolId,
    byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SymbolRefClassValueBinding {
    classnames_binding_symbol_id: SymbolId,
    byte_span: ParserByteSpanV0,
    raw_reference: String,
    root_name: String,
    decl_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TemplateScanScope {
    WholeDocument,
    Ranges(Vec<(usize, usize)>),
}

impl TemplateScanScope {
    fn as_ranges(&self) -> Option<&[(usize, usize)]> {
        match self {
            Self::WholeDocument => None,
            Self::Ranges(ranges) => Some(ranges.as_slice()),
        }
    }
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
    summarize_omena_bridge_source_syntax_index_for_source_language(
        "source.tsx",
        source,
        None,
        imported_style_bindings,
        classnames_bind_bindings,
    )
}

pub fn summarize_omena_bridge_source_syntax_index_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    imported_style_bindings: Vec<SourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> SourceSyntaxIndexV0 {
    let projected_source = project_source_for_language(source_path, source, source_language);
    let imported_style_targets = imported_style_targets(imported_style_bindings.as_slice());
    let property_access_targets = property_access_style_targets(imported_style_bindings.as_slice());
    let ast_facts = collect_source_syntax_ast_facts(
        projected_source.as_ref(),
        source_type_for_language(source_path, source_language),
        property_access_targets.as_slice(),
        imported_style_targets.as_slice(),
        classnames_bind_bindings.as_slice(),
    );
    let class_string_literals = ast_facts.class_string_literals;
    let style_property_accesses = ast_facts.style_property_accesses;
    let class_name_expression_spans = ast_facts.class_name_expression_spans;
    let classnames_bind_targets = ast_facts.classnames_bind_utility_bindings;
    let classnames_bind_call_arguments = ast_facts.classnames_bind_call_arguments;
    let local_class_values = collect_local_class_value_bindings(projected_source.as_ref());

    let mut index = SourceSyntaxIndexV0 {
        schema_version: "0",
        product: "omena-bridge.source-syntax-index",
        imported_style_bindings,
        class_string_literals,
        style_property_accesses,
        inline_style_declarations: ast_facts.inline_style_declarations,
        selector_references: Vec::new(),
        type_fact_targets: Vec::new(),
        type_fact_provider_unavailable: Vec::new(),
        class_value_universes: ast_facts.class_value_universes,
        domain_class_references: ast_facts.domain_class_references,
    };

    for span in &index.class_string_literals {
        push_string_literal_selector_references(
            source,
            *span,
            None,
            &mut index.selector_references,
        );
    }
    for span in class_name_expression_spans {
        collect_selector_references_from_js_expression(
            source,
            span.start,
            span.end,
            None,
            &local_class_values,
            &mut index.selector_references,
            &mut index.type_fact_targets,
        );
    }
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
    for argument in classnames_bind_call_arguments {
        if let Some(binding) = classnames_bind_targets
            .iter()
            .find(|binding| binding.binding_symbol_id == argument.binding_symbol_id)
        {
            collect_selector_references_from_js_expression(
                source,
                argument.byte_span.start,
                argument.byte_span.end,
                Some(binding.style_uri.as_str()),
                &local_class_values,
                &mut index.selector_references,
                &mut index.type_fact_targets,
            );
        }
    }
    collect_template_class_attribute_selector_references(
        source_path,
        source,
        source_language,
        &mut index.selector_references,
    );
    collect_template_class_expression_selector_references(
        source_path,
        source,
        source_language,
        imported_style_targets.as_slice(),
        &mut index.selector_references,
    );
    canonicalize_source_selector_references(&mut index.selector_references);

    index
}

pub(crate) fn summarize_source_control_flow_graph_with_semantic(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<crate::source_cfg::SourceControlFlowGraphCaptureV0> {
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

    let semantic = SemanticBuilder::new().build(&program).semantic;
    crate::source_cfg::summarize_source_control_flow_graph_from_program(
        &program,
        semantic.scoping(),
        variable_name,
        reference_byte_offset,
    )
}

pub fn summarize_omena_bridge_source_binding_index(
    source: &str,
    imported_style_bindings: Vec<SourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> SourceBindingIndexV0 {
    summarize_omena_bridge_source_binding_index_for_source_language(
        "source.tsx",
        source,
        None,
        imported_style_bindings,
        classnames_bind_bindings,
    )
}

pub fn summarize_omena_bridge_source_binding_index_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    imported_style_bindings: Vec<SourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> SourceBindingIndexV0 {
    let syntax_index = summarize_omena_bridge_source_syntax_index_for_source_language(
        source_path,
        source,
        source_language,
        imported_style_bindings.clone(),
        classnames_bind_bindings.clone(),
    );
    let projected_source = project_source_for_language(source_path, source, source_language);
    let imported_style_targets = imported_style_targets(imported_style_bindings.as_slice());
    let property_access_targets = property_access_style_targets(imported_style_bindings.as_slice());
    let ast_facts = collect_source_syntax_ast_facts(
        projected_source.as_ref(),
        source_type_for_language(source_path, source_language),
        property_access_targets.as_slice(),
        imported_style_targets.as_slice(),
        classnames_bind_bindings.as_slice(),
    );
    let mut binding_scopes = ast_facts.binding_scopes;
    binding_scopes.sort();
    binding_scopes.dedup();
    let mut scope_parent_edges = ast_facts.scope_parent_edges;
    scope_parent_edges.sort();
    scope_parent_edges.dedup();
    let mut binding_decls = ast_facts.binding_decls;
    binding_decls.sort();
    binding_decls.dedup();
    let mut scope_contains_decls = ast_facts.scope_contains_decls;
    scope_contains_decls.sort();
    scope_contains_decls.dedup();
    let mut style_import_bindings = imported_style_targets
        .iter()
        .filter_map(|target| {
            target
                .target_style_uri
                .as_ref()
                .map(|style_uri| SourceBindingStyleImportFactV0 {
                    local_name: target.binding.clone(),
                    style_uri: style_uri.clone(),
                })
        })
        .collect::<Vec<_>>();
    style_import_bindings.sort();
    style_import_bindings.dedup();
    let style_import_local_names_by_uri =
        style_import_local_names_by_uri(style_import_bindings.as_slice());
    let mut declares_style_imports = style_import_bindings
        .iter()
        .map(|binding| SourceDeclaresStyleImportFactV0 {
            decl_name: binding.local_name.clone(),
            styles_local_name: binding.local_name.clone(),
            style_uri: binding.style_uri.clone(),
        })
        .collect::<Vec<_>>();
    declares_style_imports.sort();
    declares_style_imports.dedup();
    let mut style_import_resolves_modules = style_import_bindings
        .iter()
        .map(|binding| SourceStyleImportResolvesModuleFactV0 {
            styles_local_name: binding.local_name.clone(),
            style_uri: binding.style_uri.clone(),
        })
        .collect::<Vec<_>>();
    style_import_resolves_modules.sort();
    style_import_resolves_modules.dedup();
    let classnames_bind_targets = ast_facts.classnames_bind_utility_bindings.clone();
    let style_access_expression_keys = syntax_index
        .style_property_accesses
        .iter()
        .filter_map(|access| {
            access.target_style_uri.as_ref().map(|style_uri| {
                (
                    access.byte_span.start,
                    access.byte_span.end,
                    style_uri.clone(),
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let symbol_ref_expression_keys = ast_facts
        .symbol_ref_class_value_bindings
        .iter()
        .filter_map(|reference| {
            let binding = classnames_bind_targets.iter().find(|binding| {
                binding.binding_symbol_id == reference.classnames_binding_symbol_id
            })?;
            Some((
                reference.byte_span.start,
                reference.byte_span.end,
                binding.style_uri.clone(),
            ))
        })
        .collect::<BTreeSet<_>>();
    let mut class_expression_nodes = syntax_index
        .selector_references
        .iter()
        .filter_map(|reference| {
            let target_style_uri = reference.target_style_uri.clone()?;
            let expression_key = (
                reference.byte_span.start,
                reference.byte_span.end,
                target_style_uri.clone(),
            );
            let kind = if style_access_expression_keys.contains(&expression_key) {
                "styleAccess"
            } else if symbol_ref_expression_keys.contains(&expression_key) {
                "symbolRef"
            } else {
                match reference.match_kind {
                    SourceSelectorReferenceMatchKindV0::Exact => "literal",
                    SourceSelectorReferenceMatchKindV0::Prefix => "template",
                }
            };
            Some(SourceClassExpressionNodeFactV0 {
                kind,
                byte_span: reference.byte_span,
                target_style_uri,
            })
        })
        .collect::<Vec<_>>();
    class_expression_nodes.extend(ast_facts.symbol_ref_class_value_bindings.iter().filter_map(
        |reference| {
            let binding = classnames_bind_targets.iter().find(|binding| {
                binding.binding_symbol_id == reference.classnames_binding_symbol_id
            })?;
            Some(SourceClassExpressionNodeFactV0 {
                kind: "symbolRef",
                byte_span: reference.byte_span,
                target_style_uri: binding.style_uri.clone(),
            })
        },
    ));
    class_expression_nodes.sort();
    class_expression_nodes.dedup();
    let mut expression_targets_modules = syntax_index
        .selector_references
        .iter()
        .filter_map(|reference| {
            reference.target_style_uri.clone().map(|target_style_uri| {
                SourceExpressionTargetsModuleFactV0 {
                    byte_span: reference.byte_span,
                    target_style_uri,
                }
            })
        })
        .collect::<Vec<_>>();
    expression_targets_modules.extend(ast_facts.symbol_ref_class_value_bindings.iter().filter_map(
        |reference| {
            let binding = classnames_bind_targets.iter().find(|binding| {
                binding.binding_symbol_id == reference.classnames_binding_symbol_id
            })?;
            Some(SourceExpressionTargetsModuleFactV0 {
                byte_span: reference.byte_span,
                target_style_uri: binding.style_uri.clone(),
            })
        },
    ));
    expression_targets_modules.sort();
    expression_targets_modules.dedup();
    let mut classnames_bind_utility_bindings = ast_facts
        .classnames_bind_utility_bindings
        .into_iter()
        .map(|binding| SourceClassnamesBindUtilityBindingFactV0 {
            local_name: binding.binding,
            styles_local_name: binding.styles_binding,
            style_uri: binding.style_uri,
            classnames_import_name: binding.classnames_import_binding,
        })
        .collect::<Vec<_>>();
    classnames_bind_utility_bindings.sort();
    classnames_bind_utility_bindings.dedup();
    let mut class_util_bindings = binding_decls
        .iter()
        .filter_map(|decl| {
            let import_path = decl.import_path.as_deref()?;
            if decl.kind == "import" && is_class_utility_import_path(import_path) {
                Some(SourceClassUtilityBindingFactV0 {
                    local_name: decl.name.clone(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    class_util_bindings.sort();
    class_util_bindings.dedup();
    let mut declares_utility_bindings = classnames_bind_utility_bindings
        .iter()
        .map(|binding| SourceDeclaresUtilityBindingFactV0 {
            decl_name: binding.local_name.clone(),
            utility_local_name: binding.local_name.clone(),
            utility_kind: "classnamesBind",
        })
        .collect::<Vec<_>>();
    declares_utility_bindings.extend(class_util_bindings.iter().map(|binding| {
        SourceDeclaresUtilityBindingFactV0 {
            decl_name: binding.local_name.clone(),
            utility_local_name: binding.local_name.clone(),
            utility_kind: "classUtil",
        }
    }));
    declares_utility_bindings.sort();
    declares_utility_bindings.dedup();
    let mut utility_uses_style_imports = classnames_bind_utility_bindings
        .iter()
        .map(|binding| SourceUtilityUsesStyleImportFactV0 {
            utility_local_name: binding.local_name.clone(),
            styles_local_name: binding.styles_local_name.clone(),
            style_uri: binding.style_uri.clone(),
        })
        .collect::<Vec<_>>();
    utility_uses_style_imports.sort();
    utility_uses_style_imports.dedup();
    let mut style_access_uses_style_imports = syntax_index
        .style_property_accesses
        .iter()
        .filter_map(|access| {
            let style_uri = access.target_style_uri.as_ref()?;
            let local_names = style_import_local_names_by_uri.get(style_uri)?;
            let styles_local_name = single_btree_set_item(local_names)?;
            Some(SourceStyleAccessUsesStyleImportFactV0 {
                byte_span: access.byte_span,
                decl_name: styles_local_name.clone(),
                styles_local_name: styles_local_name.clone(),
                style_uri: style_uri.clone(),
            })
        })
        .collect::<Vec<_>>();
    style_access_uses_style_imports.sort();
    style_access_uses_style_imports.dedup();
    let mut symbol_ref_uses_decls = ast_facts
        .symbol_ref_class_value_bindings
        .into_iter()
        .filter_map(|reference| {
            let binding = classnames_bind_targets.iter().find(|binding| {
                binding.binding_symbol_id == reference.classnames_binding_symbol_id
            })?;
            Some(SourceSymbolRefUsesDeclFactV0 {
                byte_span: reference.byte_span,
                raw_reference: reference.raw_reference,
                root_name: reference.root_name,
                decl_name: reference.decl_name,
                style_uri: binding.style_uri.clone(),
            })
        })
        .collect::<Vec<_>>();
    symbol_ref_uses_decls.sort();
    symbol_ref_uses_decls.dedup();
    let mut module_specifiers = ast_facts.module_specifiers;
    module_specifiers.sort();
    module_specifiers.dedup();

    SourceBindingIndexV0 {
        schema_version: "0",
        product: "omena-bridge.source-binding-index",
        binding_scopes,
        scope_parent_edges,
        binding_decls,
        scope_contains_decls,
        style_import_bindings,
        declares_style_imports,
        style_import_resolves_modules,
        class_expression_nodes,
        expression_targets_modules,
        classnames_bind_utility_bindings,
        class_util_bindings,
        declares_utility_bindings,
        utility_uses_style_imports,
        style_access_uses_style_imports,
        symbol_ref_uses_decls,
        module_specifiers,
    }
}

pub fn collect_omena_bridge_vue_style_module_bindings(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
) -> Vec<String> {
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
        return Vec::new();
    }
    collect_vue_use_css_module_bindings(&program)
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

fn collect_template_class_attribute_selector_references(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    let Some(scan_scope) = template_source_scan_scope(source_path, source, source_language) else {
        return;
    };
    let delimiter_family = server_template_delimiter_family(source_path, source_language);

    let mut suppressed_ranges = tag_content_ranges(source, "<script", "</script>");
    suppressed_ranges.extend(tag_content_ranges(source, "<style", "</style>"));
    suppressed_ranges.sort_unstable();

    for value_span in template_class_attribute_value_spans(
        source,
        scan_scope.as_ranges(),
        suppressed_ranges.as_slice(),
        delimiter_family.is_some(),
    ) {
        if let Some(family) = delimiter_family {
            push_server_template_class_attribute_selector_references(
                source, value_span, family, references,
            );
        } else {
            push_string_literal_selector_references(source, value_span, None, references);
        }
    }
}

fn collect_template_class_expression_selector_references(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    style_targets: &[SourceStyleBindingTarget],
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    if style_targets.is_empty() {
        return;
    }
    let Some(scan_scope) = template_source_scan_scope(source_path, source, source_language) else {
        return;
    };

    let mut suppressed_ranges = tag_content_ranges(source, "<script", "</script>");
    suppressed_ranges.extend(tag_content_ranges(source, "<style", "</style>"));
    suppressed_ranges.sort_unstable();

    for expression_span in template_class_expression_spans(
        source,
        scan_scope.as_ranges(),
        suppressed_ranges.as_slice(),
    ) {
        for target in style_targets {
            push_style_binding_selector_references_from_expression(
                source,
                expression_span,
                target,
                references,
            );
        }
    }
}

fn is_html_like_template_source(source_path: &str, source_language: Option<&str>) -> bool {
    is_vue_source(source_path, source_language)
        || is_html_source(source_path, source_language)
        || is_svelte_source(source_path, source_language)
        || is_astro_source(source_path, source_language)
        || is_server_template_source(source_path, source_language)
}

fn template_source_scan_scope(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
) -> Option<TemplateScanScope> {
    if is_html_like_template_source(source_path, source_language) {
        return Some(TemplateScanScope::WholeDocument);
    }
    if is_markdown_source(source_path, source_language) {
        return Some(TemplateScanScope::Ranges(markdown_html_template_ranges(
            source,
        )));
    }
    None
}

fn template_class_attribute_value_spans(
    source: &str,
    scan_ranges: Option<&[(usize, usize)]>,
    suppressed_ranges: &[(usize, usize)],
    allow_server_template_interpolation: bool,
) -> Vec<ParserByteSpanV0> {
    let lower = source.to_ascii_lowercase();
    let bytes = source.as_bytes();
    let mut cursor = 0usize;
    let mut spans = Vec::new();

    while let Some(relative_start) = lower[cursor..].find("class") {
        let attr_start = cursor + relative_start;
        cursor = attr_start + "class".len();
        if !byte_in_optional_ranges(attr_start, scan_ranges)
            || byte_in_ranges(attr_start, suppressed_ranges)
            || !is_template_class_attribute_name(source, attr_start)
        {
            continue;
        }

        let mut index = skip_ascii_whitespace_bytes(bytes, attr_start + "class".len());
        if bytes.get(index) != Some(&b'=') {
            continue;
        }
        index = skip_ascii_whitespace_bytes(bytes, index + 1);
        let Some((value_start, value_end)) = template_attribute_value_span(bytes, index) else {
            continue;
        };
        if value_start < value_end
            && (allow_server_template_interpolation
                || is_static_template_class_attribute_value(source, value_start, value_end))
        {
            spans.push(ParserByteSpanV0 {
                start: value_start,
                end: value_end,
            });
        }
        cursor = value_end;
    }

    spans
}

fn template_class_expression_spans(
    source: &str,
    scan_ranges: Option<&[(usize, usize)]>,
    suppressed_ranges: &[(usize, usize)],
) -> Vec<ParserByteSpanV0> {
    let lower = source.to_ascii_lowercase();
    let bytes = source.as_bytes();
    let mut cursor = 0usize;
    let mut spans = Vec::new();

    while let Some(relative_start) = lower[cursor..].find("class") {
        let attr_start = cursor + relative_start;
        cursor = attr_start + "class".len();
        if !byte_in_optional_ranges(attr_start, scan_ranges)
            || byte_in_ranges(attr_start, suppressed_ranges)
        {
            continue;
        }

        let is_dynamic_attr = is_template_dynamic_class_attribute_name(source, attr_start);
        let is_literal_attr = is_template_class_attribute_name(source, attr_start);
        if !is_dynamic_attr && !is_literal_attr {
            continue;
        }

        let mut index = skip_ascii_whitespace_bytes(bytes, attr_start + "class".len());
        if bytes.get(index) != Some(&b'=') {
            continue;
        }
        index = skip_ascii_whitespace_bytes(bytes, index + 1);
        let Some((value_start, value_end)) = template_attribute_value_span(bytes, index) else {
            continue;
        };
        let expression_span = if is_dynamic_attr {
            ParserByteSpanV0 {
                start: value_start,
                end: value_end,
            }
        } else if value_start < value_end
            && bytes.get(value_start) == Some(&b'{')
            && bytes.get(value_end - 1) == Some(&b'}')
        {
            ParserByteSpanV0 {
                start: value_start + 1,
                end: value_end - 1,
            }
        } else {
            continue;
        };
        let (start, end) = trim_js_expression(source, expression_span.start, expression_span.end);
        if start < end {
            spans.push(ParserByteSpanV0 { start, end });
        }
        cursor = value_end;
    }

    spans
}

fn markdown_html_template_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut open_fence: Option<(char, usize)> = None;
    let mut open_html_start: Option<usize> = None;
    let mut offset = 0usize;

    for line in source.split_inclusive('\n') {
        let line_start = offset;
        let line_end = offset + line.len();
        let line_without_newline = line.trim_end_matches(['\r', '\n']);
        let leading_spaces = line_without_newline
            .chars()
            .take_while(|ch| *ch == ' ')
            .count();
        let trimmed = line_without_newline.trim_start_matches(' ');
        if leading_spaces <= 3 {
            if let Some((fence_char, fence_len)) = open_fence {
                if markdown_fence_marker_for_template_scan(trimmed).is_some_and(
                    |(candidate_char, candidate_len)| {
                        candidate_char == fence_char && candidate_len >= fence_len
                    },
                ) {
                    open_fence = None;
                }
                offset = line_end;
                continue;
            }
            if let Some((fence_char, fence_len)) = markdown_fence_marker_for_template_scan(trimmed)
            {
                open_fence = Some((fence_char, fence_len));
                offset = line_end;
                continue;
            }
        }

        if let Some(start) = open_html_start {
            if trimmed.contains('>') {
                ranges.push((start, line_end));
                open_html_start = None;
            }
            offset = line_end;
            continue;
        }

        if leading_spaces >= 4 || trimmed.is_empty() {
            offset = line_end;
            continue;
        }

        if markdown_line_starts_html_tag(trimmed) {
            if trimmed.contains('>') {
                ranges.push((line_start, line_end));
            } else {
                open_html_start = Some(line_start);
            }
        }

        offset = line_end;
    }

    if let Some(start) = open_html_start {
        ranges.push((start, source.len()));
    }
    ranges
}

fn markdown_line_starts_html_tag(trimmed_line: &str) -> bool {
    let mut chars = trimmed_line.chars();
    if chars.next() != Some('<') {
        return false;
    }
    match chars.next() {
        Some('/') => chars.next().is_some_and(is_html_tag_name_start),
        Some('!') | Some('?') => false,
        Some(ch) => is_html_tag_name_start(ch),
        None => false,
    }
}

fn is_html_tag_name_start(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn markdown_fence_marker_for_template_scan(line: &str) -> Option<(char, usize)> {
    let mut chars = line.chars();
    let fence_char = chars.next()?;
    if fence_char != '`' && fence_char != '~' {
        return None;
    }
    let fence_len = 1 + chars.take_while(|ch| *ch == fence_char).count();
    if fence_len >= 3 {
        Some((fence_char, fence_len))
    } else {
        None
    }
}

fn is_template_class_attribute_name(source: &str, attr_start: usize) -> bool {
    let bytes = source.as_bytes();
    let before = attr_start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .copied();
    if before.is_some_and(is_html_attribute_name_byte) {
        return false;
    }
    let after = attr_start + "class".len();
    bytes
        .get(after)
        .is_none_or(|byte| !is_html_attribute_name_byte(*byte))
}

fn is_template_dynamic_class_attribute_name(source: &str, attr_start: usize) -> bool {
    let bytes = source.as_bytes();
    if attr_start >= 1
        && bytes.get(attr_start - 1) == Some(&b':')
        && attr_start
            .checked_sub(2)
            .and_then(|index| bytes.get(index))
            .is_none_or(|byte| !is_html_attribute_name_byte(*byte))
    {
        return true;
    }
    let prefix = "v-bind:";
    if attr_start < prefix.len() {
        return false;
    }
    let prefix_start = attr_start - prefix.len();
    source
        .get(prefix_start..attr_start)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
        && prefix_start
            .checked_sub(1)
            .and_then(|index| bytes.get(index))
            .is_none_or(|byte| !is_html_attribute_name_byte(*byte))
}

fn is_html_attribute_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_' | b'.')
}

fn skip_ascii_whitespace_bytes(bytes: &[u8], mut index: usize) -> usize {
    while bytes.get(index).is_some_and(u8::is_ascii_whitespace) {
        index += 1;
    }
    index
}

fn template_attribute_value_span(bytes: &[u8], value_start: usize) -> Option<(usize, usize)> {
    let quote = *bytes.get(value_start)?;
    if quote == b'\'' || quote == b'"' {
        let content_start = value_start + 1;
        let relative_end = bytes
            .get(content_start..)?
            .iter()
            .position(|byte| *byte == quote)?;
        return Some((content_start, content_start + relative_end));
    }

    let relative_end = bytes
        .get(value_start..)?
        .iter()
        .position(|byte| byte.is_ascii_whitespace() || *byte == b'>')
        .unwrap_or_else(|| bytes.len().saturating_sub(value_start));
    Some((value_start, value_start + relative_end))
}

fn is_static_template_class_attribute_value(source: &str, start: usize, end: usize) -> bool {
    source
        .get(start..end)
        .is_some_and(|value| !value.contains(['{', '}']))
}

fn push_server_template_class_attribute_selector_references(
    source: &str,
    literal_span: ParserByteSpanV0,
    family: ServerTemplateDelimiterFamilyV0,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    let interpolation_ranges =
        server_template_interpolation_ranges(source, literal_span.start, literal_span.end, family);
    if interpolation_ranges.is_empty() {
        if is_static_template_class_attribute_value(source, literal_span.start, literal_span.end) {
            push_string_literal_selector_references(source, literal_span, None, references);
        }
        return;
    }

    for span in class_token_byte_spans(source, literal_span.start, literal_span.end) {
        if token_intersects_dynamic_template_segment(source, span, interpolation_ranges.as_slice())
        {
            continue;
        }
        references.push(SourceSelectorReferenceFactV0 {
            byte_span: span,
            selector_name: None,
            match_kind: SourceSelectorReferenceMatchKindV0::Exact,
            target_style_uri: None,
        });
    }
}

fn server_template_interpolation_ranges(
    source: &str,
    value_start: usize,
    value_end: usize,
    family: ServerTemplateDelimiterFamilyV0,
) -> Vec<ParserByteSpanV0> {
    match family {
        ServerTemplateDelimiterFamilyV0::LiquidLike => collect_delimited_template_ranges(
            source,
            value_start,
            value_end,
            &[("{{", "}}"), ("{%", "%}"), ("{#", "#}")],
        ),
        ServerTemplateDelimiterFamilyV0::ErbLike => {
            collect_delimited_template_ranges(source, value_start, value_end, &[("<%", "%>")])
        }
        ServerTemplateDelimiterFamilyV0::Handlebars => collect_delimited_template_ranges(
            source,
            value_start,
            value_end,
            &[("{{{", "}}}"), ("{{", "}}")],
        ),
    }
}

fn collect_delimited_template_ranges(
    source: &str,
    value_start: usize,
    value_end: usize,
    delimiters: &[(&'static str, &'static str)],
) -> Vec<ParserByteSpanV0> {
    let mut ranges = Vec::new();
    let mut cursor = value_start;
    while cursor < value_end {
        let Some((range_start, open, close)) =
            next_template_delimiter(source, cursor, value_end, delimiters)
        else {
            break;
        };
        let content_start = range_start + open.len();
        let Some(relative_end) = source
            .get(content_start..value_end)
            .and_then(|value| value.find(close))
        else {
            break;
        };
        let range_end = content_start + relative_end + close.len();
        ranges.push(ParserByteSpanV0 {
            start: range_start,
            end: range_end,
        });
        cursor = range_end;
    }
    ranges
}

fn next_template_delimiter(
    source: &str,
    cursor: usize,
    limit: usize,
    delimiters: &[(&'static str, &'static str)],
) -> Option<(usize, &'static str, &'static str)> {
    let haystack = source.get(cursor..limit)?;
    delimiters
        .iter()
        .filter_map(|(open, close)| {
            haystack
                .find(open)
                .map(|relative_start| (cursor + relative_start, *open, *close))
        })
        .min_by(|(left_start, left_open, _), (right_start, right_open, _)| {
            left_start
                .cmp(right_start)
                .then_with(|| right_open.len().cmp(&left_open.len()))
        })
}

fn token_intersects_dynamic_template_segment(
    source: &str,
    token_span: ParserByteSpanV0,
    interpolation_ranges: &[ParserByteSpanV0],
) -> bool {
    interpolation_ranges.iter().any(|range| {
        spans_overlap(token_span, *range)
            || adjacent_without_ascii_whitespace(source, token_span.end, range.start)
            || adjacent_without_ascii_whitespace(source, range.end, token_span.start)
    })
}

fn spans_overlap(left: ParserByteSpanV0, right: ParserByteSpanV0) -> bool {
    left.start < right.end && right.start < left.end
}

fn adjacent_without_ascii_whitespace(source: &str, left_end: usize, right_start: usize) -> bool {
    if left_end > right_start {
        return false;
    }
    source
        .get(left_end..right_start)
        .is_some_and(|between| !between.chars().any(|ch| ch.is_ascii_whitespace()))
}

fn push_style_binding_selector_references_from_expression(
    source: &str,
    expression_span: ParserByteSpanV0,
    target: &SourceStyleBindingTarget,
    references: &mut Vec<SourceSelectorReferenceFactV0>,
) {
    let Some(expression) = source.get(expression_span.start..expression_span.end) else {
        return;
    };
    let mut cursor = 0usize;
    while let Some(relative_start) = expression[cursor..].find(target.binding.as_str()) {
        let binding_start = expression_span.start + cursor + relative_start;
        let binding_end = binding_start + target.binding.len();
        cursor += relative_start + target.binding.len();
        if !is_js_identifier_boundary(source, binding_start, binding_end) {
            continue;
        }

        let access_start = skip_ascii_whitespace(source, binding_end);
        if source.as_bytes().get(access_start) == Some(&b'.') {
            let property_start = skip_ascii_whitespace(source, access_start + 1);
            if let Some((_, property_end)) = read_js_identifier(source, property_start) {
                let span = ParserByteSpanV0 {
                    start: property_start,
                    end: property_end,
                };
                if source[span.start..span.end]
                    .chars()
                    .all(is_css_identifier_continue)
                {
                    push_selector_reference(
                        span,
                        Some(source[span.start..span.end].to_string()),
                        SourceSelectorReferenceMatchKindV0::Exact,
                        target.target_style_uri.as_deref(),
                        references,
                    );
                }
            }
            continue;
        }

        if let Some((literal_start, literal_end, _)) =
            bracket_string_literal_access(source, access_start)
        {
            push_selector_reference(
                ParserByteSpanV0 {
                    start: literal_start,
                    end: literal_end,
                },
                Some(source[literal_start..literal_end].to_string()),
                SourceSelectorReferenceMatchKindV0::Exact,
                target.target_style_uri.as_deref(),
                references,
            );
        }
    }
}

fn is_js_identifier_boundary(source: &str, start: usize, end: usize) -> bool {
    let before = start
        .checked_sub(1)
        .and_then(|index| source.get(index..start))
        .and_then(|text| text.chars().next());
    let after = source.get(end..).and_then(|text| text.chars().next());
    before.is_none_or(|ch| !is_js_identifier_continue(ch))
        && after.is_none_or(|ch| !is_js_identifier_continue(ch))
}

fn byte_in_ranges(byte_offset: usize, ranges: &[(usize, usize)]) -> bool {
    ranges
        .iter()
        .any(|(start, end)| byte_offset >= *start && byte_offset < *end)
}

fn byte_in_optional_ranges(byte_offset: usize, ranges: Option<&[(usize, usize)]>) -> bool {
    ranges.is_none_or(|ranges| byte_in_ranges(byte_offset, ranges))
}

fn imported_style_targets(
    bindings: &[SourceImportedStyleBindingV0],
) -> Vec<SourceStyleBindingTarget> {
    bindings
        .iter()
        .map(|binding| SourceStyleBindingTarget {
            binding: binding.binding.clone(),
            target_style_uri: Some(binding.style_uri.clone()),
            binding_symbol_id: None,
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
            binding_symbol_id: None,
        }]
    } else {
        imported
    }
}

fn source_style_targets_with_symbols(
    targets: &[SourceStyleBindingTarget],
    program: &Program<'_>,
) -> Vec<SourceStyleBindingTarget> {
    let import_symbols = import_local_symbol_ids_by_name(program);
    let local_symbols = top_level_local_symbol_ids_by_name(program);
    targets
        .iter()
        .map(|target| SourceStyleBindingTarget {
            binding: target.binding.clone(),
            target_style_uri: target.target_style_uri.clone(),
            binding_symbol_id: import_symbols
                .get(target.binding.as_str())
                .or_else(|| local_symbols.get(target.binding.as_str()))
                .copied(),
        })
        .collect()
}

fn classnames_bind_import_symbol_ids(
    program: &Program<'_>,
    classnames_bind_imports: &[String],
) -> BTreeSet<SymbolId> {
    let import_symbols = import_local_symbol_ids_by_name(program);
    classnames_bind_imports
        .iter()
        .filter_map(|binding| import_symbols.get(binding.as_str()).copied())
        .collect()
}

fn import_local_symbol_ids_by_name(program: &Program<'_>) -> BTreeMap<String, SymbolId> {
    let mut symbols = BTreeMap::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        collect_import_local_symbol_ids(import, &mut symbols);
    }
    symbols
}

fn top_level_local_symbol_ids_by_name(program: &Program<'_>) -> BTreeMap<String, SymbolId> {
    let mut symbols = BTreeMap::new();
    for statement in &program.body {
        collect_top_level_local_symbol_ids_from_statement(statement, &mut symbols);
    }
    symbols
}

fn collect_top_level_local_symbol_ids_from_statement(
    statement: &Statement<'_>,
    symbols: &mut BTreeMap<String, SymbolId>,
) {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            collect_top_level_local_symbol_ids_from_variable_declaration(declaration, symbols);
        }
        Statement::ExportNamedDeclaration(declaration) => {
            if let Some(Declaration::VariableDeclaration(declaration)) = &declaration.declaration {
                collect_top_level_local_symbol_ids_from_variable_declaration(declaration, symbols);
            }
        }
        _ => {}
    }
}

fn collect_top_level_local_symbol_ids_from_variable_declaration(
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
    symbols: &mut BTreeMap<String, SymbolId>,
) {
    for declarator in &declaration.declarations {
        if let Some(identifier) = binding_pattern_identifier(&declarator.id)
            && let Some(symbol_id) = binding_identifier_symbol_id(identifier)
        {
            symbols.insert(identifier.name.as_str().to_string(), symbol_id);
        }
    }
}

fn collect_import_local_symbol_ids(
    import: &ImportDeclaration<'_>,
    symbols: &mut BTreeMap<String, SymbolId>,
) {
    if import.import_kind != ImportOrExportKind::Value {
        return;
    }
    let Some(specifiers) = import.specifiers.as_ref() else {
        return;
    };
    for specifier in specifiers {
        let local = match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => &specifier.local,
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => &specifier.local,
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => &specifier.local,
        };
        if let Some(symbol_id) = binding_identifier_symbol_id(local) {
            symbols.insert(local.name.as_str().to_string(), symbol_id);
        }
    }
}

fn reference_symbol_id(
    scoping: &Scoping,
    identifier: &IdentifierReference<'_>,
) -> Option<SymbolId> {
    identifier
        .reference_id
        .get()
        .and_then(|reference_id| scoping.get_reference(reference_id).symbol_id())
}

fn binding_identifier_symbol_id(identifier: &BindingIdentifier<'_>) -> Option<SymbolId> {
    identifier.symbol_id.get()
}

struct SourceSyntaxAstFacts {
    binding_scopes: Vec<SourceBindingScopeFactV0>,
    scope_parent_edges: Vec<SourceScopeParentFactV0>,
    binding_decls: Vec<SourceBindingDeclFactV0>,
    scope_contains_decls: Vec<SourceScopeContainsDeclFactV0>,
    class_string_literals: Vec<ParserByteSpanV0>,
    style_property_accesses: Vec<SourceStylePropertyAccessFactV0>,
    inline_style_declarations: Vec<SourceInlineStyleDeclarationFactV0>,
    class_name_expression_spans: Vec<ParserByteSpanV0>,
    classnames_bind_utility_bindings: Vec<ClassnamesBindUtilityBinding>,
    classnames_bind_call_arguments: Vec<ClassnamesBindCallArgument>,
    symbol_ref_class_value_bindings: Vec<SymbolRefClassValueBinding>,
    module_specifiers: Vec<SourceModuleSpecifierFactV0>,
    class_value_universes: Vec<SourceClassValueUniverseEntryV0>,
    domain_class_references: Vec<SourceDomainClassReferenceFactV0>,
}

fn collect_source_syntax_ast_facts(
    source: &str,
    source_type: SourceType,
    property_access_targets: &[SourceStyleBindingTarget],
    style_targets: &[SourceStyleBindingTarget],
    classnames_bind_imports: &[String],
) -> SourceSyntaxAstFacts {
    let allocator = Allocator::default();
    let ParserReturn {
        program, panicked, ..
    } = Parser::new(&allocator, source, source_type).parse();
    if panicked {
        return SourceSyntaxAstFacts {
            binding_scopes: Vec::new(),
            scope_parent_edges: Vec::new(),
            binding_decls: Vec::new(),
            scope_contains_decls: Vec::new(),
            class_string_literals: Vec::new(),
            style_property_accesses: Vec::new(),
            inline_style_declarations: Vec::new(),
            class_name_expression_spans: Vec::new(),
            classnames_bind_utility_bindings: Vec::new(),
            classnames_bind_call_arguments: Vec::new(),
            symbol_ref_class_value_bindings: Vec::new(),
            module_specifiers: Vec::new(),
            class_value_universes: Vec::new(),
            domain_class_references: Vec::new(),
        };
    }

    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.scoping();
    let property_access_targets =
        source_style_targets_with_symbols(property_access_targets, &program);
    let style_targets = source_style_targets_with_symbols(style_targets, &program);
    let classnames_bind_import_symbols =
        classnames_bind_import_symbol_ids(&program, classnames_bind_imports);
    let variant_recipe_bindings = collect_variant_recipe_bindings(source, &program, scoping);
    let mut collector = SourceSyntaxAstCollector {
        source,
        scoping,
        property_access_targets: property_access_targets.as_slice(),
        style_targets: style_targets.as_slice(),
        classnames_bind_import_symbols: &classnames_bind_import_symbols,
        variant_recipe_bindings: variant_recipe_bindings.as_slice(),
        binding_scopes: Vec::new(),
        scope_parent_edges: Vec::new(),
        binding_decls: Vec::new(),
        scope_contains_decls: Vec::new(),
        scope_stack: Vec::new(),
        class_string_literals: Vec::new(),
        style_property_accesses: Vec::new(),
        inline_style_declarations: Vec::new(),
        class_name_expression_spans: Vec::new(),
        classnames_bind_utility_bindings: Vec::new(),
        classnames_bind_call_arguments: Vec::new(),
        symbol_ref_class_value_bindings: Vec::new(),
        module_specifiers: Vec::new(),
        domain_class_references: Vec::new(),
    };
    collector.collect_program(&program);
    collector.canonicalize();
    SourceSyntaxAstFacts {
        binding_scopes: collector.binding_scopes,
        scope_parent_edges: collector.scope_parent_edges,
        binding_decls: collector.binding_decls,
        scope_contains_decls: collector.scope_contains_decls,
        class_string_literals: collector.class_string_literals,
        style_property_accesses: collector.style_property_accesses,
        inline_style_declarations: collector.inline_style_declarations,
        class_name_expression_spans: collector.class_name_expression_spans,
        classnames_bind_utility_bindings: collector.classnames_bind_utility_bindings,
        classnames_bind_call_arguments: collector.classnames_bind_call_arguments,
        symbol_ref_class_value_bindings: collector.symbol_ref_class_value_bindings,
        module_specifiers: collector.module_specifiers,
        class_value_universes: variant_recipe_bindings
            .iter()
            .map(VariantRecipeBindingV0::to_universe_entry)
            .collect(),
        domain_class_references: collector.domain_class_references,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariantRecipeCallShape {
    BaseThenConfig,
    ObjectConfig,
}

#[derive(Debug, Clone, Copy)]
struct VariantRecipeConfigV0 {
    plugin_id: &'static str,
    domain: &'static str,
    import_sources: &'static [&'static str],
    import_names: &'static [&'static str],
    call_shape: VariantRecipeCallShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VariantRecipeBindingV0 {
    plugin_id: &'static str,
    domain: &'static str,
    local_name: String,
    local_symbol_id: SymbolId,
    base_class_names: Vec<String>,
    variants: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    compound_class_names: Vec<String>,
    byte_span: ParserByteSpanV0,
}

impl VariantRecipeBindingV0 {
    fn to_universe_entry(&self) -> SourceClassValueUniverseEntryV0 {
        let mut class_names = self.base_class_names.clone();
        class_names.extend(
            self.variants
                .values()
                .flat_map(|options| options.values().flatten().cloned()),
        );
        class_names.extend(self.compound_class_names.iter().cloned());
        class_names.sort();
        class_names.dedup();
        SourceClassValueUniverseEntryV0 {
            plugin_id: self.plugin_id,
            domain: self.domain,
            owner_name: self.local_name.clone(),
            class_names,
            axes: self
                .variants
                .iter()
                .map(|(axis_name, options)| {
                    let mut values = options.keys().cloned().collect::<Vec<_>>();
                    values.sort();
                    SourceClassValueUniverseAxisV0 {
                        axis_name: axis_name.clone(),
                        values,
                    }
                })
                .collect(),
            byte_span: self.byte_span,
        }
    }
}

fn variant_recipe_configs() -> [VariantRecipeConfigV0; 2] {
    [
        VariantRecipeConfigV0 {
            plugin_id: "cva-recipe-domain",
            domain: "cva-recipe",
            import_sources: &["class-variance-authority", "cva"],
            import_names: &["cva"],
            call_shape: VariantRecipeCallShape::BaseThenConfig,
        },
        VariantRecipeConfigV0 {
            plugin_id: "vanilla-extract-recipe-domain",
            domain: "vanilla-extract-recipe",
            import_sources: &["@vanilla-extract/recipes"],
            import_names: &["recipe"],
            call_shape: VariantRecipeCallShape::ObjectConfig,
        },
    ]
}

fn collect_variant_recipe_bindings(
    source: &str,
    program: &Program<'_>,
    scoping: &Scoping,
) -> Vec<VariantRecipeBindingV0> {
    let mut bindings = Vec::new();
    for config in variant_recipe_configs() {
        let imported_symbols = collect_variant_recipe_import_symbol_ids(program, config);
        if imported_symbols.is_empty() {
            continue;
        }
        for statement in &program.body {
            collect_variant_recipe_bindings_from_statement(
                source,
                statement,
                config,
                scoping,
                &imported_symbols,
                &mut bindings,
            );
        }
    }
    bindings.sort_by_key(|binding| {
        (
            binding.plugin_id,
            binding.local_name.clone(),
            binding.byte_span.start,
            binding.byte_span.end,
        )
    });
    bindings.dedup_by(|left, right| {
        left.plugin_id == right.plugin_id
            && left.local_name == right.local_name
            && left.byte_span == right.byte_span
    });
    bindings
}

fn collect_variant_recipe_import_symbol_ids(
    program: &Program<'_>,
    config: VariantRecipeConfigV0,
) -> BTreeSet<SymbolId> {
    let mut symbols = BTreeSet::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind != ImportOrExportKind::Value
            || !config
                .import_sources
                .contains(&import.source.value.as_str())
        {
            continue;
        }
        let Some(specifiers) = import.specifiers.as_ref() else {
            continue;
        };
        for specifier in specifiers {
            if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier {
                let imported_name = specifier.imported.name().as_str();
                if config.import_names.contains(&imported_name)
                    && let Some(symbol_id) = binding_identifier_symbol_id(&specifier.local)
                {
                    symbols.insert(symbol_id);
                }
            }
        }
    }
    symbols
}

fn collect_variant_recipe_bindings_from_statement(
    source: &str,
    statement: &Statement<'_>,
    config: VariantRecipeConfigV0,
    scoping: &Scoping,
    imported_symbols: &BTreeSet<SymbolId>,
    out: &mut Vec<VariantRecipeBindingV0>,
) {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            collect_variant_recipe_bindings_from_variable_declaration(
                source,
                declaration,
                config,
                scoping,
                imported_symbols,
                out,
            )
        }
        Statement::ExportNamedDeclaration(declaration) => {
            if let Some(Declaration::VariableDeclaration(declaration)) = &declaration.declaration {
                collect_variant_recipe_bindings_from_variable_declaration(
                    source,
                    declaration,
                    config,
                    scoping,
                    imported_symbols,
                    out,
                );
            }
        }
        _ => {}
    }
}

fn collect_variant_recipe_bindings_from_variable_declaration(
    source: &str,
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
    config: VariantRecipeConfigV0,
    scoping: &Scoping,
    imported_symbols: &BTreeSet<SymbolId>,
    out: &mut Vec<VariantRecipeBindingV0>,
) {
    for declarator in &declaration.declarations {
        let Some(local_identifier) = binding_pattern_identifier(&declarator.id) else {
            continue;
        };
        let local_name = local_identifier.name.as_str();
        let Some(local_symbol_id) = binding_identifier_symbol_id(local_identifier) else {
            continue;
        };
        let Some(Expression::CallExpression(call)) = declarator
            .init
            .as_ref()
            .and_then(unwrap_transparent_expression)
        else {
            continue;
        };
        let Some(callee_identifier) = expression_identifier(&call.callee) else {
            continue;
        };
        let Some(callee_symbol_id) = reference_symbol_id(scoping, callee_identifier) else {
            continue;
        };
        if !imported_symbols.contains(&callee_symbol_id) {
            continue;
        }
        let Some(config_object) = variant_recipe_config_object(call, config.call_shape) else {
            continue;
        };
        let base_class_names = variant_recipe_base_class_names(
            source,
            local_name,
            call,
            config_object,
            config.call_shape,
        );
        let variants = variant_recipe_variants(source, local_name, config_object);
        let compound_class_names =
            variant_recipe_compound_class_names(source, local_name, config_object);
        if base_class_names.is_empty() && variants.is_empty() && compound_class_names.is_empty() {
            continue;
        }
        out.push(VariantRecipeBindingV0 {
            plugin_id: config.plugin_id,
            domain: config.domain,
            local_name: local_name.to_string(),
            local_symbol_id,
            base_class_names,
            variants,
            compound_class_names,
            byte_span: parser_byte_span(call.span()),
        });
    }
}

fn variant_recipe_config_object<'a>(
    call: &'a CallExpression<'a>,
    call_shape: VariantRecipeCallShape,
) -> Option<&'a ObjectExpression<'a>> {
    let argument = match call_shape {
        VariantRecipeCallShape::BaseThenConfig => call.arguments.get(1),
        VariantRecipeCallShape::ObjectConfig => call.arguments.first(),
    }?;
    let expression = argument_expression(argument).and_then(unwrap_transparent_expression)?;
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn variant_recipe_base_class_names(
    source: &str,
    local_name: &str,
    call: &CallExpression<'_>,
    config_object: &ObjectExpression<'_>,
    call_shape: VariantRecipeCallShape,
) -> Vec<String> {
    match call_shape {
        VariantRecipeCallShape::BaseThenConfig => call
            .arguments
            .first()
            .and_then(argument_expression)
            .map(|expression| class_names_from_expression(source, expression, Some(local_name)))
            .unwrap_or_else(|| vec![local_name.to_string()]),
        VariantRecipeCallShape::ObjectConfig => {
            object_property_expression(source, config_object, "base")
                .map(|expression| class_names_from_expression(source, expression, Some(local_name)))
                .unwrap_or_default()
        }
    }
}

fn variant_recipe_variants(
    source: &str,
    recipe_name: &str,
    config_object: &ObjectExpression<'_>,
) -> BTreeMap<String, BTreeMap<String, Vec<String>>> {
    let Some(Expression::ObjectExpression(variants_object)) =
        object_property_expression(source, config_object, "variants")
            .and_then(unwrap_transparent_expression)
    else {
        return BTreeMap::new();
    };
    let mut variants = BTreeMap::new();
    for property in &variants_object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(axis_name) = property_key_text(source, &property.key) else {
            continue;
        };
        let Some(Expression::ObjectExpression(options_object)) =
            unwrap_transparent_expression(&property.value)
        else {
            continue;
        };
        let mut options = BTreeMap::new();
        for option in &options_object.properties {
            let ObjectPropertyKind::ObjectProperty(option) = option else {
                continue;
            };
            if option.computed {
                continue;
            }
            let Some(option_name) = property_key_text(source, &option.key) else {
                continue;
            };
            let fallback = format!("{recipe_name}.{axis_name}.{option_name}");
            options.insert(
                option_name,
                class_names_from_expression(source, &option.value, Some(fallback.as_str())),
            );
        }
        if !options.is_empty() {
            variants.insert(axis_name, options);
        }
    }
    variants
}

fn variant_recipe_compound_class_names(
    source: &str,
    recipe_name: &str,
    config_object: &ObjectExpression<'_>,
) -> Vec<String> {
    let Some(Expression::ArrayExpression(compounds)) =
        object_property_expression(source, config_object, "compoundVariants")
            .and_then(unwrap_transparent_expression)
    else {
        return Vec::new();
    };
    let mut class_names = Vec::new();
    for element in &compounds.elements {
        let Some(Expression::ObjectExpression(compound)) =
            array_expression_element_expression(element).and_then(unwrap_transparent_expression)
        else {
            continue;
        };
        let fallback = format!("{recipe_name}.compound");
        for property_name in ["class", "className", "style"] {
            if let Some(expression) = object_property_expression(source, compound, property_name) {
                class_names.extend(class_names_from_expression(
                    source,
                    expression,
                    Some(fallback.as_str()),
                ));
            }
        }
    }
    class_names.sort();
    class_names.dedup();
    class_names
}

fn collect_variant_recipe_call_references(
    source: &str,
    expression: &Expression<'_>,
    recipe: &VariantRecipeBindingV0,
    out: &mut Vec<SourceDomainClassReferenceFactV0>,
) {
    let Some(Expression::ObjectExpression(object)) = unwrap_transparent_expression(expression)
    else {
        return;
    };
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(axis_name) = property_key_text(source, &property.key) else {
            continue;
        };
        let Some(options) = recipe.variants.get(axis_name.as_str()) else {
            continue;
        };
        collect_variant_recipe_value_reference(
            source,
            recipe,
            axis_name.as_str(),
            options,
            &property.value,
            out,
        );
    }
}

fn collect_variant_recipe_value_reference(
    source: &str,
    recipe: &VariantRecipeBindingV0,
    axis_name: &str,
    options: &BTreeMap<String, Vec<String>>,
    expression: &Expression<'_>,
    out: &mut Vec<SourceDomainClassReferenceFactV0>,
) {
    let Some(value) = unwrap_transparent_expression(expression) else {
        return;
    };
    if let Some((option_name, byte_span)) = string_expression_value_and_span(source, value) {
        out.push(SourceDomainClassReferenceFactV0 {
            byte_span,
            plugin_id: recipe.plugin_id,
            domain: recipe.domain,
            owner_name: recipe.local_name.clone(),
            axis_name: axis_name.to_string(),
            option_name: Some(option_name),
            prefix: None,
        });
        return;
    }
    if let Expression::ConditionalExpression(conditional) = value {
        collect_variant_recipe_value_reference(
            source,
            recipe,
            axis_name,
            options,
            &conditional.consequent,
            out,
        );
        collect_variant_recipe_value_reference(
            source,
            recipe,
            axis_name,
            options,
            &conditional.alternate,
            out,
        );
        return;
    }
    if let Expression::TemplateLiteral(template) = value
        && let Some(prefix) = source
            .get(template.span.start as usize + 1..template.span.end as usize)
            .and_then(|text| text.split("${").next())
            .filter(|prefix| !prefix.is_empty())
            .map(str::to_string)
        && options
            .keys()
            .any(|option| option.starts_with(prefix.as_str()))
    {
        out.push(SourceDomainClassReferenceFactV0 {
            byte_span: parser_byte_span(template.span),
            plugin_id: recipe.plugin_id,
            domain: recipe.domain,
            owner_name: recipe.local_name.clone(),
            axis_name: axis_name.to_string(),
            option_name: None,
            prefix: Some(prefix),
        });
    }
}

fn collect_vue_use_css_module_import_names(program: &Program<'_>) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind != ImportOrExportKind::Value || import.source.value.as_str() != "vue"
        {
            continue;
        }
        let Some(specifiers) = import.specifiers.as_ref() else {
            continue;
        };
        for specifier in specifiers {
            if let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier {
                let imported_name = specifier.imported.name().as_str();
                if imported_name == "useCssModule" {
                    names.insert(specifier.local.name.as_str().to_string());
                }
            }
        }
    }
    names
}

fn collect_vue_use_css_module_bindings(program: &Program<'_>) -> Vec<String> {
    let use_css_module_names = collect_vue_use_css_module_import_names(program);
    if use_css_module_names.is_empty() {
        return Vec::new();
    }
    let mut bindings = BTreeSet::new();
    for statement in &program.body {
        collect_vue_use_css_module_bindings_from_statement(
            statement,
            &use_css_module_names,
            &mut bindings,
        );
    }
    bindings.into_iter().collect()
}

fn style_import_local_names_by_uri(
    bindings: &[SourceBindingStyleImportFactV0],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut local_names_by_uri = BTreeMap::new();
    for binding in bindings {
        local_names_by_uri
            .entry(binding.style_uri.clone())
            .or_insert_with(BTreeSet::new)
            .insert(binding.local_name.clone());
    }
    local_names_by_uri
}

fn single_btree_set_item(values: &BTreeSet<String>) -> Option<String> {
    if values.len() == 1 {
        values.iter().next().cloned()
    } else {
        None
    }
}

fn collect_vue_use_css_module_bindings_from_statement(
    statement: &Statement<'_>,
    use_css_module_names: &BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
) {
    match statement {
        Statement::VariableDeclaration(declaration) => {
            collect_vue_use_css_module_bindings_from_variable_declaration(
                declaration,
                use_css_module_names,
                bindings,
            );
        }
        Statement::ExportNamedDeclaration(declaration) => {
            if let Some(Declaration::VariableDeclaration(declaration)) = &declaration.declaration {
                collect_vue_use_css_module_bindings_from_variable_declaration(
                    declaration,
                    use_css_module_names,
                    bindings,
                );
            }
        }
        _ => {}
    }
}

fn collect_vue_use_css_module_bindings_from_variable_declaration(
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
    use_css_module_names: &BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
) {
    for declarator in &declaration.declarations {
        let Some(binding) = binding_pattern_identifier_name(&declarator.id) else {
            continue;
        };
        let Some(Expression::CallExpression(call)) = &declarator.init else {
            continue;
        };
        let Some(callee) = expression_identifier_name(&call.callee) else {
            continue;
        };
        if use_css_module_names.contains(callee) {
            bindings.insert(binding.to_string());
        }
    }
}

struct SourceSyntaxAstCollector<'a, 'b, 's> {
    source: &'a str,
    scoping: &'s Scoping,
    property_access_targets: &'a [SourceStyleBindingTarget],
    style_targets: &'a [SourceStyleBindingTarget],
    classnames_bind_import_symbols: &'a BTreeSet<SymbolId>,
    variant_recipe_bindings: &'b [VariantRecipeBindingV0],
    binding_scopes: Vec<SourceBindingScopeFactV0>,
    scope_parent_edges: Vec<SourceScopeParentFactV0>,
    binding_decls: Vec<SourceBindingDeclFactV0>,
    scope_contains_decls: Vec<SourceScopeContainsDeclFactV0>,
    scope_stack: Vec<SourceBindingScopeFactV0>,
    class_string_literals: Vec<ParserByteSpanV0>,
    style_property_accesses: Vec<SourceStylePropertyAccessFactV0>,
    inline_style_declarations: Vec<SourceInlineStyleDeclarationFactV0>,
    class_name_expression_spans: Vec<ParserByteSpanV0>,
    classnames_bind_utility_bindings: Vec<ClassnamesBindUtilityBinding>,
    classnames_bind_call_arguments: Vec<ClassnamesBindCallArgument>,
    symbol_ref_class_value_bindings: Vec<SymbolRefClassValueBinding>,
    module_specifiers: Vec<SourceModuleSpecifierFactV0>,
    domain_class_references: Vec<SourceDomainClassReferenceFactV0>,
}

impl<'a, 'b, 's> SourceSyntaxAstCollector<'a, 'b, 's> {
    fn collect_program(&mut self, program: &Program<'a>) {
        self.with_binding_scope("sourceFile", parser_byte_span(program.span), |collector| {
            for statement in &program.body {
                collector.collect_statement(statement);
            }
        });
    }

    fn collect_statement(&mut self, statement: &Statement<'a>) {
        match statement {
            Statement::BlockStatement(statement) => {
                self.collect_block_statement(statement);
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
                self.collect_function(function, true);
            }
            Statement::ClassDeclaration(class) => {
                self.collect_class(class);
            }
            Statement::ImportDeclaration(import) => {
                self.collect_import_declaration(import);
            }
            Statement::ExportNamedDeclaration(export) => {
                self.collect_export_named_module_specifier(export);
                if let Some(declaration) = &export.declaration {
                    self.collect_export_named_declaration(declaration, export.span);
                }
            }
            Statement::ExportAllDeclaration(export) => {
                self.collect_export_all_module_specifier(export);
            }
            Statement::TSImportEqualsDeclaration(declaration) => {
                self.collect_ts_import_equals_declaration(declaration);
            }
            Statement::ExportDefaultDeclaration(declaration) => {
                self.collect_export_default_declaration(&declaration.declaration, declaration.span);
            }
            Statement::TSExportAssignment(declaration) => {
                self.collect_expression(&declaration.expression);
            }
            _ => {}
        }
    }

    fn collect_export_named_declaration(
        &mut self,
        declaration: &Declaration<'a>,
        export_span: Span,
    ) {
        match declaration {
            Declaration::VariableDeclaration(declaration) => {
                self.collect_variable_declaration(declaration);
            }
            Declaration::FunctionDeclaration(function) => {
                self.collect_function_with_scope_span(function, true, export_span);
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
        export_span: Span,
    ) {
        match declaration {
            oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                self.collect_function_with_scope_span(function, true, export_span);
            }
            oxc_ast::ast::ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                self.collect_class(class);
            }
            // Every expression-kind default export (`export default <expr>`) delegates to
            // `collect_expression`, which already descends arrow/function/parenthesized bodies and
            // JSX. Previously only member/call kinds were matched and the catch-all silently
            // dropped `export default () => <JSX/>` (and parenthesized/JSX/conditional forms),
            // so their className usages were never collected -> unusedSelector false positives.
            // Non-expression kinds (`TSInterfaceDeclaration`) yield `None` and are correctly ignored.
            _ => {
                if let Some(expression) = declaration.as_expression() {
                    self.collect_expression(expression);
                }
            }
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
            self.collect_binding_pattern_decl_facts(&declarator.id, "localVar", None);
            if let Some(binding) = self.classnames_bind_utility_binding_from_declarator(declarator)
            {
                self.classnames_bind_utility_bindings.push(binding);
            }
            if let Some(init) = &declarator.init {
                self.collect_expression(init);
            }
        }
    }

    fn collect_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        if import.import_kind != ImportOrExportKind::Value {
            return;
        }
        self.module_specifiers.push(SourceModuleSpecifierFactV0 {
            kind: "import",
            specifier: import.source.value.as_str().to_string(),
            byte_span: parser_byte_span(import.source.span),
        });
        let Some(specifiers) = import.specifiers.as_ref() else {
            return;
        };
        for specifier in specifiers {
            let local = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => &specifier.local,
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => &specifier.local,
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => &specifier.local,
            };
            self.push_binding_identifier_decl_fact(
                local,
                "import",
                Some(import.source.value.as_str()),
            );
        }
    }

    fn collect_export_named_module_specifier(
        &mut self,
        export: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        if export.export_kind != ImportOrExportKind::Value {
            return;
        }
        if let Some(source) = &export.source {
            self.module_specifiers.push(SourceModuleSpecifierFactV0 {
                kind: "export",
                specifier: source.value.as_str().to_string(),
                byte_span: parser_byte_span(source.span),
            });
        }
    }

    fn collect_export_all_module_specifier(
        &mut self,
        export: &oxc_ast::ast::ExportAllDeclaration<'a>,
    ) {
        if export.export_kind != ImportOrExportKind::Value {
            return;
        }
        self.module_specifiers.push(SourceModuleSpecifierFactV0 {
            kind: "export",
            specifier: export.source.value.as_str().to_string(),
            byte_span: parser_byte_span(export.source.span),
        });
    }

    fn collect_ts_import_equals_declaration(
        &mut self,
        declaration: &oxc_ast::ast::TSImportEqualsDeclaration<'a>,
    ) {
        if declaration.import_kind != ImportOrExportKind::Value {
            return;
        }
        if let TSModuleReference::ExternalModuleReference(reference) = &declaration.module_reference
        {
            self.module_specifiers.push(SourceModuleSpecifierFactV0 {
                kind: "importEquals",
                specifier: reference.expression.value.as_str().to_string(),
                byte_span: parser_byte_span(reference.expression.span),
            });
        }
    }

    fn collect_function(&mut self, function: &oxc_ast::ast::Function<'a>, include_name: bool) {
        self.collect_function_with_scope_span(function, include_name, function.span);
    }

    fn collect_function_with_scope_span(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        include_name: bool,
        scope_span: Span,
    ) {
        if include_name && let Some(identifier) = &function.id {
            self.push_binding_identifier_decl_fact(identifier, "localVar", None);
        }
        self.with_binding_scope("function", parser_byte_span(scope_span), |collector| {
            collector.collect_function_parameters(&function.params);
            collector.collect_function_body(function.body.as_deref());
        });
    }

    fn collect_arrow_function(&mut self, function: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
        self.with_binding_scope("function", parser_byte_span(function.span), |collector| {
            collector.collect_function_parameters(&function.params);
            collector.collect_function_body(Some(&function.body));
        });
    }

    fn collect_function_parameters(&mut self, params: &oxc_ast::ast::FormalParameters<'a>) {
        for parameter in &params.items {
            self.collect_binding_pattern_decl_facts(&parameter.pattern, "parameter", None);
        }
        if let Some(rest) = &params.rest {
            self.collect_binding_pattern_decl_facts(&rest.rest.argument, "parameter", None);
        }
    }

    fn with_binding_scope(
        &mut self,
        kind: &'static str,
        byte_span: ParserByteSpanV0,
        collect: impl FnOnce(&mut Self),
    ) {
        let scope = SourceBindingScopeFactV0 { kind, byte_span };
        if let Some(parent) = self.scope_stack.last() {
            self.scope_parent_edges.push(SourceScopeParentFactV0 {
                child_kind: scope.kind,
                child_byte_span: scope.byte_span,
                parent_kind: parent.kind,
                parent_byte_span: parent.byte_span,
            });
        }
        self.binding_scopes.push(scope.clone());
        self.scope_stack.push(scope);
        collect(self);
        self.scope_stack.pop();
    }

    fn collect_binding_pattern_decl_facts(
        &mut self,
        pattern: &BindingPattern<'_>,
        kind: &'static str,
        import_path: Option<&str>,
    ) {
        let mut facts = Vec::new();
        collect_binding_pattern_decl_facts(pattern, kind, import_path, &mut facts);
        for fact in facts {
            self.push_binding_decl_fact(fact);
        }
    }

    fn push_binding_identifier_decl_fact(
        &mut self,
        identifier: &BindingIdentifier<'_>,
        kind: &'static str,
        import_path: Option<&str>,
    ) {
        let mut facts = Vec::new();
        push_binding_identifier_decl_fact(identifier, kind, import_path, &mut facts);
        for fact in facts {
            self.push_binding_decl_fact(fact);
        }
    }

    fn push_binding_decl_fact(&mut self, fact: SourceBindingDeclFactV0) {
        if let Some(scope) = self.scope_stack.last() {
            self.scope_contains_decls
                .push(SourceScopeContainsDeclFactV0 {
                    scope_kind: scope.kind,
                    scope_byte_span: scope.byte_span,
                    decl_kind: fact.kind,
                    decl_name: fact.name.clone(),
                    decl_byte_span: fact.byte_span,
                    import_path: fact.import_path.clone(),
                });
        }
        self.binding_decls.push(fact);
    }

    fn classnames_bind_utility_binding_from_declarator(
        &self,
        declarator: &VariableDeclarator<'a>,
    ) -> Option<ClassnamesBindUtilityBinding> {
        if self.style_targets.is_empty() || self.classnames_bind_import_symbols.is_empty() {
            return None;
        }
        let binding = binding_pattern_identifier(&declarator.id)?;
        let binding_symbol_id = binding_identifier_symbol_id(binding)?;
        let init = declarator.init.as_ref()?;
        let Expression::CallExpression(call) = init else {
            return None;
        };
        let Expression::StaticMemberExpression(callee) = &call.callee else {
            return None;
        };
        if callee.property.name.as_str() != "bind" {
            return None;
        }
        let callee_identifier = expression_identifier(&callee.object)?;
        let callee_symbol_id = reference_symbol_id(self.scoping, callee_identifier)?;
        if !self
            .classnames_bind_import_symbols
            .contains(&callee_symbol_id)
        {
            return None;
        }
        let style_identifier = call.arguments.first().and_then(argument_identifier)?;
        let style_symbol_id = reference_symbol_id(self.scoping, style_identifier)?;
        let style_uri = self
            .style_targets
            .iter()
            .find(|target| target.binding_symbol_id == Some(style_symbol_id))?
            .target_style_uri
            .clone()?;

        Some(ClassnamesBindUtilityBinding {
            binding: binding.name.as_str().to_string(),
            binding_symbol_id,
            styles_binding: style_identifier.name.as_str().to_string(),
            style_uri,
            classnames_import_binding: callee_identifier.name.as_str().to_string(),
        })
    }

    fn collect_function_body(&mut self, body: Option<&oxc_ast::ast::FunctionBody<'a>>) {
        let Some(body) = body else {
            return;
        };
        self.with_binding_scope("block", parser_byte_span(body.span), |collector| {
            for statement in &body.statements {
                collector.collect_statement(statement);
            }
        });
    }

    fn collect_block_statement(&mut self, block: &oxc_ast::ast::BlockStatement<'a>) {
        self.with_binding_scope("block", parser_byte_span(block.span), |collector| {
            for statement in &block.body {
                collector.collect_statement(statement);
            }
        });
    }

    fn collect_class(&mut self, class: &Class<'a>) {
        if let Some(super_class) = &class.super_class {
            self.collect_expression(super_class);
        }
        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) => {
                    self.collect_function(&method.value, false);
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
                self.collect_arrow_function(expression);
            }
            Expression::FunctionExpression(expression) => {
                self.collect_function(expression, false);
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
            ArrayExpressionElement::Elision(_) => {}
            _ => {
                if let Some(expression) = element.as_expression() {
                    self.collect_expression(expression);
                }
            }
        }
    }

    fn collect_argument(&mut self, argument: &Argument<'a>) {
        match argument {
            Argument::SpreadElement(spread) => {
                self.collect_expression(&spread.argument);
            }
            _ => {
                if let Some(expression) = argument.as_expression() {
                    self.collect_expression(expression);
                }
            }
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
            oxc_ast::ast::PropertyKey::StaticIdentifier(_)
            | oxc_ast::ast::PropertyKey::PrivateIdentifier(_) => {}
            _ => {
                if let Some(expression) = key.as_expression() {
                    self.collect_expression(expression);
                }
            }
        }
    }

    fn collect_jsx_element(&mut self, element: &oxc_ast::ast::JSXElement<'a>) {
        for attribute in &element.opening_element.attributes {
            match attribute {
                oxc_ast::ast::JSXAttributeItem::Attribute(attribute) => {
                    if is_jsx_class_name_attribute(&attribute.name)
                        && let Some(value) = &attribute.value
                    {
                        self.collect_class_name_string_literal_attribute(value);
                        self.collect_class_name_expression_attribute(value);
                    }
                    if is_jsx_style_attribute(&attribute.name)
                        && let Some(value) = &attribute.value
                    {
                        self.collect_inline_style_attribute(value);
                    }
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

    fn collect_class_name_string_literal_attribute(&mut self, value: &JSXAttributeValue<'a>) {
        let JSXAttributeValue::StringLiteral(literal) = value else {
            return;
        };
        if let Some(span) = self.string_literal_content_span(literal.span) {
            self.class_string_literals.push(span);
        }
    }

    fn collect_class_name_expression_attribute(&mut self, value: &JSXAttributeValue<'a>) {
        let JSXAttributeValue::ExpressionContainer(container) = value else {
            return;
        };
        if let Some(span) = jsx_expression_span(&container.expression) {
            self.class_name_expression_spans.push(span);
        }
    }

    fn collect_inline_style_attribute(&mut self, value: &JSXAttributeValue<'a>) {
        let JSXAttributeValue::ExpressionContainer(container) = value else {
            return;
        };
        let JSXExpression::ObjectExpression(object) = &container.expression else {
            return;
        };
        let target_style_uri = self.single_imported_style_target_uri();
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property.computed {
                continue;
            }
            let Some((property_name, byte_span)) = self.inline_style_property_name(&property.key)
            else {
                continue;
            };
            let value_byte_span = Some(parser_byte_span(property.value.span()));
            self.inline_style_declarations
                .push(SourceInlineStyleDeclarationFactV0 {
                    byte_span,
                    value_byte_span,
                    property_name,
                    value: self.inline_style_static_value(&property.value),
                    target_style_uri: target_style_uri.clone(),
                    cascade_tier: "authorInlineStyle",
                    static_value: self.inline_style_value_is_static(&property.value),
                });
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

    fn single_imported_style_target_uri(&self) -> Option<String> {
        let targets = self
            .style_targets
            .iter()
            .filter_map(|target| target.target_style_uri.as_deref())
            .collect::<BTreeSet<_>>();
        if targets.len() == 1 {
            targets.into_iter().next().map(str::to_string)
        } else {
            None
        }
    }

    fn inline_style_property_name(
        &self,
        key: &oxc_ast::ast::PropertyKey<'a>,
    ) -> Option<(String, ParserByteSpanV0)> {
        let byte_span = parser_byte_span(key.span());
        let raw = self.source.get(byte_span.start..byte_span.end)?.trim();
        let unquoted = raw
            .strip_prefix(['"', '\''])
            .and_then(|value| value.strip_suffix(['"', '\'']))
            .unwrap_or(raw);
        if unquoted.is_empty() || unquoted.contains(char::is_whitespace) {
            return None;
        }
        Some((normalize_inline_style_property_name(unquoted), byte_span))
    }

    fn inline_style_static_value(&self, expression: &Expression<'a>) -> Option<String> {
        if !self.inline_style_value_is_static(expression) {
            return None;
        }
        let span = parser_byte_span(expression.span());
        Some(self.source.get(span.start..span.end)?.trim().to_string())
    }

    fn inline_style_value_is_static(&self, expression: &Expression<'a>) -> bool {
        match expression {
            Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_) => true,
            Expression::TemplateLiteral(literal) => literal.expressions.is_empty(),
            Expression::ParenthesizedExpression(expression) => {
                self.inline_style_value_is_static(&expression.expression)
            }
            Expression::TSAsExpression(expression) => {
                self.inline_style_value_is_static(&expression.expression)
            }
            Expression::TSSatisfiesExpression(expression) => {
                self.inline_style_value_is_static(&expression.expression)
            }
            Expression::TSNonNullExpression(expression) => {
                self.inline_style_value_is_static(&expression.expression)
            }
            Expression::TSTypeAssertion(expression) => {
                self.inline_style_value_is_static(&expression.expression)
            }
            _ => false,
        }
    }

    fn collect_call_expression(&mut self, expression: &CallExpression<'a>) {
        if let Some(callee_identifier) = expression_identifier(&expression.callee) {
            let callee_symbol_id = reference_symbol_id(self.scoping, callee_identifier);
            if let Some(recipe) = self
                .variant_recipe_bindings
                .iter()
                .find(|recipe| callee_symbol_id == Some(recipe.local_symbol_id))
                && let Some(argument) = expression.arguments.first().and_then(argument_expression)
            {
                collect_variant_recipe_call_references(
                    self.source,
                    argument,
                    recipe,
                    &mut self.domain_class_references,
                );
            }
            for argument in &expression.arguments {
                if let Some(binding_symbol_id) = callee_symbol_id
                    && let Some(byte_span) = argument_expression_span(argument)
                {
                    self.classnames_bind_call_arguments
                        .push(ClassnamesBindCallArgument {
                            binding: callee_identifier.name.as_str().to_string(),
                            binding_symbol_id,
                            byte_span,
                        });
                    if let Some(expression) = argument_expression(argument) {
                        self.collect_class_value_symbol_refs_from_expression(
                            expression,
                            binding_symbol_id,
                        );
                    }
                }
            }
        }
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

    fn collect_class_value_symbol_refs_from_expression<'expr>(
        &mut self,
        expression: &'expr Expression<'a>,
        classnames_binding_symbol_id: SymbolId,
    ) {
        let Some(expression) = unwrap_transparent_expression(expression) else {
            return;
        };
        if let Some(binding) = self.symbol_ref_class_value_binding_from_expression(
            expression,
            classnames_binding_symbol_id,
        ) {
            self.symbol_ref_class_value_bindings.push(binding);
            return;
        }
        match expression {
            Expression::ArrayExpression(expression) => {
                for element in &expression.elements {
                    if let Some(expression) = array_expression_element_expression(element) {
                        self.collect_class_value_symbol_refs_from_expression(
                            expression,
                            classnames_binding_symbol_id,
                        );
                    }
                }
            }
            Expression::ObjectExpression(expression) => {
                for property in &expression.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        continue;
                    };
                    if property.computed
                        && let Some(expression) = property.key.as_expression()
                    {
                        self.collect_class_value_symbol_refs_from_expression(
                            expression,
                            classnames_binding_symbol_id,
                        );
                    }
                }
            }
            Expression::LogicalExpression(expression) if expression.operator.is_and() => {
                self.collect_class_value_symbol_refs_from_expression(
                    &expression.right,
                    classnames_binding_symbol_id,
                );
            }
            Expression::ConditionalExpression(expression) => {
                self.collect_class_value_symbol_refs_from_expression(
                    &expression.consequent,
                    classnames_binding_symbol_id,
                );
                self.collect_class_value_symbol_refs_from_expression(
                    &expression.alternate,
                    classnames_binding_symbol_id,
                );
            }
            _ => {}
        }
    }

    fn symbol_ref_class_value_binding_from_expression<'expr>(
        &self,
        expression: &'expr Expression<'a>,
        classnames_binding_symbol_id: SymbolId,
    ) -> Option<SymbolRefClassValueBinding> {
        let root = root_identifier_for_symbol_ref_expression(expression)?;
        let root_symbol_id = reference_symbol_id(self.scoping, root)?;
        if self
            .property_access_targets
            .iter()
            .any(|target| target.binding_symbol_id == Some(root_symbol_id))
        {
            return None;
        }
        let byte_span = parser_byte_span(expression.span());
        let raw_reference = self.source.get(byte_span.start..byte_span.end)?.trim();
        Some(SymbolRefClassValueBinding {
            classnames_binding_symbol_id,
            byte_span,
            raw_reference: raw_reference.to_string(),
            root_name: root.name.as_str().to_string(),
            decl_name: self.scoping.symbol_name(root_symbol_id).to_string(),
        })
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
            self.style_property_accesses
                .push(SourceStylePropertyAccessFactV0 {
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
            self.style_property_accesses
                .push(SourceStylePropertyAccessFactV0 {
                    byte_span,
                    target_style_uri: target.target_style_uri.clone(),
                });
        }
        self.collect_expression(&member.object);
        self.collect_expression(&member.expression);
    }

    fn target_for_object(&self, expression: &Expression<'a>) -> Option<&SourceStyleBindingTarget> {
        match expression {
            Expression::Identifier(identifier) => {
                let reference_symbol_id = reference_symbol_id(self.scoping, identifier)?;
                self.property_access_targets
                    .iter()
                    .find(|target| target.binding_symbol_id == Some(reference_symbol_id))
            }
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

    fn string_literal_content_span(&self, span: Span) -> Option<ParserByteSpanV0> {
        let span = parser_byte_span(span);
        if span.end <= span.start + 1 {
            return None;
        }
        let content = ParserByteSpanV0 {
            start: span.start + 1,
            end: span.end - 1,
        };
        self.source.get(content.start..content.end)?;
        Some(content)
    }

    fn canonicalize(&mut self) {
        self.binding_scopes.sort();
        self.binding_scopes.dedup();
        self.scope_parent_edges.sort();
        self.scope_parent_edges.dedup();
        self.binding_decls.sort();
        self.binding_decls.dedup();
        self.scope_contains_decls.sort();
        self.scope_contains_decls.dedup();
        self.class_string_literals.sort_by(|left, right| {
            left.start
                .cmp(&right.start)
                .then_with(|| left.end.cmp(&right.end))
        });
        self.class_string_literals.dedup();
        self.style_property_accesses.sort_by(|left, right| {
            left.byte_span
                .start
                .cmp(&right.byte_span.start)
                .then_with(|| left.byte_span.end.cmp(&right.byte_span.end))
                .then_with(|| left.target_style_uri.cmp(&right.target_style_uri))
        });
        self.style_property_accesses.dedup();
        self.inline_style_declarations.sort_by(|left, right| {
            left.byte_span
                .start
                .cmp(&right.byte_span.start)
                .then_with(|| left.byte_span.end.cmp(&right.byte_span.end))
                .then_with(|| left.property_name.cmp(&right.property_name))
                .then_with(|| left.target_style_uri.cmp(&right.target_style_uri))
        });
        self.inline_style_declarations.dedup();
        self.classnames_bind_utility_bindings
            .sort_by(|left, right| {
                left.binding
                    .cmp(&right.binding)
                    .then_with(|| left.styles_binding.cmp(&right.styles_binding))
                    .then_with(|| left.style_uri.cmp(&right.style_uri))
                    .then_with(|| {
                        left.classnames_import_binding
                            .cmp(&right.classnames_import_binding)
                    })
            });
        self.classnames_bind_utility_bindings
            .dedup_by(|left, right| {
                left.binding == right.binding
                    && left.styles_binding == right.styles_binding
                    && left.style_uri == right.style_uri
                    && left.classnames_import_binding == right.classnames_import_binding
            });
        self.classnames_bind_call_arguments.sort_by(|left, right| {
            left.binding
                .cmp(&right.binding)
                .then_with(|| left.byte_span.start.cmp(&right.byte_span.start))
                .then_with(|| left.byte_span.end.cmp(&right.byte_span.end))
        });
        self.classnames_bind_call_arguments.dedup_by(|left, right| {
            left.binding == right.binding && left.byte_span == right.byte_span
        });
        self.symbol_ref_class_value_bindings.sort_by(|left, right| {
            left.classnames_binding_symbol_id
                .cmp(&right.classnames_binding_symbol_id)
                .then_with(|| left.byte_span.start.cmp(&right.byte_span.start))
                .then_with(|| left.byte_span.end.cmp(&right.byte_span.end))
                .then_with(|| left.raw_reference.cmp(&right.raw_reference))
                .then_with(|| left.decl_name.cmp(&right.decl_name))
        });
        self.symbol_ref_class_value_bindings
            .dedup_by(|left, right| {
                left.classnames_binding_symbol_id == right.classnames_binding_symbol_id
                    && left.byte_span == right.byte_span
                    && left.raw_reference == right.raw_reference
                    && left.decl_name == right.decl_name
            });
    }
}

fn parser_byte_span(span: Span) -> ParserByteSpanV0 {
    ParserByteSpanV0 {
        start: span.start as usize,
        end: span.end as usize,
    }
}

fn is_jsx_class_name_attribute(name: &JSXAttributeName<'_>) -> bool {
    matches!(name, JSXAttributeName::Identifier(identifier) if identifier.name.as_str() == "className")
}

fn is_jsx_style_attribute(name: &JSXAttributeName<'_>) -> bool {
    matches!(name, JSXAttributeName::Identifier(identifier) if identifier.name.as_str() == "style")
}

fn normalize_inline_style_property_name(name: &str) -> String {
    if name.starts_with("--") {
        return name.to_string();
    }
    let mut normalized = String::new();
    for character in name.chars() {
        if character.is_ascii_uppercase() {
            if !normalized.is_empty() {
                normalized.push('-');
            }
            normalized.push(character.to_ascii_lowercase());
        } else {
            normalized.push(character);
        }
    }
    normalized
}

fn jsx_expression_span(expression: &JSXExpression<'_>) -> Option<ParserByteSpanV0> {
    match expression {
        JSXExpression::EmptyExpression(_) => None,
        _ => Some(parser_byte_span(expression.span())),
    }
}

fn argument_expression_span(argument: &Argument<'_>) -> Option<ParserByteSpanV0> {
    match argument {
        Argument::SpreadElement(spread) => Some(parser_byte_span(spread.argument.span())),
        _ => Some(parser_byte_span(argument.span())),
    }
}

fn argument_expression<'arg, 'ast>(
    argument: &'arg Argument<'ast>,
) -> Option<&'arg Expression<'ast>> {
    match argument {
        Argument::SpreadElement(spread) => Some(&spread.argument),
        _ => argument.as_expression(),
    }
}

fn array_expression_element_expression<'element, 'ast>(
    element: &'element ArrayExpressionElement<'ast>,
) -> Option<&'element Expression<'ast>> {
    match element {
        ArrayExpressionElement::SpreadElement(spread) => Some(&spread.argument),
        ArrayExpressionElement::Elision(_) => None,
        _ => element.as_expression(),
    }
}

fn unwrap_transparent_expression<'expr, 'ast>(
    expression: &'expr Expression<'ast>,
) -> Option<&'expr Expression<'ast>> {
    match expression {
        Expression::ParenthesizedExpression(expression) => {
            unwrap_transparent_expression(&expression.expression)
        }
        Expression::TSAsExpression(expression) => {
            unwrap_transparent_expression(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            unwrap_transparent_expression(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => {
            unwrap_transparent_expression(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            unwrap_transparent_expression(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            unwrap_transparent_expression(&expression.expression)
        }
        _ => Some(expression),
    }
}

fn root_identifier_for_symbol_ref_expression<'expr, 'ast>(
    expression: &'expr Expression<'ast>,
) -> Option<&'expr IdentifierReference<'ast>> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier),
        Expression::StaticMemberExpression(member) => {
            root_identifier_for_symbol_ref_expression(&member.object)
        }
        Expression::ParenthesizedExpression(expression) => {
            root_identifier_for_symbol_ref_expression(&expression.expression)
        }
        Expression::TSAsExpression(expression) => {
            root_identifier_for_symbol_ref_expression(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            root_identifier_for_symbol_ref_expression(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => {
            root_identifier_for_symbol_ref_expression(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            root_identifier_for_symbol_ref_expression(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            root_identifier_for_symbol_ref_expression(&expression.expression)
        }
        _ => None,
    }
}

fn object_property_expression<'a>(
    source: &str,
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_text(source, &property.key).as_deref() != Some(name) {
            return None;
        }
        Some(&property.value)
    })
}

fn property_key_text(source: &str, key: &oxc_ast::ast::PropertyKey<'_>) -> Option<String> {
    let span = parser_byte_span(key.span());
    if source.is_empty() {
        match key {
            oxc_ast::ast::PropertyKey::StaticIdentifier(identifier) => {
                return Some(identifier.name.as_str().to_string());
            }
            oxc_ast::ast::PropertyKey::PrivateIdentifier(identifier) => {
                return Some(identifier.name.as_str().to_string());
            }
            _ => return None,
        }
    }
    object_property_name(source, span.start, span.end)
}

fn class_names_from_expression(
    source: &str,
    expression: &Expression<'_>,
    fallback: Option<&str>,
) -> Vec<String> {
    let Some(value) = unwrap_transparent_expression(expression) else {
        return fallback.into_iter().map(str::to_string).collect();
    };
    if let Some((text, _)) = string_expression_value_and_span(source, value) {
        let class_names = split_class_names(text.as_str());
        return if class_names.is_empty() {
            fallback.into_iter().map(str::to_string).collect()
        } else {
            class_names
        };
    }
    if let Expression::ArrayExpression(array) = value {
        let mut values = array
            .elements
            .iter()
            .filter_map(array_expression_element_expression)
            .flat_map(|element| class_names_from_expression(source, element, None))
            .collect::<Vec<_>>();
        values.sort();
        values.dedup();
        return if values.is_empty() {
            fallback.into_iter().map(str::to_string).collect()
        } else {
            values
        };
    }
    fallback.into_iter().map(str::to_string).collect()
}

fn string_expression_value_and_span(
    source: &str,
    expression: &Expression<'_>,
) -> Option<(String, ParserByteSpanV0)> {
    match expression {
        Expression::StringLiteral(_) => {}
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {}
        _ => return None,
    }
    let span = parser_byte_span(expression.span());
    let (start, end) = trim_js_expression(source, span.start, span.end);
    let (literal_start, literal_end, next_offset) = js_string_literal_span(source, start, end)?;
    if trim_js_expression(source, next_offset, end).0 < end {
        return None;
    }
    Some((
        source.get(literal_start..literal_end)?.to_string(),
        ParserByteSpanV0 {
            start: literal_start,
            end: literal_end,
        },
    ))
}

fn split_class_names(value: &str) -> Vec<String> {
    let mut class_names = value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    class_names.sort();
    class_names.dedup();
    class_names
}

fn binding_pattern_identifier_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    binding_pattern_identifier(pattern).map(|identifier| identifier.name.as_str())
}

fn binding_pattern_identifier<'a>(
    pattern: &'a BindingPattern<'a>,
) -> Option<&'a BindingIdentifier<'a>> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier),
        _ => None,
    }
}

fn collect_binding_pattern_decl_facts(
    pattern: &BindingPattern<'_>,
    kind: &'static str,
    import_path: Option<&str>,
    out: &mut Vec<SourceBindingDeclFactV0>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            push_binding_identifier_decl_fact(identifier, kind, import_path, out);
        }
        BindingPattern::ObjectPattern(pattern) => {
            for property in &pattern.properties {
                collect_binding_pattern_decl_facts(&property.value, kind, import_path, out);
            }
            if let Some(rest) = &pattern.rest {
                collect_binding_pattern_decl_facts(&rest.argument, kind, import_path, out);
            }
        }
        BindingPattern::ArrayPattern(pattern) => {
            for element in pattern.elements.iter().flatten() {
                collect_binding_pattern_decl_facts(element, kind, import_path, out);
            }
            if let Some(rest) = &pattern.rest {
                collect_binding_pattern_decl_facts(&rest.argument, kind, import_path, out);
            }
        }
        BindingPattern::AssignmentPattern(pattern) => {
            collect_binding_pattern_decl_facts(&pattern.left, kind, import_path, out);
        }
    }
}

fn push_binding_identifier_decl_fact(
    identifier: &BindingIdentifier<'_>,
    kind: &'static str,
    import_path: Option<&str>,
    out: &mut Vec<SourceBindingDeclFactV0>,
) {
    out.push(SourceBindingDeclFactV0 {
        kind,
        name: identifier.name.as_str().to_string(),
        byte_span: parser_byte_span(identifier.span),
        import_path: import_path.map(str::to_string),
    });
}

fn expression_identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    expression_identifier(expression).map(|identifier| identifier.name.as_str())
}

fn expression_identifier<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier),
        Expression::ParenthesizedExpression(expression) => {
            expression_identifier(&expression.expression)
        }
        Expression::TSAsExpression(expression) => expression_identifier(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => {
            expression_identifier(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => expression_identifier(&expression.expression),
        Expression::TSNonNullExpression(expression) => {
            expression_identifier(&expression.expression)
        }
        Expression::TSInstantiationExpression(expression) => {
            expression_identifier(&expression.expression)
        }
        _ => None,
    }
}

fn argument_identifier<'a>(argument: &'a Argument<'a>) -> Option<&'a IdentifierReference<'a>> {
    match argument {
        Argument::Identifier(identifier) => Some(identifier),
        Argument::ParenthesizedExpression(expression) => {
            expression_identifier(&expression.expression)
        }
        Argument::TSAsExpression(expression) => expression_identifier(&expression.expression),
        Argument::TSSatisfiesExpression(expression) => {
            expression_identifier(&expression.expression)
        }
        Argument::TSNonNullExpression(expression) => expression_identifier(&expression.expression),
        Argument::TSInstantiationExpression(expression) => {
            expression_identifier(&expression.expression)
        }
        _ => None,
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

    let expression_path = js_expression_path(source, start, end);
    if let Some(value) =
        source_class_value_from_js_expression(source, start, end, local_class_values)
        && !value.is_empty()
    {
        if let Some(path) = expression_path.as_deref() {
            push_source_type_fact_target(
                ParserByteSpanV0 { start, end },
                path,
                target_style_uri,
                "",
                "",
                type_fact_targets,
            );
        }
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

    if let Some(path) = expression_path {
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
    collect_local_class_value_declarations(source, &mut values);
    collect_local_class_value_reassignments(source, &mut values);
    values
}

fn collect_local_class_value_declarations(
    source: &str,
    values: &mut BTreeMap<String, SourceClassValue>,
) {
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
            source_class_value_from_js_expression(source, expression_start, expression_end, values)
            && !value.is_empty()
        {
            values.insert(binding.to_string(), value);
        }
        let (_, property_values) = source_class_value_from_object_literal(
            source,
            expression_start,
            expression_end,
            values,
        );
        for (property, value) in property_values {
            if !value.is_empty() {
                values.insert(format!("{binding}.{property}"), value);
            }
        }
        cursor = expression_end.min(source.len());
    }
}

fn collect_local_class_value_reassignments(
    source: &str,
    values: &mut BTreeMap<String, SourceClassValue>,
) {
    let mut cursor = 0usize;
    while let Some(identifier) = next_code_identifier(source, cursor) {
        cursor = identifier.end;
        if !values.contains_key(identifier.text) {
            continue;
        }
        let equals_offset = skip_js_trivia(source, identifier.end);
        if !is_simple_js_assignment_operator(source, equals_offset) {
            continue;
        }
        let expression_start = skip_js_trivia(source, equals_offset + 1);
        let expression_end = js_statement_expression_end(source, expression_start);
        if let Some(value) =
            source_class_value_from_js_expression(source, expression_start, expression_end, values)
            && !value.is_empty()
        {
            values
                .entry(identifier.text.to_string())
                .or_default()
                .merge(value);
        }
        cursor = expression_end.min(source.len());
    }
}

fn is_simple_js_assignment_operator(source: &str, offset: usize) -> bool {
    if source.as_bytes().get(offset) != Some(&b'=') {
        return false;
    }
    let previous = offset
        .checked_sub(1)
        .and_then(|index| source.as_bytes().get(index).copied());
    let next = source.as_bytes().get(offset + 1).copied();
    !matches!(previous, Some(b'=' | b'!' | b'<' | b'>')) && !matches!(next, Some(b'=' | b'>'))
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
        if let Some(property_name) = object_property_name(source, property_start, key_end)
            && let Some(property_value) = colon
                .and_then(|colon| {
                    source_class_value_from_js_expression(
                        source,
                        colon + 1,
                        property_end,
                        local_class_values,
                    )
                })
                .filter(|value| !value.is_empty())
        {
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

fn is_class_utility_import_path(import_path: &str) -> bool {
    matches!(import_path, "clsx" | "clsx/lite" | "classnames")
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
mod tests;

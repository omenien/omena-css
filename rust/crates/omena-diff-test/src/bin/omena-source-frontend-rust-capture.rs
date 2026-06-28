use std::{collections::BTreeSet, io};

use omena_query::{
    OmenaQuerySourceImportedStyleBindingV0, OmenaQuerySourceSelectorReferenceMatchKindV0,
    ParserByteSpanV0, summarize_omena_query_source_binding_index_for_source_language,
    summarize_omena_query_source_control_flow_graph_for_source_language,
    summarize_omena_query_source_syntax_index_for_source_language,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RustCaptureRequestV0 {
    fixtures: Vec<RustCaptureFixtureV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RustCaptureFixtureV0 {
    id: String,
    source_path: String,
    source: String,
    source_language: Option<String>,
    imported_style_bindings: Vec<RustImportedStyleBindingInputV0>,
    classnames_bind_bindings: Vec<String>,
    cfg_variable_name: String,
    cfg_reference_byte_offset: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustImportedStyleBindingInputV0 {
    binding: String,
    style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustCaptureResponseV0 {
    schema_version: u8,
    product: &'static str,
    fixtures: Vec<RustFixtureCaptureV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustFixtureCaptureV0 {
    id: String,
    source_path: String,
    syntax: RustSyntaxCaptureV0,
    binding: RustBindingCaptureV0,
    cfg_snapshot: Option<omena_query::OmenaQuerySourceControlFlowGraphCaptureV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustSyntaxCaptureV0 {
    imported_style_bindings: Vec<RustImportedStyleBindingInputV0>,
    style_property_accesses: Vec<RustStylePropertyAccessCaptureV0>,
    selector_references: Vec<RustSelectorReferenceCaptureV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustBindingCaptureV0 {
    binding_scopes: Vec<RustBindingScopeCaptureV0>,
    scope_parent_edges: Vec<RustScopeParentCaptureV0>,
    binding_decls: Vec<RustBindingDeclCaptureV0>,
    scope_contains_decls: Vec<RustScopeContainsDeclCaptureV0>,
    style_modules: Vec<RustStyleModuleCaptureV0>,
    style_import_bindings: Vec<RustBindingStyleImportCaptureV0>,
    declares_style_imports: Vec<RustDeclaresStyleImportCaptureV0>,
    style_import_resolves_modules: Vec<RustStyleImportResolvesModuleCaptureV0>,
    class_expression_nodes: Vec<RustClassExpressionNodeCaptureV0>,
    expression_targets_modules: Vec<RustExpressionTargetsModuleCaptureV0>,
    classnames_bind_utility_bindings: Vec<RustClassnamesBindUtilityBindingCaptureV0>,
    class_util_bindings: Vec<RustClassUtilBindingCaptureV0>,
    declares_utility_bindings: Vec<RustDeclaresUtilityBindingCaptureV0>,
    utility_uses_style_imports: Vec<RustUtilityUsesStyleImportCaptureV0>,
    style_access_uses_style_imports: Vec<RustStyleAccessUsesStyleImportCaptureV0>,
    symbol_ref_uses_decls: Vec<RustSymbolRefUsesDeclCaptureV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustBindingScopeCaptureV0 {
    kind: &'static str,
    byte_span: ParserByteSpanV0,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustScopeParentCaptureV0 {
    child_kind: &'static str,
    child_byte_span: ParserByteSpanV0,
    parent_kind: &'static str,
    parent_byte_span: ParserByteSpanV0,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustBindingDeclCaptureV0 {
    kind: &'static str,
    name: String,
    byte_span: ParserByteSpanV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    import_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustScopeContainsDeclCaptureV0 {
    scope_kind: &'static str,
    scope_byte_span: ParserByteSpanV0,
    decl_kind: &'static str,
    decl_name: String,
    decl_byte_span: ParserByteSpanV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    import_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustStyleModuleCaptureV0 {
    style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustBindingStyleImportCaptureV0 {
    local_name: String,
    style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustDeclaresStyleImportCaptureV0 {
    decl_name: String,
    styles_local_name: String,
    style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustStyleImportResolvesModuleCaptureV0 {
    styles_local_name: String,
    style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustClassExpressionNodeCaptureV0 {
    byte_span: ParserByteSpanV0,
    kind: &'static str,
    target_style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustExpressionTargetsModuleCaptureV0 {
    byte_span: ParserByteSpanV0,
    target_style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustClassnamesBindUtilityBindingCaptureV0 {
    local_name: String,
    styles_local_name: String,
    style_uri: String,
    classnames_import_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustClassUtilBindingCaptureV0 {
    local_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustDeclaresUtilityBindingCaptureV0 {
    decl_name: String,
    utility_local_name: String,
    utility_kind: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustUtilityUsesStyleImportCaptureV0 {
    utility_local_name: String,
    styles_local_name: String,
    style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustStyleAccessUsesStyleImportCaptureV0 {
    byte_span: ParserByteSpanV0,
    decl_name: String,
    styles_local_name: String,
    style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustSymbolRefUsesDeclCaptureV0 {
    byte_span: ParserByteSpanV0,
    raw_reference: String,
    root_name: String,
    decl_name: String,
    style_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustStylePropertyAccessCaptureV0 {
    byte_span: ParserByteSpanV0,
    selector_name: String,
    target_style_uri: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RustSelectorReferenceCaptureV0 {
    byte_span: ParserByteSpanV0,
    selector_name: Option<String>,
    match_kind: &'static str,
    target_style_uri: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request: RustCaptureRequestV0 = serde_json::from_reader(io::stdin())?;
    let response = RustCaptureResponseV0 {
        schema_version: 0,
        product: "omena.source-frontend-rust-capture",
        fixtures: request
            .fixtures
            .into_iter()
            .map(capture_fixture)
            .collect::<Vec<_>>(),
    };
    serde_json::to_writer_pretty(io::stdout(), &response)?;
    Ok(())
}

fn capture_fixture(fixture: RustCaptureFixtureV0) -> RustFixtureCaptureV0 {
    let imported_style_bindings = fixture
        .imported_style_bindings
        .iter()
        .map(|binding| OmenaQuerySourceImportedStyleBindingV0 {
            binding: binding.binding.clone(),
            style_uri: binding.style_uri.clone(),
        })
        .collect::<Vec<_>>();
    let index = summarize_omena_query_source_syntax_index_for_source_language(
        fixture.source_path.as_str(),
        fixture.source.as_str(),
        fixture.source_language.as_deref(),
        imported_style_bindings.clone(),
        fixture.classnames_bind_bindings.clone(),
    );
    let binding_index = summarize_omena_query_source_binding_index_for_source_language(
        fixture.source_path.as_str(),
        fixture.source.as_str(),
        fixture.source_language.as_deref(),
        imported_style_bindings,
        fixture.classnames_bind_bindings,
    );
    let cfg_snapshot = summarize_omena_query_source_control_flow_graph_for_source_language(
        fixture.source_path.as_str(),
        fixture.source.as_str(),
        fixture.source_language.as_deref(),
        fixture.cfg_variable_name.as_str(),
        fixture.cfg_reference_byte_offset,
    );
    let style_modules = binding_index
        .style_import_resolves_modules
        .iter()
        .map(|edge| edge.style_uri.clone())
        .chain(
            binding_index
                .expression_targets_modules
                .iter()
                .map(|edge| edge.target_style_uri.clone()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|style_uri| RustStyleModuleCaptureV0 { style_uri })
        .collect::<Vec<_>>();
    RustFixtureCaptureV0 {
        id: fixture.id,
        source_path: fixture.source_path,
        syntax: RustSyntaxCaptureV0 {
            imported_style_bindings: index
                .imported_style_bindings
                .into_iter()
                .map(|binding| RustImportedStyleBindingInputV0 {
                    binding: binding.binding,
                    style_uri: binding.style_uri,
                })
                .collect(),
            style_property_accesses: index
                .style_property_accesses
                .into_iter()
                .map(|access| RustStylePropertyAccessCaptureV0 {
                    selector_name: source_slice(
                        fixture.source.as_str(),
                        access.byte_span.start,
                        access.byte_span.end,
                    ),
                    byte_span: access.byte_span,
                    target_style_uri: access.target_style_uri,
                })
                .collect(),
            selector_references: index
                .selector_references
                .into_iter()
                .map(|reference| RustSelectorReferenceCaptureV0 {
                    selector_name: reference.selector_name.or_else(|| {
                        Some(source_slice(
                            fixture.source.as_str(),
                            reference.byte_span.start,
                            reference.byte_span.end,
                        ))
                    }),
                    byte_span: reference.byte_span,
                    match_kind: match_kind_label(reference.match_kind),
                    target_style_uri: reference.target_style_uri,
                })
                .collect(),
        },
        binding: RustBindingCaptureV0 {
            binding_scopes: binding_index
                .binding_scopes
                .into_iter()
                .map(|scope| RustBindingScopeCaptureV0 {
                    kind: scope.kind,
                    byte_span: scope.byte_span,
                })
                .collect(),
            scope_parent_edges: binding_index
                .scope_parent_edges
                .into_iter()
                .map(|edge| RustScopeParentCaptureV0 {
                    child_kind: edge.child_kind,
                    child_byte_span: edge.child_byte_span,
                    parent_kind: edge.parent_kind,
                    parent_byte_span: edge.parent_byte_span,
                })
                .collect(),
            binding_decls: binding_index
                .binding_decls
                .into_iter()
                .map(|decl| RustBindingDeclCaptureV0 {
                    kind: decl.kind,
                    name: decl.name,
                    byte_span: decl.byte_span,
                    import_path: decl.import_path,
                })
                .collect(),
            scope_contains_decls: binding_index
                .scope_contains_decls
                .into_iter()
                .map(|edge| RustScopeContainsDeclCaptureV0 {
                    scope_kind: edge.scope_kind,
                    scope_byte_span: edge.scope_byte_span,
                    decl_kind: edge.decl_kind,
                    decl_name: edge.decl_name,
                    decl_byte_span: edge.decl_byte_span,
                    import_path: edge.import_path,
                })
                .collect(),
            style_modules,
            style_import_bindings: binding_index
                .style_import_bindings
                .into_iter()
                .map(|binding| RustBindingStyleImportCaptureV0 {
                    local_name: binding.local_name,
                    style_uri: binding.style_uri,
                })
                .collect(),
            declares_style_imports: binding_index
                .declares_style_imports
                .into_iter()
                .map(|edge| RustDeclaresStyleImportCaptureV0 {
                    decl_name: edge.decl_name,
                    styles_local_name: edge.styles_local_name,
                    style_uri: edge.style_uri,
                })
                .collect(),
            style_import_resolves_modules: binding_index
                .style_import_resolves_modules
                .into_iter()
                .map(|edge| RustStyleImportResolvesModuleCaptureV0 {
                    styles_local_name: edge.styles_local_name,
                    style_uri: edge.style_uri,
                })
                .collect(),
            class_expression_nodes: binding_index
                .class_expression_nodes
                .into_iter()
                .map(|node| RustClassExpressionNodeCaptureV0 {
                    kind: node.kind,
                    byte_span: node.byte_span,
                    target_style_uri: node.target_style_uri,
                })
                .collect(),
            expression_targets_modules: binding_index
                .expression_targets_modules
                .into_iter()
                .map(|edge| RustExpressionTargetsModuleCaptureV0 {
                    byte_span: edge.byte_span,
                    target_style_uri: edge.target_style_uri,
                })
                .collect(),
            classnames_bind_utility_bindings: binding_index
                .classnames_bind_utility_bindings
                .into_iter()
                .map(|binding| RustClassnamesBindUtilityBindingCaptureV0 {
                    local_name: binding.local_name,
                    styles_local_name: binding.styles_local_name,
                    style_uri: binding.style_uri,
                    classnames_import_name: binding.classnames_import_name,
                })
                .collect(),
            class_util_bindings: binding_index
                .class_util_bindings
                .into_iter()
                .map(|binding| RustClassUtilBindingCaptureV0 {
                    local_name: binding.local_name,
                })
                .collect(),
            declares_utility_bindings: binding_index
                .declares_utility_bindings
                .into_iter()
                .map(|edge| RustDeclaresUtilityBindingCaptureV0 {
                    decl_name: edge.decl_name,
                    utility_local_name: edge.utility_local_name,
                    utility_kind: edge.utility_kind,
                })
                .collect(),
            utility_uses_style_imports: binding_index
                .utility_uses_style_imports
                .into_iter()
                .map(|edge| RustUtilityUsesStyleImportCaptureV0 {
                    utility_local_name: edge.utility_local_name,
                    styles_local_name: edge.styles_local_name,
                    style_uri: edge.style_uri,
                })
                .collect(),
            style_access_uses_style_imports: binding_index
                .style_access_uses_style_imports
                .into_iter()
                .map(|edge| RustStyleAccessUsesStyleImportCaptureV0 {
                    byte_span: edge.byte_span,
                    decl_name: edge.decl_name,
                    styles_local_name: edge.styles_local_name,
                    style_uri: edge.style_uri,
                })
                .collect(),
            symbol_ref_uses_decls: binding_index
                .symbol_ref_uses_decls
                .into_iter()
                .map(|edge| RustSymbolRefUsesDeclCaptureV0 {
                    byte_span: edge.byte_span,
                    raw_reference: edge.raw_reference,
                    root_name: edge.root_name,
                    decl_name: edge.decl_name,
                    style_uri: edge.style_uri,
                })
                .collect(),
        },
        cfg_snapshot,
    }
}

fn source_slice(source: &str, start: usize, end: usize) -> String {
    source.get(start..end).unwrap_or("").to_string()
}

fn match_kind_label(kind: OmenaQuerySourceSelectorReferenceMatchKindV0) -> &'static str {
    match kind {
        OmenaQuerySourceSelectorReferenceMatchKindV0::Exact => "exact",
        OmenaQuerySourceSelectorReferenceMatchKindV0::Prefix => "prefix",
    }
}

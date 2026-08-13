use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Block, Expr, ExprCall, ExprPath, ExprStruct, File, GenericParam, ImplItemFn, ImplItemType,
    Item, ItemExternCrate, ItemFn, ItemMacro, ItemMod, ItemType, Macro, Stmt, TraitItemFn,
    TraitItemType, Type, UseTree,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceInput {
    path: String,
    source: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScannerInput {
    sources: Vec<SourceInput>,
    product_roots: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProducerOutput {
    path: String,
    symbol: String,
    occurrence: usize,
    scope_proximity_source: ScopeProximitySource,
    line: usize,
    column: usize,
}

#[derive(Clone)]
struct ProducerSite {
    symbol: String,
    scope_proximity_source: ScopeProximitySource,
    span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum ScopeProximitySource {
    ConstantZero,
    LegacySelectorContextFallback,
    CallerSupplied,
    GeneratedValue,
}

struct FunctionContext {
    name: String,
    symbol: String,
    parameter_names: BTreeSet<String>,
    fallback_local_names: BTreeSet<String>,
    fallback_symbol_shadowed: bool,
    local_binding_counts: BTreeMap<String, usize>,
    ambiguous_local_names: BTreeSet<String>,
}

const SCOPE_PROXIMITY_FALLBACK_FN: &str =
    "cascade_scope_proximity_fallback_for_selector_context_rank";

#[derive(Clone, Copy)]
struct LexicalScope {
    canonical_binding: bool,
    cascade_key_shadow: bool,
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct SpanKey {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
}

impl From<Span> for SpanKey {
    fn from(span: Span) -> Self {
        let start = span.start();
        let end = span.end();
        Self {
            start_line: start.line,
            start_column: start.column,
            end_line: end.line,
            end_column: end.column,
        }
    }
}

struct ProducerVisitor<'a> {
    path: &'a str,
    function_stack: Vec<FunctionContext>,
    module_stack: Vec<String>,
    owner_stack: Vec<String>,
    closure_depth: usize,
    cascade_key_impl_depth: usize,
    cascade_key_inherent_impl_depth: usize,
    transform_environment_scope_depth: usize,
    transform_declaration_loop_depth: usize,
    lexical_scopes: Vec<LexicalScope>,
    sites: Vec<ProducerSite>,
    canonical_new_paths: BTreeMap<SpanKey, Span>,
    consumed_new_paths: BTreeSet<SpanKey>,
    errors: Vec<String>,
}

impl<'a> ProducerVisitor<'a> {
    fn new(path: &'a str) -> Self {
        Self {
            path,
            function_stack: Vec::new(),
            module_stack: Vec::new(),
            owner_stack: Vec::new(),
            closure_depth: 0,
            cascade_key_impl_depth: 0,
            cascade_key_inherent_impl_depth: 0,
            transform_environment_scope_depth: 0,
            transform_declaration_loop_depth: 0,
            lexical_scopes: Vec::new(),
            sites: Vec::new(),
            canonical_new_paths: BTreeMap::new(),
            consumed_new_paths: BTreeSet::new(),
            errors: Vec::new(),
        }
    }

    fn finish(mut self) -> Result<Vec<ProducerSite>, String> {
        for (key, span) in &self.canonical_new_paths {
            if !self.consumed_new_paths.contains(key) {
                self.errors.push(self.error_at(
                    *span,
                    "CascadeKey::new function items must be called directly or explicitly classified",
                ));
            }
        }
        if self.errors.is_empty() {
            Ok(self.sites)
        } else {
            Err(self.errors.join("\n"))
        }
    }

    fn error_at(&self, span: Span, message: &str) -> String {
        let start = span.start();
        format!(
            "{}:{}:{}: {message}",
            self.path,
            start.line,
            start.column + 1
        )
    }

    fn record_error(&mut self, span: Span, message: &str) {
        self.errors.push(self.error_at(span, message));
    }

    fn current_symbol(&mut self, span: Span) -> Option<String> {
        if let Some(context) = self.function_stack.last() {
            Some(context.symbol.clone())
        } else {
            self.record_error(span, "CascadeKey producer is outside a function or method");
            None
        }
    }

    fn record_call(&mut self, call: &ExprCall, path: &ExprPath) {
        let Some(kind) = cascade_new_path_kind(path) else {
            return;
        };
        if kind == CascadeNewPathKind::TraitQualified {
            self.record_error(
                path.span(),
                "trait-qualified CascadeKey constructors require an explicit census rule",
            );
            return;
        }
        if kind == CascadeNewPathKind::Unclassified {
            self.record_error(
                path.span(),
                "CascadeKey constructor path requires an explicit census rule",
            );
            return;
        }
        if cascade_new_path_is_crate_qualified(path)
            && !self.path.starts_with("rust/crates/omena-cascade/")
        {
            self.record_error(
                path.span(),
                "crate-qualified CascadeKey is outside the cascade crate",
            );
            return;
        }
        if cascade_new_path_is_unqualified(path) && !self.unqualified_cascade_key_is_canonical() {
            self.record_error(
                path.span(),
                "unqualified CascadeKey constructor has no unshadowed canonical binding",
            );
            return;
        }
        self.consumed_new_paths.insert(SpanKey::from(path.span()));
        if call
            .args
            .iter()
            .any(|argument| has_cfg_attribute(expression_attributes(argument)))
        {
            self.record_error(
                call.span(),
                "cfg-gated CascadeKey constructor arguments require an explicit census rule",
            );
            return;
        }
        if call.args.len() != 5 {
            self.record_error(
                call.span(),
                "CascadeKey::new must have exactly five arguments",
            );
            return;
        }
        let Some(symbol) = self.current_symbol(call.span()) else {
            return;
        };
        let Some(scope_expression) = call.args.iter().nth(2) else {
            self.record_error(call.span(), "CascadeKey::new is missing scope proximity");
            return;
        };
        let scope_proximity_source = match self.classify_scope_proximity(scope_expression) {
            Ok(source) => source,
            Err(message) => {
                self.record_error(scope_expression.span(), message);
                return;
            }
        };
        self.sites.push(ProducerSite {
            symbol,
            scope_proximity_source,
            span: call.span(),
        });
    }

    fn record_struct(&mut self, expression: &ExprStruct) {
        if self.cascade_key_impl_depth > 0
            && normalized_path_segments(&expression.path).as_slice() == ["Self"]
        {
            if self.cascade_key_inherent_impl_depth == 0
                || self
                    .function_stack
                    .last()
                    .is_none_or(|context| context.name != "new")
            {
                self.record_error(
                    expression.path.span(),
                    "Self CascadeKey literals outside the canonical constructor require an explicit census rule",
                );
            }
            return;
        }
        if path_ends_in_cascade_key(&expression.path)
            && path_has_noncanonical_key_spelling(&expression.path)
        {
            self.record_error(
                expression.path.span(),
                "CascadeKey literal spelling requires an explicit census rule",
            );
            return;
        }
        if path_ends_in_cascade_key(&expression.path)
            && !is_canonical_cascade_key_type_path(&expression.path)
        {
            self.record_error(
                expression.path.span(),
                "CascadeKey literal path requires an explicit census rule",
            );
            return;
        }
        if !is_canonical_cascade_key_type_path(&expression.path) {
            return;
        }
        if expression
            .fields
            .iter()
            .any(|field| has_cfg_attribute(&field.attrs))
        {
            self.record_error(
                expression.span(),
                "cfg-gated CascadeKey literal fields require an explicit census rule",
            );
            return;
        }
        if path_is_crate_qualified(&expression.path)
            && !self.path.starts_with("rust/crates/omena-cascade/")
        {
            self.record_error(
                expression.path.span(),
                "crate-qualified CascadeKey literal is outside the cascade crate",
            );
            return;
        }
        if path_is_unqualified_cascade_key(&expression.path)
            && !self.unqualified_cascade_key_is_canonical()
        {
            self.record_error(
                expression.path.span(),
                "unqualified CascadeKey literal has no unshadowed canonical binding",
            );
            return;
        }
        let Some(symbol) = self.current_symbol(expression.span()) else {
            return;
        };
        let Some(scope_field) = expression.fields.iter().find(|field| {
            matches!(&field.member, syn::Member::Named(ident) if normalized_ident(ident) == "scope_proximity")
        }) else {
            self.record_error(expression.span(), "CascadeKey literal is missing scope_proximity");
            return;
        };
        let scope_proximity_source = match self.classify_scope_proximity(&scope_field.expr) {
            Ok(source) => source,
            Err(message) => {
                self.record_error(scope_field.expr.span(), message);
                return;
            }
        };
        self.sites.push(ProducerSite {
            symbol,
            scope_proximity_source,
            span: expression.span(),
        });
    }

    fn with_function(&mut self, signature: &syn::Signature, block: &Block) {
        if generics_bind_omena_cascade(&signature.generics) {
            self.record_error(
                signature.generics.span(),
                "generic bindings named omena_cascade require an explicit census rule",
            );
        }
        let mut local_bindings = local_bindings(block);
        local_bindings
            .ambiguous_names
            .extend(mutable_parameter_names(signature));
        let parameter_names = parameter_names(signature);
        let fallback_symbol_shadowed = parameter_names.contains(SCOPE_PROXIMITY_FALLBACK_FN)
            || local_bindings
                .counts
                .contains_key(SCOPE_PROXIMITY_FALLBACK_FN)
            || local_bindings
                .ambiguous_names
                .contains(SCOPE_PROXIMITY_FALLBACK_FN);
        let name = normalized_ident(&signature.ident);
        let symbol = if let Some(parent) = self.function_stack.last() {
            format!("{}::{name}", parent.symbol)
        } else if let Some(owner) = self.owner_stack.last() {
            self.module_stack
                .iter()
                .chain(std::iter::once(owner))
                .chain(std::iter::once(&name))
                .cloned()
                .collect::<Vec<_>>()
                .join("::")
        } else if !self.module_stack.is_empty() {
            self.module_stack
                .iter()
                .chain(std::iter::once(&name))
                .cloned()
                .collect::<Vec<_>>()
                .join("::")
        } else {
            name.clone()
        };
        self.function_stack.push(FunctionContext {
            name,
            symbol,
            parameter_names,
            fallback_local_names: local_bindings.fallback_names,
            fallback_symbol_shadowed,
            local_binding_counts: local_bindings.counts,
            ambiguous_local_names: local_bindings.ambiguous_names,
        });
        let generic_shadow = signature.generics.params.iter().any(
            |parameter| matches!(parameter, GenericParam::Type(ty) if normalized_ident(&ty.ident) == "CascadeKey"),
        );
        if generic_shadow {
            let inherited = self.current_scope();
            self.lexical_scopes.push(LexicalScope {
                canonical_binding: inherited.canonical_binding,
                cascade_key_shadow: true,
            });
        }
        self.visit_block(block);
        if generic_shadow {
            self.lexical_scopes.pop();
        }
        self.function_stack.pop();
    }

    fn classify_scope_proximity(
        &self,
        expression: &Expr,
    ) -> Result<ScopeProximitySource, &'static str> {
        if matches!(expression, Expr::Lit(literal) if matches!(&literal.lit, syn::Lit::Int(value) if value.base10_digits() == "0"))
        {
            return Ok(ScopeProximitySource::ConstantZero);
        }
        let rendered = expression.to_token_stream().to_string();
        if is_exact_scope_proximity_fallback_call(expression) {
            if self.closure_depth == 0
                && self.is_semantic_fallback_owner()
                && self.fallback_symbol_is_unshadowed()
            {
                return Ok(ScopeProximitySource::LegacySelectorContextFallback);
            }
            return Err("scope-proximity fallback call is outside its declared product owner");
        }
        if let Expr::Path(path) = expression
            && path.qself.is_none()
            && path.path.segments.len() == 1
        {
            if self.closure_depth > 0 {
                return Err("closure-bound scope proximity requires an explicit census rule");
            }
            let name = normalized_ident(&path.path.segments[0].ident);
            if let Some(context) = self.function_stack.last() {
                let binding_count = context
                    .local_binding_counts
                    .get(&name)
                    .copied()
                    .unwrap_or(0);
                if context.ambiguous_local_names.contains(&name)
                    || binding_count > 1
                    || (binding_count > 0 && context.parameter_names.contains(&name))
                {
                    return Err(
                        "scope-proximity binding is shadowed and requires an explicit census rule",
                    );
                }
                if binding_count == 1 && context.fallback_local_names.contains(&name) {
                    if self.is_semantic_fallback_owner() && self.fallback_symbol_is_unshadowed() {
                        return Ok(ScopeProximitySource::LegacySelectorContextFallback);
                    }
                    return Err(
                        "scope-proximity fallback binding is outside its declared product owner",
                    );
                }
                if context.parameter_names.contains(&name) {
                    return Ok(ScopeProximitySource::CallerSupplied);
                }
            }
        }
        if rendered.replace(' ', "").contains(".scope_proximity") {
            if self.closure_depth > 0 {
                return Err("closure-bound scope proximity requires an explicit census rule");
            }
            let is_transform_boundary = self.path
                == "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs"
                && self
                    .function_stack
                    .last()
                    .is_some_and(|context| context.name == "winner_for_pair")
                && self.transform_declaration_loop_depth > 0
                && self.function_stack.last().is_some_and(|context| {
                    context.local_binding_counts.get("declaration").copied() == Some(1)
                        && !context.ambiguous_local_names.contains("declaration")
                        && context.local_binding_counts.get("environment").copied() == Some(1)
                        && !context.ambiguous_local_names.contains("environment")
                        && context.parameter_names.contains("cascade_environment")
                        && context
                            .local_binding_counts
                            .get("cascade_environment")
                            .copied()
                            .unwrap_or(0)
                            == 0
                        && !context
                            .ambiguous_local_names
                            .contains("cascade_environment")
                })
                && rendered.replace(' ', "") == "declaration.scope_proximity.unwrap_or(0)";
            let is_proof_kernel_certificate_boundary = self.path
                == "rust/crates/omena-cascade-proof/src/proof_kernel.rs"
                && self
                    .function_stack
                    .last()
                    .is_some_and(|context| context.name == "cascade_key_from_certificate_v0")
                && self.function_stack.last().is_some_and(|context| {
                    context.parameter_names.contains("key")
                        && context
                            .local_binding_counts
                            .get("key")
                            .copied()
                            .unwrap_or(0)
                            == 0
                        && !context.ambiguous_local_names.contains("key")
                })
                && rendered.replace(' ', "") == "key.scope_proximity";
            if is_transform_boundary || is_proof_kernel_certificate_boundary {
                return Ok(ScopeProximitySource::CallerSupplied);
            }
            return Err("member-derived scope proximity requires an explicit caller-boundary rule");
        }
        Ok(ScopeProximitySource::GeneratedValue)
    }

    fn is_semantic_fallback_owner(&self) -> bool {
        self.path == "rust/crates/omena-semantic/src/design_tokens.rs"
            && self
                .function_stack
                .last()
                .is_some_and(|context| context.name == "cascade_key")
    }

    fn fallback_symbol_is_unshadowed(&self) -> bool {
        self.function_stack
            .last()
            .is_some_and(|context| !context.fallback_symbol_shadowed)
    }

    fn current_scope(&self) -> LexicalScope {
        self.lexical_scopes.last().copied().unwrap_or(LexicalScope {
            canonical_binding: self.path == "rust/crates/omena-cascade/src/model.rs",
            cascade_key_shadow: false,
        })
    }

    fn unqualified_cascade_key_is_canonical(&self) -> bool {
        let scope = self.current_scope();
        scope.canonical_binding && !scope.cascade_key_shadow
    }

    fn push_item_scope(&mut self, items: &[Item]) {
        let inherited = self.current_scope();
        self.lexical_scopes.push(scope_for_items(
            self.path,
            items,
            inherited.canonical_binding,
            inherited.cascade_key_shadow,
        ));
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CascadeNewPathKind {
    Inherent,
    TraitQualified,
    Unclassified,
}

impl<'ast> Visit<'ast> for ProducerVisitor<'_> {
    fn visit_expr(&mut self, node: &'ast Expr) {
        if has_test_attribute(expression_attributes(node)) {
            return;
        }
        visit::visit_expr(self, node);
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        if has_test_attribute(&node.attrs) {
            return;
        }
        visit::visit_arm(self, node);
    }

    fn visit_field_value(&mut self, node: &'ast syn::FieldValue) {
        if has_test_attribute(&node.attrs) {
            return;
        }
        visit::visit_field_value(self, node);
    }

    fn visit_impl_item(&mut self, node: &'ast syn::ImplItem) {
        if impl_item_has_test_attribute(node) {
            return;
        }
        visit::visit_impl_item(self, node);
    }

    fn visit_trait_item(&mut self, node: &'ast syn::TraitItem) {
        if trait_item_has_test_attribute(node) {
            return;
        }
        visit::visit_trait_item(self, node);
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        let excluded = match node {
            Stmt::Local(local) => has_test_attribute(&local.attrs),
            Stmt::Expr(expression, _) => has_test_attribute(expression_attributes(expression)),
            Stmt::Macro(mac) => has_test_attribute(&mac.attrs),
            Stmt::Item(_) => false,
        };
        if excluded {
            return;
        }
        visit::visit_stmt(self, node);
    }

    fn visit_item(&mut self, node: &'ast Item) {
        if item_has_test_attribute(node) {
            return;
        }
        if item_binds_omena_cascade_type_namespace(node) {
            self.record_error(
                node.span(),
                "local type-namespace bindings named omena_cascade require an explicit census rule",
            );
            return;
        }
        if item_aliases_protected_owner(node) {
            self.record_error(
                node.span(),
                "producer owner type aliases require an explicit census rule",
            );
            return;
        }
        visit::visit_item(self, node);
    }

    fn visit_file(&mut self, node: &'ast File) {
        self.push_item_scope(&node.items);
        for item in &node.items {
            self.visit_item(item);
        }
        self.lexical_scopes.pop();
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if has_test_attribute(&node.attrs) {
            return;
        }
        self.with_function(&node.sig, &node.block);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        if has_test_attribute(&node.attrs) {
            return;
        }
        self.with_function(&node.sig, &node.block);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        if has_test_attribute(&node.attrs) {
            return;
        }
        if let Some(block) = &node.default {
            self.with_function(&node.sig, block);
        }
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if has_test_attribute(&node.attrs) {
            return;
        }
        if normalized_ident(&node.ident) == "omena_cascade" {
            self.record_error(
                node.span(),
                "local modules named omena_cascade require an explicit census rule",
            );
            return;
        }
        if let Some((_, items)) = &node.content {
            self.module_stack.push(normalized_ident(&node.ident));
            self.lexical_scopes
                .push(scope_for_items(self.path, items, false, false));
            for item in items {
                self.visit_item(item);
            }
            self.lexical_scopes.pop();
            self.module_stack.pop();
        }
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if generics_bind_omena_cascade(&node.generics) {
            self.record_error(
                node.generics.span(),
                "generic bindings named omena_cascade require an explicit census rule",
            );
        }
        let generic_shadow = generics_shadow_cascade_key(&node.generics);
        let cascade_key_impl = is_canonical_cascade_key_type(&node.self_ty);
        let cascade_key_inherent_impl = cascade_key_impl && node.trait_.is_none();
        if generic_shadow {
            let inherited = self.current_scope();
            self.lexical_scopes.push(LexicalScope {
                canonical_binding: inherited.canonical_binding,
                cascade_key_shadow: true,
            });
        }
        if cascade_key_impl {
            self.cascade_key_impl_depth += 1;
        }
        if cascade_key_inherent_impl {
            self.cascade_key_inherent_impl_depth += 1;
        }
        self.owner_stack.push(impl_owner_label(node));
        visit::visit_item_impl(self, node);
        self.owner_stack.pop();
        if cascade_key_inherent_impl {
            self.cascade_key_inherent_impl_depth -= 1;
        }
        if cascade_key_impl {
            self.cascade_key_impl_depth -= 1;
        }
        if generic_shadow {
            self.lexical_scopes.pop();
        }
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        if generics_bind_omena_cascade(&node.generics) {
            self.record_error(
                node.generics.span(),
                "generic bindings named omena_cascade require an explicit census rule",
            );
        }
        let generic_shadow = generics_shadow_cascade_key(&node.generics);
        if generic_shadow {
            let inherited = self.current_scope();
            self.lexical_scopes.push(LexicalScope {
                canonical_binding: inherited.canonical_binding,
                cascade_key_shadow: true,
            });
        }
        self.owner_stack.push(normalized_ident(&node.ident));
        visit::visit_item_trait(self, node);
        self.owner_stack.pop();
        if generic_shadow {
            self.lexical_scopes.pop();
        }
    }

    fn visit_block(&mut self, node: &'ast Block) {
        let inherited = self.current_scope();
        self.lexical_scopes.push(scope_for_statements(
            self.path,
            &node.stmts,
            inherited.canonical_binding,
            inherited.cascade_key_shadow,
        ));
        visit::visit_block(self, node);
        self.lexical_scopes.pop();
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        match &*node.func {
            Expr::Path(path) => {
                if self.cascade_key_impl_depth > 0
                    && ((path.qself.is_none()
                        && normalized_path_segments(&path.path).as_slice() == ["Self", "new"])
                        || qself_is_self_constructor(path))
                {
                    self.record_error(
                        path.span(),
                        "Self::new CascadeKey calls require an explicit census rule",
                    );
                } else {
                    self.record_call(node, path);
                }
            }
            Expr::Paren(parenthesized) => {
                if let Some(path) = expression_path(&parenthesized.expr)
                    && cascade_new_path_kind(path).is_some()
                {
                    self.record_error(
                        node.func.span(),
                        "parenthesized CascadeKey constructor functions require an explicit census rule",
                    );
                }
            }
            Expr::Group(group) => {
                if let Some(path) = expression_path(&group.expr)
                    && cascade_new_path_kind(path).is_some()
                {
                    self.record_error(
                        node.func.span(),
                        "grouped CascadeKey constructor functions require an explicit census rule",
                    );
                }
            }
            _ => {}
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        if cascade_new_path_kind(node).is_some() {
            self.canonical_new_paths
                .insert(SpanKey::from(node.span()), node.span());
        }
        visit::visit_expr_path(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast ExprStruct) {
        self.record_struct(node);
        visit::visit_expr_struct(self, node);
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        self.closure_depth += 1;
        visit::visit_expr_closure(self, node);
        self.closure_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        let is_transform_declaration_loop = self.path
            == "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs"
            && self
                .function_stack
                .last()
                .is_some_and(|context| context.name == "winner_for_pair")
            && self.transform_environment_scope_depth > 0
            && pattern_is_ident(&node.pat, "declaration")
            && is_transform_environment_declarations_iteration(&node.expr);
        if is_transform_declaration_loop {
            self.transform_declaration_loop_depth += 1;
        }
        visit::visit_expr_for_loop(self, node);
        if is_transform_declaration_loop {
            self.transform_declaration_loop_depth -= 1;
        }
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        let is_transform_environment_scope = self.path
            == "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs"
            && self
                .function_stack
                .last()
                .is_some_and(|context| context.name == "winner_for_pair")
            && is_transform_environment_binding(&node.cond);
        if is_transform_environment_scope {
            self.transform_environment_scope_depth += 1;
        }
        visit::visit_expr_if(self, node);
        if is_transform_environment_scope {
            self.transform_environment_scope_depth -= 1;
        }
    }

    fn visit_item_type(&mut self, node: &'ast ItemType) {
        if type_contains_cascade_key_reference(&node.ty)
            || generics_contain_cascade_key_reference(&node.generics)
        {
            self.record_error(
                node.span(),
                "CascadeKey type aliases require an explicit census rule",
            );
        }
        visit::visit_item_type(self, node);
    }

    fn visit_impl_item_type(&mut self, node: &'ast ImplItemType) {
        if type_contains_cascade_key_reference(&node.ty)
            || generics_contain_cascade_key_reference(&node.generics)
        {
            self.record_error(
                node.span(),
                "CascadeKey associated type aliases require an explicit census rule",
            );
        }
        visit::visit_impl_item_type(self, node);
    }

    fn visit_trait_item_type(&mut self, node: &'ast TraitItemType) {
        if generics_contain_cascade_key_reference(&node.generics)
            || token_stream_mentions_cascade_key(&node.bounds.to_token_stream())
            || node
                .default
                .as_ref()
                .is_some_and(|(_, ty)| type_contains_cascade_key_reference(ty))
        {
            self.record_error(
                node.span(),
                "CascadeKey associated type declarations require an explicit census rule",
            );
        }
        visit::visit_trait_item_type(self, node);
    }

    fn visit_item_extern_crate(&mut self, node: &'ast ItemExternCrate) {
        if normalized_ident(&node.ident) == "omena_cascade"
            || node
                .rename
                .as_ref()
                .is_some_and(|(_, ident)| normalized_ident(ident) == "omena_cascade")
        {
            self.record_error(
                node.span(),
                "extern-crate CascadeKey bindings require an explicit census rule",
            );
        }
        visit::visit_item_extern_crate(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast ItemMacro) {
        self.reject_macro(&node.mac);
        visit::visit_item_macro(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.reject_macro(node);
        visit::visit_macro(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        let mut path = Vec::new();
        inspect_use_tree(self, &node.tree, &mut path);
        visit::visit_item_use(self, node);
    }
}

impl ProducerVisitor<'_> {
    fn reject_macro(&mut self, node: &Macro) {
        if !token_stream_mentions_cascade_key(&node.tokens) {
            return;
        }
        if node.path.is_ident("vec") {
            let parser = Punctuated::<Expr, syn::Token![,]>::parse_terminated;
            match parser.parse2(node.tokens.clone()) {
                Ok(expressions) => {
                    for expression in &expressions {
                        self.visit_expr(expression);
                    }
                    return;
                }
                Err(error) => {
                    self.record_error(
                        node.span(),
                        &format!("cannot parse CascadeKey vec! elements: {error}"),
                    );
                    return;
                }
            }
        }
        self.record_error(
            node.span(),
            "macros containing CascadeKey require an explicit census rule",
        );
    }
}

fn token_stream_mentions_cascade_key(tokens: &TokenStream) -> bool {
    tokens.clone().into_iter().any(|token| match token {
        TokenTree::Group(group) => token_stream_mentions_cascade_key(&group.stream()),
        TokenTree::Ident(ident) => normalized_ident(&ident) == "CascadeKey",
        TokenTree::Literal(_) | TokenTree::Punct(_) => false,
    })
}

fn scope_for_items(
    source_path: &str,
    items: &[Item],
    inherited_binding: bool,
    inherited_shadow: bool,
) -> LexicalScope {
    let mut scope = LexicalScope {
        canonical_binding: inherited_binding,
        cascade_key_shadow: inherited_shadow,
    };
    for item in items {
        if item_has_test_attribute(item) {
            continue;
        }
        match item {
            Item::Use(item_use) => {
                if use_tree_imports_cascade_key(source_path, &item_use.tree, &mut Vec::new()) {
                    scope.canonical_binding = true;
                    scope.cascade_key_shadow = false;
                }
            }
            Item::Struct(item_struct)
                if normalized_ident(&item_struct.ident) == "CascadeKey"
                    && source_path != "rust/crates/omena-cascade/src/model.rs" =>
            {
                scope.cascade_key_shadow = true;
            }
            Item::Enum(item_enum) if normalized_ident(&item_enum.ident) == "CascadeKey" => {
                scope.cascade_key_shadow = true;
            }
            Item::Union(item_union) if normalized_ident(&item_union.ident) == "CascadeKey" => {
                scope.cascade_key_shadow = true;
            }
            Item::Trait(item_trait) if normalized_ident(&item_trait.ident) == "CascadeKey" => {
                scope.cascade_key_shadow = true;
            }
            Item::Type(item_type) if normalized_ident(&item_type.ident) == "CascadeKey" => {
                scope.cascade_key_shadow = true;
            }
            Item::Mod(item_mod) if normalized_ident(&item_mod.ident) == "CascadeKey" => {
                scope.cascade_key_shadow = true;
            }
            _ => {}
        }
    }
    scope
}

fn scope_for_statements(
    source_path: &str,
    statements: &[Stmt],
    inherited_binding: bool,
    inherited_shadow: bool,
) -> LexicalScope {
    let items = statements
        .iter()
        .filter_map(|statement| match statement {
            Stmt::Item(item) => Some(item.clone()),
            Stmt::Local(_) | Stmt::Expr(_, _) | Stmt::Macro(_) => None,
        })
        .collect::<Vec<_>>();
    scope_for_items(source_path, &items, inherited_binding, inherited_shadow)
}

fn use_tree_imports_cascade_key(source_path: &str, tree: &UseTree, path: &mut Vec<String>) -> bool {
    match tree {
        UseTree::Path(entry) => {
            path.push(normalized_ident(&entry.ident));
            let found = use_tree_imports_cascade_key(source_path, &entry.tree, path);
            path.pop();
            found
        }
        UseTree::Name(entry) => {
            let mut full_path = path.clone();
            full_path.push(normalized_ident(&entry.ident));
            is_canonical_cascade_key_import(source_path, &full_path)
        }
        UseTree::Group(group) => group
            .items
            .iter()
            .any(|item| use_tree_imports_cascade_key(source_path, item, path)),
        UseTree::Rename(_) | UseTree::Glob(_) => false,
    }
}

fn inspect_use_tree(visitor: &mut ProducerVisitor<'_>, tree: &UseTree, path: &mut Vec<String>) {
    match tree {
        UseTree::Path(entry) => {
            path.push(normalized_ident(&entry.ident));
            inspect_use_tree(visitor, &entry.tree, path);
            path.pop();
        }
        UseTree::Name(entry) => {
            let mut full_path = path.clone();
            full_path.push(normalized_ident(&entry.ident));
            if full_path
                .last()
                .is_some_and(|segment| segment == "omena_cascade")
                && full_path.as_slice() != ["omena_cascade"]
            {
                visitor.record_error(
                    entry.span(),
                    "noncanonical omena_cascade imports require an explicit census rule",
                );
            }
            if full_path
                .last()
                .is_some_and(|segment| segment == "CascadeKey")
                && !is_canonical_cascade_key_import(visitor.path, &full_path)
            {
                visitor.record_error(
                    entry.span(),
                    "noncanonical CascadeKey imports require an explicit census rule",
                );
            }
        }
        UseTree::Rename(entry) => {
            let source_name = normalized_ident(&entry.ident);
            let target_name = normalized_ident(&entry.rename);
            if source_name == "CascadeKey"
                || source_name == "omena_cascade"
                || target_name == "CascadeKey"
                || target_name == "omena_cascade"
                || protected_owner_name(&source_name)
                || protected_owner_name(&target_name)
            {
                visitor.record_error(
                    entry.span(),
                    "aliased CascadeKey imports require an explicit census rule",
                );
            }
        }
        UseTree::Glob(entry) => {
            if path.iter().any(|segment| segment == "omena_cascade") {
                visitor.record_error(
                    entry.span(),
                    "glob imports that can bind CascadeKey require an explicit census rule",
                );
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                inspect_use_tree(visitor, item, path);
            }
        }
    }
}

fn is_canonical_cascade_key_import(source_path: &str, path: &[String]) -> bool {
    matches!(
        path,
        [crate_name, key] if crate_name == "omena_cascade" && key == "CascadeKey"
    ) || matches!(
        path,
        [crate_name, module, key]
            if crate_name == "omena_cascade" && module == "model" && key == "CascadeKey"
    ) || (source_path.starts_with("rust/crates/omena-cascade/")
        && path.last().is_some_and(|segment| segment == "CascadeKey")
        && path
            .first()
            .is_some_and(|segment| matches!(segment.as_str(), "crate" | "self" | "super")))
}

fn expression_path(expression: &Expr) -> Option<&ExprPath> {
    match expression {
        Expr::Path(path) => Some(path),
        Expr::Group(group) => expression_path(&group.expr),
        Expr::Paren(parenthesized) => expression_path(&parenthesized.expr),
        _ => None,
    }
}

fn cascade_new_path_kind(path: &ExprPath) -> Option<CascadeNewPathKind> {
    if normalized_path_segments(&path.path)
        .last()
        .is_none_or(|segment| segment != "new")
    {
        return None;
    }
    if let Some(qself) = &path.qself {
        if !type_ends_in_cascade_key(&qself.ty) {
            return (type_contains_cascade_key_reference(&qself.ty)
                || path_arguments_contain_cascade_key_reference(&path.path))
            .then_some(CascadeNewPathKind::Unclassified);
        }
        return Some(if qself.position == 0 {
            CascadeNewPathKind::Unclassified
        } else {
            CascadeNewPathKind::TraitQualified
        });
    }
    let segments = normalized_path_segments(&path.path);
    if segments
        .iter()
        .rev()
        .nth(1)
        .is_none_or(|segment| segment != "CascadeKey")
    {
        return path_arguments_contain_cascade_key_reference(&path.path)
            .then_some(CascadeNewPathKind::Unclassified);
    }
    if path_has_noncanonical_constructor_spelling(&path.path) {
        return Some(CascadeNewPathKind::Unclassified);
    }
    if is_canonical_cascade_key_new_path(&segments) {
        Some(CascadeNewPathKind::Inherent)
    } else if segments
        .iter()
        .rev()
        .nth(1)
        .is_some_and(|segment| segment == "CascadeKey")
    {
        Some(CascadeNewPathKind::Unclassified)
    } else {
        None
    }
}

fn qself_is_self_constructor(path: &ExprPath) -> bool {
    let Some(qself) = &path.qself else {
        return false;
    };
    matches!(
        &*qself.ty,
        Type::Path(ty)
            if ty.qself.is_none()
                && normalized_path_segments(&ty.path).as_slice() == ["Self"]
    ) && normalized_path_segments(&path.path)
        .last()
        .is_some_and(|segment| segment == "new")
}

fn path_has_noncanonical_constructor_spelling(path: &syn::Path) -> bool {
    path.segments.iter().any(|segment| {
        let ident = segment.ident.to_string();
        ((normalized_ident(&segment.ident) == "CascadeKey"
            || normalized_ident(&segment.ident) == "new")
            && ident.starts_with("r#"))
            || !matches!(segment.arguments, syn::PathArguments::None)
    })
}

fn path_has_noncanonical_key_spelling(path: &syn::Path) -> bool {
    path.segments.iter().any(|segment| {
        (normalized_ident(&segment.ident) == "CascadeKey"
            && segment.ident.to_string().starts_with("r#"))
            || !matches!(segment.arguments, syn::PathArguments::None)
    })
}

fn cascade_new_path_is_unqualified(path: &ExprPath) -> bool {
    if let Some(qself) = &path.qself {
        return type_is_unqualified_cascade_key(&qself.ty);
    }
    matches!(
        normalized_path_segments(&path.path).as_slice(),
        [key, new] if key == "CascadeKey" && new == "new"
    )
}

fn cascade_new_path_is_crate_qualified(path: &ExprPath) -> bool {
    path.qself.is_none()
        && normalized_path_segments(&path.path)
            .first()
            .is_some_and(|segment| segment == "crate")
}

fn type_ends_in_cascade_key(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path_ends_in_cascade_key(&path.path),
        Type::Group(group) => type_ends_in_cascade_key(&group.elem),
        Type::Paren(parenthesized) => type_ends_in_cascade_key(&parenthesized.elem),
        _ => false,
    }
}

fn type_is_unqualified_cascade_key(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path_is_unqualified_cascade_key(&path.path),
        Type::Group(group) => type_is_unqualified_cascade_key(&group.elem),
        Type::Paren(parenthesized) => type_is_unqualified_cascade_key(&parenthesized.elem),
        _ => false,
    }
}

fn path_ends_in_cascade_key(path: &syn::Path) -> bool {
    normalized_path_segments(path)
        .last()
        .is_some_and(|segment| segment == "CascadeKey")
}

fn path_is_unqualified_cascade_key(path: &syn::Path) -> bool {
    matches!(normalized_path_segments(path).as_slice(), [key] if key == "CascadeKey")
}

fn path_is_crate_qualified(path: &syn::Path) -> bool {
    normalized_path_segments(path)
        .first()
        .is_some_and(|segment| segment == "crate")
}

fn is_canonical_cascade_key_type(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => is_canonical_cascade_key_type_path(&path.path),
        Type::Group(group) => is_canonical_cascade_key_type(&group.elem),
        Type::Paren(parenthesized) => is_canonical_cascade_key_type(&parenthesized.elem),
        _ => false,
    }
}

fn type_contains_cascade_key_reference(ty: &Type) -> bool {
    token_stream_mentions_cascade_key(&ty.to_token_stream())
}

fn generics_contain_cascade_key_reference(generics: &syn::Generics) -> bool {
    token_stream_mentions_cascade_key(&generics.params.to_token_stream())
        || generics
            .where_clause
            .as_ref()
            .is_some_and(|clause| token_stream_mentions_cascade_key(&clause.to_token_stream()))
}

fn path_arguments_contain_cascade_key_reference(path: &syn::Path) -> bool {
    path.segments
        .iter()
        .any(|segment| token_stream_mentions_cascade_key(&segment.arguments.to_token_stream()))
}

fn is_canonical_cascade_key_type_path(path: &syn::Path) -> bool {
    let segments = normalized_path_segments(path);
    matches!(segments.as_slice(), [key] if key == "CascadeKey")
        || matches!(
            segments.as_slice(),
            [crate_name, key] if crate_name == "omena_cascade" && key == "CascadeKey"
        )
        || matches!(
            segments.as_slice(),
            [crate_name, module, key]
                if crate_name == "omena_cascade" && module == "model" && key == "CascadeKey"
        )
        || matches!(
            segments.as_slice(),
            [crate_name, key] if crate_name == "crate" && key == "CascadeKey"
        )
        || matches!(
            segments.as_slice(),
            [crate_name, module, key]
                if crate_name == "crate" && module == "model" && key == "CascadeKey"
        )
}

fn is_canonical_cascade_key_new_path(segments: &[String]) -> bool {
    matches!(segments, [key, new] if key == "CascadeKey" && new == "new")
        || matches!(
            segments,
            [crate_name, key, new]
                if crate_name == "omena_cascade" && key == "CascadeKey" && new == "new"
        )
        || matches!(
            segments,
            [crate_name, module, key, new]
                if crate_name == "omena_cascade"
                    && module == "model"
                    && key == "CascadeKey"
                    && new == "new"
        )
        || matches!(
            segments,
            [crate_name, key, new]
                if crate_name == "crate" && key == "CascadeKey" && new == "new"
        )
        || matches!(
            segments,
            [crate_name, module, key, new]
                if crate_name == "crate"
                    && module == "model"
                    && key == "CascadeKey"
                    && new == "new"
        )
}

fn normalized_path_segments(path: &syn::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| normalized_ident(&segment.ident))
        .collect()
}

fn normalized_ident(ident: &syn::Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_string()
}

fn parameter_names(signature: &syn::Signature) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for input in &signature.inputs {
        if let syn::FnArg::Typed(typed) = input {
            collect_pattern_names(&typed.pat, &mut names);
        }
    }
    names
}

fn mutable_parameter_names(signature: &syn::Signature) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for input in &signature.inputs {
        if let syn::FnArg::Typed(typed) = input {
            collect_mutable_pattern_names(&typed.pat, &mut names);
        }
    }
    names
}

fn generics_shadow_cascade_key(generics: &syn::Generics) -> bool {
    generics.params.iter().any(
        |parameter| matches!(parameter, GenericParam::Type(ty) if normalized_ident(&ty.ident) == "CascadeKey"),
    )
}

fn generics_bind_omena_cascade(generics: &syn::Generics) -> bool {
    generics.params.iter().any(|parameter| match parameter {
        GenericParam::Type(ty) => normalized_ident(&ty.ident) == "omena_cascade",
        GenericParam::Const(constant) => normalized_ident(&constant.ident) == "omena_cascade",
        GenericParam::Lifetime(_) => false,
    })
}

fn item_binds_omena_cascade_type_namespace(item: &Item) -> bool {
    match item {
        Item::Struct(item) => normalized_ident(&item.ident) == "omena_cascade",
        Item::Enum(item) => normalized_ident(&item.ident) == "omena_cascade",
        Item::Union(item) => normalized_ident(&item.ident) == "omena_cascade",
        Item::Trait(item) => normalized_ident(&item.ident) == "omena_cascade",
        Item::TraitAlias(item) => normalized_ident(&item.ident) == "omena_cascade",
        Item::Type(item) => normalized_ident(&item.ident) == "omena_cascade",
        Item::Mod(item) => normalized_ident(&item.ident) == "omena_cascade",
        _ => false,
    }
}

fn item_aliases_protected_owner(item: &Item) -> bool {
    matches!(item, Item::Type(item) if protected_owner_name(&normalized_ident(&item.ident)))
}

fn protected_owner_name(name: &str) -> bool {
    matches!(
        name,
        "LinkedStylesheetRuleV0" | "DesignTokenCandidateDeclaration"
    )
}

fn impl_owner_label(item: &syn::ItemImpl) -> String {
    let self_ty = item.self_ty.to_token_stream().to_string().replace(' ', "");
    if let Some((_, trait_path, _)) = &item.trait_ {
        let trait_name = trait_path.to_token_stream().to_string().replace(' ', "");
        format!("<{self_ty}as{trait_name}>")
    } else {
        self_ty
    }
}

fn collect_pattern_names(pattern: &syn::Pat, names: &mut BTreeSet<String>) {
    match pattern {
        syn::Pat::Ident(ident) => {
            names.insert(normalized_ident(&ident.ident));
            if let Some((_, subpattern)) = &ident.subpat {
                collect_pattern_names(subpattern, names);
            }
        }
        syn::Pat::Reference(reference) => collect_pattern_names(&reference.pat, names),
        syn::Pat::Type(typed) => collect_pattern_names(&typed.pat, names),
        syn::Pat::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::TupleStruct(tuple) => {
            for element in &tuple.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::Struct(structure) => {
            for field in &structure.fields {
                collect_pattern_names(&field.pat, names);
            }
        }
        syn::Pat::Slice(slice) => {
            for element in &slice.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::Or(or_pattern) => {
            for case in &or_pattern.cases {
                collect_pattern_names(case, names);
            }
        }
        _ => {}
    }
}

struct LocalBindings {
    counts: BTreeMap<String, usize>,
    fallback_names: BTreeSet<String>,
    ambiguous_names: BTreeSet<String>,
}

fn local_bindings(block: &Block) -> LocalBindings {
    struct Collector {
        counts: BTreeMap<String, usize>,
        fallback_names: BTreeSet<String>,
        ambiguous_names: BTreeSet<String>,
    }

    impl<'ast> Visit<'ast> for Collector {
        fn visit_expr(&mut self, expression: &'ast Expr) {
            if has_test_attribute(expression_attributes(expression)) {
                return;
            }
            visit::visit_expr(self, expression);
        }

        fn visit_stmt(&mut self, statement: &'ast Stmt) {
            let excluded = match statement {
                Stmt::Local(local) => has_test_attribute(&local.attrs),
                Stmt::Expr(expression, _) => has_test_attribute(expression_attributes(expression)),
                Stmt::Macro(mac) => has_test_attribute(&mac.attrs),
                Stmt::Item(item) => item_has_test_attribute(item),
            };
            if excluded {
                return;
            }
            visit::visit_stmt(self, statement);
        }

        fn visit_local(&mut self, local: &'ast syn::Local) {
            let mut names = BTreeSet::new();
            collect_pattern_names(&local.pat, &mut names);
            for name in &names {
                *self.counts.entry(name.clone()).or_default() += 1;
            }
            collect_mutable_pattern_names(&local.pat, &mut self.ambiguous_names);
            if local
                .init
                .as_ref()
                .is_some_and(|init| is_exact_scope_proximity_fallback_call(&init.expr))
            {
                self.fallback_names.extend(names);
            }
            visit::visit_local(self, local);
        }

        fn visit_expr_let(&mut self, expression: &'ast syn::ExprLet) {
            increment_pattern_counts(&expression.pat, &mut self.counts);
            collect_mutable_pattern_names(&expression.pat, &mut self.ambiguous_names);
            visit::visit_expr_let(self, expression);
        }

        fn visit_arm(&mut self, arm: &'ast syn::Arm) {
            if has_test_attribute(&arm.attrs) {
                return;
            }
            increment_pattern_counts(&arm.pat, &mut self.counts);
            collect_mutable_pattern_names(&arm.pat, &mut self.ambiguous_names);
            visit::visit_arm(self, arm);
        }

        fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
            increment_pattern_counts(&expression.pat, &mut self.counts);
            collect_mutable_pattern_names(&expression.pat, &mut self.ambiguous_names);
            visit::visit_expr_for_loop(self, expression);
        }

        fn visit_expr_assign(&mut self, assignment: &'ast syn::ExprAssign) {
            collect_assignment_target_names(&assignment.left, &mut self.ambiguous_names);
            visit::visit_expr_assign(self, assignment);
        }

        fn visit_expr_binary(&mut self, binary: &'ast syn::ExprBinary) {
            if bin_op_is_assignment(&binary.op) {
                collect_assignment_target_names(&binary.left, &mut self.ambiguous_names);
            }
            visit::visit_expr_binary(self, binary);
        }

        fn visit_item_fn(&mut self, node: &'ast ItemFn) {
            *self
                .counts
                .entry(normalized_ident(&node.sig.ident))
                .or_default() += 1;
        }

        fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
            *self
                .counts
                .entry(normalized_ident(&node.ident))
                .or_default() += 1;
        }

        fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
            *self
                .counts
                .entry(normalized_ident(&node.ident))
                .or_default() += 1;
        }

        fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
            *self
                .counts
                .entry(normalized_ident(&node.ident))
                .or_default() += 1;
        }

        fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
            if use_tree_can_bind_name(&node.tree, SCOPE_PROXIMITY_FALLBACK_FN) {
                self.ambiguous_names
                    .insert(SCOPE_PROXIMITY_FALLBACK_FN.to_string());
            }
        }
    }

    let mut collector = Collector {
        counts: BTreeMap::new(),
        fallback_names: BTreeSet::new(),
        ambiguous_names: BTreeSet::new(),
    };
    collector.visit_block(block);
    LocalBindings {
        counts: collector.counts,
        fallback_names: collector.fallback_names,
        ambiguous_names: collector.ambiguous_names,
    }
}

fn increment_pattern_counts(pattern: &syn::Pat, counts: &mut BTreeMap<String, usize>) {
    let mut names = BTreeSet::new();
    collect_pattern_names(pattern, &mut names);
    for name in names {
        *counts.entry(name).or_default() += 1;
    }
}

fn pattern_is_ident(pattern: &syn::Pat, expected: &str) -> bool {
    matches!(
        pattern,
        syn::Pat::Ident(ident)
            if ident.subpat.is_none()
                && ident.mutability.is_none()
                && ident.by_ref.is_none()
                && normalized_ident(&ident.ident) == expected
    )
}

fn is_transform_environment_declarations_iteration(expression: &Expr) -> bool {
    let Expr::MethodCall(filter) = expression else {
        return false;
    };
    if normalized_ident(&filter.method) != "filter"
        || filter.turbofish.is_some()
        || filter.args.len() != 1
    {
        return false;
    }
    let Expr::MethodCall(iter) = &*filter.receiver else {
        return false;
    };
    if normalized_ident(&iter.method) != "iter" || iter.turbofish.is_some() || !iter.args.is_empty()
    {
        return false;
    }
    let Expr::Field(declarations) = &*iter.receiver else {
        return false;
    };
    if !matches!(&declarations.member, syn::Member::Named(ident) if normalized_ident(ident) == "declarations")
    {
        return false;
    }
    matches!(
        &*declarations.base,
        Expr::Path(path)
            if path.qself.is_none()
                && normalized_path_segments(&path.path).as_slice() == ["environment"]
    )
}

fn is_transform_environment_binding(expression: &Expr) -> bool {
    let Expr::Let(binding) = expression else {
        return false;
    };
    let syn::Pat::TupleStruct(pattern) = &*binding.pat else {
        return false;
    };
    if normalized_path_segments(&pattern.path).as_slice() != ["Some"]
        || pattern.elems.len() != 1
        || !pattern_is_ident(&pattern.elems[0], "environment")
    {
        return false;
    }
    matches!(
        &*binding.expr,
        Expr::Path(path)
            if path.qself.is_none()
                && normalized_path_segments(&path.path).as_slice() == ["cascade_environment"]
    )
}

fn bin_op_is_assignment(operator: &syn::BinOp) -> bool {
    matches!(
        operator,
        syn::BinOp::AddAssign(_)
            | syn::BinOp::SubAssign(_)
            | syn::BinOp::MulAssign(_)
            | syn::BinOp::DivAssign(_)
            | syn::BinOp::RemAssign(_)
            | syn::BinOp::BitXorAssign(_)
            | syn::BinOp::BitAndAssign(_)
            | syn::BinOp::BitOrAssign(_)
            | syn::BinOp::ShlAssign(_)
            | syn::BinOp::ShrAssign(_)
    )
}

fn collect_assignment_target_names(expression: &Expr, names: &mut BTreeSet<String>) {
    match expression {
        Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            names.insert(normalized_ident(&path.path.segments[0].ident));
        }
        Expr::Array(array) => {
            for element in &array.elems {
                collect_assignment_target_names(element, names);
            }
        }
        Expr::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_assignment_target_names(element, names);
            }
        }
        Expr::Paren(parenthesized) => collect_assignment_target_names(&parenthesized.expr, names),
        Expr::Group(group) => collect_assignment_target_names(&group.expr, names),
        Expr::Struct(structure) => {
            for field in &structure.fields {
                collect_assignment_target_names(&field.expr, names);
            }
        }
        _ => {}
    }
}

fn is_exact_scope_proximity_fallback_call(expression: &Expr) -> bool {
    let Expr::Call(call) = expression else {
        return false;
    };
    let Expr::Path(path) = &*call.func else {
        return false;
    };
    path.qself.is_none()
        && path.path.segments.len() == 1
        && path.path.segments.last().is_some_and(|segment| {
            normalized_ident(&segment.ident) == SCOPE_PROXIMITY_FALLBACK_FN
                && matches!(segment.arguments, syn::PathArguments::None)
        })
}

fn use_tree_can_bind_name(tree: &UseTree, name: &str) -> bool {
    match tree {
        UseTree::Path(entry) => use_tree_can_bind_name(&entry.tree, name),
        UseTree::Name(entry) => normalized_ident(&entry.ident) == name,
        UseTree::Rename(entry) => normalized_ident(&entry.rename) == name,
        UseTree::Glob(_) => true,
        UseTree::Group(group) => group
            .items
            .iter()
            .any(|entry| use_tree_can_bind_name(entry, name)),
    }
}

fn collect_mutable_pattern_names(pattern: &syn::Pat, names: &mut BTreeSet<String>) {
    match pattern {
        syn::Pat::Ident(ident) => {
            if ident.mutability.is_some() {
                names.insert(normalized_ident(&ident.ident));
            }
            if let Some((_, subpattern)) = &ident.subpat {
                collect_mutable_pattern_names(subpattern, names);
            }
        }
        syn::Pat::Reference(reference) => collect_mutable_pattern_names(&reference.pat, names),
        syn::Pat::Type(typed) => collect_mutable_pattern_names(&typed.pat, names),
        syn::Pat::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_mutable_pattern_names(element, names);
            }
        }
        syn::Pat::TupleStruct(tuple) => {
            for element in &tuple.elems {
                collect_mutable_pattern_names(element, names);
            }
        }
        syn::Pat::Struct(structure) => {
            for field in &structure.fields {
                collect_mutable_pattern_names(&field.pat, names);
            }
        }
        syn::Pat::Slice(slice) => {
            for element in &slice.elems {
                collect_mutable_pattern_names(element, names);
            }
        }
        syn::Pat::Or(or_pattern) => {
            for case in &or_pattern.cases {
                collect_mutable_pattern_names(case, names);
            }
        }
        _ => {}
    }
}

fn expression_attributes(expression: &Expr) -> &[syn::Attribute] {
    match expression {
        Expr::Array(expr) => &expr.attrs,
        Expr::Assign(expr) => &expr.attrs,
        Expr::Async(expr) => &expr.attrs,
        Expr::Await(expr) => &expr.attrs,
        Expr::Binary(expr) => &expr.attrs,
        Expr::Block(expr) => &expr.attrs,
        Expr::Break(expr) => &expr.attrs,
        Expr::Call(expr) => &expr.attrs,
        Expr::Cast(expr) => &expr.attrs,
        Expr::Closure(expr) => &expr.attrs,
        Expr::Const(expr) => &expr.attrs,
        Expr::Continue(expr) => &expr.attrs,
        Expr::Field(expr) => &expr.attrs,
        Expr::ForLoop(expr) => &expr.attrs,
        Expr::Group(expr) => &expr.attrs,
        Expr::If(expr) => &expr.attrs,
        Expr::Index(expr) => &expr.attrs,
        Expr::Infer(expr) => &expr.attrs,
        Expr::Let(expr) => &expr.attrs,
        Expr::Lit(expr) => &expr.attrs,
        Expr::Loop(expr) => &expr.attrs,
        Expr::Macro(expr) => &expr.attrs,
        Expr::Match(expr) => &expr.attrs,
        Expr::MethodCall(expr) => &expr.attrs,
        Expr::Paren(expr) => &expr.attrs,
        Expr::Path(expr) => &expr.attrs,
        Expr::Range(expr) => &expr.attrs,
        Expr::RawAddr(expr) => &expr.attrs,
        Expr::Reference(expr) => &expr.attrs,
        Expr::Repeat(expr) => &expr.attrs,
        Expr::Return(expr) => &expr.attrs,
        Expr::Struct(expr) => &expr.attrs,
        Expr::Try(expr) => &expr.attrs,
        Expr::TryBlock(expr) => &expr.attrs,
        Expr::Tuple(expr) => &expr.attrs,
        Expr::Unary(expr) => &expr.attrs,
        Expr::Unsafe(expr) => &expr.attrs,
        Expr::While(expr) => &expr.attrs,
        Expr::Yield(expr) => &expr.attrs,
        Expr::Verbatim(_) => &[],
        _ => &[],
    }
}

fn has_test_attribute(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        if attribute.path().is_ident("test") {
            return true;
        }
        let syn::Meta::List(list) = &attribute.meta else {
            return false;
        };
        if !attribute.path().is_ident("cfg") {
            return false;
        }
        matches!(
            cfg_tokens_classification(&list.tokens),
            CfgClassification::AlwaysFalse | CfgClassification::RequiresTest
        )
    })
}

fn has_cfg_attribute(attributes: &[syn::Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg"))
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CfgClassification {
    AlwaysFalse,
    AlwaysTrue,
    RequiresTest,
    Unknown,
}

fn cfg_tokens_classification(tokens: &TokenStream) -> CfgClassification {
    let parser = Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let Ok(predicates) = parser.parse2(tokens.clone()) else {
        return CfgClassification::Unknown;
    };
    if predicates.len() != 1 {
        return CfgClassification::Unknown;
    }
    cfg_meta_classification(&predicates[0])
}

fn cfg_meta_classification(meta: &syn::Meta) -> CfgClassification {
    match meta {
        syn::Meta::Path(path) if path.is_ident("test") => CfgClassification::RequiresTest,
        syn::Meta::Path(_) | syn::Meta::NameValue(_) => CfgClassification::Unknown,
        syn::Meta::List(list) if list.path.is_ident("all") => {
            let children = cfg_meta_children(list);
            if children.contains(&CfgClassification::AlwaysFalse) {
                CfgClassification::AlwaysFalse
            } else if children.contains(&CfgClassification::RequiresTest) {
                CfgClassification::RequiresTest
            } else if children
                .iter()
                .all(|child| *child == CfgClassification::AlwaysTrue)
            {
                CfgClassification::AlwaysTrue
            } else {
                CfgClassification::Unknown
            }
        }
        syn::Meta::List(list) if list.path.is_ident("any") => {
            let children = cfg_meta_children(list);
            if children.contains(&CfgClassification::AlwaysTrue) {
                CfgClassification::AlwaysTrue
            } else {
                let live = children
                    .iter()
                    .copied()
                    .filter(|child| *child != CfgClassification::AlwaysFalse)
                    .collect::<Vec<_>>();
                if live.is_empty() {
                    CfgClassification::AlwaysFalse
                } else if live
                    .iter()
                    .all(|child| *child == CfgClassification::RequiresTest)
                {
                    CfgClassification::RequiresTest
                } else {
                    CfgClassification::Unknown
                }
            }
        }
        syn::Meta::List(list) if list.path.is_ident("not") => {
            match cfg_meta_children(list).as_slice() {
                [CfgClassification::AlwaysFalse] => CfgClassification::AlwaysTrue,
                [CfgClassification::AlwaysTrue] => CfgClassification::AlwaysFalse,
                [_] | [] | [_, _, ..] => CfgClassification::Unknown,
            }
        }
        syn::Meta::List(_) => CfgClassification::Unknown,
    }
}

fn cfg_meta_children(list: &syn::MetaList) -> Vec<CfgClassification> {
    let parser = Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    parser
        .parse2(list.tokens.clone())
        .map(|children| {
            children
                .iter()
                .map(cfg_meta_classification)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|_| vec![CfgClassification::Unknown])
}

fn item_has_test_attribute(item: &Item) -> bool {
    let attributes = match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        Item::Verbatim(_) => return false,
        _ => return false,
    };
    has_test_attribute(attributes)
}

fn impl_item_has_test_attribute(item: &syn::ImplItem) -> bool {
    let attributes = match item {
        syn::ImplItem::Const(item) => &item.attrs,
        syn::ImplItem::Fn(item) => &item.attrs,
        syn::ImplItem::Type(item) => &item.attrs,
        syn::ImplItem::Macro(item) => &item.attrs,
        syn::ImplItem::Verbatim(_) => return false,
        _ => return false,
    };
    has_test_attribute(attributes)
}

fn trait_item_has_test_attribute(item: &syn::TraitItem) -> bool {
    let attributes = match item {
        syn::TraitItem::Const(item) => &item.attrs,
        syn::TraitItem::Fn(item) => &item.attrs,
        syn::TraitItem::Type(item) => &item.attrs,
        syn::TraitItem::Macro(item) => &item.attrs,
        syn::TraitItem::Verbatim(_) => return false,
        _ => return false,
    };
    has_test_attribute(attributes)
}

fn parse_source(input: &SourceInput) -> Result<File, String> {
    syn::parse_file(&input.source).map_err(|error| format!("{}: {error}", input.path))
}

fn scan_input(input: &SourceInput) -> Result<Vec<ProducerOutput>, String> {
    let syntax = parse_source(input)?;
    let mut visitor = ProducerVisitor::new(&input.path);
    visitor.visit_file(&syntax);
    let mut sites = visitor.finish()?;
    sites.sort_by_key(|site| SpanKey::from(site.span));
    let mut occurrences = BTreeMap::<String, usize>::new();
    let mut outputs = Vec::new();
    for site in sites {
        let occurrence = occurrences.entry(site.symbol.clone()).or_default();
        *occurrence += 1;
        let start = site.span.start();
        outputs.push(ProducerOutput {
            path: input.path.clone(),
            symbol: site.symbol,
            occurrence: *occurrence,
            scope_proximity_source: site.scope_proximity_source,
            line: start.line,
            column: start.column,
        });
    }
    Ok(outputs)
}

fn production_reachable_source_paths(
    inputs: &[SourceInput],
    product_roots: &[String],
) -> Result<BTreeSet<String>, String> {
    let syntax = inputs
        .iter()
        .map(|input| parse_source(input).map(|file| (input.path.clone(), file)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let available = syntax.keys().cloned().collect::<BTreeSet<_>>();
    let mut reachable = BTreeSet::new();
    for root in product_roots {
        if !available.contains(root) {
            return Err(format!(
                "Cargo product target root is missing from the scan input: {root}"
            ));
        }
        let module_dir = Path::new(root)
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| format!("product crate root has no parent: {root}"))?;
        walk_reachable_module_file(root, &module_dir, &syntax, &available, &mut reachable)?;
    }
    Ok(reachable)
}

fn walk_reachable_module_file(
    path: &str,
    module_dir: &Path,
    syntax: &BTreeMap<String, File>,
    available: &BTreeSet<String>,
    reachable: &mut BTreeSet<String>,
) -> Result<(), String> {
    if !reachable.insert(path.to_string()) {
        return Ok(());
    }
    let file = syntax
        .get(path)
        .ok_or_else(|| format!("reachable Rust module is missing from the scan input: {path}"))?;
    let path_attribute_base = Path::new(path)
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("reachable Rust module has no parent: {path}"))?;
    walk_reachable_module_items(
        path,
        &file.items,
        module_dir,
        &path_attribute_base,
        syntax,
        available,
        reachable,
    )
}

fn walk_reachable_module_items(
    current_path: &str,
    items: &[Item],
    module_dir: &Path,
    path_attribute_base: &Path,
    syntax: &BTreeMap<String, File>,
    available: &BTreeSet<String>,
    reachable: &mut BTreeSet<String>,
) -> Result<(), String> {
    for item in items {
        if item_has_test_attribute(item) {
            continue;
        }
        let Item::Mod(module) = item else {
            reject_nested_outlined_modules(current_path, item)?;
            continue;
        };
        if let Some(attribute) = module
            .attrs
            .iter()
            .find(|attribute| cfg_attr_contains_path_attribute(attribute))
        {
            return Err(format!(
                "{current_path}:{}: cfg_attr-selected module paths require an explicit module-graph rule",
                attribute.span().start().line
            ));
        }
        let name = normalized_ident(&module.ident);
        let explicit_path = module_path_attribute(&module.attrs);
        let child_dir = module_dir.join(&name);
        if let Some((_, inline_items)) = &module.content {
            let inline_child_dir = if let Some(explicit) = &explicit_path {
                PathBuf::from(normalize_module_candidate(
                    current_path,
                    explicit.line,
                    &path_attribute_base.join(&explicit.path),
                )?)
            } else {
                child_dir
            };
            walk_reachable_module_items(
                current_path,
                inline_items,
                &inline_child_dir,
                &inline_child_dir,
                syntax,
                available,
                reachable,
            )?;
            continue;
        }
        let candidates = if let Some(explicit) = explicit_path {
            vec![normalize_module_candidate(
                current_path,
                explicit.line,
                &path_attribute_base.join(explicit.path),
            )?]
        } else {
            let line = module.span().start().line;
            vec![
                normalize_module_candidate(
                    current_path,
                    line,
                    &module_dir.join(format!("{name}.rs")),
                )?,
                normalize_module_candidate(current_path, line, &child_dir.join("mod.rs"))?,
            ]
        };
        let present = candidates
            .into_iter()
            .filter(|candidate| available.contains(candidate))
            .collect::<Vec<_>>();
        match present.as_slice() {
            [] => {
                return Err(format!(
                    "{current_path}: external module {name} has no tracked Rust source"
                ));
            }
            [child] => walk_reachable_module_file(child, &child_dir, syntax, available, reachable)?,
            _ => {
                return Err(format!(
                    "{current_path}: external module {name} resolves to multiple tracked sources"
                ));
            }
        }
    }
    Ok(())
}

fn reject_nested_outlined_modules(current_path: &str, item: &Item) -> Result<(), String> {
    struct Detector<'a> {
        current_path: &'a str,
        error: Option<String>,
    }

    impl<'ast> Visit<'ast> for Detector<'_> {
        fn visit_expr(&mut self, expression: &'ast Expr) {
            if has_test_attribute(expression_attributes(expression)) {
                return;
            }
            visit::visit_expr(self, expression);
        }

        fn visit_arm(&mut self, arm: &'ast syn::Arm) {
            if has_test_attribute(&arm.attrs) {
                return;
            }
            visit::visit_arm(self, arm);
        }

        fn visit_field_value(&mut self, field: &'ast syn::FieldValue) {
            if has_test_attribute(&field.attrs) {
                return;
            }
            visit::visit_field_value(self, field);
        }

        fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
            if impl_item_has_test_attribute(item) {
                return;
            }
            visit::visit_impl_item(self, item);
        }

        fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
            if trait_item_has_test_attribute(item) {
                return;
            }
            visit::visit_trait_item(self, item);
        }

        fn visit_stmt(&mut self, statement: &'ast Stmt) {
            let excluded = match statement {
                Stmt::Local(local) => has_test_attribute(&local.attrs),
                Stmt::Expr(expression, _) => has_test_attribute(expression_attributes(expression)),
                Stmt::Macro(mac) => has_test_attribute(&mac.attrs),
                Stmt::Item(item) => item_has_test_attribute(item),
            };
            if excluded {
                return;
            }
            visit::visit_stmt(self, statement);
        }

        fn visit_item(&mut self, item: &'ast Item) {
            if self.error.is_some() || item_has_test_attribute(item) {
                return;
            }
            if let Item::Mod(module) = item
                && module.content.is_none()
            {
                self.error = Some(format!(
                    "{}:{}: block-local outlined module requires an explicit module-graph rule",
                    self.current_path,
                    module.span().start().line
                ));
                return;
            }
            visit::visit_item(self, item);
        }
    }

    let mut detector = Detector {
        current_path,
        error: None,
    };
    detector.visit_item(item);
    detector.error.map_or(Ok(()), Err)
}

struct ModulePathAttribute {
    path: String,
    line: usize,
}

fn module_path_attribute(attributes: &[syn::Attribute]) -> Option<ModulePathAttribute> {
    attributes.iter().find_map(|attribute| {
        if !attribute.path().is_ident("path") {
            return None;
        }
        let syn::Meta::NameValue(value) = &attribute.meta else {
            return None;
        };
        let Expr::Lit(literal) = &value.value else {
            return None;
        };
        let syn::Lit::Str(path) = &literal.lit else {
            return None;
        };
        Some(ModulePathAttribute {
            path: path.value(),
            line: attribute.span().start().line,
        })
    })
}

fn cfg_attr_contains_path_attribute(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("cfg_attr") {
        return false;
    }
    let syn::Meta::List(list) = &attribute.meta else {
        return false;
    };
    cfg_attr_tokens_contain_path_attribute(&list.tokens)
}

fn cfg_attr_tokens_contain_path_attribute(tokens: &TokenStream) -> bool {
    let parser = Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated;
    let Ok(arguments) = parser.parse2(tokens.clone()) else {
        return false;
    };
    arguments
        .iter()
        .skip(1)
        .any(meta_contains_module_path_attribute)
}

fn meta_contains_module_path_attribute(meta: &syn::Meta) -> bool {
    match meta {
        syn::Meta::NameValue(value) => value.path.is_ident("path"),
        syn::Meta::List(list) if list.path.is_ident("cfg_attr") => {
            cfg_attr_tokens_contain_path_attribute(&list.tokens)
        }
        syn::Meta::Path(_) | syn::Meta::List(_) => false,
    }
}

fn normalize_module_candidate(
    current_path: &str,
    line: usize,
    path: &Path,
) -> Result<String, String> {
    normalize_repository_path(path).map_err(|reason| format!("{current_path}:{line}: {reason}"))
}

fn normalize_repository_path(path: &Path) -> Result<String, &'static str> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err("module path escapes the repository root");
                }
            }
            Component::Normal(segment) => normalized.push(segment),
            Component::Prefix(_) => {
                return Err("module path uses a platform prefix outside the repository");
            }
            Component::RootDir => return Err("absolute module paths leave the repository"),
        }
    }
    Ok(normalized.to_string_lossy().replace('\\', "/"))
}

fn validate_scanner_contract() -> Result<(), String> {
    let positive = SourceInput {
        path: "rust/crates/scanner-contract/src/lib.rs".to_string(),
        source: [
            "use omena_cascade::CascadeKey;",
            "#[cfg(not(test))] unsafe fn direct(scope: u32) {",
            "    CascadeKey /* unlimited trivia is AST-inert */ ::new(a, b, scope, c, d);",
            "}",
            "fn shorthand(scope_proximity: u32) {",
            "    let _ = CascadeKey { level, layer_rank, scope_proximity, specificity, source_order };",
            "}",
            "fn pattern(key: CascadeKey) {",
            "    let CascadeKey { scope_proximity: extracted, .. } = key;",
            "    let _ = extracted;",
            "}",
            "#[cfg(test)] fn excluded() { CascadeKey::new(a, b, 0, c, d); }",
            "#[cfg(all(test, unix))] fn excluded_conjunction() { CascadeKey::new(a, b, 0, c, d); }",
            "#[cfg(any())] fn excluded_dead_code() { CascadeKey::new(a, b, 0, c, d); }",
        ]
        .join("\n"),
    };
    let rows = scan_input(&positive)?;
    if rows.len() != 2
        || rows
            .iter()
            .any(|row| row.scope_proximity_source != ScopeProximitySource::CallerSupplied)
    {
        return Err(format!(
            "CascadeKey scanner positive matrix drifted: rows={}, sources={:?}",
            rows.len(),
            rows.iter()
                .map(|row| row.scope_proximity_source)
                .collect::<Vec<_>>()
        ));
    }

    let conditional_fallback = SourceInput {
        path: "rust/crates/scanner-contract/src/conditional-fallback.rs".to_string(),
        source: [
            "use omena_cascade::CascadeKey;",
            "fn generated() {",
            "    let scope = if false { cascade_scope_proximity_fallback_for_selector_context_rank(0) } else { 0 };",
            "    CascadeKey::new(a, b, scope, c, d);",
            "}",
        ]
        .join("\n"),
    };
    let conditional_rows = scan_input(&conditional_fallback)?;
    if conditional_rows.len() != 1
        || conditional_rows[0].scope_proximity_source != ScopeProximitySource::GeneratedValue
    {
        return Err(
            "conditional fallback must not be classified as the declared driver".to_string(),
        );
    }

    let rejected = [
        (
            "alias",
            "use omena_cascade::CascadeKey as Key; fn f(){Key::new(a,b,0,c,d);}",
        ),
        (
            "generic-shadow",
            "use omena_cascade::CascadeKey; fn f<CascadeKey>(){CascadeKey::new(a,b,0,c,d);}",
        ),
        (
            "function-item",
            "use omena_cascade::CascadeKey; fn f(){let constructor=CascadeKey::new;constructor(a,b,0,c,d);}",
        ),
        (
            "ufcs",
            "use omena_cascade::CascadeKey; fn f(){<CascadeKey>::new(a,b,0,c,d);}",
        ),
        (
            "turbofish",
            "use omena_cascade::CascadeKey; fn f(){CascadeKey::<>::new(a,b,0,c,d);}",
        ),
        (
            "mutable-driver",
            "use omena_cascade::CascadeKey; fn f(){let mut driver=fallback();driver=other();CascadeKey::new(a,b,driver,c,d);}",
        ),
        (
            "member-driver",
            "use omena_cascade::CascadeKey; fn f(){let generated=GeneratedScope{scope_proximity:1};CascadeKey::new(a,b,generated.scope_proximity,c,d);}",
        ),
        (
            "parameter-pattern-shadow",
            "use omena_cascade::CascadeKey; fn f(scope:u32){if let Some(scope)=maybe{CascadeKey::new(a,b,scope,c,d);}}",
        ),
    ];
    for (name, source) in rejected {
        let input = SourceInput {
            path: format!("rust/crates/scanner-contract/src/{name}.rs"),
            source: source.to_string(),
        };
        if scan_input(&input).is_ok() {
            return Err(format!(
                "CascadeKey scanner negative matrix accepted {name}"
            ));
        }
    }
    let fallback_shadow = SourceInput {
        path: "rust/crates/omena-semantic/src/design_tokens.rs".to_string(),
        source: [
            "use omena_cascade::CascadeKey;",
            "fn cascade_key() {",
            "    fn cascade_scope_proximity_fallback_for_selector_context_rank(_: usize) -> u32 { 7 }",
            "    let scope = cascade_scope_proximity_fallback_for_selector_context_rank(0);",
            "    CascadeKey::new(a, b, scope, c, d);",
            "}",
        ]
        .join("\n"),
    };
    if scan_input(&fallback_shadow).is_ok() {
        return Err("CascadeKey scanner negative matrix accepted fallback-shadow".to_string());
    }
    let module_graph = [
        SourceInput {
            path: "rust/crates/scanner-contract/src/lib.rs".to_string(),
            source: "mod product; #[cfg(test)] mod tests;".to_string(),
        },
        SourceInput {
            path: "rust/crates/scanner-contract/src/product.rs".to_string(),
            source: "use omena_cascade::CascadeKey; fn product(){CascadeKey::new(a,b,0,c,d);}"
                .to_string(),
        },
        SourceInput {
            path: "rust/crates/scanner-contract/src/tests.rs".to_string(),
            source: "use omena_cascade::CascadeKey; fn test_only(){CascadeKey::new(a,b,1,c,d);}"
                .to_string(),
        },
        SourceInput {
            path: "rust/crates/scanner-contract/src/orphan.rs".to_string(),
            source: "use omena_cascade::CascadeKey; fn orphan(){CascadeKey::new(a,b,2,c,d);}"
                .to_string(),
        },
    ];
    let reachable = production_reachable_source_paths(
        &module_graph,
        &["rust/crates/scanner-contract/src/lib.rs".to_string()],
    )?;
    let expected = BTreeSet::from([
        "rust/crates/scanner-contract/src/lib.rs".to_string(),
        "rust/crates/scanner-contract/src/product.rs".to_string(),
    ]);
    if reachable != expected {
        return Err(format!(
            "CascadeKey product module graph drifted: {reachable:?}"
        ));
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    validate_scanner_contract().map_err(io::Error::other)?;
    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin)?;
    let input: ScannerInput = serde_json::from_str(&stdin)?;
    let reachable = production_reachable_source_paths(&input.sources, &input.product_roots)
        .map_err(io::Error::other)?;
    let mut outputs = Vec::new();

    for source in input
        .sources
        .iter()
        .filter(|source| reachable.contains(&source.path))
    {
        outputs.extend(scan_input(source).map_err(io::Error::other)?);
    }

    serde_json::to_writer(io::stdout().lock(), &outputs)?;
    Ok(())
}

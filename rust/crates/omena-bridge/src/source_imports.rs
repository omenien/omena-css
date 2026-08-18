use omena_parser::ParserByteSpanV0;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_parser::{Parser, ParserReturn};
use serde::{Deserialize, Serialize};

use crate::source_language::{
    project_source_for_language, recover_panicked_editor_source, source_type_for_language,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImportDeclarationSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub import_count: usize,
    pub imports: Vec<SourceImportDeclarationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceImportDeclarationV0 {
    pub binding: String,
    pub specifier: String,
    pub specifier_byte_span: ParserByteSpanV0,
    pub declaration_id: String,
}

impl SourceImportDeclarationV0 {
    pub fn style_resolution(&self, style_uri: &str) -> SourceStyleImportResolutionV0 {
        SourceStyleImportResolutionV0 {
            declaration_id: self.declaration_id.clone(),
            style_uri: style_uri.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStyleImportResolutionV0 {
    pub declaration_id: String,
    pub style_uri: String,
}

pub fn summarize_omena_bridge_source_import_declarations(
    source: &str,
) -> SourceImportDeclarationSummaryV0 {
    summarize_omena_bridge_source_import_declarations_for_path("source.tsx", source)
}

pub fn summarize_omena_bridge_source_import_declarations_for_path(
    source_path: &str,
    source: &str,
) -> SourceImportDeclarationSummaryV0 {
    summarize_omena_bridge_source_import_declarations_for_source_language(source_path, source, None)
}

pub fn summarize_omena_bridge_source_import_declarations_for_source_language(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
) -> SourceImportDeclarationSummaryV0 {
    let projected_source = project_source_for_language(source_path, source, source_language);
    let source_type = source_type_for_language(source_path, source_language);
    let allocator = Allocator::default();
    let ParserReturn {
        program,
        panicked,
        diagnostics,
        ..
    } = Parser::new(&allocator, projected_source.as_ref(), source_type).parse();

    let mut imports = Vec::new();
    if !panicked {
        push_import_declarations_from_program(source, &program.body, source.len(), &mut imports);
    } else if let Some(recovered) = recover_panicked_editor_source(
        projected_source.as_ref(),
        source_type,
        diagnostics
            .iter()
            .flat_map(|diagnostic| diagnostic.labels.as_slice())
            .map(|label| label.offset() as usize)
            .collect::<Vec<_>>()
            .as_slice(),
    ) {
        let recovered_allocator = Allocator::default();
        let recovered_parse =
            Parser::new(&recovered_allocator, recovered.source.as_str(), source_type).parse();
        push_import_declarations_from_program(
            source,
            &recovered_parse.program.body,
            recovered.trusted_byte_end,
            &mut imports,
        );
    }
    canonicalize_import_declarations(&mut imports);

    SourceImportDeclarationSummaryV0 {
        schema_version: "0",
        product: "omena-bridge.source-import-declarations",
        import_count: imports.len(),
        imports,
    }
}

fn push_import_declarations_from_program(
    source: &str,
    statements: &[Statement<'_>],
    trusted_byte_end: usize,
    imports: &mut Vec<SourceImportDeclarationV0>,
) {
    for statement in statements {
        if let Statement::ImportDeclaration(import) = statement
            && import.span.end as usize <= trusted_byte_end
        {
            push_import_declarations_from_ast(source, import, imports);
        }
    }
}

fn push_import_declarations_from_ast(
    source: &str,
    import: &ImportDeclaration<'_>,
    imports: &mut Vec<SourceImportDeclarationV0>,
) {
    if import.import_kind != ImportOrExportKind::Value {
        return;
    }
    let Some(specifiers) = import.specifiers.as_ref() else {
        return;
    };
    let specifier = import.source.value.as_str();
    let literal_start = import.source.span.start as usize;
    let literal_end = import.source.span.end as usize;
    let specifier_byte_span = source
        .get(literal_start..literal_end)
        .filter(|literal| {
            literal.len() >= 2
                && matches!(literal.as_bytes().first(), Some(b'\'' | b'"'))
                && literal.as_bytes().first() == literal.as_bytes().last()
        })
        .map_or(
            ParserByteSpanV0 {
                start: literal_start,
                end: literal_end,
            },
            |_| ParserByteSpanV0 {
                start: literal_start + 1,
                end: literal_end - 1,
            },
        );

    for specifier_item in specifiers {
        match specifier_item {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default_specifier) => {
                let binding = default_specifier.local.name.as_str();
                imports.push(SourceImportDeclarationV0 {
                    declaration_id: source_declaration_id(
                        source,
                        "import",
                        binding,
                        default_specifier.local.span.start as usize,
                        default_specifier.local.span.end as usize,
                        specifier,
                    ),
                    binding: binding.to_string(),
                    specifier: specifier.to_string(),
                    specifier_byte_span,
                });
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_specifier) => {
                let binding = namespace_specifier.local.name.as_str();
                imports.push(SourceImportDeclarationV0 {
                    declaration_id: source_declaration_id(
                        source,
                        "import",
                        binding,
                        namespace_specifier.local.span.start as usize,
                        namespace_specifier.local.span.end as usize,
                        specifier,
                    ),
                    binding: binding.to_string(),
                    specifier: specifier.to_string(),
                    specifier_byte_span,
                });
            }
            ImportDeclarationSpecifier::ImportSpecifier(_) => {}
        }
    }
}

pub(crate) fn source_declaration_id(
    source: &str,
    kind: &str,
    binding: &str,
    byte_start: usize,
    byte_end: usize,
    specifier: &str,
) -> String {
    let utf16_start = source
        .get(..byte_start)
        .map_or(byte_start, |prefix| prefix.encode_utf16().count());
    let utf16_end = source
        .get(..byte_end)
        .map_or(byte_end, |prefix| prefix.encode_utf16().count());
    format!("rust-decl:{kind}:{binding}:{utf16_start}:{utf16_end}:{specifier}")
}

fn canonicalize_import_declarations(imports: &mut Vec<SourceImportDeclarationV0>) {
    imports.sort_by(|left, right| {
        left.binding
            .cmp(&right.binding)
            .then_with(|| left.specifier.cmp(&right.specifier))
            .then_with(|| left.declaration_id.cmp(&right.declaration_id))
    });
    imports.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_default_and_namespace_imports_from_oxc_ast() {
        let summary = summarize_omena_bridge_source_import_declarations_for_path(
            "Component.tsx",
            r#"
import bind from "classnames/bind";
import styles from "./Button.module.scss";
import * as tokens from "./tokens.module.css";
import { type BadgeProps } from "./types";
const lazy = import("./ignored.module.scss");
"#,
        );

        assert_eq!(summary.product, "omena-bridge.source-import-declarations");
        assert_eq!(
            summary
                .imports
                .iter()
                .map(|import| (import.binding.as_str(), import.specifier.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("bind", "classnames/bind"),
                ("styles", "./Button.module.scss"),
                ("tokens", "./tokens.module.css"),
            ],
        );
    }

    #[test]
    fn ignores_import_like_strings_and_type_only_default_imports() {
        let summary = summarize_omena_bridge_source_import_declarations_for_path(
            "Component.tsx",
            r#"
const text = "import fake from './Fake.module.scss'";
import type styles from "./Typed.module.scss";
import real from "./Real.module.scss";
"#,
        );

        assert_eq!(
            summary
                .imports
                .iter()
                .map(|import| (import.binding.as_str(), import.specifier.as_str()))
                .collect::<Vec<_>>(),
            vec![("real", "./Real.module.scss")],
        );
    }

    #[test]
    fn declaration_id_reuses_projection_identity_with_utf16_offsets() {
        let source = "const label = '한글';\nimport styles from \"./Card.module.scss\";\n";
        let summary = summarize_omena_bridge_source_import_declarations(source);
        let declaration = &summary.imports[0];
        let byte_start = source.find("styles").unwrap_or(usize::MAX);
        assert_ne!(byte_start, usize::MAX);
        let utf16_start = source[..byte_start].encode_utf16().count();
        assert_eq!(
            declaration.declaration_id,
            format!(
                "rust-decl:import:styles:{utf16_start}:{}:./Card.module.scss",
                utf16_start + "styles".len()
            )
        );
        let specifier_start = source.find("./Card.module.scss").unwrap_or(usize::MAX);
        assert_ne!(specifier_start, usize::MAX);
        assert_eq!(
            declaration.specifier_byte_span,
            ParserByteSpanV0 {
                start: specifier_start,
                end: specifier_start + "./Card.module.scss".len(),
            }
        );
    }

    #[test]
    fn incomplete_editor_buffer_recovers_only_oxc_import_declarations() {
        let source = r#"// import phantom from "./Phantom.module.scss";
import styles from "./Card.module.scss";
const value = styles.
"#;
        let summary = summarize_omena_bridge_source_import_declarations(source);

        assert_eq!(summary.import_count, 1);
        assert_eq!(summary.imports[0].binding, "styles");
        assert_eq!(summary.imports[0].specifier, "./Card.module.scss");
        assert!(!summary.imports[0].declaration_id.contains("phantom"));
    }

    #[test]
    fn extracts_imports_from_vue_sfc_script_projection() {
        let source = r#"<template><button /></template>
<script setup lang="ts">
import styles from "./Card.module.scss";
const local = "not a style import";
</script>
<style module>
.root {}
</style>
"#;
        let summary = summarize_omena_bridge_source_import_declarations_for_source_language(
            "Card.vue",
            source,
            Some("vue"),
        );

        assert_eq!(
            summary
                .imports
                .iter()
                .map(|import| (import.binding.as_str(), import.specifier.as_str()))
                .collect::<Vec<_>>(),
            vec![("styles", "./Card.module.scss")],
        );
    }

    #[test]
    fn extracts_imports_from_html_script_projection() {
        let source = r#"<main>not script</main>
<script type="module">
import styles from "./Page.module.scss";
</script>
"#;
        let summary = summarize_omena_bridge_source_import_declarations_for_source_language(
            "Page.html",
            source,
            Some("html"),
        );

        assert_eq!(
            summary
                .imports
                .iter()
                .map(|import| (import.binding.as_str(), import.specifier.as_str()))
                .collect::<Vec<_>>(),
            vec![("styles", "./Page.module.scss")],
        );
    }
}

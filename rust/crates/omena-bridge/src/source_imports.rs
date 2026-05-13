use omena_parser::ParserByteSpanV0;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind, Statement};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use serde::Serialize;

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
    let allocator = Allocator::default();
    let source_type = match SourceType::from_path(source_path) {
        Ok(source_type) => source_type,
        Err(_) => SourceType::tsx(),
    };
    let ParserReturn {
        program, panicked, ..
    } = Parser::new(&allocator, source, source_type).parse();

    let mut imports = Vec::new();
    if !panicked {
        for statement in &program.body {
            if let Statement::ImportDeclaration(import) = statement {
                push_import_declarations_from_ast(import, &mut imports);
            }
        }
        canonicalize_import_declarations(&mut imports);
    }

    SourceImportDeclarationSummaryV0 {
        schema_version: "0",
        product: "omena-bridge.source-import-declarations",
        import_count: imports.len(),
        imports,
    }
}

fn push_import_declarations_from_ast(
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
    let specifier_byte_span = ParserByteSpanV0 {
        start: import.source.span.start as usize,
        end: import.source.span.end as usize,
    };

    for specifier_item in specifiers {
        match specifier_item {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default_specifier) => {
                imports.push(SourceImportDeclarationV0 {
                    binding: default_specifier.local.name.as_str().to_string(),
                    specifier: specifier.to_string(),
                    specifier_byte_span,
                });
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace_specifier) => {
                imports.push(SourceImportDeclarationV0 {
                    binding: namespace_specifier.local.name.as_str().to_string(),
                    specifier: specifier.to_string(),
                    specifier_byte_span,
                });
            }
            ImportDeclarationSpecifier::ImportSpecifier(_) => {}
        }
    }
}

fn canonicalize_import_declarations(imports: &mut Vec<SourceImportDeclarationV0>) {
    imports.sort_by(|left, right| {
        left.binding
            .cmp(&right.binding)
            .then_with(|| left.specifier.cmp(&right.specifier))
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
}

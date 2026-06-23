use crate::{
    ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2,
};

pub(super) fn backend<'a>(
    summary: &'a crate::SelectedQueryAdapterCapabilitiesV0,
    backend_kind: &str,
) -> Option<&'a crate::SelectedQueryBackendCapabilityV0> {
    summary
        .backend_kinds
        .iter()
        .find(|backend| backend.backend_kind == backend_kind)
}

pub(super) fn sample_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![
                    ClassExpressionInputV2 {
                        id: "expr-1".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: range(4, 12, 4, 16),
                        class_name: None,
                        root_binding_decl_id: Some("decl-1".to_string()),
                        access_path: None,
                    },
                    ClassExpressionInputV2 {
                        id: "expr-2".to_string(),
                        kind: "styleAccess".to_string(),
                        scss_module_path: "/tmp/Card.module.scss".to_string(),
                        range: range(6, 9, 6, 20),
                        class_name: Some("card-header".to_string()),
                        root_binding_decl_id: None,
                        access_path: Some(vec!["card".to_string(), "header".to_string()]),
                    },
                ],
            },
        }],
        styles: vec![
            StyleAnalysisInputV2 {
                file_path: "/tmp/App.module.scss".to_string(),
                source: None,
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "btn-active".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("btn-active".to_string()),
                        range: range(1, 1, 1, 12),
                        nested_safety: Some("safe".to_string()),
                        composes: None,
                        bem_suffix: None,
                    }],
                },
            },
            StyleAnalysisInputV2 {
                file_path: "/tmp/Card.module.scss".to_string(),
                source: None,
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "card-header".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("card-header".to_string()),
                        range: range(3, 1, 3, 13),
                        nested_safety: Some("unsafe".to_string()),
                        composes: None,
                        bem_suffix: None,
                    }],
                },
            },
        ],
        type_facts: vec![
            TypeFactEntryV2 {
                file_path: "/tmp/App.tsx".to_string(),
                expression_id: "expr-1".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "constrained".to_string(),
                    constraint_kind: Some("prefixSuffix".to_string()),
                    values: None,
                    prefix: Some("btn-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_len: Some(10),
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: None,
                },
            },
            TypeFactEntryV2 {
                file_path: "/tmp/Card.tsx".to_string(),
                expression_id: "expr-2".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "finiteSet".to_string(),
                    constraint_kind: None,
                    values: Some(vec!["card-header".to_string(), "card-body".to_string()]),
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: None,
                },
            },
        ],
    }
}

pub(super) fn reduced_product_iteration_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: vec![TypeFactEntryV2 {
            file_path: "/tmp/App.tsx".to_string(),
            expression_id: "expr-reduced".to_string(),
            facts: StringTypeFactsV2 {
                kind: "constrained".to_string(),
                constraint_kind: Some("composite".to_string()),
                values: None,
                prefix: Some("btn-".to_string()),
                suffix: Some("-active".to_string()),
                min_len: None,
                max_len: None,
                char_must: Some("a".to_string()),
                char_may: Some("-abceintv".to_string()),
                may_include_other_chars: Some(false),
                provenance: None,
            },
        }],
    }
}

pub(super) fn reduced_product_projection_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![
                    ClassExpressionInputV2 {
                        id: "expr-primary".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: range(4, 12, 4, 16),
                        class_name: None,
                        root_binding_decl_id: Some("decl-primary".to_string()),
                        access_path: None,
                    },
                    ClassExpressionInputV2 {
                        id: "expr-secondary".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: range(5, 12, 5, 16),
                        class_name: None,
                        root_binding_decl_id: Some("decl-secondary".to_string()),
                        access_path: None,
                    },
                ],
            },
        }],
        styles: vec![StyleAnalysisInputV2 {
            file_path: "/tmp/App.module.scss".to_string(),
            source: None,
            document: StyleDocumentV2 {
                selectors: vec![
                    style_selector("btn--active"),
                    style_selector("btn-primary--active"),
                    style_selector("btn-secondary--active"),
                    style_selector("card-active"),
                ],
            },
        }],
        type_facts: vec![
            TypeFactEntryV2 {
                file_path: "/tmp/App.tsx".to_string(),
                expression_id: "expr-primary".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "constrained".to_string(),
                    constraint_kind: Some("prefixSuffix".to_string()),
                    values: None,
                    prefix: Some("btn-primary-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_len: Some("btn-primary--active".len()),
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: None,
                },
            },
            TypeFactEntryV2 {
                file_path: "/tmp/App.tsx".to_string(),
                expression_id: "expr-secondary".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "constrained".to_string(),
                    constraint_kind: Some("prefixSuffix".to_string()),
                    values: None,
                    prefix: Some("btn-secondary-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_len: Some("btn-secondary--active".len()),
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                    provenance: None,
                },
            },
        ],
    }
}

pub(super) fn style_selector(name: &str) -> StyleSelectorV2 {
    StyleSelectorV2 {
        name: name.to_string(),
        view_kind: "canonical".to_string(),
        canonical_name: Some(name.to_string()),
        range: range(1, 1, 1, 1 + name.len()),
        nested_safety: Some("safe".to_string()),
        composes: None,
        bem_suffix: None,
    }
}

fn range(
    start_line: usize,
    start_character: usize,
    end_line: usize,
    end_character: usize,
) -> RangeV2 {
    RangeV2 {
        start: PositionV2 {
            line: start_line,
            character: start_character,
        },
        end: PositionV2 {
            line: end_line,
            character: end_character,
        },
    }
}

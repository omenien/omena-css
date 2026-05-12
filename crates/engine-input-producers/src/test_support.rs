use serde_json::Value;

use crate::{
    ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2,
};

pub fn sample_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![
                    ClassExpressionInputV2 {
                        id: "expr-1".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: RangeV2 {
                            start: PositionV2 {
                                line: 4,
                                character: 12,
                            },
                            end: PositionV2 {
                                line: 4,
                                character: 16,
                            },
                        },
                        class_name: None,
                        root_binding_decl_id: Some("decl-1".to_string()),
                        access_path: None,
                    },
                    ClassExpressionInputV2 {
                        id: "expr-2".to_string(),
                        kind: "styleAccess".to_string(),
                        scss_module_path: "/tmp/Card.module.scss".to_string(),
                        range: RangeV2 {
                            start: PositionV2 {
                                line: 6,
                                character: 9,
                            },
                            end: PositionV2 {
                                line: 6,
                                character: 20,
                            },
                        },
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
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "btn-active".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("btn-active".to_string()),
                        range: RangeV2 {
                            start: PositionV2 {
                                line: 1,
                                character: 1,
                            },
                            end: PositionV2 {
                                line: 1,
                                character: 12,
                            },
                        },
                        nested_safety: Some("safe".to_string()),
                        composes: Some(vec![Value::Null]),
                        bem_suffix: None,
                    }],
                },
            },
            StyleAnalysisInputV2 {
                file_path: "/tmp/Card.module.scss".to_string(),
                document: StyleDocumentV2 {
                    selectors: vec![
                        StyleSelectorV2 {
                            name: "card-header".to_string(),
                            view_kind: "canonical".to_string(),
                            canonical_name: Some("card-header".to_string()),
                            range: RangeV2 {
                                start: PositionV2 {
                                    line: 3,
                                    character: 1,
                                },
                                end: PositionV2 {
                                    line: 3,
                                    character: 13,
                                },
                            },
                            nested_safety: Some("unsafe".to_string()),
                            composes: None,
                            bem_suffix: None,
                        },
                        StyleSelectorV2 {
                            name: "card-header:hover".to_string(),
                            view_kind: "nested".to_string(),
                            canonical_name: Some("card-header".to_string()),
                            range: RangeV2 {
                                start: PositionV2 {
                                    line: 5,
                                    character: 3,
                                },
                                end: PositionV2 {
                                    line: 5,
                                    character: 20,
                                },
                            },
                            nested_safety: Some("unknown".to_string()),
                            composes: Some(vec![Value::Null, Value::Null]),
                            bem_suffix: None,
                        },
                    ],
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
                },
            },
        ],
    }
}

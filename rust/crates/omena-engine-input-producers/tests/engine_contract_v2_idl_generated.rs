use std::fs;
use std::path::{Path, PathBuf};

use engine_input_producers::engine_contract_v2_idl_generated::{
    EngineInputV2Json, EngineOutputV2Json, OmenaQueryCodeActionPlanV0Json,
};
use serde::Serialize;
use serde_json::Value;

#[test]
fn engine_contract_v2_idl_fixtures_round_trip_canonically() {
    let mut fixture_paths = fs::read_dir(contract_parity_v2_fixture_dir())
        .expect("contract parity v2 fixture dir should be readable")
        .map(|entry| entry.expect("fixture entry should be readable").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    fixture_paths.sort();

    let mut query_kinds = Vec::<String>::new();
    for fixture_path in fixture_paths {
        let fixture = read_json(&fixture_path);
        let input_value = fixture
            .get("input")
            .expect("fixture must carry input")
            .clone();
        let output_value = fixture
            .get("output")
            .expect("fixture must carry output")
            .clone();

        let input: EngineInputV2Json =
            serde_json::from_value(input_value.clone()).expect("generated input contract parses");
        let output: EngineOutputV2Json =
            serde_json::from_value(output_value.clone()).expect("generated output contract parses");

        assert_eq!(
            canonical_json(&input_value),
            canonical_json(&serde_json::to_value(input).expect("generated input serializes")),
            "{} input canonical round-trip drifted",
            fixture_path.display()
        );
        assert_eq!(
            canonical_json(&output_value),
            canonical_json(&serde_json::to_value(&output).expect("generated output serializes")),
            "{} output canonical round-trip drifted",
            fixture_path.display()
        );

        for result in output.query_results {
            query_kinds.push(
                result_kind(&serde_json::to_value(result).expect("query result serializes"))
                    .to_string(),
            );
        }
    }

    query_kinds.sort();
    query_kinds.dedup();
    assert_eq!(
        query_kinds,
        vec![
            "expression-semantics".to_string(),
            "selector-usage".to_string(),
            "source-expression-resolution".to_string(),
        ]
    );
}

#[test]
fn engine_contract_v2_idl_code_action_plan_round_trips_canonically() {
    let value = serde_json::json!({
        "schemaVersion": "0",
        "product": "omena-query.code-actions",
        "fileUri": "file:///repo/src/App.module.scss",
        "fileKind": "style",
        "actionCount": 1,
        "actions": [
            {
                "title": "Extract CSS custom property",
                "kind": "refactor.extract",
                "edits": [
                    {
                        "uri": "file:///repo/src/App.module.scss",
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": 0, "character": 5 }
                        },
                        "newText": "var(--token)"
                    }
                ],
                "source": "omenaQueryStyleExtractCodeActions"
            }
        ],
        "readySurfaces": ["productFacingCodeActions"]
    });

    let plan: OmenaQueryCodeActionPlanV0Json =
        serde_json::from_value(value.clone()).expect("generated code-action plan contract parses");

    assert_eq!(
        canonical_json(&value),
        canonical_json(&serde_json::to_value(plan).expect("generated code-action plan serializes")),
    );
}

fn contract_parity_v2_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("test/_fixtures/contract-parity-v2")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("fixture should be readable"))
        .expect("fixture should be valid JSON")
}

fn canonical_json<T: Serialize>(value: &T) -> String {
    let value = serde_json::to_value(value).expect("value should convert to JSON");
    serde_json::to_string(&sort_json(value)).expect("value should serialize canonically")
}

fn sort_json(value: Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.into_iter().map(sort_json).collect()),
        Value::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, sort_json(value)))
                    .collect(),
            )
        }
        other => other,
    }
}

fn result_kind(value: &Value) -> &str {
    value
        .get("kind")
        .and_then(Value::as_str)
        .expect("query result must carry kind")
}

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use engine_input_producers::engine_contract_v2_idl_generated::{
    EngineInputV2Json, EngineOutputV2Json, OmenaQueryCodeActionPlanV0Json,
};
use serde::Serialize;
use serde_json::Value;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn engine_contract_v2_idl_fixtures_round_trip_canonically() -> TestResult {
    let mut fixture_paths = fs::read_dir(contract_parity_v2_fixture_dir())?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    fixture_paths.sort();

    let mut query_kinds = Vec::<String>::new();
    for fixture_path in fixture_paths {
        let fixture = read_json(&fixture_path)?;
        let input_value = fixture
            .get("input")
            .ok_or_else(|| missing_fixture_field(&fixture_path, "input"))?
            .clone();
        let output_value = fixture
            .get("output")
            .ok_or_else(|| missing_fixture_field(&fixture_path, "output"))?
            .clone();

        let input: EngineInputV2Json = serde_json::from_value(input_value.clone())?;
        let output: EngineOutputV2Json = serde_json::from_value(output_value.clone())?;

        assert_eq!(
            canonical_json(&input_value)?,
            canonical_json(&serde_json::to_value(input)?)?,
            "{} input canonical round-trip drifted",
            fixture_path.display()
        );
        assert_eq!(
            canonical_json(&output_value)?,
            canonical_json(&serde_json::to_value(&output)?)?,
            "{} output canonical round-trip drifted",
            fixture_path.display()
        );

        for result in output.query_results {
            query_kinds.push(result_kind(&serde_json::to_value(result)?)?.to_string());
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
    Ok(())
}

#[test]
fn engine_contract_v2_idl_code_action_plan_round_trips_canonically() -> TestResult {
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

    let plan: OmenaQueryCodeActionPlanV0Json = serde_json::from_value(value.clone())?;

    assert_eq!(
        canonical_json(&value)?,
        canonical_json(&serde_json::to_value(plan)?)?,
    );
    Ok(())
}

fn contract_parity_v2_fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("test/_fixtures/contract-parity-v2")
}

fn read_json(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn canonical_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let value = serde_json::to_value(value)?;
    serde_json::to_string(&sort_json(value))
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

fn result_kind(value: &Value) -> io::Result<&str> {
    value
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "query result must carry kind"))
}

fn missing_fixture_field(path: &Path, field: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{} fixture must carry {field}", path.display()),
    )
}

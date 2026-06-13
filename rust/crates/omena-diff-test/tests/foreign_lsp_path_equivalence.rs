use omena_lsp_server::{
    LspDocumentOrigin, LspShellState, handle_lsp_message,
    test_support::{file_uri_equivalent, path_to_file_uri},
};
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn foreign_sass_lsp_paths_are_equivalent_across_index_and_disk() -> TestResult {
    let root = unique_temp_root("omena-diff-foreign-lsp-path-equivalence")?;
    let source = root.join("src/App.module.scss");
    let package_root = root.join("node_modules/@acme/tokens");
    let index_style = package_root.join("_index.scss");
    let primitives_style = package_root.join("_primitives.scss");
    fs::create_dir_all(parent(source.as_path())?)?;
    fs::create_dir_all(package_root.as_path())?;
    fs::write(
        package_root.join("package.json"),
        r#"{"name":"@acme/tokens","version":"1.2.3","sass":"./_index.scss"}"#,
    )?;
    let source_text = r#"@use "@acme/tokens" as t;
.button { border-radius: t.$token-radius-small; }
"#;
    fs::write(source.as_path(), source_text)?;
    fs::write(
        index_style.as_path(),
        r#"@forward "primitives" as token-*;"#,
    )?;
    fs::write(primitives_style.as_path(), "$radius-small: 4px;\n")?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let index_uri = path_to_file_uri(index_style.as_path());
    let primitives_uri = path_to_file_uri(primitives_style.as_path());
    let reference_position = position_for_offset(
        source_text,
        source_text
            .find("$token-radius-small")
            .ok_or("missing source Sass variable reference")?
            + 1,
    );

    let mut state = LspShellState::default();
    initialize_workspace(&mut state, workspace_uri.as_str());
    open_style_document(&mut state, source_uri.as_str(), source_text);

    assert_eq!(
        state
            .document(index_uri.as_str())
            .map(|document| &document.origin),
        Some(&LspDocumentOrigin::Foreign),
        "package entrypoint must be admitted as a read-only foreign document"
    );
    assert_eq!(
        state
            .document(primitives_uri.as_str())
            .map(|document| &document.origin),
        Some(&LspDocumentOrigin::Foreign),
        "forwarded package primitive must be admitted as a read-only foreign document"
    );

    let warm_definition = definition_at(&mut state, source_uri.as_str(), &reference_position, 2)?;
    let warm_references = references_at(&mut state, source_uri.as_str(), &reference_position, 3)?;
    assert_single_definition_target(&warm_definition, primitives_uri.as_str())?;
    assert_references_include_uri(&warm_references, source_uri.as_str(), "local consumer")?;
    assert_references_include_uri(
        &warm_references,
        primitives_uri.as_str(),
        "foreign declaration",
    )?;

    let warm_definition_json = serde_json::to_string(&warm_definition)?;
    let warm_references_json = serde_json::to_string(&warm_references)?;
    state.evict_document_for_test(index_uri.as_str());
    state.evict_document_for_test(primitives_uri.as_str());
    state.clear_workspace_occurrence_index_memo_for_test();
    assert!(
        state.document(index_uri.as_str()).is_none()
            && state.document(primitives_uri.as_str()).is_none(),
        "evicted foreign documents force the disk-read path"
    );

    let cold_definition = definition_at(&mut state, source_uri.as_str(), &reference_position, 30)?;
    let cold_references = references_at(&mut state, source_uri.as_str(), &reference_position, 31)?;
    assert_eq!(
        warm_definition_json,
        serde_json::to_string(&cold_definition)?,
        "source-available foreign definitions must be byte-identical across indexed and disk-read paths"
    );
    assert_eq!(
        warm_references_json,
        serde_json::to_string(&cold_references)?,
        "source-available foreign references must be byte-identical across indexed and disk-read paths"
    );

    fs::remove_dir_all(root.as_path())?;
    Ok(())
}

#[test]
fn source_absent_sif_lsp_path_keeps_no_location_contract() -> TestResult {
    let root = unique_temp_root("omena-diff-source-absent-sif-lsp")?;
    let source = root.join("src/App.module.scss");
    let sif_path = root.join("sif/tokens.sif.json");
    fs::create_dir_all(parent(source.as_path())?)?;
    fs::create_dir_all(parent(sif_path.as_path())?)?;

    let source_text = r#"@use "https://cdn.example/tokens.scss" as tokens;
.button {
  color: tokens.$brand;
  border-color: tokens.$brand;
}"#;
    fs::write(source.as_path(), source_text)?;
    let sif = fixture_external_sif("https://cdn.example/tokens.scss")?;
    fs::write(
        sif_path.as_path(),
        omena_sif::write_omena_sif_json_v1(&sif)?,
    )?;
    let lock = omena_sif::OmenaLockV1::new(vec![omena_sif::build_omena_lock_sif_entry_v1(
        "sif/tokens.sif.json",
        &sif,
    )?]);
    fs::write(
        root.join("omena.lock"),
        omena_sif::write_omena_lock_json_v1(&lock)?,
    )?;

    let workspace_uri = path_to_file_uri(root.as_path());
    let source_uri = path_to_file_uri(source.as_path());
    let reference_position = position_for_offset(
        source_text,
        source_text
            .find("$brand")
            .ok_or("missing SIF-backed Sass variable reference")?
            + 1,
    );

    let mut state = LspShellState::default();
    initialize_workspace(&mut state, workspace_uri.as_str());
    open_style_document(&mut state, source_uri.as_str(), source_text);

    let definition = definition_at(&mut state, source_uri.as_str(), &reference_position, 2)?;
    assert!(
        definition.is_null(),
        "fingerprint-only SIF symbols must not fabricate definition locations: {definition:?}"
    );

    let references = references_at(&mut state, source_uri.as_str(), &reference_position, 3)?;
    let locations = references
        .as_array()
        .ok_or("SIF-backed references should return an array")?;
    assert_eq!(
        locations.len(),
        2,
        "SIF-backed references should join local uses without adding a fake declaration"
    );
    assert!(
        locations.iter().all(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, source_uri.as_str()))),
        "SIF-backed references must stay on local source locations: {references:?}"
    );

    let definition_json = serde_json::to_string(&definition)?;
    let references_json = serde_json::to_string(&references)?;
    state.clear_workspace_occurrence_index_memo_for_test();
    assert_eq!(
        definition_json,
        serde_json::to_string(&definition_at(
            &mut state,
            source_uri.as_str(),
            &reference_position,
            40,
        )?)?,
        "fingerprint-only definitions must stay byte-identical after occurrence memo rebuild"
    );
    assert_eq!(
        references_json,
        serde_json::to_string(&references_at(
            &mut state,
            source_uri.as_str(),
            &reference_position,
            41,
        )?)?,
        "fingerprint-only references must stay byte-identical after occurrence memo rebuild"
    );

    fs::remove_dir_all(root.as_path())?;
    Ok(())
}

fn initialize_workspace(state: &mut LspShellState, workspace_uri: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "workspace",
                    },
                ],
            },
        }),
    );
}

fn open_style_document(state: &mut LspShellState, uri: &str, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": text,
                },
            },
        }),
    );
}

fn definition_at(
    state: &mut LspShellState,
    uri: &str,
    position: &Value,
    id: u64,
) -> Result<Value, Box<dyn Error>> {
    let response = handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {
                    "uri": uri,
                },
                "position": position,
            },
        }),
    )
    .ok_or("definition request should produce a response")?;
    response
        .get("result")
        .cloned()
        .ok_or_else(|| "definition response should include result".into())
}

fn references_at(
    state: &mut LspShellState,
    uri: &str,
    position: &Value,
    id: u64,
) -> Result<Value, Box<dyn Error>> {
    let response = handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/references",
            "params": {
                "textDocument": {
                    "uri": uri,
                },
                "position": position,
                "context": {
                    "includeDeclaration": true,
                },
            },
        }),
    )
    .ok_or("references request should produce a response")?;
    response
        .get("result")
        .cloned()
        .ok_or_else(|| "references response should include result".into())
}

fn assert_single_definition_target(result: &Value, expected_uri: &str) -> TestResult {
    let definitions = result
        .as_array()
        .ok_or("definition result should be an array")?;
    assert_eq!(definitions.len(), 1, "expected one definition: {result:?}");
    assert!(
        definitions
            .first()
            .and_then(|definition| definition.get("uri"))
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, expected_uri)),
        "definition should target {expected_uri}: {result:?}"
    );
    Ok(())
}

fn assert_references_include_uri(result: &Value, expected_uri: &str, label: &str) -> TestResult {
    let locations = result
        .as_array()
        .ok_or("references result should be an array")?;
    assert!(
        locations.iter().any(|location| location
            .get("uri")
            .and_then(Value::as_str)
            .is_some_and(|uri| file_uri_equivalent(uri, expected_uri))),
        "references should include {label} {expected_uri}: {result:?}"
    );
    Ok(())
}

fn fixture_external_sif(canonical_url: &str) -> Result<omena_sif::OmenaSifV1, serde_json::Error> {
    omena_sif::OmenaSifV1::from_static_exports(
        canonical_url,
        omena_sif::OmenaSifGeneratorV1 {
            name: "fixture-sifgen".to_string(),
            version: "0.1.0".to_string(),
            toolchain_id: "fixture-sifgen@0.1.0".to_string(),
        },
        omena_sif::OmenaSifSourceV1 {
            syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
        },
        omena_sif::OmenaSifExportsV1 {
            variables: vec![omena_sif::OmenaSifVariableExportV1 {
                name: "$brand".to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            }],
            mixins: Vec::new(),
            functions: Vec::new(),
            placeholders: Vec::new(),
            forwards: Vec::new(),
        },
        Vec::new(),
        b"$brand: red !default;",
    )
}

fn position_for_offset(text: &str, offset: usize) -> Value {
    let prefix = &text[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let character = prefix
        .rsplit_once('\n')
        .map(|(_, tail)| tail.len())
        .unwrap_or(prefix.len());
    json!({
        "line": line,
        "character": character,
    })
}

fn parent(path: &Path) -> Result<&Path, Box<dyn Error>> {
    path.parent()
        .ok_or_else(|| format!("path has no parent: {}", path.display()).into())
}

fn unique_temp_root(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let root = std::env::temp_dir().join(format!("{prefix}-{}-{millis}", std::process::id()));
    let _ = fs::remove_dir_all(root.as_path());
    Ok(root)
}

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn linked_emission_preserves_import_graph_order_for_selectorless_modules() -> Result<(), String> {
    let element_only = run_linked_build(
        "element-only",
        "app.css",
        &[
            ("app.css", "@import \"./reset.css\"; div { color: red; }"),
            ("reset.css", "div { color: green; }"),
        ],
    )?;
    assert_before(
        element_only.as_str(),
        "color: green",
        "color: red",
        "an imported element-only rule must precede the importing rule",
    )?;

    let mixed_rules = run_linked_build(
        "mixed-rules",
        "app.css",
        &[
            (
                "app.css",
                "@import \"./reset.css\"; .card { padding: 1px; } div { color: red; }",
            ),
            ("reset.css", "div { color: green; }"),
        ],
    )?;
    assert_before(
        mixed_rules.as_str(),
        "color: green",
        ".card",
        "an imported element-only rule must not move past named rules",
    )?;

    let renamed = run_linked_build(
        "renamed",
        "zzz-app.css",
        &[
            (
                "zzz-app.css",
                "@import \"./aaa-reset.css\"; div { color: red; }",
            ),
            ("aaa-reset.css", "div { color: green; }"),
        ],
    )?;
    assert_before(
        renamed.as_str(),
        "color: green",
        "color: red",
        "module names must not influence import-graph placement",
    )?;

    let layered = run_linked_build(
        "cascade-layers",
        "app.css",
        &[
            (
                "app.css",
                "@import \"./layers.css\"; @layer theme { .card { color: blue; } } \
                 @layer base { .card { color: orange; } }",
            ),
            ("layers.css", "@layer base, theme;"),
        ],
    )?;
    assert_before(
        layered.as_str(),
        "@layer base, theme",
        "@layer theme",
        "the layer-order statement must establish order before layer blocks",
    )?;

    Ok(())
}

#[test]
fn linked_emission_accepts_empty_and_comment_only_modules() -> Result<(), String> {
    let output = run_linked_build(
        "empty-modules",
        "app.css",
        &[
            (
                "app.css",
                "@import \"./empty.css\"; @import \"./license.css\"; .app { color: red; }",
            ),
            ("empty.css", ""),
            ("license.css", "/* retained license */"),
        ],
    )?;

    assert_before(
        output.as_str(),
        "/* retained license */",
        ".app",
        "a comment-only dependency must retain its import-graph placement",
    )?;
    Ok(())
}

#[test]
fn linked_emission_routes_reachability_by_owning_module() -> Result<(), String> {
    let fixture = FixtureDir::new("module-owned-reachability")?;
    let entry_path = fixture.path().join("entry.module.css");
    let dependency_path = fixture.path().join("dependency.module.css");
    let engine_input_path = fixture.path().join("engine-input.json");
    let output_path = fixture.path().join("bundle.css");
    let entry_source =
        "@import \"./dependency.module.css\"; .shared { color: red; } .entry-only { color: tan; }";
    let dependency_source =
        ".shared { padding: 8px; } .b-only { color: blue; } .dependency-dead { color: gray; }";

    fs::write(&entry_path, entry_source)
        .map_err(|error| format!("failed to write {}: {error}", entry_path.display()))?;
    fs::write(&dependency_path, dependency_source)
        .map_err(|error| format!("failed to write {}: {error}", dependency_path.display()))?;

    let range = serde_json::json!({
        "start": { "line": 0, "character": 0 },
        "end": { "line": 0, "character": 1 }
    });
    let selector = |name: &str| {
        serde_json::json!({
            "name": name,
            "viewKind": "canonical",
            "canonicalName": name,
            "range": range,
            "nestedSafety": "safe",
            "composes": null,
            "bemSuffix": null
        })
    };
    let engine_input = serde_json::json!({
        "version": "2",
        "workspace": {
            "root": fixture.path(),
            "classnameTransform": "asIs",
            "settingsKey": "module-owned-reachability"
        },
        "sources": [
            {
                "filePath": fixture.path().join("entry.tsx"),
                "document": {
                    "classExpressions": [{
                        "id": "entry-shared",
                        "kind": "styleAccess",
                        "scssModulePath": entry_path,
                        "range": range,
                        "className": "shared",
                        "rootBindingDeclId": null,
                        "accessPath": ["shared"]
                    }]
                },
                "bindingGraph": null
            },
            {
                "filePath": fixture.path().join("dependency.tsx"),
                "document": {
                    "classExpressions": [{
                        "id": "dependency-own",
                        "kind": "styleAccess",
                        "scssModulePath": dependency_path,
                        "range": range,
                        "className": "b-only",
                        "rootBindingDeclId": null,
                        "accessPath": ["b-only"]
                    }]
                },
                "bindingGraph": null
            }
        ],
        "styles": [
            {
                "filePath": entry_path,
                "source": entry_source,
                "document": {
                    "selectors": [
                        selector("shared"),
                        selector("entry-only")
                    ]
                }
            },
            {
                "filePath": dependency_path,
                "source": dependency_source,
                "document": {
                    "selectors": [
                        selector("shared"),
                        selector("b-only"),
                        selector("dependency-dead")
                    ]
                }
            }
        ],
        "typeFacts": [
            {
                "filePath": fixture.path().join("entry.tsx"),
                "expressionId": "entry-shared",
                "facts": {
                    "kind": "exact",
                    "values": ["shared"]
                },
                "controlFlowGraph": null
            },
            {
                "filePath": fixture.path().join("dependency.tsx"),
                "expressionId": "dependency-own",
                "facts": {
                    "kind": "exact",
                    "values": ["b-only"]
                },
                "controlFlowGraph": null
            }
        ]
    });
    fs::write(
        &engine_input_path,
        serde_json::to_vec_pretty(&engine_input)
            .map_err(|error| format!("failed to serialize engine input: {error}"))?,
    )
    .map_err(|error| format!("failed to write {}: {error}", engine_input_path.display()))?;

    let output = Command::new(env!("CARGO_BIN_EXE_omena"))
        .current_dir(fixture.path())
        .arg("build")
        .arg(&entry_path)
        .arg("--bundle")
        .arg("--linked-emission")
        .arg("--tree-shake")
        .arg("--strict-verification")
        .arg("--engine-input-json")
        .arg(&engine_input_path)
        .arg("--source")
        .arg(&dependency_path)
        .arg("--output")
        .arg(&output_path)
        .arg("--json")
        .output()
        .map_err(|error| format!("failed to run omena build: {error}"))?;
    if !output.status.success() {
        // FALSIFIER: id=linked-emission-cli-001 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        return Err(format!(
            "omena build failed\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let response: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("build JSON should parse: {error}"))?;
    let executed_pass_ids = response["payload"]["execution"]["executedPassIds"]
        .as_array()
        .ok_or_else(|| "build JSON omitted executedPassIds".to_string())?;
    // FALSIFIER: id=linked-emission-cli-002 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        executed_pass_ids
            .iter()
            .any(|pass_id| pass_id == "tree-shake-class"),
        "the product witness must deliver non-empty semantic reachability to tree shaking"
    );
    // FALSIFIER: id=linked-emission-cli-003 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        response["payload"]["readySurfaces"]
            .as_array()
            .is_some_and(|surfaces| surfaces
                .iter()
                .any(|surface| surface == "expressionDomainSelectorProjection")),
        "the product witness must consume EngineInputV2 selector projections"
    );
    // FALSIFIER: id=linked-emission-cli-004 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert_eq!(
        response["payload"]["execution"]["strictPolicy"]["profileId"], "strict-verification",
        "the linked module route must preserve the requested verification policy"
    );
    // FALSIFIER: id=linked-emission-cli-005 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        response["payload"]["execution"]["moduleQualifiedShake"]["removedCount"]
            .as_u64()
            .is_some_and(|count| count > 0),
        "the linked entry execution must expose its module-qualified removal summary"
    );

    let css = fs::read_to_string(&output_path)
        .map_err(|error| format!("failed to read {}: {error}", output_path.display()))?;
    // FALSIFIER: id=linked-emission-cli-006 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        css.contains("color: red"),
        "the entry-owned shared selector must remain: {css:?}"
    );
    let dependency_shared_declaration_removed = !css.contains("padding: 8px");
    let dependency_owned_selector_retained = css.contains("color: blue");
    // FALSIFIER: id=linked-emission-cli-007 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        dependency_shared_declaration_removed && dependency_owned_selector_retained,
        "module-qualified ownership mismatch: \
         dependency_shared_declaration_removed={dependency_shared_declaration_removed}, \
         dependency_owned_selector_retained={dependency_owned_selector_retained}, css={css:?}"
    );
    // FALSIFIER: id=linked-emission-cli-008 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        !css.contains("color: gray"),
        "an unreferenced dependency selector must be removed: {css:?}"
    );

    Ok(())
}

#[test]
fn workspace_reachability_outside_build_sources_does_not_block_cli_bundle() -> Result<(), String> {
    let fixture = FixtureDir::new("workspace-reachability-build-domain")?;
    let entry_path = fixture.path().join("entry.module.css");
    let workspace_only_path = fixture.path().join("workspace-only.module.css");
    let engine_input_path = fixture.path().join("engine-input.json");
    let entry_source = ".entry-live { color: red; } .entry-dead { color: gray; }";
    let workspace_only_source = ".workspace-only { color: blue; }";

    fs::write(&entry_path, entry_source)
        .map_err(|error| format!("failed to write {}: {error}", entry_path.display()))?;
    fs::write(&workspace_only_path, workspace_only_source)
        .map_err(|error| format!("failed to write {}: {error}", workspace_only_path.display()))?;

    let range = serde_json::json!({
        "start": { "line": 0, "character": 0 },
        "end": { "line": 0, "character": 1 }
    });
    let selector = |name: &str| {
        serde_json::json!({
            "name": name,
            "viewKind": "canonical",
            "canonicalName": name,
            "range": range,
            "nestedSafety": "safe",
            "composes": null,
            "bemSuffix": null
        })
    };
    let source_reference = |id: &str, style_path: &Path, class_name: &str| {
        serde_json::json!({
            "filePath": fixture.path().join(format!("{id}.tsx")),
            "document": {
                "classExpressions": [{
                    "id": id,
                    "kind": "styleAccess",
                    "scssModulePath": style_path,
                    "range": range,
                    "className": class_name,
                    "rootBindingDeclId": null,
                    "accessPath": [class_name]
                }]
            },
            "bindingGraph": null
        })
    };
    let type_fact = |id: &str, class_name: &str| {
        serde_json::json!({
            "filePath": fixture.path().join(format!("{id}.tsx")),
            "expressionId": id,
            "facts": {
                "kind": "exact",
                "values": [class_name]
            },
            "controlFlowGraph": null
        })
    };
    let engine_input = serde_json::json!({
        "version": "2",
        "workspace": {
            "root": fixture.path(),
            "classnameTransform": "asIs",
            "settingsKey": "workspace-reachability-build-domain"
        },
        "sources": [
            source_reference("entry-live", &entry_path, "entry-live"),
            source_reference(
                "workspace-only",
                &workspace_only_path,
                "workspace-only"
            )
        ],
        "styles": [
            {
                "filePath": entry_path,
                "source": entry_source,
                "document": {
                    "selectors": [
                        selector("entry-live"),
                        selector("entry-dead")
                    ]
                }
            },
            {
                "filePath": workspace_only_path,
                "source": workspace_only_source,
                "document": {
                    "selectors": [selector("workspace-only")]
                }
            }
        ],
        "typeFacts": [
            type_fact("entry-live", "entry-live"),
            type_fact("workspace-only", "workspace-only")
        ]
    });
    fs::write(
        &engine_input_path,
        serde_json::to_vec_pretty(&engine_input)
            .map_err(|error| format!("failed to serialize engine input: {error}"))?,
    )
    .map_err(|error| format!("failed to write {}: {error}", engine_input_path.display()))?;

    for linked in [false, true] {
        let label = if linked { "linked" } else { "default" };
        let output_path = fixture.path().join(format!("{label}.css"));
        let mut command = Command::new(env!("CARGO_BIN_EXE_omena"));
        command
            .current_dir(fixture.path())
            .arg("build")
            .arg(&entry_path)
            .arg("--bundle")
            .arg("--tree-shake")
            .arg("--strict-verification")
            .arg("--engine-input-json")
            .arg(&engine_input_path)
            .arg("--output")
            .arg(&output_path);
        if linked {
            command.arg("--linked-emission");
        }
        let output = command
            .output()
            .map_err(|error| format!("failed to run {label} omena build: {error}"))?;
        if !output.status.success() {
            // FALSIFIER: id=linked-emission-cli-009 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
            return Err(format!(
                "{label} omena build failed\nstdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let css = fs::read_to_string(&output_path)
            .map_err(|error| format!("failed to read {}: {error}", output_path.display()))?;
        // FALSIFIER: id=linked-emission-cli-010 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(
            css.contains("color: red"),
            "{label} build must retain the referenced entry selector: {css:?}"
        );
        // FALSIFIER: id=linked-emission-cli-011 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(
            !css.contains("color: gray"),
            "{label} build must remove the unreferenced entry selector: {css:?}"
        );
        // FALSIFIER: id=linked-emission-cli-012 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        assert!(
            !css.contains("color: blue"),
            "{label} build must not emit a workspace style outside its source domain: {css:?}"
        );
    }

    Ok(())
}

#[test]
fn linked_emission_preserves_composes_target_from_dependency() -> Result<(), String> {
    let fixture = FixtureDir::new("composes-target-reachability")?;
    let entry_path = fixture.path().join("entry.module.css");
    let base_path = fixture.path().join("base.module.css");
    let engine_input_path = fixture.path().join("engine-input.json");
    let entry_source = "@import \"./base.module.css\"; \
        .card { composes: base from \"./base.module.css\"; color: red; }";
    let base_source =
        ".base { padding: 2px; } .base-live { color: blue; } .base-dead { color: gray; }";

    fs::write(&entry_path, entry_source)
        .map_err(|error| format!("failed to write {}: {error}", entry_path.display()))?;
    fs::write(&base_path, base_source)
        .map_err(|error| format!("failed to write {}: {error}", base_path.display()))?;

    let range = serde_json::json!({
        "start": { "line": 0, "character": 0 },
        "end": { "line": 0, "character": 1 }
    });
    let selector = |name: &str| {
        serde_json::json!({
            "name": name,
            "viewKind": "canonical",
            "canonicalName": name,
            "range": range,
            "nestedSafety": "safe",
            "composes": null,
            "bemSuffix": null
        })
    };
    let source_reference = |id: &str, style_path: &Path, class_name: &str| {
        serde_json::json!({
            "filePath": fixture.path().join(format!("{id}.tsx")),
            "document": {
                "classExpressions": [{
                    "id": id,
                    "kind": "styleAccess",
                    "scssModulePath": style_path,
                    "range": range,
                    "className": class_name,
                    "rootBindingDeclId": null,
                    "accessPath": [class_name]
                }]
            },
            "bindingGraph": null
        })
    };
    let type_fact = |id: &str, class_name: &str| {
        serde_json::json!({
            "filePath": fixture.path().join(format!("{id}.tsx")),
            "expressionId": id,
            "facts": {
                "kind": "exact",
                "values": [class_name]
            },
            "controlFlowGraph": null
        })
    };
    let engine_input = serde_json::json!({
        "version": "2",
        "workspace": {
            "root": fixture.path(),
            "classnameTransform": "asIs",
            "settingsKey": "composes-target-reachability"
        },
        "sources": [
            source_reference("entry-card", &entry_path, "card"),
            source_reference("base-live", &base_path, "base-live")
        ],
        "styles": [
            {
                "filePath": entry_path,
                "source": entry_source,
                "document": { "selectors": [selector("card")] }
            },
            {
                "filePath": base_path,
                "source": base_source,
                "document": {
                    "selectors": [
                        selector("base"),
                        selector("base-live"),
                        selector("base-dead")
                    ]
                }
            }
        ],
        "typeFacts": [
            type_fact("entry-card", "card"),
            type_fact("base-live", "base-live")
        ]
    });
    fs::write(
        &engine_input_path,
        serde_json::to_vec_pretty(&engine_input)
            .map_err(|error| format!("failed to serialize engine input: {error}"))?,
    )
    .map_err(|error| format!("failed to write {}: {error}", engine_input_path.display()))?;

    let run = |label: &str, linked: bool| -> Result<String, String> {
        let output_path = fixture.path().join(format!("{label}.css"));
        let mut command = Command::new(env!("CARGO_BIN_EXE_omena"));
        command
            .current_dir(fixture.path())
            .arg("build")
            .arg(&entry_path)
            .arg("--bundle")
            .arg("--tree-shake")
            .arg("--strict-verification")
            .arg("--engine-input-json")
            .arg(&engine_input_path)
            .arg("--source")
            .arg(&base_path)
            .arg("--output")
            .arg(&output_path);
        if linked {
            command.arg("--linked-emission");
        }
        let output = command
            .output()
            .map_err(|error| format!("failed to run omena build: {error}"))?;
        if !output.status.success() {
            // FALSIFIER: id=linked-emission-cli-013 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
            return Err(format!(
                "omena build failed for {label}\nstdout={}\nstderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        fs::read_to_string(&output_path)
            .map_err(|error| format!("failed to read {}: {error}", output_path.display()))
    };

    let default_css = run("default", false)?;
    let linked_css = run("linked", true)?;
    // FALSIFIER: id=linked-emission-cli-014 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        default_css.contains("padding: 2px"),
        "the default path must retain the composed dependency declaration: {default_css:?}"
    );
    // FALSIFIER: id=linked-emission-cli-015 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        linked_css.contains("padding: 2px"),
        "linked emission must retain the composed dependency declaration preserved by the default path: {linked_css:?}"
    );
    // FALSIFIER: id=linked-emission-cli-016 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        linked_css.contains("color: blue"),
        "the directly referenced dependency selector must remain: {linked_css:?}"
    );
    // FALSIFIER: id=linked-emission-cli-017 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert!(
        !linked_css.contains("color: gray"),
        "an unreferenced dependency selector must be removed: {linked_css:?}"
    );

    Ok(())
}

fn run_linked_build(
    label: &str,
    entrypoint: &str,
    sources: &[(&str, &str)],
) -> Result<String, String> {
    let fixture = FixtureDir::new(label)?;
    for (relative_path, source) in sources {
        let path = fixture.path().join(relative_path);
        fs::write(&path, source)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    }

    let output_path = fixture.path().join("bundle.css");
    let mut command = Command::new(env!("CARGO_BIN_EXE_omena"));
    command
        .current_dir(fixture.path())
        .arg("build")
        .arg(entrypoint)
        .arg("--bundle")
        .arg("--linked-emission")
        .arg("--output")
        .arg(&output_path);
    for (relative_path, _) in sources {
        if *relative_path != entrypoint {
            command.arg("--source").arg(relative_path);
        }
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to run omena build: {error}"))?;
    if !output.status.success() {
        // FALSIFIER: id=linked-emission-cli-018 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        return Err(format!(
            "omena build failed for {label}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    fs::read_to_string(&output_path)
        .map_err(|error| format!("failed to read {}: {error}", output_path.display()))
}

fn assert_before(output: &str, first: &str, second: &str, contract: &str) -> Result<(), String> {
    let first_index = output
        .find(first)
        .ok_or_else(|| format!("{contract}: missing {first:?} in {output:?}"))?;
    let second_index = output
        .find(second)
        .ok_or_else(|| format!("{contract}: missing {second:?} in {output:?}"))?;
    if first_index >= second_index {
        // FALSIFIER: id=linked-emission-cli-019 class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
        return Err(format!(
            "{contract}: expected {first:?} before {second:?}, got {output:?}"
        ));
    }
    Ok(())
}

struct FixtureDir {
    path: PathBuf,
}

impl FixtureDir {
    fn new(label: &str) -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "omena-linked-emission-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .map_err(|error| format!("failed to create {}: {error}", path.display()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

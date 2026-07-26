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

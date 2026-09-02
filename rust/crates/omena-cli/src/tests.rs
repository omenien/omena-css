#[cfg(feature = "zk-audit")]
use crate::audit::zk_audit_cli_result_v0;
#[cfg(feature = "zk-audit")]
use crate::commands::{AuditCommand, ZkAuditCommand};
use crate::{
    build::{BUNDLE_CODE_SPLIT_MANIFEST_FILE_NAME, bundle_split_file_name},
    bundle::{BundleCommandOptions, plan_bundle},
    commands::{Cli, Command, LockCommand, ProvenanceCommand, ReportCommand, SifCommand},
    diagnostics::{
        dynamic_classname_diagnostics_summary, resolve_in_process_external_sifs,
        source_diagnostics_summary, style_diagnostics_summary,
        summarize_cross_file_streaming_reachability_diagnostics,
        workspace_source_diagnostics_summaries,
        workspace_source_diagnostics_summaries_unthreaded_for_test,
        workspace_style_diagnostics_summaries,
        workspace_style_diagnostics_summaries_unthreaded_for_test,
    },
    dispatch::{run, run_with_exit},
    external_sif_authority::{
        CliExternalSifLockFailureModeV0, CliExternalSifSelectionV0,
        resolve_cli_external_sif_authority,
    },
    io::read_source,
    lock::{
        AttestationStatementPolicy, build_recorded_shard_verdict,
        collect_lock_source_coverage_issues, collect_lock_trust_tier_issues,
        require_statement_policy_matches, sha256_hex, summarize_verified_attestation_statement,
        try_read_verified_shard_artifact_binding,
        validate_verified_published_sif_statement_binding, write_recorded_shard_verdicts,
    },
    paths::path_string,
    perceptual::perceptual_check_summary,
    product_verb::{CliExit, ProductVerb},
};
use clap::{CommandFactory, Parser};
use omena_query::{
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0,
};
use omena_sif::{OmenaSifAttestationSubjectDigestV1, read_omena_lock_json_v1};
#[cfg(unix)]
use omena_testkit::{InstrumentationSessionV0, with_instrumentation_session};
#[cfg(feature = "zk-audit")]
use omena_zk_audit::{
    ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0, ZK_AUDIT_MECHANISM_SCOPE_V0, cascade_zk_audit_v0,
    prove_and_verify_canonical_margin_cascade_with_arkworks_v0, zk_audit_ci_matrix_v0,
};
#[cfg(unix)]
use std::os::unix::fs as unix_fs;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn product_command_slots_are_complete_and_typed() -> Result<(), String> {
    let command = Cli::command();
    let product_names = [
        "check", "lint", "fmt", "minify", "bundle", "modules", "sass", "intel", "migrate",
        "verify", "ci", "explain",
    ];
    let available = command
        .get_subcommands()
        .map(|subcommand| subcommand.get_name())
        .collect::<Vec<_>>();
    for name in product_names {
        assert!(available.contains(&name), "missing product command {name}");
    }

    let lint = Cli::try_parse_from(["omena", "lint"])
        .map_err(|error| format!("lint product command should parse: {error}"))?;
    run_with_exit(lint).map_err(|error| format!("wired lint command should succeed: {error}"))?;
    let explain = Cli::try_parse_from(["omena", "explain", "bundle", "--chunk", "main"])
        .map_err(|error| format!("explain product command should parse: {error}"))?;
    run_with_exit(explain)
        .map_err(|error| format!("wired explain command should succeed: {error}"))?;
    Cli::try_parse_from([
        "omena",
        "migrate",
        "css-modules-rename",
        "old",
        "new",
        "--plan",
        "migration.json",
    ])
    .map_err(|error| format!("typed migrate command should parse: {error}"))?;
    Cli::try_parse_from(["omena", "sass", "graph", "--json"])
        .map_err(|error| format!("typed Sass graph command should parse: {error}"))?;
    Cli::try_parse_from(["omena", "sass", "unsupported", "--json"])
        .map_err(|error| format!("typed Sass unsupported command should parse: {error}"))?;
    for args in [
        vec!["omena", "check", "app.css", "--watch", "--json"],
        vec!["omena", "lint", "--watch", "--json"],
        vec!["omena", "fmt", "--watch", "--json"],
        vec![
            "omena",
            "explain",
            "cascade",
            "app.css",
            "--line",
            "0",
            "--character",
            "0",
            "--watch",
            "--json",
        ],
    ] {
        Cli::try_parse_from(args)
            .map_err(|error| format!("watch-enabled product command should parse: {error}"))?;
    }

    let verification_root = temp_dir("verification-product-commands");
    fs::create_dir_all(&verification_root).map_err(|error| error.to_string())?;
    fs::write(
        verification_root.join("app.css"),
        ".app {\n  color: red;\n}\n",
    )
    .map_err(|error| error.to_string())?;
    for name in ["verify", "ci"] {
        let cli =
            Cli::try_parse_from(["omena", name, verification_root.to_string_lossy().as_ref()])
                .map_err(|error| format!("{name} product command should parse: {error}"))?;
        run_with_exit(cli)
            .map_err(|error| format!("wired {name} command should succeed: {error}"))?;
    }
    fs::remove_dir_all(verification_root).map_err(|error| error.to_string())?;

    let stub_cases = [("check", ProductVerb::Check)];
    for (name, expected_verb) in stub_cases {
        let cli = Cli::try_parse_from(["omena", name])
            .map_err(|error| format!("{name} product command should parse: {error}"))?;
        let error = match run_with_exit(cli) {
            Ok(()) => return Err(format!("{name} product command unexpectedly succeeded")),
            Err(error) => error,
        };
        assert!(matches!(
            &error,
            CliExit::NotYetWired { verb } if *verb == expected_verb
        ));
        assert_eq!(error.code(), 2);
        assert_eq!(
            error.to_string(),
            format!(
                "omena {name} is reserved but not yet wired; run `omena {name} --help` to inspect its command contract"
            )
        );
    }
    Ok(())
}

#[test]
fn bundle_command_emits_css_and_deterministic_evidence() -> Result<(), String> {
    let root = temp_dir("bundle-command-evidence");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let entry = root.join("app.css");
    let dependency = root.join("tokens.css");
    let css_out = root.join("bundle.css");
    let evidence = root.join("bundle.evidence.json");
    fs::write(&entry, "@import \"./tokens.css\"; .app { color: green; }")
        .map_err(|error| error.to_string())?;
    fs::write(&dependency, ".token { color: blue; }").map_err(|error| error.to_string())?;

    let command = || Cli {
        command: Command::Bundle {
            entry: Some(entry.clone()),
            css_out: Some(css_out.clone()),
            evidence: Some(evidence.clone()),
            source_paths: vec![dependency.clone()],
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: None,
        },
    };
    run(command())?;
    let first = fs::read(&evidence).map_err(|error| error.to_string())?;
    run(command())?;
    let second = fs::read(&evidence).map_err(|error| error.to_string())?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&second).map_err(|error| error.to_string())?;

    assert_eq!(first, second);
    assert_eq!(manifest["outcomeStatus"], "closed");
    assert!(manifest["blockers"].as_array().is_some_and(Vec::is_empty));
    assert_eq!(
        manifest["reachability"]["guarantee"],
        "notClaimedExactTraversal"
    );
    assert_eq!(
        manifest["executionScope"]["product"],
        "omena-query.bundle-execution-scope"
    );
    assert!(css_out.is_file());
    assert!(
        fs::read_to_string(&css_out)
            .map_err(|error| error.to_string())?
            .contains(".app")
    );
    Ok(())
}

#[test]
fn bundle_command_publishes_no_artifact_when_one_destination_is_locked() -> Result<(), String> {
    let root = temp_dir("bundle-command-atomic-output-lock");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let entry = root.join("app.css");
    let css_out = root.join("bundle.css");
    let evidence = root.join("bundle.evidence.json");
    let css_original = b".original {}\n";
    let evidence_original = b"{\"original\":true}\n";
    fs::write(&entry, ".app { color: green; }").map_err(|error| error.to_string())?;
    fs::write(&css_out, css_original).map_err(|error| error.to_string())?;
    fs::write(&evidence, evidence_original).map_err(|error| error.to_string())?;
    let css_lock = root.join(".bundle.css.omena-workspace-edit.lock");
    fs::write(&css_lock, "foreign-lock\n").map_err(|error| error.to_string())?;

    let result = run(Cli {
        command: Command::Bundle {
            entry: Some(entry),
            css_out: Some(css_out.clone()),
            evidence: Some(evidence.clone()),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: None,
        },
    });
    let error = match result {
        Ok(()) => {
            return Err("a foreign lock unexpectedly allowed the bundle output transaction".into());
        }
        Err(error) => error,
    };
    assert!(error.contains("refused concurrent or unrecovered transaction"));
    assert_eq!(
        fs::read(&css_out).map_err(|error| error.to_string())?,
        css_original
    );
    assert_eq!(
        fs::read(&evidence).map_err(|error| error.to_string())?,
        evidence_original,
        "locking one bundle destination must publish neither artifact"
    );
    fs::remove_file(css_lock).map_err(|error| error.to_string())?;
    cleanup_dir(&root);
    Ok(())
}

#[test]
fn bundle_command_rewrites_non_ascii_module_classes() -> Result<(), String> {
    let root = temp_dir("bundle-command-non-ascii-module-class");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let entry = root.join("BracketAccess.module.scss");
    let css_out = root.join("bundle.css");
    fs::write(
        &entry,
        include_str!(
            "../../../../examples/src/scenarios/17-bracket-access/BracketAccess.module.scss"
        ),
    )
    .map_err(|error| error.to_string())?;

    run(Cli {
        command: Command::Bundle {
            entry: Some(entry),
            css_out: Some(css_out.clone()),
            evidence: None,
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: None,
        },
    })?;

    let css = fs::read_to_string(css_out).map_err(|error| error.to_string())?;
    assert!(
        !css.contains(".한글-라벨"),
        "the bundle product path must rewrite non-ASCII module class names"
    );
    let declaration_offset = css
        .find("font-weight: 600")
        .ok_or_else(|| format!("the non-ASCII class declaration was dropped: {css}"))?;
    let rule_start = css[..declaration_offset]
        .rfind('{')
        .ok_or_else(|| format!("the non-ASCII declaration has no containing rule: {css}"))?;
    let selector_start = css[..rule_start].rfind('}').map_or(0, |offset| offset + 1);
    let rewritten_selector = css[selector_start..rule_start].trim();
    assert!(
        rewritten_selector.starts_with("._") && !rewritten_selector.contains("한글-라벨"),
        "the retained declaration must belong to a scoped selector: {css}"
    );
    Ok(())
}

#[test]
fn bundle_command_loads_configured_workspace_sources_without_flags() -> Result<(), String> {
    let root = temp_dir("bundle-command-config-sources");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let entry = root.join("app.css");
    let dependency = root.join("tokens.css");
    let css_out = root.join("bundle.css");
    fs::write(&entry, "@import \"./tokens.css\"; .app { color: green; }")
        .map_err(|error| error.to_string())?;
    fs::write(&dependency, ".configured-token { color: blue; }")
        .map_err(|error| error.to_string())?;
    fs::write(
        root.join("omena.toml"),
        "[build]\nsources = [\"tokens.css\"]\n",
    )
    .map_err(|error| error.to_string())?;

    run(Cli {
        command: Command::Bundle {
            entry: Some(entry),
            css_out: Some(css_out.clone()),
            evidence: None,
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: None,
        },
    })?;

    let css = fs::read_to_string(css_out).map_err(|error| error.to_string())?;
    assert!(css.contains(".configured-token"));
    assert!(css.contains(".app"));
    Ok(())
}

#[test]
fn bundle_command_refuses_missing_dependency_without_legacy_fallback() -> Result<(), String> {
    let root = temp_dir("bundle-command-open-world");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let entry = root.join("app.css");
    let css_out = root.join("bundle.css");
    let evidence = root.join("bundle.evidence.json");
    fs::write(&entry, "@import \"./missing.css\"; .app { color: green; }")
        .map_err(|error| error.to_string())?;

    let error = match run(Cli {
        command: Command::Bundle {
            entry: Some(entry),
            css_out: Some(css_out.clone()),
            evidence: Some(evidence.clone()),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: None,
        },
    }) {
        Ok(()) => {
            return Err("an incomplete bundle world unexpectedly passed admission".to_string());
        }
        Err(error) => error,
    };
    assert!(
        error.contains("linked bundle emission failed: MissingDependency"),
        "{error}"
    );
    assert!(!evidence.exists());
    assert!(!css_out.exists());
    cleanup_dir(&root);
    Ok(())
}

#[test]
fn bundle_lock_is_fallback_behind_local_external_sif_regeneration() -> Result<(), String> {
    let fixture_root = temp_dir("bundle-lock-local-regeneration");
    fs::create_dir_all(&fixture_root).map_err(|error| error.to_string())?;
    let root = fs::canonicalize(&fixture_root).map_err(|error| error.to_string())?;
    let sif_dir = root.join("sif");
    let package_dir = root.join("node_modules/@fixture/tokens");
    fs::create_dir_all(&sif_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&package_dir).map_err(|error| error.to_string())?;
    let entry = root.join("app.module.scss");
    let external = package_dir.join("index.scss");
    let package_manifest = package_dir.join("package.json");
    let sif_path = sif_dir.join("tokens.sif.json");
    let lockfile = root.join("omena.lock");
    fs::write(&external, "$clean: red !default;\n").map_err(|error| error.to_string())?;
    fs::write(
        &package_manifest,
        r#"{"exports":{".":{"sass":"./index.scss"}}}"#,
    )
    .map_err(|error| error.to_string())?;
    let external_url = crate::paths::cli_path_to_file_uri(external.as_path());
    fs::write(
        &entry,
        "@use \"@fixture/tokens\" as tokens;\n.button { color: tokens.$clean; }\n",
    )
    .map_err(|error| error.to_string())?;

    let poisoned = cli_fixture_sif(external_url.as_str(), b"$poisoned: blue !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&poisoned).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/tokens.sif.json", &poisoned)
            .map_err(|error| error.to_string())?,
    ]);
    fs::write(
        &lockfile,
        omena_sif::write_omena_lock_json_v1(&lock).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let plan = |lockfile| {
        plan_bundle(&BundleCommandOptions {
            entry: Some(entry.clone()),
            css_out: None,
            evidence_path: None,
            source_paths: vec![external.clone()],
            package_manifest_paths: vec![package_manifest.clone()],
            external_sif_selection:
                crate::external_sif_authority::CliExternalSifSelectionV0::from_test_arguments(
                    Vec::new(),
                    lockfile,
                ),
        })
    };
    let local_only = serde_json::to_value(plan(None)?.evidence).map_err(|e| e.to_string())?;
    let with_lock =
        serde_json::to_value(plan(Some(lockfile))?.evidence).map_err(|e| e.to_string())?;
    assert_eq!(
        local_only["outcomeStatus"], "closed",
        "bundle authority fixture must be closed: {local_only}"
    );
    assert!(
        local_only["interfaceHashes"]
            .as_array()
            .is_some_and(|entries| !entries.is_empty()),
        "bundle authority fixture must exercise closed-world interface evidence: {local_only}"
    );
    assert_eq!(
        with_lock, local_only,
        "explicit lock bytes changed bundle evidence for a locally regenerable module"
    );

    cleanup_dir(&root);
    Ok(())
}

#[cfg(unix)]
#[test]
fn bundle_lock_symlink_spelling_is_fallback_behind_realpath_local_regeneration()
-> Result<(), String> {
    use std::os::unix::fs::symlink;

    let root = temp_dir("bundle-lock-symlink-local-regeneration");
    let sif_dir = root.join("sif");
    let real_package_dir = root.join(".pnpm/@fixture+tokens@1.0.0/node_modules/@fixture/tokens");
    let linked_package_dir = root.join("node_modules/@fixture/tokens");
    fs::create_dir_all(&sif_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&real_package_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(
        linked_package_dir
            .parent()
            .ok_or_else(|| "linked package fixture should have a parent".to_string())?,
    )
    .map_err(|error| error.to_string())?;
    symlink(&real_package_dir, &linked_package_dir).map_err(|error| error.to_string())?;

    let entry = root.join("app.module.scss");
    let real_external = real_package_dir.join("index.scss");
    let linked_external = linked_package_dir.join("index.scss");
    let real_package_manifest = real_package_dir.join("package.json");
    let linked_package_manifest = linked_package_dir.join("package.json");
    let alias_sif_path = sif_dir.join("tokens-alias.sif.json");
    let realpath_sif_path = sif_dir.join("tokens-realpath.sif.json");
    let alias_lockfile = root.join("omena-alias.lock");
    let realpath_lockfile = root.join("omena-realpath.lock");
    fs::write(&real_external, "$clean: red !default;\n").map_err(|error| error.to_string())?;
    fs::write(
        &real_package_manifest,
        r#"{"exports":{".":{"sass":"./index.scss"}}}"#,
    )
    .map_err(|error| error.to_string())?;
    let canonical_real_external =
        fs::canonicalize(&real_external).map_err(|error| error.to_string())?;
    let real_url = crate::paths::cli_path_to_file_uri(canonical_real_external.as_path());
    let linked_url = crate::paths::cli_path_to_file_uri(linked_external.as_path());
    fs::write(
        &entry,
        "@use \"@fixture/tokens\" as tokens;\n.button { color: tokens.$clean; }\n",
    )
    .map_err(|error| error.to_string())?;

    assert_ne!(
        linked_url, real_url,
        "fixture must preserve distinct symlink and realpath URL spellings"
    );
    assert_eq!(
        fs::canonicalize(&linked_external).map_err(|error| error.to_string())?,
        canonical_real_external,
        "fixture URL spellings must resolve to the same file identity"
    );

    let alias_poisoned = cli_fixture_sif(linked_url.as_str(), b"$poisoned: blue !default;")?;
    fs::write(
        &alias_sif_path,
        omena_sif::write_omena_sif_json_v1(&alias_poisoned).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let alias_lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/tokens-alias.sif.json", &alias_poisoned)
            .map_err(|error| error.to_string())?,
    ]);
    fs::write(
        &alias_lockfile,
        omena_sif::write_omena_lock_json_v1(&alias_lock).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let realpath_poisoned = cli_fixture_sif(real_url.as_str(), b"$poisoned: blue !default;")?;
    fs::write(
        &realpath_sif_path,
        omena_sif::write_omena_sif_json_v1(&realpath_poisoned)
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let realpath_lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1(
            "sif/tokens-realpath.sif.json",
            &realpath_poisoned,
        )
        .map_err(|error| error.to_string())?,
    ]);
    fs::write(
        &realpath_lockfile,
        omena_sif::write_omena_lock_json_v1(&realpath_lock).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let plan = |lockfile| {
        plan_bundle(&BundleCommandOptions {
            entry: Some(entry.clone()),
            css_out: None,
            evidence_path: None,
            source_paths: vec![linked_external.clone()],
            package_manifest_paths: vec![linked_package_manifest.clone()],
            external_sif_selection:
                crate::external_sif_authority::CliExternalSifSelectionV0::from_test_arguments(
                    Vec::new(),
                    lockfile,
                ),
        })
    };
    let local_only = serde_json::to_value(plan(None)?.evidence).map_err(|e| e.to_string())?;
    let with_realpath_lock =
        serde_json::to_value(plan(Some(realpath_lockfile))?.evidence).map_err(|e| e.to_string())?;
    let with_alias_lock =
        serde_json::to_value(plan(Some(alias_lockfile))?.evidence).map_err(|e| e.to_string())?;
    assert_eq!(
        local_only["outcomeStatus"], "closed",
        "symlink authority fixture must be closed: {local_only}"
    );
    assert!(
        local_only["interfaceHashes"]
            .as_array()
            .is_some_and(|entries| !entries.is_empty()),
        "symlink authority fixture must exercise interface evidence: {local_only}"
    );
    assert_eq!(
        with_realpath_lock, local_only,
        "the realpath-spelled control lock changed local-regeneration evidence"
    );
    assert_eq!(
        with_alias_lock, local_only,
        "a symlink-spelled lock URL replaced realpath local-regeneration evidence"
    );

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn bundle_lock_relative_spelling_is_fallback_behind_local_regeneration() -> Result<(), String> {
    let invocation_root = std::env::current_dir()
        .map_err(|error| format!("fixture invocation root should resolve: {error}"))?;
    let fixture_name = temp_path("bundle-lock-relative-local-regeneration")
        .file_name()
        .ok_or_else(|| "relative authority fixture should have a directory name".to_string())?
        .to_os_string();
    let root = invocation_root
        .join("rust/target/omena-cli-tests")
        .join(fixture_name);
    let sif_dir = root.join("sif");
    let package_dir = root.join("node_modules/@fixture/tokens");
    fs::create_dir_all(&sif_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&package_dir).map_err(|error| error.to_string())?;

    let entry = root.join("app.module.scss");
    let external = package_dir.join("index.scss");
    let package_manifest = package_dir.join("package.json");
    let relative_sif_path = sif_dir.join("tokens-relative.sif.json");
    let absolute_sif_path = sif_dir.join("tokens-absolute.sif.json");
    let relative_lockfile = root.join("omena-relative.lock");
    let absolute_lockfile = root.join("omena-absolute.lock");
    fs::write(&external, "$clean: red !default;\n").map_err(|error| error.to_string())?;
    fs::write(
        &package_manifest,
        r#"{"exports":{".":{"sass":"./index.scss"}}}"#,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &entry,
        "@use \"@fixture/tokens\" as tokens;\n.button { color: tokens.$clean; }\n",
    )
    .map_err(|error| error.to_string())?;

    assert!(
        !fs::symlink_metadata(&package_dir)
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink(),
        "relative authority fixture must use a plain node_modules directory"
    );
    let canonical_external = fs::canonicalize(&external).map_err(|error| error.to_string())?;
    let invocation_relative = |path: &Path| {
        path.strip_prefix(&invocation_root)
            .map(Path::to_path_buf)
            .map_err(|error| format!("fixture path should be invocation-relative: {error}"))
    };
    let relative_entry = invocation_relative(&entry)?;
    let relative_external_path = invocation_relative(&external)?;
    let relative_package_manifest = invocation_relative(&package_manifest)?;
    let relative_lockfile_path = invocation_relative(&relative_lockfile)?;
    let absolute_lockfile_path = invocation_relative(&absolute_lockfile)?;
    let relative_url = path_string(relative_external_path.as_path());
    let absolute_url = crate::paths::cli_path_to_file_uri(canonical_external.as_path());
    assert!(
        !Path::new(&relative_url).is_absolute() && !relative_url.starts_with("file://"),
        "fixture must preserve a relative canonicalUrl spelling: {relative_url}"
    );

    let relative_poisoned = cli_fixture_sif(relative_url.as_str(), b"$poisoned: blue !default;")?;
    fs::write(
        &relative_sif_path,
        omena_sif::write_omena_sif_json_v1(&relative_poisoned)
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let relative_lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1(
            "sif/tokens-relative.sif.json",
            &relative_poisoned,
        )
        .map_err(|error| error.to_string())?,
    ]);
    fs::write(
        &relative_lockfile,
        omena_sif::write_omena_lock_json_v1(&relative_lock).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let absolute_poisoned = cli_fixture_sif(absolute_url.as_str(), b"$poisoned: blue !default;")?;
    fs::write(
        &absolute_sif_path,
        omena_sif::write_omena_sif_json_v1(&absolute_poisoned)
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let absolute_lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1(
            "sif/tokens-absolute.sif.json",
            &absolute_poisoned,
        )
        .map_err(|error| error.to_string())?,
    ]);
    fs::write(
        &absolute_lockfile,
        omena_sif::write_omena_lock_json_v1(&absolute_lock).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let plan = |lockfile| {
        plan_bundle(&BundleCommandOptions {
            entry: Some(relative_entry.clone()),
            css_out: None,
            evidence_path: None,
            source_paths: vec![relative_external_path.clone()],
            package_manifest_paths: vec![relative_package_manifest.clone()],
            external_sif_selection:
                crate::external_sif_authority::CliExternalSifSelectionV0::from_test_arguments(
                    Vec::new(),
                    lockfile,
                ),
        })
    };
    let local_only = serde_json::to_value(plan(None)?.evidence).map_err(|e| e.to_string())?;
    let with_absolute_lock = serde_json::to_value(plan(Some(absolute_lockfile_path))?.evidence)
        .map_err(|e| e.to_string())?;
    let with_relative_lock = serde_json::to_value(plan(Some(relative_lockfile_path))?.evidence)
        .map_err(|e| e.to_string())?;
    assert_eq!(
        local_only["outcomeStatus"], "closed",
        "relative authority fixture must be closed: {local_only}"
    );
    assert!(
        local_only["interfaceHashes"]
            .as_array()
            .is_some_and(|entries| !entries.is_empty()),
        "relative authority fixture must exercise interface evidence: {local_only}"
    );
    assert_eq!(
        with_absolute_lock, local_only,
        "the absolute-realpath control lock changed local-regeneration evidence"
    );
    assert_eq!(
        with_relative_lock, local_only,
        "a relative-spelled lock URL replaced invocation-relative local-regeneration evidence"
    );

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn bundle_lock_backslash_spelling_is_fallback_behind_local_regeneration() -> Result<(), String> {
    let invocation_root = std::env::current_dir()
        .map_err(|error| format!("fixture invocation root should resolve: {error}"))?;
    let fixture_name = temp_path("bundle-lock-backslash-local-regeneration")
        .file_name()
        .ok_or_else(|| "backslash authority fixture should have a directory name".to_string())?
        .to_os_string();
    let root = invocation_root
        .join("rust/target/omena-cli-tests")
        .join(fixture_name);
    let sif_dir = root.join("sif");
    let package_dir = root.join("node_modules/@fixture/tokens");
    fs::create_dir_all(&sif_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&package_dir).map_err(|error| error.to_string())?;

    let entry = root.join("app.module.scss");
    let external = package_dir.join("index.scss");
    let package_manifest = package_dir.join("package.json");
    let forward_sif_path = sif_dir.join("tokens-forward.sif.json");
    let backslash_sif_path = sif_dir.join("tokens-backslash.sif.json");
    let forward_lockfile = root.join("omena-forward.lock");
    let backslash_lockfile = root.join("omena-backslash.lock");
    fs::write(&external, "$clean: red !default;\n").map_err(|error| error.to_string())?;
    fs::write(
        &package_manifest,
        r#"{"exports":{".":{"sass":"./index.scss"}}}"#,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &entry,
        "@use \"@fixture/tokens\" as tokens;\n.button { color: tokens.$clean; }\n",
    )
    .map_err(|error| error.to_string())?;

    assert!(
        !fs::symlink_metadata(&package_dir)
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink(),
        "backslash authority fixture must use a plain node_modules directory"
    );
    let invocation_relative = |path: &Path| {
        path.strip_prefix(&invocation_root)
            .map(Path::to_path_buf)
            .map_err(|error| format!("fixture path should be invocation-relative: {error}"))
    };
    let relative_entry = invocation_relative(&entry)?;
    let relative_external_path = invocation_relative(&external)?;
    let relative_package_manifest = invocation_relative(&package_manifest)?;
    let forward_lockfile_path = invocation_relative(&forward_lockfile)?;
    let backslash_lockfile_path = invocation_relative(&backslash_lockfile)?;
    let forward_url = path_string(relative_external_path.as_path()).replace('\\', "/");
    let backslash_url = forward_url.replace('/', "\\");
    assert!(
        !Path::new(&forward_url).is_absolute() && !forward_url.starts_with("file://"),
        "fixture must preserve a relative forward-slash canonicalUrl: {forward_url}"
    );
    assert_ne!(
        backslash_url, forward_url,
        "fixture must preserve a distinct backslash canonicalUrl spelling"
    );
    assert!(
        backslash_url.contains('\\'),
        "fixture canonicalUrl must exercise the backslash axis: {backslash_url}"
    );

    let forward_poisoned = cli_fixture_sif(forward_url.as_str(), b"$poisoned: blue !default;")?;
    fs::write(
        &forward_sif_path,
        omena_sif::write_omena_sif_json_v1(&forward_poisoned).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let forward_lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/tokens-forward.sif.json", &forward_poisoned)
            .map_err(|error| error.to_string())?,
    ]);
    fs::write(
        &forward_lockfile,
        omena_sif::write_omena_lock_json_v1(&forward_lock).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let backslash_poisoned = cli_fixture_sif(backslash_url.as_str(), b"$poisoned: blue !default;")?;
    fs::write(
        &backslash_sif_path,
        omena_sif::write_omena_sif_json_v1(&backslash_poisoned)
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let backslash_lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1(
            "sif/tokens-backslash.sif.json",
            &backslash_poisoned,
        )
        .map_err(|error| error.to_string())?,
    ]);
    fs::write(
        &backslash_lockfile,
        omena_sif::write_omena_lock_json_v1(&backslash_lock).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    let plan = |lockfile| {
        plan_bundle(&BundleCommandOptions {
            entry: Some(relative_entry.clone()),
            css_out: None,
            evidence_path: None,
            source_paths: vec![relative_external_path.clone()],
            package_manifest_paths: vec![relative_package_manifest.clone()],
            external_sif_selection:
                crate::external_sif_authority::CliExternalSifSelectionV0::from_test_arguments(
                    Vec::new(),
                    lockfile,
                ),
        })
    };
    let local_only = serde_json::to_value(plan(None)?.evidence).map_err(|e| e.to_string())?;
    let with_forward_lock = serde_json::to_value(plan(Some(forward_lockfile_path))?.evidence)
        .map_err(|e| e.to_string())?;
    let with_backslash_lock = serde_json::to_value(plan(Some(backslash_lockfile_path))?.evidence)
        .map_err(|e| e.to_string())?;
    assert_eq!(
        local_only["outcomeStatus"], "closed",
        "backslash authority fixture must be closed: {local_only}"
    );
    assert!(
        local_only["interfaceHashes"]
            .as_array()
            .is_some_and(|entries| !entries.is_empty()),
        "backslash authority fixture must exercise interface evidence: {local_only}"
    );
    assert_eq!(
        with_forward_lock, local_only,
        "the forward-slash control lock changed local-regeneration evidence"
    );
    assert_eq!(
        with_backslash_lock, local_only,
        "a backslash-spelled lock URL replaced invocation-relative local-regeneration evidence"
    );

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn minify_profiles_change_the_executed_pass_set_and_output() -> Result<(), String> {
    let root = temp_dir("minify-profiles");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let input = root.join("app.css");
    let context = root.join("context.json");
    let safe_output = root.join("safe.css");
    let semantic_output = root.join("semantic.css");
    let closed_world_output = root.join("closed-world.css");
    fs::write(
        &input,
        ".used { color: #FFFFFF; } .dead { color: red; } .empty {}",
    )
    .map_err(|error| error.to_string())?;
    fs::write(&context, r#"{"reachableClassNames":["used"]}"#)
        .map_err(|error| error.to_string())?;

    for (profile, output, context_path) in [
        ("safe", &safe_output, None),
        ("semantic", &semantic_output, None),
        ("closed-world", &closed_world_output, Some(&context)),
    ] {
        let mut args = vec![
            "omena".to_string(),
            "minify".to_string(),
            input.to_string_lossy().into_owned(),
            "--profile".to_string(),
            profile.to_string(),
            "--output".to_string(),
            output.to_string_lossy().into_owned(),
        ];
        if let Some(context_path) = context_path {
            args.push("--context-json".to_string());
            args.push(context_path.to_string_lossy().into_owned());
        }
        let cli = Cli::try_parse_from(args).map_err(|error| error.to_string())?;
        run_with_exit(cli).map_err(|error| error.to_string())?;
    }

    let safe = fs::read_to_string(&safe_output).map_err(|error| error.to_string())?;
    let semantic = fs::read_to_string(&semantic_output).map_err(|error| error.to_string())?;
    let closed_world =
        fs::read_to_string(&closed_world_output).map_err(|error| error.to_string())?;
    assert_ne!(safe, semantic);
    assert_ne!(semantic, closed_world);
    assert!(safe.contains(".empty"));
    assert!(!semantic.contains(".empty"));
    assert!(semantic.contains(".dead"));
    assert!(!closed_world.contains(".dead"));

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn compatibility_prefixing_composes_with_build_and_minify_products() -> Result<(), String> {
    let root = temp_dir("compatibility-target-products");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let input = root.join("app.css");
    let build_output = root.join("build.css");
    let minify_output = root.join("minify.css");
    fs::write(&input, "/* note */ .card { display: flex; } .empty {}")
        .map_err(|error| error.to_string())?;

    let build = Cli::try_parse_from([
        "omena",
        "build",
        input.to_string_lossy().as_ref(),
        "--target-query",
        "ie 11",
        "--minify",
        "--output",
        build_output.to_string_lossy().as_ref(),
    ])
    .map_err(|error| error.to_string())?;
    run_with_exit(build).map_err(|error| error.to_string())?;

    fs::write(
        root.join("omena.toml"),
        "[minify]\nprofile = \"semantic\"\ntarget = \"ie 11\"\n",
    )
    .map_err(|error| error.to_string())?;
    let minify = Cli::try_parse_from([
        "omena",
        "minify",
        input.to_string_lossy().as_ref(),
        "--output",
        minify_output.to_string_lossy().as_ref(),
    ])
    .map_err(|error| error.to_string())?;
    run_with_exit(minify).map_err(|error| error.to_string())?;

    let build_css = fs::read_to_string(&build_output).map_err(|error| error.to_string())?;
    let minify_css = fs::read_to_string(&minify_output).map_err(|error| error.to_string())?;
    for output in [&build_css, &minify_css] {
        assert!(output.contains("display:-ms-flexbox"));
        assert!(!output.contains("/* note */"));
        assert!(!output.contains(".empty"));
    }

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn target_color_lowering_runs_through_the_minify_product() -> Result<(), String> {
    let root = temp_dir("target-color-lowering");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let input = root.join("app.css");
    let legacy_output = root.join("legacy.css");
    let modern_output = root.join("modern.css");
    fs::write(
        &input,
        ".card { color: light-dark(#000, #fff); background: color-mix(in srgb, red 50%, blue 50%); border-color: rgb(from red r g b); }",
    )
    .map_err(|error| error.to_string())?;

    for (target_query, output) in [("ie 11", &legacy_output), ("chrome 123", &modern_output)] {
        let command = Cli::try_parse_from([
            "omena",
            "minify",
            input.to_string_lossy().as_ref(),
            "--target-query",
            target_query,
            "--output",
            output.to_string_lossy().as_ref(),
        ])
        .map_err(|error| error.to_string())?;
        run_with_exit(command).map_err(|error| error.to_string())?;
    }

    let legacy_css = fs::read_to_string(&legacy_output).map_err(|error| error.to_string())?;
    let modern_css = fs::read_to_string(&modern_output).map_err(|error| error.to_string())?;
    assert!(!legacy_css.contains("light-dark("));
    assert!(!legacy_css.contains("color-mix("));
    assert!(!legacy_css.contains("rgb(from"));
    assert!(legacy_css.contains("@media"));
    assert!(modern_css.contains("light-dark("));
    assert!(modern_css.contains("color-mix("));
    assert!(modern_css.contains("rgb(from"));

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn closed_world_minify_fails_without_reachability_evidence() -> Result<(), String> {
    let input = temp_path("closed-world-minify.css");
    fs::write(&input, ".used { color: red; } .dead { color: blue; }")
        .map_err(|error| error.to_string())?;
    let cli = Cli::try_parse_from([
        "omena",
        "minify",
        input.to_string_lossy().as_ref(),
        "--profile",
        "closed-world",
    ])
    .map_err(|error| error.to_string())?;
    let error = match run_with_exit(cli) {
        Ok(()) => return Err("closed-world minify unexpectedly succeeded".to_string()),
        Err(error) => error,
    };
    let message = error.to_string();
    assert!(message.contains("closed-world minification refused typed blockers"));
    assert!(message.contains("closedWorldPassUnavailable"));
    cleanup(&input);
    Ok(())
}

#[cfg(feature = "lightning-lowering")]
#[test]
fn hybrid_lightning_minify_routes_through_the_product_command() -> Result<(), String> {
    let root = temp_dir("hybrid-minify");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let input = root.join("app.css");
    let output = root.join("app.min.css");
    let source = ".app { display: block; }";
    fs::write(&input, source).map_err(|error| error.to_string())?;
    let cli = Cli::try_parse_from([
        "omena",
        "minify",
        input.to_string_lossy().as_ref(),
        "--profile",
        "safe",
        "--backend",
        "hybrid-lightning",
        "--output",
        output.to_string_lossy().as_ref(),
    ])
    .map_err(|error| error.to_string())?;

    run_with_exit(cli).map_err(|error| error.to_string())?;
    let minified = fs::read_to_string(&output).map_err(|error| error.to_string())?;
    assert!(minified.len() < source.len());
    assert!(minified.contains("display:block"));

    cleanup_dir(&root);
    Ok(())
}

#[cfg(not(feature = "lightning-lowering"))]
#[test]
fn hybrid_lightning_minify_requires_the_optional_backend_feature() -> Result<(), String> {
    let input = temp_path("hybrid-minify-unavailable.css");
    fs::write(&input, ".app { display: block; }").map_err(|error| error.to_string())?;
    let cli = Cli::try_parse_from([
        "omena",
        "minify",
        input.to_string_lossy().as_ref(),
        "--backend",
        "hybrid-lightning",
    ])
    .map_err(|error| error.to_string())?;
    let error = match run_with_exit(cli) {
        Ok(()) => {
            return Err("hybrid minify unexpectedly succeeded without its feature".to_string());
        }
        Err(error) => error,
    };
    assert!(error.to_string().contains("`lightning-lowering` feature"));

    cleanup(&input);
    Ok(())
}

#[test]
fn minify_profile_reads_config_and_cli_override_wins() -> Result<(), String> {
    let root = temp_dir("minify-config-profile");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let input = root.join("app.css");
    let configured_output = root.join("configured.css");
    let overridden_output = root.join("overridden.css");
    fs::write(&input, ".used { color: #FFFFFF; } .empty {}").map_err(|error| error.to_string())?;
    fs::write(root.join("omena.toml"), "[minify]\nprofile = \"safe\"\n")
        .map_err(|error| error.to_string())?;

    let configured = Cli::try_parse_from([
        "omena",
        "minify",
        input.to_string_lossy().as_ref(),
        "--output",
        configured_output.to_string_lossy().as_ref(),
    ])
    .map_err(|error| error.to_string())?;
    run_with_exit(configured).map_err(|error| error.to_string())?;

    let overridden = Cli::try_parse_from([
        "omena",
        "minify",
        input.to_string_lossy().as_ref(),
        "--profile",
        "semantic",
        "--output",
        overridden_output.to_string_lossy().as_ref(),
    ])
    .map_err(|error| error.to_string())?;
    run_with_exit(overridden).map_err(|error| error.to_string())?;

    let configured = fs::read_to_string(&configured_output).map_err(|error| error.to_string())?;
    let overridden = fs::read_to_string(&overridden_output).map_err(|error| error.to_string())?;
    assert!(configured.contains(".empty"));
    assert!(!overridden.contains(".empty"));

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn format_command_checks_then_writes_through_the_product_router() -> Result<(), String> {
    let root = temp_dir("format-command");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let path = root.join("app.css");
    let source = ".app,.panel{color:red;}";
    fs::write(&path, source).map_err(|error| error.to_string())?;
    let path_arg = path.to_string_lossy().into_owned();

    let check = Cli::try_parse_from(["omena", "fmt", path_arg.as_str(), "--check"])
        .map_err(|error| error.to_string())?;
    let Err(check_error) = run_with_exit(check) else {
        return Err("unformatted input unexpectedly passed --check".to_string());
    };
    assert_eq!(check_error.code(), 1);
    assert_eq!(
        fs::read_to_string(&path).map_err(|error| error.to_string())?,
        source
    );

    let write = Cli::try_parse_from(["omena", "fmt", path_arg.as_str(), "--mode", "pretty"])
        .map_err(|error| error.to_string())?;
    run_with_exit(write).map_err(|error| error.to_string())?;
    assert_ne!(
        fs::read_to_string(&path).map_err(|error| error.to_string())?,
        source
    );

    let clean_check = Cli::try_parse_from(["omena", "fmt", path_arg.as_str(), "--check"])
        .map_err(|error| error.to_string())?;
    run_with_exit(clean_check).map_err(|error| error.to_string())?;
    fs::remove_dir_all(root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn build_command_exposes_source_map_flag() {
    let command = Cli::command();
    command.clone().debug_assert();
    let build_argument_names = command
        .get_subcommands()
        .find(|subcommand| subcommand.get_name() == "build")
        .map(|build| {
            build
                .get_arguments()
                .filter_map(|argument| argument.get_long())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    assert!(build_argument_names.contains(&"source-map"));
    assert!(build_argument_names.contains(&"input-source-map"));
    assert!(build_argument_names.contains(&"minify"));
    assert!(build_argument_names.contains(&"strict-verification"));
    assert!(build_argument_names.contains(&"tree-shake"));
    assert!(build_argument_names.contains(&"bundle"));
    assert!(build_argument_names.contains(&"legacy-emission"));
    assert!(build_argument_names.contains(&"bundle-entry"));
}

#[test]
fn legacy_emission_requires_bundle_mode() -> Result<(), String> {
    let path = temp_path("legacy-emission-requires-bundle.css");
    fs::write(&path, ".app { color: red; }").map_err(|error| error.to_string())?;
    let command = Cli::try_parse_from([
        "omena",
        "build",
        path.to_string_lossy().as_ref(),
        "--legacy-emission",
        "--json",
    ])
    .map_err(|error| error.to_string())?;

    let result = run_with_exit(command);

    assert!(result.as_ref().is_err_and(|error| {
        error
            .to_string()
            .contains("--legacy-emission requires --bundle")
    }));
    cleanup(&path);
    Ok(())
}

#[test]
fn lock_verify_attestation_cli_requires_issuer() -> Result<(), String> {
    let missing_issuer = Cli::try_parse_from([
        "omena",
        "lock",
        "verify-attestation",
        "pkg:design-system/_tokens.scss",
        "--artifact",
        "design-system.sif.json",
        "--bundle",
        "design-system.sigstore.json",
        "--reference",
        "sif/design-system.sigstore.json",
    ]);
    assert!(
        missing_issuer
            .as_ref()
            .is_err_and(|error| error.to_string().contains("--issuer")),
        "{missing_issuer:?}"
    );

    let parsed = Cli::try_parse_from([
        "omena",
        "lock",
        "verify-attestation",
        "pkg:design-system/_tokens.scss",
        "--artifact",
        "design-system.sif.json",
        "--bundle",
        "design-system.sigstore.json",
        "--reference",
        "sif/design-system.sigstore.json",
        "--issuer",
        "https://token.actions.githubusercontent.com",
        "--statement-type",
        "https://in-toto.io/Statement/v1",
        "--statement-predicate-type",
        "https://slsa.dev/provenance/v1",
        "--statement-source-repository",
        "https://github.com/omenien/omena-css",
        "--statement-source-ref",
        "refs/heads/master",
        "--statement-source-commit",
        "abcdef0123456789",
        "--statement-builder-id",
        "https://github.com/actions/runner/github-hosted",
        "--statement-build-type",
        "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1",
        "--statement-subject-name",
        "pkg:npm/@omenacss/omena-css@1.0.0",
        "--statement-subject-digest",
        "pkg:npm/@omenacss/omena-css@1.0.0=sha256:0123456789abcdef",
    ])
    .map_err(|error| {
        format!("issuer-bound attestation verification command should parse: {error}")
    })?;

    let Command::Lock {
        command:
            Some(LockCommand::VerifyAttestation {
                issuer,
                identity,
                kind,
                verified_tier,
                statement_type,
                statement_predicate_type,
                statement_source_repository,
                statement_source_ref,
                statement_source_commit,
                statement_builder_id,
                statement_build_type,
                statement_subject_names,
                statement_subject_digests,
                ..
            }),
        ..
    } = parsed.command
    else {
        return Err("expected lock verify-attestation command".to_string());
    };
    assert_eq!(issuer, "https://token.actions.githubusercontent.com");
    assert_eq!(identity, None);
    assert_eq!(kind, "omena-toolchain.sigstore");
    assert_eq!(verified_tier, "t2");
    assert_eq!(
        statement_type.as_deref(),
        Some("https://in-toto.io/Statement/v1")
    );
    assert_eq!(
        statement_predicate_type.as_deref(),
        Some("https://slsa.dev/provenance/v1")
    );
    assert_eq!(
        statement_source_repository.as_deref(),
        Some("https://github.com/omenien/omena-css")
    );
    assert_eq!(statement_source_ref.as_deref(), Some("refs/heads/master"));
    assert_eq!(statement_source_commit.as_deref(), Some("abcdef0123456789"));
    assert_eq!(
        statement_builder_id.as_deref(),
        Some("https://github.com/actions/runner/github-hosted")
    );
    assert_eq!(
        statement_build_type.as_deref(),
        Some("https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1")
    );
    assert_eq!(
        statement_subject_names,
        vec!["pkg:npm/@omenacss/omena-css@1.0.0".to_string()]
    );
    assert_eq!(
        statement_subject_digests,
        vec!["pkg:npm/@omenacss/omena-css@1.0.0=sha256:0123456789abcdef".to_string()]
    );
    Ok(())
}

#[test]
fn lock_verify_attestation_statement_policy_matches_slsa_payload() -> Result<(), String> {
    let statement = serde_json::json!({
        "_type": "https://in-toto.io/Statement/v1",
        "predicateType": "https://slsa.dev/provenance/v1",
        "subject": [
            {
                "name": "pkg:npm/@omenacss/omena-css@1.0.0",
                "digest": {
                    "sha256": "0123456789abcdef"
                }
            }
        ],
        "predicate": {
            "buildDefinition": {
                "buildType": "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1",
                "externalParameters": {
                    "workflow": {
                        "repository": "https://github.com/omenien/omena-css",
                        "ref": "refs/heads/master",
                        "path": ".github/workflows/release.yml"
                    }
                },
                "resolvedDependencies": [
                    {
                        "uri": "git+https://github.com/omenien/omena-css@refs/heads/master",
                        "digest": {
                            "gitCommit": "abcdef0123456789"
                        }
                    }
                ]
            },
            "runDetails": {
                "builder": {
                    "id": "https://github.com/actions/runner/github-hosted"
                }
            }
        }
    });

    let summary = summarize_verified_attestation_statement(&statement);

    assert_eq!(
        summary.statement_type.as_deref(),
        Some("https://in-toto.io/Statement/v1")
    );
    assert_eq!(
        summary.predicate_type.as_deref(),
        Some("https://slsa.dev/provenance/v1")
    );
    assert_eq!(
        summary.source_repository.as_deref(),
        Some("https://github.com/omenien/omena-css")
    );
    assert_eq!(summary.source_ref.as_deref(), Some("refs/heads/master"));
    assert_eq!(summary.source_commit.as_deref(), Some("abcdef0123456789"));
    assert_eq!(
        summary.builder_id.as_deref(),
        Some("https://github.com/actions/runner/github-hosted")
    );
    assert_eq!(
        summary.build_type.as_deref(),
        Some("https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1")
    );
    assert_eq!(
        summary.subject_names,
        vec!["pkg:npm/@omenacss/omena-css@1.0.0".to_string()]
    );
    assert_eq!(
        summary.subject_digests,
        vec![OmenaSifAttestationSubjectDigestV1 {
            name: "pkg:npm/@omenacss/omena-css@1.0.0".to_string(),
            algorithm: "sha256".to_string(),
            digest: "0123456789abcdef".to_string(),
        }]
    );

    require_statement_policy_matches(
        &summary,
        &AttestationStatementPolicy {
            statement_type: Some("https://in-toto.io/Statement/v1".to_string()),
            predicate_type: Some("https://slsa.dev/provenance/v1".to_string()),
            source_repository: Some("https://github.com/omenien/omena-css".to_string()),
            source_ref: Some("refs/heads/master".to_string()),
            source_commit: Some("abcdef0123456789".to_string()),
            builder_id: Some("https://github.com/actions/runner/github-hosted".to_string()),
            build_type: Some(
                "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1"
                    .to_string(),
            ),
            subject_names: vec!["pkg:npm/@omenacss/omena-css@1.0.0".to_string()],
            subject_digests: vec![
                "pkg:npm/@omenacss/omena-css@1.0.0=sha256:0123456789abcdef".to_string(),
            ],
        },
    )?;

    let statement_type_mismatch = require_statement_policy_matches(
        &summary,
        &AttestationStatementPolicy {
            statement_type: Some("https://example.com/Statement/v1".to_string()),
            predicate_type: None,
            source_repository: None,
            source_ref: None,
            source_commit: None,
            builder_id: None,
            build_type: None,
            subject_names: Vec::new(),
            subject_digests: Vec::new(),
        },
    );
    assert!(
        statement_type_mismatch
            .as_ref()
            .is_err_and(|error| error.contains("statementType mismatch")),
        "{statement_type_mismatch:?}"
    );

    let mismatch = require_statement_policy_matches(
        &summary,
        &AttestationStatementPolicy {
            statement_type: None,
            predicate_type: None,
            source_repository: None,
            source_ref: Some("refs/tags/v1.0.0".to_string()),
            source_commit: None,
            builder_id: None,
            build_type: None,
            subject_names: Vec::new(),
            subject_digests: Vec::new(),
        },
    );
    assert!(
        mismatch
            .as_ref()
            .is_err_and(|error| error.contains("sourceRef mismatch")),
        "{mismatch:?}"
    );

    let commit_mismatch = require_statement_policy_matches(
        &summary,
        &AttestationStatementPolicy {
            statement_type: None,
            predicate_type: None,
            source_repository: None,
            source_ref: None,
            source_commit: Some("fedcba9876543210".to_string()),
            builder_id: None,
            build_type: None,
            subject_names: Vec::new(),
            subject_digests: Vec::new(),
        },
    );
    assert!(
        commit_mismatch
            .as_ref()
            .is_err_and(|error| error.contains("sourceCommit mismatch")),
        "{commit_mismatch:?}"
    );

    let build_type_mismatch = require_statement_policy_matches(
        &summary,
        &AttestationStatementPolicy {
            statement_type: None,
            predicate_type: None,
            source_repository: None,
            source_ref: None,
            source_commit: None,
            builder_id: None,
            build_type: Some("https://example.com/build-type/v1".to_string()),
            subject_names: Vec::new(),
            subject_digests: Vec::new(),
        },
    );
    assert!(
        build_type_mismatch
            .as_ref()
            .is_err_and(|error| error.contains("buildType mismatch")),
        "{build_type_mismatch:?}"
    );

    let subject_mismatch = require_statement_policy_matches(
        &summary,
        &AttestationStatementPolicy {
            statement_type: None,
            predicate_type: None,
            source_repository: None,
            source_ref: None,
            source_commit: None,
            builder_id: None,
            build_type: None,
            subject_names: vec!["pkg:npm/@omenacss/other@1.0.0".to_string()],
            subject_digests: Vec::new(),
        },
    );
    assert!(
        subject_mismatch
            .as_ref()
            .is_err_and(|error| error.contains("subjectName")),
        "{subject_mismatch:?}"
    );

    let subject_digest_mismatch = require_statement_policy_matches(
        &summary,
        &AttestationStatementPolicy {
            statement_type: None,
            predicate_type: None,
            source_repository: None,
            source_ref: None,
            source_commit: None,
            builder_id: None,
            build_type: None,
            subject_names: Vec::new(),
            subject_digests: vec![
                "pkg:npm/@omenacss/omena-css@1.0.0=sha256:fedcba9876543210".to_string(),
            ],
        },
    );
    assert!(
        subject_digest_mismatch
            .as_ref()
            .is_err_and(|error| error.contains("subjectDigest")),
        "{subject_digest_mismatch:?}"
    );

    Ok(())
}

#[test]
fn lock_verify_attestation_requires_signed_published_subject_binding() -> Result<(), String> {
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    let sif_source = omena_sif::write_omena_sif_json_v1(&sif)
        .map_err(|error| format!("fixture SIF should serialize: {error}"))?;
    let entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    let binding = try_read_verified_shard_artifact_binding(
        sif_source.as_bytes(),
        omena_sif::OmenaSifTrustTierV1::T3,
    )?
    .ok_or_else(|| "canonical fixture SIF must produce a shard binding".to_string())?;
    let subject_name = "design-system.attestation-subject.json".to_string();
    let mut statement = cli_fixture_provenance_statement();
    statement.subject_names = vec![subject_name.clone()];
    statement.subject_digests = vec![omena_sif::OmenaSifAttestationSubjectDigestV1 {
        name: subject_name.clone(),
        algorithm: "sha256".to_string(),
        digest: binding.attested_subject_sha256.clone(),
    }];

    validate_verified_published_sif_statement_binding(&entry, &binding, Some(&statement))?;

    let mut digest_mismatch = statement.clone();
    digest_mismatch.subject_digests = vec![omena_sif::OmenaSifAttestationSubjectDigestV1 {
        name: subject_name,
        algorithm: "sha256".to_string(),
        digest: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    }];
    let digest_result =
        validate_verified_published_sif_statement_binding(&entry, &binding, Some(&digest_mismatch));
    assert!(
        digest_result
            .as_ref()
            .is_err_and(|error| error.contains("subjectDigests")),
        "{digest_result:?}"
    );

    let mut subject_mismatch = statement;
    subject_mismatch.subject_names = vec!["other.attestation-subject.json".to_string()];
    let subject_result = validate_verified_published_sif_statement_binding(
        &entry,
        &binding,
        Some(&subject_mismatch),
    );
    assert!(
        subject_result
            .as_ref()
            .is_err_and(|error| error.contains("named *.attestation-subject.json")),
        "{subject_result:?}"
    );

    let missing_statement =
        validate_verified_published_sif_statement_binding(&entry, &binding, None);
    assert!(
        missing_statement
            .as_ref()
            .is_err_and(|error| error.contains("signed SIF provenance statement")),
        "{missing_statement:?}"
    );

    Ok(())
}

#[test]
fn lock_verification_records_an_immutable_hash_keyed_shard_verdict() -> Result<(), String> {
    let workspace_path = temp_dir("lock-shard-verdict");
    let lockfile_path = workspace_path.join("omena.lock");
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    let sif_source = omena_sif::write_omena_sif_json_v1(&sif)
        .map_err(|error| format!("fixture SIF should serialize: {error}"))?;
    let entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    let binding = try_read_verified_shard_artifact_binding(
        sif_source.as_bytes(),
        omena_sif::OmenaSifTrustTierV1::T3,
    )?
    .ok_or_else(|| "canonical fixture SIF must produce a shard binding".to_string())?;
    let verdict = build_recorded_shard_verdict(
        &entry,
        omena_sif::OmenaSifTrustTierV1::T3,
        "sif-shards/design-system.batch.sigstore.json",
        &binding,
    )?;

    write_recorded_shard_verdicts(&lockfile_path, std::slice::from_ref(&verdict))?;
    write_recorded_shard_verdicts(&lockfile_path, std::slice::from_ref(&verdict))?;

    let verdict_dir = workspace_path
        .join(".cache")
        .join("omena")
        .join(omena_sif::OMENA_SIF_SHARD_VERDICT_DIR_V1);
    let verdict_paths = fs::read_dir(verdict_dir.as_path())
        .map_err(|error| format!("fixture verdict directory should be readable: {error}"))?
        .flatten()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    assert_eq!(verdict_paths.len(), 1, "{verdict_paths:?}");
    let recorded_source = read_source(&verdict_paths[0])?;
    assert_eq!(
        omena_sif::read_omena_sif_shard_recorded_verdict_json_v1(recorded_source.as_str())
            .map_err(|error| format!("recorded verdict should parse: {error}"))?,
        verdict
    );

    let mut conflicting = verdict;
    conflicting.signature.reference = "sif-shards/conflicting.sigstore.json".to_string();
    let conflict = write_recorded_shard_verdicts(&lockfile_path, &[conflicting]);
    assert!(
        conflict
            .as_ref()
            .is_err_and(|error| error.contains("immutable recorded shard verdict")),
        "{conflict:?}"
    );
    let _ = fs::remove_dir_all(workspace_path);
    Ok(())
}

#[test]
fn lock_verify_attestation_sha256_hex_is_lowercase_stable() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn lock_verify_attestation_t3_requires_omena_toolchain_kind() {
    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::VerifyAttestation {
                package: "pkg:design-system/_tokens.scss".to_string(),
                lockfile: PathBuf::from("missing.lock"),
                artifact: PathBuf::from("missing.sif.json"),
                bundle: PathBuf::from("missing.sigstore.json"),
                reference: "sif/design-system.sigstore.json".to_string(),
                kind: "npm-provenance.sigstore".to_string(),
                verified_tier: "t3".to_string(),
                identity: None,
                issuer: "https://token.actions.githubusercontent.com".to_string(),
                statement_type: None,
                statement_predicate_type: None,
                statement_source_repository: None,
                statement_source_ref: None,
                statement_source_commit: None,
                statement_builder_id: None,
                statement_build_type: None,
                statement_subject_names: Vec::new(),
                statement_subject_digests: Vec::new(),
                json: true,
            }),
        },
    });

    assert!(
        result.as_ref().is_err_and(|error| error.contains(
            "lock verify-attestation elevated provenance requires --kind omena-toolchain.*"
        )),
        "{result:?}"
    );
}

#[test]
fn lock_verify_attestation_t3_requires_identity() {
    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::VerifyAttestation {
                package: "pkg:design-system/_tokens.scss".to_string(),
                lockfile: PathBuf::from("missing.lock"),
                artifact: PathBuf::from("missing.sif.json"),
                bundle: PathBuf::from("missing.sigstore.json"),
                reference: "sif/design-system.sigstore.json".to_string(),
                kind: "omena-toolchain.sigstore".to_string(),
                verified_tier: "t3".to_string(),
                identity: None,
                issuer: "https://github.com/login/oauth".to_string(),
                statement_type: None,
                statement_predicate_type: None,
                statement_source_repository: None,
                statement_source_ref: None,
                statement_source_commit: None,
                statement_builder_id: None,
                statement_build_type: None,
                statement_subject_names: Vec::new(),
                statement_subject_digests: Vec::new(),
                json: true,
            }),
        },
    });

    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("requires --identity")),
        "{result:?}"
    );
}

#[test]
fn lock_verify_attestation_t3_requires_trusted_workflow_identity() {
    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::VerifyAttestation {
                package: "pkg:design-system/_tokens.scss".to_string(),
                lockfile: PathBuf::from("missing.lock"),
                artifact: PathBuf::from("missing.sif.json"),
                bundle: PathBuf::from("missing.sigstore.json"),
                reference: "sif/design-system.sigstore.json".to_string(),
                kind: "omena-toolchain.sigstore".to_string(),
                verified_tier: "t3".to_string(),
                identity: Some("https://github.com/example/untrusted-workflow.yml".to_string()),
                issuer: "https://token.actions.githubusercontent.com".to_string(),
                statement_type: None,
                statement_predicate_type: None,
                statement_source_repository: None,
                statement_source_ref: None,
                statement_source_commit: None,
                statement_builder_id: None,
                statement_build_type: None,
                statement_subject_names: Vec::new(),
                statement_subject_digests: Vec::new(),
                json: true,
            }),
        },
    });

    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("requires the Omena keyless workflow identity")),
        "{result:?}"
    );
}

#[test]
fn report_resolution_policy_outputs_resolver_contract() {
    let result = run(Cli {
        command: Command::Report {
            command: ReportCommand::ResolutionPolicy { json: true },
        },
    });

    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn report_sass_module_conformance_outputs_bounded_ledger() {
    let result = run(Cli {
        command: Command::Report {
            command: ReportCommand::SassModuleConformance { json: true },
        },
    });

    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn build_command_writes_query_owned_transform_output() -> Result<(), String> {
    let source_path = temp_path("input.css");
    let output_path = temp_path("output.css");
    fs::write(&source_path, ".card { color: #ffffff; }")
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: source_path.clone(),
            output: Some(output_path.clone()),
            passes: vec![
                "whitespace-strip".to_string(),
                "color-compression".to_string(),
            ],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("build output should be written: {error}"))?;
    assert!(output.contains("#fff"));
    assert!(!output.contains("#ffffff"));

    cleanup(&source_path);
    cleanup(&output_path);
    Ok(())
}

#[test]
fn build_strict_verification_refuses_unestablished_winner_evidence() -> Result<(), String> {
    let source_path = temp_path("strict-input.css");
    let strict_output_path = temp_path("strict-output.css");
    let descriptive_output_path = temp_path("descriptive-output.css");
    let source = ".card { color: red; } .card { background: blue; }";
    fs::write(&source_path, source)
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let strict_cli = Cli::try_parse_from([
        "omena".to_string(),
        "build".to_string(),
        path_string(&source_path),
        "--output".to_string(),
        path_string(&strict_output_path),
        "--pass".to_string(),
        "rule-merging".to_string(),
        "--strict-verification".to_string(),
    ])
    .map_err(|error| error.to_string())?;
    run(strict_cli)?;

    let descriptive_cli = Cli::try_parse_from([
        "omena".to_string(),
        "build".to_string(),
        path_string(&source_path),
        "--output".to_string(),
        path_string(&descriptive_output_path),
        "--pass".to_string(),
        "rule-merging".to_string(),
    ])
    .map_err(|error| error.to_string())?;
    run(descriptive_cli)?;

    let strict_output = fs::read_to_string(&strict_output_path)
        .map_err(|error| format!("strict build output should be readable: {error}"))?;
    let descriptive_output = fs::read_to_string(&descriptive_output_path)
        .map_err(|error| format!("descriptive build output should be readable: {error}"))?;
    assert_eq!(strict_output, source);
    assert_ne!(descriptive_output, source);

    cleanup(&source_path);
    cleanup(&strict_output_path);
    cleanup(&descriptive_output_path);
    Ok(())
}

#[test]
fn build_config_runs_the_manifest_bound_postcss_compatibility_stage() -> Result<(), String> {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/postcss-compat");
    let output_root = temp_dir("postcss-compat-output");
    fs::create_dir_all(&output_root).map_err(|error| error.to_string())?;
    let output = output_root.join("output.css");
    let cli = Cli::try_parse_from([
        "omena".to_string(),
        "build".to_string(),
        fixture_root
            .join("input.css")
            .to_string_lossy()
            .into_owned(),
        "--output".to_string(),
        output.to_string_lossy().into_owned(),
    ])
    .map_err(|error| error.to_string())?;

    run(cli)?;
    let css = fs::read_to_string(&output).map_err(|error| error.to_string())?;
    assert!(!css.contains("-webkit-appearance"));
    assert!(!css.contains("::-moz-placeholder"));
    assert!(css.contains("::placeholder"));
    assert!(css.contains("appearance:none"));
    cleanup_dir(&output_root);
    Ok(())
}

#[test]
fn build_command_honors_find_up_toml_config_section() -> Result<(), String> {
    let root = temp_dir("build-config-section");
    let source_dir = root.join("src");
    let source_path = source_dir.join("input.css");
    let output_path = root.join("dist.css");
    fs::create_dir_all(&source_dir)
        .map_err(|error| format!("fixture source dir should be writable: {error}"))?;
    fs::write(
        root.join("omena.config.toml"),
        r#"
[build]
minify = true
output = "dist.css"
"#,
    )
    .map_err(|error| format!("fixture config should be writable: {error}"))?;
    fs::write(&source_path, ".card { color: #ffffff; }")
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: source_path.clone(),
            output: None,
            passes: Vec::new(),
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("configured output should be written: {error}"))?;
    assert!(output.contains("#fff"));
    assert!(!output.contains("#ffffff"));

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_command_reports_malformed_toml_config() -> Result<(), String> {
    let root = temp_dir("build-config-malformed");
    let source_path = root.join("input.css");
    fs::create_dir_all(&root)
        .map_err(|error| format!("fixture root should be writable: {error}"))?;
    fs::write(root.join("omena.config.toml"), "[build\nminify = true")
        .map_err(|error| format!("fixture config should be writable: {error}"))?;
    fs::write(&source_path, ".card { color: red; }")
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: source_path.clone(),
            output: None,
            passes: Vec::new(),
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("failed to parse Omena TOML config")),
        "{result:?}"
    );

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_source_map_requires_json_output() -> Result<(), String> {
    let source_path = temp_path("source-map-requires-json.css");
    fs::write(&source_path, ".card { color: red; }")
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: source_path.clone(),
            output: None,
            passes: vec!["print-css".to_string()],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            source_map: true,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert_eq!(result, Err("--source-map requires --json".to_string()));
    cleanup(&source_path);
    Ok(())
}

#[test]
fn build_minify_preset_is_structural_not_only_trivia() -> Result<(), String> {
    let source_path = temp_path("minify-input.css");
    let trivia_output_path = temp_path("minify-trivia-output.css");
    let minify_output_path = temp_path("minify-output.css");
    fs::write(
        &source_path,
        "/* remove */\n.card {\n  color: #ffffff;\n  margin-top: 1px;\n  margin-right: 2px;\n  margin-bottom: 3px;\n  margin-left: 4px;\n}\n.empty { }\n",
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let trivia_result = run(Cli {
        command: Command::Build {
            path: source_path.clone(),
            output: Some(trivia_output_path.clone()),
            passes: vec!["comment-strip".to_string(), "whitespace-strip".to_string()],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });
    assert!(trivia_result.is_ok(), "{trivia_result:?}");

    let minify_result = run(Cli {
        command: Command::Build {
            path: source_path.clone(),
            output: Some(minify_output_path.clone()),
            passes: Vec::new(),
            minify: true,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });
    assert!(minify_result.is_ok(), "{minify_result:?}");

    let trivia_output = fs::read_to_string(&trivia_output_path)
        .map_err(|error| format!("trivia output should be readable: {error}"))?;
    let minify_output = fs::read_to_string(&minify_output_path)
        .map_err(|error| format!("minify output should be readable: {error}"))?;
    assert!(minify_output.len() < trivia_output.len());
    assert!(minify_output.contains("#fff"));
    assert!(!minify_output.contains(".empty"));

    cleanup(&source_path);
    cleanup(&trivia_output_path);
    cleanup(&minify_output_path);
    Ok(())
}

#[test]
fn build_tree_shake_mode_rejects_target_query() -> Result<(), String> {
    let source_path = temp_path("tree-shake-target-query.css");
    fs::write(&source_path, ".used { color: blue; }")
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: source_path.clone(),
            output: None,
            passes: Vec::new(),
            minify: false,
            strict_verification: false,
            target_query: Some("ie 11".to_string()),
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: true,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert_eq!(
        result,
        Err(
            "cannot combine --target-query with --tree-shake; use --tree-shake without --target-query"
                .to_string(),
        )
    );
    cleanup(&source_path);
    Ok(())
}

#[test]
fn build_tree_shake_mode_removes_unreachable_css_module_selectors() -> Result<(), String> {
    let source_path = temp_path("tree-shake-input.module.css");
    let output_path = temp_path("tree-shake-output.module.css");
    let context_path = temp_path("tree-shake-context.json");
    fs::write(&source_path, ".used { color: blue; } .dead { color: red; }")
        .map_err(|error| format!("fixture source should be writable: {error}"))?;
    fs::write(
        &context_path,
        r#"{
  "reachableClassNames": ["used"]
}"#,
    )
    .map_err(|error| format!("fixture context should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: source_path.clone(),
            output: Some(output_path.clone()),
            passes: Vec::new(),
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: Some(context_path.clone()),
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: true,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("build output should be written: {error}"))?;
    assert!(output.contains(".used"));
    assert!(!output.contains(".dead"));

    cleanup(&source_path);
    cleanup(&output_path);
    cleanup(&context_path);
    Ok(())
}

#[test]
fn build_bundle_mode_inlines_transitive_workspace_imports() -> Result<(), String> {
    let target_path = temp_path("bundle-app.css");
    let tokens_path = temp_path("bundle-tokens.css");
    let base_path = temp_path("bundle-base.css");
    let output_path = temp_path("bundle-output.css");
    let linked_output_path = temp_path("bundle-linked-output.css");
    let tokens_file_name = tokens_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| "tokens fixture should have a file name".to_string())?;
    let base_file_name = base_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| "base fixture should have a file name".to_string())?;

    fs::write(&base_path, ".base { color: red; }")
        .map_err(|error| format!("fixture base source should be writable: {error}"))?;
    fs::write(
        &tokens_path,
        format!(r#"@import "./{base_file_name}"; .token {{ color: blue; }}"#),
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(
        &target_path,
        format!(
            r#".app-before {{ color: green; }} @import "./{tokens_file_name}"; .app-after {{ color: yellow; }}"#
        ),
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let run_bundle = |output: PathBuf, legacy_emission: bool| {
        run(Cli {
            command: Command::Build {
                path: target_path.clone(),
                output: Some(output),
                passes: Vec::new(),
                minify: false,
                strict_verification: false,
                target_query: None,
                allow_logical_to_physical: false,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
                drop_dark_mode_media_queries: false,
                context_json: None,
                engine_input_json: None,
                closed_style_world: false,
                tree_shake: false,
                bundle: true,
                legacy_emission,
                split_out_dir: None,
                bundle_entry_paths: Vec::new(),
                source_paths: vec![tokens_path.clone(), base_path.clone()],
                package_manifest_paths: Vec::new(),
                source_map: false,
                input_source_maps: Vec::new(),
                json: false,
            },
        })
    };
    let result = run_bundle(output_path.clone(), true);

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("bundle output should be written: {error}"))?;
    assert!(output.contains(".base { color: red; }"));
    assert!(output.contains(".token { color: blue; }"));
    assert!(output.contains(".app-before { color: green; }"));
    assert!(output.contains(".app-after { color: yellow; }"));
    assert!(!output.contains("@import"));
    let legacy_before = output
        .find(".app-before")
        .ok_or_else(|| "before rule should exist".to_string())?;
    let legacy_token = output
        .find(".token")
        .ok_or_else(|| "token rule should exist".to_string())?;
    let legacy_after = output
        .find(".app-after")
        .ok_or_else(|| "after rule should exist".to_string())?;
    assert!(legacy_before < legacy_token && legacy_token < legacy_after);

    let linked_result = run_bundle(linked_output_path.clone(), false);
    assert!(linked_result.is_ok(), "{linked_result:?}");
    let linked_output = fs::read_to_string(&linked_output_path)
        .map_err(|error| format!("linked bundle output should be written: {error}"))?;
    assert_eq!(linked_output.matches(".base { color: red; }").count(), 1);
    assert_eq!(linked_output.matches(".token { color: blue; }").count(), 1);
    assert_eq!(
        linked_output
            .matches(".app-before { color: green; }")
            .count(),
        1
    );
    assert_eq!(
        linked_output
            .matches(".app-after { color: yellow; }")
            .count(),
        1
    );
    assert!(!linked_output.contains("@import"));
    let linked_before = linked_output
        .find(".app-before")
        .ok_or_else(|| "linked before rule should exist".to_string())?;
    let linked_token = linked_output
        .find(".token")
        .ok_or_else(|| "linked token rule should exist".to_string())?;
    let linked_after = linked_output
        .find(".app-after")
        .ok_or_else(|| "linked after rule should exist".to_string())?;
    assert!(!(linked_before < linked_token && linked_token < linked_after));
    assert_ne!(linked_output, output);

    cleanup(&target_path);
    cleanup(&tokens_path);
    cleanup(&base_path);
    cleanup(&output_path);
    cleanup(&linked_output_path);
    Ok(())
}

#[test]
fn build_bundle_mode_emits_code_split_outputs() -> Result<(), String> {
    let root = temp_dir("bundle-code-split-output");
    let theme_dir = root.join("theme");
    let split_dir = root.join("split");
    fs::create_dir_all(&theme_dir)
        .map_err(|error| format!("fixture theme dir should be writable: {error}"))?;
    let target_path = root.join("app.css");
    let tokens_path = theme_dir.join("tokens.css");
    let base_path = theme_dir.join("base.css");
    let target_style_path = path_string(&target_path);
    let tokens_style_path = path_string(&tokens_path);
    let base_style_path = path_string(&base_path);
    let target_split_file = bundle_split_file_name(&target_style_path);
    let tokens_split_file = bundle_split_file_name(&tokens_style_path);
    let base_split_file = bundle_split_file_name(&base_style_path);

    fs::write(&base_path, ".base { color: red; }")
        .map_err(|error| format!("fixture base source should be writable: {error}"))?;
    fs::write(
        &tokens_path,
        r#"@import "./base.css" layer(tokens); .token { color: blue; }"#,
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(
        &target_path,
        r#"@import "./theme/tokens.css" supports(display: grid) screen; .app { color: green; }"#,
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: None,
            passes: Vec::new(),
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: true,
            legacy_emission: false,
            split_out_dir: Some(split_dir.clone()),
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone(), base_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let target_output = fs::read_to_string(split_dir.join(&target_split_file))
        .map_err(|error| format!("target split output should be readable: {error}"))?;
    let tokens_output = fs::read_to_string(split_dir.join(&tokens_split_file))
        .map_err(|error| format!("tokens split output should be readable: {error}"))?;
    let base_output = fs::read_to_string(split_dir.join(&base_split_file))
        .map_err(|error| format!("base split output should be readable: {error}"))?;
    let manifest_json = fs::read_to_string(split_dir.join(BUNDLE_CODE_SPLIT_MANIFEST_FILE_NAME))
        .map_err(|error| format!("split manifest should be readable: {error}"))?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_json)
        .map_err(|error| format!("split manifest should be valid JSON: {error}"))?;

    assert!(
        target_output.contains(&format!(
            r#"@import "{tokens_split_file}" supports(display: grid) screen;"#
        )),
        "{target_output}"
    );
    assert!(
        tokens_output.contains(&format!(r#"@import "{base_split_file}" layer(tokens);"#)),
        "{tokens_output}"
    );
    assert!(
        base_output.contains(".base { color: red; }"),
        "{base_output}"
    );
    assert!(
        !target_output.contains("./theme/tokens.css"),
        "{target_output}"
    );
    assert!(!tokens_output.contains("./base.css"), "{tokens_output}");
    assert_eq!(
        manifest.get("product").and_then(|value| value.as_str()),
        Some("omena-cli.bundle-code-split-manifest")
    );
    assert_eq!(
        manifest
            .get("entryStylePath")
            .and_then(|value| value.as_str()),
        Some(target_style_path.as_str())
    );
    assert_eq!(
        manifest.get("entryFile").and_then(|value| value.as_str()),
        Some(target_split_file.as_str())
    );
    assert_eq!(
        manifest.get("outputCount").and_then(|value| value.as_u64()),
        Some(3)
    );
    let manifest_outputs = manifest
        .get("outputs")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "split manifest should include outputs".to_string())?;
    let target_manifest = manifest_outputs
        .iter()
        .find(|value| {
            value.get("fileName").and_then(|value| value.as_str())
                == Some(target_split_file.as_str())
        })
        .ok_or_else(|| "split manifest should include target output".to_string())?;
    let tokens_manifest = manifest_outputs
        .iter()
        .find(|value| {
            value.get("fileName").and_then(|value| value.as_str())
                == Some(tokens_split_file.as_str())
        })
        .ok_or_else(|| "split manifest should include tokens output".to_string())?;
    let base_manifest = manifest_outputs
        .iter()
        .find(|value| {
            value.get("fileName").and_then(|value| value.as_str()) == Some(base_split_file.as_str())
        })
        .ok_or_else(|| "split manifest should include base output".to_string())?;
    assert_eq!(
        target_manifest
            .get("sourceMapFile")
            .and_then(|value| value.as_str()),
        None
    );
    assert_eq!(
        target_manifest
            .get("isEntry")
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        target_manifest
            .get("imports")
            .and_then(|value| value.as_array())
            .and_then(|imports| imports.first())
            .and_then(|import| import.get("fileName"))
            .and_then(|value| value.as_str()),
        Some(tokens_split_file.as_str())
    );
    assert_eq!(
        tokens_manifest
            .get("imports")
            .and_then(|value| value.as_array())
            .and_then(|imports| imports.first())
            .and_then(|import| import.get("fileName"))
            .and_then(|value| value.as_str()),
        Some(base_split_file.as_str())
    );
    assert_eq!(
        base_manifest
            .get("imports")
            .and_then(|value| value.as_array())
            .map(|imports| imports.len()),
        Some(0)
    );

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_bundle_mode_emits_sass_dependency_without_rewriting_module_directive() -> Result<(), String>
{
    let root = temp_dir("bundle-code-split-sass-output");
    let split_dir = root.join("split");
    fs::create_dir_all(&root)
        .map_err(|error| format!("fixture root should be writable: {error}"))?;
    let target_path = root.join("entry.scss");
    let tokens_path = root.join("_tokens.scss");
    fs::write(
        &target_path,
        "@use \"./tokens\";\n.entry { color: green; }\n",
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;
    fs::write(&tokens_path, ".token { color: teal; }\n")
        .map_err(|error| format!("fixture dependency source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: None,
            passes: Vec::new(),
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: true,
            legacy_emission: false,
            split_out_dir: Some(split_dir.clone()),
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let manifest_json = fs::read_to_string(split_dir.join(BUNDLE_CODE_SPLIT_MANIFEST_FILE_NAME))
        .map_err(|error| format!("split manifest should be readable: {error}"))?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_json)
        .map_err(|error| format!("split manifest should be valid JSON: {error}"))?;
    assert_eq!(
        manifest.get("outputCount").and_then(|value| value.as_u64()),
        Some(2)
    );
    let outputs = manifest
        .get("outputs")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "split manifest should include outputs".to_string())?;
    assert!(outputs.iter().any(|output| {
        output.get("sourcePath").and_then(|value| value.as_str())
            == Some(path_string(&tokens_path).as_str())
            && output.get("splitBoundary").and_then(|value| value.as_str())
                == Some("styleDependency")
    }));
    let entry_output = outputs
        .iter()
        .find(|output| output.get("isEntry").and_then(|value| value.as_bool()) == Some(true))
        .ok_or_else(|| "split manifest should include the entry output".to_string())?;
    assert_eq!(
        entry_output
            .get("imports")
            .and_then(|value| value.as_array())
            .map(Vec::len),
        Some(0)
    );
    let entry_file = entry_output
        .get("fileName")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "entry output should name its file".to_string())?;
    let entry_css = fs::read_to_string(split_dir.join(entry_file))
        .map_err(|error| format!("entry split output should be readable: {error}"))?;
    assert!(entry_css.contains("@use \"./tokens\""));

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_bundle_mode_tree_shakes_code_split_outputs() -> Result<(), String> {
    let root = temp_dir("bundle-code-split-tree-shake");
    let theme_dir = root.join("theme");
    let split_dir = root.join("split");
    fs::create_dir_all(&theme_dir)
        .map_err(|error| format!("fixture theme dir should be writable: {error}"))?;
    let target_path = root.join("app.module.css");
    let tokens_path = theme_dir.join("tokens.module.css");
    let context_path = root.join("context.json");
    let output_path = root.join("bundle-output.css");
    let target_style_path = path_string(&target_path);
    let tokens_style_path = path_string(&tokens_path);
    let target_split_file = bundle_split_file_name(&target_style_path);
    let tokens_split_file = bundle_split_file_name(&tokens_style_path);

    fs::write(
        &tokens_path,
        r#".token { color: blue; } .ghost { color: gray; }"#,
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(
        &target_path,
        r#"@import "./theme/tokens.module.css"; .used { color: green; } .dead { color: red; }"#,
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;
    fs::write(
        &context_path,
        r#"{
  "reachableClassNames": ["used", "token"]
}"#,
    )
    .map_err(|error| format!("fixture context should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: Some(output_path.clone()),
            passes: Vec::new(),
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: Some(context_path.clone()),
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: true,
            bundle: true,
            legacy_emission: false,
            split_out_dir: Some(split_dir.clone()),
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let main_output = fs::read_to_string(&output_path)
        .map_err(|error| format!("main bundle output should be readable: {error}"))?;
    let target_output = fs::read_to_string(split_dir.join(&target_split_file))
        .map_err(|error| format!("target split output should be readable: {error}"))?;
    let tokens_output = fs::read_to_string(split_dir.join(&tokens_split_file))
        .map_err(|error| format!("tokens split output should be readable: {error}"))?;

    assert!(main_output.contains("_used"), "{main_output}");
    assert!(main_output.contains("_token"), "{main_output}");
    assert!(!main_output.contains("dead"), "{main_output}");
    assert!(target_output.contains(".used"), "{target_output}");
    assert!(!target_output.contains(".dead"), "{target_output}");
    assert!(tokens_output.contains(".token"), "{tokens_output}");
    assert!(!tokens_output.contains(".ghost"), "{tokens_output}");

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_scss_module_mode_shares_preconfigured_transitive_module_instance() -> Result<(), String> {
    let root = temp_dir("scss-module-preconfigured-transitive-instance");
    let output_path = root.join("output.css");
    let target_path = root.join("App.module.scss");
    let theme_path = root.join("theme.scss");
    let tokens_path = root.join("tokens.scss");
    fs::create_dir_all(&root)
        .map_err(|error| format!("fixture root dir should be writable: {error}"))?;

    fs::write(
        &tokens_path,
        "$brand: blue !default; .base { color: $brand; }",
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(&theme_path, r#"@forward "./tokens";"#)
        .map_err(|error| format!("fixture theme source should be writable: {error}"))?;
    fs::write(
        &target_path,
        r#"@use "./tokens" as tokens with ($brand: red);
@use "./theme" as theme;
.button { color: tokens.$brand; border-color: theme.$brand; }"#,
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: Some(output_path.clone()),
            passes: vec!["scss-module-evaluate".to_string(), "print-css".to_string()],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone(), theme_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("build output should be readable: {error}"))?;
    assert_eq!(output.matches(".base { color: red; }").count(), 1);
    assert!(!output.contains(".base { color: blue; }"), "{output}");
    assert!(
        output.contains(".button { color: red; border-color: red; }"),
        "{output}"
    );

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_scss_module_mode_shares_relative_and_load_path_module_identity() -> Result<(), String> {
    let root = temp_dir("scss-module-relative-load-path-identity");
    let output_path = root.join("output.css");
    let target_dir = root.join("components");
    let tokens_dir = root.join("src/scss");
    let target_path = target_dir.join("App.module.scss");
    let tokens_path = tokens_dir.join("design-system.scss");
    fs::create_dir_all(&target_dir)
        .map_err(|error| format!("fixture target dir should be writable: {error}"))?;
    fs::create_dir_all(&tokens_dir)
        .map_err(|error| format!("fixture token dir should be writable: {error}"))?;

    fs::write(
        &tokens_path,
        "$brand: blue !default; .base { color: $brand; }",
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(
        &target_path,
        r#"@use "../src/scss/design-system.scss" as rel with ($brand: red);
@use "src/scss/design-system.scss" as lp;
.button { color: rel.$brand; border-color: lp.$brand; }"#,
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: Some(output_path.clone()),
            passes: vec!["scss-module-evaluate".to_string(), "print-css".to_string()],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("build output should be readable: {error}"))?;
    assert_eq!(output.matches(".base { color: red; }").count(), 1);
    assert!(!output.contains(".base { color: blue; }"), "{output}");
    assert!(
        output.contains(".button { color: red; border-color: red; }"),
        "{output}"
    );

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_scss_module_mode_imports_used_module_public_variables() -> Result<(), String> {
    let root = temp_dir("scss-module-import-context-used-module");
    let output_path = root.join("output.css");
    let target_dir = root.join("components");
    let theme_dir = root.join("theme");
    let target_path = target_dir.join("App.module.scss");
    let tokens_path = theme_dir.join("_tokens.scss");
    let api_path = theme_dir.join("_api.scss");
    fs::create_dir_all(&target_dir)
        .map_err(|error| format!("fixture target dir should be writable: {error}"))?;
    fs::create_dir_all(&theme_dir)
        .map_err(|error| format!("fixture theme dir should be writable: {error}"))?;

    fs::write(
        &tokens_path,
        "$brand: blue !default; .base { color: $brand; }",
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(
        &api_path,
        r#"@use "./tokens" as tokens with ($brand: red);
$brand: tokens.$brand;
.api { color: $brand; }"#,
    )
    .map_err(|error| format!("fixture api source should be writable: {error}"))?;
    fs::write(
        &target_path,
        r#"@import "../theme/api";
.button { color: $brand; }"#,
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: Some(output_path.clone()),
            passes: vec!["scss-module-evaluate".to_string(), "print-css".to_string()],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone(), api_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("build output should be readable: {error}"))?;
    assert_eq!(output.matches(".base { color: red; }").count(), 1);
    assert!(output.contains(".api { color: red; }"), "{output}");
    assert!(output.contains(".button { color: red; }"), "{output}");
    assert!(!output.contains("tokens.$brand"), "{output}");

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_scss_module_mode_configures_forwarded_module_instance() -> Result<(), String> {
    let root = temp_dir("scss-module-configured-forwarded-instance");
    let output_path = root.join("output.css");
    let target_path = root.join("App.module.scss");
    let theme_path = root.join("theme.scss");
    let tokens_path = root.join("tokens.scss");
    fs::create_dir_all(&root)
        .map_err(|error| format!("fixture root dir should be writable: {error}"))?;

    fs::write(
        &tokens_path,
        "$brand: blue !default; .base { color: $brand; }",
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(&theme_path, r#"@forward "./tokens" as token-*;"#)
        .map_err(|error| format!("fixture theme source should be writable: {error}"))?;
    fs::write(
        &target_path,
        r#"@use "./theme" as theme with ($token-brand: red);
.button { color: theme.$token-brand; }"#,
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: Some(output_path.clone()),
            passes: vec!["scss-module-evaluate".to_string(), "print-css".to_string()],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone(), theme_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("build output should be readable: {error}"))?;
    assert_eq!(output.matches(".base { color: red; }").count(), 1);
    assert!(!output.contains(".base { color: blue; }"), "{output}");
    assert!(output.contains(".button { color: red; }"), "{output}");

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_scss_module_mode_preserves_repeated_source_configuration_conflict() -> Result<(), String> {
    let root = temp_dir("scss-module-repeated-source-configuration-conflict");
    let output_path = root.join("output.css");
    let target_path = root.join("App.module.scss");
    let tokens_path = root.join("tokens.scss");
    fs::create_dir_all(&root)
        .map_err(|error| format!("fixture root dir should be writable: {error}"))?;

    fs::write(
        &tokens_path,
        "$brand: blue !default; .base { color: $brand; }",
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(
        &target_path,
        r#"@use "./tokens" as redTokens with ($brand: red);
@use "./tokens" as blueTokens with ($brand: blue);
.button { color: redTokens.$brand; border-color: blueTokens.$brand; }"#,
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: Some(output_path.clone()),
            passes: vec!["scss-module-evaluate".to_string(), "print-css".to_string()],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: false,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("build output should be readable: {error}"))?;
    assert_eq!(output.matches(".base { color: red; }").count(), 1);
    assert!(!output.contains(".base { color: blue; }"), "{output}");
    assert!(
        output.contains(r#"@use "./tokens" as blueTokens with ($brand: blue)"#),
        "{output}"
    );
    assert!(
        output.contains(".button { color: red; border-color: blueTokens.$brand; }"),
        "{output}"
    );

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_bundle_mode_rewrites_asset_urls_by_source_path() -> Result<(), String> {
    let root = temp_dir("bundle-asset-url-rewrite");
    let target_path = root.join("app.css");
    let tokens_dir = root.join("theme");
    let tokens_path = tokens_dir.join("tokens.css");
    let output_path = root.join("bundle-output.css");
    let target_asset_path = root.join("assets/app.svg");
    let token_asset_path = tokens_dir.join("icons/token.svg");
    fs::create_dir_all(root.join("assets"))
        .map_err(|error| format!("fixture target asset dir should be writable: {error}"))?;
    fs::create_dir_all(tokens_dir.join("icons"))
        .map_err(|error| format!("fixture token asset dir should be writable: {error}"))?;
    fs::write(&target_asset_path, "<svg />")
        .map_err(|error| format!("fixture target asset should be writable: {error}"))?;
    fs::write(&token_asset_path, "<svg />")
        .map_err(|error| format!("fixture token asset should be writable: {error}"))?;
    fs::write(
        &tokens_path,
        r#".token { background-image: url("./icons/token.svg"); }"#,
    )
    .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(
        &target_path,
        r#"@import "./theme/tokens.css"; .app { background: url("./assets/app.svg"); }"#,
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: Some(output_path.clone()),
            passes: Vec::new(),
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: true,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: false,
            input_source_maps: Vec::new(),
            json: false,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let output = fs::read_to_string(&output_path)
        .map_err(|error| format!("bundle output should be written: {error}"))?;
    assert!(
        output.contains(&format!(r#"url("{}")"#, path_string(&target_asset_path))),
        "{output}"
    );
    assert!(
        output.contains(&format!(r#"url("{}")"#, path_string(&token_asset_path))),
        "{output}"
    );
    assert!(!output.contains("./assets/app.svg"), "{output}");
    assert!(!output.contains("./icons/token.svg"), "{output}");

    cleanup_dir(&root);
    Ok(())
}

#[test]
fn build_bundle_mode_combines_json_source_map_origin_chain() -> Result<(), String> {
    let target_path = temp_path("bundle-sourcemap-app.css");
    let tokens_path = temp_path("bundle-sourcemap-tokens.css");
    let tokens_file_name = tokens_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| "tokens fixture should have a file name".to_string())?;

    fs::write(&tokens_path, ".token { color: blue; }")
        .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    fs::write(
        &target_path,
        format!(r#"@import "./{tokens_file_name}"; .app {{ color: green; }}"#),
    )
    .map_err(|error| format!("fixture target source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Build {
            path: target_path.clone(),
            output: None,
            passes: vec!["import-inline".to_string(), "print-css".to_string()],
            minify: false,
            strict_verification: false,
            target_query: None,
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
            context_json: None,
            engine_input_json: None,
            closed_style_world: false,
            tree_shake: false,
            bundle: true,
            legacy_emission: false,
            split_out_dir: None,
            bundle_entry_paths: Vec::new(),
            source_paths: vec![tokens_path.clone()],
            package_manifest_paths: Vec::new(),
            source_map: true,
            input_source_maps: Vec::new(),
            json: true,
        },
    });

    cleanup(&target_path);
    cleanup(&tokens_path);
    result
}

#[cfg(feature = "mdl")]
#[test]
fn compress_command_enforces_budget_bits() -> Result<(), String> {
    let source_path = temp_path("compress.css");
    fs::write(
        &source_path,
        ".card {\n  color: red;\n  background: blue;\n}\n",
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Compress {
            path: source_path.clone(),
            budget_bits: Some(1.0),
            json: true,
        },
    });

    assert!(result.is_err(), "{result:?}");

    cleanup(&source_path);
    Ok(())
}

#[test]
fn cascade_and_context_index_commands_read_query_surfaces() -> Result<(), String> {
    let source_path = temp_path("input.module.css");
    fs::write(
        &source_path,
        "@layer components { :root { --brand: #2563eb; } }\n.button { color: var(--brand); }\n",
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let cascade_result = run(Cli {
        command: Command::Cascade {
            path: source_path.clone(),
            line: 1,
            character: 24,
            engine_input_json: None,
            categorical_evidence: false,
            json: true,
        },
    });
    assert!(cascade_result.is_ok(), "{cascade_result:?}");

    let categorical_cascade_result = run(Cli {
        command: Command::Cascade {
            path: source_path.clone(),
            line: 1,
            character: 24,
            engine_input_json: None,
            categorical_evidence: true,
            json: true,
        },
    });
    assert!(
        categorical_cascade_result.is_ok(),
        "{categorical_cascade_result:?}"
    );

    let context_result = run(Cli {
        command: Command::ContextIndex {
            path: source_path.clone(),
            engine_input_json: None,
            json: true,
        },
    });
    assert!(context_result.is_ok(), "{context_result:?}");

    cleanup(&source_path);
    Ok(())
}

#[test]
fn style_diagnostics_command_reads_query_owned_diagnostics() -> Result<(), String> {
    let source_path = temp_path("diagnostics.module.css");
    fs::write(
        &source_path,
        ":root { --known: #2563eb; }\n.button { color: var(--missing); animation: fade 1s; }\n",
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::StyleDiagnostics {
            path: source_path.clone(),
            source_paths: Vec::new(),
            source_document_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: None,
            external: Some("ignored".to_string()),
            deep_analysis: false,
            json: true,
        },
    });

    assert!(result.is_ok(), "{result:?}");

    cleanup(&source_path);
    Ok(())
}

#[test]
#[ignore = "explicit tracked-style default CLI diagnostics census receipt"]
fn tracked_style_diagnostics_default_cli_preserves_the_pinned_rule_census() -> Result<(), String> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let tracked = std::process::Command::new("git")
        .args(["ls-files", "--", "*.css", "*.scss", "*.less"])
        .current_dir(&repo_root)
        .output()
        .map_err(|error| error.to_string())?;
    if !tracked.status.success() {
        return Err(format!(
            "cannot enumerate tracked styles: {}",
            String::from_utf8_lossy(&tracked.stderr)
        ));
    }

    let mut analyzed_file_count = 0usize;
    let mut counts = BTreeMap::<String, usize>::new();
    for relative_path in String::from_utf8(tracked.stdout)
        .map_err(|error| error.to_string())?
        .lines()
    {
        if relative_path.split('/').any(|component| {
            matches!(
                component,
                "node_modules" | "target" | "dist" | "build" | "coverage" | ".next" | ".turbo"
            )
        }) {
            continue;
        }
        let summary = style_diagnostics_summary(
            repo_root.join(relative_path),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            CliExternalSifSelectionV0::none(),
            None,
            false,
        )?;
        analyzed_file_count += 1;
        for diagnostic in summary.diagnostics {
            *counts.entry(diagnostic.code.to_string()).or_default() += 1;
        }
    }

    println!(
        "trackedStyleDiagnosticsDefaultCliFileCount={analyzed_file_count} ruleCounts={counts:?}"
    );
    assert_eq!(analyzed_file_count, 200);
    assert_eq!(
        counts,
        BTreeMap::from([
            ("deprecatedSassImport".to_string(), 1),
            ("invalidPropertyValue".to_string(), 13),
            ("missing-module".to_string(), 7),
            ("missingComposedModule".to_string(), 2),
            ("missingComposedSelector".to_string(), 11),
            ("missingCustomProperty".to_string(), 13),
            ("missingImportedValue".to_string(), 8),
            ("missingKeyframes".to_string(), 1),
            ("missingSassSymbol".to_string(), 5),
            ("missingValueModule".to_string(), 1),
            ("sassModuleSymlinkResolution".to_string(), 4),
            ("unreachableDeclaration".to_string(), 9),
            ("unresolvedExternalReference".to_string(), 2),
            ("unspecifiedCascadeTie".to_string(), 7),
        ])
    );
    Ok(())
}

#[test]
fn style_diagnostics_ignored_external_mode_does_not_read_explicit_lock() -> Result<(), String> {
    let source_path = temp_path("diagnostics-ignored-lock.module.css");
    let lockfile_path = temp_path("diagnostics-ignored-invalid.lock");
    fs::write(&source_path, ".button { color: red; }\n")
        .map_err(|error| format!("fixture source should be writable: {error}"))?;
    fs::write(&lockfile_path, "not valid lock JSON\n")
        .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::StyleDiagnostics {
            path: source_path.clone(),
            source_paths: Vec::new(),
            source_document_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: Some(lockfile_path.clone()),
            external: Some("ignored".to_string()),
            deep_analysis: false,
            json: true,
        },
    });

    assert!(
        result.is_ok(),
        "ignored external mode read the lock: {result:?}"
    );
    cleanup(&source_path);
    cleanup(&lockfile_path);
    Ok(())
}

#[test]
fn style_diagnostics_command_accepts_external_sif_mode() -> Result<(), String> {
    let source_path = temp_path("external-sif.module.scss");
    fs::write(
        &source_path,
        r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$color; }"#,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    omena_query::reset_workspace_cross_file_summary_direct_recompute_count_for_test();
    omena_query::reset_sass_module_resolution_direct_recompute_count_for_test();
    omena_query::reset_committed_style_semantic_graph_compute_count_for_test();
    let result = run(Cli {
        command: Command::StyleDiagnostics {
            path: source_path.clone(),
            source_paths: Vec::new(),
            source_document_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: None,
            external: Some("sif".to_string()),
            deep_analysis: false,
            json: true,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        omena_query::read_committed_style_semantic_graph_compute_count_for_test(),
        1,
        "style diagnostics CLI should commit one semantic graph for the selector",
    );
    assert_eq!(
        omena_query::read_workspace_cross_file_summary_direct_recompute_count_for_test(),
        0,
        "style diagnostics CLI must read the committed selector summary instead of calling the direct workspace summary API",
    );
    assert_eq!(
        omena_query::read_sass_module_resolution_direct_recompute_count_for_test(),
        0,
        "style diagnostics CLI must read committed Sass resolution instead of the direct workspace API",
    );
    cleanup(&source_path);
    Ok(())
}

#[test]
fn soundiness_report_reads_committed_workspace_diagnostics() -> Result<(), String> {
    let source_path = temp_path("soundiness-report.module.scss");
    fs::write(
        &source_path,
        r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$color; }"#,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    omena_query::reset_workspace_cross_file_summary_direct_recompute_count_for_test();
    omena_query::reset_sass_module_resolution_direct_recompute_count_for_test();
    omena_query::reset_committed_style_semantic_graph_compute_count_for_test();
    let result = run(Cli {
        command: Command::Report {
            command: ReportCommand::Soundiness {
                source_paths: vec![source_path.clone()],
                source_document_paths: Vec::new(),
                package_manifest_paths: Vec::new(),
                sif_paths: Vec::new(),
                lockfile: None,
                external: "sif".to_string(),
                no_suppress: false,
                max_suppressions: None,
                report_stale_suppressions: false,
                json: true,
            },
        },
    });

    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        omena_query::read_committed_style_semantic_graph_compute_count_for_test(),
        1,
        "soundiness report should commit one semantic graph for the workspace selector",
    );
    assert_eq!(
        omena_query::read_workspace_cross_file_summary_direct_recompute_count_for_test(),
        0,
        "soundiness report must read the committed selector summary instead of calling the direct workspace summary API",
    );
    assert_eq!(
        omena_query::read_sass_module_resolution_direct_recompute_count_for_test(),
        0,
        "soundiness report must read committed Sass resolution instead of the direct workspace API",
    );
    cleanup(&source_path);
    Ok(())
}

#[test]
fn style_diagnostics_command_reads_external_sif_artifact() -> Result<(), String> {
    let source_path = temp_path("external-sif-resolved.module.scss");
    let sif_path = temp_path("external-sif-resolved.sif.json");
    fs::write(
        &source_path,
        r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$color; }"#,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;
    let sif = cli_fixture_sif("https://cdn.example/tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::StyleDiagnostics {
            path: source_path.clone(),
            source_paths: Vec::new(),
            source_document_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: vec![sif_path.clone()],
            lockfile: None,
            external: Some("sif".to_string()),
            deep_analysis: false,
            json: true,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    cleanup(&source_path);
    cleanup(&sif_path);
    Ok(())
}

#[test]
fn style_diagnostics_command_reads_external_sif_lockfile() -> Result<(), String> {
    let workspace_path = temp_dir("external-sif-lockfile");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let source_path = workspace_path.join("app.module.scss");
    let sif_path = sif_dir.join("tokens.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    fs::write(
        &source_path,
        r#"@use "design-system/tokens" as remote;
.button { color: remote.$color; }"#,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;
    let sif = cli_fixture_sif("design-system/tokens", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/tokens.sif.json", &sif)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?,
    ]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::StyleDiagnostics {
            path: source_path.clone(),
            source_paths: vec![source_path.clone()],
            source_document_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            sif_paths: Vec::new(),
            lockfile: Some(lockfile_path.clone()),
            external: Some("sif".to_string()),
            deep_analysis: false,
            json: true,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn explicit_cli_lock_input_rejects_sif_hash_mismatch() -> Result<(), String> {
    let workspace_path = temp_dir("external-sif-lock-hash-mismatch");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("tokens.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let expected = cli_fixture_sif("design-system/tokens", b"$color: red !default;")?;
    let mut poisoned = expected.clone();
    poisoned.exports.variables[0].value_repr = Some("poisoned".to_string());
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&poisoned)
            .map_err(|error| format!("poisoned fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("poisoned fixture SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/tokens.sif.json", &expected)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?,
    ]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = resolve_cli_external_sif_authority(
        &CliExternalSifSelectionV0::from_test_arguments(Vec::new(), Some(lockfile_path.clone())),
        &[],
        &OmenaQueryStyleResolutionInputsV0::default(),
        CliExternalSifLockFailureModeV0::Refuse,
    );
    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("expected sifHash") && error.contains("hashes to")),
        "explicit CLI lock input accepted artifact bytes that do not match sifHash: {result:?}"
    );

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn auto_discovered_lock_cannot_outrank_local_external_sif_regeneration() -> Result<(), String> {
    let workspace_path = temp_dir("auto-lock-local-regeneration");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let source_path = workspace_path.join("app.module.scss");
    let external_path = workspace_path.join("_tokens.scss");
    let sif_path = sif_dir.join("tokens.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    fs::write(&external_path, "$clean: red !default;\n")
        .map_err(|error| format!("local external source should be writable: {error}"))?;
    let external_url = format!("file://{}", external_path.to_string_lossy());
    fs::write(
        &source_path,
        format!("@use \"{external_url}\" as tokens;\n.button {{ color: tokens.$evil; }}\n"),
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let mut poisoned = cli_fixture_sif(external_url.as_str(), b"$evil: red !default;")?;
    poisoned.exports.variables[0].name = "$evil".to_string();
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&poisoned)
            .map_err(|error| format!("poisoned fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("poisoned fixture SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/tokens.sif.json", &poisoned)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?,
    ]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let summary = style_diagnostics_summary(
        source_path.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        CliExternalSifSelectionV0::none(),
        Some("sif".to_string()),
        false,
    )?;
    assert!(
        summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "partialExternalSif"),
        "an auto-discovered lock must not make a locally absent symbol appear valid: {:?}",
        summary.diagnostics
    );

    let explicit_summary = style_diagnostics_summary(
        source_path.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        CliExternalSifSelectionV0::from_test_arguments(Vec::new(), Some(lockfile_path.clone())),
        Some("sif".to_string()),
        false,
    )?;
    assert!(
        explicit_summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "partialExternalSif"),
        "an explicit lock must remain behind locally regenerated bytes: {:?}",
        explicit_summary.diagnostics
    );

    cleanup_dir(&workspace_path);
    Ok(())
}

/// #33 P1: the in-process external-SIF resolution must walk a generated SIF's `@forward`
/// chain transitively. A workspace entry `@forward`s `a.scss`, which `@forward`s `b.scss`,
/// which defines `$brand`. Before this change only `a`'s SIF was generated; `b` was reachable
/// only inside the external forward chain, so its symbol stayed invisible. The walk must now
/// generate `b`'s SIF too, keyed to its resolved `file://` identity.
#[test]
fn in_process_external_sifs_walk_transitive_forward_chain() -> Result<(), String> {
    let workspace = temp_dir("transitive-forward");
    fs::create_dir_all(&workspace)
        .map_err(|error| format!("fixture workspace should be writable: {error}"))?;
    let a_path = workspace.join("a.scss");
    let b_path = workspace.join("b.scss");
    // a re-exports b; b defines the symbol. The bridge records `@forward "./b"` as a RAW
    // specifier on a's SIF, so the walk must re-resolve it relative to a's resolved URI.
    fs::write(&a_path, "@forward \"./b\";\n")
        .map_err(|error| format!("a.scss should be writable: {error}"))?;
    fs::write(&b_path, "$brand: #0af;\n")
        .map_err(|error| format!("b.scss should be writable: {error}"))?;

    // The workspace entry forwards `a` over a literal `file://` edge (the shape that already
    // routes through the external branch); `b` is reachable ONLY through a's forward chain.
    // (Match on the resolved inner `sif.canonical_url` suffix: the bridge normalizes the path
    // — e.g. macOS `/tmp` -> `/private/tmp` — so the verbatim temp path is not a sound key.)
    let _ = (&a_path, &b_path);
    let a_file_uri = format!("file://{}", a_path.to_string_lossy());
    let entry = OmenaQueryStyleSourceInputV0 {
        style_path: workspace.join("entry.scss").to_string_lossy().into_owned(),
        style_source: format!("@forward \"{a_file_uri}\";\n"),
    };

    let resolved = resolve_in_process_external_sifs(
        std::slice::from_ref(&entry),
        &[],
        &OmenaQueryStyleResolutionInputsV0::default(),
    );

    // The directly-forwarded module a is present.
    assert!(
        resolved
            .iter()
            .any(|input| input.sif.canonical_url.ends_with("/a.scss")),
        "directly-forwarded a.scss SIF should be generated: {resolved:?}"
    );
    // The TRANSITIVELY-forwarded module b is present and carries the `$brand` symbol that was
    // previously invisible. Its entry key equals its resolved inner `sif.canonical_url`.
    let b_input = resolved
        .iter()
        .find(|input| input.sif.canonical_url.ends_with("/b.scss"))
        .ok_or_else(|| {
            format!("transitively-forwarded b.scss SIF should be generated: {resolved:?}")
        })?;
    assert_eq!(b_input.canonical_url, b_input.sif.canonical_url);
    assert!(
        b_input
            .sif
            .exports
            .variables
            .iter()
            .any(|variable| variable.name == "$brand"),
        "b.scss SIF must export $brand: {:?}",
        b_input.sif.exports.variables
    );

    cleanup_dir(&workspace);
    Ok(())
}

/// #33 P1 over-correction guard: a forward cycle (a `@forward`s b, b `@forward`s a) must
/// terminate — the resolved-`file://`-identity dedup set stops the walk — and must not
/// duplicate either module's SIF.
#[test]
fn in_process_external_sifs_terminate_on_forward_cycle() -> Result<(), String> {
    let workspace = temp_dir("forward-cycle");
    fs::create_dir_all(&workspace)
        .map_err(|error| format!("fixture workspace should be writable: {error}"))?;
    let a_path = workspace.join("a.scss");
    let b_path = workspace.join("b.scss");
    fs::write(&a_path, "@forward \"./b\";\n")
        .map_err(|error| format!("a.scss should be writable: {error}"))?;
    fs::write(&b_path, "@forward \"./a\";\n")
        .map_err(|error| format!("b.scss should be writable: {error}"))?;

    let _ = &b_path;
    let a_file_uri = format!("file://{}", a_path.to_string_lossy());
    let entry = OmenaQueryStyleSourceInputV0 {
        style_path: workspace.join("entry.scss").to_string_lossy().into_owned(),
        style_source: format!("@forward \"{a_file_uri}\";\n"),
    };

    // Must not hang; the dedup set breaks the a<->b cycle.
    let resolved = resolve_in_process_external_sifs(
        std::slice::from_ref(&entry),
        &[],
        &OmenaQueryStyleResolutionInputsV0::default(),
    );

    // Each physical module appears exactly once despite the cycle (dedup on resolved identity).
    let a_count = resolved
        .iter()
        .filter(|input| input.sif.canonical_url.ends_with("/a.scss"))
        .count();
    let b_count = resolved
        .iter()
        .filter(|input| input.sif.canonical_url.ends_with("/b.scss"))
        .count();
    assert_eq!(
        a_count, 1,
        "a.scss SIF must appear exactly once: {resolved:?}"
    );
    assert_eq!(
        b_count, 1,
        "b.scss SIF must appear exactly once: {resolved:?}"
    );

    cleanup_dir(&workspace);
    Ok(())
}

/// #33 P1 over-correction guard: a genuinely-missing transitively-forwarded module produces
/// no fabricated SIF. a forwards `./missing`, which does not exist on disk, so the walk
/// resolves+reads nothing for it — the boundary state is preserved.
#[test]
fn in_process_external_sifs_skip_missing_transitive_forward() -> Result<(), String> {
    let workspace = temp_dir("absent-forward");
    fs::create_dir_all(&workspace)
        .map_err(|error| format!("fixture workspace should be writable: {error}"))?;
    let a_path = workspace.join("a.scss");
    // a forwards a module that does not exist on disk.
    fs::write(&a_path, "@forward \"./gone\";\n")
        .map_err(|error| format!("a.scss should be writable: {error}"))?;

    let a_file_uri = format!("file://{}", a_path.to_string_lossy());
    let entry = OmenaQueryStyleSourceInputV0 {
        style_path: workspace.join("entry.scss").to_string_lossy().into_owned(),
        style_source: format!("@forward \"{a_file_uri}\";\n"),
    };

    let resolved = resolve_in_process_external_sifs(
        std::slice::from_ref(&entry),
        &[],
        &OmenaQueryStyleResolutionInputsV0::default(),
    );

    // a is generated, but no SIF is fabricated for the missing `./gone` target.
    assert!(
        resolved
            .iter()
            .any(|input| input.sif.canonical_url.ends_with("/a.scss")),
        "a.scss SIF should be generated: {resolved:?}"
    );
    assert!(
        resolved
            .iter()
            .all(|input| !input.sif.canonical_url.ends_with("/gone.scss")
                && !input.sif.canonical_url.ends_with("/_gone.scss")),
        "no SIF may be fabricated for the genuinely-missing forward: {resolved:?}"
    );

    cleanup_dir(&workspace);
    Ok(())
}

#[test]
fn in_process_external_sifs_resolve_bare_package_forward_aliases() -> Result<(), String> {
    let workspace = temp_dir("bare-package-forward");
    let app_package = workspace.join("node_modules/@app/theme");
    let design_package = workspace.join("node_modules/@design/tokens");
    fs::create_dir_all(&app_package)
        .map_err(|error| format!("app package should be writable: {error}"))?;
    fs::create_dir_all(&design_package)
        .map_err(|error| format!("design package should be writable: {error}"))?;
    fs::write(
        app_package.join("package.json"),
        r#"{"exports":{"./index":{"sass":"./index.scss"}}}"#,
    )
    .map_err(|error| format!("app package manifest should be writable: {error}"))?;
    fs::write(
        design_package.join("package.json"),
        r#"{"exports":{"./colors":{"sass":"./colors.scss"}}}"#,
    )
    .map_err(|error| format!("design package manifest should be writable: {error}"))?;
    fs::write(
        app_package.join("index.scss"),
        "@forward \"@design/tokens/colors\";\n@forward \"./radius\";\n",
    )
    .map_err(|error| format!("app barrel should be writable: {error}"))?;
    fs::write(app_package.join("_radius.scss"), "$ds_radius-card: 12px;\n")
        .map_err(|error| format!("app radius token should be writable: {error}"))?;
    fs::write(design_package.join("colors.scss"), "$ds_gray-700: #333;\n")
        .map_err(|error| format!("design tokens should be writable: {error}"))?;

    let entry = OmenaQueryStyleSourceInputV0 {
        style_path: workspace
            .join("src/App.module.scss")
            .to_string_lossy()
            .into_owned(),
        style_source: "@use \"@app/theme/index\" as ds;\n.button { color: ds.$ds_gray-700; }\n"
            .to_string(),
    };
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        package_manifests: vec![
            OmenaQueryStylePackageManifestV0 {
                package_json_path: app_package
                    .join("package.json")
                    .to_string_lossy()
                    .into_owned(),
                package_json_source: r#"{"exports":{"./index":{"sass":"./index.scss"}}}"#
                    .to_string(),
            },
            OmenaQueryStylePackageManifestV0 {
                package_json_path: design_package
                    .join("package.json")
                    .to_string_lossy()
                    .into_owned(),
                package_json_source: r#"{"exports":{"./colors":{"sass":"./colors.scss"}}}"#
                    .to_string(),
            },
        ],
        ..OmenaQueryStyleResolutionInputsV0::default()
    };

    let resolved =
        resolve_in_process_external_sifs(std::slice::from_ref(&entry), &[], &resolution_inputs);

    assert!(
        resolved
            .iter()
            .any(|input| input.canonical_url == "@app/theme/index"),
        "root package edge should keep a verbatim alias entry: {resolved:?}"
    );
    let design = resolved
        .iter()
        .find(|input| input.canonical_url == "@design/tokens/colors")
        .ok_or_else(|| {
            format!("bare transitive forward should keep a verbatim alias entry: {resolved:?}")
        })?;
    assert!(
        design
            .sif
            .exports
            .variables
            .iter()
            .any(|variable| variable.name == "$ds_gray-700"),
        "design SIF must expose the forwarded token: {:?}",
        design.sif.exports.variables
    );
    assert!(
        resolved.iter().any(|input| input
            .sif
            .exports
            .variables
            .iter()
            .any(|variable| variable.name == "$ds_radius-card")),
        "relative forward inside the package barrel should expose radius token: {resolved:?}"
    );
    let unknown_entry = OmenaQueryStyleSourceInputV0 {
        style_path: workspace
            .join("src/Unknown.module.scss")
            .to_string_lossy()
            .into_owned(),
        style_source: "@use \"@app/theme/index\" as ds;\n.button { color: ds.$does-not-exist; }\n"
            .to_string(),
    };
    let diagnostics =
        omena_query::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
            unknown_entry.style_path.as_str(),
            std::slice::from_ref(&unknown_entry),
            &[],
            resolution_inputs.package_manifests.as_slice(),
            None,
            omena_query::OmenaQueryExternalModuleModeV0::Auto,
            resolved.as_slice(),
            &resolution_inputs,
        )
        .ok_or_else(|| "SIF-backed negative control diagnostics should be available".to_string())?;
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "partialExternalSif"),
        "unknown symbol on a SIF-backed package boundary should stay partialExternalSif: {:?}",
        diagnostics.diagnostics
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol"),
        "partial external SIF boundary must not double-report missingSassSymbol: {:?}",
        diagnostics.diagnostics
    );

    cleanup_dir(&workspace);
    Ok(())
}

#[cfg(unix)]
#[test]
fn in_process_external_sifs_resolve_bare_package_forward_aliases_through_pnpm_symlinks()
-> Result<(), String> {
    let workspace = temp_dir("bare-package-forward-pnpm-symlink");
    let source = workspace.join("src/App.module.scss");
    let real_app_package = workspace.join(".pnpm/@app+theme@1.0.0/node_modules/@app/theme");
    let real_design_package =
        workspace.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
    let linked_app_scope = workspace.join("node_modules/@app");
    let linked_design_scope = workspace.join("node_modules/@design");
    let linked_app_package = linked_app_scope.join("theme");
    let linked_design_package = linked_design_scope.join("tokens");

    fs::create_dir_all(
        source
            .parent()
            .ok_or_else(|| "source parent should exist".to_string())?,
    )
    .map_err(|error| format!("source parent should be writable: {error}"))?;
    fs::create_dir_all(&real_app_package)
        .map_err(|error| format!("real app package should be writable: {error}"))?;
    fs::create_dir_all(&real_design_package)
        .map_err(|error| format!("real design package should be writable: {error}"))?;
    fs::create_dir_all(&linked_app_scope)
        .map_err(|error| format!("linked app scope should be writable: {error}"))?;
    fs::create_dir_all(&linked_design_scope)
        .map_err(|error| format!("linked design scope should be writable: {error}"))?;
    unix_fs::symlink(real_app_package.as_path(), linked_app_package.as_path())
        .map_err(|error| format!("app package symlink should be creatable: {error}"))?;
    unix_fs::symlink(
        real_design_package.as_path(),
        linked_design_package.as_path(),
    )
    .map_err(|error| format!("design package symlink should be creatable: {error}"))?;

    let app_manifest = r#"{"exports":{"./index":{"sass":"./index.scss"}}}"#;
    let design_manifest = r#"{"exports":{"./colors":{"sass":"./colors.scss"}}}"#;
    fs::write(real_app_package.join("package.json"), app_manifest)
        .map_err(|error| format!("app package manifest should be writable: {error}"))?;
    fs::write(real_design_package.join("package.json"), design_manifest)
        .map_err(|error| format!("design package manifest should be writable: {error}"))?;
    fs::write(
        real_app_package.join("index.scss"),
        "@forward \"@design/tokens/colors\";\n@forward \"./radius\";\n",
    )
    .map_err(|error| format!("app barrel should be writable: {error}"))?;
    fs::write(
        real_app_package.join("_radius.scss"),
        "$ds_radius-card: 12px;\n",
    )
    .map_err(|error| format!("app radius token should be writable: {error}"))?;
    fs::write(
        real_design_package.join("colors.scss"),
        "$ds_gray-700: #374151;\n",
    )
    .map_err(|error| format!("design colors should be writable: {error}"))?;

    let source_text = "@use \"@app/theme/index\" as ds;\n.button { color: ds.$ds_gray-700; border-radius: ds.$ds_radius-card; }\n";
    fs::write(source.as_path(), source_text)
        .map_err(|error| format!("source should be writable: {error}"))?;
    let entry = OmenaQueryStyleSourceInputV0 {
        style_path: source.to_string_lossy().into_owned(),
        style_source: source_text.to_string(),
    };
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        package_manifests: vec![
            OmenaQueryStylePackageManifestV0 {
                package_json_path: real_app_package
                    .join("package.json")
                    .to_string_lossy()
                    .into_owned(),
                package_json_source: app_manifest.to_string(),
            },
            OmenaQueryStylePackageManifestV0 {
                package_json_path: real_design_package
                    .join("package.json")
                    .to_string_lossy()
                    .into_owned(),
                package_json_source: design_manifest.to_string(),
            },
        ],
        ..OmenaQueryStyleResolutionInputsV0::default()
    };

    let resolved =
        resolve_in_process_external_sifs(std::slice::from_ref(&entry), &[], &resolution_inputs);

    assert!(
        resolved
            .iter()
            .any(|input| input.canonical_url == "@app/theme/index"),
        "root package edge should keep a verbatim alias entry through pnpm symlinks: {resolved:?}"
    );
    assert!(
        resolved
            .iter()
            .any(|input| input.canonical_url == "@design/tokens/colors"
                && input
                    .sif
                    .exports
                    .variables
                    .iter()
                    .any(|variable| variable.name == "$ds_gray-700")),
        "bare transitive forward should keep a verbatim alias and exported token through pnpm symlinks: {resolved:?}"
    );
    assert!(
        resolved.iter().any(|input| input
            .sif
            .exports
            .variables
            .iter()
            .any(|variable| variable.name == "$ds_radius-card")),
        "relative forward inside the symlinked package barrel should expose radius token: {resolved:?}"
    );

    cleanup_dir(&workspace);
    Ok(())
}

#[test]
fn style_hover_and_completion_commands_read_query_owned_surfaces() -> Result<(), String> {
    let source_path = temp_path("hover.module.css");
    fs::write(
        &source_path,
        ":root { --brand: #2563eb; }\n.button { color: var(--); }\n",
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let hover_result = run(Cli {
        command: Command::StyleHoverCandidates {
            path: source_path.clone(),
            json: true,
        },
    });
    assert!(hover_result.is_ok(), "{hover_result:?}");

    let completion_result = run(Cli {
        command: Command::StyleCompletion {
            path: source_path.clone(),
            line: 1,
            character: 23,
            json: true,
        },
    });
    assert!(completion_result.is_ok(), "{completion_result:?}");

    cleanup(&source_path);
    Ok(())
}

#[test]
fn source_diagnostics_command_reads_query_owned_diagnostics() -> Result<(), String> {
    let candidates_path = temp_path("source-diagnostics.json");
    fs::write(
        &candidates_path,
        r#"[
          {
            "targetStyleUri": "file:///workspace/src/App.module.css",
            "targetStyleSource": ".root {\n}\n",
            "selectorName": "missing",
            "sourceReferenceRange": {
              "start": { "line": 2, "character": 18 },
              "end": { "line": 2, "character": 25 }
            }
          }
        ]"#,
    )
    .map_err(|error| format!("fixture candidates should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::SourceDiagnostics {
            source_uri: "file:///workspace/src/App.tsx".to_string(),
            candidates_json: Some(candidates_path.clone()),
            source_path: None,
            source_paths: Vec::new(),
            package_manifest_paths: Vec::new(),
            json: true,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let summary = source_diagnostics_summary(
        "file:///workspace/src/App.tsx".to_string(),
        Some(candidates_path.clone()),
        None,
        Vec::new(),
        Vec::new(),
    )?;
    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.diagnostics-for-file");
    assert_eq!(summary.file_uri, "file:///workspace/src/App.tsx");
    assert_eq!(summary.file_kind, "source");
    assert_eq!(summary.diagnostic_count, summary.diagnostics.len());
    assert!(summary.ready_surfaces.contains(&"crossLanguageDiagnostics"));
    assert!(
        summary
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.provenance.is_empty())
    );

    cleanup(&candidates_path);
    Ok(())
}

#[test]
fn source_diagnostics_command_reads_workspace_query_owned_diagnostics() -> Result<(), String> {
    let source_path = temp_path("App.tsx");
    let style_path = temp_path("App.module.scss");
    let style_file_name = style_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "style fixture should have a UTF-8 filename".to_string())?;
    fs::write(
        &source_path,
        r#"import bind from "classnames/bind";
import styles from "./__STYLE_FILE_NAME__";
const cx = bind.bind(styles);
const variant = Math.random() > 0.5 ? "chip" : "ghost";
export function App() {
  return <div className={cx(variant)} />;
}
"#
        .replace("__STYLE_FILE_NAME__", style_file_name),
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;
    fs::write(&style_path, ".chip {}\n")
        .map_err(|error| format!("fixture style should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::SourceDiagnostics {
            source_uri: path_string(&source_path),
            candidates_json: None,
            source_path: Some(source_path.clone()),
            source_paths: vec![style_path.clone()],
            package_manifest_paths: Vec::new(),
            json: true,
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let summary = source_diagnostics_summary(
        path_string(&source_path),
        None,
        Some(source_path.clone()),
        vec![style_path.clone()],
        Vec::new(),
    )?;
    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.diagnostics-for-file");
    assert_eq!(summary.file_kind, "source");
    assert_eq!(summary.diagnostic_count, summary.diagnostics.len());
    assert!(
        summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missingResolvedClassValues")
    );
    assert!(
        summary
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.provenance.is_empty())
    );

    cleanup(&source_path);
    cleanup(&style_path);
    Ok(())
}

#[test]
fn workspace_source_diagnostics_summaries_match_per_file_summaries() -> Result<(), String> {
    let first_source_path = temp_path("BatchA.tsx");
    let second_source_path = temp_path("BatchB.tsx");
    let style_path = temp_path("Batch.module.scss");
    let style_file_name = style_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "style fixture should have a UTF-8 filename".to_string())?;
    let source_template = r#"import styles from "./__STYLE_FILE_NAME__";
export function Component() {
  return <div className={styles.chip} />;
}
"#
    .replace("__STYLE_FILE_NAME__", style_file_name);
    fs::write(&first_source_path, &source_template)
        .map_err(|error| format!("fixture source should be writable: {error}"))?;
    fs::write(&second_source_path, &source_template)
        .map_err(|error| format!("fixture source should be writable: {error}"))?;
    fs::write(&style_path, ".chip {}\n")
        .map_err(|error| format!("fixture style should be writable: {error}"))?;

    let source_paths = vec![first_source_path.clone(), second_source_path.clone()];
    let batch = workspace_source_diagnostics_summaries(
        source_paths.as_slice(),
        std::slice::from_ref(&style_path),
        &[],
    )?;
    assert_eq!(batch.len(), source_paths.len());
    for (source_path, batch_summary) in source_paths.iter().zip(batch.iter()) {
        let per_file = source_diagnostics_summary(
            path_string(source_path),
            None,
            Some(source_path.clone()),
            vec![style_path.clone()],
            Vec::new(),
        )?;
        let batch_json = serde_json::to_value(batch_summary)
            .map_err(|error| format!("batch summary should serialize: {error}"))?;
        let per_file_json = serde_json::to_value(&per_file)
            .map_err(|error| format!("per-file summary should serialize: {error}"))?;
        assert_eq!(
            batch_json,
            per_file_json,
            "workspace batch path must match the per-file path for {}",
            path_string(source_path)
        );
    }
    assert!(workspace_source_diagnostics_summaries(&[], &[], &[])?.is_empty());

    cleanup(&first_source_path);
    cleanup(&second_source_path);
    cleanup(&style_path);
    Ok(())
}

#[cfg(unix)]
#[test]
fn source_diagnostics_share_one_identity_index_for_miss_bearing_imports() -> Result<(), String> {
    with_instrumentation_session(InstrumentationSessionV0::default(), || {
        let root = temp_dir("source-identity-index-once");
        let real_targets = root.join("real-targets");
        let linked_targets = root.join("linked-targets");
        fs::create_dir_all(&real_targets).map_err(|error| {
            format!("source identity-index fixture should be writable: {error}")
        })?;
        unix_fs::symlink(&real_targets, &linked_targets).map_err(|error| {
            format!("source identity-index fixture symlink should be writable: {error}")
        })?;

        let target_path = real_targets.join("Shared.module.css");
        fs::write(&target_path, ".shared { color: red; }\n")
            .map_err(|error| format!("source identity-index target should be writable: {error}"))?;
        let mut source_paths = Vec::new();
        for index in 0..8 {
            let source_path = root.join(format!("Consumer{index}.tsx"));
            fs::write(
                &source_path,
                format!(
                    "import styles from \"./linked-targets/Shared.module.css\";\nexport const Consumer{index} = () => <div className={{styles.shared}} />;\n"
                ),
            )
            .map_err(|error| format!("source identity-index consumer should be writable: {error}"))?;
            source_paths.push(source_path);
        }

        omena_query::reset_omena_resolver_style_identity_cache_for_test();
        let baseline_started = std::time::Instant::now();
        let baseline = workspace_source_diagnostics_summaries_unthreaded_for_test(
            source_paths.as_slice(),
            std::slice::from_ref(&target_path),
            &[],
        )?;
        let baseline_elapsed = baseline_started.elapsed();
        let baseline_build_count =
            omena_query::omena_resolver_style_identity_index_build_count_for_test();
        let baseline_work_count =
            omena_query::omena_resolver_style_identity_index_build_work_count_for_test();

        omena_query::reset_omena_resolver_style_identity_cache_for_test();
        let threaded_started = std::time::Instant::now();
        let threaded = workspace_source_diagnostics_summaries(
            source_paths.as_slice(),
            std::slice::from_ref(&target_path),
            &[],
        )?;
        let threaded_elapsed = threaded_started.elapsed();
        let threaded_build_count =
            omena_query::omena_resolver_style_identity_index_build_count_for_test();
        let threaded_work_count =
            omena_query::omena_resolver_style_identity_index_build_work_count_for_test();
        cleanup_dir(&root);

        let baseline_bytes = serde_json::to_vec(&baseline)
            .map_err(|error| format!("baseline summaries should serialize: {error}"))?;
        let threaded_bytes = serde_json::to_vec(&threaded)
            .map_err(|error| format!("threaded summaries should serialize: {error}"))?;
        assert_eq!(threaded.len(), source_paths.len());
        assert_eq!(
            threaded_bytes, baseline_bytes,
            "threading the resolver identity index must not change diagnostics bytes"
        );
        eprintln!(
            "source-resolver-identity-index baseline_builds={baseline_build_count} baseline_work={baseline_work_count} baseline_ms={} threaded_builds={threaded_build_count} threaded_work={threaded_work_count} threaded_ms={}",
            baseline_elapsed.as_millis(),
            threaded_elapsed.as_millis(),
        );
        assert_eq!(
            baseline_build_count,
            source_paths.len(),
            "the permanent unthreaded control must expose one resolver identity-index build per imported source document; work={baseline_work_count}"
        );
        assert_eq!(
            threaded_build_count, 1,
            "source diagnostics must construct one resolver identity index per workspace invocation; work={threaded_work_count}"
        );
        Ok(())
    })
}

#[cfg(unix)]
#[test]
fn workspace_style_diagnostics_build_one_identity_index_for_miss_bearing_composes_corpus()
-> Result<(), String> {
    with_instrumentation_session(InstrumentationSessionV0::default(), || {
        let root = temp_dir("identity-index-once");
        let real_targets = root.join("real-targets");
        let linked_targets = root.join("linked-targets");
        fs::create_dir_all(&real_targets)
            .map_err(|error| format!("identity-index fixture should be writable: {error}"))?;
        unix_fs::symlink(&real_targets, &linked_targets).map_err(|error| {
            format!("identity-index fixture symlink should be writable: {error}")
        })?;

        let target_path = real_targets.join("Shared.module.scss");
        fs::write(&target_path, ".shared { color: red; }\n")
            .map_err(|error| format!("identity-index target should be writable: {error}"))?;
        let mut style_paths = vec![target_path];
        for index in 0..8 {
            let consumer_path = root.join(format!("Consumer{index}.module.scss"));
            fs::write(
                &consumer_path,
                format!(
                    ".consumer{index} {{ composes: shared from \"./linked-targets/Shared.module.scss\"; }}\n"
                ),
            )
            .map_err(|error| format!("identity-index consumer should be writable: {error}"))?;
            style_paths.push(consumer_path);
        }

        omena_query::reset_omena_resolver_style_identity_cache_for_test();
        let baseline_started = std::time::Instant::now();
        let baseline =
            workspace_style_diagnostics_summaries_unthreaded_for_test(&style_paths, &[], &[])?;
        let baseline_elapsed = baseline_started.elapsed();
        let baseline_build_count =
            omena_query::omena_resolver_style_identity_index_build_count_for_test();
        let baseline_work_count =
            omena_query::omena_resolver_style_identity_index_build_work_count_for_test();

        omena_query::reset_omena_resolver_style_identity_cache_for_test();
        let threaded_started = std::time::Instant::now();
        let summaries = workspace_style_diagnostics_summaries(&style_paths, &[], &[])?;
        let threaded_elapsed = threaded_started.elapsed();
        let build_count = omena_query::omena_resolver_style_identity_index_build_count_for_test();
        let work_count =
            omena_query::omena_resolver_style_identity_index_build_work_count_for_test();
        cleanup_dir(&root);

        assert_eq!(summaries.len(), style_paths.len());
        assert_eq!(
            serde_json::to_value(&summaries).map_err(|error| error.to_string())?,
            serde_json::to_value(&baseline).map_err(|error| error.to_string())?,
            "threading the index must preserve diagnostics bytes"
        );
        assert!(
            baseline_build_count > 1,
            "the miss-bearing perturbation arm must rebuild more than once"
        );
        assert_eq!(
            build_count, 1,
            "this miss-bearing composes fixture must construct one resolver identity index; work={work_count}"
        );
        eprintln!(
            "resolver-identity-index baseline_builds={baseline_build_count} baseline_work={baseline_work_count} baseline_ms={} threaded_builds={build_count} threaded_work={work_count} threaded_ms={}",
            baseline_elapsed.as_millis(),
            threaded_elapsed.as_millis(),
        );
        Ok(())
    })
}

#[test]
fn workspace_style_diagnostics_build_one_identity_index_for_exact_match_source_corpus()
-> Result<(), String> {
    with_instrumentation_session(InstrumentationSessionV0::default(), || {
        const STYLE_COUNT: usize = 60;
        const SOURCE_COUNT: usize = 150;

        let root = temp_dir("identity-index-exact-match-once");
        fs::create_dir_all(&root)
            .map_err(|error| format!("exact-match fixture root should be writable: {error}"))?;
        fs::write(
            root.join("package.json"),
            "{\"name\":\"identity-index-exact-match-once\",\"private\":true}\n",
        )
        .map_err(|error| format!("exact-match package fixture should be writable: {error}"))?;
        let mut style_paths = Vec::with_capacity(STYLE_COUNT);
        for index in 0..STYLE_COUNT {
            let style_path = root.join(format!("Style{index:03}.module.css"));
            fs::write(
                &style_path,
                format!(".item{index:03} {{ color: rgb({} 0 0); }}\n", index % 255),
            )
            .map_err(|error| format!("exact-match style fixture should be writable: {error}"))?;
            style_paths.push(style_path);
        }

        let mut source_paths = Vec::with_capacity(SOURCE_COUNT);
        for index in 0..SOURCE_COUNT {
            let style_index = index % STYLE_COUNT;
            let source_path = root.join(format!("Component{index:03}.tsx"));
            fs::write(
                &source_path,
                format!(
                    "import styles from \"./Style{style_index:03}.module.css\";\nexport const Component{index:03} = () => <div className={{styles.item{style_index:03}}} />;\n"
                ),
            )
            .map_err(|error| format!("exact-match source fixture should be writable: {error}"))?;
            source_paths.push(source_path);
        }

        omena_query::reset_omena_resolver_style_identity_cache_for_test();
        let control_started = std::time::Instant::now();
        let control = workspace_style_diagnostics_summaries_unthreaded_for_test(
            style_paths.as_slice(),
            source_paths.as_slice(),
            &[],
        )?;
        let control_elapsed = control_started.elapsed();
        let control_build_count =
            omena_query::omena_resolver_style_identity_index_build_count_for_test();
        let control_work_count =
            omena_query::omena_resolver_style_identity_index_build_work_count_for_test();

        omena_query::reset_omena_resolver_style_identity_cache_for_test();
        let threaded_started = std::time::Instant::now();
        let threaded = workspace_style_diagnostics_summaries(
            style_paths.as_slice(),
            source_paths.as_slice(),
            &[],
        )?;
        let threaded_elapsed = threaded_started.elapsed();
        let threaded_build_count =
            omena_query::omena_resolver_style_identity_index_build_count_for_test();
        let threaded_work_count =
            omena_query::omena_resolver_style_identity_index_build_work_count_for_test();
        cleanup_dir(&root);

        let control_bytes = serde_json::to_vec(&control)
            .map_err(|error| format!("exact-match control should serialize: {error}"))?;
        let threaded_bytes = serde_json::to_vec(&threaded)
            .map_err(|error| format!("exact-match threaded result should serialize: {error}"))?;
        assert_eq!(threaded.len(), STYLE_COUNT);
        assert_eq!(
            threaded_bytes, control_bytes,
            "sharing the resolver identity index must preserve exact-match diagnostics bytes"
        );
        eprintln!(
            "resolver-identity-index-exact-match styles={STYLE_COUNT} sources={SOURCE_COUNT} control_builds={control_build_count} control_work={control_work_count} control_ms={} threaded_builds={threaded_build_count} threaded_work={threaded_work_count} threaded_ms={}",
            control_elapsed.as_millis(),
            threaded_elapsed.as_millis(),
        );
        assert_eq!(
            control_build_count, 0,
            "the exact in-graph control must not build a fallback identity index; work={control_work_count}"
        );
        assert_eq!(
            threaded_build_count, 1,
            "workspace style diagnostics must construct one shared identity index for an exact-match S×D corpus; work={threaded_work_count}"
        );
        assert_eq!(
            threaded_work_count,
            STYLE_COUNT * 2,
            "the single shared index must visit each available and disk identity exactly once"
        );
        Ok(())
    })
}

#[test]
fn dynamic_classname_diagnostics_command_exposes_k_bound_branch_merge() -> Result<(), String> {
    let zero_path = temp_path("dynamic-classname-zero.json");
    let two_path = temp_path("dynamic-classname-two.json");
    fs::write(&zero_path, dynamic_classname_diagnostics_input_json(0))
        .map_err(|error| format!("zero-CFA fixture input should be writable: {error}"))?;
    fs::write(&two_path, dynamic_classname_diagnostics_input_json(2))
        .map_err(|error| format!("two-CFA fixture input should be writable: {error}"))?;

    let zero_cfa = dynamic_classname_diagnostics_summary(&zero_path)?;
    let two_cfa = dynamic_classname_diagnostics_summary(&two_path)?;

    assert_eq!(zero_cfa.product, "omena-query.diagnostics-for-file");
    assert_eq!(two_cfa.product, "omena-query.diagnostics-for-file");
    assert!(
        zero_cfa
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "noImpossibleSelector"),
        "0-CFA branch merge must surface the joined out-of-universe selector"
    );
    assert!(
        zero_cfa.diagnostics.iter().all(|diagnostic| diagnostic
            .provenance
            .contains(&"omena-abstract-value.k-limited-call-site-flow")),
        "CLI diagnostics must preserve k-limited flow provenance"
    );
    assert!(
        zero_cfa.diagnostic_count > two_cfa.diagnostic_count,
        "raising k must reduce the branch-merge diagnostic set: zero={zero_cfa:?} two={two_cfa:?}"
    );
    assert!(
        two_cfa
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.range.start.line == 20),
        "2-CFA must keep diagnostics anchored to the secondary branch only"
    );

    let result = run(Cli {
        command: Command::DynamicClassnameDiagnostics {
            input_json: two_path.clone(),
            json: true,
        },
    });
    assert!(result.is_ok(), "{result:?}");

    cleanup(&zero_path);
    cleanup(&two_path);
    Ok(())
}

#[test]
fn source_diagnostics_command_uses_package_manifest_override_paths() -> Result<(), String> {
    let workspace_path = temp_dir("package-manifest-override");
    let source_dir = workspace_path.join("src");
    let package_dir = workspace_path.join("node_modules/@design/tokens");
    let style_dir = package_dir.join("dist");
    fs::create_dir_all(&source_dir)
        .map_err(|error| format!("fixture source dir should be writable: {error}"))?;
    fs::create_dir_all(&style_dir)
        .map_err(|error| format!("fixture package dir should be writable: {error}"))?;

    let source_path = source_dir.join("App.tsx");
    let style_path = style_dir.join("theme.module.css");
    let package_manifest_path = package_dir.join("package.json");
    fs::write(
        &source_path,
        r#"import bind from "classnames/bind";
import styles from "@design/tokens/theme.module.css";
const cx = bind.bind(styles);
export function App() {
  return <div className={cx("ghost")} />;
}
"#,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;
    fs::write(&style_path, ".chip {}\n")
        .map_err(|error| format!("fixture style should be writable: {error}"))?;
    fs::write(
        &package_manifest_path,
        r#"{"exports":{"./theme.module.css":{"style":"./dist/theme.module.css"}}}"#,
    )
    .map_err(|error| format!("fixture package manifest should be writable: {error}"))?;

    let summary = source_diagnostics_summary(
        path_string(&source_path),
        None,
        Some(source_path.clone()),
        vec![style_path.clone()],
        vec![package_manifest_path.clone()],
    )?;

    assert!(
        summary
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missing-module"),
        "{summary:?}"
    );
    let diagnostic = summary
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "missingStaticClass")
        .ok_or_else(|| format!("expected missingStaticClass diagnostic: {summary:?}"))?;
    let create_selector = diagnostic
        .create_selector
        .as_ref()
        .ok_or_else(|| format!("expected create selector action: {diagnostic:?}"))?;
    assert_eq!(create_selector.uri, path_string(&style_path));
    assert_eq!(create_selector.selector_name, "ghost");

    let result = run(Cli {
        command: Command::SourceDiagnostics {
            source_uri: path_string(&source_path),
            candidates_json: None,
            source_path: Some(source_path.clone()),
            source_paths: vec![style_path],
            package_manifest_paths: vec![package_manifest_path],
            json: true,
        },
    });
    assert!(result.is_ok(), "{result:?}");

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn source_diagnostics_command_resolves_package_import_array_fallbacks() -> Result<(), String> {
    let workspace_path = temp_dir("package-import-array-fallback");
    let source_dir = workspace_path.join("src");
    fs::create_dir_all(&source_dir)
        .map_err(|error| format!("fixture source dir should be writable: {error}"))?;

    let source_path = source_dir.join("App.tsx");
    let style_path = source_dir.join("theme.module.css");
    let package_manifest_path = workspace_path.join("package.json");
    fs::write(
        &source_path,
        r##"import bind from "classnames/bind";
import styles from "#theme.module.css";
const cx = bind.bind(styles);
export function App() {
  return <div className={cx("ghost")} />;
}
"##,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;
    fs::write(&style_path, ".chip {}\n")
        .map_err(|error| format!("fixture style should be writable: {error}"))?;
    fs::write(
        &package_manifest_path,
        r##"{"imports":{"#theme.module.css":[{"node":"./src/theme.js"},{"style":"./src/theme.module.css"}]}}"##,
    )
    .map_err(|error| format!("fixture package manifest should be writable: {error}"))?;

    let summary = source_diagnostics_summary(
        path_string(&source_path),
        None,
        Some(source_path.clone()),
        vec![style_path.clone()],
        vec![package_manifest_path.clone()],
    )?;

    assert!(
        summary
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missing-module"),
        "{summary:?}"
    );
    let diagnostic = summary
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "missingStaticClass")
        .ok_or_else(|| format!("expected missingStaticClass diagnostic: {summary:?}"))?;
    let create_selector = diagnostic
        .create_selector
        .as_ref()
        .ok_or_else(|| format!("expected create selector action: {diagnostic:?}"))?;
    assert_eq!(create_selector.uri, path_string(&style_path));
    assert_eq!(create_selector.selector_name, "ghost");

    let result = run(Cli {
        command: Command::SourceDiagnostics {
            source_uri: path_string(&source_path),
            candidates_json: None,
            source_path: Some(source_path.clone()),
            source_paths: vec![style_path],
            package_manifest_paths: vec![package_manifest_path],
            json: true,
        },
    });
    assert!(result.is_ok(), "{result:?}");

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn workspace_alias_resolution_surface_uses_project_config() -> Result<(), String> {
    let examples_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples");
    let source_path = examples_root.join("src/scenarios/11-ts-path/TsPathScenario.tsx");
    let style_path = examples_root.join("src/scenarios/11-ts-path/TsPath.module.scss");
    let summary = source_diagnostics_summary(
        path_string(&source_path),
        None,
        Some(source_path.clone()),
        vec![style_path],
        Vec::new(),
    )?;

    assert!(
        summary
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missing-module"),
        "existing tsconfig alias target was reported missing: {summary:?}"
    );

    let control_root = temp_dir("missing-tsconfig-alias-control");
    let control_source_dir = control_root.join("src");
    fs::create_dir_all(&control_source_dir)
        .map_err(|error| format!("control source dir should be writable: {error}"))?;
    fs::write(
        control_root.join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@styles/*":["src/styles/*"]}}}"#,
    )
    .map_err(|error| format!("control tsconfig should be writable: {error}"))?;
    let control_source_path = control_source_dir.join("App.tsx");
    fs::write(
        &control_source_path,
        r#"import styles from "@styles/Missing.module.scss";
export const app = <div className={styles.root} />;"#,
    )
    .map_err(|error| format!("control source should be writable: {error}"))?;

    let control = source_diagnostics_summary(
        path_string(&control_source_path),
        None,
        Some(control_source_path),
        Vec::new(),
        Vec::new(),
    )?;
    assert!(
        control
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missing-module"),
        "genuinely absent alias target must remain diagnostic: {control:?}"
    );

    cleanup_dir(&control_root);
    Ok(())
}

#[test]
fn perceptual_check_command_emits_exact_color_wcag_witness_from_query_facts() -> Result<(), String>
{
    let source_path = temp_path("perceptual.module.css");
    fs::write(
        &source_path,
        ":root { --fg: #000; }\n.button { color: #000 !IMPORTANT; background: #fff; border-color: var(--fg); }\n",
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let report = perceptual_check_summary(&source_path)?;
    assert_eq!(report.product, "omena-cli.perceptual-check");
    assert_eq!(report.claim_level, "fixtureWitnessExactColorWcagContrast");
    assert_eq!(report.command, "perceptual-check");
    assert!(report.json_schema_ready);
    assert!(report.downstream_tool_scaffold_ready);
    assert!(report.consumes_omena_facts);
    assert_eq!(report.selector_count, 1);
    assert_eq!(report.custom_property_declaration_count, 1);
    assert_eq!(report.custom_property_reference_count, 1);
    assert!(report.wcag_algorithm_ready);
    assert_eq!(report.wcag_exact_color_contrast_bound_count, 1);
    let contrast = &report.wcag_exact_color_contrast_bounds[0];
    assert_eq!(
        contrast.product,
        "omena-cli.perceptual-check.wcag-exact-color-contrast"
    );
    assert_eq!(contrast.feature_gate, "wcag-exact-color-contrast-v0");
    assert_eq!(contrast.claim_level, "fixtureWitnessExactColorWcagContrast");
    assert_eq!(contrast.selector_name, "button");
    assert_eq!(contrast.foreground, "#000");
    assert_eq!(contrast.background, "#fff");
    assert!(contrast.passes_aa_normal_text);
    assert!(!contrast.public_safety_claim_ready);
    assert!(contrast.contrast_ratio >= 21.0);
    assert!(!report.apca_algorithm_ready);
    assert!(!report.oklab_perceptual_operator_ready);
    assert!(!report.full_perceptual_algorithm_ready);
    assert!(!report.public_safety_claim_ready);
    assert!(
        report
            .fact_source_products
            .contains(&"omena-query.style-document-summary")
    );
    assert!(
        report
            .fact_source_products
            .contains(&"omena-query.consumer-check-style-source")
    );

    let result = run(Cli {
        command: Command::PerceptualCheck {
            path: source_path.clone(),
            json: true,
        },
    });
    assert!(result.is_ok(), "{result:?}");

    cleanup(&source_path);
    Ok(())
}

#[test]
fn perceptual_check_help_is_available() {
    let help = Cli::command().render_long_help().to_string();
    assert!(help.contains("perceptual-check"));
    assert!(help.contains("downstream perceptual-check JSON"));
}

#[test]
fn lock_verify_frozen_passes_for_matching_sif_artifact() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-pass");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?,
    ]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: None,
                json: true,
            }),
        },
    });

    assert!(result.is_ok(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_frozen_fails_for_changed_sif_artifact() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-fail");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let locked_sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    let changed_sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: blue !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&changed_sif)
            .map_err(|error| format!("fixture changed SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture changed SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &locked_sif)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?,
    ]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: None,
                json: true,
            }),
        },
    });

    assert!(result.is_err(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_frozen_fails_for_invalid_recorded_attestation() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-invalid-attestation");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;

    let reference = "sif/design-system.sigstore.json".to_string();
    let mut entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    entry.trust_tier = omena_sif::OmenaSifTrustTierV1::T3;
    entry
        .attestation_references
        .push(omena_sif::OmenaSifAttestationReferenceV1 {
            kind: "sigstore-bundle".to_string(),
            reference: reference.clone(),
        });
    let mut statement = cli_fixture_provenance_statement();
    statement.subject_digests = vec![omena_sif::OmenaSifAttestationSubjectDigestV1 {
        name: "sif/design-system.sif.json".to_string(),
        algorithm: "sha256".to_string(),
        digest: "abcdef0123456789".to_string(),
    }];
    entry
        .attestation_verifications
        .push(omena_sif::OmenaSifAttestationVerificationV1 {
            kind: "omena-toolchain.sigstore".to_string(),
            reference,
            verifier: "sigstore-verify".to_string(),
            verified_trust_tier: omena_sif::OmenaSifTrustTierV1::T3,
            verified_tlog_integrated_time: Some(1_764_787_003),
            sigstore_verification_policy: Some(
                omena_sif::OmenaSifSigstoreVerificationPolicyV1 {
                    trusted_root: "sigstore-production-trusted-root".to_string(),
                    transparency_log: true,
                    timestamp: true,
                    certificate_chain: true,
                    signed_certificate_timestamp: true,
                },
            ),
            certificate_issuer: Some(
                omena_sif::OMENA_SIF_PUBLISHED_ATTESTATION_CERTIFICATE_ISSUER_V1.to_string(),
            ),
            certificate_identity: Some("https://github.com/omenien/omena-css/.github/workflows/sif-keyless-attestation.yml@refs/heads/master".to_string()),
            attestation_statement: Some(statement),
        });
    let lock = omena_sif::OmenaLockV1::new(vec![entry]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: None,
                json: true,
            }),
        },
    });

    assert!(result.is_err(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_t3_rejects_identityless_toolchain_evidence() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-t3-policy");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let reference = "sif/design-system.sigstore.json".to_string();
    let mut entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    entry.trust_tier = omena_sif::OmenaSifTrustTierV1::T3;
    entry
        .attestation_references
        .push(omena_sif::OmenaSifAttestationReferenceV1 {
            kind: "sigstore-bundle".to_string(),
            reference: reference.clone(),
        });
    entry
        .attestation_verifications
        .push(omena_sif::OmenaSifAttestationVerificationV1 {
            kind: "omena-toolchain.sigstore".to_string(),
            reference,
            verifier: "sigstore-verify".to_string(),
            verified_trust_tier: omena_sif::OmenaSifTrustTierV1::T3,
            verified_tlog_integrated_time: Some(1_764_787_003),
            sigstore_verification_policy: Some(omena_sif::OmenaSifSigstoreVerificationPolicyV1 {
                trusted_root: "sigstore-production-trusted-root".to_string(),
                transparency_log: true,
                timestamp: true,
                certificate_chain: true,
                signed_certificate_timestamp: true,
            }),
            certificate_issuer: Some(
                omena_sif::OMENA_SIF_PUBLISHED_ATTESTATION_CERTIFICATE_ISSUER_V1.to_string(),
            ),
            certificate_identity: None,
            attestation_statement: None,
        });
    let lock = omena_sif::OmenaLockV1::new(vec![entry]);
    let issues = collect_lock_trust_tier_issues(&lock, omena_sif::OmenaSifTrustTierV1::T3);
    assert_eq!(issues.len(), 1, "{issues:?}");
    assert_eq!(issues[0].code, "attestationVerificationInvalid");
    assert!(
        issues[0]
            .message
            .contains("requires the Omena keyless workflow certificateIdentity"),
        "{issues:?}"
    );
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: Some("t3".to_string()),
                json: true,
            }),
        },
    });

    assert!(result.is_err(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_frozen_fails_for_source_referenced_missing_sif_entry() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-source-coverage");
    fs::create_dir_all(&workspace_path)
        .map_err(|error| format!("fixture workspace should be writable: {error}"))?;
    let source_path = workspace_path.join("app.module.scss");
    let lockfile_path = workspace_path.join("omena.lock");
    fs::write(
        &source_path,
        r#"@use "sass:map";
@use "./local" as local;
@use "design-system/tokens" as tokens;
.button { color: tokens.$color; }"#,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(Vec::new());
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let issues = collect_lock_source_coverage_issues(&lock, std::slice::from_ref(&source_path))?;
    assert_eq!(issues.len(), 1, "{issues:?}");
    assert_eq!(issues[0].code, "sourceSifMissingFromLock");
    assert_eq!(issues[0].canonical_url, "design-system/tokens");

    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: vec![source_path.clone()],
                frozen: true,
                tier: None,
                json: true,
            }),
        },
    });

    assert!(result.is_err(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_update_authors_lock_then_verify_frozen_passes_and_fails_when_tampered() -> Result<(), String>
{
    let workspace_path = temp_dir("lock-update-roundtrip");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;

    // Author the lock from the generated SIF artifact.
    let update_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Update {
                package: None,
                lockfile: lockfile_path.clone(),
                sif_paths: vec![sif_path.clone()],
                json: true,
            }),
        },
    });
    assert!(update_result.is_ok(), "{update_result:?}");
    assert!(
        lockfile_path.exists(),
        "lock update should write {}",
        path_string(&lockfile_path)
    );

    // The authored lock verifies against the SIF it was produced from.
    let verify_pass = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path.clone(),
                source_paths: Vec::new(),
                frozen: true,
                tier: None,
                json: true,
            }),
        },
    });
    assert!(verify_pass.is_ok(), "{verify_pass:?}");

    // Tampering the SIF on disk must break frozen verification (over-correction guard).
    let tampered = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: blue !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&tampered)
            .map_err(|error| format!("tampered SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("tampered SIF should be writable: {error}"))?;
    let verify_fail = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: None,
                json: true,
            }),
        },
    });
    assert!(verify_fail.is_err(), "{verify_fail:?}");

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_fetch_provenance_records_npm_attestations_without_t2_verification() -> Result<(), String> {
    let workspace_path = temp_dir("lock-fetch-provenance");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let metadata_path = workspace_path.join("npm-metadata.json");
    let absent_metadata_path = workspace_path.join("npm-metadata-absent.json");
    let mismatched_metadata_path = workspace_path.join("npm-metadata-mismatched.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?,
    ]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;
    fs::write(
        &metadata_path,
        serde_json::json!({
            "name": "design-system",
            "version": "1.0.0",
            "dist": {
                "attestations": {
                    "provenance": "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance"
                }
            }
        })
        .to_string(),
    )
    .map_err(|error| format!("fixture metadata should be writable: {error}"))?;
    fs::write(
        &absent_metadata_path,
        serde_json::json!({
            "name": "design-system",
            "version": "1.0.0",
            "dist": {
                "tarball": "https://registry.npmjs.org/design-system/-/design-system-1.0.0.tgz"
            }
        })
        .to_string(),
    )
    .map_err(|error| format!("absent fixture metadata should be writable: {error}"))?;
    fs::write(
        &mismatched_metadata_path,
        serde_json::json!({
            "name": "design-system",
            "version": "1.0.0",
            "dist": {
                "attestations": {
                    "provenance": "https://registry.npmjs.org/-/npm/v1/attestations/lookalike@1.0.0/provenance"
                }
            }
        })
        .to_string(),
    )
    .map_err(|error| format!("mismatched fixture metadata should be writable: {error}"))?;

    let verify_t2_before = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path.clone(),
                source_paths: Vec::new(),
                frozen: true,
                tier: Some("t2".to_string()),
                json: true,
            }),
        },
    });
    assert!(verify_t2_before.is_err(), "{verify_t2_before:?}");

    let absent_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::FetchProvenance {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                npm_metadata: absent_metadata_path,
                json: true,
            }),
        },
    });
    let absent_error = match absent_result {
        Err(error) => error,
        Ok(()) => return Err("absent provenance unexpectedly upgraded trust".to_string()),
    };
    assert!(
        absent_error.contains("does not contain dist.attestations.provenance"),
        "{absent_error}"
    );
    let unchanged_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
        .map_err(|error| format!("unchanged fixture lock should parse: {error}"))?;
    assert_eq!(
        unchanged_lock.entries[0].trust_tier,
        omena_sif::OmenaSifTrustTierV1::T1
    );
    assert!(unchanged_lock.entries[0].attestation_references.is_empty());

    let mismatched_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::FetchProvenance {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                npm_metadata: mismatched_metadata_path,
                json: true,
            }),
        },
    });
    let mismatched_error = match mismatched_result {
        Err(error) => error,
        Ok(()) => return Err("mismatched provenance subject was accepted".to_string()),
    };
    assert!(
        mismatched_error.contains("provenanceSubjectMismatch"),
        "{mismatched_error}"
    );

    let fetch_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::FetchProvenance {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                npm_metadata: metadata_path,
                json: true,
            }),
        },
    });
    assert!(fetch_result.is_ok(), "{fetch_result:?}");

    let refreshed_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
        .map_err(|error| format!("fixture lock should parse: {error}"))?;
    assert_eq!(
        refreshed_lock.entries[0].trust_tier,
        omena_sif::OmenaSifTrustTierV1::T1
    );
    assert_eq!(refreshed_lock.entries[0].attestation_references.len(), 1);
    assert!(
        refreshed_lock.entries[0]
            .attestation_verifications
            .is_empty()
    );

    let verify_t2_after = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: Some("t2".to_string()),
                json: true,
            }),
        },
    });
    assert!(verify_t2_after.is_err(), "{verify_t2_after:?}");

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_record_verification_rejects_elevated_local_report() -> Result<(), String> {
    let workspace_path = temp_dir("lock-record-verification");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let verification_path = workspace_path.join("attestation-verification.json");
    let bad_verification_path = workspace_path.join("bad-attestation-verification.json");
    let bad_t3_verification_path = workspace_path.join("bad-t3-attestation-verification.json");
    let metadata_path = workspace_path.join("npm-metadata.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    fs::write(
        &verification_path,
        serde_json::json!({
            "schemaVersion": "1",
            "product": "omena-sif.attestation-verification-report",
            "verified": true,
            "kind": "npm-provenance.sigstore",
            "reference": "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance",
            "verifier": "offline-sigstore-verifier",
            "verifiedTrustTier": "t2",
            "verifiedTlogIntegratedTime": 1717000000,
            "sigstoreVerificationPolicy": {
                "trustedRoot": "sigstore-production-trusted-root",
                "transparencyLog": true,
                "timestamp": true,
                "certificateChain": true,
                "signedCertificateTimestamp": true
            },
            "certificateIssuer": "https://token.actions.githubusercontent.com",
            "certificateIdentity": "https://github.com/omenien/omena-css/.github/workflows/release.yml@refs/tags/v1.0.0",
            "attestationStatement": {
                "statementType": "https://in-toto.io/Statement/v1",
                "predicateType": "https://slsa.dev/provenance/v1",
                "sourceRepository": "https://github.com/omenien/omena-css",
                "sourceRef": "refs/tags/v1.0.0",
                "sourceCommit": "abcdef0123456789",
                "builderId": "https://github.com/actions/runner/github-hosted",
                "buildType": "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1",
                "subjectNames": [
                    "pkg:npm/@omenacss/omena-css@1.0.0"
                ],
                "subjectDigests": [
                    {
                        "name": "pkg:npm/@omenacss/omena-css@1.0.0",
                        "algorithm": "sha256",
                        "digest": "0123456789abcdef"
                    }
                ]
            },
            "subjectCanonicalUrl": entry.canonical_url.as_str(),
            "subjectSifHash": entry.sif_hash.as_str()
        })
        .to_string(),
    )
    .map_err(|error| format!("fixture verification should be writable: {error}"))?;
    fs::write(
        &bad_verification_path,
        serde_json::json!({
            "schemaVersion": "0",
            "product": "omena-sif.attestation-verification-report",
            "verified": true,
            "kind": "npm-provenance.sigstore",
            "reference": "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance",
            "verifier": "offline-sigstore-verifier",
            "verifiedTrustTier": "t2",
            "verifiedTlogIntegratedTime": 1717000000,
            "sigstoreVerificationPolicy": {
                "trustedRoot": "sigstore-production-trusted-root",
                "transparencyLog": true,
                "timestamp": true,
                "certificateChain": true,
                "signedCertificateTimestamp": true
            },
            "certificateIssuer": "https://token.actions.githubusercontent.com",
            "subjectCanonicalUrl": entry.canonical_url.as_str(),
            "subjectSifHash": entry.sif_hash.as_str()
        })
        .to_string(),
    )
    .map_err(|error| format!("fixture bad verification should be writable: {error}"))?;
    fs::write(
        &bad_t3_verification_path,
        serde_json::json!({
            "schemaVersion": "1",
            "product": "omena-sif.attestation-verification-report",
            "verified": true,
            "kind": "npm-provenance.sigstore",
            "reference": "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance",
            "verifier": "offline-sigstore-verifier",
            "verifiedTrustTier": "t3",
            "verifiedTlogIntegratedTime": 1717000000,
            "sigstoreVerificationPolicy": {
                "trustedRoot": "sigstore-production-trusted-root",
                "transparencyLog": true,
                "timestamp": true,
                "certificateChain": true,
                "signedCertificateTimestamp": true
            },
            "certificateIssuer": "https://token.actions.githubusercontent.com",
            "subjectCanonicalUrl": entry.canonical_url.as_str(),
            "subjectSifHash": entry.sif_hash.as_str()
        })
        .to_string(),
    )
    .map_err(|error| format!("fixture bad T3 verification should be writable: {error}"))?;
    fs::write(
        &metadata_path,
        serde_json::json!({
            "name": "design-system",
            "version": "1.0.0",
            "dist": {
                "attestations": {
                    "provenance": "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance"
                }
            }
        })
        .to_string(),
    )
    .map_err(|error| format!("fixture metadata should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![entry]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let verify_t2_before = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path.clone(),
                source_paths: Vec::new(),
                frozen: true,
                tier: Some("t2".to_string()),
                json: true,
            }),
        },
    });
    assert!(verify_t2_before.is_err(), "{verify_t2_before:?}");

    let rejected_record_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::RecordVerification {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                verification: bad_verification_path,
                artifact: None,
                json: true,
            }),
        },
    });
    assert!(
        rejected_record_result.is_err(),
        "{rejected_record_result:?}"
    );

    let unreferenced_record_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::RecordVerification {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                verification: verification_path.clone(),
                artifact: None,
                json: true,
            }),
        },
    });
    assert!(
        unreferenced_record_result.is_err(),
        "{unreferenced_record_result:?}"
    );

    let fetch_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::FetchProvenance {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                npm_metadata: metadata_path,
                json: true,
            }),
        },
    });
    assert!(fetch_result.is_ok(), "{fetch_result:?}");

    let rejected_t3_record_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::RecordVerification {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                verification: bad_t3_verification_path,
                artifact: None,
                json: true,
            }),
        },
    });
    assert!(
        rejected_t3_record_result
            .as_ref()
            .is_err_and(|error| error.contains("cannot establish elevated trust")),
        "{rejected_t3_record_result:?}"
    );

    let record_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::RecordVerification {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                verification: verification_path,
                artifact: None,
                json: true,
            }),
        },
    });
    assert!(
        record_result.as_ref().is_err_and(
            |error| error.contains("cannot establish elevated trust from a local report")
        ),
        "{record_result:?}"
    );

    let refreshed_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
        .map_err(|error| format!("fixture lock should parse: {error}"))?;
    assert_eq!(
        refreshed_lock.entries[0].trust_tier,
        omena_sif::OmenaSifTrustTierV1::T1
    );
    assert!(
        refreshed_lock.entries[0]
            .attestation_verifications
            .is_empty()
    );

    let verify_t2_after = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: Some("t2".to_string()),
                json: true,
            }),
        },
    });
    assert!(verify_t2_after.is_err(), "{verify_t2_after:?}");

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_record_verification_t3_cannot_bypass_bundle_verification() -> Result<(), String> {
    let workspace_path = temp_dir("lock-record-verification-t3-artifact");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let verification_path = workspace_path.join("t3-attestation-verification.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let reference = "sif/design-system.sigstore.json";
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    let sif_source = omena_sif::write_omena_sif_json_v1(&sif)
        .map_err(|error| format!("fixture SIF should serialize: {error}"))?;
    fs::write(&sif_path, &sif_source)
        .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let mut entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    entry
        .attestation_references
        .push(omena_sif::OmenaSifAttestationReferenceV1 {
            kind: "sigstore-bundle".to_string(),
            reference: reference.to_string(),
        });
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&omena_sif::OmenaLockV1::new(vec![entry.clone()]))
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;
    fs::write(
        &verification_path,
        serde_json::json!({
            "schemaVersion": "1",
            "product": "omena-sif.attestation-verification-report",
            "verified": true,
            "kind": "omena-toolchain.sigstore",
            "reference": reference,
            "verifier": "offline-sigstore-verifier",
            "verifiedTrustTier": "t3",
            "verifiedTlogIntegratedTime": 1717000000,
            "sigstoreVerificationPolicy": {
                "trustedRoot": "sigstore-production-trusted-root",
                "transparencyLog": true,
                "timestamp": true,
                "certificateChain": true,
                "signedCertificateTimestamp": true
            },
            "certificateIssuer": "https://token.actions.githubusercontent.com",
            "certificateIdentity": "https://github.com/omenien/omena-css/.github/workflows/sif-keyless-attestation.yml@refs/heads/master",
            "attestationStatement": {
                "statementType": "https://in-toto.io/Statement/v1",
                "predicateType": "https://slsa.dev/provenance/v1",
                "sourceRepository": "https://github.com/omenien/omena-css",
                "sourceRef": "refs/heads/master",
                "sourceCommit": "abcdef0123456789",
                "builderId": "https://github.com/actions/runner/github-hosted",
                "buildType": "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1",
                "subjectNames": [
                    entry.sif_path.as_str()
                ],
                "subjectDigests": [
                    {
                        "name": entry.sif_path.as_str(),
                        "algorithm": "sha256",
                        "digest": sha256_hex(sif_source.as_bytes())
                    }
                ]
            },
            "subjectCanonicalUrl": entry.canonical_url.as_str(),
            "subjectSifHash": entry.sif_hash.as_str()
        })
        .to_string(),
    )
    .map_err(|error| format!("fixture T3 verification should be writable: {error}"))?;

    let missing_artifact_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::RecordVerification {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                verification: verification_path.clone(),
                artifact: None,
                json: true,
            }),
        },
    });
    assert!(
        missing_artifact_result
            .as_ref()
            .is_err_and(|error| error.contains("cannot establish elevated trust")),
        "{missing_artifact_result:?}"
    );

    let record_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::RecordVerification {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                verification: verification_path,
                artifact: Some(sif_path),
                json: true,
            }),
        },
    });
    assert!(
        record_result
            .as_ref()
            .is_err_and(|error| error.contains("cannot establish elevated trust")),
        "{record_result:?}"
    );

    let verify_t3_after = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: Some("t3".to_string()),
                json: true,
            }),
        },
    });
    assert!(verify_t3_after.is_err(), "{verify_t3_after:?}");

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_attestation_rejects_unverified_sigstore_bundle() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-attestation");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let metadata_path = workspace_path.join("npm-metadata.json");
    let artifact_path = workspace_path.join("artifact.tgz");
    let bundle_path = workspace_path.join("bundle.sigstore.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let provenance_reference =
        "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance";
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?,
    ]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;
    fs::write(
        &metadata_path,
        serde_json::json!({
            "name": "design-system",
            "version": "1.0.0",
            "dist": {
                "attestations": {
                    "provenance": provenance_reference
                }
            }
        })
        .to_string(),
    )
    .map_err(|error| format!("fixture metadata should be writable: {error}"))?;
    fs::write(&artifact_path, b"package bytes")
        .map_err(|error| format!("fixture artifact should be writable: {error}"))?;
    fs::write(&bundle_path, "{}")
        .map_err(|error| format!("fixture bundle should be writable: {error}"))?;

    let fetch_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::FetchProvenance {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                npm_metadata: metadata_path,
                json: true,
            }),
        },
    });
    assert!(fetch_result.is_ok(), "{fetch_result:?}");

    let verify_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::VerifyAttestation {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                artifact: artifact_path,
                bundle: bundle_path,
                reference: provenance_reference.to_string(),
                kind: "npm-provenance.sigstore".to_string(),
                verified_tier: "t2".to_string(),
                identity: None,
                issuer: "https://token.actions.githubusercontent.com".to_string(),
                statement_type: None,
                statement_predicate_type: None,
                statement_source_repository: None,
                statement_source_ref: None,
                statement_source_commit: None,
                statement_builder_id: None,
                statement_build_type: None,
                statement_subject_names: Vec::new(),
                statement_subject_digests: Vec::new(),
                json: true,
            }),
        },
    });
    assert!(verify_result.is_err(), "{verify_result:?}");

    let refreshed_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
        .map_err(|error| format!("fixture lock should parse: {error}"))?;
    assert_eq!(
        refreshed_lock.entries[0].trust_tier,
        omena_sif::OmenaSifTrustTierV1::T1
    );
    assert_eq!(refreshed_lock.entries[0].attestation_references.len(), 1);
    assert!(
        refreshed_lock.entries[0]
            .attestation_verifications
            .is_empty()
    );

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn third_party_sigstore_bundle_remains_advisory_at_t1() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-attestation-sigstore");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let artifact_path = workspace_path.join("cosign-v3-blob.txt");
    let bundle_path = workspace_path.join("cosign-v3-blob.sigstore.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let provenance_reference = "sif/cosign-v3-blob.sigstore.json";
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let mut entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    entry
        .attestation_references
        .push(omena_sif::OmenaSifAttestationReferenceV1 {
            kind: "sigstore-bundle".to_string(),
            reference: provenance_reference.to_string(),
        });
    let lock = omena_sif::OmenaLockV1::new(vec![entry]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;
    fs::write(
        &artifact_path,
        include_bytes!("../fixtures/sigstore/cosign-v3-blob.txt"),
    )
    .map_err(|error| format!("fixture artifact should be writable: {error}"))?;
    fs::write(
        &bundle_path,
        include_str!("../fixtures/sigstore/cosign-v3-blob.sigstore.json"),
    )
    .map_err(|error| format!("fixture bundle should be writable: {error}"))?;

    let verify_attestation_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::VerifyAttestation {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                artifact: artifact_path,
                bundle: bundle_path,
                reference: provenance_reference.to_string(),
                kind: "sigstore-bundle".to_string(),
                verified_tier: "t2".to_string(),
                identity: None,
                issuer: "https://github.com/login/oauth".to_string(),
                statement_type: None,
                statement_predicate_type: None,
                statement_source_repository: None,
                statement_source_ref: None,
                statement_source_commit: None,
                statement_builder_id: None,
                statement_build_type: None,
                statement_subject_names: Vec::new(),
                statement_subject_digests: Vec::new(),
                json: true,
            }),
        },
    });
    assert!(
        verify_attestation_result
            .as_ref()
            .is_err_and(|error| error.contains("requires --kind omena-toolchain.*")),
        "{verify_attestation_result:?}"
    );

    let refreshed_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
        .map_err(|error| format!("fixture lock should parse: {error}"))?;
    assert_eq!(
        refreshed_lock.entries[0].trust_tier,
        omena_sif::OmenaSifTrustTierV1::T1
    );
    assert!(
        refreshed_lock.entries[0]
            .attestation_verifications
            .is_empty()
    );

    let verify_t2_after = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: Some("t2".to_string()),
                json: true,
            }),
        },
    });
    assert!(verify_t2_after.is_err(), "{verify_t2_after:?}");

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_attestation_publishes_bundle_backed_shard_verdict() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-attestation-keyless-sif");
    fs::create_dir_all(workspace_path.as_path())
        .map_err(|error| format!("fixture workspace should be writable: {error}"))?;
    let sif_path = workspace_path.join("published-sif-attestation.sif.json");
    let bundle_path = workspace_path.join("published-sif-attestation.sigstore.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let user_reference = "fixture:published-sif-attestation";
    let sif_source =
        include_str!("../../omena-bridge/tests/fixtures/published-sif-attestation.sif.json")
            .trim_end();
    let bundle_source =
        include_bytes!("../../omena-bridge/tests/fixtures/published-sif-attestation.sigstore.json");
    fs::write(sif_path.as_path(), sif_source)
        .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    fs::write(bundle_path.as_path(), bundle_source)
        .map_err(|error| format!("fixture bundle should be writable: {error}"))?;
    let sif = omena_sif::read_omena_sif_json_v1(sif_source)
        .map_err(|error| format!("fixture SIF should parse: {error}"))?;
    let mut entry =
        omena_sif::build_omena_lock_sif_entry_v1("published-sif-attestation.sif.json", &sif)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    entry
        .attestation_references
        .push(omena_sif::OmenaSifAttestationReferenceV1 {
            kind: "sigstore-bundle".to_string(),
            reference: user_reference.to_string(),
        });
    fs::write(
        lockfile_path.as_path(),
        omena_sif::write_omena_lock_json_v1(&omena_sif::OmenaLockV1::new(vec![entry]))
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::VerifyAttestation {
                package: sif.canonical_url.clone(),
                lockfile: lockfile_path.clone(),
                artifact: sif_path,
                bundle: bundle_path,
                reference: user_reference.to_string(),
                kind: "omena-toolchain.sigstore".to_string(),
                verified_tier: "t3".to_string(),
                identity: Some(
                    "https://github.com/omenien/omena-css/.github/workflows/sif-keyless-attestation.yml@refs/heads/master"
                        .to_string(),
                ),
                issuer: "https://token.actions.githubusercontent.com".to_string(),
                statement_type: None,
                statement_predicate_type: None,
                statement_source_repository: None,
                statement_source_ref: None,
                statement_source_commit: None,
                statement_builder_id: None,
                statement_build_type: None,
                statement_subject_names: Vec::new(),
                statement_subject_digests: Vec::new(),
                json: true,
            }),
        },
    });
    assert!(result.is_ok(), "{result:?}");

    let lock = read_omena_lock_json_v1(&read_source(lockfile_path.as_path())?)
        .map_err(|error| format!("updated lock should parse: {error}"))?;
    assert_eq!(
        lock.entries[0].trust_tier,
        omena_sif::OmenaSifTrustTierV1::T3
    );
    let verdict_address = omena_sif::compute_omena_sif_shard_recorded_verdict_address_v1(
        lock.entries[0].canonical_url.as_str(),
        &lock.entries[0].sif_hash,
    )
    .map_err(|error| format!("verdict address should compute: {error}"))?;
    let verdict_hex = verdict_address
        .as_str()
        .strip_prefix("blake3:")
        .ok_or_else(|| "fixture verdict address should be blake3".to_string())?;
    let verdict_dir = workspace_path
        .join(".cache")
        .join("omena")
        .join(omena_sif::OMENA_SIF_SHARD_VERDICT_DIR_V1);
    let verdict_source = read_source(&verdict_dir.join(format!("{verdict_hex}.json")))?;
    let verdict = omena_sif::read_omena_sif_shard_recorded_verdict_json_v1(verdict_source.as_str())
        .map_err(|error| format!("recorded verdict should parse: {error}"))?;
    assert_ne!(verdict.signature.reference, user_reference);
    assert!(verdict.signature.reference.starts_with("bundles-v1/"));
    let stored_bundle = fs::read(verdict_dir.join(verdict.signature.reference.as_str()))
        .map_err(|error| format!("recorded bundle should be readable: {error}"))?;
    assert_eq!(stored_bundle, bundle_source);

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_attestation_rejects_third_party_elevated_provenance() -> Result<(), String> {
    let workspace_path = temp_dir("lock-verify-attestation-provenance-statement");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let artifact_path = workspace_path.join("cosign-v3-blob.txt");
    let bundle_path = workspace_path.join("cosign-v3-blob.sigstore.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let provenance_reference =
        "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance";
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let mut entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    entry
        .attestation_references
        .push(omena_sif::OmenaSifAttestationReferenceV1 {
            kind: "npm-provenance.url".to_string(),
            reference: provenance_reference.to_string(),
        });
    let lock = omena_sif::OmenaLockV1::new(vec![entry]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;
    fs::write(
        &artifact_path,
        include_bytes!("../fixtures/sigstore/cosign-v3-blob.txt"),
    )
    .map_err(|error| format!("fixture artifact should be writable: {error}"))?;
    fs::write(
        &bundle_path,
        include_str!("../fixtures/sigstore/cosign-v3-blob.sigstore.json"),
    )
    .map_err(|error| format!("fixture bundle should be writable: {error}"))?;

    let verify_attestation_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::VerifyAttestation {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                artifact: artifact_path,
                bundle: bundle_path,
                reference: provenance_reference.to_string(),
                kind: "npm-provenance.sigstore".to_string(),
                verified_tier: "t2".to_string(),
                identity: None,
                issuer: "https://github.com/login/oauth".to_string(),
                statement_type: None,
                statement_predicate_type: None,
                statement_source_repository: None,
                statement_source_ref: None,
                statement_source_commit: None,
                statement_builder_id: None,
                statement_build_type: None,
                statement_subject_names: Vec::new(),
                statement_subject_digests: Vec::new(),
                json: true,
            }),
        },
    });
    assert!(
        verify_attestation_result
            .as_ref()
            .is_err_and(|error| error.contains("requires --kind omena-toolchain.*")),
        "{verify_attestation_result:?}"
    );
    let refreshed_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
        .map_err(|error| format!("fixture lock should parse: {error}"))?;
    assert_eq!(
        refreshed_lock.entries[0].trust_tier,
        omena_sif::OmenaSifTrustTierV1::T1
    );
    assert!(
        refreshed_lock.entries[0]
            .attestation_verifications
            .is_empty()
    );

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_attestation_t3_rejects_non_omena_oidc_root_before_artifact_use() -> Result<(), String>
{
    let workspace_path = temp_dir("lock-verify-attestation-t3-artifact-binding");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let metadata_path = workspace_path.join("npm-metadata.json");
    let artifact_path = workspace_path.join("cosign-v3-blob.txt");
    let bundle_path = workspace_path.join("cosign-v3-blob.sigstore.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let provenance_reference = "sif/cosign-v3-blob.sigstore.json";
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new(vec![
        omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?,
    ]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;
    fs::write(
        &metadata_path,
        serde_json::json!({
            "name": "design-system",
            "version": "1.0.0",
            "dist": {
                "attestations": {
                    "provenance": provenance_reference
                }
            }
        })
        .to_string(),
    )
    .map_err(|error| format!("fixture metadata should be writable: {error}"))?;
    fs::write(
        &artifact_path,
        include_bytes!("../fixtures/sigstore/cosign-v3-blob.txt"),
    )
    .map_err(|error| format!("fixture artifact should be writable: {error}"))?;
    fs::write(
        &bundle_path,
        include_str!("../fixtures/sigstore/cosign-v3-blob.sigstore.json"),
    )
    .map_err(|error| format!("fixture bundle should be writable: {error}"))?;

    let fetch_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::FetchProvenance {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                npm_metadata: metadata_path,
                json: true,
            }),
        },
    });
    assert!(fetch_result.is_ok(), "{fetch_result:?}");

    let verify_attestation_result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::VerifyAttestation {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                artifact: artifact_path,
                bundle: bundle_path,
                reference: provenance_reference.to_string(),
                kind: "omena-toolchain.sigstore".to_string(),
                verified_tier: "t3".to_string(),
                identity: Some("w.vollprecht@gmail.com".to_string()),
                issuer: "https://github.com/login/oauth".to_string(),
                statement_type: None,
                statement_predicate_type: None,
                statement_source_repository: None,
                statement_source_ref: None,
                statement_source_commit: None,
                statement_builder_id: None,
                statement_build_type: None,
                statement_subject_names: Vec::new(),
                statement_subject_digests: Vec::new(),
                json: true,
            }),
        },
    });
    let error = verify_attestation_result
        .err()
        .unwrap_or_else(|| "verify-attestation unexpectedly succeeded".to_string());
    assert!(
        error.contains("requires the Omena CI OIDC issuer"),
        "{error}"
    );

    let refreshed_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
        .map_err(|error| format!("fixture lock should parse: {error}"))?;
    assert_eq!(
        refreshed_lock.entries[0].trust_tier,
        omena_sif::OmenaSifTrustTierV1::T1
    );
    assert_eq!(refreshed_lock.entries[0].attestation_references.len(), 1);
    assert!(
        refreshed_lock.entries[0]
            .attestation_verifications
            .is_empty()
    );

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_verify_frozen_fails_when_lock_requires_future_omena() -> Result<(), String> {
    let workspace_path = temp_dir("lock-min-version");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let sif_path = sif_dir.join("design-system.sif.json");
    let lockfile_path = workspace_path.join("omena.lock");
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    fs::write(
        &sif_path,
        omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    let lock = omena_sif::OmenaLockV1::new_with_min_version(
        vec![
            omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
                .map_err(|error| format!("fixture lock entry should build: {error}"))?,
        ],
        Some("999.0.0".to_string()),
    );
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Verify {
                lockfile: lockfile_path,
                source_paths: Vec::new(),
                frozen: true,
                tier: None,
                json: true,
            }),
        },
    });

    assert!(result.is_err(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn lock_status_and_package_authoring_manage_entries_without_overwriting_others()
-> Result<(), String> {
    let workspace_path = temp_dir("lock-package-authoring");
    let sif_dir = workspace_path.join("sif");
    fs::create_dir_all(&sif_dir)
        .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
    let lockfile_path = workspace_path.join("omena.lock");
    let design_sif_path = sif_dir.join("design-system.sif.json");
    let palette_sif_path = sif_dir.join("palette.sif.json");
    let updated_design_sif_path = sif_dir.join("design-system-updated.sif.json");
    let design_sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red;")?;
    let palette_sif = cli_fixture_sif("pkg:palette/_colors.scss", b"$brand: blue;")?;
    let updated_design_sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: green;")?;
    for (path, sif) in [
        (&design_sif_path, &design_sif),
        (&palette_sif_path, &palette_sif),
        (&updated_design_sif_path, &updated_design_sif),
    ] {
        fs::write(
            path,
            omena_sif::write_omena_sif_json_v1(sif)
                .map_err(|error| format!("fixture SIF should serialize: {error}"))?,
        )
        .map_err(|error| format!("fixture SIF should be writable: {error}"))?;
    }

    let add_design = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Add {
                package: "design-system".to_string(),
                lockfile: lockfile_path.clone(),
                sif_paths: vec![design_sif_path.clone()],
                json: true,
            }),
        },
    });
    assert!(add_design.is_ok(), "{add_design:?}");

    let add_palette = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Add {
                package: "palette".to_string(),
                lockfile: lockfile_path.clone(),
                sif_paths: vec![palette_sif_path.clone()],
                json: true,
            }),
        },
    });
    assert!(add_palette.is_ok(), "{add_palette:?}");

    let update_design = run(Cli {
        command: Command::Lock {
            lockfile: PathBuf::from("omena.lock"),
            json: false,
            command: Some(LockCommand::Update {
                package: Some("design-system".to_string()),
                lockfile: lockfile_path.clone(),
                sif_paths: vec![updated_design_sif_path.clone()],
                json: true,
            }),
        },
    });
    assert!(update_design.is_ok(), "{update_design:?}");

    let lock = omena_sif::read_omena_lock_json_v1(
        &fs::read_to_string(&lockfile_path)
            .map_err(|error| format!("fixture lock should be readable: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should parse: {error}"))?;
    assert_eq!(lock.entries.len(), 2, "{lock:?}");
    assert!(
        lock.entries
            .iter()
            .any(|entry| entry.canonical_url == "pkg:palette/_colors.scss"),
        "{lock:?}"
    );
    let design_entry = lock
        .entries
        .iter()
        .find(|entry| entry.canonical_url == "pkg:design-system/_tokens.scss")
        .ok_or_else(|| format!("updated design-system entry should exist: {lock:?}"))?;
    assert!(
        design_entry
            .sif_path
            .ends_with("design-system-updated.sif.json"),
        "{design_entry:?}"
    );
    assert!(lock.omena_min_version.is_some(), "{lock:?}");

    let status = run(Cli {
        command: Command::Lock {
            lockfile: lockfile_path,
            json: true,
            command: None,
        },
    });
    assert!(status.is_ok(), "{status:?}");

    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn provenance_status_reports_reference_only_advisory_lock_metadata() -> Result<(), String> {
    let workspace_path = temp_dir("provenance-status");
    let lockfile_path = workspace_path.join("omena.lock");
    fs::create_dir_all(&workspace_path)
        .map_err(|error| format!("fixture workspace should be writable: {error}"))?;
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    let mut entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    entry.trust_tier = omena_sif::OmenaSifTrustTierV1::T3;
    entry
        .attestation_references
        .push(omena_sif::OmenaSifAttestationReferenceV1 {
            kind: "sigstore-bundle".to_string(),
            reference: "sif/design-system.sigstore.json".to_string(),
        });
    let lock = omena_sif::OmenaLockV1::new(vec![entry]);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Provenance {
            command: ProvenanceCommand::Status {
                lockfile: lockfile_path,
                json: true,
            },
        },
    });

    assert!(result.is_ok(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn provenance_status_reports_verified_policy_lock_metadata() -> Result<(), String> {
    let workspace_path = temp_dir("provenance-status-verified-policy");
    let lockfile_path = workspace_path.join("omena.lock");
    fs::create_dir_all(&workspace_path)
        .map_err(|error| format!("fixture workspace should be writable: {error}"))?;
    let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
    let sif_source = omena_sif::write_omena_sif_json_v1(&sif)
        .map_err(|error| format!("fixture SIF should serialize: {error}"))?;
    let subject_binding = try_read_verified_shard_artifact_binding(
        sif_source.as_bytes(),
        omena_sif::OmenaSifTrustTierV1::T3,
    )?
    .ok_or_else(|| "fixture SIF must produce a published subject binding".to_string())?;
    let mut entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
        .map_err(|error| format!("fixture lock entry should build: {error}"))?;
    let reference = "sif/design-system.sigstore.json".to_string();
    entry.trust_tier = omena_sif::OmenaSifTrustTierV1::T3;
    entry
        .attestation_references
        .push(omena_sif::OmenaSifAttestationReferenceV1 {
            kind: "sigstore-bundle".to_string(),
            reference: reference.clone(),
        });
    entry
        .attestation_verifications
        .push(omena_sif::OmenaSifAttestationVerificationV1 {
            kind: "omena-toolchain.sigstore".to_string(),
            reference,
            verifier: "sigstore-verify".to_string(),
            verified_trust_tier: omena_sif::OmenaSifTrustTierV1::T3,
            verified_tlog_integrated_time: Some(1_764_787_003),
            sigstore_verification_policy: Some(omena_sif::OmenaSifSigstoreVerificationPolicyV1 {
                trusted_root: "sigstore-production-trusted-root".to_string(),
                transparency_log: true,
                timestamp: true,
                certificate_chain: true,
                signed_certificate_timestamp: true,
            }),
            certificate_issuer: Some(
                omena_sif::OMENA_SIF_PUBLISHED_ATTESTATION_CERTIFICATE_ISSUER_V1.to_string(),
            ),
            certificate_identity: Some(
                omena_sif::OMENA_SIF_PUBLISHED_ATTESTATION_CERTIFICATE_IDENTITY_V1.to_string(),
            ),
            attestation_statement: Some({
                let subject_name = "design-system.attestation-subject.json".to_string();
                let mut statement = cli_fixture_provenance_statement();
                statement.subject_names = vec![subject_name.clone()];
                statement.subject_digests = vec![OmenaSifAttestationSubjectDigestV1 {
                    name: subject_name,
                    algorithm: "sha256".to_string(),
                    digest: subject_binding.attested_subject_sha256,
                }];
                statement
            }),
        });
    let lock = omena_sif::OmenaLockV1::new(vec![entry]);
    let report = omena_sif::summarize_omena_sif_provenance_advisory_v1(&lock);
    assert_eq!(report.entries[0].attestation_verification_count, 1);
    assert_eq!(report.entries[0].invalid_attestation_verification_count, 0);
    fs::write(
        &lockfile_path,
        omena_sif::write_omena_lock_json_v1(&lock)
            .map_err(|error| format!("fixture lock should serialize: {error}"))?,
    )
    .map_err(|error| format!("fixture lock should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Provenance {
            command: ProvenanceCommand::Status {
                lockfile: lockfile_path,
                json: true,
            },
        },
    });

    assert!(result.is_ok(), "{result:?}");
    cleanup_dir(&workspace_path);
    Ok(())
}

#[test]
fn sif_generate_command_writes_static_sif_artifact() -> Result<(), String> {
    let source_path = temp_path("tokens.scss");
    let output_path = temp_path("tokens.sif.json");
    fs::write(
        &source_path,
        r#"$brand: red !default; @mixin button($size: 1rem) { @content; }"#,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Sif {
            command: SifCommand::Generate {
                path: source_path.clone(),
                canonical_url: Some("pkg:design-system/_tokens.scss".to_string()),
                output: Some(output_path.clone()),
                syntax: Some("scss".to_string()),
                json: false,
            },
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let sif_json = fs::read_to_string(&output_path)
        .map_err(|error| format!("generated SIF should be readable: {error}"))?;
    assert!(sif_json.contains(r#""canonicalUrl":"pkg:design-system/_tokens.scss""#));
    assert!(sif_json.contains(r#""name":"$brand""#));
    assert!(sif_json.contains(r#""name":"button""#));

    cleanup(&source_path);
    cleanup(&output_path);
    Ok(())
}

#[test]
fn sif_generate_command_accepts_less_source_syntax() -> Result<(), String> {
    let source_path = temp_path("tokens.css");
    let output_path = temp_path("tokens-less.sif.json");
    fs::write(&source_path, r#"@brand: red; .button { color: @brand; }"#)
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Sif {
            command: SifCommand::Generate {
                path: source_path.clone(),
                canonical_url: Some("pkg:design-system/tokens.less".to_string()),
                output: Some(output_path.clone()),
                syntax: Some("less".to_string()),
                json: false,
            },
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let sif_json = fs::read_to_string(&output_path)
        .map_err(|error| format!("generated SIF should be readable: {error}"))?;
    assert!(sif_json.contains(r#""canonicalUrl":"pkg:design-system/tokens.less""#));
    assert!(sif_json.contains(r#""syntax":"less""#));

    cleanup(&source_path);
    cleanup(&output_path);
    Ok(())
}

#[test]
fn sif_generate_attestation_subject_binds_url_tier_and_artifact_hash() -> Result<(), String> {
    let source_path = temp_path("published.scss");
    let sif_path = temp_path("published.sif.json");
    let subject_path = temp_path("published.attestation-subject.json");
    fs::write(&source_path, "$brand: #0af;")
        .map_err(|error| format!("fixture source should be writable: {error}"))?;
    run(Cli {
        command: Command::Sif {
            command: SifCommand::Generate {
                path: source_path.clone(),
                canonical_url: Some("pkg:omena-fixture/published.scss".to_string()),
                output: Some(sif_path.clone()),
                syntax: Some("scss".to_string()),
                json: false,
            },
        },
    })?;

    run(Cli {
        command: Command::Sif {
            command: SifCommand::GenerateAttestationSubject {
                sif: sif_path.clone(),
                trust_tier: "t3".to_string(),
                output: Some(subject_path.clone()),
                json: false,
            },
        },
    })?;

    let subject_source = fs::read_to_string(&subject_path)
        .map_err(|error| format!("generated subject should be readable: {error}"))?;
    let subject =
        omena_sif::read_omena_sif_published_attestation_subject_json_v1(subject_source.as_str())
            .map_err(|error| format!("generated subject should parse: {error}"))?;
    let sif_source = fs::read_to_string(&sif_path)
        .map_err(|error| format!("generated SIF should be readable: {error}"))?;
    let sif = omena_sif::read_omena_sif_json_v1(sif_source.as_str())
        .map_err(|error| format!("generated SIF should parse: {error}"))?;
    assert_eq!(subject.canonical_url, "pkg:omena-fixture/published.scss");
    assert_eq!(subject.trust_tier, omena_sif::OmenaSifTrustTierV1::T3);
    assert_eq!(
        subject.sif_hash,
        omena_sif::compute_omena_sif_artifact_hash_v1(&sif).map_err(|error| error.to_string())?
    );

    cleanup(&source_path);
    cleanup(&sif_path);
    cleanup(&subject_path);
    Ok(())
}

#[test]
fn sif_generate_lif_exports_command_writes_less_interface_facts() -> Result<(), String> {
    let source_path = temp_path("tokens.less");
    let output_path = temp_path("tokens.lif-exports.json");
    fs::write(
        &source_path,
        r#"
@brand: #fff;
@tokens: { primary: @brand; @gap: 2px; };
.button(@gap: 1rem, @rest...) when (@gap > 0) { color: @brand; }
"#,
    )
    .map_err(|error| format!("fixture source should be writable: {error}"))?;

    let result = run(Cli {
        command: Command::Sif {
            command: SifCommand::GenerateLifExports {
                path: source_path.clone(),
                output: Some(output_path.clone()),
                syntax: Some("less".to_string()),
                json: false,
            },
        },
    });

    assert!(result.is_ok(), "{result:?}");
    let lif_json = fs::read_to_string(&output_path)
        .map_err(|error| format!("generated LIF exports should be readable: {error}"))?;
    assert!(lif_json.contains(r##""lessVariables":[{"name":"@brand","valueRepr":"#fff"}]"##));
    assert!(lif_json.contains(r#""lessMixins":[{"guarded":true,"name":".button""#));
    assert!(lif_json.contains(
        r#""lessDetachedRulesets":[{"memberNames":["@gap","primary"],"name":"@tokens"}]"#
    ));

    cleanup(&source_path);
    cleanup(&output_path);
    Ok(())
}

#[cfg(feature = "zk-audit")]
#[test]
fn audit_zk_commands_are_feature_gated_surfaces() {
    let prove_result = run(Cli {
        command: Command::Audit {
            command: AuditCommand::Zk {
                command: ZkAuditCommand::Prove {
                    audit_id: "test-audit".to_string(),
                    reorder: false,
                    json: true,
                },
            },
        },
    });
    assert!(prove_result.is_ok(), "{prove_result:?}");

    let verify_result = run(Cli {
        command: Command::Audit {
            command: AuditCommand::Zk {
                command: ZkAuditCommand::Verify {
                    audit_id: "test-audit".to_string(),
                    reorder: false,
                    json: true,
                },
            },
        },
    });
    assert!(verify_result.is_ok(), "{verify_result:?}");

    let setup_result = run(Cli {
        command: Command::Audit {
            command: AuditCommand::Zk {
                command: ZkAuditCommand::SetupStatus { json: true },
            },
        },
    });
    assert!(setup_result.is_ok(), "{setup_result:?}");
}

#[cfg(feature = "zk-audit")]
#[test]
fn audit_zk_cli_result_exposes_opt_in_scope_without_default_proving() {
    let setup = zk_audit_cli_result_v0(
        "omena-cli.audit.zk.setup-status",
        "setup-status",
        None,
        Some(zk_audit_ci_matrix_v0()),
        None,
        false,
    );

    assert_eq!(setup.mechanism_scope, ZK_AUDIT_MECHANISM_SCOPE_V0);
    assert_eq!(
        setup.default_proof_backend_enabled,
        ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0
    );
    assert_eq!(
        setup.active_proof_backend_scope,
        "optInArkworksGroth16RealBackendLinked"
    );
    assert!(!setup.verified);
    assert!(setup.groth16_roundtrip.is_none());

    let roundtrip = prove_and_verify_canonical_margin_cascade_with_arkworks_v0(false);
    assert!(roundtrip.is_ok(), "{roundtrip:?}");
    if let Ok(roundtrip) = roundtrip {
        let verified = roundtrip.proof_generated && roundtrip.proof_verified;
        let prove = zk_audit_cli_result_v0(
            "omena-cli.audit.zk.prove",
            "prove",
            Some(cascade_zk_audit_v0("test-audit")),
            None,
            Some(roundtrip),
            verified,
        );

        assert_eq!(prove.mechanism_scope, ZK_AUDIT_MECHANISM_SCOPE_V0);
        assert!(!prove.default_proof_backend_enabled);
        assert_eq!(
            prove.active_proof_backend_scope,
            "optInArkworksGroth16RealBackendLinked"
        );
        assert!(prove.verified);
    }
}

/// Mechanism-depth test for the CLI zk-audit product path: the exact function
/// the `prove`/`verify` arms invoke must verify a satisfiable cascade
/// obligation and reject an unsatisfiable one. The discriminating
/// `require:canonical-longhand-quartet` term is computed by the cascade
/// algorithm; this test only flips the longhand order via `reorder`, so it
/// fails if `proof_verified` were a constant.
#[cfg(feature = "zk-audit")]
#[test]
fn audit_zk_cli_prove_emits_verified_proof_only_for_satisfiable_cascade_obligation() {
    let canonical = prove_and_verify_canonical_margin_cascade_with_arkworks_v0(false);
    let reordered = prove_and_verify_canonical_margin_cascade_with_arkworks_v0(true);
    assert!(canonical.is_ok(), "{canonical:?}");
    assert!(reordered.is_ok(), "{reordered:?}");

    if let (Ok(canonical), Ok(reordered)) = (canonical, reordered) {
        // Satisfiable cascade obligation -> verified proof with real R1CS.
        assert!(canonical.proof_generated);
        assert!(canonical.proof_verified);
        assert!(canonical.circuit.constraint_count > 0);

        // Unsatisfiable cascade obligation -> proof rejected, no panic.
        assert!(!reordered.proof_generated);
        assert!(!reordered.proof_verified);

        // Same R1CS shape, opposite proof outcome: the verdict is driven by
        // the cascade obligation, not by the circuit size.
        assert_eq!(
            canonical.requirement_count, reordered.requirement_count,
            "both obligations encode the same requirement set"
        );
        assert_ne!(canonical.proof_verified, reordered.proof_verified);
    }
}

#[test]
fn cross_file_streaming_reachability_fires_on_use_chain_not_self_contained() {
    let tokens = OmenaQueryStyleSourceInputV0 {
        style_path: "/ws/_tokens.scss".to_string(),
        style_source: "$brand: red;\n".to_string(),
    };
    let importer = OmenaQueryStyleSourceInputV0 {
        style_path: "/ws/Button.module.scss".to_string(),
        style_source: "@use \"./tokens\" as tokens;\n.root { color: tokens.$brand; }\n".to_string(),
    };
    let sources = vec![tokens.clone(), importer.clone()];

    // The importer reaches a foreign module over the resolved `@use` edge: the demand-sliced-monotone-fact-propagation
    // oracle propagates a seeded fact from a Button node to a `_tokens.scss` node, so the
    // computed foreign-reachable set is non-empty.
    let importer_diagnostics = summarize_cross_file_streaming_reachability_diagnostics(
        &importer.style_path,
        sources.as_slice(),
        &[],
        &[],
    );
    assert_eq!(importer_diagnostics.len(), 1);
    assert_eq!(
        importer_diagnostics[0].code,
        "crossFileStreamingReachability"
    );
    assert!(
        importer_diagnostics[0].message.contains("/ws/_tokens.scss"),
        "diagnostic names the genuinely-reached foreign module: {}",
        importer_diagnostics[0].message
    );

    // The leaf token module imports nothing: seeded at its own nodes, the same oracle never
    // leaves its file, so no cross-file reachability diagnostic is computed. This is the
    // discriminating negative — the value is COMPUTED, not asserted against a literal.
    let leaf_diagnostics = summarize_cross_file_streaming_reachability_diagnostics(
        &tokens.style_path,
        sources.as_slice(),
        &[],
        &[],
    );
    assert!(
        leaf_diagnostics.is_empty(),
        "a module that reaches no foreign file must not surface a reachability fact: {leaf_diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_reads_configured_module_instance_key() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme-a.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme-a" as theme;"#.to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleInstanceIdentity"
                && diagnostic
                    .message
                    .contains("configured module instance '/tmp/tokens.scss'")
        }),
        "style diagnostics must consume module_instance_identity_key through the graph closure: {diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_reports_conflicting_sass_module_configurations() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme-red.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme-blue.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: blue);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme-red" as redTheme; @use "./theme-blue" as blueTheme;"#
                .to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleConfigurationConflict"
                && diagnostic.severity == "error"
                && diagnostic.message.contains("/tmp/tokens.scss")
                && diagnostic.message.contains("brand=3:red")
                && diagnostic.message.contains("brand=4:blue")
        }),
        "style diagnostics must reject incompatible Sass module configurations: {diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_reports_repeated_forward_configuration_conflicts() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme.scss".to_string(),
            style_source:
                r#"@forward "./tokens" with ($brand: red); @forward "./tokens" with ($brand: blue);"#
                    .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme" as theme;"#.to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleConfigurationConflict"
                && diagnostic.severity == "error"
                && diagnostic.message.contains("/tmp/tokens.scss")
                && diagnostic.message.contains("brand=3:red")
                && diagnostic.message.contains("brand=4:blue")
        }),
        "style diagnostics must reject repeated @forward configuration conflicts: {diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_allows_shared_sass_module_configuration() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme-a.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme-b.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme-a" as themeA; @use "./theme-b" as themeB;"#.to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "sassModuleConfigurationConflict"),
        "shared Sass module configuration must remain shareable: {diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_reports_configured_after_unconfigured_load_order() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme.scss".to_string(),
            style_source: r#"@forward "./tokens";"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source:
                r#"@use "./theme" as theme; @use "./tokens" as tokens with ($brand: red);"#
                    .to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleConfigurationConflict"
                && diagnostic.severity == "error"
                && diagnostic.message.contains("/tmp/tokens.scss")
                && diagnostic.message.contains("with:none")
                && diagnostic.message.contains("brand=3:red")
        }),
        "style diagnostics must reject configuring a Sass module after an unconfigured load: {diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_reports_non_default_sass_module_configuration() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./tokens" as tokens with ($brand: red);"#.to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleInvalidConfiguration"
                && diagnostic.severity == "error"
                && diagnostic.message.contains("/tmp/tokens.scss")
                && diagnostic.message.contains("$brand")
                && diagnostic.message.contains("!default")
        }),
        "style diagnostics must reject non-!default Sass module configuration: {diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_reports_downstream_configuration_after_forward_override() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme" as theme with ($brand: green);"#.to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleInvalidConfiguration"
                && diagnostic.severity == "error"
                && diagnostic.message.contains("/tmp/theme.scss")
                && diagnostic.message.contains("$brand")
                && diagnostic.message.contains("!default")
        }),
        "style diagnostics must reject downstream configuration after a non-default @forward with(...): {diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_uses_downstream_forward_default_configuration() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red !default);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme" as theme with ($brand: green);"#.to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleInstanceIdentity"
                && diagnostic.message.contains("/tmp/tokens.scss")
                && diagnostic.message.contains("brand=5:green")
        }),
        "style diagnostics must propagate downstream Sass configuration through @forward !default: {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().all(|diagnostic| {
            !(diagnostic.code == "sassModuleInstanceIdentity"
                && diagnostic.message.contains("/tmp/tokens.scss")
                && diagnostic.message.contains("brand=3:red"))
        }),
        "style diagnostics must not report the @forward !default value after a downstream override: {diagnostics:?}"
    );
}

#[test]
fn style_diagnostics_query_identity_accepts_path_mapped_forwarded_configuration() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_tokens.scss".to_string(),
            style_source: "$brand: blue !default;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_theme.scss".to_string(),
            style_source: r#"@forward "@design/tokens" with ($brand: red !default);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.scss".to_string(),
            style_source: r#"@use "./theme" as theme with ($brand: green);"#.to_string(),
        },
    ];
    let resolution_inputs = omena_query::OmenaQueryStyleResolutionInputsV0 {
        tsconfig_path_mappings: vec![omena_query::OmenaQueryTsconfigPathMappingV0 {
            base_path: "/workspace".to_string(),
            pattern: "@design/*".to_string(),
            target_patterns: vec!["src/*".to_string()],
        }],
        ..Default::default()
    };

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        "/workspace/src/App.module.scss",
        sources.as_slice(),
        &[],
        &resolution_inputs,
    );

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "sassModuleInvalidConfiguration"),
        "path-mapped forwarded Sass configuration must not be reported as invalid: {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleInstanceIdentity"
                && diagnostic.message.contains("/workspace/src/_tokens.scss")
                && diagnostic.message.contains("brand=5:green")
        }),
        "style diagnostics must propagate downstream configuration through the aliased @forward edge: {diagnostics:?}"
    );
}

#[cfg(unix)]
#[test]
fn style_diagnostics_query_identity_reads_symlink_chain_metadata() -> Result<(), String> {
    let workspace = temp_dir("sass-module-identity-symlink");
    let real_dir = workspace.join("real");
    let linked_dir = workspace.join("linked");
    fs::remove_dir_all(&workspace).ok();
    fs::create_dir_all(&real_dir)
        .map_err(|error| format!("fixture real dir should be writable: {error}"))?;
    fs::write(real_dir.join("tokens.scss"), "$brand: blue;")
        .map_err(|error| format!("fixture tokens source should be writable: {error}"))?;
    unix_fs::symlink(&real_dir, &linked_dir)
        .map_err(|error| format!("fixture symlink should be creatable: {error}"))?;

    let app_path = workspace.join("App.module.scss");
    let linked_tokens_path = linked_dir.join("tokens.scss");
    let app_path = app_path.to_string_lossy().into_owned();
    let linked_tokens_path = linked_tokens_path.to_string_lossy().into_owned();
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: app_path.clone(),
            style_source: r#"@use "./linked/tokens" as tokens;"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: linked_tokens_path,
            style_source: "$brand: blue;".to_string(),
        },
    ];

    let diagnostics =
        omena_query::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
        app_path.as_str(),
        sources.as_slice(),
        &[],
        &omena_query::OmenaQueryStyleResolutionInputsV0::default(),
    );

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "sassModuleSymlinkResolution"
                && diagnostic.message.contains("symlink link")
                && diagnostic.message.contains("/linked")
        }),
        "style diagnostics must consume symlink_chain_links: {diagnostics:?}"
    );

    cleanup_dir(&workspace);
    Ok(())
}

fn dynamic_classname_diagnostics_input_json(max_context_depth: usize) -> String {
    format!(
        r#"{{
  "sourceUri": "file:///Routes.tsx",
  "selectorUniverse": ["btn-primary"],
  "maxContextDepth": {max_context_depth},
  "callSites": [
{{
  "calleeKey": "classForVariant",
  "callSiteStack": ["RouteA.tsx:render", "PrimaryButton.tsx:className"],
  "exitValue": {{ "kind": "exact", "value": "btn-primary" }},
  "referenceRange": {{
    "start": {{ "line": 10, "character": 0 }},
    "end": {{ "line": 10, "character": 12 }}
  }}
}},
{{
  "calleeKey": "classForVariant",
  "callSiteStack": ["RouteB.tsx:render", "SecondaryButton.tsx:className"],
  "exitValue": {{ "kind": "exact", "value": "btn-secondary" }},
  "referenceRange": {{
    "start": {{ "line": 20, "character": 0 }},
    "end": {{ "line": 20, "character": 12 }}
  }}
}}
  ]
}}"#
    )
}

fn temp_path(name: &str) -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!("omena-cli-{nanos}-{name}"))
}

fn temp_dir(name: &str) -> PathBuf {
    temp_path(name)
}

fn cli_fixture_sif(
    canonical_url: &str,
    source_bytes: &[u8],
) -> Result<omena_sif::OmenaSifV1, String> {
    omena_sif::OmenaSifV1::from_static_exports(
        canonical_url,
        omena_sif::OmenaSifGeneratorV1 {
            name: "omena-sifgen".to_string(),
            version: "0.1.0".to_string(),
            toolchain_id: "omena-sifgen@0.1".to_string(),
        },
        omena_sif::OmenaSifSourceV1 {
            syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
        },
        omena_sif::OmenaSifExportsV1 {
            variables: vec![omena_sif::OmenaSifVariableExportV1 {
                name: "$color".to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            }],
            mixins: Vec::new(),
            functions: Vec::new(),
            placeholders: Vec::new(),
            forwards: Vec::new(),
        },
        Vec::new(),
        source_bytes,
    )
    .map_err(|error| format!("fixture SIF should build: {error}"))
}

fn cli_fixture_provenance_statement() -> omena_sif::OmenaSifAttestationStatementV1 {
    omena_sif::OmenaSifAttestationStatementV1 {
        statement_type: Some("https://in-toto.io/Statement/v1".to_string()),
        predicate_type: Some("https://slsa.dev/provenance/v1".to_string()),
        source_repository: Some("https://github.com/omenien/omena-css".to_string()),
        source_ref: Some("refs/tags/v1.0.0".to_string()),
        source_commit: Some("0123456789abcdef".to_string()),
        builder_id: Some("https://github.com/actions/runner/github-hosted".to_string()),
        build_type: Some(
            "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1".to_string(),
        ),
        subject_names: vec!["sif/design-system.sif.json".to_string()],
        subject_digests: vec![omena_sif::OmenaSifAttestationSubjectDigestV1 {
            name: "sif/design-system.sif.json".to_string(),
            algorithm: "sha256".to_string(),
            digest: "0123456789abcdef".to_string(),
        }],
    }
}

fn cleanup(path: &Path) {
    let _ = fs::remove_file(path);
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

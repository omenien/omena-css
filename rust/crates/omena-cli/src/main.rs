use clap::{Parser, Subcommand};
use omena_query::generate_omena_bridge_sif_for_resolved_style_path;
#[cfg(feature = "mdl")]
use omena_query::summarize_omena_query_design_system_minimum_description;
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryDiagnosticSuppressionModeV0,
    OmenaQueryDynamicClassnameMTierInputV0, OmenaQueryEngineInputV2,
    OmenaQueryExpressionDomainFlowRuntimeV0, OmenaQueryExternalModuleModeV0,
    OmenaQueryExternalSifInputV0, OmenaQuerySourceDiagnosticsForFileV0,
    OmenaQuerySourceDocumentInputV0, OmenaQuerySourceMissingSelectorDiagnosticCandidateV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformExecutionContextV0, ParserPositionV0, TransformBundleEdgeKind,
    attach_omena_query_consumer_build_bundle_summary,
    attach_omena_query_consumer_build_source_map_v3_with_sources,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    list_omena_query_transform_pass_summaries, read_omena_query_cascade_at_position,
    read_omena_query_cascade_at_position_with_categorical_evidence,
    read_omena_query_style_context_index,
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    rewrite_omena_transform_bundle_asset_urls_in_source,
    summarize_omena_query_bundle_code_split_source_map_v3,
    summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_sass_module_cross_file_resolution_for_workspace,
    summarize_omena_query_sass_module_sources, summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_style_completion_at_position,
    summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_suppression_mode,
    summarize_omena_query_style_document, summarize_omena_query_style_hover_candidates,
    summarize_omena_query_style_resolution_policy_v0,
    summarize_omena_query_transform_context_from_engine_input,
    summarize_omena_transform_bundle_from_source,
};
use omena_query::{
    OmenaQueryDiagnosticSuppressionReasonV0, OmenaQueryStyleDiagnosticV0,
    OmenaQueryStyleDiagnosticsForFileV0, ParserRangeV0,
    load_omena_query_workspace_style_resolution_inputs,
};
use omena_sif::{
    OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_PRODUCT_V1,
    OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_SCHEMA_VERSION_V1, OmenaLockV1,
    OmenaLockVerificationIssueV1, OmenaSifAttestationStatementV1,
    OmenaSifAttestationSubjectDigestV1, OmenaSifAttestationVerificationReportV1,
    OmenaSifSigstoreVerificationPolicyV1, OmenaSifSourceSyntaxV1, OmenaSifStaticGeneratorInputV1,
    apply_omena_sif_attestation_verification_report_to_lock_entry_v1,
    apply_omena_sif_npm_provenance_references_to_lock_entry_v1, build_omena_lock_sif_entry_v1,
    collect_omena_sif_npm_provenance_attestation_references_v1, compute_omena_sif_artifact_hash_v1,
    generate_static_omena_sif_v1, read_omena_lock_json_v1,
    read_omena_sif_attestation_verification_report_json_v1, read_omena_sif_json_v1,
    summarize_omena_sif_provenance_advisory_v1, verify_omena_lock_frozen_v1,
    write_omena_lock_json_v1, write_omena_sif_json_v1,
};
use omena_streaming_ifds::summarize_streaming_ifds_workspace_cross_file_reachability_v0;
#[cfg(feature = "zk-audit")]
use omena_zk_audit::{
    ArkworksGroth16RoundTripV0, CascadeZKAuditV0, ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0,
    ZK_AUDIT_MECHANISM_SCOPE_V0, ZKAuditCiMatrixV0, active_zk_audit_proof_backend_scope_v0,
    cascade_zk_audit_v0, prove_and_verify_canonical_margin_cascade_with_arkworks_v0,
    zk_audit_ci_matrix_v0,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, Parser)]
#[command(
    name = "omena",
    about = "Check and transform CSS-family sources with the Omena CSS workspace"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Command {
    /// Parse a CSS-family source and report parser-owned facts.
    Check {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to check.
        path: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Run the conservative transform pipeline.
    Build {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to transform.
        path: PathBuf,
        /// Optional output file. Prints to stdout when omitted.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Transform pass id. Repeat to choose specific passes.
        #[arg(long = "pass")]
        passes: Vec<String>,
        /// Browserslist query or named target profile used to plan target-sensitive passes.
        #[arg(long)]
        target_query: Option<String>,
        /// Allow logical property lowering when target query says it is needed.
        #[arg(long)]
        allow_logical_to_physical: bool,
        /// Allow @scope flattening when target query says it is needed.
        #[arg(long)]
        allow_scope_flatten: bool,
        /// Allow cascade layer flattening when target query says it is needed.
        #[arg(long)]
        allow_layer_flatten: bool,
        /// Enable static @supports branch evaluation.
        #[arg(long)]
        enable_supports_static_eval: bool,
        /// Enable static @media branch evaluation.
        #[arg(long)]
        enable_media_static_eval: bool,
        /// Drop dark color-scheme media branches when workspace policy proves no dark-mode runtime.
        #[arg(long)]
        drop_dark_mode_media_queries: bool,
        /// JSON file containing a TransformExecutionContextV0 evaluator/provenance bridge.
        #[arg(long)]
        context_json: Option<PathBuf>,
        /// JSON file containing EngineInputV2 source/style/type facts for semantic reachability.
        #[arg(long)]
        engine_input_json: Option<PathBuf>,
        /// Treat the provided context/engine input as a closed style world for tree shaking.
        #[arg(long)]
        closed_style_world: bool,
        /// Enable the public CSS Modules tree-shake build mode.
        #[arg(long)]
        tree_shake: bool,
        /// Enable bundle-planned workspace build mode over the provided --source graph.
        #[arg(long)]
        bundle: bool,
        /// Emit bundle code-split CSS files into this directory.
        #[arg(long = "split-out-dir")]
        split_out_dir: Option<PathBuf>,
        /// Additional workspace style source used to derive import/composes build context.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports for workspace sources.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
        /// Include a Source Map V3 payload in --json output.
        #[arg(long)]
        source_map: bool,
        /// Print a machine-readable execution summary.
        #[arg(long)]
        json: bool,
    },
    /// List transform pass ids accepted by `omena build --pass`.
    Passes {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Estimate an MDL minimum-description summary for a style source.
    #[cfg(feature = "mdl")]
    Compress {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to summarize.
        path: PathBuf,
        /// Fail when the estimated description length exceeds this bit budget.
        #[arg(long)]
        budget_bits: Option<f64>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Derive transform context from EngineInputV2 semantic reachability.
    Context {
        /// Target CSS, SCSS, Sass, Less, or CSS Modules path.
        path: PathBuf,
        /// JSON file containing EngineInputV2 source/style/type facts.
        #[arg(long)]
        engine_input_json: PathBuf,
        /// Treat the engine input as a closed style world for tree shaking.
        #[arg(long)]
        closed_style_world: bool,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Analyze cross-language class-value flow from EngineInputV2.
    ExpressionFlow {
        /// JSON file containing EngineInputV2 source/style/type facts.
        #[arg(long)]
        engine_input_json: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Project expression-domain flow values to target style selectors.
    SelectorProjection {
        /// JSON file containing EngineInputV2 source/style/type facts.
        #[arg(long)]
        engine_input_json: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read cascade and custom-property LFP information at a source position.
    Cascade {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Zero-based line number.
        #[arg(long)]
        line: usize,
        /// Zero-based UTF-16-like character offset used by the query protocol.
        #[arg(long)]
        character: usize,
        /// Optional EngineInputV2 JSON file for source/type context.
        #[arg(long)]
        engine_input_json: Option<PathBuf>,
        /// Attach opt-in categorical cascade evidence to the query result.
        #[arg(long)]
        categorical_evidence: bool,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read @layer, @container, and @scope context indexes.
    ContextIndex {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Optional EngineInputV2 JSON file for source/type context.
        #[arg(long)]
        engine_input_json: Option<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned style diagnostics for a CSS-family file.
    StyleDiagnostics {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Additional workspace style source used to resolve CSS Modules imports.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// Additional source document used to resolve selector usage.
        #[arg(long = "source-document")]
        source_document_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports for workspace sources.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
        /// SIF v1 artifact used to resolve opt-in external Sass modules.
        #[arg(long = "sif")]
        sif_paths: Vec<PathBuf>,
        /// Omena lockfile whose SIF entries should resolve opt-in external Sass modules.
        #[arg(long = "lockfile")]
        lockfile: Option<PathBuf>,
        /// External Sass module mode: omitted enables SIF discovery; use ignored as the compatibility opt-out.
        #[arg(long)]
        external: Option<String>,
        /// Opt-in deep analysis: also surface the rg-flow / categorical theory hints
        /// (off by default; deduplicated against the circular-var warning). Single-file path only.
        #[arg(long)]
        deep_analysis: bool,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned style hover candidates for a CSS-family file.
    StyleHoverCandidates {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned style completions at a source position.
    StyleCompletion {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Zero-based line number.
        #[arg(long)]
        line: usize,
        /// Zero-based UTF-16-like character offset used by the query protocol.
        #[arg(long)]
        character: usize,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned source diagnostics from precomputed missing-selector candidates.
    SourceDiagnostics {
        /// Source document URI used in the diagnostics result.
        source_uri: String,
        /// JSON file containing source missing-selector diagnostic candidates.
        #[arg(long)]
        candidates_json: Option<PathBuf>,
        /// Source document path used to compute workspace diagnostics directly.
        #[arg(long)]
        source_path: Option<PathBuf>,
        /// Workspace style source used to resolve CSS Module selectors.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports for workspace sources.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Read query-owned dynamic className M-tier diagnostics from an input JSON contract.
    DynamicClassnameDiagnostics {
        /// JSON file containing OmenaQueryDynamicClassnameMTierInputV0.
        #[arg(long)]
        input_json: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Emit downstream perceptual-check JSON from Omena style facts.
    PerceptualCheck {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Verify local Omena lockfile integrity.
    Lock {
        /// Lockfile path used by bare `omena lock` status. Subcommands keep their own --lockfile flag.
        #[arg(long, default_value = "omena.lock")]
        lockfile: PathBuf,
        /// Print machine-readable JSON for bare `omena lock` status.
        #[arg(long)]
        json: bool,
        #[command(subcommand)]
        command: Option<LockCommand>,
    },
    /// Generate local Sass Interface File artifacts.
    Sif {
        #[command(subcommand)]
        command: SifCommand,
    },
    /// Inspect deferred/advisory SIF provenance metadata without network access.
    Provenance {
        #[command(subcommand)]
        command: ProvenanceCommand,
    },
    /// Report soundiness and diagnostic-noise visibility for a workspace slice.
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Run feature-gated audit surfaces.
    #[cfg(feature = "zk-audit")]
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
}

// Clap subcommand enums are parsed once per process; direct fields keep argv
// parsing simple without meaningful runtime pressure.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
enum LockCommand {
    /// Author or refresh omena.lock from generated SIF artifacts.
    Update {
        /// Optional package/canonical URL selector to refresh while preserving other entries.
        package: Option<String>,
        /// Lockfile path. Defaults to ./omena.lock.
        #[arg(long, default_value = "omena.lock")]
        lockfile: PathBuf,
        /// SIF artifact to record. Repeat for multiple entries.
        #[arg(long = "sif")]
        sif_paths: Vec<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Add a package/canonical URL to omena.lock from generated SIF artifacts.
    Add {
        /// Package or canonical URL selector whose SIF artifacts should be added.
        package: String,
        /// Lockfile path. Defaults to ./omena.lock.
        #[arg(long, default_value = "omena.lock")]
        lockfile: PathBuf,
        /// SIF artifact to record. Repeat for multiple entries.
        #[arg(long = "sif")]
        sif_paths: Vec<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Record npm provenance references for existing omena.lock entries.
    FetchProvenance {
        /// Package or canonical URL selector whose lock entries should receive provenance references.
        package: String,
        /// Lockfile path. Defaults to ./omena.lock.
        #[arg(long, default_value = "omena.lock")]
        lockfile: PathBuf,
        /// npm registry metadata JSON containing dist.attestations.provenance.
        #[arg(long = "npm-metadata")]
        npm_metadata: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Record locally verified attestation evidence for an existing omena.lock entry.
    RecordVerification {
        /// Package or canonical URL selector whose lock entries should receive verified evidence.
        package: String,
        /// Lockfile path. Defaults to ./omena.lock.
        #[arg(long, default_value = "omena.lock")]
        lockfile: PathBuf,
        /// Verification report JSON produced by an offline attestation verifier.
        #[arg(long)]
        verification: PathBuf,
        /// SIF artifact bytes covered by a T3 offline verification report.
        #[arg(long)]
        artifact: Option<PathBuf>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Verify a Sigstore bundle locally and record verified lock evidence.
    VerifyAttestation {
        /// Package or canonical URL selector whose lock entries should receive verified evidence.
        package: String,
        /// Lockfile path. Defaults to ./omena.lock.
        #[arg(long, default_value = "omena.lock")]
        lockfile: PathBuf,
        /// Artifact bytes covered by the Sigstore bundle. T3 requires the matching SIF JSON.
        #[arg(long)]
        artifact: PathBuf,
        /// Sigstore bundle JSON to verify.
        #[arg(long)]
        bundle: PathBuf,
        /// Recorded provenance reference that this verification satisfies.
        #[arg(long)]
        reference: String,
        /// Verification evidence kind stored in omena.lock.
        #[arg(long, default_value = "npm-provenance.sigstore")]
        kind: String,
        /// Verified trust tier recorded after local Sigstore verification: t2 or t3.
        #[arg(long = "verified-tier", default_value = "t2")]
        verified_tier: String,
        /// Required certificate identity.
        #[arg(long)]
        identity: Option<String>,
        /// Required certificate OIDC issuer.
        #[arg(long)]
        issuer: String,
        /// Expected in-toto statement _type.
        #[arg(long = "statement-type")]
        statement_type: Option<String>,
        /// Expected in-toto/SLSA statement predicateType.
        #[arg(long = "statement-predicate-type")]
        statement_predicate_type: Option<String>,
        /// Expected source repository recorded in the signed provenance statement.
        #[arg(long = "statement-source-repository")]
        statement_source_repository: Option<String>,
        /// Expected source ref recorded in the signed provenance statement.
        #[arg(long = "statement-source-ref")]
        statement_source_ref: Option<String>,
        /// Expected source commit recorded in the signed provenance statement.
        #[arg(long = "statement-source-commit")]
        statement_source_commit: Option<String>,
        /// Expected builder id recorded in the signed provenance statement.
        #[arg(long = "statement-builder-id")]
        statement_builder_id: Option<String>,
        /// Expected build type recorded in the signed provenance statement.
        #[arg(long = "statement-build-type")]
        statement_build_type: Option<String>,
        /// Required subject name recorded in the signed provenance statement. Repeat for multiple subjects.
        #[arg(long = "statement-subject-name")]
        statement_subject_names: Vec<String>,
        /// Required subject digest recorded in the signed provenance statement as name=algorithm:digest. Repeat for multiple subjects.
        #[arg(long = "statement-subject-digest")]
        statement_subject_digests: Vec<String>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Verify that checked-in SIF artifacts match omena.lock.
    Verify {
        /// Lockfile path. Defaults to ./omena.lock.
        #[arg(long, default_value = "omena.lock")]
        lockfile: PathBuf,
        /// Source file whose external Sass module references must be covered by omena.lock.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// Refuse to update lockfile state and fail on any drift.
        #[arg(long)]
        frozen: bool,
        /// Require every lock entry to meet at least this trust tier: t0, t1, t2, or t3.
        #[arg(long = "tier")]
        tier: Option<String>,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum SifCommand {
    /// Generate a SIF v1 artifact from a Sass-family source without evaluating Sass.
    Generate {
        /// CSS, SCSS, or Sass source to scan.
        path: PathBuf,
        /// Stable canonical URL stored in the generated SIF. Defaults to the input path.
        #[arg(long)]
        canonical_url: Option<String>,
        /// Output path. Prints SIF JSON to stdout when omitted.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Source syntax: css, scss, or sass. Defaults from extension.
        #[arg(long)]
        syntax: Option<String>,
        /// Print generated SIF JSON even when --output is provided.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ProvenanceCommand {
    /// Report recorded trust tiers and attestations without verifying T2/T3 provenance.
    Status {
        /// Lockfile path. Defaults to ./omena.lock.
        #[arg(long, default_value = "omena.lock")]
        lockfile: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ReportCommand {
    /// Summarize boundary-state compromises, suppressions, and noise-budget status.
    Soundiness {
        /// Workspace style source to include in the report. Repeat for multiple files.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// Additional source document used to resolve selector usage.
        #[arg(long = "source-document")]
        source_document_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports for workspace sources.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
        /// SIF v1 artifact used to resolve opt-in external Sass modules.
        #[arg(long = "sif")]
        sif_paths: Vec<PathBuf>,
        /// Omena lockfile whose SIF entries should resolve opt-in external Sass modules.
        #[arg(long = "lockfile")]
        lockfile: Option<PathBuf>,
        /// External Sass module mode: ignored preserves compatibility, sif reports boundary states.
        #[arg(long, default_value = "sif")]
        external: String,
        /// Report diagnostics without hiding entries matched by omena suppression directives.
        #[arg(long = "no-suppress")]
        no_suppress: bool,
        /// Fail when the report observes more suppressions than this threshold.
        #[arg(long = "max-suppressions")]
        max_suppressions: Option<usize>,
        /// Fail when stale expect-error suppressions are observed.
        #[arg(long = "report-stale-suppressions")]
        report_stale_suppressions: bool,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Report the ordered style-resolution policy used by resolver-backed product paths.
    ResolutionPolicy {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[cfg(feature = "zk-audit")]
#[derive(Debug, Subcommand)]
enum AuditCommand {
    /// Run zero-knowledge cascade audit commands.
    Zk {
        #[command(subcommand)]
        command: ZkAuditCommand,
    },
}

#[cfg(feature = "zk-audit")]
#[derive(Debug, Subcommand)]
enum ZkAuditCommand {
    /// Generate a real arkworks Groth16 proof from a cascade obligation.
    Prove {
        /// Stable audit identifier.
        #[arg(long, default_value = "cli-zk-audit")]
        audit_id: String,
        /// Reorder the margin longhand quartet so the cascade obligation is
        /// unsatisfiable (no verified proof is produced).
        #[arg(long)]
        reorder: bool,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Verify a real arkworks Groth16 proof generated from a cascade obligation.
    Verify {
        /// Stable audit identifier.
        #[arg(long, default_value = "cli-zk-audit")]
        audit_id: String,
        /// Reorder the margin longhand quartet so the cascade obligation is
        /// unsatisfiable (verification fails).
        #[arg(long)]
        reorder: bool,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Report the default Halo2+IPA setup status and opt-in CI cells.
    SetupStatus {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[cfg(feature = "zk-audit")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZkAuditCliResultV0 {
    schema_version: &'static str,
    product: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
    mechanism_scope: &'static str,
    default_proof_backend_enabled: bool,
    active_proof_backend_scope: &'static str,
    command: &'static str,
    audit: Option<CascadeZKAuditV0>,
    ci_matrix: Option<ZKAuditCiMatrixV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    groth16_roundtrip: Option<ArkworksGroth16RoundTripV0>,
    verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct PerceptualCheckCliReportV0 {
    schema_version: &'static str,
    product: &'static str,
    command: &'static str,
    claim_level: &'static str,
    style_path: String,
    language: &'static str,
    fact_source_products: Vec<&'static str>,
    selector_count: usize,
    custom_property_declaration_count: usize,
    custom_property_reference_count: usize,
    diagnostic_count: usize,
    color_machinery_source: &'static str,
    json_schema_ready: bool,
    downstream_tool_scaffold_ready: bool,
    consumes_omena_facts: bool,
    wcag_algorithm_ready: bool,
    wcag_exact_color_contrast_bound_count: usize,
    wcag_exact_color_contrast_bounds: Vec<PerceptualExactColorContrastBoundV0>,
    apca_algorithm_ready: bool,
    oklab_perceptual_operator_ready: bool,
    full_perceptual_algorithm_ready: bool,
    public_safety_claim_ready: bool,
    supported_claims: Vec<&'static str>,
    deferred_claims: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct PerceptualExactColorContrastBoundV0 {
    schema_version: &'static str,
    product: &'static str,
    feature_gate: &'static str,
    claim_level: &'static str,
    selector_name: String,
    foreground_property: &'static str,
    background_property: &'static str,
    foreground: String,
    background: String,
    foreground_luminance: f64,
    background_luminance: f64,
    contrast_ratio: f64,
    wcag_aa_normal_text_threshold: f64,
    passes_aa_normal_text: bool,
    public_safety_claim_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PerceptualExactSrgbColorV0 {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PerceptualDeclarationColorV0 {
    property: &'static str,
    value: String,
    color: PerceptualExactSrgbColorV0,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Check { path, json } => check_file(path, json),
        Command::Build {
            path,
            output,
            passes,
            target_query,
            allow_logical_to_physical,
            allow_scope_flatten,
            allow_layer_flatten,
            enable_supports_static_eval,
            enable_media_static_eval,
            drop_dark_mode_media_queries,
            context_json,
            engine_input_json,
            closed_style_world,
            tree_shake,
            bundle,
            split_out_dir,
            source_paths,
            package_manifest_paths,
            source_map,
            json,
        } => build_file(BuildFileOptions {
            path,
            output,
            pass_ids: passes,
            target_query,
            context_json,
            engine_input_json,
            closed_style_world,
            tree_shake,
            bundle,
            split_out_dir,
            source_paths,
            package_manifest_paths,
            source_map,
            target_options: OmenaQueryTargetTransformOptionsV0 {
                allow_logical_to_physical,
                allow_scope_flatten,
                allow_layer_flatten,
                enable_supports_static_eval,
                enable_media_static_eval,
                drop_dark_mode_media_queries,
            },
            json,
        }),
        Command::Passes { json } => list_passes(json),
        #[cfg(feature = "mdl")]
        Command::Compress {
            path,
            budget_bits,
            json,
        } => compress_file(path, budget_bits, json),
        Command::Context {
            path,
            engine_input_json,
            closed_style_world,
            json,
        } => context_from_engine_input(path, engine_input_json, closed_style_world, json),
        Command::ExpressionFlow {
            engine_input_json,
            json,
        } => expression_flow(engine_input_json, json),
        Command::SelectorProjection {
            engine_input_json,
            json,
        } => selector_projection(engine_input_json, json),
        Command::Cascade {
            path,
            line,
            character,
            engine_input_json,
            categorical_evidence,
            json,
        } => cascade_at_position(
            path,
            line,
            character,
            engine_input_json,
            categorical_evidence,
            json,
        ),
        Command::ContextIndex {
            path,
            engine_input_json,
            json,
        } => context_index(path, engine_input_json, json),
        Command::StyleDiagnostics {
            path,
            source_paths,
            source_document_paths,
            package_manifest_paths,
            sif_paths,
            lockfile,
            external,
            deep_analysis,
            json,
        } => style_diagnostics(
            path,
            source_paths,
            source_document_paths,
            package_manifest_paths,
            sif_paths,
            lockfile,
            external,
            deep_analysis,
            json,
        ),
        Command::StyleHoverCandidates { path, json } => style_hover_candidates(path, json),
        Command::StyleCompletion {
            path,
            line,
            character,
            json,
        } => style_completion(path, line, character, json),
        Command::SourceDiagnostics {
            source_uri,
            candidates_json,
            source_path,
            source_paths,
            package_manifest_paths,
            json,
        } => source_diagnostics(
            source_uri,
            candidates_json,
            source_path,
            source_paths,
            package_manifest_paths,
            json,
        ),
        Command::DynamicClassnameDiagnostics { input_json, json } => {
            dynamic_classname_diagnostics(input_json, json)
        }
        Command::PerceptualCheck { path, json } => perceptual_check(path, json),
        Command::Lock {
            lockfile,
            json,
            command,
        } => lock_command(lockfile, json, command),
        Command::Sif { command } => sif_command(command),
        Command::Provenance { command } => provenance_command(command),
        Command::Report { command } => report_command(command),
        #[cfg(feature = "zk-audit")]
        Command::Audit { command } => audit_command(command),
    }
}

fn lock_command(
    status_lockfile: PathBuf,
    status_json: bool,
    command: Option<LockCommand>,
) -> Result<(), String> {
    match command {
        None => lock_status(status_lockfile, status_json),
        Some(LockCommand::Update {
            package,
            lockfile,
            sif_paths,
            json,
        }) => lock_update(lockfile, package, sif_paths, json),
        Some(LockCommand::Add {
            package,
            lockfile,
            sif_paths,
            json,
        }) => lock_add(lockfile, package, sif_paths, json),
        Some(LockCommand::FetchProvenance {
            package,
            lockfile,
            npm_metadata,
            json,
        }) => lock_fetch_provenance(lockfile, package, npm_metadata, json),
        Some(LockCommand::RecordVerification {
            package,
            lockfile,
            verification,
            artifact,
            json,
        }) => lock_record_verification(lockfile, package, verification, artifact, json),
        Some(LockCommand::VerifyAttestation {
            package,
            lockfile,
            artifact,
            bundle,
            reference,
            kind,
            verified_tier,
            identity,
            issuer,
            statement_type,
            statement_predicate_type,
            statement_source_repository,
            statement_source_ref,
            statement_source_commit,
            statement_builder_id,
            statement_build_type,
            statement_subject_names,
            statement_subject_digests,
            json,
        }) => lock_verify_attestation(LockVerifyAttestationInput {
            lockfile,
            package,
            artifact,
            bundle,
            reference,
            kind,
            verified_tier,
            identity,
            issuer,
            statement_policy: AttestationStatementPolicy {
                statement_type,
                predicate_type: statement_predicate_type,
                source_repository: statement_source_repository,
                source_ref: statement_source_ref,
                source_commit: statement_source_commit,
                builder_id: statement_builder_id,
                build_type: statement_build_type,
                subject_names: statement_subject_names,
                subject_digests: statement_subject_digests,
            },
            json,
        }),
        Some(LockCommand::Verify {
            lockfile,
            source_paths,
            frozen,
            tier,
            json,
        }) => lock_verify(lockfile, source_paths, frozen, tier, json),
    }
}

fn lock_update(
    lockfile: PathBuf,
    package: Option<String>,
    sif_paths: Vec<PathBuf>,
    json: bool,
) -> Result<(), String> {
    if sif_paths.is_empty() {
        return Err("omena lock update requires at least one --sif <path>".to_string());
    }

    let mut entries = build_lock_entries_from_sif_paths(&lockfile, &sif_paths)?;
    if let Some(package) = package.as_deref() {
        entries = filter_lock_entries_for_package(entries, package)?;
        let existing = read_lockfile_or_empty(&lockfile)?;
        let mut merged = existing
            .entries
            .into_iter()
            .filter(|entry| !lock_entry_matches_package_selector(entry, package))
            .collect::<Vec<_>>();
        merged.extend(entries);
        entries = merged;
    }

    let lock = OmenaLockV1::new(entries);
    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    if json {
        print_json(&lock)?;
    } else {
        println!(
            "omena.lock updated: {} SIF entr{} recorded at {}",
            lock.entries.len(),
            if lock.entries.len() == 1 { "y" } else { "ies" },
            path_string(&lockfile)
        );
    }

    Ok(())
}

fn lock_add(
    lockfile: PathBuf,
    package: String,
    sif_paths: Vec<PathBuf>,
    json: bool,
) -> Result<(), String> {
    if sif_paths.is_empty() {
        return Err("omena lock add requires at least one --sif <path>".to_string());
    }

    let existing = read_lockfile_or_empty(&lockfile)?;
    if existing
        .entries
        .iter()
        .any(|entry| lock_entry_matches_package_selector(entry, &package))
    {
        return Err(format!(
            "omena.lock already contains entries for '{package}'; use `omena lock update {package}` to refresh them"
        ));
    }

    let mut entries = existing.entries;
    entries.extend(filter_lock_entries_for_package(
        build_lock_entries_from_sif_paths(&lockfile, &sif_paths)?,
        &package,
    )?);
    let lock = OmenaLockV1::new(entries);
    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    let added_count = lock
        .entries
        .iter()
        .filter(|entry| lock_entry_matches_package_selector(entry, &package))
        .count();
    if json {
        print_json(&lock)?;
    } else {
        println!(
            "omena.lock added '{package}': {} SIF entr{} recorded at {}",
            added_count,
            if added_count == 1 { "y" } else { "ies" },
            path_string(&lockfile)
        );
    }

    Ok(())
}

fn lock_fetch_provenance(
    lockfile: PathBuf,
    package: String,
    npm_metadata: PathBuf,
    json: bool,
) -> Result<(), String> {
    let mut lock = read_lockfile_or_empty(&lockfile)?;
    let metadata_source = read_source(&npm_metadata)?;
    let references =
        collect_omena_sif_npm_provenance_attestation_references_v1(&metadata_source)
            .map_err(|error| format!("failed to parse {}: {error}", path_string(&npm_metadata)))?;
    if references.is_empty() {
        return Err(format!(
            "npm metadata {} does not contain dist.attestations.provenance",
            path_string(&npm_metadata)
        ));
    }

    let mut matched_count = 0usize;
    let mut added_reference_count = 0usize;
    for entry in &mut lock.entries {
        if !lock_entry_matches_package_selector(entry, &package) {
            continue;
        }
        matched_count += 1;
        added_reference_count += apply_omena_sif_npm_provenance_references_to_lock_entry_v1(
            entry,
            references.as_slice(),
        );
    }
    if matched_count == 0 {
        return Err(format!(
            "omena.lock contains no entries for package or canonical URL selector '{package}'"
        ));
    }

    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    if json {
        print_json(&lock)?;
    } else {
        println!(
            "omena.lock provenance updated: {matched_count} entr{} matched, {added_reference_count} reference{} added",
            if matched_count == 1 { "y" } else { "ies" },
            if added_reference_count == 1 { "" } else { "s" },
        );
    }

    Ok(())
}

fn lock_record_verification(
    lockfile: PathBuf,
    package: String,
    verification: PathBuf,
    artifact: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let mut lock = read_lockfile_or_empty(&lockfile)?;
    let verification_source = read_source(&verification)?;
    let report = read_omena_sif_attestation_verification_report_json_v1(&verification_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(&verification)))?;

    let mut matched_count = 0usize;
    let mut applied_count = 0usize;
    for entry in &mut lock.entries {
        if !lock_entry_matches_package_selector(entry, &package) {
            continue;
        }
        matched_count += 1;
        let mut updated_entry = entry.clone();
        let applied = apply_omena_sif_attestation_verification_report_to_lock_entry_v1(
            &mut updated_entry,
            &report,
        )
        .map_err(|error| {
            format!(
                "attestation verification report {} rejected for {}: {error}",
                path_string(&verification),
                entry.canonical_url
            )
        })?;
        if applied {
            if report.verified_trust_tier == omena_sif::OmenaSifTrustTierV1::T3 {
                let artifact = artifact.as_ref().ok_or_else(|| {
                    "lock record-verification with verifiedTrustTier t3 requires --artifact to be the matching SIF JSON".to_string()
                })?;
                let artifact_bytes = fs::read(artifact).map_err(|error| {
                    format!("failed to read {}: {error}", path_string(artifact))
                })?;
                let binding = read_verified_t3_attestation_artifact_binding(
                    artifact,
                    artifact_bytes.as_slice(),
                )?;
                validate_verified_t3_attestation_artifact_binding(entry, &binding)?;
                validate_verified_t3_attestation_statement_binding(
                    entry,
                    &binding,
                    report.attestation_statement.as_ref(),
                )?;
            }
            *entry = updated_entry;
            applied_count += 1;
        }
    }
    if matched_count == 0 {
        return Err(format!(
            "omena.lock contains no entries for package or canonical URL selector '{package}'"
        ));
    }
    if applied_count == 0 {
        return Err(format!(
            "attestation verification report {} did not match any selected lock entry subject",
            path_string(&verification)
        ));
    }

    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    if json {
        print_json(&lock)?;
    } else {
        println!(
            "omena.lock verification updated: {applied_count} entr{} recorded from {}",
            if applied_count == 1 { "y" } else { "ies" },
            path_string(&verification)
        );
    }

    Ok(())
}

struct LockVerifyAttestationInput {
    lockfile: PathBuf,
    package: String,
    artifact: PathBuf,
    bundle: PathBuf,
    reference: String,
    kind: String,
    verified_tier: String,
    identity: Option<String>,
    issuer: String,
    statement_policy: AttestationStatementPolicy,
    json: bool,
}

#[derive(Debug, Clone, Default)]
struct AttestationStatementPolicy {
    statement_type: Option<String>,
    predicate_type: Option<String>,
    source_repository: Option<String>,
    source_ref: Option<String>,
    source_commit: Option<String>,
    builder_id: Option<String>,
    build_type: Option<String>,
    subject_names: Vec<String>,
    subject_digests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttestationStatementSubjectDigestPolicy {
    name: String,
    algorithm: String,
    digest: String,
}

impl AttestationStatementPolicy {
    fn is_empty(&self) -> bool {
        self.statement_type.is_none()
            && self.predicate_type.is_none()
            && self.source_repository.is_none()
            && self.source_ref.is_none()
            && self.source_commit.is_none()
            && self.builder_id.is_none()
            && self.build_type.is_none()
            && self.subject_names.is_empty()
            && self.subject_digests.is_empty()
    }
}

fn lock_verify_attestation(input: LockVerifyAttestationInput) -> Result<(), String> {
    let LockVerifyAttestationInput {
        lockfile,
        package,
        artifact,
        bundle,
        reference,
        kind,
        verified_tier,
        identity,
        issuer,
        statement_policy,
        json,
    } = input;

    let verified_trust_tier = parse_lock_trust_tier(Some(verified_tier.as_str()))?
        .ok_or_else(|| "lock verify-attestation requires --verified-tier".to_string())?;
    if verified_trust_tier < omena_sif::OmenaSifTrustTierV1::T2 {
        return Err(format!(
            "lock verify-attestation --verified-tier must be t2 or t3, got {}",
            verified_trust_tier.as_str()
        ));
    }
    validate_attestation_policy_for_verified_tier(
        kind.as_str(),
        verified_trust_tier,
        Some(issuer.as_str()),
        identity.as_deref(),
    )?;

    let mut lock = read_lockfile_or_empty(&lockfile)?;
    let artifact_bytes = fs::read(&artifact)
        .map_err(|error| format!("failed to read {}: {error}", path_string(&artifact)))?;
    let bundle_source = read_source(&bundle)?;
    let sigstore_bundle = sigstore_verify::types::Bundle::from_json(&bundle_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(&bundle)))?;
    let trusted_root = sigstore_verify::trust_root::TrustedRoot::from_json(
        sigstore_verify::trust_root::SIGSTORE_PRODUCTION_TRUSTED_ROOT,
    )
    .map_err(|error| format!("failed to load Sigstore production trust root: {error}"))?;

    let mut policy = sigstore_verify::VerificationPolicy::default();
    if let Some(identity) = identity.as_ref() {
        policy = policy.require_identity(identity.clone());
    }
    policy = policy.require_issuer(issuer.clone());
    let verification_result = sigstore_verify::verify(
        artifact_bytes.as_slice(),
        &sigstore_bundle,
        &policy,
        &trusted_root,
    )
    .map_err(|error| {
        format!(
            "sigstore verification failed for {} with {}: {error}",
            path_string(&artifact),
            path_string(&bundle)
        )
    })?;

    if !verification_result.success {
        return Err(format!(
            "sigstore verification did not succeed for {} with {}",
            path_string(&artifact),
            path_string(&bundle)
        ));
    }
    let t3_artifact_binding = if verified_trust_tier == omena_sif::OmenaSifTrustTierV1::T3 {
        Some(read_verified_t3_attestation_artifact_binding(
            &artifact,
            artifact_bytes.as_slice(),
        )?)
    } else {
        None
    };
    let attestation_statement = extract_verified_attestation_statement(
        &sigstore_bundle,
        &statement_policy,
        attestation_kind_requires_provenance_statement(kind.as_str()),
    )?;

    let mut matched_count = 0usize;
    let mut applied_count = 0usize;
    for entry in &mut lock.entries {
        if !lock_entry_matches_package_selector(entry, &package) {
            continue;
        }
        matched_count += 1;
        if let Some(binding) = t3_artifact_binding.as_ref() {
            validate_verified_t3_attestation_artifact_binding(entry, binding)?;
            validate_verified_t3_attestation_statement_binding(
                entry,
                binding,
                attestation_statement.as_ref(),
            )?;
        }
        let report = OmenaSifAttestationVerificationReportV1 {
            schema_version: OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_SCHEMA_VERSION_V1.to_string(),
            product: OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_PRODUCT_V1.to_string(),
            verified: true,
            kind: kind.clone(),
            reference: reference.clone(),
            verifier: "sigstore-verify".to_string(),
            verified_trust_tier,
            verified_tlog_integrated_time: verification_result.integrated_time,
            sigstore_verification_policy: Some(OmenaSifSigstoreVerificationPolicyV1 {
                trusted_root: "sigstore-production-trusted-root".to_string(),
                transparency_log: policy.verify_tlog,
                timestamp: policy.verify_timestamp,
                certificate_chain: policy.verify_certificate,
                signed_certificate_timestamp: policy.verify_sct,
            }),
            certificate_issuer: Some(issuer.clone()),
            certificate_identity: identity.clone(),
            attestation_statement: attestation_statement.clone(),
            subject_canonical_url: entry.canonical_url.clone(),
            subject_sif_hash: entry.sif_hash.clone(),
        };
        let applied =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(entry, &report)
                .map_err(|error| {
                    format!(
                        "sigstore verification evidence {} rejected for {}: {error}",
                        path_string(&bundle),
                        entry.canonical_url
                    )
                })?;
        if applied {
            applied_count += 1;
        }
    }
    if matched_count == 0 {
        return Err(format!(
            "omena.lock contains no entries for package or canonical URL selector '{package}'"
        ));
    }
    if applied_count == 0 {
        return Err(format!(
            "sigstore verification evidence {} did not match any selected lock entry subject",
            path_string(&bundle)
        ));
    }

    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    if json {
        print_json(&lock)?;
    } else {
        println!(
            "omena.lock sigstore verification updated: {applied_count} entr{} recorded from {}",
            if applied_count == 1 { "y" } else { "ies" },
            path_string(&bundle)
        );
    }

    Ok(())
}

fn extract_verified_attestation_statement(
    bundle: &sigstore_verify::types::Bundle,
    policy: &AttestationStatementPolicy,
    require_provenance_statement: bool,
) -> Result<Option<OmenaSifAttestationStatementV1>, String> {
    let sigstore_verify::types::SignatureContent::DsseEnvelope(envelope) = &bundle.content else {
        if policy.is_empty() && !require_provenance_statement {
            return Ok(None);
        }
        return Err(
            "lock verify-attestation provenance evidence requires a DSSE in-toto provenance bundle"
                .to_string(),
        );
    };
    if envelope.payload_type != "application/vnd.in-toto+json" {
        if policy.is_empty() && !require_provenance_statement {
            return Ok(None);
        }
        return Err(format!(
            "lock verify-attestation provenance evidence requires payloadType application/vnd.in-toto+json, got {}",
            envelope.payload_type
        ));
    }
    let payload = envelope.decode_payload();
    let statement: serde_json::Value = serde_json::from_slice(&payload)
        .map_err(|error| format!("failed to parse verified in-toto statement payload: {error}"))?;
    let statement = summarize_verified_attestation_statement(&statement);
    require_statement_policy_matches(&statement, policy)?;
    Ok(Some(statement))
}

fn attestation_kind_requires_provenance_statement(kind: &str) -> bool {
    kind.starts_with("npm-provenance.") || kind.starts_with("omena-toolchain.")
}

fn summarize_verified_attestation_statement(
    statement: &serde_json::Value,
) -> OmenaSifAttestationStatementV1 {
    let (repository_from_config, ref_from_config) = statement
        .pointer("/predicate/invocation/configSource/uri")
        .and_then(|value| value.as_str())
        .map(split_git_source_uri)
        .unwrap_or((None, None));
    let (repository_from_material, ref_from_material) = statement
        .pointer("/predicate/materials")
        .and_then(|value| value.as_array())
        .and_then(|materials| {
            materials
                .iter()
                .filter_map(|material| material.get("uri").and_then(|value| value.as_str()))
                .map(split_git_source_uri)
                .find(|(repository, _)| repository.is_some())
        })
        .unwrap_or((None, None));
    let source_repository = first_owned_string([
        statement_string(
            statement,
            "/predicate/buildDefinition/externalParameters/workflow/repository",
        ),
        statement_string(
            statement,
            "/predicate/invocation/environment/GITHUB_REPOSITORY",
        )
        .map(|repository| github_repository_url(repository.as_str())),
        repository_from_config,
        repository_from_material,
    ]);
    let source_ref = first_owned_string([
        statement_string(
            statement,
            "/predicate/buildDefinition/externalParameters/workflow/ref",
        ),
        statement_string(statement, "/predicate/invocation/environment/GITHUB_REF"),
        ref_from_config,
        ref_from_material,
    ]);
    let source_commit = first_owned_string([
        statement
            .pointer("/predicate/buildDefinition/resolvedDependencies")
            .and_then(|value| value.as_array())
            .and_then(|dependencies| {
                dependencies.iter().find_map(|dependency| {
                    dependency
                        .pointer("/digest/gitCommit")
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned)
                })
            }),
        statement_string(statement, "/predicate/invocation/configSource/digest/sha1"),
        statement
            .pointer("/predicate/materials")
            .and_then(|value| value.as_array())
            .and_then(|materials| {
                materials.iter().find_map(|material| {
                    material
                        .pointer("/digest/sha1")
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned)
                })
            }),
    ]);
    let subject_digests = statement
        .pointer("/subject")
        .and_then(|value| value.as_array())
        .map(|subjects| {
            let mut result = Vec::new();
            for subject in subjects {
                let Some(name) = subject.get("name").and_then(|value| value.as_str()) else {
                    continue;
                };
                let Some(digests) = subject.get("digest").and_then(|value| value.as_object())
                else {
                    continue;
                };
                for (algorithm, digest) in digests {
                    if let Some(digest) = digest.as_str() {
                        result.push(OmenaSifAttestationSubjectDigestV1 {
                            name: name.to_string(),
                            algorithm: algorithm.to_string(),
                            digest: digest.to_string(),
                        });
                    }
                }
            }
            result
        })
        .unwrap_or_default();

    OmenaSifAttestationStatementV1 {
        statement_type: statement_string(statement, "/_type"),
        predicate_type: statement_string(statement, "/predicateType"),
        source_repository,
        source_ref,
        source_commit,
        builder_id: first_owned_string([
            statement_string(statement, "/predicate/runDetails/builder/id"),
            statement_string(statement, "/predicate/builder/id"),
        ]),
        build_type: first_owned_string([
            statement_string(statement, "/predicate/buildDefinition/buildType"),
            statement_string(statement, "/predicate/buildType"),
        ]),
        subject_names: statement
            .pointer("/subject")
            .and_then(|value| value.as_array())
            .map(|subjects| {
                subjects
                    .iter()
                    .filter_map(|subject| {
                        subject
                            .get("name")
                            .and_then(|value| value.as_str())
                            .map(ToOwned::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        subject_digests,
    }
}

fn parse_attestation_statement_subject_digest_policy(
    input: &str,
) -> Result<AttestationStatementSubjectDigestPolicy, String> {
    let input = input.trim();
    let Some((name, digest_with_algorithm)) = input.split_once('=') else {
        return Err(
            "lock verify-attestation --statement-subject-digest must use name=algorithm:digest"
                .to_string(),
        );
    };
    let Some((algorithm, digest)) = digest_with_algorithm.split_once(':') else {
        return Err(
            "lock verify-attestation --statement-subject-digest must use name=algorithm:digest"
                .to_string(),
        );
    };
    let name = name.trim();
    let algorithm = algorithm.trim();
    let digest = digest.trim();
    if name.is_empty() || algorithm.is_empty() || digest.is_empty() {
        return Err(
            "lock verify-attestation --statement-subject-digest must include non-empty name, algorithm, and digest"
                .to_string(),
        );
    }
    Ok(AttestationStatementSubjectDigestPolicy {
        name: name.to_string(),
        algorithm: algorithm.to_string(),
        digest: digest.to_string(),
    })
}

fn require_statement_policy_matches(
    statement: &OmenaSifAttestationStatementV1,
    policy: &AttestationStatementPolicy,
) -> Result<(), String> {
    for (field, flag, expected, observed) in [
        (
            "statementType",
            "--statement-type",
            policy.statement_type.as_ref(),
            statement.statement_type.as_ref(),
        ),
        (
            "predicateType",
            "--statement-predicate-type",
            policy.predicate_type.as_ref(),
            statement.predicate_type.as_ref(),
        ),
        (
            "sourceRepository",
            "--statement-source-repository",
            policy.source_repository.as_ref(),
            statement.source_repository.as_ref(),
        ),
        (
            "sourceRef",
            "--statement-source-ref",
            policy.source_ref.as_ref(),
            statement.source_ref.as_ref(),
        ),
        (
            "sourceCommit",
            "--statement-source-commit",
            policy.source_commit.as_ref(),
            statement.source_commit.as_ref(),
        ),
        (
            "builderId",
            "--statement-builder-id",
            policy.builder_id.as_ref(),
            statement.builder_id.as_ref(),
        ),
        (
            "buildType",
            "--statement-build-type",
            policy.build_type.as_ref(),
            statement.build_type.as_ref(),
        ),
    ] {
        if let Some(expected) = expected {
            if expected.trim().is_empty() {
                return Err(format!("lock verify-attestation {flag} must not be empty"));
            }
            match observed {
                Some(observed) if observed == expected => {}
                Some(observed) => {
                    return Err(format!(
                        "verified attestation statement {field} mismatch: expected {expected}, got {observed}"
                    ));
                }
                None => {
                    return Err(format!(
                        "verified attestation statement missing required {field}: expected {expected}"
                    ));
                }
            }
        }
    }
    for expected in &policy.subject_names {
        if expected.trim().is_empty() {
            return Err(
                "lock verify-attestation --statement-subject-name must not be empty".to_string(),
            );
        }
        if !statement
            .subject_names
            .iter()
            .any(|subject| subject == expected)
        {
            return Err(format!(
                "verified attestation statement missing required subjectName: expected {expected}"
            ));
        }
    }
    for expected in &policy.subject_digests {
        let expected = parse_attestation_statement_subject_digest_policy(expected)?;
        if !statement.subject_digests.iter().any(|observed| {
            observed.name == expected.name
                && observed.algorithm == expected.algorithm
                && observed.digest == expected.digest
        }) {
            return Err(format!(
                "verified attestation statement missing required subjectDigest: expected {}={}:{}",
                expected.name, expected.algorithm, expected.digest
            ));
        }
    }
    Ok(())
}

fn statement_string(statement: &serde_json::Value, pointer: &str) -> Option<String> {
    statement
        .pointer(pointer)
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn first_owned_string(values: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

fn split_git_source_uri(uri: &str) -> (Option<String>, Option<String>) {
    let source = uri.strip_prefix("git+").unwrap_or(uri);
    let Some((repository, source_ref)) = source.rsplit_once('@') else {
        return (Some(source.to_string()), None);
    };
    (Some(repository.to_string()), Some(source_ref.to_string()))
}

fn github_repository_url(repository: &str) -> String {
    if repository.starts_with("http://") || repository.starts_with("https://") {
        repository.to_string()
    } else {
        format!("https://github.com/{repository}")
    }
}

struct VerifiedT3AttestationArtifactBinding {
    canonical_url: String,
    sif_hash: omena_sif::OmenaSifDigestV1,
    artifact_sha256: String,
}

fn read_verified_t3_attestation_artifact_binding(
    artifact: &Path,
    artifact_bytes: &[u8],
) -> Result<VerifiedT3AttestationArtifactBinding, String> {
    let artifact_source = std::str::from_utf8(artifact_bytes).map_err(|error| {
        format!(
            "lock verify-attestation --verified-tier t3 requires --artifact to be the SIF JSON for the selected lock entry; {} is not UTF-8 JSON: {error}",
            path_string(artifact)
        )
    })?;
    let sif = read_omena_sif_json_v1(artifact_source).map_err(|error| {
        format!(
            "lock verify-attestation --verified-tier t3 requires --artifact to be the SIF JSON for the selected lock entry; failed to parse {}: {error}",
            path_string(artifact)
        )
    })?;
    let sif_hash = compute_omena_sif_artifact_hash_v1(&sif).map_err(|error| {
        format!(
            "lock verify-attestation --verified-tier t3 failed to hash SIF artifact {}: {error}",
            path_string(artifact)
        )
    })?;
    Ok(VerifiedT3AttestationArtifactBinding {
        canonical_url: sif.canonical_url,
        sif_hash,
        artifact_sha256: sha256_hex(artifact_bytes),
    })
}

fn validate_verified_t3_attestation_artifact_binding(
    entry: &omena_sif::OmenaLockSifEntryV1,
    binding: &VerifiedT3AttestationArtifactBinding,
) -> Result<(), String> {
    if binding.canonical_url != entry.canonical_url {
        return Err(format!(
            "lock verify-attestation --verified-tier t3 requires --artifact to match the selected SIF subject; artifact canonical URL {} does not match lock entry {}",
            binding.canonical_url, entry.canonical_url
        ));
    }
    if binding.sif_hash != entry.sif_hash {
        return Err(format!(
            "lock verify-attestation --verified-tier t3 requires --artifact to match the selected SIF subject; artifact hash {} does not match lock entry {}",
            binding.sif_hash.as_str(),
            entry.sif_hash.as_str()
        ));
    }
    Ok(())
}

fn validate_verified_t3_attestation_statement_binding(
    entry: &omena_sif::OmenaLockSifEntryV1,
    binding: &VerifiedT3AttestationArtifactBinding,
    statement: Option<&OmenaSifAttestationStatementV1>,
) -> Result<(), String> {
    let statement = statement.ok_or_else(|| {
        "lock verify-attestation --verified-tier t3 requires a signed SIF provenance statement"
            .to_string()
    })?;
    if !statement
        .subject_names
        .iter()
        .any(|subject| subject == &entry.sif_path)
    {
        return Err(format!(
            "lock verify-attestation --verified-tier t3 requires the signed provenance statement subjectNames to include selected SIF path {}",
            entry.sif_path
        ));
    }
    if !statement.subject_digests.iter().any(|digest| {
        digest.name == entry.sif_path
            && digest.algorithm.eq_ignore_ascii_case("sha256")
            && digest.digest.eq_ignore_ascii_case(&binding.artifact_sha256)
    }) {
        return Err(format!(
            "lock verify-attestation --verified-tier t3 requires the signed provenance statement subjectDigests to bind {} to sha256:{}",
            entry.sif_path, binding.artifact_sha256
        ));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn validate_attestation_policy_for_verified_tier(
    kind: &str,
    verified_trust_tier: omena_sif::OmenaSifTrustTierV1,
    certificate_issuer: Option<&str>,
    certificate_identity: Option<&str>,
) -> Result<(), String> {
    if verified_trust_tier == omena_sif::OmenaSifTrustTierV1::T3 {
        if !kind.starts_with("omena-toolchain.") {
            return Err(format!(
                "lock verify-attestation --verified-tier t3 requires --kind omena-toolchain.*, got {kind}"
            ));
        }
        if certificate_issuer.is_none_or(|issuer| issuer.trim().is_empty()) {
            return Err("lock verify-attestation --verified-tier t3 requires --issuer".to_string());
        }
        if certificate_identity.is_none_or(|identity| identity.trim().is_empty()) {
            return Err(
                "lock verify-attestation --verified-tier t3 requires --identity".to_string(),
            );
        }
    }
    Ok(())
}

fn build_lock_entries_from_sif_paths(
    lockfile: &Path,
    sif_paths: &[PathBuf],
) -> Result<Vec<omena_sif::OmenaLockSifEntryV1>, String> {
    let mut entries = Vec::with_capacity(sif_paths.len());
    for sif_path in sif_paths {
        let sif_json = read_source(sif_path)?;
        let sif = read_omena_sif_json_v1(&sif_json)
            .map_err(|error| format!("failed to parse SIF {}: {error}", path_string(sif_path)))?;
        let entry_path = relativize_lock_sif_path(lockfile, sif_path);
        let entry = build_omena_lock_sif_entry_v1(entry_path, &sif).map_err(|error| {
            format!(
                "failed to build lock entry for {}: {error}",
                path_string(sif_path)
            )
        })?;
        entries.push(entry);
    }
    Ok(entries)
}

fn read_lockfile_or_empty(lockfile: &Path) -> Result<OmenaLockV1, String> {
    if !lockfile.exists() {
        return Ok(OmenaLockV1::new(Vec::new()));
    }
    let lockfile_source = read_source(lockfile)?;
    read_omena_lock_json_v1(&lockfile_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(lockfile)))
}

fn filter_lock_entries_for_package(
    entries: Vec<omena_sif::OmenaLockSifEntryV1>,
    package: &str,
) -> Result<Vec<omena_sif::OmenaLockSifEntryV1>, String> {
    let filtered = entries
        .into_iter()
        .filter(|entry| lock_entry_matches_package_selector(entry, package))
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Err(format!(
            "no provided SIF entries match package or canonical URL selector '{package}'"
        ));
    }
    Ok(filtered)
}

fn lock_entry_matches_package_selector(
    entry: &omena_sif::OmenaLockSifEntryV1,
    selector: &str,
) -> bool {
    let canonical_url = entry
        .canonical_url
        .strip_prefix("pkg:")
        .unwrap_or(&entry.canonical_url);
    let selector = selector.strip_prefix("pkg:").unwrap_or(selector);
    canonical_url == selector || canonical_url.starts_with(&format!("{selector}/"))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LockStatusReportV1 {
    product: &'static str,
    lockfile: String,
    present: bool,
    lockfile_version: Option<String>,
    omena_min_version: Option<String>,
    running_omena_version: &'static str,
    entry_count: usize,
    t0_entry_count: usize,
    t1_entry_count: usize,
    t2_entry_count: usize,
    t3_entry_count: usize,
}

fn lock_status(lockfile: PathBuf, json: bool) -> Result<(), String> {
    let report = if lockfile.exists() {
        let lockfile_source = read_source(&lockfile)?;
        let lock = read_omena_lock_json_v1(&lockfile_source)
            .map_err(|error| format!("failed to parse {}: {error}", path_string(&lockfile)))?;
        LockStatusReportV1 {
            product: "omena-cli.lock-status",
            lockfile: path_string(&lockfile),
            present: true,
            lockfile_version: Some(lock.lockfile_version.clone()),
            omena_min_version: lock.omena_min_version.clone(),
            running_omena_version: env!("CARGO_PKG_VERSION"),
            entry_count: lock.entries.len(),
            t0_entry_count: lock
                .entries
                .iter()
                .filter(|entry| entry.trust_tier == omena_sif::OmenaSifTrustTierV1::T0)
                .count(),
            t1_entry_count: lock
                .entries
                .iter()
                .filter(|entry| entry.trust_tier == omena_sif::OmenaSifTrustTierV1::T1)
                .count(),
            t2_entry_count: lock
                .entries
                .iter()
                .filter(|entry| entry.trust_tier == omena_sif::OmenaSifTrustTierV1::T2)
                .count(),
            t3_entry_count: lock
                .entries
                .iter()
                .filter(|entry| entry.trust_tier == omena_sif::OmenaSifTrustTierV1::T3)
                .count(),
        }
    } else {
        LockStatusReportV1 {
            product: "omena-cli.lock-status",
            lockfile: path_string(&lockfile),
            present: false,
            lockfile_version: None,
            omena_min_version: None,
            running_omena_version: env!("CARGO_PKG_VERSION"),
            entry_count: 0,
            t0_entry_count: 0,
            t1_entry_count: 0,
            t2_entry_count: 0,
            t3_entry_count: 0,
        }
    };

    if json {
        print_json(&report)?;
    } else if report.present {
        println!(
            "omena.lock status: {} entr{} at {} (requires omena >= {})",
            report.entry_count,
            if report.entry_count == 1 { "y" } else { "ies" },
            report.lockfile,
            report.omena_min_version.as_deref().unwrap_or("unspecified")
        );
    } else {
        println!(
            "omena.lock status: no lockfile found at {}",
            report.lockfile
        );
    }
    Ok(())
}

/// Store `sif_path` so it round-trips through [`resolve_lock_relative_path`]:
/// drop the lockfile's parent prefix when `sif_path` lives under it, otherwise
/// keep the path as authored (already relative to the lockfile location).
fn relativize_lock_sif_path(lockfile: &Path, sif_path: &Path) -> String {
    let parent = lockfile
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    if let Some(parent) = parent
        && let Ok(stripped) = sif_path.strip_prefix(parent)
    {
        return path_string(stripped);
    }
    path_string(sif_path)
}

fn lock_verify(
    lockfile: PathBuf,
    source_paths: Vec<PathBuf>,
    frozen: bool,
    tier: Option<String>,
    json: bool,
) -> Result<(), String> {
    if !frozen {
        return Err("omena lock verify currently requires --frozen".to_string());
    }

    let lockfile_source = read_source(&lockfile)?;
    let lock = read_omena_lock_json_v1(&lockfile_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(&lockfile)))?;
    let mut report = verify_omena_lock_frozen_v1(&lock, |entry| {
        let sif_path = resolve_lock_relative_path(&lockfile, &entry.sif_path);
        read_source(&sif_path)
    });
    let source_coverage_issues =
        collect_lock_source_coverage_issues(&lock, source_paths.as_slice())?;
    if !source_coverage_issues.is_empty() {
        report.verified = false;
        report.issues.extend(source_coverage_issues);
    }
    if let Some(minimum_tier) = parse_lock_trust_tier(tier.as_deref())? {
        let tier_issues = collect_lock_trust_tier_issues(&lock, minimum_tier);
        if !tier_issues.is_empty() {
            report.verified = false;
            report.issues.extend(tier_issues);
        }
    }

    if json {
        print_json(&report)?;
    } else if report.verified {
        println!(
            "omena.lock frozen verification passed: {} SIF entr{} checked",
            report.entries_checked,
            if report.entries_checked == 1 {
                "y"
            } else {
                "ies"
            }
        );
    } else {
        println!(
            "omena.lock frozen verification failed: {} issue{}",
            report.issues.len(),
            if report.issues.len() == 1 { "" } else { "s" }
        );
        for issue in &report.issues {
            println!("{} {}: {}", issue.code, issue.sif_path, issue.message);
        }
    }

    if !report.verified {
        return Err("omena.lock frozen verification failed".to_string());
    }
    Ok(())
}

fn parse_lock_trust_tier(
    tier: Option<&str>,
) -> Result<Option<omena_sif::OmenaSifTrustTierV1>, String> {
    let Some(tier) = tier else {
        return Ok(None);
    };
    match tier {
        "t0" | "T0" => Ok(Some(omena_sif::OmenaSifTrustTierV1::T0)),
        "t1" | "T1" => Ok(Some(omena_sif::OmenaSifTrustTierV1::T1)),
        "t2" | "T2" => Ok(Some(omena_sif::OmenaSifTrustTierV1::T2)),
        "t3" | "T3" => Ok(Some(omena_sif::OmenaSifTrustTierV1::T3)),
        _ => Err(format!(
            "unsupported omena.lock trust tier '{tier}'; expected t0, t1, t2, or t3"
        )),
    }
}

fn collect_lock_trust_tier_issues(
    lock: &OmenaLockV1,
    minimum_tier: omena_sif::OmenaSifTrustTierV1,
) -> Vec<OmenaLockVerificationIssueV1> {
    let mut issues = Vec::new();
    for entry in &lock.entries {
        if entry.trust_tier < minimum_tier {
            issues.push(OmenaLockVerificationIssueV1 {
                canonical_url: entry.canonical_url.clone(),
                sif_path: entry.sif_path.clone(),
                code: "trustTierBelowMinimum".to_string(),
                message: format!(
                    "lock entry trust tier {} is below required {}",
                    entry.trust_tier.as_str(),
                    minimum_tier.as_str()
                ),
            });
            continue;
        }
        if minimum_tier >= omena_sif::OmenaSifTrustTierV1::T2 {
            let mut candidate_count = 0usize;
            let mut invalid_reasons = Vec::new();
            let mut has_valid_evidence = false;
            for verification in &entry.attestation_verifications {
                if verification.verified_trust_tier < minimum_tier {
                    continue;
                }
                candidate_count += 1;
                match omena_sif::validate_omena_sif_lock_entry_attestation_verification_v1(
                    entry,
                    verification,
                ) {
                    Ok(()) => {
                        has_valid_evidence = true;
                        break;
                    }
                    Err(error) => invalid_reasons.push(error),
                }
            }
            if has_valid_evidence {
                continue;
            }
            let (code, message) = if candidate_count == 0 {
                (
                    "attestationVerificationMissing",
                    format!(
                        "lock entry trust tier {} must carry verified attestation evidence for required {}",
                        entry.trust_tier.as_str(),
                        minimum_tier.as_str()
                    ),
                )
            } else {
                (
                    "attestationVerificationInvalid",
                    format!(
                        "lock entry trust tier {} has attestation evidence for required {}, but the evidence is invalid: {}",
                        entry.trust_tier.as_str(),
                        minimum_tier.as_str(),
                        invalid_reasons
                            .first()
                            .map_or("unknown validation failure", String::as_str)
                    ),
                )
            };
            issues.push(OmenaLockVerificationIssueV1 {
                canonical_url: entry.canonical_url.clone(),
                sif_path: entry.sif_path.clone(),
                code: code.to_string(),
                message,
            });
        }
    }
    issues
}

fn collect_lock_source_coverage_issues(
    lock: &OmenaLockV1,
    source_paths: &[PathBuf],
) -> Result<Vec<OmenaLockVerificationIssueV1>, String> {
    if source_paths.is_empty() {
        return Ok(Vec::new());
    }

    let locked_sources = lock
        .entries
        .iter()
        .map(|entry| entry.canonical_url.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut missing = std::collections::BTreeSet::new();
    for source_path in source_paths {
        let style_source = read_source(source_path)?;
        let style_path = path_string(source_path);
        let Some(module_sources) =
            summarize_omena_query_sass_module_sources(&style_path, &style_source)
        else {
            continue;
        };
        for module_source in module_sources
            .module_use_edges
            .iter()
            .map(|edge| edge.source.as_str())
            .chain(
                module_sources
                    .module_forward_sources
                    .iter()
                    .map(String::as_str),
            )
        {
            if !lock_source_requires_sif(module_source) {
                continue;
            }
            if locked_sources.contains(module_source) {
                continue;
            }
            missing.insert((module_source.to_string(), style_path.clone()));
        }
    }

    Ok(missing
        .into_iter()
        .map(|(canonical_url, style_path)| OmenaLockVerificationIssueV1 {
            canonical_url: canonical_url.clone(),
            sif_path: style_path.clone(),
            code: "sourceSifMissingFromLock".to_string(),
            message: format!(
                "source {} references external Sass module '{}' but omena.lock has no matching SIF entry",
                style_path, canonical_url
            ),
        })
        .collect())
}

fn lock_source_requires_sif(source: &str) -> bool {
    if source.starts_with("sass:") {
        return false;
    }
    if source.starts_with('.') || source.starts_with('/') || source.starts_with("file://") {
        return false;
    }
    true
}

fn resolve_lock_relative_path(lockfile: &Path, entry_path: &str) -> PathBuf {
    let entry_path = Path::new(entry_path);
    if entry_path.is_absolute() {
        return entry_path.to_path_buf();
    }
    lockfile
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(entry_path)
}

fn sif_command(command: SifCommand) -> Result<(), String> {
    match command {
        SifCommand::Generate {
            path,
            canonical_url,
            output,
            syntax,
            json,
        } => generate_sif(path, canonical_url, output, syntax, json),
    }
}

fn provenance_command(command: ProvenanceCommand) -> Result<(), String> {
    match command {
        ProvenanceCommand::Status { lockfile, json } => provenance_status(lockfile, json),
    }
}

fn report_command(command: ReportCommand) -> Result<(), String> {
    match command {
        ReportCommand::Soundiness {
            source_paths,
            source_document_paths,
            package_manifest_paths,
            sif_paths,
            lockfile,
            external,
            no_suppress,
            max_suppressions,
            report_stale_suppressions,
            json,
        } => report_soundiness(
            source_paths,
            source_document_paths,
            package_manifest_paths,
            sif_paths,
            lockfile,
            external,
            no_suppress,
            max_suppressions,
            report_stale_suppressions,
            json,
        ),
        ReportCommand::ResolutionPolicy { json } => report_resolution_policy(json),
    }
}

fn report_resolution_policy(json: bool) -> Result<(), String> {
    let report = summarize_omena_query_style_resolution_policy_v0();
    if json {
        print_json(&report)?;
    } else {
        println!(
            "{} candidateStrategy={} networkAccess={}",
            report.product, report.candidate_strategy, report.network_access
        );
        for step in &report.steps {
            println!(
                "{} {}: {} ({})",
                step.order, step.key, step.precedence, step.candidate_semantics
            );
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessReportV0 {
    schema_version: &'static str,
    product: &'static str,
    file_count: usize,
    line_count: usize,
    original_diagnostic_count: usize,
    emitted_diagnostic_count: usize,
    suppressed_diagnostic_count: usize,
    unused_expect_error_count: usize,
    diagnostic_suppression_mode: &'static str,
    boundary_diagnostics: SoundinessBoundaryDiagnosticsV0,
    strictness_distribution: SoundinessStrictnessDistributionV0,
    suppression_reasons: Vec<OmenaQueryDiagnosticSuppressionReasonV0>,
    file_reports: Vec<SoundinessFileReportV0>,
    noise_budget: SoundinessNoiseBudgetV0,
    ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessBoundaryDiagnosticsV0 {
    stale_external_sif: usize,
    partial_external_sif: usize,
    missing_external_sif: usize,
    unresolved_external_reference: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessStrictnessDistributionV0 {
    relaxed: usize,
    standard: usize,
    strict: usize,
    closed: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessFileReportV0 {
    file_uri: String,
    line_count: usize,
    original_diagnostic_count: usize,
    emitted_diagnostic_count: usize,
    suppressed_diagnostic_count: usize,
    unused_expect_error_count: usize,
    diagnostic_suppression_mode: &'static str,
    suppression_reasons: Vec<OmenaQueryDiagnosticSuppressionReasonV0>,
    suppressed_per_100_loc: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessNoiseBudgetV0 {
    per_pr_suppressed_diagnostic_ratio: SoundinessNoiseBudgetCheckV0,
    per_file_suppressed_density: SoundinessNoiseBudgetCheckV0,
    project_suppression_rate: SoundinessNoiseBudgetCheckV0,
    within_budget: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SoundinessNoiseBudgetCheckV0 {
    metric: &'static str,
    value: f64,
    threshold: f64,
    status: &'static str,
}

#[allow(clippy::too_many_arguments)]
fn report_soundiness(
    source_paths: Vec<PathBuf>,
    source_document_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    sif_paths: Vec<PathBuf>,
    lockfile: Option<PathBuf>,
    external: String,
    no_suppress: bool,
    max_suppressions: Option<usize>,
    report_stale_suppressions: bool,
    json: bool,
) -> Result<(), String> {
    let report = summarize_soundiness_report(
        source_paths,
        source_document_paths,
        package_manifest_paths,
        sif_paths,
        lockfile,
        external,
        no_suppress,
    )?;
    enforce_soundiness_report_audit_flags(&report, max_suppressions, report_stale_suppressions)?;
    if json {
        print_json(&report)?;
    } else {
        println!("files analysed: {}", report.file_count);
        println!("diagnostics emitted: {}", report.emitted_diagnostic_count);
        println!(
            "diagnostics suppressed: {}",
            report.suppressed_diagnostic_count
        );
        println!(
            "noise budget: {}",
            if report.noise_budget.within_budget {
                "within limits"
            } else {
                "review recommended"
            }
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn summarize_soundiness_report(
    source_paths: Vec<PathBuf>,
    source_document_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    sif_paths: Vec<PathBuf>,
    lockfile: Option<PathBuf>,
    external: String,
    no_suppress: bool,
) -> Result<SoundinessReportV0, String> {
    if source_paths.is_empty() {
        return Err("omena report soundiness requires at least one --source <path>".to_string());
    }

    let style_sources = read_style_sources(source_paths.as_slice())?;
    let source_documents = read_source_documents(source_document_paths.as_slice())?;
    let package_manifests = read_package_manifests(package_manifest_paths.as_slice())?;
    let mut external_sifs = read_external_sifs(sif_paths.as_slice())?;
    if let Some(lockfile) = lockfile.as_ref() {
        external_sifs.extend(read_lock_external_sifs(lockfile)?);
    }
    let in_process_external_sifs =
        resolve_in_process_external_sifs(style_sources.as_slice(), external_sifs.as_slice());
    external_sifs.extend(in_process_external_sifs);
    let external_mode = parse_external_module_mode(&external)?;
    let suppression_mode = if no_suppress {
        OmenaQueryDiagnosticSuppressionModeV0::ReportOnly
    } else {
        OmenaQueryDiagnosticSuppressionModeV0::Apply
    };

    let mut boundary_diagnostics = SoundinessBoundaryDiagnosticsV0::default();
    let mut strictness_distribution = SoundinessStrictnessDistributionV0::default();
    let mut file_reports = Vec::new();
    let mut original_diagnostic_count = 0usize;
    let mut emitted_diagnostic_count = 0usize;
    let mut suppressed_diagnostic_count = 0usize;
    let mut unused_expect_error_count = 0usize;
    let mut suppression_reasons = Vec::new();
    let mut line_count = 0usize;

    for source in &style_sources {
        let summary =
            summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_suppression_mode(
                source.style_path.as_str(),
                style_sources.as_slice(),
                source_documents.as_slice(),
                package_manifests.as_slice(),
                None,
                external_mode,
                external_sifs.as_slice(),
                suppression_mode,
            )
            .ok_or_else(|| {
                format!(
                    "failed to read workspace style diagnostics for {}",
                    source.style_path
                )
            })?;
        let file_line_count = source.style_source.lines().count().max(1);
        line_count += file_line_count;
        strictness_distribution.add(parse_report_strictness_label(&source.style_source));
        boundary_diagnostics.add_summary(&summary);
        let suppression = summary.suppression_summary.as_ref();
        let original = suppression
            .map(|summary| summary.original_diagnostic_count)
            .unwrap_or(summary.diagnostic_count);
        let emitted = summary.diagnostic_count;
        let suppressed = suppression
            .map(|summary| summary.suppressed_diagnostic_count)
            .unwrap_or(0);
        let unused_expect_errors = suppression
            .map(|summary| summary.unused_expect_error_count)
            .unwrap_or(0);
        let file_suppression_reasons = suppression
            .map(|summary| summary.suppression_reasons.clone())
            .unwrap_or_default();
        original_diagnostic_count += original;
        emitted_diagnostic_count += emitted;
        suppressed_diagnostic_count += suppressed;
        unused_expect_error_count += unused_expect_errors;
        suppression_reasons.extend(file_suppression_reasons.iter().cloned());
        file_reports.push(SoundinessFileReportV0 {
            file_uri: source.style_path.clone(),
            line_count: file_line_count,
            original_diagnostic_count: original,
            emitted_diagnostic_count: emitted,
            suppressed_diagnostic_count: suppressed,
            unused_expect_error_count: unused_expect_errors,
            diagnostic_suppression_mode: suppression_mode.as_str(),
            suppression_reasons: file_suppression_reasons,
            suppressed_per_100_loc: ratio_per_100(suppressed, file_line_count),
        });
    }

    let max_file_suppressed_density = file_reports
        .iter()
        .map(|report| report.suppressed_per_100_loc)
        .fold(0.0_f64, f64::max);
    let per_pr_ratio = percentage(suppressed_diagnostic_count, original_diagnostic_count);
    let project_suppression_rate = per_pr_ratio;
    let noise_budget = SoundinessNoiseBudgetV0 {
        per_pr_suppressed_diagnostic_ratio: noise_budget_check(
            "perPrSuppressedDiagnosticRatio",
            per_pr_ratio,
            30.0,
        ),
        per_file_suppressed_density: noise_budget_check(
            "perFileSuppressedDiagnosticsPer100Loc",
            max_file_suppressed_density,
            5.0,
        ),
        project_suppression_rate: noise_budget_check(
            "projectSuppressionRate",
            project_suppression_rate,
            20.0,
        ),
        within_budget: per_pr_ratio <= 30.0
            && max_file_suppressed_density <= 5.0
            && project_suppression_rate <= 20.0,
    };

    Ok(SoundinessReportV0 {
        schema_version: "0",
        product: "omena-cli.soundiness-report",
        file_count: style_sources.len(),
        line_count,
        original_diagnostic_count,
        emitted_diagnostic_count,
        suppressed_diagnostic_count,
        unused_expect_error_count,
        diagnostic_suppression_mode: suppression_mode.as_str(),
        boundary_diagnostics,
        strictness_distribution,
        suppression_reasons,
        file_reports,
        noise_budget,
        ready_surfaces: vec![
            "soundinessReport",
            "externalBoundaryStateSummary",
            "diagnosticSuppressionRateSummary",
            "diagnosticSuppressionReasonSummary",
            "noiseBudgetVisibilityGates",
        ],
    })
}

fn enforce_soundiness_report_audit_flags(
    report: &SoundinessReportV0,
    max_suppressions: Option<usize>,
    report_stale_suppressions: bool,
) -> Result<(), String> {
    if let Some(max_suppressions) = max_suppressions
        && report.suppressed_diagnostic_count > max_suppressions
    {
        return Err(format!(
            "suppression budget exceeded: {} suppressions observed, max {}",
            report.suppressed_diagnostic_count, max_suppressions
        ));
    }
    if report_stale_suppressions && report.unused_expect_error_count > 0 {
        return Err(format!(
            "stale suppressions observed: {} unused omena-expect-error directives",
            report.unused_expect_error_count
        ));
    }
    Ok(())
}

impl SoundinessBoundaryDiagnosticsV0 {
    fn add_summary(&mut self, summary: &OmenaQueryStyleDiagnosticsForFileV0) {
        for diagnostic in &summary.diagnostics {
            match diagnostic.code {
                "staleExternalSif" => self.stale_external_sif += 1,
                "partialExternalSif" => self.partial_external_sif += 1,
                "missingExternalSif" => self.missing_external_sif += 1,
                "unresolvedExternalReference" => self.unresolved_external_reference += 1,
                _ => {}
            }
        }
    }
}

impl SoundinessStrictnessDistributionV0 {
    fn add(&mut self, strictness: &'static str) {
        match strictness {
            "relaxed" => self.relaxed += 1,
            "strict" => self.strict += 1,
            "closed" => self.closed += 1,
            _ => self.standard += 1,
        }
    }
}

fn parse_report_strictness_label(source: &str) -> &'static str {
    let mut level = "standard";
    for line in source.lines() {
        let Some(offset) = line.find("@omena-strict") else {
            continue;
        };
        let tail = &line[offset + "@omena-strict".len()..];
        for token in tail
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
            .filter(|token| !token.is_empty())
        {
            match token {
                "relaxed" => level = "relaxed",
                "standard" => level = "standard",
                "strict" => level = "strict",
                "closed" => level = "closed",
                _ => {}
            }
        }
    }
    level
}

fn noise_budget_check(
    metric: &'static str,
    value: f64,
    threshold: f64,
) -> SoundinessNoiseBudgetCheckV0 {
    SoundinessNoiseBudgetCheckV0 {
        metric,
        value,
        threshold,
        status: if value <= threshold {
            "within"
        } else {
            "review"
        },
    }
}

fn percentage(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    (numerator as f64 / denominator as f64) * 100.0
}

fn ratio_per_100(count: usize, line_count: usize) -> f64 {
    if line_count == 0 {
        return 0.0;
    }
    (count as f64 / line_count as f64) * 100.0
}

fn provenance_status(lockfile: PathBuf, json: bool) -> Result<(), String> {
    let lockfile_source = read_source(&lockfile)?;
    let lock = read_omena_lock_json_v1(&lockfile_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(&lockfile)))?;
    let report = summarize_omena_sif_provenance_advisory_v1(&lock);

    if json {
        print_json(&report)?;
    } else {
        match report.enforcement {
            "lockVerifyTier2Tier3WhenRequested" => {
                println!(
                    "SIF provenance enforcement is available through `omena lock verify --tier t2|t3`."
                );
            }
            "invalidRecordedAttestationEvidence" => {
                println!(
                    "SIF provenance has invalid recorded attestation evidence; run `omena lock verify --tier t2|t3`."
                );
            }
            _ => {
                println!(
                    "SIF provenance references are advisory until verified attestation evidence is recorded."
                );
            }
        }
        println!(
            "network access: {}; entries: {}",
            report.network_access,
            report.entries.len()
        );
        for entry in &report.entries {
            println!(
                "{} {} attestations={} recordedVerifications={} verifiedAttestations={} invalidAttestations={}: {}",
                entry.trust_tier.as_str(),
                entry.canonical_url,
                entry.attestation_reference_count,
                entry.recorded_attestation_verification_count,
                entry.attestation_verification_count,
                entry.invalid_attestation_verification_count,
                entry.advisory_message
            );
            for policy in &entry.attestation_verification_policies {
                println!(
                    "  verified policy {} kind={} issuer={} identity={}",
                    policy.verified_trust_tier.as_str(),
                    policy.kind,
                    policy
                        .certificate_issuer
                        .as_deref()
                        .unwrap_or("unspecified"),
                    policy
                        .certificate_identity
                        .as_deref()
                        .unwrap_or("unspecified")
                );
            }
            for issue in &entry.invalid_attestation_verification_issues {
                println!(
                    "  invalid attestation {} kind={} verifier={}: {}",
                    issue.verified_trust_tier.as_str(),
                    issue.kind,
                    issue.verifier,
                    issue.reason
                );
            }
        }
    }

    Ok(())
}

fn generate_sif(
    path: PathBuf,
    canonical_url: Option<String>,
    output: Option<PathBuf>,
    syntax: Option<String>,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let syntax = match syntax {
        Some(syntax) => parse_sif_source_syntax(&syntax)?,
        None => infer_sif_source_syntax(&path),
    };
    let canonical_url = canonical_url.unwrap_or_else(|| path_string(&path));
    let sif = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
        canonical_url: &canonical_url,
        source: &source,
        syntax,
    })
    .map_err(|error| format!("failed to generate SIF: {error}"))?;
    let sif_json = write_omena_sif_json_v1(&sif)
        .map_err(|error| format!("failed to serialize SIF: {error}"))?;
    let wrote_output = output.is_some();

    if let Some(output_path) = output {
        fs::write(&output_path, &sif_json).map_err(|error| {
            format!(
                "failed to write SIF artifact to {}: {error}",
                path_string(&output_path)
            )
        })?;
        if !json {
            println!("generated SIF: {}", path_string(&output_path));
        }
    }

    if !wrote_output || json {
        println!("{sif_json}");
    }

    Ok(())
}

fn parse_sif_source_syntax(syntax: &str) -> Result<OmenaSifSourceSyntaxV1, String> {
    match syntax {
        "css" => Ok(OmenaSifSourceSyntaxV1::Css),
        "scss" => Ok(OmenaSifSourceSyntaxV1::Scss),
        "sass" => Ok(OmenaSifSourceSyntaxV1::Sass),
        _ => Err(format!(
            "unsupported SIF source syntax '{syntax}'; expected css, scss, or sass"
        )),
    }
}

fn infer_sif_source_syntax(path: &Path) -> OmenaSifSourceSyntaxV1 {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("css") => OmenaSifSourceSyntaxV1::Css,
        Some("sass") => OmenaSifSourceSyntaxV1::Sass,
        _ => OmenaSifSourceSyntaxV1::Scss,
    }
}

#[cfg(feature = "zk-audit")]
fn audit_command(command: AuditCommand) -> Result<(), String> {
    match command {
        AuditCommand::Zk { command } => zk_audit_command(command),
    }
}

#[cfg(feature = "zk-audit")]
fn zk_audit_command(command: ZkAuditCommand) -> Result<(), String> {
    match command {
        ZkAuditCommand::Prove {
            audit_id,
            reorder,
            json,
        } => {
            let roundtrip = prove_and_verify_canonical_margin_cascade_with_arkworks_v0(reorder)?;
            let verified = roundtrip.proof_generated && roundtrip.proof_verified;
            let result = zk_audit_cli_result_v0(
                "omena-cli.audit.zk.prove",
                "prove",
                Some(cascade_zk_audit_v0(audit_id)),
                None,
                Some(roundtrip),
                verified,
            );
            print_zk_audit_result(&result, json)
        }
        ZkAuditCommand::Verify {
            audit_id,
            reorder,
            json,
        } => {
            let roundtrip = prove_and_verify_canonical_margin_cascade_with_arkworks_v0(reorder)?;
            let verified = roundtrip.proof_generated && roundtrip.proof_verified;
            let result = zk_audit_cli_result_v0(
                "omena-cli.audit.zk.verify",
                "verify",
                Some(cascade_zk_audit_v0(audit_id)),
                None,
                Some(roundtrip),
                verified,
            );
            print_zk_audit_result(&result, json)
        }
        ZkAuditCommand::SetupStatus { json } => {
            let result = zk_audit_cli_result_v0(
                "omena-cli.audit.zk.setup-status",
                "setup-status",
                None,
                Some(zk_audit_ci_matrix_v0()),
                None,
                false,
            );
            print_zk_audit_result(&result, json)
        }
    }
}

#[cfg(feature = "zk-audit")]
fn zk_audit_cli_result_v0(
    product: &'static str,
    command: &'static str,
    audit: Option<CascadeZKAuditV0>,
    ci_matrix: Option<ZKAuditCiMatrixV0>,
    groth16_roundtrip: Option<ArkworksGroth16RoundTripV0>,
    verified: bool,
) -> ZkAuditCliResultV0 {
    ZkAuditCliResultV0 {
        schema_version: "0",
        product,
        layer_marker: "cryptographic-implementation",
        feature_gate: "zk-audit",
        mechanism_scope: ZK_AUDIT_MECHANISM_SCOPE_V0,
        default_proof_backend_enabled: ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0,
        active_proof_backend_scope: active_zk_audit_proof_backend_scope_v0(),
        command,
        audit,
        ci_matrix,
        groth16_roundtrip,
        verified,
    }
}

#[cfg(feature = "zk-audit")]
fn print_zk_audit_result(result: &ZkAuditCliResultV0, json: bool) -> Result<(), String> {
    if json {
        print_json(result)?;
        return Ok(());
    }

    println!("product: {}", result.product);
    println!("command: {}", result.command);
    println!("feature gate: {}", result.feature_gate);
    println!("mechanism scope: {}", result.mechanism_scope);
    println!(
        "default proof backend enabled: {}",
        result.default_proof_backend_enabled
    );
    println!(
        "active proof backend scope: {}",
        result.active_proof_backend_scope
    );
    println!("verified: {}", result.verified);
    if let Some(audit) = &result.audit {
        println!("audit: {}", audit.audit_id);
        println!("setup: {:?}", audit.setup_kind);
        println!("recursion overhead: {}", audit.recursion_overhead);
    }
    if let Some(roundtrip) = &result.groth16_roundtrip {
        println!("backend: {}", roundtrip.backend);
        println!("obligation: {}", roundtrip.obligation_id);
        println!("constraint count: {}", roundtrip.circuit.constraint_count);
        println!("requirement count: {}", roundtrip.requirement_count);
        println!("proof generated: {}", roundtrip.proof_generated);
        println!("proof verified: {}", roundtrip.proof_verified);
    }
    if let Some(ci_matrix) = &result.ci_matrix {
        println!("ci cells: {}", ci_matrix.cells.join(","));
        println!(
            "heavy dependencies default off: {}",
            ci_matrix.heavy_dependencies_default_off
        );
    }
    Ok(())
}

fn check_file(path: PathBuf, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let summary = summarize_omena_query_consumer_check_style_source(&path_string(&path), &source);

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.style_path);
    println!("dialect: {}", summary.dialect);
    println!("tokens: {}", summary.token_count);
    println!("parse errors: {}", summary.parser_error_count);
    println!("class selectors: {}", summary.class_selector_count);
    println!("custom properties: {}", summary.custom_property_count);
    println!("keyframes: {}", summary.keyframe_count);
    Ok(())
}

struct BuildFileOptions {
    path: PathBuf,
    output: Option<PathBuf>,
    pass_ids: Vec<String>,
    target_query: Option<String>,
    context_json: Option<PathBuf>,
    engine_input_json: Option<PathBuf>,
    closed_style_world: bool,
    tree_shake: bool,
    bundle: bool,
    split_out_dir: Option<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    source_map: bool,
    target_options: OmenaQueryTargetTransformOptionsV0,
    json: bool,
}

fn build_file(options: BuildFileOptions) -> Result<(), String> {
    let BuildFileOptions {
        path,
        output,
        pass_ids,
        target_query,
        context_json,
        engine_input_json,
        closed_style_world,
        tree_shake,
        bundle,
        split_out_dir,
        source_paths,
        package_manifest_paths,
        source_map,
        target_options,
        json,
    } = options;

    if target_query.is_some() && !pass_ids.is_empty() {
        return Err("cannot combine --target-query with explicit --pass values".to_string());
    }
    if target_query.is_some() && tree_shake {
        return Err(
            "cannot combine --target-query with --tree-shake; use --tree-shake without --target-query"
                .to_string(),
        );
    }
    if target_query.is_some() && bundle {
        return Err(
            "cannot combine --target-query with --bundle; use --bundle without --target-query"
                .to_string(),
        );
    }
    if split_out_dir.is_some() && !bundle {
        return Err("--split-out-dir requires --bundle".to_string());
    }
    if source_map && !json {
        return Err("--source-map requires --json".to_string());
    }

    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let mut pass_ids = pass_ids;
    let mut context = read_context_json(context_json.as_deref())?;
    if closed_style_world || tree_shake {
        context.closed_style_world = true;
    }
    if tree_shake {
        append_tree_shake_build_passes(&mut pass_ids);
    }
    if bundle {
        append_bundle_build_passes(&mut pass_ids, &style_path, &source);
    }
    let used_engine_input = engine_input_json.is_some();
    if let Some(engine_input_path) = engine_input_json.as_deref() {
        let engine_input = read_engine_input_json(engine_input_path)?;
        let engine_context = summarize_omena_query_transform_context_from_engine_input(
            &engine_input,
            &style_path,
            context.closed_style_world,
        )
        .context;
        context = merge_cli_transform_context(context, &engine_context);
    }
    let original_workspace_sources = read_workspace_sources(&path, &source, &source_paths)?;
    let (workspace_sources, bundle_asset_url_rewrite_count) = if bundle {
        rewrite_bundle_asset_urls_for_build_sources(&original_workspace_sources)
    } else {
        (original_workspace_sources.clone(), 0)
    };
    let mut split_transform_pass_ids = Vec::new();
    if tree_shake {
        append_tree_shake_build_passes(&mut split_transform_pass_ids);
    }
    let source_for_build = workspace_sources
        .iter()
        .find(|style_source| style_source.style_path == style_path)
        .map(|style_source| style_source.style_source.as_str())
        .unwrap_or(source.as_str());
    let package_manifests = read_package_manifests(&package_manifest_paths)?;
    let resolution_inputs = resolution_inputs_for_build_path(&path, package_manifests.as_slice());
    let mut summary = if let Some(target_query) = target_query {
        if workspace_sources.len() > 1 {
            execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
                &style_path,
                &workspace_sources,
                &target_query,
                &context,
                target_options,
                &package_manifests,
            )?
        } else {
            execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
                &style_path,
                source_for_build,
                &target_query,
                &context,
                target_options,
            )
        }
    } else if workspace_sources.len() > 1 {
        execute_omena_query_consumer_build_style_sources_with_context(
            &style_path,
            &workspace_sources,
            &pass_ids,
            &context,
            &package_manifests,
        )?
    } else {
        execute_omena_query_consumer_build_style_source_with_context(
            &style_path,
            source_for_build,
            &pass_ids,
            &context,
        )
    };
    if used_engine_input {
        push_ready_surface(
            &mut summary.ready_surfaces,
            "semanticReachabilityTransformContext",
        );
        push_ready_surface(
            &mut summary.ready_surfaces,
            "expressionDomainSelectorProjection",
        );
    }
    if tree_shake {
        push_ready_surface(&mut summary.ready_surfaces, "treeShakeBuildMode");
    }
    if bundle {
        attach_omena_query_consumer_build_bundle_summary(&mut summary, &source);
        push_ready_surface(&mut summary.ready_surfaces, "bundleBuildMode");
        if bundle_asset_url_rewrite_count > 0 {
            push_ready_surface(&mut summary.ready_surfaces, "bundleAssetUrlRewrite");
        }
    }
    if source_map {
        attach_omena_query_consumer_build_source_map_v3_with_sources(
            &mut summary,
            &original_workspace_sources,
            &package_manifests,
        );
    }
    if let Some(split_out_dir) = split_out_dir.as_ref() {
        emit_bundle_code_split_outputs(BundleCodeSplitOutputOptions {
            out_dir: split_out_dir,
            entry_style_path: &style_path,
            sources: &workspace_sources,
            source_map_sources: &original_workspace_sources,
            resolution_inputs: &resolution_inputs,
            split_transform_pass_ids: &split_transform_pass_ids,
            context: &context,
            source_map,
        })?;
        push_ready_surface(&mut summary.ready_surfaces, "bundleCodeSplitEmission");
        push_ready_surface(
            &mut summary.ready_surfaces,
            "bundleCodeSplitManifestEmission",
        );
        if tree_shake {
            push_ready_surface(
                &mut summary.ready_surfaces,
                "bundleCodeSplitTreeShakeEmission",
            );
        }
        if source_map {
            push_ready_surface(
                &mut summary.ready_surfaces,
                "bundleCodeSplitSourceMapEmission",
            );
        }
    }

    if !summary.unknown_pass_ids.is_empty() {
        return Err(format!(
            "unknown transform pass id: {}",
            summary.unknown_pass_ids.join(", ")
        ));
    }

    if let Some(output_path) = output {
        fs::write(&output_path, &summary.execution.output_css).map_err(|error| {
            format!(
                "failed to write transformed CSS to {}: {error}",
                path_string(&output_path)
            )
        })?;
    } else if !json {
        print!("{}", summary.execution.output_css);
    }

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    eprintln!(
        "executed passes: {}",
        summary.execution.executed_pass_ids.join(", ")
    );
    eprintln!(
        "planned-only passes: {}",
        summary.execution.planned_only_pass_ids.join(", ")
    );
    eprintln!("mutations: {}", summary.execution.mutation_count);
    Ok(())
}

fn read_workspace_sources(
    target_path: &Path,
    target_source: &str,
    additional_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStyleSourceInputV0>, String> {
    let target_path_string = path_string(target_path);
    let mut sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: target_path_string.clone(),
        style_source: target_source.to_string(),
    }];

    for source_path in additional_paths {
        let source_path_string = path_string(source_path);
        if source_path_string == target_path_string {
            continue;
        }
        sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: source_path_string,
            style_source: read_source(source_path)?,
        });
    }

    Ok(sources)
}

fn rewrite_bundle_asset_urls_for_build_sources(
    sources: &[OmenaQueryStyleSourceInputV0],
) -> (Vec<OmenaQueryStyleSourceInputV0>, usize) {
    let mut rewrite_count = 0usize;
    let rewritten_sources = sources
        .iter()
        .map(|source| {
            let rewrite = rewrite_omena_transform_bundle_asset_urls_in_source(
                source.style_path.as_str(),
                source.style_source.as_str(),
            );
            rewrite_count = rewrite_count.saturating_add(rewrite.rewrite_count);
            OmenaQueryStyleSourceInputV0 {
                style_path: source.style_path.clone(),
                style_source: rewrite.output_css,
            }
        })
        .collect::<Vec<_>>();
    (rewritten_sources, rewrite_count)
}

fn resolution_inputs_for_build_path(
    path: &Path,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryStyleResolutionInputsV0 {
    let workspace_folder_uri = style_resolution_workspace_uri_for_path(path);
    load_omena_query_workspace_style_resolution_inputs(
        workspace_folder_uri.as_deref(),
        package_manifests,
    )
}

struct BundleCodeSplitOutputOptions<'a> {
    out_dir: &'a Path,
    entry_style_path: &'a str,
    sources: &'a [OmenaQueryStyleSourceInputV0],
    source_map_sources: &'a [OmenaQueryStyleSourceInputV0],
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    split_transform_pass_ids: &'a [String],
    context: &'a OmenaQueryTransformExecutionContextV0,
    source_map: bool,
}

const BUNDLE_CODE_SPLIT_MANIFEST_FILE_NAME: &str = "omena.bundle-split.manifest.json";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleCodeSplitManifestV0 {
    schema_version: u8,
    product: &'static str,
    entry_style_path: String,
    entry_file: String,
    output_count: usize,
    outputs: Vec<BundleCodeSplitManifestOutputV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleCodeSplitManifestOutputV0 {
    source_path: String,
    file_name: String,
    is_entry: bool,
    source_map_file: Option<String>,
    imports: Vec<BundleCodeSplitManifestImportV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleCodeSplitManifestImportV0 {
    import_source: String,
    resolved_style_path: String,
    file_name: String,
}

fn emit_bundle_code_split_outputs(options: BundleCodeSplitOutputOptions<'_>) -> Result<(), String> {
    let BundleCodeSplitOutputOptions {
        out_dir,
        entry_style_path,
        sources,
        source_map_sources,
        resolution_inputs,
        split_transform_pass_ids,
        context,
        source_map,
    } = options;

    fs::create_dir_all(out_dir).map_err(|error| {
        format!(
            "failed to create bundle split output directory {}: {error}",
            path_string(out_dir)
        )
    })?;
    let source_by_path = sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let source_map_source_by_path = source_map_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let file_name_by_path = sources
        .iter()
        .map(|source| {
            (
                source.style_path.as_str(),
                bundle_split_file_name(source.style_path.as_str()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let source_path_lookup = bundle_split_source_path_lookup(sources);
    let reachable_paths = collect_bundle_code_split_reachable_style_paths(
        entry_style_path,
        sources,
        resolution_inputs,
        &source_path_lookup,
    );

    let mut manifest_outputs = Vec::new();
    for style_path in reachable_paths {
        let Some(source) = source_by_path.get(style_path.as_str()) else {
            continue;
        };
        let Some(file_name) = file_name_by_path.get(style_path.as_str()) else {
            continue;
        };
        let manifest_imports = bundle_code_split_manifest_imports_for_source(
            style_path.as_str(),
            source,
            &file_name_by_path,
            resolution_inputs,
            &source_path_lookup,
        );
        let rewritten_source = rewrite_bundle_code_split_imports_for_source(
            style_path.as_str(),
            source,
            &file_name_by_path,
            resolution_inputs,
            &source_path_lookup,
        );
        let mut output_css = rewritten_source;
        let source_map_file = source_map.then(|| format!("{file_name}.map"));
        if !split_transform_pass_ids.is_empty() {
            let split_summary = execute_omena_query_consumer_build_style_source_with_context(
                style_path.as_str(),
                output_css.as_str(),
                split_transform_pass_ids,
                context,
            );
            if !split_summary.unknown_pass_ids.is_empty() {
                return Err(format!(
                    "unknown transform pass id for bundle split output {}: {}",
                    style_path,
                    split_summary.unknown_pass_ids.join(", ")
                ));
            }
            output_css = split_summary.execution.output_css;
        }
        if let Some(map_file_name) = source_map_file.as_deref() {
            let source_map_source = source_map_source_by_path
                .get(style_path.as_str())
                .copied()
                .unwrap_or(source);
            let source_map_v3 = summarize_omena_query_bundle_code_split_source_map_v3(
                file_name,
                output_css.as_str(),
                style_path.as_str(),
                source_map_source,
            );
            let map_output_path = out_dir.join(map_file_name);
            let source_map_json = serde_json::to_string_pretty(&source_map_v3)
                .map_err(|error| format!("failed to serialize split source map: {error}"))?;
            fs::write(&map_output_path, source_map_json).map_err(|error| {
                format!(
                    "failed to write bundle split source map {}: {error}",
                    path_string(&map_output_path)
                )
            })?;
            output_css.push_str("\n/*# sourceMappingURL=");
            output_css.push_str(map_file_name);
            output_css.push_str(" */\n");
        }
        let output_path = out_dir.join(file_name);
        fs::write(&output_path, output_css).map_err(|error| {
            format!(
                "failed to write bundle split output {}: {error}",
                path_string(&output_path)
            )
        })?;
        manifest_outputs.push(BundleCodeSplitManifestOutputV0 {
            source_path: style_path.clone(),
            file_name: file_name.clone(),
            is_entry: style_path == entry_style_path,
            source_map_file,
            imports: manifest_imports,
        });
    }
    let entry_file = file_name_by_path
        .get(entry_style_path)
        .cloned()
        .unwrap_or_else(|| bundle_split_file_name(entry_style_path));
    let manifest = BundleCodeSplitManifestV0 {
        schema_version: 0,
        product: "omena-cli.bundle-code-split-manifest",
        entry_style_path: entry_style_path.to_string(),
        entry_file,
        output_count: manifest_outputs.len(),
        outputs: manifest_outputs,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("failed to serialize bundle split manifest: {error}"))?;
    let manifest_path = out_dir.join(BUNDLE_CODE_SPLIT_MANIFEST_FILE_NAME);
    fs::write(&manifest_path, manifest_json).map_err(|error| {
        format!(
            "failed to write bundle split manifest {}: {error}",
            path_string(&manifest_path)
        )
    })?;
    Ok(())
}

fn collect_bundle_code_split_reachable_style_paths(
    entry_style_path: &str,
    sources: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    source_path_lookup: &BTreeMap<String, String>,
) -> Vec<String> {
    let source_by_path = sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut reachable = Vec::new();
    let mut visited = BTreeSet::new();
    let mut stack = vec![entry_style_path.to_string()];

    while let Some(style_path) = stack.pop() {
        if !visited.insert(style_path.clone()) {
            continue;
        }
        let Some(source) = source_by_path.get(style_path.as_str()) else {
            continue;
        };
        reachable.push(style_path.clone());
        let bundle = summarize_omena_transform_bundle_from_source(
            style_path.as_str(),
            source,
            infer_cli_style_dialect(style_path.as_str()),
        );
        for edge in bundle.bundle_edges {
            if !matches!(
                edge.kind,
                TransformBundleEdgeKind::CssImport | TransformBundleEdgeKind::LessImport
            ) {
                continue;
            }
            let Some(import_source) = edge.import_source.as_deref() else {
                continue;
            };
            let Some(target_path) = resolve_bundle_code_split_import_path(
                style_path.as_str(),
                import_source,
                resolution_inputs,
            ) else {
                continue;
            };
            if let Some(source_path) = source_path_lookup.get(target_path.as_str())
                && source_by_path.contains_key(source_path.as_str())
            {
                stack.push(source_path.clone());
            }
        }
    }

    reachable
}

fn rewrite_bundle_code_split_imports_for_source(
    style_path: &str,
    source: &str,
    file_name_by_path: &BTreeMap<&str, String>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    source_path_lookup: &BTreeMap<String, String>,
) -> String {
    let bundle = summarize_omena_transform_bundle_from_source(
        style_path,
        source,
        infer_cli_style_dialect(style_path),
    );
    let mut output = source.to_string();
    for edge in bundle.bundle_edges.iter().rev() {
        if !matches!(
            edge.kind,
            TransformBundleEdgeKind::CssImport | TransformBundleEdgeKind::LessImport
        ) {
            continue;
        }
        let Some(import_source) = edge.import_source.as_deref() else {
            continue;
        };
        let Some(target_path) =
            resolve_bundle_code_split_import_path(style_path, import_source, resolution_inputs)
        else {
            continue;
        };
        let Some(source_path) = source_path_lookup.get(target_path.as_str()) else {
            continue;
        };
        let Some(target_file_name) = file_name_by_path.get(source_path.as_str()) else {
            continue;
        };
        let range_start = edge.range_start as usize;
        let range_end = edge.range_end as usize;
        if range_start > range_end || range_end > output.len() {
            continue;
        }
        let rule_text = &output[range_start..range_end];
        let Some(relative_source_start) = rule_text.find(import_source) else {
            continue;
        };
        let source_start = range_start + relative_source_start;
        let source_end = source_start + import_source.len();
        output.replace_range(source_start..source_end, target_file_name);
    }
    output
}

fn bundle_code_split_manifest_imports_for_source(
    style_path: &str,
    source: &str,
    file_name_by_path: &BTreeMap<&str, String>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    source_path_lookup: &BTreeMap<String, String>,
) -> Vec<BundleCodeSplitManifestImportV0> {
    let bundle = summarize_omena_transform_bundle_from_source(
        style_path,
        source,
        infer_cli_style_dialect(style_path),
    );
    let mut imports = Vec::new();
    for edge in bundle.bundle_edges {
        if !matches!(
            edge.kind,
            TransformBundleEdgeKind::CssImport | TransformBundleEdgeKind::LessImport
        ) {
            continue;
        }
        let Some(import_source) = edge.import_source else {
            continue;
        };
        let Some(target_path) =
            resolve_bundle_code_split_import_path(style_path, &import_source, resolution_inputs)
        else {
            continue;
        };
        let Some(source_path) = source_path_lookup.get(target_path.as_str()) else {
            continue;
        };
        let Some(file_name) = file_name_by_path.get(source_path.as_str()) else {
            continue;
        };
        imports.push(BundleCodeSplitManifestImportV0 {
            import_source,
            resolved_style_path: source_path.clone(),
            file_name: file_name.clone(),
        });
    }
    imports
}

fn bundle_split_source_path_lookup(
    sources: &[OmenaQueryStyleSourceInputV0],
) -> BTreeMap<String, String> {
    let mut lookup = BTreeMap::new();
    for source in sources {
        lookup.insert(source.style_path.clone(), source.style_path.clone());
        if let Ok(canonical_path) = fs::canonicalize(&source.style_path) {
            lookup.insert(path_string(&canonical_path), source.style_path.clone());
        }
    }
    lookup
}

fn resolve_bundle_code_split_import_path(
    style_path: &str,
    import_source: &str,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<String> {
    let base_uri = cli_path_to_file_uri(Path::new(style_path));
    let workspace_folder_uri = Path::new(style_path).parent().map(cli_path_to_file_uri);
    let resolved_uri = resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
        base_uri.as_str(),
        workspace_folder_uri.as_deref(),
        import_source,
        resolution_inputs,
    )?;
    cli_file_uri_to_path(resolved_uri.as_str()).map(|path| path_string(&path))
}

fn bundle_split_file_name(style_path: &str) -> String {
    let path = Path::new(style_path);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("css");
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("chunk");
    let mut sanitized = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            sanitized.push(ch);
        } else {
            sanitized.push('-');
        }
    }
    if sanitized.is_empty() {
        sanitized.push_str("chunk");
    }
    let hash = bundle_split_path_hash(style_path);
    format!("{sanitized}-{hash:016x}.{extension}")
}

fn bundle_split_path_hash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn cli_path_to_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

fn cli_file_uri_to_path(uri: &str) -> Option<PathBuf> {
    uri.strip_prefix("file://").map(PathBuf::from)
}

fn read_source_documents(
    source_document_paths: &[PathBuf],
) -> Result<Vec<OmenaQuerySourceDocumentInputV0>, String> {
    source_document_paths
        .iter()
        .map(|path| {
            Ok(OmenaQuerySourceDocumentInputV0 {
                source_path: path_string(path),
                source_source: read_source(path)?,
            })
        })
        .collect()
}

fn read_style_sources(
    source_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStyleSourceInputV0>, String> {
    source_paths
        .iter()
        .map(|path| {
            Ok(OmenaQueryStyleSourceInputV0 {
                style_path: path_string(path),
                style_source: read_source(path)?,
            })
        })
        .collect()
}

fn read_package_manifests(
    package_manifest_paths: &[PathBuf],
) -> Result<Vec<OmenaQueryStylePackageManifestV0>, String> {
    package_manifest_paths
        .iter()
        .map(|path| {
            Ok(OmenaQueryStylePackageManifestV0 {
                package_json_path: path_string(path),
                package_json_source: read_source(path)?,
            })
        })
        .collect()
}

fn list_passes(json: bool) -> Result<(), String> {
    let passes = list_omena_query_transform_pass_summaries();

    if json {
        print_json(&passes)?;
        return Ok(());
    }

    for pass in passes {
        println!("{}\t{}", pass.id, pass.title);
    }
    Ok(())
}

#[cfg(feature = "mdl")]
fn compress_file(path: PathBuf, budget_bits: Option<f64>, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let rule_count = source.matches('{').count();
    let observation_count = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let source_hash = format!(
        "len{}-sum{}",
        source.len(),
        source.bytes().map(u64::from).sum::<u64>()
    );
    // Empirical value-symbol histogram: count how often each declaration value
    // recurs across the design system. A token reused everywhere (a design token)
    // peaks the distribution and compresses; many one-off values flatten it. This
    // drives the real entropy term of the MDL residual.
    let value_frequencies = css_declaration_value_histogram(&source);
    let summary = summarize_omena_query_design_system_minimum_description(
        path_string(&path),
        source_hash,
        rule_count,
        observation_count,
        &value_frequencies,
    );
    if let Some(budget_bits) = budget_bits
        && summary.total_bits > budget_bits
    {
        return Err(format!(
            "MDL budget exceeded: total_bits={} budget_bits={budget_bits}",
            summary.total_bits
        ));
    }

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("total bits: {}", summary.total_bits);
    println!("model bits: {}", summary.model_bits);
    println!("residual bits: {}", summary.residual_bits);
    println!("unit: {}", summary.unit);
    Ok(())
}

/// Build an empirical value-symbol frequency histogram from CSS declarations.
///
/// Each `prop: value;` declaration contributes its trimmed value string as a
/// symbol; the returned vector is the per-symbol occurrence count. Recurring
/// design-token values peak the histogram (low entropy / compressible); scattered
/// one-off values flatten it. Deterministic and dependency-light.
#[cfg(feature = "mdl")]
fn css_declaration_value_histogram(source: &str) -> Vec<usize> {
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for line in source.lines() {
        let line = line.trim();
        let Some((_, rhs)) = line.split_once(':') else {
            continue;
        };
        let value = rhs.trim().trim_end_matches([';', '}']).trim();
        if value.is_empty() || value.contains('{') {
            continue;
        }
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts.into_values().collect()
}

fn context_from_engine_input(
    path: PathBuf,
    engine_input_json: PathBuf,
    closed_style_world: bool,
    json: bool,
) -> Result<(), String> {
    let style_path = path_string(&path);
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let summary = summarize_omena_query_transform_context_from_engine_input(
        &engine_input,
        &style_path,
        closed_style_world,
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("target: {}", summary.target_style_path);
    println!("closed style world: {}", summary.closed_style_world);
    println!("projections: {}", summary.projection_count);
    println!(
        "selected projections: {}",
        summary.selected_projection_count
    );
    println!("reachable classes: {}", summary.reachable_class_name_count);
    for class_name in &summary.context.reachable_class_names {
        println!("  {class_name}");
    }
    Ok(())
}

fn expression_flow(engine_input_json: PathBuf, json: bool) -> Result<(), String> {
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let summary = summarize_omena_query_expression_domain_incremental_flow_analysis(
        &engine_input,
        &mut runtime,
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("product: {}", summary.product);
    println!("revision: {}", summary.revision);
    println!("graphs: {}", summary.graph_count);
    println!("dirty graphs: {}", summary.dirty_graph_count);
    println!("reused graphs: {}", summary.reused_graph_count);
    for entry in &summary.analyses {
        println!(
            "{}\tnodes={}\tdirty={}\treused={}",
            entry.graph_id,
            entry.analysis.analysis.nodes.len(),
            entry.analysis.incremental_plan.dirty_node_count,
            entry.analysis.reused_previous_analysis
        );
    }
    Ok(())
}

fn selector_projection(engine_input_json: PathBuf, json: bool) -> Result<(), String> {
    let engine_input = read_engine_input_json(&engine_input_json)?;
    let summary = summarize_omena_query_expression_domain_selector_projection(&engine_input);

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("product: {}", summary.product);
    println!("projections: {}", summary.projection_count);
    for projection in &summary.projections {
        println!(
            "{}\t{}\t{:?}\t{}",
            projection.graph_id,
            projection.node_id,
            projection.certainty,
            projection.selector_names.join(",")
        );
    }
    Ok(())
}

fn cascade_at_position(
    path: PathBuf,
    line: usize,
    character: usize,
    engine_input_json: Option<PathBuf>,
    categorical_evidence: bool,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let engine_input = if let Some(engine_input_path) = engine_input_json.as_deref() {
        read_engine_input_json(engine_input_path)?
    } else {
        empty_engine_input()
    };
    let position = ParserPositionV0 { line, character };
    let summary = if categorical_evidence {
        read_omena_query_cascade_at_position_with_categorical_evidence(
            &style_path,
            &source,
            &engine_input,
            position,
            true,
        )
    } else {
        read_omena_query_cascade_at_position(&style_path, &source, &engine_input, position)
    };
    let Some(summary) = summary else {
        return Err(format!(
            "failed to read cascade information for {style_path}:{line}:{character}",
        ));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.style_path);
    println!("status: {}", summary.status);
    println!(
        "reference: {}",
        summary.reference_name.as_deref().unwrap_or("-")
    );
    println!(
        "computed status: {}",
        summary
            .referenced_declaration_computed_value_status
            .unwrap_or("-")
    );
    println!(
        "computed value: {}",
        summary
            .referenced_declaration_computed_value
            .as_deref()
            .unwrap_or("-")
    );
    println!(
        "lfp status: {}",
        summary
            .reference_custom_property_fixed_point_status
            .unwrap_or("-")
    );
    println!(
        "lfp value: {}",
        summary
            .reference_custom_property_fixed_point_value
            .as_deref()
            .unwrap_or("-")
    );
    println!(
        "lfp iterations: {}",
        summary.custom_property_fixed_point_iteration_count
    );
    println!(
        "lfp guaranteed-invalid count: {}",
        summary.custom_property_fixed_point_guaranteed_invalid_count
    );
    if let Some(evidence) = summary.categorical_evidence.as_ref() {
        println!("categorical evidence: attached");
        println!("categorical endpoints: {}", evidence.endpoint_count);
        println!(
            "categorical functor accepted: {}",
            evidence
                .functor_applications
                .first()
                .map(|functor| functor.accepted)
                .unwrap_or(false)
        );
    }
    Ok(())
}

fn context_index(
    path: PathBuf,
    engine_input_json: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let engine_input = if let Some(engine_input_path) = engine_input_json.as_deref() {
        read_engine_input_json(engine_input_path)?
    } else {
        empty_engine_input()
    };
    let Some(summary) = read_omena_query_style_context_index(&style_path, &source, &engine_input)
    else {
        return Err(format!("failed to read context index for {style_path}"));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.style_path);
    println!("source: {}", summary.context_index_source);
    println!(
        "layer blocks: {}",
        summary.context_index.layer_index.block_layers.len()
    );
    println!(
        "layer statements: {}",
        summary.context_index.layer_index.statement_layers.len()
    );
    println!(
        "containers: {}",
        summary.context_index.container_index.containers.len()
    );
    println!("scopes: {}", summary.context_index.scope_index.scopes.len());
    println!(
        "selector context memberships: {}",
        summary.context_index.selector_context_count
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn style_diagnostics(
    path: PathBuf,
    source_paths: Vec<PathBuf>,
    source_document_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    sif_paths: Vec<PathBuf>,
    lockfile: Option<PathBuf>,
    external: Option<String>,
    deep_analysis: bool,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let package_manifests = read_package_manifests(&package_manifest_paths)?;
    let resolved_lockfile = lockfile.or_else(|| discover_omena_lockfile_for_path(&path));
    let external_mode = resolve_external_module_mode_for_style_diagnostics(
        external.as_deref(),
        &resolved_lockfile,
    )?;
    let uses_external_sif_path = external_mode == OmenaQueryExternalModuleModeV0::Sif;
    let summary = if source_paths.is_empty()
        && source_document_paths.is_empty()
        && package_manifests.is_empty()
        && sif_paths.is_empty()
        && !uses_external_sif_path
    {
        let Some(candidates) = summarize_omena_query_style_hover_candidates(&style_path, &source)
        else {
            return Err(format!(
                "failed to read style candidates for {}",
                path_string(&path)
            ));
        };
        summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis(
            &style_path,
            &source,
            candidates.candidates.as_slice(),
            deep_analysis,
        )
    } else {
        let workspace_sources = read_workspace_sources(&path, &source, &source_paths)?;
        let source_documents = read_source_documents(&source_document_paths)?;
        let mut external_sifs = read_external_sifs(&sif_paths)?;
        let mut lockfile_diagnostics = Vec::new();
        if uses_external_sif_path && let Some(lockfile) = resolved_lockfile.as_ref() {
            match read_lock_external_sifs(lockfile) {
                Ok(lock_sifs) => external_sifs.extend(lock_sifs),
                Err(error) => lockfile_diagnostics
                    .push(lockfile_invalid_style_diagnostic(lockfile, error.as_str())),
            }
        }
        // #33: an `@use "file:///…"` edge now routes through the external-SIF branch
        // (resolver `is_external_style_module_source`). Generate the bridge SIF for each such
        // on-disk external edge in-process so an external `missingSassSymbol` is suppressed
        // without a manual `--sif`. Edges already covered by an explicit `--sif` (matching
        // canonical URL) keep the user-provided artifact; unreadable edges (a genuinely-missing
        // module, or a `http(s)://`/`sass:` scheme the bridge cannot read) are skipped so they
        // still surface their boundary state.
        let in_process_external_sifs = resolve_in_process_external_sifs(
            workspace_sources.as_slice(),
            external_sifs.as_slice(),
        );
        external_sifs.extend(in_process_external_sifs);
        let workspace_folder_uri = style_resolution_workspace_uri_for_path(&path);
        let resolution_inputs = load_omena_query_workspace_style_resolution_inputs(
            workspace_folder_uri.as_deref(),
            package_manifests.as_slice(),
        );
        let mut summary =
            summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
                &style_path,
                workspace_sources.as_slice(),
                source_documents.as_slice(),
                package_manifests.as_slice(),
                None,
                external_mode,
                external_sifs.as_slice(),
                &resolution_inputs,
            )
            .ok_or_else(|| {
                format!("failed to read workspace style diagnostics for {style_path}")
            })?;
        // Drive the crate-owned streaming-IFDS cross-file reachability report from
        // the resolved cross-file hypergraph, not a synthetic harness.
        summary
            .diagnostics
            .extend(summarize_cross_file_streaming_reachability_diagnostics(
                &style_path,
                workspace_sources.as_slice(),
                source_documents.as_slice(),
                package_manifests.as_slice(),
            ));
        summary
            .diagnostics
            .extend(summarize_sass_module_resolution_identity_diagnostics(
                &style_path,
                workspace_sources.as_slice(),
                package_manifests.as_slice(),
                &resolution_inputs,
            ));
        push_cli_ready_surface(
            &mut summary.ready_surfaces,
            "sassModuleResolutionIdentityDiagnostics",
        );
        summary.diagnostics.extend(lockfile_diagnostics);
        summary.diagnostic_count = summary.diagnostics.len();
        summary
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("diagnostics: {}", summary.diagnostic_count);
    for diagnostic in &summary.diagnostics {
        println!("{}\t{}", diagnostic.code, diagnostic.message);
    }
    Ok(())
}

/// Surface a real cross-file dataflow reachability fact through the product diagnostics.
///
/// The streaming-IFDS crate projects the resolved workspace cross-file summary
/// to the unified hypergraph — the SAME real `composes`/`@use`/`@forward`/
/// `@import`/value/icss/foreign-reference edges the analyzer already resolves.
/// It then owns the exact propagation report: every node owned by the target
/// file is seeded and foreign module paths are reached by facts over those
/// edges. A self-contained file has no foreign reachable path and no diagnostic
/// is emitted. No synthetic hyperedges are fed in.
fn summarize_cross_file_streaming_reachability_diagnostics(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let report = summarize_streaming_ifds_workspace_cross_file_reachability_v0(
        target_style_path,
        workspace_sources,
        source_documents,
        package_manifests,
    );
    if report.reachable_foreign_paths.is_empty() {
        return Vec::new();
    }

    let reachable_modules = report.reachable_foreign_paths.join(", ");
    vec![OmenaQueryStyleDiagnosticV0 {
        code: "crossFileStreamingReachability",
        severity: "hint",
        provenance: vec![
            "omena-streaming-ifds.cross-file-reachability-report",
            "omena-streaming-ifds.analysis-report",
            "omena-query.unified-cross-file-hypergraph",
            "omena-query.cross-file-summary",
        ],
        range: ParserRangeV0::default(),
        message: format!(
            "cross-file dataflow reaches {} module(s) via resolved edges: {reachable_modules}",
            report.reachable_foreign_path_count
        ),
        tags: Vec::new(),
        create_custom_property: None,
        cascade_narrowing: None,
        cascade_confidence: None,
        polynomial_provenance: None,
        cross_file_scc: None,
    }]
}

fn summarize_sass_module_resolution_identity_diagnostics(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &omena_query::OmenaQueryStyleResolutionInputsV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = workspace_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        workspace_sources,
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    let range = whole_file_range(target.style_source.as_str());
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
    {
        let visible_symlink_links = edge
            .symlink_chain_links
            .iter()
            .filter(|link| !is_platform_alias_symlink_link(link))
            .collect::<Vec<_>>();
        if !visible_symlink_links.is_empty()
            && emitted.insert((
                "sassModuleSymlinkResolution",
                edge.source.clone(),
                edge.resolved_style_path.clone(),
            ))
        {
            let target_path = edge
                .resolved_style_path
                .as_deref()
                .unwrap_or(edge.source.as_str());
            let link_summary = visible_symlink_links
                .first()
                .map(|link| format!("; first link {} -> {}", link.link_path, link.target_path))
                .unwrap_or_default();
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleSymlinkResolution",
                severity: "hint",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-resolver.symlink-chain-metadata",
                    "omena-cli.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' resolves to '{}' through {} symlink link(s){}.",
                    edge.source,
                    target_path,
                    visible_symlink_links.len(),
                    link_summary
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }

        if edge.configuration_variable_count > 0
            && let Some(identity_key) = edge.module_instance_identity_key.as_ref()
            && emitted.insert((
                "sassModuleInstanceIdentity",
                edge.source.clone(),
                Some(identity_key.clone()),
            ))
        {
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleInstanceIdentity",
                severity: "hint",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-query.module-instance-identity",
                    "omena-cli.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' uses {} configured variable(s); module instance identity is {}.",
                    edge.source, edge.configuration_variable_count, identity_key
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }

        if !edge.invalid_configuration_variable_names.is_empty()
            && emitted.insert((
                "sassModuleInvalidConfiguration",
                edge.source.clone(),
                edge.resolved_style_path.clone(),
            ))
        {
            let target_path = edge
                .resolved_style_path
                .as_deref()
                .unwrap_or(edge.source.as_str());
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleInvalidConfiguration",
                severity: "error",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-query.module-instance-identity",
                    "omena-cli.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' configures {} on '{}', but Sass @use/@forward with(...) can configure only public !default variables.",
                    edge.source,
                    format_sass_configuration_variable_names(
                        edge.invalid_configuration_variable_names.as_slice()
                    ),
                    target_path
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }
    }

    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.configuration_variable_count > 0)
    {
        let Some(identity_key) = edge.module_instance_identity_key.as_ref() else {
            continue;
        };
        if !emitted.insert((
            "sassModuleInstanceIdentity",
            edge.target_style_path.clone(),
            Some(identity_key.clone()),
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleInstanceIdentity",
            severity: "hint",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-cli.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module graph reaches configured module instance '{}' in {} hop(s); module instance identity is {}.",
                edge.target_style_path, edge.depth, identity_key
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| !edge.invalid_configuration_variable_names.is_empty())
    {
        if !emitted.insert((
            "sassModuleInvalidConfiguration",
            edge.target_style_path.clone(),
            Some(edge.configuration_signature.clone()),
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleInvalidConfiguration",
            severity: "error",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-cli.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module graph reaches invalid configuration for '{}': {} are not public !default variables.",
                edge.target_style_path,
                format_sass_configuration_variable_names(
                    edge.invalid_configuration_variable_names.as_slice()
                )
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    diagnostics.extend(summarize_sass_module_configuration_conflict_diagnostics(
        target_style_path,
        workspace_sources,
        &resolution,
        range,
    ));

    diagnostics
}

fn summarize_sass_module_configuration_conflict_diagnostics(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &omena_query::OmenaQuerySassModuleCrossFileResolutionV0,
    range: ParserRangeV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let mut signatures_by_target = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.configuration_variable_count > 0)
    {
        signatures_by_target
            .entry(edge.target_style_path.clone())
            .or_default()
            .insert(edge.configuration_signature.clone());
    }
    for (target, signatures) in collect_sass_module_load_order_configuration_conflicts(
        target_style_path,
        workspace_sources,
        resolution,
    ) {
        signatures_by_target
            .entry(target)
            .or_default()
            .extend(signatures);
    }

    signatures_by_target
        .into_iter()
        .filter(|(_, signatures)| signatures.len() > 1)
        .map(|(target, signatures)| OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleConfigurationConflict",
            severity: "error",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-cli.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module '{target}' is reached with {} different configurations ({}); Sass modules can be configured only once per compilation.",
                signatures.len(),
                signatures.into_iter().collect::<Vec<_>>().join(", ")
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        })
        .collect()
}

fn collect_sass_module_load_order_configuration_conflicts(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &omena_query::OmenaQuerySassModuleCrossFileResolutionV0,
) -> BTreeMap<String, BTreeSet<String>> {
    let source_by_path = workspace_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut edges_by_from =
        BTreeMap::<&str, Vec<&omena_query::OmenaQuerySassModuleEdgeResolutionV0>>::new();
    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "resolved" && edge.resolved_style_path.is_some())
    {
        edges_by_from
            .entry(edge.from_style_path.as_str())
            .or_default()
            .push(edge);
    }
    for (style_path, edges) in &mut edges_by_from {
        let style_source = source_by_path.get(style_path).copied().unwrap_or_default();
        edges.sort_by_key(|edge| {
            (
                sass_module_edge_source_offset(style_source, edge.edge_kind, edge.source.as_str()),
                edge.edge_kind,
                edge.source.clone(),
            )
        });
    }

    let mut loaded_signatures_by_target = BTreeMap::new();
    let mut active_stack = BTreeSet::new();
    let mut conflicts_by_target = BTreeMap::new();
    collect_sass_module_load_order_configuration_conflicts_for_style(
        target_style_path,
        &edges_by_from,
        &mut loaded_signatures_by_target,
        &mut active_stack,
        &mut conflicts_by_target,
    );
    conflicts_by_target
}

fn collect_sass_module_load_order_configuration_conflicts_for_style(
    style_path: &str,
    edges_by_from: &BTreeMap<&str, Vec<&omena_query::OmenaQuerySassModuleEdgeResolutionV0>>,
    loaded_signatures_by_target: &mut BTreeMap<String, String>,
    active_stack: &mut BTreeSet<String>,
    conflicts_by_target: &mut BTreeMap<String, BTreeSet<String>>,
) {
    if !active_stack.insert(style_path.to_string()) {
        return;
    }
    if let Some(edges) = edges_by_from.get(style_path) {
        for edge in edges {
            let Some(target_style_path) = edge.resolved_style_path.as_ref() else {
                continue;
            };
            let requested_signature = edge.configuration_signature.clone();
            let should_visit_target =
                match loaded_signatures_by_target.get(target_style_path.as_str()) {
                    Some(existing_signature)
                        if is_unconfigured_sass_module_signature(requested_signature.as_str())
                            || existing_signature == &requested_signature =>
                    {
                        false
                    }
                    Some(existing_signature) => {
                        let signatures = conflicts_by_target
                            .entry(target_style_path.clone())
                            .or_default();
                        signatures.insert(existing_signature.clone());
                        signatures.insert(requested_signature);
                        false
                    }
                    None => {
                        loaded_signatures_by_target
                            .insert(target_style_path.clone(), requested_signature);
                        true
                    }
                };
            if should_visit_target {
                collect_sass_module_load_order_configuration_conflicts_for_style(
                    target_style_path.as_str(),
                    edges_by_from,
                    loaded_signatures_by_target,
                    active_stack,
                    conflicts_by_target,
                );
            }
        }
    }
    active_stack.remove(style_path);
}

fn is_unconfigured_sass_module_signature(signature: &str) -> bool {
    signature == "with:none"
}

fn format_sass_configuration_variable_names(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("${name}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn sass_module_edge_source_offset(style_source: &str, edge_kind: &str, source: &str) -> usize {
    let keyword = match edge_kind {
        "sassUse" => "@use",
        "sassForward" => "@forward",
        _ => return usize::MAX,
    };
    let mut search_start = 0usize;
    while let Some(relative_keyword_start) = style_source[search_start..].find(keyword) {
        let keyword_start = search_start + relative_keyword_start;
        let after_keyword = &style_source[keyword_start + keyword.len()..];
        let Some(relative_source_start) = after_keyword.find(source) else {
            search_start = keyword_start + keyword.len();
            continue;
        };
        let between_keyword_and_source = &after_keyword[..relative_source_start];
        if !between_keyword_and_source.contains(';') && !between_keyword_and_source.contains('{') {
            return keyword_start;
        }
        search_start = keyword_start + keyword.len();
    }
    usize::MAX
}

fn is_platform_alias_symlink_link(link: &omena_query::OmenaQuerySymlinkChainLinkV0) -> bool {
    matches!(
        (link.link_path.as_str(), link.target_path.as_str()),
        ("/var", "/private/var") | ("/tmp", "/private/tmp") | ("/etc", "/private/etc")
    )
}

fn whole_file_range(source: &str) -> ParserRangeV0 {
    let mut line = 0usize;
    let mut character = 0usize;
    for ch in source.chars() {
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }
    ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 { line, character },
    }
}

fn push_cli_ready_surface(surfaces: &mut Vec<&'static str>, surface: &'static str) {
    if !surfaces.contains(&surface) {
        surfaces.push(surface);
    }
}

fn lockfile_invalid_style_diagnostic(
    lockfile: &Path,
    message: &str,
) -> OmenaQueryStyleDiagnosticV0 {
    OmenaQueryStyleDiagnosticV0 {
        code: "lockfileInvalid",
        severity: "error",
        provenance: vec![
            "omena-cli.lockfile-loader",
            "omena-query.external-sif-boundary-diagnostics",
        ],
        range: ParserRangeV0::default(),
        message: format!(
            "Failed to load {} for external SIF diagnostics: {message}",
            path_string(lockfile)
        ),
        tags: Vec::new(),
        create_custom_property: None,
        cascade_narrowing: None,
        cascade_confidence: None,
        polynomial_provenance: None,
        cross_file_scc: None,
    }
}

fn read_external_sifs(paths: &[PathBuf]) -> Result<Vec<OmenaQueryExternalSifInputV0>, String> {
    paths
        .iter()
        .map(|path| {
            let sif_json = read_source(path)?;
            let sif = read_omena_sif_json_v1(&sif_json)
                .map_err(|error| format!("failed to parse SIF {}: {error}", path_string(path)))?;
            Ok(OmenaQueryExternalSifInputV0 {
                canonical_url: sif.canonical_url.clone(),
                sif,
            })
        })
        .collect()
}

fn read_lock_external_sifs(lockfile: &Path) -> Result<Vec<OmenaQueryExternalSifInputV0>, String> {
    let lockfile_source = read_source(lockfile)?;
    let lock = read_omena_lock_json_v1(&lockfile_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(lockfile)))?;
    lock.entries
        .iter()
        .map(|entry| {
            let sif_path = resolve_lock_relative_path(lockfile, &entry.sif_path);
            let sif_json = read_source(&sif_path)?;
            let sif = read_omena_sif_json_v1(&sif_json).map_err(|error| {
                format!("failed to parse SIF {}: {error}", path_string(&sif_path))
            })?;
            if sif.canonical_url != entry.canonical_url {
                return Err(format!(
                    "lock entry {} points to SIF {} with canonicalUrl {}",
                    entry.canonical_url,
                    path_string(&sif_path),
                    sif.canonical_url
                ));
            }
            Ok(OmenaQueryExternalSifInputV0 {
                canonical_url: entry.canonical_url.clone(),
                sif,
            })
        })
        .collect()
}

/// Generate, in-process, the external SIFs for every on-disk (`file://`) external Sass module
/// edge in the workspace — and, transitively, every module reachable through a generated SIF's
/// `@forward` chain — so the existing SIF-mode query path can pair them against import targets
/// without a manual `--sif`. (#33)
///
/// Each `file://` `@use`/`@forward`/`@import` source now classifies as an external edge
/// (resolver `is_external_style_module_source`), so its symbols are otherwise invisible and every
/// reference flags `missingSassSymbol`. The bridge reads the resolved on-disk module and produces an
/// [`omena_sif::OmenaSifV1`]; we key the resulting `OmenaQueryExternalSifInputV0.canonical_url` to
/// the *verbatim* edge source so it matches the import target 1:1 in
/// `find_omena_query_external_sif` (the inner SIF still carries the bridge's normalized URL).
///
/// After generating a SIF the walk recurses into that SIF's `exports.forwards[].canonical_url`
/// (each a *raw* relative/bare specifier as written, e.g. `"./tokens"`), re-resolving every
/// forwarded specifier against the forwarding SIF's resolved inner `canonical_url` (a `file://`
/// URI) via the `omena_query` resolver facade and generating those modules' SIFs too. So a
/// transitively-forwarded module (A `@forward` B where B defines/re-exports the symbols) gets a
/// SIF generated even though no workspace source imports B directly. The walk is a breadth-first
/// worklist with cycle/diamond detection keyed on the *resolved* `file://` identity, so an
/// A↔B forward cycle terminates without hanging or duplicating SIFs.
///
/// Skipped, never fabricated:
/// - an edge already covered by an explicit `--sif` (matching canonical URL) — the user artifact
///   wins, so a stale/partial `--sif` is never silently overwritten by a fresh bridge SIF;
/// - a `file://` edge the bridge cannot read (a genuinely-missing module) — left out so it keeps
///   surfacing its `missingExternalSif`/`missingSassSymbol` boundary state (no over-correction);
/// - a forwarded specifier that does not resolve to an on-disk module (resolver returns `None`)
///   or that the bridge cannot read — left out so a genuinely-missing transitive forward still
///   flags;
/// - `http(s)://`/`sass:` schemes — not on-disk, so the bridge cannot read them in-process.
///
/// The query layer consumes the generated chain by flattening forwarded external-SIF exports when
/// it computes the visible Sass symbol set for the root external module.
fn resolve_in_process_external_sifs(
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Vec<OmenaQueryExternalSifInputV0> {
    // A single `file://`-namespace dedup/cycle set: seeded with the verbatim canonical URLs of any
    // explicit `--sif`, then extended with each generated SIF's resolved inner `file://` URI. A
    // workspace `file://` edge source already *is* its resolved URI, so the two keying schemes
    // coincide in that namespace.
    let mut covered = existing_external_sifs
        .iter()
        .map(|input| input.canonical_url.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mut resolved = Vec::new();
    // Worklist of generated SIFs whose `exports.forwards` still need to be walked.
    let mut worklist: std::collections::VecDeque<omena_sif::OmenaSifV1> =
        std::collections::VecDeque::new();

    // Direct workspace pass: each `file://` `@use`/`@forward`/`@import` edge written in a workspace
    // source. Key to the verbatim edge source (which already IS its resolved `file://` URI).
    for source in workspace_sources {
        let Some(module_sources) =
            summarize_omena_query_sass_module_sources(&source.style_path, &source.style_source)
        else {
            continue;
        };
        let edge_sources = module_sources
            .module_use_edges
            .iter()
            .map(|edge| edge.source.as_str())
            .chain(
                module_sources
                    .module_forward_sources
                    .iter()
                    .map(String::as_str),
            );
        for edge_source in edge_sources {
            // Only `file://` edges are on-disk external modules the bridge can read in-process.
            if !edge_source.starts_with("file://") {
                continue;
            }
            if !covered.insert(edge_source.to_string()) {
                // Already covered by an explicit `--sif` or an earlier edge in this workspace.
                continue;
            }
            // The bridge errors gracefully (never panics) on an unreadable/missing module; we
            // simply skip it so the boundary state still surfaces — we never fabricate a SIF.
            if let Ok(sif) = generate_omena_bridge_sif_for_resolved_style_path(edge_source) {
                // The bridge normalizes the path (symlinks/`..`), so the inner `sif.canonical_url`
                // can differ from the verbatim `file://` edge source. Record BOTH in `covered`:
                // the verbatim key matches the workspace import 1:1 here, and the resolved key is
                // what the transitive walk dedups on — without it a forward cycle that resolves
                // back to this module would regenerate it.
                covered.insert(sif.canonical_url.clone());
                worklist.push_back(sif.clone());
                resolved.push(OmenaQueryExternalSifInputV0 {
                    canonical_url: edge_source.to_string(),
                    sif,
                });
            }
        }
    }

    // Transitive `@forward` walk: pop a generated SIF and resolve each forwarded specifier against
    // that SIF's resolved inner `file://` base, generating the forwarded module's SIF and enqueueing
    // it so the chain (and any diamond) is followed to a fixpoint.
    while let Some(sif) = worklist.pop_front() {
        let base_file_uri = sif.canonical_url.clone();
        for forward in &sif.exports.forwards {
            let specifier = forward.canonical_url.as_str();
            // `sass:` builtins and `http(s)://` modules are not on-disk; the bridge cannot read
            // them in-process, so they keep surfacing their boundary state.
            if specifier.starts_with("sass:")
                || specifier.starts_with("http://")
                || specifier.starts_with("https://")
            {
                continue;
            }
            // Resolve the raw forwarded specifier (e.g. `"./tokens"`) relative to the forwarding
            // module's resolved `file://` URI. `None` => genuinely unresolvable; never fabricate.
            let Some(child_url) = omena_query::resolve_omena_query_style_uri_for_specifier(
                base_file_uri.as_str(),
                None,
                specifier,
            ) else {
                continue;
            };
            // Cycle/diamond guard: dedup on the resolved `file://` identity. A relative specifier
            // can reach the same physical module via different strings, so the verbatim string is
            // not a sound key — the resolved URI is.
            if !covered.insert(child_url.clone()) {
                continue;
            }
            // Unreadable/missing forwarded module: skip so it keeps surfacing its boundary state.
            if let Ok(child) = generate_omena_bridge_sif_for_resolved_style_path(child_url.as_str())
            {
                worklist.push_back(child.clone());
                // Key the entry to the resolved `file://` URI; it equals `child.canonical_url`, so
                // `find_omena_query_external_sif` matches on either field.
                resolved.push(OmenaQueryExternalSifInputV0 {
                    canonical_url: child_url,
                    sif: child,
                });
            }
        }
    }

    resolved
}

fn parse_external_module_mode(external: &str) -> Result<OmenaQueryExternalModuleModeV0, String> {
    match external {
        "ignored" => Ok(OmenaQueryExternalModuleModeV0::Ignored),
        "sif" => Ok(OmenaQueryExternalModuleModeV0::Sif),
        _ => Err(format!(
            "unsupported external mode '{external}'; expected ignored or sif"
        )),
    }
}

fn resolve_external_module_mode_for_style_diagnostics(
    external: Option<&str>,
    _lockfile: &Option<PathBuf>,
) -> Result<OmenaQueryExternalModuleModeV0, String> {
    match external {
        Some(external) => parse_external_module_mode(external),
        None => Ok(OmenaQueryExternalModuleModeV0::Sif),
    }
}

fn discover_omena_lockfile_for_path(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(directory) = current {
        let candidate = directory.join("omena.lock");
        if candidate.exists() {
            return Some(candidate);
        }
        current = directory.parent();
    }
    let cwd_candidate = PathBuf::from("omena.lock");
    cwd_candidate.exists().then_some(cwd_candidate)
}

fn style_hover_candidates(path: PathBuf, json: bool) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let Some(summary) = summarize_omena_query_style_hover_candidates(&style_path, &source) else {
        return Err(format!(
            "failed to read style candidates for {}",
            path_string(&path)
        ));
    };

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {style_path}");
    println!("language: {}", summary.language);
    println!("candidates: {}", summary.candidates.len());
    for candidate in &summary.candidates {
        println!(
            "{}\t{}\t{}",
            candidate.kind, candidate.name, candidate.source
        );
    }
    Ok(())
}

fn style_completion(
    path: PathBuf,
    line: usize,
    character: usize,
    json: bool,
) -> Result<(), String> {
    let source = read_source(&path)?;
    let style_path = path_string(&path);
    let Some(candidates) = summarize_omena_query_style_hover_candidates(&style_path, &source)
    else {
        return Err(format!(
            "failed to read style candidates for {}",
            path_string(&path)
        ));
    };
    let summary = summarize_omena_query_style_completion_at_position(
        &style_path,
        &source,
        ParserPositionV0 { line, character },
        candidates.candidates.as_slice(),
    );

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("context: {}", summary.context_kind);
    println!("items: {}", summary.item_count);
    for item in &summary.items {
        println!("{}\t{}\t{}", item.label, item.detail, item.source);
    }
    Ok(())
}

fn source_diagnostics(
    source_uri: String,
    candidates_json: Option<PathBuf>,
    source_path: Option<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let summary = source_diagnostics_summary(
        source_uri,
        candidates_json,
        source_path,
        source_paths,
        package_manifest_paths,
    )?;

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("diagnostics: {}", summary.diagnostic_count);
    for diagnostic in &summary.diagnostics {
        println!("{}\t{}", diagnostic.code, diagnostic.message);
    }
    Ok(())
}

fn dynamic_classname_diagnostics(input_json: PathBuf, json: bool) -> Result<(), String> {
    let summary = dynamic_classname_diagnostics_summary(&input_json)?;

    if json {
        print_json(&summary)?;
        return Ok(());
    }

    println!("file: {}", summary.file_uri);
    println!("diagnostics: {}", summary.diagnostic_count);
    for diagnostic in &summary.diagnostics {
        println!("{}\t{}", diagnostic.code, diagnostic.message);
    }
    Ok(())
}

fn perceptual_check(path: PathBuf, json: bool) -> Result<(), String> {
    let report = perceptual_check_summary(&path)?;

    if json {
        print_json(&report)?;
        return Ok(());
    }

    println!("product: {}", report.product);
    println!("file: {}", report.style_path);
    println!("language: {}", report.language);
    println!("claim level: {}", report.claim_level);
    println!("selectors: {}", report.selector_count);
    println!(
        "custom property declarations: {}",
        report.custom_property_declaration_count
    );
    println!(
        "custom property references: {}",
        report.custom_property_reference_count
    );
    println!("diagnostics: {}", report.diagnostic_count);
    println!(
        "downstream scaffold ready: {}",
        report.downstream_tool_scaffold_ready
    );
    println!(
        "WCAG exact color contrast bounds: {}",
        report.wcag_exact_color_contrast_bound_count
    );
    println!(
        "full perceptual algorithm ready: {}",
        report.full_perceptual_algorithm_ready
    );
    Ok(())
}

fn perceptual_check_summary(path: &Path) -> Result<PerceptualCheckCliReportV0, String> {
    let source = read_source(path)?;
    let style_path = path_string(path);
    let style_document = summarize_omena_query_style_document(&style_path, &source)
        .ok_or_else(|| format!("failed to read style document facts for {style_path}"))?;
    let check = summarize_omena_query_consumer_check_style_source(&style_path, &source);
    let wcag_exact_color_contrast_bounds = collect_wcag_exact_color_contrast_bounds_v0(&source);
    let wcag_exact_color_contrast_bound_count = wcag_exact_color_contrast_bounds.len();

    Ok(PerceptualCheckCliReportV0 {
        schema_version: "0",
        product: "omena-cli.perceptual-check",
        command: "perceptual-check",
        claim_level: "fixtureWitnessExactColorWcagContrast",
        style_path,
        language: style_document.language,
        fact_source_products: vec![style_document.product, check.product],
        selector_count: style_document.selector_names.len(),
        custom_property_declaration_count: style_document.custom_property_decl_names.len(),
        custom_property_reference_count: style_document.custom_property_ref_names.len(),
        diagnostic_count: style_document
            .diagnostic_count
            .max(check.parser_error_count),
        color_machinery_source: "omena-cli.perceptual-check.exact-srgb-wcag",
        json_schema_ready: true,
        downstream_tool_scaffold_ready: true,
        consumes_omena_facts: true,
        wcag_algorithm_ready: wcag_exact_color_contrast_bound_count > 0,
        wcag_exact_color_contrast_bound_count,
        wcag_exact_color_contrast_bounds,
        apca_algorithm_ready: false,
        oklab_perceptual_operator_ready: false,
        full_perceptual_algorithm_ready: false,
        public_safety_claim_ready: false,
        supported_claims: vec![
            "perceptual-check CLI report",
            "JSON output schema",
            "Omena fact-level input consumption",
            "WCAG contrast bound for exact sRGB color/background pairs",
        ],
        deferred_claims: vec![
            "non-exact and cascade-computed color contrast",
            "APCA algorithm",
            "OKLab perceptual operator",
            "full perceptual algorithm",
            "public safety claim",
        ],
    })
}

fn collect_wcag_exact_color_contrast_bounds_v0(
    source: &str,
) -> Vec<PerceptualExactColorContrastBoundV0> {
    let mut bounds = Vec::new();
    for block in source.split('}') {
        let Some((selector_text, declaration_text)) = block.split_once('{') else {
            continue;
        };
        let selector_name =
            extract_first_class_selector_name_v0(selector_text).unwrap_or_else(|| {
                selector_text
                    .trim()
                    .split(',')
                    .next()
                    .unwrap_or("<unknown>")
                    .trim()
                    .to_string()
            });
        let mut foreground = None;
        let mut background = None;
        for declaration in declaration_text.split(';') {
            let Some((property, value)) = declaration.split_once(':') else {
                continue;
            };
            let property = property.trim().to_ascii_lowercase();
            let value = strip_declaration_priority_v0(value.trim());
            let Some(color) = parse_perceptual_exact_srgb_color_v0(value) else {
                continue;
            };
            match property.as_str() {
                "color" => {
                    foreground = Some(PerceptualDeclarationColorV0 {
                        property: "color",
                        value: value.to_string(),
                        color,
                    });
                }
                "background" | "background-color" => {
                    background = Some(PerceptualDeclarationColorV0 {
                        property: if property == "background" {
                            "background"
                        } else {
                            "background-color"
                        },
                        value: value.to_string(),
                        color,
                    });
                }
                _ => {}
            }
        }
        let (Some(foreground), Some(background)) = (foreground, background) else {
            continue;
        };
        let foreground_luminance = wcag_relative_luminance_v0(foreground.color);
        let background_luminance = wcag_relative_luminance_v0(background.color);
        let contrast_ratio = wcag_contrast_ratio_v0(foreground_luminance, background_luminance);
        bounds.push(PerceptualExactColorContrastBoundV0 {
            schema_version: "0",
            product: "omena-cli.perceptual-check.wcag-exact-color-contrast",
            feature_gate: "wcag-exact-color-contrast-v0",
            claim_level: "fixtureWitnessExactColorWcagContrast",
            selector_name,
            foreground_property: foreground.property,
            background_property: background.property,
            foreground: foreground.value,
            background: background.value,
            foreground_luminance,
            background_luminance,
            contrast_ratio,
            wcag_aa_normal_text_threshold: 4.5,
            passes_aa_normal_text: contrast_ratio >= 4.5,
            public_safety_claim_ready: false,
        });
    }
    bounds
}

fn extract_first_class_selector_name_v0(selector_text: &str) -> Option<String> {
    let start = selector_text.find('.')? + 1;
    let name = selector_text[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn strip_declaration_priority_v0(value: &str) -> &str {
    value
        .strip_suffix("!important")
        .map(str::trim)
        .unwrap_or(value)
}

fn parse_perceptual_exact_srgb_color_v0(value: &str) -> Option<PerceptualExactSrgbColorV0> {
    let trimmed = value.trim();
    parse_perceptual_hex_color_v0(trimmed)
        .or_else(|| parse_perceptual_basic_named_color_v0(trimmed))
        .or_else(|| parse_perceptual_rgb_function_v0(trimmed))
}

fn parse_perceptual_hex_color_v0(value: &str) -> Option<PerceptualExactSrgbColorV0> {
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let mut chars = hex.chars();
            Some(PerceptualExactSrgbColorV0 {
                red: parse_repeated_hex_digit_v0(chars.next()?)?,
                green: parse_repeated_hex_digit_v0(chars.next()?)?,
                blue: parse_repeated_hex_digit_v0(chars.next()?)?,
            })
        }
        6 => Some(PerceptualExactSrgbColorV0 {
            red: u8::from_str_radix(hex.get(0..2)?, 16).ok()?,
            green: u8::from_str_radix(hex.get(2..4)?, 16).ok()?,
            blue: u8::from_str_radix(hex.get(4..6)?, 16).ok()?,
        }),
        _ => None,
    }
}

fn parse_repeated_hex_digit_v0(ch: char) -> Option<u8> {
    let value = ch.to_digit(16)? as u8;
    Some(value * 17)
}

fn parse_perceptual_basic_named_color_v0(value: &str) -> Option<PerceptualExactSrgbColorV0> {
    match value.to_ascii_lowercase().as_str() {
        "black" => Some(PerceptualExactSrgbColorV0 {
            red: 0,
            green: 0,
            blue: 0,
        }),
        "white" => Some(PerceptualExactSrgbColorV0 {
            red: 255,
            green: 255,
            blue: 255,
        }),
        "red" => Some(PerceptualExactSrgbColorV0 {
            red: 255,
            green: 0,
            blue: 0,
        }),
        "green" => Some(PerceptualExactSrgbColorV0 {
            red: 0,
            green: 128,
            blue: 0,
        }),
        "blue" => Some(PerceptualExactSrgbColorV0 {
            red: 0,
            green: 0,
            blue: 255,
        }),
        _ => None,
    }
}

fn parse_perceptual_rgb_function_v0(value: &str) -> Option<PerceptualExactSrgbColorV0> {
    let inner = value
        .strip_prefix("rgb(")
        .or_else(|| value.strip_prefix("rgba("))?
        .strip_suffix(')')?;
    if inner.contains('/') {
        return None;
    }
    let components = inner
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let [red, green, blue] = components.as_slice() else {
        return None;
    };
    Some(PerceptualExactSrgbColorV0 {
        red: parse_perceptual_rgb_channel_v0(red)?,
        green: parse_perceptual_rgb_channel_v0(green)?,
        blue: parse_perceptual_rgb_channel_v0(blue)?,
    })
}

fn parse_perceptual_rgb_channel_v0(value: &str) -> Option<u8> {
    let parsed = value.parse::<u8>().ok()?;
    Some(parsed)
}

fn wcag_relative_luminance_v0(color: PerceptualExactSrgbColorV0) -> f64 {
    0.2126 * wcag_linear_srgb_channel_v0(color.red)
        + 0.7152 * wcag_linear_srgb_channel_v0(color.green)
        + 0.0722 * wcag_linear_srgb_channel_v0(color.blue)
}

fn wcag_linear_srgb_channel_v0(channel: u8) -> f64 {
    let value = f64::from(channel) / 255.0;
    if value <= 0.039_28 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn wcag_contrast_ratio_v0(left_luminance: f64, right_luminance: f64) -> f64 {
    let lighter = left_luminance.max(right_luminance);
    let darker = left_luminance.min(right_luminance);
    (lighter + 0.05) / (darker + 0.05)
}

fn source_diagnostics_summary(
    source_uri: String,
    candidates_json: Option<PathBuf>,
    source_path: Option<PathBuf>,
    source_paths: Vec<PathBuf>,
    package_manifest_paths: Vec<PathBuf>,
) -> Result<OmenaQuerySourceDiagnosticsForFileV0, String> {
    if let Some(candidates_json) = candidates_json {
        let candidates = read_source_diagnostic_candidates_json(&candidates_json)?;
        Ok(summarize_omena_query_source_diagnostics_for_file(
            source_uri.as_str(),
            candidates.as_slice(),
        ))
    } else {
        let source_path = source_path.ok_or_else(|| {
            "source-diagnostics requires either --candidates-json or --source-path".to_string()
        })?;
        let source_source = read_source(&source_path)?;
        let style_sources = read_style_sources(&source_paths)?;
        let package_manifests = read_package_manifests(&package_manifest_paths)?;
        Ok(summarize_omena_query_source_diagnostics_for_workspace_file(
            source_uri.as_str(),
            source_source.as_str(),
            style_sources.as_slice(),
            package_manifests.as_slice(),
        ))
    }
}

fn dynamic_classname_diagnostics_summary(
    input_json: &Path,
) -> Result<OmenaQuerySourceDiagnosticsForFileV0, String> {
    let json = fs::read_to_string(input_json).map_err(|error| {
        format!(
            "failed to read dynamic className diagnostics input JSON {}: {error}",
            path_string(input_json)
        )
    })?;
    let input: OmenaQueryDynamicClassnameMTierInputV0 =
        serde_json::from_str(&json).map_err(|error| {
            format!(
                "failed to parse dynamic className diagnostics input JSON {}: {error}",
                path_string(input_json)
            )
        })?;
    Ok(summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth(&input))
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path_string(path)))
}

fn read_context_json(path: Option<&Path>) -> Result<OmenaQueryTransformExecutionContextV0, String> {
    let Some(path) = path else {
        return Ok(OmenaQueryTransformExecutionContextV0::default());
    };
    let json = fs::read_to_string(path)
        .map_err(|error| format!("failed to read context JSON {}: {error}", path_string(path)))?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse context JSON {}: {error}",
            path_string(path)
        )
    })
}

fn read_engine_input_json(path: &Path) -> Result<OmenaQueryEngineInputV2, String> {
    let json = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read engine input JSON {}: {error}",
            path_string(path)
        )
    })?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse engine input JSON {}: {error}",
            path_string(path)
        )
    })
}

fn read_source_diagnostic_candidates_json(
    path: &Path,
) -> Result<Vec<OmenaQuerySourceMissingSelectorDiagnosticCandidateV0>, String> {
    let json = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read source diagnostics candidates JSON {}: {error}",
            path_string(path)
        )
    })?;
    serde_json::from_str(&json).map_err(|error| {
        format!(
            "failed to parse source diagnostics candidates JSON {}: {error}",
            path_string(path)
        )
    })
}

fn empty_engine_input() -> OmenaQueryEngineInputV2 {
    OmenaQueryEngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: Vec::new(),
    }
}

fn merge_cli_transform_context(
    mut base: OmenaQueryTransformExecutionContextV0,
    additional: &OmenaQueryTransformExecutionContextV0,
) -> OmenaQueryTransformExecutionContextV0 {
    base.closed_style_world = base.closed_style_world || additional.closed_style_world;
    base.drop_dark_mode_media_queries =
        base.drop_dark_mode_media_queries || additional.drop_dark_mode_media_queries;
    merge_cli_context_list(
        &mut base.reachable_class_names,
        &additional.reachable_class_names,
    );
    merge_cli_context_list(
        &mut base.reachable_keyframe_names,
        &additional.reachable_keyframe_names,
    );
    merge_cli_context_list(
        &mut base.reachable_value_names,
        &additional.reachable_value_names,
    );
    merge_cli_context_list(
        &mut base.reachable_custom_property_names,
        &additional.reachable_custom_property_names,
    );
    base
}

fn append_tree_shake_build_passes(pass_ids: &mut Vec<String>) {
    for pass_id in [
        "tree-shake-class",
        "tree-shake-keyframes",
        "tree-shake-value",
        "tree-shake-custom-property",
    ] {
        if !pass_ids.iter().any(|existing| existing == pass_id) {
            pass_ids.push(pass_id.to_string());
        }
    }
}

fn append_bundle_build_passes(pass_ids: &mut Vec<String>, style_path: &str, source: &str) {
    let bundle = summarize_omena_transform_bundle_from_source(
        style_path,
        source,
        infer_cli_style_dialect(style_path),
    );
    for pass_id in bundle.planned_pass_ids {
        if !pass_ids.iter().any(|existing| existing == pass_id) {
            pass_ids.push(pass_id.to_string());
        }
    }
}

fn infer_cli_style_dialect(style_path: &str) -> OmenaParserStyleDialect {
    match Path::new(style_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("scss") => OmenaParserStyleDialect::Scss,
        Some("sass") => OmenaParserStyleDialect::Sass,
        Some("less") => OmenaParserStyleDialect::Less,
        _ => OmenaParserStyleDialect::Css,
    }
}

fn push_ready_surface(surfaces: &mut Vec<&'static str>, surface: &'static str) {
    if !surfaces.contains(&surface) {
        surfaces.push(surface);
    }
}

fn merge_cli_context_list(target: &mut Vec<String>, additional: &[String]) {
    for item in additional {
        if !target.contains(item) {
            target.push(item.clone());
        }
    }
    target.sort();
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize JSON: {error}"))?;
    println!("{json}");
    Ok(())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn style_resolution_workspace_uri_for_path(path: &Path) -> Option<String> {
    path.parent()
        .and_then(discover_style_resolution_workspace_root)
        .map(|workspace_root| format!("file://{}", workspace_root.to_string_lossy()))
}

fn discover_style_resolution_workspace_root(path: &Path) -> Option<&Path> {
    path.ancestors().find(|candidate| {
        [
            "tsconfig.json",
            "tsconfig.base.json",
            "jsconfig.json",
            "package.json",
            "vite.config.ts",
            "vite.config.js",
            "webpack.config.js",
        ]
        .iter()
        .any(|marker| candidate.join(marker).is_file())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        assert!(build_argument_names.contains(&"tree-shake"));
        assert!(build_argument_names.contains(&"bundle"));
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
    fn lock_verify_attestation_t3_requires_signed_statement_artifact_digest_binding()
    -> Result<(), String> {
        let sif = cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let sif_source = omena_sif::write_omena_sif_json_v1(&sif)
            .map_err(|error| format!("fixture SIF should serialize: {error}"))?;
        let entry = omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| format!("fixture lock entry should build: {error}"))?;
        let binding = VerifiedT3AttestationArtifactBinding {
            canonical_url: sif.canonical_url.clone(),
            sif_hash: omena_sif::compute_omena_sif_artifact_hash_v1(&sif)
                .map_err(|error| format!("fixture SIF should hash: {error}"))?,
            artifact_sha256: sha256_hex(sif_source.as_bytes()),
        };
        let mut statement = cli_fixture_provenance_statement();
        statement.subject_names = vec![entry.sif_path.clone()];
        statement.subject_digests = vec![omena_sif::OmenaSifAttestationSubjectDigestV1 {
            name: entry.sif_path.clone(),
            algorithm: "sha256".to_string(),
            digest: binding.artifact_sha256.clone(),
        }];

        validate_verified_t3_attestation_statement_binding(&entry, &binding, Some(&statement))?;

        let mut digest_mismatch = statement.clone();
        digest_mismatch.subject_digests = vec![omena_sif::OmenaSifAttestationSubjectDigestV1 {
            name: entry.sif_path.clone(),
            algorithm: "sha256".to_string(),
            digest: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        }];
        let digest_result = validate_verified_t3_attestation_statement_binding(
            &entry,
            &binding,
            Some(&digest_mismatch),
        );
        assert!(
            digest_result
                .as_ref()
                .is_err_and(|error| error.contains("subjectDigests")),
            "{digest_result:?}"
        );

        let mut subject_mismatch = statement;
        subject_mismatch.subject_names = vec!["sif/other.sif.json".to_string()];
        let subject_result = validate_verified_t3_attestation_statement_binding(
            &entry,
            &binding,
            Some(&subject_mismatch),
        );
        assert!(
            subject_result
                .as_ref()
                .is_err_and(|error| error.contains("subjectNames")),
            "{subject_result:?}"
        );

        let missing_statement =
            validate_verified_t3_attestation_statement_binding(&entry, &binding, None);
        assert!(
            missing_statement
                .as_ref()
                .is_err_and(|error| error.contains("signed SIF provenance statement")),
            "{missing_statement:?}"
        );

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
                "lock verify-attestation --verified-tier t3 requires --kind omena-toolchain.*"
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
    fn report_resolution_policy_outputs_resolver_contract() {
        let result = run(Cli {
            command: Command::Report {
                command: ReportCommand::ResolutionPolicy { json: true },
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
                split_out_dir: None,
                source_paths: Vec::new(),
                package_manifest_paths: Vec::new(),
                source_map: false,
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
    fn build_source_map_requires_json_output() -> Result<(), String> {
        let source_path = temp_path("source-map-requires-json.css");
        fs::write(&source_path, ".card { color: red; }")
            .map_err(|error| format!("fixture source should be writable: {error}"))?;

        let result = run(Cli {
            command: Command::Build {
                path: source_path.clone(),
                output: None,
                passes: vec!["print-css".to_string()],
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
                split_out_dir: None,
                source_paths: Vec::new(),
                package_manifest_paths: Vec::new(),
                source_map: true,
                json: false,
            },
        });

        assert_eq!(result, Err("--source-map requires --json".to_string()));
        cleanup(&source_path);
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
                split_out_dir: None,
                source_paths: Vec::new(),
                package_manifest_paths: Vec::new(),
                source_map: false,
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
                split_out_dir: None,
                source_paths: Vec::new(),
                package_manifest_paths: Vec::new(),
                source_map: false,
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
            format!(r#"@import "./{tokens_file_name}"; .app {{ color: green; }}"#),
        )
        .map_err(|error| format!("fixture target source should be writable: {error}"))?;

        let result = run(Cli {
            command: Command::Build {
                path: target_path.clone(),
                output: Some(output_path.clone()),
                passes: Vec::new(),
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
                split_out_dir: None,
                source_paths: vec![tokens_path.clone(), base_path.clone()],
                package_manifest_paths: Vec::new(),
                source_map: false,
                json: false,
            },
        });

        assert!(result.is_ok(), "{result:?}");
        let output = fs::read_to_string(&output_path)
            .map_err(|error| format!("bundle output should be written: {error}"))?;
        assert!(output.contains(".base { color: red; }"));
        assert!(output.contains(".token { color: blue; }"));
        assert!(output.contains(".app { color: green; }"));
        assert!(!output.contains("@import"));

        cleanup(&target_path);
        cleanup(&tokens_path);
        cleanup(&base_path);
        cleanup(&output_path);
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
                split_out_dir: Some(split_dir.clone()),
                source_paths: vec![tokens_path.clone(), base_path.clone()],
                package_manifest_paths: Vec::new(),
                source_map: false,
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
        let manifest_json =
            fs::read_to_string(split_dir.join(BUNDLE_CODE_SPLIT_MANIFEST_FILE_NAME))
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
                value.get("fileName").and_then(|value| value.as_str())
                    == Some(base_split_file.as_str())
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
                split_out_dir: Some(split_dir.clone()),
                source_paths: vec![tokens_path.clone()],
                package_manifest_paths: Vec::new(),
                source_map: false,
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
        assert!(main_output.contains(".token"), "{main_output}");
        assert!(!main_output.contains("dead"), "{main_output}");
        assert!(target_output.contains(".used"), "{target_output}");
        assert!(!target_output.contains(".dead"), "{target_output}");
        assert!(tokens_output.contains(".token"), "{tokens_output}");
        assert!(!tokens_output.contains(".ghost"), "{tokens_output}");

        cleanup_dir(&root);
        Ok(())
    }

    #[test]
    fn build_scss_module_mode_shares_preconfigured_transitive_module_instance() -> Result<(), String>
    {
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
                split_out_dir: None,
                source_paths: vec![tokens_path.clone(), theme_path.clone()],
                package_manifest_paths: Vec::new(),
                source_map: false,
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
                split_out_dir: None,
                source_paths: vec![tokens_path.clone(), theme_path.clone()],
                package_manifest_paths: Vec::new(),
                source_map: false,
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
                split_out_dir: None,
                source_paths: vec![tokens_path.clone()],
                package_manifest_paths: Vec::new(),
                source_map: false,
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
                split_out_dir: None,
                source_paths: vec![tokens_path.clone()],
                package_manifest_paths: Vec::new(),
                source_map: true,
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
        fs::write(&source_path, ".card { color: red; }\n")
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
    fn style_diagnostics_command_accepts_external_sif_mode() -> Result<(), String> {
        let source_path = temp_path("external-sif.module.scss");
        fs::write(
            &source_path,
            r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$color; }"#,
        )
        .map_err(|error| format!("fixture source should be writable: {error}"))?;

        let result = run(Cli {
            command: Command::StyleDiagnostics {
                path: source_path.clone(),
                source_paths: vec![source_path.clone()],
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
                source_paths: vec![source_path.clone()],
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

        let resolved = resolve_in_process_external_sifs(std::slice::from_ref(&entry), &[]);

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
        let resolved = resolve_in_process_external_sifs(std::slice::from_ref(&entry), &[]);

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

        let resolved = resolve_in_process_external_sifs(std::slice::from_ref(&entry), &[]);

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

        cleanup(&candidates_path);
        Ok(())
    }

    #[test]
    fn source_diagnostics_command_reads_workspace_query_owned_diagnostics() -> Result<(), String> {
        let source_path = temp_path("App.tsx");
        let style_path = temp_path("App.module.scss");
        fs::write(
            &source_path,
            r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const variant = Math.random() > 0.5 ? "chip" : "ghost";
export function App() {
  return <div className={cx(variant)} />;
}
"#,
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

        cleanup(&source_path);
        cleanup(&style_path);
        Ok(())
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
                .all(|diagnostic| diagnostic.code != "missingModule"),
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
                .all(|diagnostic| diagnostic.code != "missingModule"),
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
    fn perceptual_check_command_emits_exact_color_wcag_witness_from_query_facts()
    -> Result<(), String> {
        let source_path = temp_path("perceptual.module.css");
        fs::write(
            &source_path,
            ":root { --fg: #000; }\n.button { color: #000; background: #fff; border-color: var(--fg); }\n",
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
        let locked_sif =
            cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let changed_sif =
            cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: blue !default;")?;
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
        let mut entry =
            omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
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
            algorithm: "blake3".to_string(),
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
                certificate_issuer: Some("https://github.com/login/oauth".to_string()),
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
        let mut entry =
            omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
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
                sigstore_verification_policy: Some(
                    omena_sif::OmenaSifSigstoreVerificationPolicyV1 {
                        trusted_root: "sigstore-production-trusted-root".to_string(),
                        transparency_log: true,
                        timestamp: true,
                        certificate_chain: true,
                        signed_certificate_timestamp: true,
                    },
                ),
                certificate_issuer: Some("https://github.com/login/oauth".to_string()),
                certificate_identity: None,
                attestation_statement: None,
            });
        let lock = omena_sif::OmenaLockV1::new(vec![entry]);
        let issues = collect_lock_trust_tier_issues(&lock, omena_sif::OmenaSifTrustTierV1::T3);
        assert_eq!(issues.len(), 1, "{issues:?}");
        assert_eq!(issues[0].code, "attestationVerificationInvalid");
        assert!(
            issues[0].message.contains("requires certificateIdentity"),
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

        let issues =
            collect_lock_source_coverage_issues(&lock, std::slice::from_ref(&source_path))?;
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
    fn lock_update_authors_lock_then_verify_frozen_passes_and_fails_when_tampered()
    -> Result<(), String> {
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
        let tampered =
            cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: blue !default;")?;
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
    fn lock_fetch_provenance_records_npm_attestations_without_t2_verification() -> Result<(), String>
    {
        let workspace_path = temp_dir("lock-fetch-provenance");
        let sif_dir = workspace_path.join("sif");
        fs::create_dir_all(&sif_dir)
            .map_err(|error| format!("fixture SIF dir should be writable: {error}"))?;
        let sif_path = sif_dir.join("design-system.sif.json");
        let metadata_path = workspace_path.join("npm-metadata.json");
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
    fn lock_record_verification_enables_t2_trust_gate() -> Result<(), String> {
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
                .is_err_and(|error| error.contains("tier t3 requires kind omena-toolchain.*")),
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
        assert!(record_result.is_ok(), "{record_result:?}");

        let refreshed_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
            .map_err(|error| format!("fixture lock should parse: {error}"))?;
        assert_eq!(
            refreshed_lock.entries[0].trust_tier,
            omena_sif::OmenaSifTrustTierV1::T2
        );
        assert_eq!(refreshed_lock.entries[0].attestation_verifications.len(), 1);
        assert_eq!(
            refreshed_lock.entries[0].attestation_verifications[0]
                .certificate_issuer
                .as_deref(),
            Some("https://token.actions.githubusercontent.com")
        );
        assert_eq!(
            refreshed_lock.entries[0].attestation_verifications[0]
                .certificate_identity
                .as_deref(),
            Some(
                "https://github.com/omenien/omena-css/.github/workflows/release.yml@refs/tags/v1.0.0"
            )
        );
        let statement = refreshed_lock.entries[0].attestation_verifications[0]
            .attestation_statement
            .as_ref()
            .ok_or_else(|| "recorded verification should preserve statement claims".to_string())?;
        assert_eq!(
            statement.predicate_type.as_deref(),
            Some("https://slsa.dev/provenance/v1")
        );
        assert_eq!(
            statement.source_repository.as_deref(),
            Some("https://github.com/omenien/omena-css")
        );
        assert_eq!(statement.source_ref.as_deref(), Some("refs/tags/v1.0.0"));

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
        assert!(verify_t2_after.is_ok(), "{verify_t2_after:?}");

        cleanup_dir(&workspace_path);
        Ok(())
    }

    #[test]
    fn lock_record_verification_t3_requires_matching_sif_artifact_digest() -> Result<(), String> {
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
        let mut entry =
            omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
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
                .is_err_and(|error| error.contains("verifiedTrustTier t3 requires --artifact")),
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
        assert!(record_result.is_ok(), "{record_result:?}");

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
        assert!(verify_t3_after.is_ok(), "{verify_t3_after:?}");

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
    fn lock_verify_attestation_records_verified_sigstore_bundle() -> Result<(), String> {
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
        let mut entry =
            omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
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
            verify_attestation_result.is_ok(),
            "{verify_attestation_result:?}"
        );

        let refreshed_lock = read_omena_lock_json_v1(&read_source(&lockfile_path)?)
            .map_err(|error| format!("fixture lock should parse: {error}"))?;
        assert_eq!(
            refreshed_lock.entries[0].trust_tier,
            omena_sif::OmenaSifTrustTierV1::T2
        );
        assert_eq!(refreshed_lock.entries[0].attestation_verifications.len(), 1);
        assert_eq!(
            refreshed_lock.entries[0].attestation_verifications[0].kind,
            "sigstore-bundle"
        );
        assert_eq!(
            refreshed_lock.entries[0].attestation_verifications[0].verifier,
            "sigstore-verify"
        );
        assert_eq!(
            refreshed_lock.entries[0].attestation_verifications[0].verified_trust_tier,
            omena_sif::OmenaSifTrustTierV1::T2
        );
        assert_eq!(
            refreshed_lock.entries[0].attestation_verifications[0]
                .certificate_issuer
                .as_deref(),
            Some("https://github.com/login/oauth")
        );
        assert_eq!(
            refreshed_lock.entries[0].attestation_verifications[0]
                .certificate_identity
                .as_deref(),
            None
        );
        let policy = refreshed_lock.entries[0].attestation_verifications[0]
            .sigstore_verification_policy
            .as_ref()
            .ok_or_else(|| "verified sigstore evidence should retain its policy".to_string())?;
        assert_eq!(policy.trusted_root, "sigstore-production-trusted-root");
        assert!(policy.transparency_log);
        assert!(policy.timestamp);
        assert!(policy.certificate_chain);
        assert!(policy.signed_certificate_timestamp);

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
        assert!(verify_t2_after.is_ok(), "{verify_t2_after:?}");

        cleanup_dir(&workspace_path);
        Ok(())
    }

    #[test]
    fn lock_verify_attestation_rejects_non_provenance_bundle_for_provenance_kind()
    -> Result<(), String> {
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
        let mut entry =
            omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
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
                .is_err_and(|error| error.contains("requires a DSSE in-toto provenance bundle")),
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
    fn lock_verify_attestation_t3_rejects_non_sif_artifact() -> Result<(), String> {
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
            error.contains("requires --artifact to be the SIF JSON"),
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
        let updated_design_sif =
            cli_fixture_sif("pkg:design-system/_tokens.scss", b"$color: green;")?;
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
        let mut entry =
            omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
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
        let mut entry =
            omena_sif::build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
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
                sigstore_verification_policy: Some(
                    omena_sif::OmenaSifSigstoreVerificationPolicyV1 {
                        trusted_root: "sigstore-production-trusted-root".to_string(),
                        transparency_log: true,
                        timestamp: true,
                        certificate_chain: true,
                        signed_certificate_timestamp: true,
                    },
                ),
                certificate_issuer: Some("https://github.com/login/oauth".to_string()),
                certificate_identity: Some("w.vollprecht@gmail.com".to_string()),
                attestation_statement: Some(cli_fixture_provenance_statement()),
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
            style_source: "@use \"./tokens\" as tokens;\n.root { color: tokens.$brand; }\n"
                .to_string(),
        };
        let sources = vec![tokens.clone(), importer.clone()];

        // The importer reaches a foreign module over the resolved `@use` edge: the streaming-IFDS
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
    fn style_diagnostics_cli_identity_reads_configured_module_instance_key() {
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

        let diagnostics = summarize_sass_module_resolution_identity_diagnostics(
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
            "CLI diagnostics must consume module_instance_identity_key through the graph closure: {diagnostics:?}"
        );
    }

    #[test]
    fn style_diagnostics_cli_identity_reports_conflicting_sass_module_configurations() {
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
                style_source:
                    r#"@use "./theme-red" as redTheme; @use "./theme-blue" as blueTheme;"#
                        .to_string(),
            },
        ];

        let diagnostics = summarize_sass_module_resolution_identity_diagnostics(
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
            "CLI diagnostics must reject incompatible Sass module configurations: {diagnostics:?}"
        );
    }

    #[test]
    fn style_diagnostics_cli_identity_allows_shared_sass_module_configuration() {
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
                style_source: r#"@use "./theme-a" as themeA; @use "./theme-b" as themeB;"#
                    .to_string(),
            },
        ];

        let diagnostics = summarize_sass_module_resolution_identity_diagnostics(
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
    fn style_diagnostics_cli_identity_reports_configured_after_unconfigured_load_order() {
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

        let diagnostics = summarize_sass_module_resolution_identity_diagnostics(
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
            "CLI diagnostics must reject configuring a Sass module after an unconfigured load: {diagnostics:?}"
        );
    }

    #[test]
    fn style_diagnostics_cli_identity_reports_non_default_sass_module_configuration() {
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

        let diagnostics = summarize_sass_module_resolution_identity_diagnostics(
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
            "CLI diagnostics must reject non-!default Sass module configuration: {diagnostics:?}"
        );
    }

    #[test]
    fn style_diagnostics_cli_identity_reports_downstream_configuration_after_forward_override() {
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

        let diagnostics = summarize_sass_module_resolution_identity_diagnostics(
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
            "CLI diagnostics must reject downstream configuration after a non-default @forward with(...): {diagnostics:?}"
        );
    }

    #[test]
    fn style_diagnostics_cli_identity_uses_downstream_forward_default_configuration() {
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

        let diagnostics = summarize_sass_module_resolution_identity_diagnostics(
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
            "CLI diagnostics must propagate downstream Sass configuration through @forward !default: {diagnostics:?}"
        );
        assert!(
            diagnostics.iter().all(|diagnostic| {
                !(diagnostic.code == "sassModuleInstanceIdentity"
                    && diagnostic.message.contains("/tmp/tokens.scss")
                    && diagnostic.message.contains("brand=3:red"))
            }),
            "CLI diagnostics must not report the @forward !default value after a downstream override: {diagnostics:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn style_diagnostics_cli_identity_reads_symlink_chain_metadata() -> Result<(), String> {
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

        let diagnostics = summarize_sass_module_resolution_identity_diagnostics(
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
            "CLI diagnostics must consume symlink_chain_links: {diagnostics:?}"
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
                "https://slsa-framework.github.io/github-actions-buildtypes/workflow/v1"
                    .to_string(),
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
}

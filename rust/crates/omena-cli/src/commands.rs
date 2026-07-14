use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "omena",
    about = "Check and transform CSS-family sources with the Omena CSS workspace"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Command {
    /// Run the unified format, lint, diagnostic, and safe-fix workflow.
    Check {
        /// Compatibility input for the parser-facts command. Use `omena facts` instead.
        path: Option<PathBuf>,
        /// Apply formatting and evidence-backed safe fixes when the unified workflow is wired.
        #[arg(long)]
        write: bool,
        /// Print machine-readable JSON for the compatibility parser-facts command.
        #[arg(long)]
        json: bool,
    },
    /// Parse a CSS-family source and report parser-owned facts.
    Facts {
        /// CSS, SCSS, Sass, Less, or CSS Modules file to inspect.
        path: PathBuf,
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Run semantic and compatibility lint rules.
    Lint {
        /// Workspace root or individual source file. Defaults to the current directory.
        root: Option<PathBuf>,
        /// Rule profile to activate.
        #[arg(long, value_enum)]
        profile: Option<LintProfile>,
        /// Optional Stylelint configuration for compatibility rules.
        #[arg(long = "stylelint-config")]
        stylelint_config: Option<PathBuf>,
        /// Apply source edits that pass the shared FixSafety gate.
        #[arg(long)]
        write: bool,
        /// Print a machine-readable lint report.
        #[arg(long)]
        json: bool,
    },
    /// Format CSS-family sources through the typed CST formatter contract.
    Fmt {
        /// Workspace root or individual CSS-family source. Defaults to the current directory.
        path: Option<PathBuf>,
        /// Formatting strategy.
        #[arg(long, value_enum)]
        mode: Option<FormatMode>,
        /// Check formatting without writing changes.
        #[arg(long)]
        check: bool,
        /// Print a machine-readable formatting report.
        #[arg(long)]
        json: bool,
    },
    /// Minify a stylesheet with an explicit semantic profile and backend.
    Minify {
        /// CSS-family source to minify.
        input: Option<PathBuf>,
        /// Safety and closed-world profile.
        #[arg(long, value_enum)]
        profile: Option<MinifyProfile>,
        /// Minification backend.
        #[arg(long, value_enum)]
        backend: Option<MinifyBackend>,
        /// Transform context containing closed-world reachability evidence.
        #[arg(long)]
        context_json: Option<PathBuf>,
        /// Write the resulting CSS to a file instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Print a machine-readable report including typed transform decisions.
        #[arg(long)]
        json: bool,
    },
    /// Bundle a source entry and emit CSS plus optional evidence.
    Bundle {
        /// Source entry whose style graph should be bundled.
        entry: Option<PathBuf>,
        /// CSS output path.
        #[arg(long = "css-out")]
        css_out: Option<PathBuf>,
        /// Evidence manifest output path.
        #[arg(long)]
        evidence: Option<PathBuf>,
        /// Additional workspace style source used to resolve the entry graph.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
        /// Semantic interface file associated with a bundle module.
        #[arg(long = "sif")]
        sif_paths: Vec<PathBuf>,
        /// Omena lockfile whose semantic interfaces should be loaded.
        #[arg(long)]
        lockfile: Option<PathBuf>,
    },
    /// Emit or verify typed CSS Modules interfaces.
    Modules {
        #[command(subcommand)]
        command: ModulesCommand,
    },
    /// Inspect Sass module graphs and compatibility diagnostics.
    Sass {
        #[command(subcommand)]
        command: SassCommand,
    },
    /// Query workspace style-intelligence providers.
    Intel {
        /// Workspace root. Defaults to the current directory.
        root: Option<PathBuf>,
        /// Print a machine-readable intelligence report.
        #[arg(long)]
        json: bool,
    },
    /// Plan a named source migration without applying unsafe edits.
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
    /// Verify configured product, engine, and evidence checks.
    Verify,
    /// Run the configured CI product workflow.
    Ci,
    /// Explain a diagnostic, transform decision, or retained artifact.
    Explain {
        #[command(subcommand)]
        command: ExplainCommand,
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
        /// Enable the built-in structural minify preset.
        #[arg(long)]
        minify: bool,
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
        /// Compatibility alias for `omena bundle` over the provided --source graph.
        #[arg(long)]
        bundle: bool,
        /// Emit bundle code-split CSS files into this directory.
        #[arg(long = "split-out-dir")]
        split_out_dir: Option<PathBuf>,
        /// Additional entry CSS file for bundle code-split emission.
        #[arg(long = "bundle-entry")]
        bundle_entry_paths: Vec<PathBuf>,
        /// Additional workspace style source used to derive import/composes build context.
        #[arg(long = "source")]
        source_paths: Vec<PathBuf>,
        /// package.json file used to resolve package style exports for workspace sources.
        #[arg(long = "package-manifest")]
        package_manifest_paths: Vec<PathBuf>,
        /// Include a Source Map V3 payload in --json output.
        #[arg(long)]
        source_map: bool,
        /// Compose an upstream Source Map V3 into build output maps. Use STYLE=MAP; omit STYLE for the entry file.
        #[arg(long = "input-source-map")]
        input_source_maps: Vec<String>,
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

#[derive(Debug, Subcommand)]
pub(crate) enum MigrateCommand {
    /// Rename a CSS Modules selector across style and source documents.
    CssModulesRename {
        /// Existing selector name, with or without a leading dot.
        selector_name: Option<String>,
        /// Replacement selector name, with or without a leading dot.
        new_name: Option<String>,
        /// Workspace root. Defaults to the current directory.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Restrict the rename to one CSS Module source.
        #[arg(long = "target-style")]
        target_style: Option<PathBuf>,
        #[command(flatten)]
        mode: MigrationModeArgs,
    },
    /// Replace eligible Sass imports with module-system use rules.
    SassImportToUse {
        /// Workspace root or Sass entry. Defaults to the current directory.
        #[arg(long)]
        root: Option<PathBuf>,
        #[command(flatten)]
        mode: MigrationModeArgs,
    },
    /// Rename a CSS custom property across its indexed workspace occurrences.
    TokenRename {
        /// Existing custom-property name, with or without a leading `--`.
        token_name: Option<String>,
        /// Replacement custom-property name, with or without a leading `--`.
        new_name: Option<String>,
        /// Workspace root. Defaults to the current directory.
        #[arg(long)]
        root: Option<PathBuf>,
        #[command(flatten)]
        mode: MigrationModeArgs,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum SassCommand {
    /// Inspect the resolved Sass module graph without rebuilding parser facts.
    Graph {
        /// Workspace root. Defaults to the selected module's parent or the current directory.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Restrict the view to edges rooted at one Sass-family module.
        #[arg(long)]
        module: Option<PathBuf>,
        /// Print a machine-readable graph report.
        #[arg(long)]
        json: bool,
    },
    /// Compare two SIF v1 artifacts and classify structural compatibility changes.
    Diff {
        /// Previous SIF v1 artifact.
        old: PathBuf,
        /// Candidate SIF v1 artifact.
        new: PathBuf,
        /// Print a machine-readable structural diff.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Args)]
pub(crate) struct MigrationModeArgs {
    /// Write a deterministic migration plan without changing source files.
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with = "apply",
        required_unless_present = "apply"
    )]
    pub(crate) plan: Option<PathBuf>,
    /// Apply an existing migration plan through the shared source-write gate.
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with = "plan",
        required_unless_present = "plan"
    )]
    pub(crate) apply: Option<PathBuf>,
    /// Opt into conservative edits after reviewing the plan.
    #[arg(long, requires = "apply")]
    pub(crate) approve_review: bool,
    /// Print a machine-readable migration report.
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ExplainCommand {
    /// Explain a query diagnostic and its evidence provenance.
    Diagnostic {
        /// CSS-family source that produces the diagnostic.
        path: PathBuf,
        /// Diagnostic code to explain.
        #[arg(long)]
        code: String,
        /// Print a machine-readable response envelope.
        #[arg(long)]
        json: bool,
    },
    /// Explain one transform decision from a real execution.
    Transform {
        /// CSS-family source to execute.
        path: PathBuf,
        /// Transform pass whose decision should be explained.
        #[arg(long = "pass")]
        pass_id: String,
        /// Print a machine-readable response envelope.
        #[arg(long)]
        json: bool,
    },
    /// Explain whether a symbol is retained by closed-world reachability.
    WhyNotTreeShaken {
        /// CSS-family source whose closed-world graph should be inspected.
        path: PathBuf,
        /// Kind of symbol to inspect.
        #[arg(long = "symbol-kind", value_enum)]
        symbol_kind: ExplainSymbolKind,
        /// Symbol name to inspect.
        #[arg(long)]
        symbol: String,
        /// Transform context JSON containing the authoritative reachability roots.
        #[arg(long = "context-json")]
        context_json: PathBuf,
        /// Print a machine-readable response envelope.
        #[arg(long)]
        json: bool,
    },
    /// Explain the precision assigned to a source value fact.
    Precision {
        /// JavaScript or TypeScript source containing the fact.
        path: PathBuf,
        /// Variable whose value fact should be inspected.
        #[arg(long)]
        variable: String,
        /// Byte offset of the variable reference.
        #[arg(long = "byte-offset")]
        byte_offset: usize,
        /// Optional source language id used by the source frontend.
        #[arg(long = "source-language")]
        source_language: Option<String>,
        /// Print a machine-readable response envelope.
        #[arg(long)]
        json: bool,
    },
    /// Explain cascade resolution at a source position.
    Cascade {
        /// CSS-family source to inspect.
        path: PathBuf,
        /// Zero-based line number.
        #[arg(long)]
        line: usize,
        /// Zero-based character offset.
        #[arg(long)]
        character: usize,
        /// Print a machine-readable response envelope.
        #[arg(long)]
        json: bool,
    },
    /// Explain a bundle chunk when bundle outcome evidence is available.
    Bundle {
        /// Chunk reference to inspect.
        #[arg(long)]
        chunk: String,
        /// Print a machine-readable response envelope.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ExplainSymbolKind {
    Class,
    Keyframes,
    Value,
    CustomProperty,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ModulesCommand {
    /// Emit deterministic TypeScript declarations and a module-interface manifest.
    Emit {
        /// Workspace root. Defaults to the current directory.
        root: Option<PathBuf>,
        /// Override the configured declaration output directory.
        #[arg(long = "declaration-dir")]
        declaration_dir: Option<PathBuf>,
        /// Override the configured module-interface JSON path.
        #[arg(long = "interface-file")]
        interface_file: Option<PathBuf>,
        /// Print a machine-readable operation report.
        #[arg(long)]
        json: bool,
    },
    /// Verify committed declarations and module-interface JSON byte-for-byte.
    Check {
        /// Workspace root. Defaults to the current directory.
        root: Option<PathBuf>,
        /// Override the configured declaration output directory.
        #[arg(long = "declaration-dir")]
        declaration_dir: Option<PathBuf>,
        /// Override the configured module-interface JSON path.
        #[arg(long = "interface-file")]
        interface_file: Option<PathBuf>,
        /// Print a machine-readable operation report.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum LintProfile {
    Recommended,
    Strict,
}

impl LintProfile {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Recommended => "recommended",
            Self::Strict => "strict",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum FormatMode {
    Pretty,
    Stable,
}

impl FormatMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Pretty => "pretty",
            Self::Stable => "stable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum MinifyProfile {
    Safe,
    Semantic,
    ClosedWorld,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum MinifyBackend {
    Omena,
    Lightning,
    HybridLightning,
}

impl MinifyBackend {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Omena => "omena",
            Self::Lightning => "lightning",
            Self::HybridLightning => "hybrid-lightning",
        }
    }
}

// Clap subcommand enums are parsed once per process; direct fields keep argv
// parsing simple without meaningful runtime pressure.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub(crate) enum LockCommand {
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
pub(crate) enum SifCommand {
    /// Generate a SIF v1 artifact from a Sass-family source without evaluating Sass or Less.
    Generate {
        /// CSS, SCSS, Sass, or Less source to scan.
        path: PathBuf,
        /// Stable canonical URL stored in the generated SIF. Defaults to the input path.
        #[arg(long)]
        canonical_url: Option<String>,
        /// Output path. Prints SIF JSON to stdout when omitted.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Source syntax: css, scss, sass, or less. Defaults from extension.
        #[arg(long)]
        syntax: Option<String>,
        /// Print generated SIF JSON even when --output is provided.
        #[arg(long)]
        json: bool,
    },
    /// Generate static LIF exports, including Less-specific interface facts.
    GenerateLifExports {
        /// CSS, SCSS, Sass, or Less source to scan.
        path: PathBuf,
        /// Output path. Prints LIF export JSON to stdout when omitted.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Source syntax: css, scss, sass, or less. Defaults from extension.
        #[arg(long)]
        syntax: Option<String>,
        /// Print generated LIF export JSON even when --output is provided.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProvenanceCommand {
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
pub(crate) enum ReportCommand {
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
    /// Report bounded Sass module-system conformance coverage for static analysis.
    SassModuleConformance {
        /// Print machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

#[cfg(feature = "zk-audit")]
#[derive(Debug, Subcommand)]
pub(crate) enum AuditCommand {
    /// Run zero-knowledge cascade audit commands.
    Zk {
        #[command(subcommand)]
        command: ZkAuditCommand,
    },
}

#[cfg(feature = "zk-audit")]
#[derive(Debug, Subcommand)]
pub(crate) enum ZkAuditCommand {
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

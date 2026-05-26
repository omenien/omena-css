use super::*;

const WORKSPACE_STYLE_URL_PREFIX: &str = "workspace:///";

/// Reference context for the shared omena resolver protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverReferenceContextV0 {
    /// Workspace-relative style file that contains the reference.
    pub referencing_file: String,
}

/// Canonical URL returned by `OmenaResolverV0::canonicalize`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverCanonicalUrlV0 {
    /// Stable canonical URL used as the resolver/SIF/lockfile key.
    pub url: String,
}

impl OmenaResolverCanonicalUrlV0 {
    /// Build a workspace-local canonical URL for a style source path.
    pub fn workspace_style_path(path: &str) -> Self {
        Self {
            url: format!(
                "{WORKSPACE_STYLE_URL_PREFIX}{}",
                normalize_style_path(PathBuf::from(path))
            ),
        }
    }

    /// Return the workspace-local style path when this URL uses omena's
    /// workspace scheme.
    pub fn as_workspace_style_path(&self) -> Option<&str> {
        self.url.strip_prefix(WORKSPACE_STYLE_URL_PREFIX)
    }
}

/// Successful source load result for a canonical URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverLoadedSourceV0 {
    /// Canonical URL that was loaded.
    pub canonical_url: OmenaResolverCanonicalUrlV0,
    /// Loaded UTF-8 source text.
    pub source: String,
}

/// Five-state external boundary model used by the SIF migration path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaResolverBoundaryStateKindV0 {
    Resolved,
    Partial,
    Stale,
    Missing,
    Unresolved,
}

impl OmenaResolverBoundaryStateKindV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Partial => "partial",
            Self::Stale => "stale",
            Self::Missing => "missing",
            Self::Unresolved => "unresolved",
        }
    }
}

/// Abstract value top used when a boundary is not fully resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaResolverBoundaryTopV0 {
    /// The interface is known enough to keep diagnostics scoped to declared
    /// exported symbols.
    TopOpaque,
    /// The interface is unknown and diagnostics must avoid false positives
    /// over potentially valid external symbols.
    TopAny,
}

impl OmenaResolverBoundaryTopV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TopOpaque => "topOpaque",
            Self::TopAny => "topAny",
        }
    }
}

/// Boundary-state witness for external-reference diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverBoundaryStateV0 {
    pub state: OmenaResolverBoundaryStateKindV0,
    pub state_name: &'static str,
    pub top: OmenaResolverBoundaryTopV0,
    pub top_name: &'static str,
    pub canonical_url: Option<OmenaResolverCanonicalUrlV0>,
    pub reason: String,
}

impl OmenaResolverBoundaryStateV0 {
    pub fn resolved(canonical_url: OmenaResolverCanonicalUrlV0) -> Self {
        Self::new(
            OmenaResolverBoundaryStateKindV0::Resolved,
            OmenaResolverBoundaryTopV0::TopOpaque,
            Some(canonical_url),
            "resolved local or SIF-backed interface",
        )
    }

    pub fn partial(reason: impl Into<String>) -> Self {
        Self::new(
            OmenaResolverBoundaryStateKindV0::Partial,
            OmenaResolverBoundaryTopV0::TopAny,
            None,
            reason,
        )
    }

    pub fn stale(canonical_url: OmenaResolverCanonicalUrlV0, reason: impl Into<String>) -> Self {
        Self::new(
            OmenaResolverBoundaryStateKindV0::Stale,
            OmenaResolverBoundaryTopV0::TopAny,
            Some(canonical_url),
            reason,
        )
    }

    pub fn missing(
        canonical_url: Option<OmenaResolverCanonicalUrlV0>,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(
            OmenaResolverBoundaryStateKindV0::Missing,
            OmenaResolverBoundaryTopV0::TopAny,
            canonical_url,
            reason,
        )
    }

    pub fn unresolved(reason: impl Into<String>) -> Self {
        Self::new(
            OmenaResolverBoundaryStateKindV0::Unresolved,
            OmenaResolverBoundaryTopV0::TopAny,
            None,
            reason,
        )
    }

    fn new(
        state: OmenaResolverBoundaryStateKindV0,
        top: OmenaResolverBoundaryTopV0,
        canonical_url: Option<OmenaResolverCanonicalUrlV0>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            state,
            state_name: state.as_str(),
            top,
            top_name: top.as_str(),
            canonical_url,
            reason: reason.into(),
        }
    }
}

pub fn omena_resolver_boundary_state_from_error_v0(
    error: &OmenaResolverErrorV0,
) -> OmenaResolverBoundaryStateV0 {
    match error.kind {
        OmenaResolverErrorKindV0::ExternalIgnored => {
            OmenaResolverBoundaryStateV0::partial(error.message.clone())
        }
        OmenaResolverErrorKindV0::NotFound => {
            OmenaResolverBoundaryStateV0::missing(None, error.message.clone())
        }
        OmenaResolverErrorKindV0::Unresolved
        | OmenaResolverErrorKindV0::NetworkForbidden
        | OmenaResolverErrorKindV0::UnsupportedCanonicalUrl => {
            OmenaResolverBoundaryStateV0::unresolved(error.message.clone())
        }
    }
}

/// Error family for the shared resolver protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaResolverErrorKindV0 {
    /// The reference is not resolvable from the current workspace snapshot.
    Unresolved,
    /// The reference is intentionally left at the existing external boundary.
    ExternalIgnored,
    /// Network references are never fetched by omena's resolver protocol.
    NetworkForbidden,
    /// The canonical URL is not loadable by this resolver implementation.
    UnsupportedCanonicalUrl,
    /// The canonical URL is valid but no local source is available.
    NotFound,
}

impl OmenaResolverErrorKindV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unresolved => "unresolved",
            Self::ExternalIgnored => "externalIgnored",
            Self::NetworkForbidden => "networkForbidden",
            Self::UnsupportedCanonicalUrl => "unsupportedCanonicalUrl",
            Self::NotFound => "notFound",
        }
    }
}

/// Resolver protocol error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverErrorV0 {
    pub kind: OmenaResolverErrorKindV0,
    pub kind_name: &'static str,
    pub message: String,
}

impl OmenaResolverErrorV0 {
    pub fn new(kind: OmenaResolverErrorKindV0, message: impl Into<String>) -> Self {
        Self {
            kind,
            kind_name: kind.as_str(),
            message: message.into(),
        }
    }
}

/// Shared resolver protocol for CLI, LSP, fixture, and query paths.
///
/// `canonicalize` must be deterministic over an immutable workspace snapshot
/// and must not perform filesystem or network I/O. `load` may be implemented
/// by local-disk-backed resolvers, but it must never fetch from the network.
pub trait OmenaResolverV0 {
    fn canonicalize(
        &self,
        context: &OmenaResolverReferenceContextV0,
        raw_reference: &str,
    ) -> Result<OmenaResolverCanonicalUrlV0, OmenaResolverErrorV0>;

    fn load(
        &self,
        canonical_url: &OmenaResolverCanonicalUrlV0,
    ) -> Result<OmenaResolverLoadedSourceV0, OmenaResolverErrorV0>;
}

/// Snapshot-backed resolver that adapts today's style module resolver to the
/// RFC 0004 protocol without adding I/O to canonicalization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OmenaResolverStyleModuleSnapshotV0 {
    pub available_style_paths: BTreeSet<String>,
    pub file_sources: BTreeMap<String, String>,
    pub package_manifests: Vec<OmenaResolverStylePackageManifestV0>,
    pub bundler_path_mappings: Vec<OmenaResolverBundlerPathAliasMappingV0>,
    pub tsconfig_path_mappings: Vec<OmenaResolverTsconfigPathMappingV0>,
}

impl OmenaResolverStyleModuleSnapshotV0 {
    pub fn new<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            available_style_paths: paths.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    pub fn with_file_source(mut self, path: impl Into<String>, source: impl Into<String>) -> Self {
        self.file_sources.insert(path.into(), source.into());
        self
    }

    pub fn with_package_manifests(
        mut self,
        manifests: Vec<OmenaResolverStylePackageManifestV0>,
    ) -> Self {
        self.package_manifests = manifests;
        self
    }

    pub fn with_bundler_path_mappings(
        mut self,
        mappings: Vec<OmenaResolverBundlerPathAliasMappingV0>,
    ) -> Self {
        self.bundler_path_mappings = mappings;
        self
    }

    pub fn with_tsconfig_path_mappings(
        mut self,
        mappings: Vec<OmenaResolverTsconfigPathMappingV0>,
    ) -> Self {
        self.tsconfig_path_mappings = mappings;
        self
    }

    fn available_style_path_refs(&self) -> BTreeSet<&str> {
        self.available_style_paths
            .iter()
            .map(String::as_str)
            .collect()
    }
}

impl OmenaResolverV0 for OmenaResolverStyleModuleSnapshotV0 {
    fn canonicalize(
        &self,
        context: &OmenaResolverReferenceContextV0,
        raw_reference: &str,
    ) -> Result<OmenaResolverCanonicalUrlV0, OmenaResolverErrorV0> {
        if raw_reference.starts_with("http://") || raw_reference.starts_with("https://") {
            return Err(OmenaResolverErrorV0::new(
                OmenaResolverErrorKindV0::NetworkForbidden,
                "omena resolver canonicalization never fetches network references",
            ));
        }

        let available_style_paths = self.available_style_path_refs();
        let resolution = summarize_omena_resolver_style_module_resolution_with_path_mappings(
            &context.referencing_file,
            raw_reference,
            &available_style_paths,
            self.package_manifests.as_slice(),
            self.bundler_path_mappings.as_slice(),
            self.tsconfig_path_mappings.as_slice(),
        );

        if let Some(path) = resolution.resolved_style_path {
            return Ok(OmenaResolverCanonicalUrlV0::workspace_style_path(&path));
        }

        let kind = if resolution.resolution_kind == "externalIgnored" {
            OmenaResolverErrorKindV0::ExternalIgnored
        } else {
            OmenaResolverErrorKindV0::Unresolved
        };
        Err(OmenaResolverErrorV0::new(
            kind,
            format!(
                "could not canonicalize `{raw_reference}` from `{}`",
                context.referencing_file
            ),
        ))
    }

    fn load(
        &self,
        canonical_url: &OmenaResolverCanonicalUrlV0,
    ) -> Result<OmenaResolverLoadedSourceV0, OmenaResolverErrorV0> {
        let Some(path) = canonical_url.as_workspace_style_path() else {
            return Err(OmenaResolverErrorV0::new(
                OmenaResolverErrorKindV0::UnsupportedCanonicalUrl,
                format!("unsupported canonical URL `{}`", canonical_url.url),
            ));
        };
        let Some(source) = self.file_sources.get(path) else {
            return Err(OmenaResolverErrorV0::new(
                OmenaResolverErrorKindV0::NotFound,
                format!("no source snapshot for `{path}`"),
            ));
        };
        Ok(OmenaResolverLoadedSourceV0 {
            canonical_url: canonical_url.clone(),
            source: source.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_resolver_canonicalizes_and_loads_relative_style_modules() -> Result<(), String> {
        let resolver = OmenaResolverStyleModuleSnapshotV0::new(["src/Button.module.scss"])
            .with_file_source("src/Button.module.scss", ".button { color: red; }");
        let context = OmenaResolverReferenceContextV0 {
            referencing_file: "src/App.module.scss".to_string(),
        };

        let canonical = resolver
            .canonicalize(&context, "./Button.module.scss")
            .map_err(|error| format!("expected canonical style URL: {error:?}"))?;
        assert_eq!(canonical.url, "workspace:///src/Button.module.scss");

        let loaded = resolver
            .load(&canonical)
            .map_err(|error| format!("expected loaded style source: {error:?}"))?;
        assert_eq!(loaded.source, ".button { color: red; }");
        Ok(())
    }

    #[test]
    fn snapshot_resolver_forbids_network_references_during_canonicalization() -> Result<(), String>
    {
        let resolver = OmenaResolverStyleModuleSnapshotV0::new(["src/Button.module.scss"]);
        let context = OmenaResolverReferenceContextV0 {
            referencing_file: "src/App.module.scss".to_string(),
        };

        let error = match resolver.canonicalize(&context, "https://example.com/reset.css") {
            Ok(canonical) => {
                return Err(format!(
                    "expected network reference to fail, got {canonical:?}"
                ));
            }
            Err(error) => error,
        };

        assert_eq!(error.kind, OmenaResolverErrorKindV0::NetworkForbidden);
        assert_eq!(error.kind_name, "networkForbidden");
        Ok(())
    }

    #[test]
    fn snapshot_resolver_reports_missing_snapshot_sources() -> Result<(), String> {
        let resolver = OmenaResolverStyleModuleSnapshotV0::new(["src/Button.module.scss"]);
        let context = OmenaResolverReferenceContextV0 {
            referencing_file: "src/App.module.scss".to_string(),
        };

        let canonical = resolver
            .canonicalize(&context, "./Button.module.scss")
            .map_err(|error| format!("expected canonical style URL: {error:?}"))?;
        let error = match resolver.load(&canonical) {
            Ok(source) => return Err(format!("expected missing snapshot source, got {source:?}")),
            Err(error) => error,
        };

        assert_eq!(error.kind, OmenaResolverErrorKindV0::NotFound);
        Ok(())
    }

    #[test]
    fn boundary_state_matrix_preserves_m7_external_states_and_top_semantics() {
        let canonical = OmenaResolverCanonicalUrlV0::workspace_style_path("src/tokens.scss");
        let states = [
            OmenaResolverBoundaryStateV0::resolved(canonical.clone()),
            OmenaResolverBoundaryStateV0::partial("external boundary kept in ignored mode"),
            OmenaResolverBoundaryStateV0::stale(canonical.clone(), "lockfile hash drift"),
            OmenaResolverBoundaryStateV0::missing(Some(canonical), "expected SIF is missing"),
            OmenaResolverBoundaryStateV0::unresolved("specifier did not resolve"),
        ];

        assert_eq!(states[0].state_name, "resolved");
        assert_eq!(states[0].top_name, "topOpaque");
        assert_eq!(states[1].state_name, "partial");
        assert_eq!(states[1].top_name, "topAny");
        assert_eq!(states[2].state_name, "stale");
        assert_eq!(states[2].top_name, "topAny");
        assert_eq!(states[3].state_name, "missing");
        assert_eq!(states[3].top_name, "topAny");
        assert_eq!(states[4].state_name, "unresolved");
        assert_eq!(states[4].top_name, "topAny");
    }

    #[test]
    fn boundary_state_maps_existing_external_ignored_error_to_partial() {
        let error = OmenaResolverErrorV0::new(
            OmenaResolverErrorKindV0::ExternalIgnored,
            "sass:map remains external in compatibility mode",
        );

        let state = omena_resolver_boundary_state_from_error_v0(&error);

        assert_eq!(state.state, OmenaResolverBoundaryStateKindV0::Partial);
        assert_eq!(state.top, OmenaResolverBoundaryTopV0::TopAny);
        assert_eq!(
            state.reason,
            "sass:map remains external in compatibility mode"
        );
    }

    #[test]
    fn snapshot_resolver_preserves_tsconfig_path_mapping_resolution() -> Result<(), String> {
        let resolver = OmenaResolverStyleModuleSnapshotV0::new([
            "/fake/workspace/src/styles/Button.module.scss",
        ])
        .with_tsconfig_path_mappings(vec![OmenaResolverTsconfigPathMappingV0 {
            base_path: "/fake/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/styles/*".to_string()],
        }]);
        let context = OmenaResolverReferenceContextV0 {
            referencing_file: "/fake/workspace/src/App.module.scss".to_string(),
        };

        let canonical = resolver
            .canonicalize(&context, "@styles/Button")
            .map_err(|error| format!("expected canonical style URL: {error:?}"))?;

        assert_eq!(
            canonical.as_workspace_style_path(),
            Some("/fake/workspace/src/styles/Button.module.scss")
        );
        Ok(())
    }

    #[test]
    fn snapshot_resolver_preserves_bundler_path_mapping_precedence() -> Result<(), String> {
        let resolver = OmenaResolverStyleModuleSnapshotV0::new([
            "/fake/workspace/src/bundler/Button.module.scss",
            "/fake/workspace/src/tsconfig/Button.module.scss",
        ])
        .with_bundler_path_mappings(vec![OmenaResolverBundlerPathAliasMappingV0 {
            pattern: "@styles".to_string(),
            target_path: "/fake/workspace/src/bundler".to_string(),
        }])
        .with_tsconfig_path_mappings(vec![OmenaResolverTsconfigPathMappingV0 {
            base_path: "/fake/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/tsconfig/*".to_string()],
        }]);
        let context = OmenaResolverReferenceContextV0 {
            referencing_file: "/fake/workspace/src/App.module.scss".to_string(),
        };

        let canonical = resolver
            .canonicalize(&context, "@styles/Button")
            .map_err(|error| format!("expected canonical style URL: {error:?}"))?;

        assert_eq!(
            canonical.as_workspace_style_path(),
            Some("/fake/workspace/src/bundler/Button.module.scss")
        );
        Ok(())
    }

    #[test]
    fn snapshot_resolver_preserves_package_manifest_resolution() -> Result<(), String> {
        let resolver = OmenaResolverStyleModuleSnapshotV0::new([
            "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
        ])
        .with_package_manifests(vec![OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                .to_string(),
        }]);
        let context = OmenaResolverReferenceContextV0 {
            referencing_file: "/fake/workspace/src/App.module.scss".to_string(),
        };

        let canonical = resolver
            .canonicalize(&context, "@design/tokens/theme")
            .map_err(|error| format!("expected canonical style URL: {error:?}"))?;

        assert_eq!(
            canonical.as_workspace_style_path(),
            Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
        );
        Ok(())
    }
}

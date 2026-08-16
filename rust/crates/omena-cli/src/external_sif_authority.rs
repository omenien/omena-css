use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use omena_query::{
    OmenaQueryExternalSifInputV0, OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    resolve_omena_query_bridge_external_sifs_for_style_sources,
};
use omena_sif::{
    compute_omena_sif_artifact_hash_v1, read_omena_lock_json_v1, read_omena_sif_json_v1,
};

use crate::{io::read_source, lock::resolve_lock_relative_path, paths::path_string};

/// The only CLI input shape that can request external-SIF artifact reads.
///
/// Paths enter this type at an explicit command boundary. Automated product
/// paths use [`Self::none`], so workspace discovery cannot silently become a
/// lock authority.
#[derive(Debug, Clone, Default)]
pub(crate) struct CliExternalSifSelectionV0 {
    explicit_sif_paths: Vec<PathBuf>,
    explicit_lockfile: Option<PathBuf>,
}

impl CliExternalSifSelectionV0 {
    pub(crate) fn from_explicit_cli_arguments(
        explicit_sif_paths: Vec<PathBuf>,
        explicit_lockfile: Option<PathBuf>,
    ) -> Self {
        Self {
            explicit_sif_paths,
            explicit_lockfile,
        }
    }

    pub(crate) const fn none() -> Self {
        Self {
            explicit_sif_paths: Vec::new(),
            explicit_lockfile: None,
        }
    }

    pub(crate) fn has_explicit_sif_paths(&self) -> bool {
        !self.explicit_sif_paths.is_empty()
    }

    pub(crate) fn without_lock_fallback(mut self) -> Self {
        self.explicit_lockfile = None;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliExternalSifLockFailureModeV0 {
    Refuse,
    SurfaceDiagnostic,
}

#[derive(Debug, Clone)]
pub(crate) struct CliExternalSifLockErrorV0 {
    lockfile: PathBuf,
    message: String,
}

impl CliExternalSifLockErrorV0 {
    pub(crate) fn lockfile(&self) -> &Path {
        self.lockfile.as_path()
    }

    pub(crate) fn message(&self) -> &str {
        self.message.as_str()
    }
}

/// Opaque output from the sole CLI external-SIF authority boundary.
///
/// Callers receive only the already-reconciled sequence. They cannot append a
/// lock result ahead of local regeneration because the lock reader and the
/// owned vector both remain private to this module.
#[derive(Debug, Clone)]
pub(crate) struct CliExternalSifAuthorityResolutionV0 {
    external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    lock_error: Option<CliExternalSifLockErrorV0>,
}

impl CliExternalSifAuthorityResolutionV0 {
    pub(crate) fn external_sifs(&self) -> &[OmenaQueryExternalSifInputV0] {
        self.external_sifs.as_slice()
    }

    pub(crate) fn lock_error(&self) -> Option<&CliExternalSifLockErrorV0> {
        self.lock_error.as_ref()
    }
}

/// Resolve every CLI external-SIF source with one fixed authority order:
/// explicit SIF artifacts, locally regenerated bridge artifacts, then the
/// explicitly selected lock as a fallback for still-uncovered URLs.
pub(crate) fn resolve_cli_external_sif_authority(
    selection: &CliExternalSifSelectionV0,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    lock_failure_mode: CliExternalSifLockFailureModeV0,
) -> Result<CliExternalSifAuthorityResolutionV0, String> {
    let explicit_sifs = read_explicit_external_sifs(selection.explicit_sif_paths.as_slice())?;
    let locally_regenerated_sifs = resolve_omena_query_bridge_external_sifs_for_style_sources(
        workspace_sources,
        explicit_sifs.as_slice(),
        resolution_inputs,
    )
    .external_sifs;

    let (lock_sifs, lock_error) = match selection.explicit_lockfile.as_deref() {
        Some(lockfile) => match read_explicit_lock_external_sifs(lockfile) {
            Ok(lock_sifs) => (lock_sifs, None),
            Err(message) if lock_failure_mode == CliExternalSifLockFailureModeV0::Refuse => {
                return Err(message);
            }
            Err(message) => (
                Vec::new(),
                Some(CliExternalSifLockErrorV0 {
                    lockfile: lockfile.to_path_buf(),
                    message,
                }),
            ),
        },
        None => (Vec::new(), None),
    };

    let mut external_sifs = Vec::new();
    let mut covered = BTreeSet::new();
    for candidates in [explicit_sifs, locally_regenerated_sifs, lock_sifs] {
        for candidate in candidates {
            let outer_url = candidate.canonical_url.clone();
            let resolved_url = candidate.sif.canonical_url.clone();
            if covered.contains(outer_url.as_str()) || covered.contains(resolved_url.as_str()) {
                continue;
            }
            covered.insert(outer_url);
            covered.insert(resolved_url);
            external_sifs.push(candidate);
        }
    }
    Ok(CliExternalSifAuthorityResolutionV0 {
        external_sifs,
        lock_error,
    })
}

fn read_explicit_external_sifs(
    paths: &[PathBuf],
) -> Result<Vec<OmenaQueryExternalSifInputV0>, String> {
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

fn read_explicit_lock_external_sifs(
    lockfile: &Path,
) -> Result<Vec<OmenaQueryExternalSifInputV0>, String> {
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
            let actual_sif_hash = compute_omena_sif_artifact_hash_v1(&sif).map_err(|error| {
                format!("failed to hash SIF {}: {error}", path_string(&sif_path))
            })?;
            if actual_sif_hash != entry.sif_hash {
                return Err(format!(
                    "lock entry {} expected sifHash {}, but SIF {} hashes to {}",
                    entry.canonical_url,
                    entry.sif_hash.as_str(),
                    path_string(&sif_path),
                    actual_sif_hash.as_str()
                ));
            }
            Ok(OmenaQueryExternalSifInputV0 {
                canonical_url: entry.canonical_url.clone(),
                sif,
            })
        })
        .collect()
}

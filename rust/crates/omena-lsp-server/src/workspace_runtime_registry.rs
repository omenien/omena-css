use crate::{LspWorkspaceFolderState, file_uri_to_path, normalize_path};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRuntimeRegistryBoundaryV0 {
    pub product: &'static str,
    pub owner: &'static str,
    pub folder_state_owner: &'static str,
    pub ownership_policy: Vec<&'static str>,
    pub indexed_document_policy: Vec<&'static str>,
    pub request_path_policy: Vec<&'static str>,
}

pub fn workspace_runtime_registry_contract() -> WorkspaceRuntimeRegistryBoundaryV0 {
    WorkspaceRuntimeRegistryBoundaryV0 {
        product: "omena-lsp-server.workspace-runtime-registry",
        owner: "omena-lsp-server/runtime/workspaceRuntimeRegistry",
        folder_state_owner: "omena-lsp-server",
        ownership_policy: vec![
            "longestWorkspaceRootOwnsDocument",
            "filePathComponentBoundariesBeforeUriPrefix",
            "workspaceFolderChangesRefreshDocumentOwnership",
        ],
        indexed_document_policy: vec![
            "indexStyleDocumentsPerWorkspaceRoot",
            "indexSourceDocumentsPerWorkspaceRoot",
            "backgroundWorkspaceIndexAppliesOnLoopThread",
            "evictIndexedDocumentsOnWorkspaceRemoval",
            "retargetIndexedDocumentsToRemainingOwnerOnWorkspaceRemoval",
            "openedDocumentsRemainAuthoritative",
        ],
        request_path_policy: vec![
            "noNodeWorkspaceRuntimeManagerOnRustLspPath",
            "resolveWorkspaceOwnershipBeforeProviderExecution",
            "keepWorkspaceOwnershipDeterministic",
        ],
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorkspaceRuntimeRegistry {
    folders: BTreeMap<String, WorkspaceRuntimeFolderEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkspaceRuntimeFolderEntry {
    folder: LspWorkspaceFolderState,
    root_path: Option<PathBuf>,
}

impl WorkspaceRuntimeRegistry {
    pub(crate) fn clear(&mut self) {
        self.folders.clear();
    }

    pub(crate) fn insert(&mut self, uri: impl Into<String>, name: impl Into<String>) {
        let uri = uri.into();
        let root_path = file_uri_to_path(uri.as_str()).map(normalize_path);
        self.folders.insert(
            uri.clone(),
            WorkspaceRuntimeFolderEntry {
                folder: LspWorkspaceFolderState {
                    uri,
                    name: name.into(),
                },
                root_path,
            },
        );
    }

    pub(crate) fn remove(&mut self, uri: &str) -> Option<LspWorkspaceFolderState> {
        self.folders.remove(uri).map(|entry| entry.folder)
    }

    pub(crate) fn get(&self, uri: &str) -> Option<&LspWorkspaceFolderState> {
        self.folders.get(uri).map(|entry| &entry.folder)
    }

    pub(crate) fn len(&self) -> usize {
        self.folders.len()
    }

    pub(crate) fn folders(&self) -> impl Iterator<Item = &LspWorkspaceFolderState> {
        self.folders.values().map(|entry| &entry.folder)
    }

    pub(crate) fn folder_snapshots(&self) -> Vec<LspWorkspaceFolderState> {
        self.folders().cloned().collect()
    }

    pub(crate) fn resolve_owner_uri(&self, document_uri: &str) -> Option<String> {
        let document_path = file_uri_to_path(document_uri).map(normalize_path);
        self.folders
            .values()
            .filter_map(|entry| {
                workspace_owner_score(entry, document_uri, document_path.as_deref())
                    .map(|score| (score, entry.folder.uri.clone()))
            })
            .max_by_key(|(score, _)| *score)
            .map(|(_, uri)| uri)
    }
}

fn workspace_owner_score(
    entry: &WorkspaceRuntimeFolderEntry,
    document_uri: &str,
    document_path: Option<&Path>,
) -> Option<(u8, usize, usize)> {
    if let (Some(root_path), Some(document_path)) = (entry.root_path.as_deref(), document_path)
        && path_is_equal_or_descendant(document_path, root_path)
    {
        return Some((1, path_depth(root_path), entry.folder.uri.len()));
    }

    if uri_is_equal_or_descendant(entry.folder.uri.as_str(), document_uri) {
        return Some((0, 0, entry.folder.uri.len()));
    }

    None
}

fn path_is_equal_or_descendant(document_path: &Path, workspace_root: &Path) -> bool {
    !workspace_root.as_os_str().is_empty()
        && (document_path == workspace_root || document_path.starts_with(workspace_root))
}

fn path_depth(path: &Path) -> usize {
    path.components().count()
}

fn uri_is_equal_or_descendant(workspace_uri: &str, document_uri: &str) -> bool {
    document_uri == workspace_uri
        || document_uri
            .strip_prefix(workspace_uri)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_owner_by_longest_workspace_root() {
        let mut registry = WorkspaceRuntimeRegistry::default();
        registry.insert("file:///repo", "repo");
        registry.insert("file:///repo/packages/app", "app");

        assert_eq!(
            registry.resolve_owner_uri("file:///repo/packages/app/src/Button.module.scss"),
            Some("file:///repo/packages/app".to_string()),
        );
    }

    #[test]
    fn path_boundaries_prevent_prefix_only_ownership() {
        let mut registry = WorkspaceRuntimeRegistry::default();
        registry.insert("file:///repo/app", "app");

        assert_eq!(
            registry.resolve_owner_uri("file:///repo/app2/src/Button.module.scss"),
            None,
        );
    }
}

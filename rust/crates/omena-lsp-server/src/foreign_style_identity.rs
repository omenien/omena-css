use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use omena_sif::compute_omena_sif_leaf_hash_v1;
use serde_json::Value;

use crate::{
    LspQueryReadView, file_uri_to_path, is_style_document_uri, state::LspStyleHoverCandidate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ForeignSassPackageIdentity {
    pub(crate) package_name: String,
    pub(crate) version: String,
    pub(crate) subpath: String,
}

pub(crate) fn foreign_sass_package_identity_for_uri(
    state: &dyn LspQueryReadView,
    uri: &str,
) -> Option<ForeignSassPackageIdentity> {
    let path = file_uri_to_path(uri)?;
    let (package_name, package_root, subpath) = node_modules_package_for_path(path.as_path())?;
    let version = package_version_for_root(package_root.as_path()).unwrap_or_else(|| {
        state
            .document(uri)
            .map(|document| format!("leaf:{}", document.text_hash))
            .or_else(|| {
                fs::read(path.as_path()).ok().map(|bytes| {
                    format!(
                        "leaf:{}",
                        compute_omena_sif_leaf_hash_v1(bytes.as_slice()).as_str()
                    )
                })
            })
            .unwrap_or_else(|| "leaf:unknown".to_string())
    });
    Some(ForeignSassPackageIdentity {
        package_name,
        version,
        subpath,
    })
}

pub(crate) fn is_foreign_style_document_uri(uri: &str) -> bool {
    is_style_document_uri(uri)
        && file_uri_to_path(uri)
            .as_deref()
            .and_then(node_modules_package_for_path)
            .is_some()
}

pub(crate) fn node_modules_package_for_path(path: &Path) -> Option<(String, PathBuf, String)> {
    let components = path.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        if !matches!(component, Component::Normal(name) if name.to_str() == Some("node_modules")) {
            continue;
        }
        let package_start = index + 1;
        let first = component_normal_str(components.get(package_start)?)?;
        let (package_name, package_end) = if first.starts_with('@') {
            let second = component_normal_str(components.get(package_start + 1)?)?;
            (format!("{first}/{second}"), package_start + 2)
        } else {
            (first.to_string(), package_start + 1)
        };
        let package_root =
            components[..package_end]
                .iter()
                .fold(PathBuf::new(), |mut root, component| {
                    root.push(component.as_os_str());
                    root
                });
        let subpath = components[package_end..]
            .iter()
            .filter_map(component_normal_str)
            .collect::<Vec<_>>()
            .join("/");
        return Some((
            package_name,
            package_root,
            if subpath.is_empty() {
                ".".to_string()
            } else {
                subpath
            },
        ));
    }
    None
}

pub(crate) fn style_foreign_sass_symbol_moniker(
    state: &dyn LspQueryReadView,
    uri: &str,
    candidate: &LspStyleHoverCandidate,
) -> Option<String> {
    let identity = foreign_sass_package_identity_for_uri(state, uri)?;
    let family = crate::sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol");
    Some(format!(
        "sass-symbol-foreign:pkg:{}@{}/{}#{}:{}",
        identity.package_name, identity.version, identity.subpath, family, candidate.name
    ))
}

fn package_version_for_root(package_root: &Path) -> Option<String> {
    let source = fs::read_to_string(package_root.join("package.json")).ok()?;
    serde_json::from_str::<Value>(source.as_str())
        .ok()
        .and_then(|json| {
            json.get("version")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .filter(|version| !version.is_empty())
}

fn component_normal_str<'a>(component: &'a Component<'a>) -> Option<&'a str> {
    match component {
        Component::Normal(value) => value.to_str(),
        _ => None,
    }
}

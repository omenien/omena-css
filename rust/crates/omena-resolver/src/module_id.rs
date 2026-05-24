use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleCanonicalIdV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub canonical_id: String,
    pub source_uri: String,
    pub package_name: Option<String>,
    pub export_name: Option<String>,
    pub layer_marker: &'static str,
}

pub fn canonicalize_module_id_v0(source_uri: impl Into<String>) -> ModuleCanonicalIdV0 {
    let source_uri = source_uri.into();
    ModuleCanonicalIdV0 {
        schema_version: "0",
        product: "omena-resolver.module-canonical-id",
        canonical_id: source_uri.clone(),
        source_uri,
        package_name: None,
        export_name: None,
        layer_marker: "frame-rule",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_canonical_id_uses_frame_rule_layer_marker() {
        let module = canonicalize_module_id_v0("file:///workspace/a.module.css");

        assert_eq!(module.schema_version, "0");
        assert_eq!(module.product, "omena-resolver.module-canonical-id");
        assert_eq!(module.layer_marker, "frame-rule");
        assert_eq!(module.canonical_id, module.source_uri);
    }
}

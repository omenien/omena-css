use omena_cascade::{DiagnosticFrameFootprintV0, derive_frame_for_diagnostic};
use omena_resolver::ModuleCanonicalIdV0;
use serde::Serialize;

use crate::OmenaCheckerRuleCodeV0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAwareDiagnosticV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub diagnostic_code: OmenaCheckerRuleCodeV0,
    pub diagnostic_code_name: &'static str,
    pub diagnostic_instance_id: String,
    pub frame: DiagnosticFrameFootprintV0,
    pub layer_marker: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAwareDiagnosticSetV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub diagnostic_count: usize,
    pub diagnostics: Vec<FrameAwareDiagnosticV0>,
    pub conservative: bool,
}

pub fn emit_frame_aware_diagnostic(
    code: OmenaCheckerRuleCodeV0,
    diagnostic_instance_id: impl Into<String>,
    modules: &[ModuleCanonicalIdV0],
) -> FrameAwareDiagnosticV0 {
    let diagnostic_instance_id = diagnostic_instance_id.into();
    let module_ids = modules
        .iter()
        .map(|module| module.canonical_id.clone())
        .collect::<Vec<_>>();
    let frame =
        derive_frame_for_diagnostic(code.as_str(), diagnostic_instance_id.clone(), module_ids);

    FrameAwareDiagnosticV0 {
        schema_version: "0",
        product: "omena-checker.frame-aware-diagnostic",
        diagnostic_code: code,
        diagnostic_code_name: code.as_str(),
        diagnostic_instance_id,
        frame,
        layer_marker: "frame-rule",
    }
}

pub fn emit_frame_aware_diagnostic_set(
    diagnostics: Vec<FrameAwareDiagnosticV0>,
) -> FrameAwareDiagnosticSetV0 {
    FrameAwareDiagnosticSetV0 {
        schema_version: "0",
        product: "omena-checker.frame-aware-diagnostic-set",
        diagnostic_count: diagnostics.len(),
        diagnostics,
        conservative: true,
    }
}

#[cfg(test)]
mod tests {
    use omena_resolver::canonicalize_module_id_v0;

    use super::*;

    #[test]
    fn emits_frame_aware_diagnostic_from_canonical_module_ids() {
        let module = canonicalize_module_id_v0("file:///workspace/a.module.css");
        let diagnostic = emit_frame_aware_diagnostic(
            OmenaCheckerRuleCodeV0::MissingStaticClass,
            "d1",
            &[module],
        );

        assert_eq!(diagnostic.schema_version, "0");
        assert_eq!(diagnostic.diagnostic_code_name, "missing-static-class");
        assert_eq!(diagnostic.frame.layer_marker, "frame-rule");
        assert!(diagnostic.frame.conservative);
    }
}

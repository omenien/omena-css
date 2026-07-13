use omena_evidence_graph::EvidenceNodeSeedV0;
#[cfg(any(test, feature = "lightning-lowering"))]
use omena_evidence_graph::{
    EvidenceNodeKeyV0, ExternalToolRunWitnessV0, FamilyStampV0, GuaranteeKindV0,
};
#[cfg(any(test, feature = "lightning-lowering"))]
use omena_query::{
    OmenaQueryTransformStyleDialect, compare_omena_query_transform_css_semantics_v0,
};
use serde::Serialize;

#[cfg(any(test, feature = "lightning-lowering"))]
use crate::lock::sha256_hex;

#[cfg(any(test, feature = "lightning-lowering"))]
const LIGHTNINGCSS_CRATE_VERSION: &str = "1.0.0-alpha.71";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MinifySemanticCheckV0 {
    pub preserved: bool,
    pub input_entry_count: usize,
    pub output_entry_count: usize,
    pub mismatch_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MinifyDelegationReportV0 {
    pub tool_name: &'static str,
    pub tool_version: &'static str,
    pub input_digest: String,
    pub exit_status: i32,
    pub evidence: EvidenceNodeSeedV0,
    pub candidate_byte_len: Option<usize>,
    pub semantic_check: Option<MinifySemanticCheckV0>,
    pub adopted: bool,
    pub fallback_reason: Option<String>,
}

pub(crate) struct MinifyDelegationOutcomeV0 {
    pub output_css: String,
    pub report: MinifyDelegationReportV0,
}

#[cfg(feature = "lightning-lowering")]
pub(crate) fn run_hybrid_lightning_lowering(
    semantic_output_css: &str,
) -> Result<MinifyDelegationOutcomeV0, String> {
    run_external_lowering(semantic_output_css, lightning_lower_css)
}

#[cfg(not(feature = "lightning-lowering"))]
pub(crate) fn run_hybrid_lightning_lowering(
    _semantic_output_css: &str,
) -> Result<MinifyDelegationOutcomeV0, String> {
    Err(
        "the hybrid-lightning backend requires an omena-cli build with the `lightning-lowering` feature"
            .to_string(),
    )
}

#[cfg(any(test, feature = "lightning-lowering"))]
fn run_external_lowering(
    semantic_output_css: &str,
    lower: impl FnOnce(&str) -> Result<String, String>,
) -> Result<MinifyDelegationOutcomeV0, String> {
    let input_digest = sha256_hex(semantic_output_css.as_bytes());
    let lowering = lower(semantic_output_css);
    let exit_status = i32::from(lowering.is_err());
    let witness = ExternalToolRunWitnessV0 {
        tool_name: "lightningcss".to_string(),
        tool_version: LIGHTNINGCSS_CRATE_VERSION.to_string(),
        input_digest: input_digest.clone(),
        exit_status,
    };
    let evidence = EvidenceNodeSeedV0::with_family(
        EvidenceNodeKeyV0::new("omena-cli.minify.external-lowering", input_digest.clone()),
        vec![
            "externalTool:lightningcss".to_string(),
            format!("toolVersion:{LIGHTNINGCSS_CRATE_VERSION}"),
            format!("exitStatus:{exit_status}"),
        ],
        GuaranteeKindV0::for_label_less_family(),
        FamilyStampV0::external_tool(&witness),
    );

    match lowering {
        Ok(candidate) => {
            let decision = compare_omena_query_transform_css_semantics_v0(
                semantic_output_css,
                candidate.as_str(),
                OmenaQueryTransformStyleDialect::Css,
            );
            let semantic_check = MinifySemanticCheckV0 {
                preserved: decision.preserved,
                input_entry_count: decision.input_entry_count,
                output_entry_count: decision.output_entry_count,
                mismatch_count: decision.mismatch_count,
            };
            let adopted = semantic_check.preserved;
            Ok(MinifyDelegationOutcomeV0 {
                output_css: if adopted {
                    candidate.clone()
                } else {
                    semantic_output_css.to_string()
                },
                report: MinifyDelegationReportV0 {
                    tool_name: "lightningcss",
                    tool_version: LIGHTNINGCSS_CRATE_VERSION,
                    input_digest,
                    exit_status,
                    evidence,
                    candidate_byte_len: Some(candidate.len()),
                    semantic_check: Some(semantic_check),
                    adopted,
                    fallback_reason: (!adopted)
                        .then(|| "semantic observation mismatch".to_string()),
                },
            })
        }
        Err(error) => Ok(MinifyDelegationOutcomeV0 {
            output_css: semantic_output_css.to_string(),
            report: MinifyDelegationReportV0 {
                tool_name: "lightningcss",
                tool_version: LIGHTNINGCSS_CRATE_VERSION,
                input_digest,
                exit_status,
                evidence,
                candidate_byte_len: None,
                semantic_check: None,
                adopted: false,
                fallback_reason: Some(format!("external lowering failed: {error}")),
            },
        }),
    }
}

#[cfg(feature = "lightning-lowering")]
fn lightning_lower_css(source: &str) -> Result<String, String> {
    use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};

    let mut stylesheet = StyleSheet::parse(source, ParserOptions::default())
        .map_err(|error| format!("lightningcss parse failed: {error}"))?;
    stylesheet
        .minify(MinifyOptions::default())
        .map_err(|error| format!("lightningcss minify failed: {error}"))?;
    stylesheet
        .to_css(PrinterOptions {
            minify: true,
            ..PrinterOptions::default()
        })
        .map(|result| result.code)
        .map_err(|error| format!("lightningcss print failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_mismatch_discards_external_output() -> Result<(), String> {
        let input = ".a { color: red; }";
        let outcome = run_external_lowering(input, |_| Ok(".a{color:blue}".to_string()))?;

        assert_eq!(outcome.output_css, input);
        assert!(!outcome.report.adopted);
        assert_eq!(outcome.report.exit_status, 0);
        assert_eq!(
            outcome.report.evidence.earned_via.describe(),
            "externalTool"
        );
        assert!(
            outcome
                .report
                .semantic_check
                .as_ref()
                .is_some_and(|check| check.mismatch_count > 0)
        );
        Ok(())
    }

    #[test]
    fn external_failure_preserves_semantic_output() -> Result<(), String> {
        let input = ".a { color: red; }";
        let outcome = run_external_lowering(input, |_| Err("injected failure".to_string()))?;

        assert_eq!(outcome.output_css, input);
        assert!(!outcome.report.adopted);
        assert_eq!(outcome.report.exit_status, 1);
        assert!(outcome.report.semantic_check.is_none());
        Ok(())
    }

    #[cfg(feature = "lightning-lowering")]
    #[test]
    fn lightning_lowering_adopts_semantically_equivalent_output() -> Result<(), String> {
        let input = ".a { display: block; }";
        let outcome = run_hybrid_lightning_lowering(input)?;

        assert!(outcome.report.adopted);
        assert!(outcome.output_css.len() < input.len());
        assert!(
            outcome
                .report
                .semantic_check
                .as_ref()
                .is_some_and(|check| check.preserved)
        );
        Ok(())
    }
}

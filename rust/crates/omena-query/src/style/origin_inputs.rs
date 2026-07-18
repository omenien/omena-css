use omena_cascade::CascadeOriginV0;
use omena_query_checker_orchestrator::OmenaCheckerCascadeDeclarationInputV0;
use serde::Serialize;

use super::cascade_checker::collect_query_checker_cascade_declarations;

const HTML_STANDARD_USER_AGENT_SAMPLE_CSS: &str =
    include_str!("../../data/html-standard-user-agent-sample/stylesheet.css");

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCascadeOriginStylesheetV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_name: String,
    pub origin: CascadeOriginV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_pin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<&'static str>,
    pub declarations: Vec<OmenaCheckerCascadeDeclarationInputV0>,
}

pub fn summarize_omena_query_cascade_origin_stylesheet_v0(
    source_name: impl Into<String>,
    source: &str,
    origin: CascadeOriginV0,
) -> OmenaQueryCascadeOriginStylesheetV0 {
    let declarations = collect_query_checker_cascade_declarations(source)
        .into_iter()
        .map(|mut declaration| {
            declaration.input.origin = origin;
            declaration.input
        })
        .collect();
    OmenaQueryCascadeOriginStylesheetV0 {
        schema_version: "0",
        product: "omena-query.cascade-origin-stylesheet",
        source_name: source_name.into(),
        origin,
        source_url: None,
        source_pin: None,
        license: None,
        declarations,
    }
}

pub fn summarize_html_standard_user_agent_sample_v0() -> OmenaQueryCascadeOriginStylesheetV0 {
    let mut summary = summarize_omena_query_cascade_origin_stylesheet_v0(
        "html-standard-rendering-sample",
        HTML_STANDARD_USER_AGENT_SAMPLE_CSS,
        CascadeOriginV0::UserAgent,
    );
    summary.source_url =
        Some("https://html.spec.whatwg.org/multipage/rendering.html#flow-content-3".to_string());
    summary.source_pin = Some("whatwg/html@9377fd656f519b60524b92f09bcc9e6d937b2017".to_string());
    summary.license = Some("BSD-3-Clause");
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_user_stylesheets_through_the_shared_cascade_input_contract() {
        let summary = summarize_omena_query_cascade_origin_stylesheet_v0(
            "workspace-user-styles",
            ".card { color: green !important; }",
            CascadeOriginV0::User,
        );

        assert_eq!(summary.declarations.len(), 1);
        assert_eq!(summary.declarations[0].origin, CascadeOriginV0::User);
        assert!(summary.declarations[0].important);
        let user_json = serde_json::to_value(&summary.declarations[0]).unwrap_or_default();
        assert_eq!(user_json.get("origin"), Some(&serde_json::json!("user")));

        let author = summarize_omena_query_cascade_origin_stylesheet_v0(
            "author-styles",
            ".card { color: green; }",
            CascadeOriginV0::Author,
        );
        let author_json = serde_json::to_value(&author.declarations[0]).unwrap_or_default();
        assert!(author_json.get("origin").is_none());
    }

    #[test]
    fn projects_the_licensed_html_sample_as_user_agent_input() {
        let summary = summarize_html_standard_user_agent_sample_v0();

        assert!(summary.declarations.len() >= 8);
        assert!(
            summary
                .declarations
                .iter()
                .all(|declaration| declaration.origin == CascadeOriginV0::UserAgent)
        );
        assert_eq!(summary.license, Some("BSD-3-Clause"));
        assert!(
            summary
                .source_pin
                .as_deref()
                .is_some_and(|pin| pin.starts_with("whatwg/html@"))
        );
    }
}

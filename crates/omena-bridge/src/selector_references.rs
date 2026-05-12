use engine_input_producers::{
    EngineInputV2, RangeV2, summarize_selector_usage_evaluator_candidates_input,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorReferenceEngineSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: Option<String>,
    pub selector_count: usize,
    pub referenced_selector_count: usize,
    pub unreferenced_selector_count: usize,
    pub total_reference_sites: usize,
    pub selectors: Vec<SelectorReferenceSummaryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorReferenceSummaryV0 {
    pub canonical_id: String,
    pub file_path: String,
    pub local_name: String,
    pub total_references: usize,
    pub direct_reference_count: usize,
    pub editable_direct_reference_count: usize,
    pub exact_reference_count: usize,
    pub inferred_or_better_reference_count: usize,
    pub has_expanded_references: bool,
    pub has_style_dependency_references: bool,
    pub has_any_references: bool,
    pub sites: Vec<SelectorReferenceSiteV0>,
    pub editable_direct_sites: Vec<SelectorEditableDirectReferenceSiteV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorReferenceSiteV0 {
    pub file_path: String,
    pub range: RangeV2,
    pub expansion: String,
    pub reference_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorEditableDirectReferenceSiteV0 {
    pub file_path: String,
    pub range: RangeV2,
    pub class_name: String,
}

pub fn summarize_omena_bridge_selector_reference_engine(
    input: &EngineInputV2,
    style_path: Option<&str>,
) -> SelectorReferenceEngineSummaryV0 {
    let candidates = summarize_selector_usage_evaluator_candidates_input(input);
    let selectors = candidates
        .results
        .into_iter()
        .filter(|candidate| match style_path {
            Some(path) => candidate.file_path == path,
            None => true,
        })
        .map(|candidate| {
            let payload = candidate.payload;
            SelectorReferenceSummaryV0 {
                canonical_id: format!("selector:{}", payload.canonical_name),
                file_path: candidate.file_path,
                local_name: payload.canonical_name,
                total_references: payload.total_references,
                direct_reference_count: payload.direct_reference_count,
                editable_direct_reference_count: payload.editable_direct_reference_count,
                exact_reference_count: payload.exact_reference_count,
                inferred_or_better_reference_count: payload.inferred_or_better_reference_count,
                has_expanded_references: payload.has_expanded_references,
                has_style_dependency_references: payload.has_style_dependency_references,
                has_any_references: payload.has_any_references,
                sites: payload
                    .all_sites
                    .into_iter()
                    .map(|site| SelectorReferenceSiteV0 {
                        file_path: site.file_path,
                        range: site.range,
                        expansion: site.expansion,
                        reference_kind: site.reference_kind,
                    })
                    .collect(),
                editable_direct_sites: payload
                    .editable_direct_sites
                    .into_iter()
                    .map(|site| SelectorEditableDirectReferenceSiteV0 {
                        file_path: site.file_path,
                        range: site.range,
                        class_name: site.class_name,
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();

    let referenced_selector_count = selectors
        .iter()
        .filter(|selector| selector.has_any_references)
        .count();
    let total_reference_sites = selectors
        .iter()
        .map(|selector| selector.sites.len())
        .sum::<usize>();

    SelectorReferenceEngineSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.selector-references",
        style_path: style_path.map(ToOwned::to_owned),
        selector_count: selectors.len(),
        referenced_selector_count,
        unreferenced_selector_count: selectors.len() - referenced_selector_count,
        total_reference_sites,
        selectors,
    }
}

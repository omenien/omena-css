use oxc_span::SourceType;
use serde::Serialize;
use std::borrow::Cow;

pub(crate) trait SourceLanguageParserV0 {
    fn parser_id(&self) -> &'static str;
    fn language(&self) -> &'static str;
    fn projection_kind(&self) -> &'static str;
    fn source_type(&self, source_path: &str) -> SourceType;
    fn project<'a>(&self, source: &'a str) -> Cow<'a, str>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceLanguageParserKindV0 {
    OxcTsx,
    VueSfcScript,
    HtmlScript,
}

impl SourceLanguageParserV0 for SourceLanguageParserKindV0 {
    fn parser_id(&self) -> &'static str {
        match self {
            Self::OxcTsx => "oxcTsxSourceLanguageParserV0",
            Self::VueSfcScript => "vueSfcScriptProjectionParserV0",
            Self::HtmlScript => "htmlScriptProjectionParserV0",
        }
    }

    fn language(&self) -> &'static str {
        match self {
            Self::OxcTsx => "tsx",
            Self::VueSfcScript => "vue",
            Self::HtmlScript => "html",
        }
    }

    fn projection_kind(&self) -> &'static str {
        match self {
            Self::OxcTsx => "identityOxc",
            Self::VueSfcScript => "bytePreservingScriptBlocks",
            Self::HtmlScript => "bytePreservingScriptBlocks",
        }
    }

    fn source_type(&self, source_path: &str) -> SourceType {
        match self {
            Self::OxcTsx => {
                SourceType::from_path(source_path).unwrap_or_else(|_| SourceType::tsx())
            }
            Self::VueSfcScript | Self::HtmlScript => SourceType::tsx(),
        }
    }

    fn project<'a>(&self, source: &'a str) -> Cow<'a, str> {
        match self {
            Self::OxcTsx => Cow::Borrowed(source),
            Self::VueSfcScript => Cow::Owned(project_tag_contents_to_typescript_source(
                source,
                "<script",
                "</script>",
            )),
            Self::HtmlScript => Cow::Owned(project_tag_contents_to_typescript_source(
                source,
                "<script",
                "</script>",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLanguageParserBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub parser_count: usize,
    pub parsers: Vec<SourceLanguageParserDescriptorV0>,
    pub external_parser_abi_stable: bool,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLanguageParserDescriptorV0 {
    pub parser_id: &'static str,
    pub language: &'static str,
    pub projection_kind: &'static str,
    pub fixture_witnessed: bool,
}

pub fn summarize_omena_bridge_source_language_parser_boundary_v0()
-> SourceLanguageParserBoundarySummaryV0 {
    let parsers = [
        SourceLanguageParserKindV0::OxcTsx,
        SourceLanguageParserKindV0::VueSfcScript,
        SourceLanguageParserKindV0::HtmlScript,
    ]
    .into_iter()
    .map(|parser| SourceLanguageParserDescriptorV0 {
        parser_id: parser.parser_id(),
        language: parser.language(),
        projection_kind: parser.projection_kind(),
        fixture_witnessed: true,
    })
    .collect::<Vec<_>>();

    SourceLanguageParserBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-bridge.source-language-parser-boundary",
        parser_count: parsers.len(),
        parsers,
        external_parser_abi_stable: false,
        ready_surfaces: vec![
            "sourceLanguageParserV0",
            "oxcTsxParserBoundary",
            "vueSfcScriptProjection",
            "htmlScriptProjection",
        ],
    }
}

pub(crate) fn is_vue_source(source_path: &str, source_language: Option<&str>) -> bool {
    source_language == Some("vue") || source_path.ends_with(".vue")
}

pub(crate) fn is_html_source(source_path: &str, source_language: Option<&str>) -> bool {
    source_language == Some("html")
        || source_path.ends_with(".html")
        || source_path.ends_with(".htm")
}

fn source_language_parser_for_path(
    source_path: &str,
    source_language: Option<&str>,
) -> SourceLanguageParserKindV0 {
    if is_vue_source(source_path, source_language) {
        SourceLanguageParserKindV0::VueSfcScript
    } else if is_html_source(source_path, source_language) {
        SourceLanguageParserKindV0::HtmlScript
    } else {
        SourceLanguageParserKindV0::OxcTsx
    }
}

pub(crate) fn project_source_for_language<'a>(
    source_path: &str,
    source: &'a str,
    source_language: Option<&str>,
) -> Cow<'a, str> {
    source_language_parser_for_path(source_path, source_language).project(source)
}

pub(crate) fn source_type_for_language(
    source_path: &str,
    source_language: Option<&str>,
) -> SourceType {
    source_language_parser_for_path(source_path, source_language).source_type(source_path)
}

#[cfg(test)]
fn project_vue_sfc_script_to_typescript_source(source: &str) -> String {
    project_tag_contents_to_typescript_source(source, "<script", "</script>")
}

#[cfg(test)]
fn project_html_script_to_typescript_source(source: &str) -> String {
    project_tag_contents_to_typescript_source(source, "<script", "</script>")
}

fn project_tag_contents_to_typescript_source(
    source: &str,
    open_tag: &str,
    close_tag: &str,
) -> String {
    let lower = source.to_ascii_lowercase();
    let mut keep = vec![false; source.len()];
    let mut cursor = 0usize;

    while let Some(relative_start) = lower[cursor..].find(open_tag) {
        let tag_start = cursor + relative_start;
        let Some(relative_tag_end) = lower[tag_start..].find('>') else {
            break;
        };
        let content_start = tag_start + relative_tag_end + 1;
        let Some(relative_close_start) = lower[content_start..].find(close_tag) else {
            break;
        };
        let content_end = content_start + relative_close_start;
        for item in keep.iter_mut().take(content_end).skip(content_start) {
            *item = true;
        }
        cursor = content_end + close_tag.len();
    }

    let mut projected = String::with_capacity(source.len());
    for (index, ch) in source.char_indices() {
        if ch == '\n' {
            projected.push('\n');
        } else if keep[index] {
            projected.push(ch);
        } else {
            for _ in 0..ch.len_utf8() {
                projected.push(' ');
            }
        }
    }
    projected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vue_sfc_projection_preserves_byte_offsets_and_script_text() {
        let source = "<template>한글</template>\n<script setup lang=\"ts\">\nconst styles = useCssModule();\n</script>\n<style module>.root {}</style>\n";
        let projected = project_vue_sfc_script_to_typescript_source(source);

        assert_eq!(projected.len(), source.len());
        assert_eq!(
            projected.find("styles = useCssModule"),
            source.find("styles = useCssModule")
        );
        assert!(!projected.contains("한글"));
        assert!(!projected.contains(".root"));
    }

    #[test]
    fn html_projection_preserves_script_import_offsets() {
        let source = "<main>ignored</main>\n<script type=\"module\">\nimport styles from \"./App.module.scss\";\n</script>\n";
        let projected = project_html_script_to_typescript_source(source);

        assert_eq!(projected.len(), source.len());
        assert_eq!(
            projected.find("import styles"),
            source.find("import styles")
        );
        assert!(!projected.contains("ignored"));
    }

    #[test]
    fn source_language_parser_boundary_lists_fixture_witnessed_v0_parsers() {
        let summary = summarize_omena_bridge_source_language_parser_boundary_v0();

        assert_eq!(
            summary.product,
            "omena-bridge.source-language-parser-boundary"
        );
        assert_eq!(summary.parser_count, 3);
        assert!(!summary.external_parser_abi_stable);
        assert!(summary.parsers.iter().any(|parser| {
            parser.parser_id == "oxcTsxSourceLanguageParserV0" && parser.fixture_witnessed
        }));
        assert!(summary.parsers.iter().any(|parser| {
            parser.parser_id == "htmlScriptProjectionParserV0" && parser.language == "html"
        }));
    }
}

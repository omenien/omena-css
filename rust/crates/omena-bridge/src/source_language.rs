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
    SvelteComponentScript,
    AstroComponentScript,
    MarkdownFencedCode,
    ServerTemplateMarkup,
}

impl SourceLanguageParserV0 for SourceLanguageParserKindV0 {
    fn parser_id(&self) -> &'static str {
        match self {
            Self::OxcTsx => "oxcTsxSourceLanguageParserV0",
            Self::VueSfcScript => "vueSfcScriptProjectionParserV0",
            Self::HtmlScript => "htmlScriptProjectionParserV0",
            Self::SvelteComponentScript => "svelteComponentScriptProjectionParserV0",
            Self::AstroComponentScript => "astroComponentScriptProjectionParserV0",
            Self::MarkdownFencedCode => "markdownFencedCodeProjectionParserV0",
            Self::ServerTemplateMarkup => "serverTemplateMarkupProjectionParserV0",
        }
    }

    fn language(&self) -> &'static str {
        match self {
            Self::OxcTsx => "tsx",
            Self::VueSfcScript => "vue",
            Self::HtmlScript => "html",
            Self::SvelteComponentScript => "svelte",
            Self::AstroComponentScript => "astro",
            Self::MarkdownFencedCode => "markdown",
            Self::ServerTemplateMarkup => "server-template",
        }
    }

    fn projection_kind(&self) -> &'static str {
        match self {
            Self::OxcTsx => "identityOxc",
            Self::VueSfcScript => "bytePreservingScriptBlocks",
            Self::HtmlScript => "bytePreservingScriptBlocks",
            Self::SvelteComponentScript => "bytePreservingScriptBlocks",
            Self::AstroComponentScript => "bytePreservingFrontmatterAndScriptBlocks",
            Self::MarkdownFencedCode => "bytePreservingFencedCodeBlocks",
            Self::ServerTemplateMarkup => "bytePreservingTemplateMarkupScan",
        }
    }

    fn source_type(&self, source_path: &str) -> SourceType {
        match self {
            Self::OxcTsx => {
                SourceType::from_path(source_path).unwrap_or_else(|_| SourceType::tsx())
            }
            Self::VueSfcScript
            | Self::HtmlScript
            | Self::SvelteComponentScript
            | Self::AstroComponentScript
            | Self::MarkdownFencedCode
            | Self::ServerTemplateMarkup => SourceType::tsx(),
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
            Self::SvelteComponentScript => Cow::Owned(project_tag_contents_to_typescript_source(
                source,
                "<script",
                "</script>",
            )),
            Self::AstroComponentScript => {
                Cow::Owned(project_astro_component_to_typescript_source(source))
            }
            Self::MarkdownFencedCode => Cow::Owned(project_markdown_to_typescript_source(source)),
            Self::ServerTemplateMarkup => Cow::Owned(String::new()),
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
        SourceLanguageParserKindV0::SvelteComponentScript,
        SourceLanguageParserKindV0::AstroComponentScript,
        SourceLanguageParserKindV0::MarkdownFencedCode,
        SourceLanguageParserKindV0::ServerTemplateMarkup,
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
            "svelteComponentScriptProjection",
            "astroComponentScriptProjection",
            "markdownFencedCodeProjection",
            "serverTemplateMarkupScan",
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

pub(crate) fn is_svelte_source(source_path: &str, source_language: Option<&str>) -> bool {
    source_language == Some("svelte") || source_path.ends_with(".svelte")
}

pub(crate) fn is_astro_source(source_path: &str, source_language: Option<&str>) -> bool {
    source_language == Some("astro") || source_path.ends_with(".astro")
}

pub(crate) fn is_markdown_source(source_path: &str, source_language: Option<&str>) -> bool {
    source_language == Some("markdown")
        || source_language == Some("mdx")
        || source_path.ends_with(".md")
        || source_path.ends_with(".mdx")
}

pub(crate) fn is_server_template_source(source_path: &str, source_language: Option<&str>) -> bool {
    matches!(
        source_language,
        Some(
            "liquid"
                | "twig"
                | "nunjucks"
                | "handlebars"
                | "erb"
                | "ejs"
                | "django-html"
                | "jinja"
                | "html-eex"
                | "heex"
        )
    ) || source_path.ends_with(".liquid")
        || source_path.ends_with(".twig")
        || source_path.ends_with(".njk")
        || source_path.ends_with(".nunjucks")
        || source_path.ends_with(".hbs")
        || source_path.ends_with(".handlebars")
        || source_path.ends_with(".erb")
        || source_path.ends_with(".ejs")
        || source_path.ends_with(".html.eex")
        || source_path.ends_with(".heex")
}

fn source_language_parser_for_path(
    source_path: &str,
    source_language: Option<&str>,
) -> SourceLanguageParserKindV0 {
    if is_vue_source(source_path, source_language) {
        SourceLanguageParserKindV0::VueSfcScript
    } else if is_html_source(source_path, source_language) {
        SourceLanguageParserKindV0::HtmlScript
    } else if is_svelte_source(source_path, source_language) {
        SourceLanguageParserKindV0::SvelteComponentScript
    } else if is_astro_source(source_path, source_language) {
        SourceLanguageParserKindV0::AstroComponentScript
    } else if is_markdown_source(source_path, source_language) {
        SourceLanguageParserKindV0::MarkdownFencedCode
    } else if is_server_template_source(source_path, source_language) {
        SourceLanguageParserKindV0::ServerTemplateMarkup
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

#[cfg(test)]
fn project_svelte_component_script_to_typescript_source(source: &str) -> String {
    project_tag_contents_to_typescript_source(source, "<script", "</script>")
}

#[cfg(test)]
fn project_astro_component_script_to_typescript_source(source: &str) -> String {
    project_astro_component_to_typescript_source(source)
}

#[cfg(test)]
fn project_markdown_fenced_code_to_typescript_source(source: &str) -> String {
    project_markdown_to_typescript_source(source)
}

fn project_tag_contents_to_typescript_source(
    source: &str,
    open_tag: &str,
    close_tag: &str,
) -> String {
    project_ranges_to_typescript_source(source, tag_content_ranges(source, open_tag, close_tag))
}

fn project_astro_component_to_typescript_source(source: &str) -> String {
    let mut ranges = Vec::new();
    if let Some(range) = astro_frontmatter_range(source) {
        ranges.push(range);
    }
    ranges.extend(tag_content_ranges(source, "<script", "</script>"));
    project_ranges_to_typescript_source(source, ranges)
}

fn project_markdown_to_typescript_source(source: &str) -> String {
    project_ranges_to_typescript_source(source, markdown_typescript_fence_ranges(source))
}

fn markdown_typescript_fence_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut open_fence: Option<(char, usize, usize)> = None;
    let mut offset = 0usize;

    for line in source.split_inclusive('\n') {
        let line_start = offset;
        let line_end = offset + line.len();
        let line_without_newline = line.trim_end_matches(['\r', '\n']);
        let leading_spaces = line_without_newline
            .chars()
            .take_while(|ch| *ch == ' ')
            .count();
        let trimmed = line_without_newline.trim_start_matches(' ');
        if leading_spaces <= 3 {
            if let Some((fence_char, fence_len, content_start)) = open_fence {
                if markdown_fence_marker(trimmed).is_some_and(|(candidate_char, candidate_len)| {
                    candidate_char == fence_char && candidate_len >= fence_len
                }) {
                    ranges.push((content_start, line_start));
                    open_fence = None;
                }
            } else if let Some((fence_char, fence_len)) = markdown_fence_marker(trimmed) {
                let language = trimmed[fence_len..].trim();
                if markdown_fence_language_is_typescript(language) {
                    open_fence = Some((fence_char, fence_len, line_end));
                }
            }
        }
        offset = line_end;
    }

    if let Some((_, _, content_start)) = open_fence {
        ranges.push((content_start, source.len()));
    }
    ranges
}

fn markdown_fence_marker(line: &str) -> Option<(char, usize)> {
    let mut chars = line.chars();
    let fence_char = chars.next()?;
    if fence_char != '`' && fence_char != '~' {
        return None;
    }
    let fence_len = 1 + chars.take_while(|ch| *ch == fence_char).count();
    if fence_len >= 3 {
        Some((fence_char, fence_len))
    } else {
        None
    }
}

fn markdown_fence_language_is_typescript(language: &str) -> bool {
    let normalized = language
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "ts" | "tsx"
            | "typescript"
            | "typescriptreact"
            | "js"
            | "jsx"
            | "javascript"
            | "javascriptreact"
    )
}

pub(crate) fn tag_content_ranges(
    source: &str,
    open_tag: &str,
    close_tag: &str,
) -> Vec<(usize, usize)> {
    let lower = source.to_ascii_lowercase();
    let mut cursor = 0usize;
    let mut ranges = Vec::new();

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
        ranges.push((content_start, content_end));
        cursor = content_end + close_tag.len();
    }
    ranges
}

fn astro_frontmatter_range(source: &str) -> Option<(usize, usize)> {
    if !source.starts_with("---") {
        return None;
    }
    let content_start = source[3..].find('\n')? + 4;
    let relative_close = source[content_start..].find("\n---")?;
    Some((content_start, content_start + relative_close))
}

fn project_ranges_to_typescript_source(
    source: &str,
    ranges: impl IntoIterator<Item = (usize, usize)>,
) -> String {
    let mut keep = vec![false; source.len()];
    for (start, end) in ranges {
        for item in keep.iter_mut().take(end).skip(start) {
            *item = true;
        }
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
    fn svelte_projection_preserves_script_import_offsets() {
        let source = "<script lang=\"ts\">\nimport styles from \"./Card.module.scss\";\nexport const root = styles.root;\n</script>\n<section>ignored</section>\n<style>.root { color: red; }</style>\n";
        let projected = project_svelte_component_script_to_typescript_source(source);

        assert_eq!(projected.len(), source.len());
        assert_eq!(
            projected.find("import styles"),
            source.find("import styles")
        );
        assert_eq!(projected.find("styles.root"), source.find("styles.root"));
        assert!(!projected.contains("ignored"));
        assert!(!projected.contains("color: red"));
    }

    #[test]
    fn astro_projection_preserves_frontmatter_and_script_import_offsets() {
        let source = "---\nimport styles from \"./Card.module.scss\";\nconst root = styles.root;\n---\n<div class={root}>ignored</div>\n<script>\nconst local = styles.root;\n</script>\n<style>.root { color: red; }</style>\n";
        let projected = project_astro_component_script_to_typescript_source(source);

        assert_eq!(projected.len(), source.len());
        assert_eq!(
            projected.find("import styles"),
            source.find("import styles")
        );
        assert_eq!(projected.find("const local"), source.find("const local"));
        assert!(!projected.contains("ignored"));
        assert!(!projected.contains("color: red"));
    }

    #[test]
    fn markdown_projection_preserves_typescript_fenced_code_offsets() {
        let source = "# Notes\n\nignored text\n\n```tsx\nimport styles from \"./Card.module.scss\";\nconst root = styles.root;\n```\n\n```css\n.root { color: red; }\n```\n";
        let projected = project_markdown_fenced_code_to_typescript_source(source);

        assert_eq!(projected.len(), source.len());
        assert_eq!(
            projected.find("import styles"),
            source.find("import styles")
        );
        assert_eq!(projected.find("styles.root"), source.find("styles.root"));
        assert!(!projected.contains("ignored text"));
        assert!(!projected.contains("color: red"));
    }

    #[test]
    fn source_language_parser_boundary_lists_fixture_witnessed_v0_parsers() {
        let summary = summarize_omena_bridge_source_language_parser_boundary_v0();

        assert_eq!(
            summary.product,
            "omena-bridge.source-language-parser-boundary"
        );
        assert_eq!(summary.parser_count, 7);
        assert!(!summary.external_parser_abi_stable);
        assert!(summary.parsers.iter().any(|parser| {
            parser.parser_id == "oxcTsxSourceLanguageParserV0" && parser.fixture_witnessed
        }));
        assert!(summary.parsers.iter().any(|parser| {
            parser.parser_id == "htmlScriptProjectionParserV0" && parser.language == "html"
        }));
        assert!(summary.parsers.iter().any(|parser| {
            parser.parser_id == "svelteComponentScriptProjectionParserV0"
                && parser.language == "svelte"
        }));
        assert!(summary.parsers.iter().any(|parser| {
            parser.parser_id == "astroComponentScriptProjectionParserV0"
                && parser.language == "astro"
        }));
        assert!(summary.parsers.iter().any(|parser| {
            parser.parser_id == "markdownFencedCodeProjectionParserV0"
                && parser.language == "markdown"
        }));
        assert!(summary.parsers.iter().any(|parser| {
            parser.parser_id == "serverTemplateMarkupProjectionParserV0"
                && parser.language == "server-template"
        }));
    }
}

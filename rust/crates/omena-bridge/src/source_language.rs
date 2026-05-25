use oxc_span::SourceType;
use std::borrow::Cow;

pub(crate) fn is_vue_source(source_path: &str, source_language: Option<&str>) -> bool {
    source_language == Some("vue") || source_path.ends_with(".vue")
}

pub(crate) fn project_source_for_language<'a>(
    source_path: &str,
    source: &'a str,
    source_language: Option<&str>,
) -> Cow<'a, str> {
    if is_vue_source(source_path, source_language) {
        Cow::Owned(project_vue_sfc_script_to_typescript_source(source))
    } else {
        Cow::Borrowed(source)
    }
}

pub(crate) fn source_type_for_language(
    source_path: &str,
    source_language: Option<&str>,
) -> SourceType {
    if is_vue_source(source_path, source_language) {
        return SourceType::tsx();
    }
    SourceType::from_path(source_path).unwrap_or_else(|_| SourceType::tsx())
}

fn project_vue_sfc_script_to_typescript_source(source: &str) -> String {
    let lower = source.to_ascii_lowercase();
    let mut keep = vec![false; source.len()];
    let mut cursor = 0usize;

    while let Some(relative_start) = lower[cursor..].find("<script") {
        let tag_start = cursor + relative_start;
        let Some(relative_tag_end) = lower[tag_start..].find('>') else {
            break;
        };
        let content_start = tag_start + relative_tag_end + 1;
        let Some(relative_close_start) = lower[content_start..].find("</script>") else {
            break;
        };
        let content_end = content_start + relative_close_start;
        for item in keep.iter_mut().take(content_end).skip(content_start) {
            *item = true;
        }
        cursor = content_end + "</script>".len();
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
}

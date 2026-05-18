pub const Z5_PERFORMANCE_BASELINE: &str = "z5-performance-baseline";

use omena_parser::StyleDialect;

pub struct StyleSample {
    pub name: &'static str,
    pub path: &'static str,
    pub dialect: StyleDialect,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserProductBenchmarkBoundaryV0 {
    pub lane: &'static str,
    pub input_boundary: &'static str,
    pub measured_operation: &'static str,
    pub includes_parse: bool,
    pub includes_product_summary: bool,
}

pub fn style_corpus() -> Vec<StyleSample> {
    vec![
        StyleSample {
            name: "nextjs14-dashboard-scss",
            path: "DashboardCard.module.scss",
            dialect: StyleDialect::Scss,
            source: build_nextjs14_dashboard_scss(96),
        },
        StyleSample {
            name: "vite-component-css",
            path: "MarketingGrid.module.css",
            dialect: StyleDialect::Css,
            source: build_vite_component_css(128),
        },
        StyleSample {
            name: "scss-heavy-design-system",
            path: "DesignSystem.module.scss",
            dialect: StyleDialect::Scss,
            source: build_scss_heavy_design_system(72),
        },
    ]
}

pub fn parser_product_benchmark_boundaries() -> [ParserProductBenchmarkBoundaryV0; 2] {
    [
        ParserProductBenchmarkBoundaryV0 {
            lane: "legacy",
            input_boundary: "raw-style-source",
            measured_operation: "parse-plus-product-summary",
            includes_parse: true,
            includes_product_summary: true,
        },
        ParserProductBenchmarkBoundaryV0 {
            lane: "omena",
            input_boundary: "raw-style-source",
            measured_operation: "parse-plus-product-summary",
            includes_parse: true,
            includes_product_summary: true,
        },
    ]
}

pub fn validate_parser_product_benchmark_boundary_symmetry() -> Result<(), String> {
    let [legacy, omena] = parser_product_benchmark_boundaries();
    if legacy.input_boundary != omena.input_boundary {
        return Err(format!(
            "parser product benchmark input boundary mismatch: legacy={} omena={}",
            legacy.input_boundary, omena.input_boundary,
        ));
    }
    if legacy.measured_operation != omena.measured_operation {
        return Err(format!(
            "parser product benchmark operation mismatch: legacy={} omena={}",
            legacy.measured_operation, omena.measured_operation,
        ));
    }
    if !(legacy.includes_parse
        && omena.includes_parse
        && legacy.includes_product_summary
        && omena.includes_product_summary)
    {
        return Err(
            "parser product benchmark must include parse and product summary for both lanes"
                .to_string(),
        );
    }
    Ok(())
}

pub fn parse_legacy_style_sample(
    path: &str,
    source: &str,
) -> Option<engine_style_parser::Stylesheet> {
    engine_style_parser::parse_style_module(path, source)
}

pub fn summarize_legacy_style_sample(
    path: &str,
    source: &str,
) -> Option<engine_style_parser::ParserIndexSummaryV0> {
    summarize_legacy_parser_product_sample(path, source)
}

pub fn summarize_legacy_parser_product_sample(
    path: &str,
    source: &str,
) -> Option<engine_style_parser::ParserIndexSummaryV0> {
    let sheet = parse_legacy_style_sample(path, source)?;
    Some(engine_style_parser::summarize_css_modules_intermediate(
        &sheet,
    ))
}

pub fn summarize_omena_style_sample(
    source: &str,
    dialect: StyleDialect,
) -> omena_parser::ParserIndexSummaryV0 {
    summarize_omena_parser_product_sample(source, dialect)
}

pub fn summarize_omena_parser_product_sample(
    source: &str,
    dialect: StyleDialect,
) -> omena_parser::ParserIndexSummaryV0 {
    let parsed = omena_parser::parse(source, dialect);
    let _ = std::hint::black_box(parsed.token_count());
    omena_parser::summarize_css_modules_intermediate(source, dialect)
}

pub fn validate_omena_style_sample(source: &str, dialect: StyleDialect) -> Result<(), String> {
    let parsed = omena_parser::parse(source, dialect);
    if parsed.token_count() > 0 {
        Ok(())
    } else {
        Err(format!(
            "benchmark style sample should produce omena parser tokens: {dialect:?}",
        ))
    }
}

pub fn validate_legacy_style_sample(path: &str, source: &str) -> Result<(), String> {
    if parse_legacy_style_sample(path, source).is_some() {
        Ok(())
    } else {
        Err(format!(
            "benchmark style sample should be accepted by legacy parser: {path}",
        ))
    }
}

fn build_nextjs14_dashboard_scss(count: usize) -> String {
    let mut source = String::from(
        r#"
@use "./tokens" as tokens;
@value brand: #0f766e;

.dashboard {
  display: grid;
  gap: 12px;
"#,
    );
    for index in 0..count {
        source.push_str(&format!(
            r#"
  &__card{index} {{
    color: tokens.$accent;
    --card-tone-{index}: brand;

    &--active {{
      border-color: var(--card-tone-{index});
    }}
  }}
"#
        ));
    }
    source.push_str("}\n");
    source
}

fn build_vite_component_css(count: usize) -> String {
    let mut source = String::new();
    for index in 0..count {
        source.push_str(&format!(
            r#"
.tile{index} {{
  color: rgb({red}, {green}, 40);
  animation: tilePulse{index} 120ms ease-out;
}}

@keyframes tilePulse{index} {{
  from {{ opacity: 0; }}
  to {{ opacity: 1; }}
}}
"#,
            red = index % 255,
            green = (index * 7) % 255,
        ));
    }
    source
}

fn build_scss_heavy_design_system(count: usize) -> String {
    let mut source = String::from(
        r#"
@forward "./palette";
@mixin elevation($level) {
  box-shadow: 0 $level 12px rgb(15 23 42 / 16%);
}

.component {
"#,
    );
    for index in 0..count {
        source.push_str(&format!(
            r#"
  &--tone-{index} {{
    @include elevation({level}px);

    .component__label{index} {{
      color: var(--tone-{index});
    }}
  }}
"#,
            level = (index % 8) + 1,
        ));
    }
    source.push_str("}\n");
    source
}

#[cfg(test)]
mod tests {
    use super::{
        parser_product_benchmark_boundaries, style_corpus, summarize_legacy_parser_product_sample,
        summarize_omena_parser_product_sample, validate_legacy_style_sample,
        validate_omena_style_sample, validate_parser_product_benchmark_boundary_symmetry,
    };

    #[test]
    fn parser_product_benchmarks_declare_symmetric_measurement_boundary() -> Result<(), String> {
        validate_parser_product_benchmark_boundary_symmetry()?;
        let boundaries = parser_product_benchmark_boundaries();

        assert_eq!(boundaries.len(), 2);
        assert!(boundaries.iter().all(|boundary| boundary.includes_parse));
        assert!(
            boundaries
                .iter()
                .all(|boundary| boundary.includes_product_summary)
        );
        assert!(
            boundaries
                .iter()
                .all(|boundary| boundary.input_boundary == "raw-style-source")
        );
        Ok(())
    }

    #[test]
    fn parser_product_samples_use_symmetric_parse_plus_summary_boundaries() -> Result<(), String> {
        for sample in style_corpus() {
            validate_legacy_style_sample(sample.path, sample.source.as_str())?;
            validate_omena_style_sample(sample.source.as_str(), sample.dialect)?;

            let legacy =
                summarize_legacy_parser_product_sample(sample.path, sample.source.as_str())
                    .ok_or_else(|| format!("legacy parser product failed for {}", sample.name))?;
            let omena =
                summarize_omena_parser_product_sample(sample.source.as_str(), sample.dialect);
            let legacy = serde_json::to_value(legacy).map_err(|error| error.to_string())?;
            let omena = serde_json::to_value(omena).map_err(|error| error.to_string())?;

            assert_eq!(legacy["language"], omena["language"]);
            assert!(legacy["selectors"]["names"].as_array().is_some());
            assert!(omena["selectors"]["names"].as_array().is_some());
            assert!(legacy["wrappers"].as_object().is_some());
            assert!(omena["wrappers"].as_object().is_some());
        }
        Ok(())
    }
}

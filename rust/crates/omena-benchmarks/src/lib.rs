pub const Z5_PERFORMANCE_BASELINE: &str = "z5-performance-baseline";

use omena_parser::StyleDialect;

pub struct StyleSample {
    pub name: &'static str,
    pub path: &'static str,
    pub dialect: StyleDialect,
    pub source: String,
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
    let sheet = parse_legacy_style_sample(path, source)?;
    Some(engine_style_parser::summarize_css_modules_intermediate(
        &sheet,
    ))
}

pub fn summarize_omena_style_sample_with_parse(
    source: &str,
    dialect: StyleDialect,
) -> omena_parser::ParserIndexSummaryV0 {
    let parsed = omena_parser::parse(source, dialect);
    std::hint::black_box(parsed);
    omena_parser::summarize_css_modules_intermediate(source, dialect)
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

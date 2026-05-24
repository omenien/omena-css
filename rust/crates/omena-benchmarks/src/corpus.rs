use omena_parser::StyleDialect;
use serde::Serialize;

use crate::Z5_PERFORMANCE_BASELINE;

pub struct StyleSample {
    pub name: &'static str,
    pub path: &'static str,
    pub dialect: StyleDialect,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleCorpusSampleSnapshotV0 {
    pub name: &'static str,
    pub path: &'static str,
    pub dialect: &'static str,
    pub byte_length: usize,
    pub line_count: usize,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleCorpusSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub benchmark_family: &'static str,
    pub corpus_sample_count: usize,
    pub samples: Vec<StyleCorpusSampleSnapshotV0>,
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
            name: "css-sizing-width-corpus",
            path: "SizingGrid.module.css",
            dialect: StyleDialect::Css,
            source: build_css_sizing_width_corpus(96),
        },
        StyleSample {
            name: "css-backgrounds-longhand-corpus",
            path: "BackgroundLayers.module.css",
            dialect: StyleDialect::Css,
            source: build_css_backgrounds_longhand_corpus(96),
        },
        StyleSample {
            name: "css-display-layout-corpus",
            path: "DisplayModes.module.css",
            dialect: StyleDialect::Css,
            source: build_css_display_layout_corpus(96),
        },
        StyleSample {
            name: "css-position-layout-corpus",
            path: "PositionModes.module.css",
            dialect: StyleDialect::Css,
            source: build_css_position_layout_corpus(96),
        },
        StyleSample {
            name: "css-ui-box-model-corpus",
            path: "BoxModel.module.css",
            dialect: StyleDialect::Css,
            source: build_css_ui_box_model_corpus(96),
        },
        StyleSample {
            name: "scss-heavy-design-system",
            path: "DesignSystem.module.scss",
            dialect: StyleDialect::Scss,
            source: build_scss_heavy_design_system(72),
        },
    ]
}

pub fn summarize_style_corpus_snapshot() -> StyleCorpusSnapshotV0 {
    let samples = style_corpus()
        .into_iter()
        .map(|sample| {
            let line_count = sample.source.lines().count();
            let byte_length = sample.source.len();
            StyleCorpusSampleSnapshotV0 {
                name: sample.name,
                path: sample.path,
                dialect: style_dialect_label(sample.dialect),
                byte_length,
                line_count,
                source: sample.source,
            }
        })
        .collect::<Vec<_>>();

    StyleCorpusSnapshotV0 {
        schema_version: "0",
        product: "omena-benchmarks.style-corpus-snapshot",
        benchmark_family: Z5_PERFORMANCE_BASELINE,
        corpus_sample_count: samples.len(),
        samples,
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

fn build_css_sizing_width_corpus(count: usize) -> String {
    let width_values = [
        "min-content",
        "max-content",
        "fit-content(12rem)",
        "calc(100% - 1.5rem)",
    ];
    let mut source = String::new();
    for index in 0..count {
        let width = width_values[index % width_values.len()];
        source.push_str(&format!(
            r#"
.sizingCell{index} {{
  width: {width};
  min-width: min(100%, {min_width}rem);
  max-width: max-content;
}}
"#,
            min_width = (index % 12) + 4,
        ));
    }
    source
}

fn build_css_backgrounds_longhand_corpus(count: usize) -> String {
    let repeat_values = ["repeat", "repeat-x", "no-repeat", "space round"];
    let position_values = ["top left", "center", "right 12% bottom", "20% 0%"];
    let size_values = ["cover", "contain", "auto 4%", "2% 3%"];
    let mut source = String::new();
    for index in 0..count {
        source.push_str(&format!(
            r#"
.backgroundLayer{index} {{
  background-color: rgb({red}, {green}, 40);
  background-image: none;
  background-repeat: {repeat};
  background-position: {position};
  background-size: {size};
}}
"#,
            red = index % 255,
            green = (index * 11) % 255,
            repeat = repeat_values[index % repeat_values.len()],
            position = position_values[index % position_values.len()],
            size = size_values[index % size_values.len()],
        ));
    }
    source
}

fn build_css_display_layout_corpus(count: usize) -> String {
    let display_values = ["grid", "inline-grid", "flex", "contents"];
    let mut source = String::new();
    for index in 0..count {
        source.push_str(&format!(
            r#"
.displayMode{index} {{
  display: {display};
  gap: {gap}px;
}}
"#,
            display = display_values[index % display_values.len()],
            gap = (index % 16) + 1,
        ));
    }
    source
}

fn build_css_position_layout_corpus(count: usize) -> String {
    let position_values = ["static", "relative", "absolute", "sticky", "fixed"];
    let mut source = String::new();
    for index in 0..count {
        source.push_str(&format!(
            r#"
.positionMode{index} {{
  position: {position};
  inset: {offset}px;
}}
"#,
            position = position_values[index % position_values.len()],
            offset = index % 24,
        ));
    }
    source
}

fn build_css_ui_box_model_corpus(count: usize) -> String {
    let box_sizing_values = ["content-box", "border-box"];
    let mut source = String::new();
    for index in 0..count {
        source.push_str(&format!(
            r#"
.boxModel{index} {{
  box-sizing: {box_sizing};
  inline-size: {width}px;
  block-size: {height}px;
}}
"#,
            box_sizing = box_sizing_values[index % box_sizing_values.len()],
            width = (index % 32) + 16,
            height = (index % 24) + 12,
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

fn style_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

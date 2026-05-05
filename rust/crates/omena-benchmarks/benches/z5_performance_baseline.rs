use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use engine_style_parser::{Stylesheet, parse_style_module};
use omena_abstract_value::{
    ClassValueFlowGraphV0, ClassValueFlowNodeV0, ClassValueFlowTransferV0,
    ExternalStringTypeFactsV0, OneCfaCallSiteFlowInputV0, analyze_class_value_flow,
    analyze_one_cfa_call_site_flows, finite_set_class_value, intersect_abstract_class_values,
    prefix_class_value,
};
use omena_semantic::summarize_style_semantic_boundary;

struct StyleSample {
    name: &'static str,
    path: &'static str,
    source: String,
}

struct ParsedStyleSample {
    name: &'static str,
    sheet: Stylesheet,
}

fn parser_benchmarks(c: &mut Criterion) {
    let samples = style_corpus();
    let mut group = c.benchmark_group("z5/parser");
    for sample in &samples {
        group.bench_function(sample.name, |b| {
            b.iter(|| {
                black_box(parse_style_module(
                    black_box(sample.path),
                    black_box(&sample.source),
                ))
            });
        });
    }
    group.finish();
}

fn semantic_benchmarks(c: &mut Criterion) {
    let samples = parsed_style_corpus();
    let mut group = c.benchmark_group("z5/semantic");
    for sample in &samples {
        group.bench_function(sample.name, |b| {
            b.iter(|| {
                black_box(summarize_style_semantic_boundary(black_box(&sample.sheet)));
            });
        });
    }
    group.finish();
}

fn abstract_value_benchmarks(c: &mut Criterion) {
    let graph = build_flow_graph(256);
    let call_site_inputs = build_one_cfa_inputs(40, 64);
    let finite = finite_set_class_value([
        "button-primary",
        "button-secondary",
        "button-danger",
        "button-muted",
        "card-primary",
        "card-secondary",
    ]);
    let prefix = prefix_class_value("button-", None);

    let mut group = c.benchmark_group("z5/abstract-value");
    group.bench_function("flow-1cfa-256-nodes", |b| {
        b.iter(|| {
            black_box(analyze_class_value_flow(black_box(&graph)));
        });
    });
    group.bench_function("one-cfa-40-call-sites", |b| {
        b.iter(|| {
            black_box(analyze_one_cfa_call_site_flows(black_box(
                &call_site_inputs,
            )));
        });
    });
    group.bench_function("reduced-product-intersection", |b| {
        b.iter(|| {
            black_box(intersect_abstract_class_values(
                black_box(&finite),
                black_box(&prefix),
            ));
        });
    });
    group.finish();
}

fn parsed_style_corpus() -> Vec<ParsedStyleSample> {
    style_corpus()
        .into_iter()
        .filter_map(|sample| {
            parse_style_module(sample.path, &sample.source).map(|sheet| ParsedStyleSample {
                name: sample.name,
                sheet,
            })
        })
        .collect()
}

fn style_corpus() -> Vec<StyleSample> {
    vec![
        StyleSample {
            name: "nextjs14-dashboard-scss",
            path: "DashboardCard.module.scss",
            source: build_nextjs14_dashboard_scss(96),
        },
        StyleSample {
            name: "vite-component-css",
            path: "MarketingGrid.module.css",
            source: build_vite_component_css(128),
        },
        StyleSample {
            name: "scss-heavy-design-system",
            path: "DesignSystem.module.scss",
            source: build_scss_heavy_design_system(72),
        },
    ]
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

fn build_flow_graph(node_count: usize) -> ClassValueFlowGraphV0 {
    let mut nodes = Vec::with_capacity(node_count);
    for index in 0..node_count {
        let id = format!("n{index}");
        let predecessors = if index == 0 {
            Vec::new()
        } else {
            vec![format!("n{}", index - 1)]
        };
        let transfer = match index % 4 {
            0 => ClassValueFlowTransferV0::AssignFacts(finite_fact(index)),
            1 => ClassValueFlowTransferV0::RefineFacts(prefix_fact(index)),
            2 => ClassValueFlowTransferV0::ConcatFacts(exact_fact(format!("--s{index}"))),
            _ => ClassValueFlowTransferV0::Join,
        };
        nodes.push(ClassValueFlowNodeV0 {
            id,
            predecessors,
            transfer,
        });
    }

    ClassValueFlowGraphV0 {
        context_key: Some("benchmark-flow".to_string()),
        nodes,
    }
}

fn build_one_cfa_inputs(
    call_site_count: usize,
    node_count: usize,
) -> Vec<OneCfaCallSiteFlowInputV0> {
    (0..call_site_count)
        .map(|index| OneCfaCallSiteFlowInputV0 {
            callee_key: "cxFactory".to_string(),
            call_site_id: format!("call-site-{index}"),
            graph: build_flow_graph(node_count),
            exit_node_id: format!("n{}", node_count.saturating_sub(1)),
        })
        .collect()
}

fn exact_fact(value: String) -> ExternalStringTypeFactsV0 {
    ExternalStringTypeFactsV0 {
        kind: "exact".to_string(),
        constraint_kind: None,
        values: Some(vec![value]),
        prefix: None,
        suffix: None,
        min_len: None,
        max_len: None,
        char_must: None,
        char_may: None,
        may_include_other_chars: None,
    }
}

fn finite_fact(index: usize) -> ExternalStringTypeFactsV0 {
    ExternalStringTypeFactsV0 {
        kind: "finiteSet".to_string(),
        constraint_kind: None,
        values: Some(vec![
            format!("button-primary-{index}"),
            format!("button-secondary-{index}"),
            format!("button-danger-{index}"),
        ]),
        prefix: None,
        suffix: None,
        min_len: None,
        max_len: None,
        char_must: None,
        char_may: None,
        may_include_other_chars: None,
    }
}

fn prefix_fact(index: usize) -> ExternalStringTypeFactsV0 {
    ExternalStringTypeFactsV0 {
        kind: "constrained".to_string(),
        constraint_kind: Some("prefix".to_string()),
        values: None,
        prefix: Some(format!("button-{index}")),
        suffix: None,
        min_len: None,
        max_len: None,
        char_must: None,
        char_may: None,
        may_include_other_chars: None,
    }
}

criterion_group!(
    benches,
    parser_benchmarks,
    semantic_benchmarks,
    abstract_value_benchmarks
);
criterion_main!(benches);

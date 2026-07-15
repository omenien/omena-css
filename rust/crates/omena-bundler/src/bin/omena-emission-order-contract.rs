use omena_bundler::{
    EmissionOrderingPolicyV0, TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0, TransformBundleModuleInputV0,
    compare_omena_transform_bundle_emission_policies,
};
use omena_cross_file_summary::CROSS_FILE_SUMMARY_RAW_EDGE_KIND_VARIANTS_V0;
use omena_parser::StyleDialect;

fn main() {
    let inject_unclassified = std::env::args().any(|arg| arg == "--inject-unclassified-edge-kind");
    let observed_raw_kind_count = CROSS_FILE_SUMMARY_RAW_EDGE_KIND_VARIANTS_V0
        .len()
        .saturating_add(usize::from(inject_unclassified));
    let classified_raw_kind_count = CROSS_FILE_SUMMARY_RAW_EDGE_KIND_VARIANTS_V0.len();
    if observed_raw_kind_count != classified_raw_kind_count {
        eprintln!(
            "unclassified edge kind reached emission ordering: observed={observed_raw_kind_count}, classified={classified_raw_kind_count}"
        );
        std::process::exit(1);
    }

    println!(
        "report\t0\tomena-bundler.emission-order-contract\t{}\t{}\t{}",
        classified_raw_kind_count,
        TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0.len(),
        EmissionOrderingPolicyV0::default().as_wire_label()
    );
    for kind in CROSS_FILE_SUMMARY_RAW_EDGE_KIND_VARIANTS_V0 {
        println!(
            "raw\t{}\t{}",
            kind.as_wire_label(),
            kind.order_relevance().as_wire_label()
        );
    }
    for kind in TRANSFORM_BUNDLE_EDGE_KIND_VARIANTS_V0 {
        println!(
            "bundle\t{}\t{}\t{}",
            kind.as_wire_label(),
            kind.order_relevance().as_wire_label(),
            kind.order_relevance_reason()
        );
    }
    for (fixture_id, modules) in differential_fixtures() {
        let report = compare_omena_transform_bundle_emission_policies(
            &[format!("src/app.{fixture_id}")],
            &modules,
        )
        .unwrap_or_else(|error| {
            eprintln!("emission policy differential failed for {fixture_id}: {error:?}");
            std::process::exit(1);
        });
        println!(
            "policy\t{}\t{}\t{}\t{}\t{}",
            fixture_id,
            report.module_id_legacy_rule_count,
            report.import_order_rule_count,
            report.difference_count,
            report.equivalent
        );
        for difference in report.differences {
            println!(
                "difference\t{}\t{}\t{}\t{}\t{}\t{}",
                fixture_id,
                difference.output_index,
                module_label(difference.module_id_legacy_module.as_ref()),
                difference
                    .module_id_legacy_selector
                    .as_deref()
                    .unwrap_or(""),
                module_label(difference.import_order_module.as_ref()),
                difference.import_order_selector.as_deref().unwrap_or("")
            );
        }
    }
}

fn module_label(module: Option<&omena_parser::ModuleInstanceKeyV0>) -> &str {
    module.map_or("", |module| module.module().as_str())
}

fn differential_fixtures() -> [(&'static str, Vec<TransformBundleModuleInputV0>); 3] {
    [
        (
            "css",
            fixture_modules(
                "css",
                r#"@import "./z.css"; @import "./a.css"; .app { color: red; }"#,
                StyleDialect::Css,
            ),
        ),
        (
            "scss",
            fixture_modules(
                "scss",
                r#"@use "./z.scss"; @use "./a.scss"; .app { color: red; }"#,
                StyleDialect::Scss,
            ),
        ),
        (
            "less",
            fixture_modules(
                "less",
                r#"@import "./z.less"; @import "./a.less"; .app { color: red; }"#,
                StyleDialect::Less,
            ),
        ),
    ]
}

fn fixture_modules(
    extension: &str,
    entry_source: &str,
    dialect: StyleDialect,
) -> Vec<TransformBundleModuleInputV0> {
    vec![
        TransformBundleModuleInputV0::new(format!("src/app.{extension}"), entry_source, dialect),
        TransformBundleModuleInputV0::new(
            format!("src/a.{extension}"),
            ".a { color: blue; }",
            dialect,
        ),
        TransformBundleModuleInputV0::new(
            format!("src/z.{extension}"),
            ".z { color: green; }",
            dialect,
        ),
    ]
}

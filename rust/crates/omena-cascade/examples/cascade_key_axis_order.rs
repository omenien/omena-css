use omena_cascade::summarize_cascade_margin_schema_v0;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CascadeKeyAxisOrderArtifactV0 {
    schema_version: &'static str,
    product: &'static str,
    axis_order: Vec<&'static str>,
    ranked_set_prefix_axis_vocabulary: Vec<&'static str>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = summarize_cascade_margin_schema_v0();
    let specificity_start = schema
        .axis_order
        .iter()
        .position(|axis| axis.starts_with("specificity"))
        .ok_or("cascade key axis order is missing specificity")?;
    let ranked_set_prefix_axis_vocabulary = schema.axis_order[..specificity_start].to_vec();
    let artifact = CascadeKeyAxisOrderArtifactV0 {
        schema_version: "0",
        product: "omena-cascade.key-axis-order",
        axis_order: schema.axis_order,
        ranked_set_prefix_axis_vocabulary,
    };
    println!("{}", serde_json::to_string_pretty(&artifact)?);
    Ok(())
}

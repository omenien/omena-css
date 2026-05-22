use omena_query::summarize_omena_query_m4_axis_c_readiness;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let summary = summarize_omena_query_m4_axis_c_readiness();
    serde_json::to_writer_pretty(std::io::stdout(), &summary)?;
    println!();

    if summary.status != "m4AxisCReady" {
        return Err("M4 Axis C readiness summary is not ready".into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let summary = omena_spec_audit::summarize_omena_spec_audit_boundary();
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

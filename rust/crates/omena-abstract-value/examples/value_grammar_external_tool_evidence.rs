use std::{env, error::Error};

use omena_abstract_value::css_value_grammar_external_tool_evidence_v0;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let tool_name = args.next().ok_or("missing tool name")?;
    let tool_version = args.next().ok_or("missing tool version")?;
    let input_digest = args.next().ok_or("missing input digest")?;
    let exit_status = args.next().ok_or("missing exit status")?.parse::<i32>()?;
    if args.next().is_some() {
        return Err("unexpected trailing arguments".into());
    }
    let evidence = css_value_grammar_external_tool_evidence_v0(
        tool_name.as_str(),
        tool_version.as_str(),
        input_digest.as_str(),
        exit_status,
    );
    println!("{}", serde_json::to_string_pretty(&evidence)?);
    Ok(())
}

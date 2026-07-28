use std::io::{self, Read};

use omena_tsgo_client::{
    TsgoJsonRpcTypeFactProviderV0, TsgoSpanTypeFactRequestV0, TsgoTypeFactRequestV0,
    TsgoWorkspaceProcessConfigV0, TsgoWorkspaceProcessPoolV0, build_tsgo_process_command,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptureRequest {
    tsgo_path: String,
    request: TsgoSpanTypeFactRequestV0,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let capture: CaptureRequest = serde_json::from_str(input.as_str())?;
    let position_request = TsgoTypeFactRequestV0 {
        workspace_root: capture.request.workspace_root.clone(),
        config_path: capture.request.config_path.clone(),
        targets: Vec::new(),
    };
    let mut pool = TsgoWorkspaceProcessPoolV0::default();
    pool.ensure_workspace_process(TsgoWorkspaceProcessConfigV0 {
        workspace_root: capture.request.workspace_root.clone(),
        command: build_tsgo_process_command(
            capture.tsgo_path.as_str(),
            capture.request.workspace_root.as_str(),
            None,
        ),
    })?;
    let mut provider = TsgoJsonRpcTypeFactProviderV0::new(pool);
    let result =
        provider.collect_type_facts_with_span_targets(&position_request, &capture.request)?;
    serde_json::to_writer_pretty(io::stdout(), &result.span_type_fact_entries)?;
    Ok(())
}

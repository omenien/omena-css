use omena_cascade::{
    DiagnosticFrameFootprintV0, ModuleFootprintV0, RecheckSelectionV0, compute_edit_footprint,
    select_recheck_set,
};

pub fn select_frame_aware_recheck_set(
    frames: &[DiagnosticFrameFootprintV0],
    edited_module_ids: Vec<String>,
) -> RecheckSelectionV0 {
    let footprint: ModuleFootprintV0 = compute_edit_footprint(edited_module_ids);
    select_recheck_set(frames, &footprint)
}

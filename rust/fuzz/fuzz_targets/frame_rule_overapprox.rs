#![no_main]

use std::collections::BTreeSet;

use libfuzzer_sys::fuzz_target;
use omena_cascade::{
    compute_edit_footprint, derive_frames_for_diagnostic_set, intersect_frame_with_footprint,
    select_recheck_set,
};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let frame_count = usize::from(data[0] % 32) + 1;
    let module_modulus = usize::from(data.get(1).copied().unwrap_or(7) % 16) + 1;
    let mut diagnostics = Vec::with_capacity(frame_count);
    for frame_index in 0..frame_count {
        let seed = data.get(frame_index + 2).copied().unwrap_or(frame_index as u8);
        let module_count = usize::from(seed % 4) + 1;
        let module_ids = (0..module_count)
            .map(|module_index| {
                let module_id = (usize::from(seed) + frame_index + module_index) % module_modulus;
                format!("file:///workspace/module-{module_id}.module.css")
            })
            .collect::<Vec<_>>();
        diagnostics.push((
            "missing-static-class".to_string(),
            format!("diagnostic-{frame_index}"),
            module_ids,
        ));
    }

    let edit_seed = data.get(frame_count + 2).copied().unwrap_or_default();
    let edit_count = usize::from(edit_seed % 4) + 1;
    let edited_module_ids = (0..edit_count)
        .map(|index| {
            let module_id = (usize::from(edit_seed) + index) % module_modulus;
            format!("file:///workspace/module-{module_id}.module.css")
        })
        .collect::<Vec<_>>();

    let frames = derive_frames_for_diagnostic_set(diagnostics);
    let footprint = compute_edit_footprint(edited_module_ids);
    let selection = select_recheck_set(&frames, &footprint);
    let selected = selection
        .selected_diagnostic_instance_ids
        .iter()
        .collect::<BTreeSet<_>>();
    let skipped = selection
        .skipped_diagnostic_instance_ids
        .iter()
        .collect::<BTreeSet<_>>();

    assert!(selection.conservative);
    assert!(frames.iter().all(|frame| frame.conservative));
    assert!(frames.iter().all(|frame| {
        frame.evidence_module_ids == {
            let mut sorted = frame.evidence_module_ids.clone();
            sorted.sort();
            sorted.dedup();
            sorted
        }
    }));

    for frame in &frames {
        let intersects = intersect_frame_with_footprint(frame, &footprint);
        let id = &frame.diagnostic_instance_id;
        assert_ne!(selected.contains(id), skipped.contains(id));
        if intersects {
            assert!(selected.contains(id));
        } else {
            assert!(skipped.contains(id));
        }
    }
});

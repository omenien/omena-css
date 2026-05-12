#![no_main]

use libfuzzer_sys::fuzz_target;
use omena_transform_passes::{
    TransformCascadeSafetyFuzzCaseV0, run_transform_cascade_safe_fuzz_case,
};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let result = run_transform_cascade_safe_fuzz_case(TransformCascadeSafetyFuzzCaseV0 {
        seed: read_u64(data, 0),
        pass_count: usize::from(data[0]) + 1,
    });

    assert!(result.passed);
});

fn read_u64(data: &[u8], offset: usize) -> u64 {
    let mut bytes = [0_u8; 8];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = data.get(offset + index).copied().unwrap_or_default();
    }
    u64::from_le_bytes(bytes)
}

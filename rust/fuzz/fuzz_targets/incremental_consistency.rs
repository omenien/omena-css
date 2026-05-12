#![no_main]

use libfuzzer_sys::fuzz_target;
use omena_incremental::{IncrementalConsistencyFuzzCaseV0, run_incremental_consistency_fuzz_case};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let node_count = usize::from(data[0]) + 1;
    let changed_node_index = data.get(1).map(|value| usize::from(*value) % node_count);
    let result = run_incremental_consistency_fuzz_case(IncrementalConsistencyFuzzCaseV0 {
        seed: read_u64(data, 2),
        node_count,
        changed_node_index,
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

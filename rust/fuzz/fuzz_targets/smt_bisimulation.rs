#![no_main]

use libfuzzer_sys::fuzz_target;
use omena_smt::{SmtBisimulationFuzzCaseV0, run_smt_bisimulation_fuzz_case_v0};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let report = run_smt_bisimulation_fuzz_case_v0(SmtBisimulationFuzzCaseV0 {
        seed: read_u64(data, 0),
    });

    assert!(report.passed);
});

fn read_u64(data: &[u8], offset: usize) -> u64 {
    let mut bytes = [0_u8; 8];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = data.get(offset + index).copied().unwrap_or_default();
    }
    u64::from_le_bytes(bytes)
}

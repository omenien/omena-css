#![no_main]

use libfuzzer_sys::fuzz_target;
use omena_cascade::{VarSubstitutionFuzzCaseV0, run_var_substitution_fuzz_case};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let result = run_var_substitution_fuzz_case(VarSubstitutionFuzzCaseV0 {
        seed: read_u64(data, 0),
        chain_len: usize::from(data[0]) + 1,
        cycle: data.get(1).copied().unwrap_or_default().is_multiple_of(2),
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

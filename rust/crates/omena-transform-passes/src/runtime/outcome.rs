//! Outcome constructors for transform pass runtime dispatch.
//!
//! The executor owns pass ordering, while this module owns the small status
//! policy used to turn mutation counts and guard failures into public V0
//! execution outcomes.

use crate::{
    TransformPassExecutionOutcomeV0,
    TransformPassRuntimeStatus::{self, Applied, NoChange, PlannedOnly},
};

pub(crate) fn mutation_outcome(
    pass_id: &'static str,
    input_byte_len: usize,
    output_byte_len: usize,
    mutation_count: usize,
    detail: &'static str,
) -> TransformPassExecutionOutcomeV0 {
    TransformPassExecutionOutcomeV0 {
        pass_id,
        status: mutation_status(mutation_count),
        input_byte_len,
        output_byte_len,
        mutation_count,
        provenance_preserved: true,
        detail,
    }
}

pub(crate) fn no_change_outcome(
    pass_id: &'static str,
    input_byte_len: usize,
    output_byte_len: usize,
    detail: &'static str,
) -> TransformPassExecutionOutcomeV0 {
    TransformPassExecutionOutcomeV0 {
        pass_id,
        status: NoChange,
        input_byte_len,
        output_byte_len,
        mutation_count: 0,
        provenance_preserved: true,
        detail,
    }
}

pub(crate) fn planned_only_outcome(
    pass_id: &'static str,
    input_byte_len: usize,
    output_byte_len: usize,
    detail: &'static str,
) -> TransformPassExecutionOutcomeV0 {
    TransformPassExecutionOutcomeV0 {
        pass_id,
        status: PlannedOnly,
        input_byte_len,
        output_byte_len,
        mutation_count: 0,
        provenance_preserved: true,
        detail,
    }
}

fn mutation_status(mutation_count: usize) -> TransformPassRuntimeStatus {
    if mutation_count == 0 {
        NoChange
    } else {
        Applied
    }
}

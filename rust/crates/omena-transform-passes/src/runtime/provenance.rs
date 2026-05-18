use crate::{
    TransformPassExecutionOutcomeV0, TransformProvenanceDerivationForestV0,
    TransformProvenanceDerivationNodeV0, TransformProvenanceMutationSpanV0,
};

pub(crate) fn provenance_derivation_forest_from_outcomes(
    outcomes: &[TransformPassExecutionOutcomeV0],
    outcome_mutation_spans: &[Vec<TransformProvenanceMutationSpanV0>],
) -> TransformProvenanceDerivationForestV0 {
    let nodes = outcomes
        .iter()
        .enumerate()
        .map(|(index, outcome)| {
            let mutation_spans = outcome_mutation_spans
                .get(index)
                .cloned()
                .unwrap_or_default();
            let (source_span_start, source_span_end, generated_span_start, generated_span_end) =
                provenance_node_span_envelope(
                    outcome.input_byte_len,
                    outcome.output_byte_len,
                    mutation_spans.as_slice(),
                );

            TransformProvenanceDerivationNodeV0 {
                node_index: index,
                parent_index: index.checked_sub(1),
                pass_id: outcome.pass_id,
                status: outcome.status,
                input_byte_len: outcome.input_byte_len,
                output_byte_len: outcome.output_byte_len,
                source_span_start,
                source_span_end,
                generated_span_start,
                generated_span_end,
                mutation_spans,
                mutation_count: outcome.mutation_count,
                provenance_preserved: outcome.provenance_preserved,
                detail: outcome.detail,
            }
        })
        .collect::<Vec<_>>();

    TransformProvenanceDerivationForestV0 {
        schema_version: "0",
        product: "omena-transform-passes.provenance-derivation-forest",
        root_count: usize::from(!nodes.is_empty()),
        node_count: nodes.len(),
        nodes,
    }
}

fn provenance_node_span_envelope(
    input_byte_len: usize,
    output_byte_len: usize,
    mutation_spans: &[TransformProvenanceMutationSpanV0],
) -> (usize, usize, usize, usize) {
    if mutation_spans.is_empty() {
        return (0, input_byte_len, 0, output_byte_len);
    }

    let source_span_start = mutation_spans
        .iter()
        .map(|span| span.source_span_start)
        .min()
        .unwrap_or(0);
    let source_span_end = mutation_spans
        .iter()
        .map(|span| span.source_span_end)
        .max()
        .unwrap_or(input_byte_len);
    let generated_span_start = mutation_spans
        .iter()
        .map(|span| span.generated_span_start)
        .min()
        .unwrap_or(0);
    let generated_span_end = mutation_spans
        .iter()
        .map(|span| span.generated_span_end)
        .max()
        .unwrap_or(output_byte_len);

    (
        source_span_start,
        source_span_end,
        generated_span_start,
        generated_span_end,
    )
}

pub(crate) fn derive_transform_mutation_spans(
    input: &str,
    output: &str,
) -> Vec<TransformProvenanceMutationSpanV0> {
    if input == output {
        return Vec::new();
    }

    let input_line_spans = line_spans(input);
    let output_line_spans = line_spans(output);
    if input_line_spans.len() == output_line_spans.len() {
        let spans = input_line_spans
            .iter()
            .zip(output_line_spans.iter())
            .filter_map(
                |(&(source_start, source_end), &(generated_start, generated_end))| {
                    derive_changed_slice_mutation_span(
                        input,
                        output,
                        source_start,
                        source_end,
                        generated_start,
                        generated_end,
                    )
                },
            )
            .collect::<Vec<_>>();
        if !spans.is_empty() {
            return spans;
        }
    }

    let prefix = common_prefix_byte_len(input.as_bytes(), output.as_bytes());
    let suffix = common_suffix_byte_len(input.as_bytes(), output.as_bytes(), prefix);
    vec![TransformProvenanceMutationSpanV0 {
        source_span_start: prefix,
        source_span_end: input.len().saturating_sub(suffix),
        generated_span_start: prefix,
        generated_span_end: output.len().saturating_sub(suffix),
    }]
}

fn derive_changed_slice_mutation_span(
    input: &str,
    output: &str,
    source_start: usize,
    source_end: usize,
    generated_start: usize,
    generated_end: usize,
) -> Option<TransformProvenanceMutationSpanV0> {
    let source_slice = &input[source_start..source_end];
    let generated_slice = &output[generated_start..generated_end];
    if source_slice == generated_slice {
        return None;
    }

    let prefix = common_prefix_byte_len(source_slice.as_bytes(), generated_slice.as_bytes());
    let suffix =
        common_suffix_byte_len(source_slice.as_bytes(), generated_slice.as_bytes(), prefix);

    Some(TransformProvenanceMutationSpanV0 {
        source_span_start: source_start + prefix,
        source_span_end: source_end.saturating_sub(suffix),
        generated_span_start: generated_start + prefix,
        generated_span_end: generated_end.saturating_sub(suffix),
    })
}

fn line_spans(source: &str) -> Vec<(usize, usize)> {
    if source.is_empty() {
        return vec![(0, 0)];
    }

    let mut spans = Vec::new();
    let mut start = 0usize;
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            let end = index + 1;
            spans.push((start, end));
            start = end;
        }
    }

    if start < source.len() {
        spans.push((start, source.len()));
    }

    spans
}

fn common_prefix_byte_len(left: &[u8], right: &[u8]) -> usize {
    left.iter()
        .zip(right.iter())
        .take_while(|(left, right)| left == right)
        .count()
}

fn common_suffix_byte_len(left: &[u8], right: &[u8], prefix_len: usize) -> usize {
    let mut suffix_len = 0usize;
    while left.len() > prefix_len + suffix_len
        && right.len() > prefix_len + suffix_len
        && left[left.len() - suffix_len - 1] == right[right.len() - suffix_len - 1]
    {
        suffix_len += 1;
    }
    suffix_len
}

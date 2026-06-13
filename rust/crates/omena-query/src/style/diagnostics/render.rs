use crate::{ParserPositionV0, ParserRangeV0};

/// Reduce a cycle `path` (a node ring whose first and last entries repeat, e.g. `[a, b, a]`) to a
/// rotation-invariant key: drop the repeated closing node, then return the lexicographically
/// smallest rotation. Two rotations of the same loop (`[a, b, a]` / `[b, a, b]`) collapse to one
/// key, so each distinct loop is surfaced exactly once per anchoring edge. A self-loop `[a, a]`
/// reduces to `[a]`.
pub(super) fn canonical_sass_module_cycle(path: &[String]) -> Vec<String> {
    let ring: &[String] = match path.split_last() {
        Some((last, head)) if Some(last) == path.first() && !head.is_empty() => head,
        _ => path,
    };
    if ring.is_empty() {
        return path.to_vec();
    }
    let len = ring.len();
    (0..len)
        .map(|offset| {
            (0..len)
                .map(|index| ring[(offset + index) % len].clone())
                .collect::<Vec<_>>()
        })
        .min()
        .unwrap_or_else(|| ring.to_vec())
}

/// Render a canonical cycle ring as a closed `start -> … -> start` path beginning at `start`, so
/// each participating file describes the loop from its own perspective. `start` is guaranteed to be
/// in the ring by the caller (the target participates in the cycle).
pub(super) fn render_sass_module_cycle_from(canonical_cycle: &[String], start: &str) -> String {
    let len = canonical_cycle.len();
    let begin = canonical_cycle
        .iter()
        .position(|node| node == start)
        .unwrap_or(0);
    let mut ordered = (0..len)
        .map(|index| canonical_cycle[(begin + index) % len].clone())
        .collect::<Vec<_>>();
    // Re-close the ring so the loop reads `a -> b -> a` (or `a -> a` for a self-loop).
    ordered.push(canonical_cycle[begin].clone());
    ordered.join(" -> ")
}

pub(super) fn whole_file_omena_query_style_range(source: &str) -> ParserRangeV0 {
    let mut line = 0usize;
    let mut character = 0usize;
    for ch in source.chars() {
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }
    ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 { line, character },
    }
}

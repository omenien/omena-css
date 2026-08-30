use omena_query::{ParserPositionV0, ParserRangeV0};
use omena_syntax::OmenaLineIndexV0;

pub(crate) fn apply_text_edit(
    source: &str,
    range: ParserRangeV0,
    new_text: &str,
) -> Result<String, String> {
    let line_index = OmenaLineIndexV0::new(source);
    let start = byte_offset_for_position_with_line_index(source, &line_index, range.start)
        .ok_or_else(|| "edit start position is outside the target source".to_string())?;
    let end = byte_offset_for_position_with_line_index(source, &line_index, range.end)
        .ok_or_else(|| "edit end position is outside the target source".to_string())?;
    apply_byte_edit(source, start, end, new_text)
}

pub(crate) fn apply_byte_edit(
    source: &str,
    start: usize,
    end: usize,
    new_text: &str,
) -> Result<String, String> {
    if start > end {
        return Err("edit range is reversed".to_string());
    }
    if !source.is_char_boundary(start) || !source.is_char_boundary(end) || end > source.len() {
        return Err("edit range is outside a UTF-8 boundary".to_string());
    }
    let mut edited = String::with_capacity(source.len() - (end - start) + new_text.len());
    edited.push_str(&source[..start]);
    edited.push_str(new_text);
    edited.push_str(&source[end..]);
    Ok(edited)
}

pub(crate) fn byte_span_for_range(source: &str, range: ParserRangeV0) -> Option<(usize, usize)> {
    let line_index = OmenaLineIndexV0::new(source);
    byte_span_for_range_with_line_index(source, &line_index, range)
}

pub(crate) fn byte_span_for_range_with_line_index(
    source: &str,
    line_index: &OmenaLineIndexV0,
    range: ParserRangeV0,
) -> Option<(usize, usize)> {
    let start = byte_offset_for_position_with_line_index(source, line_index, range.start)?;
    let end = byte_offset_for_position_with_line_index(source, line_index, range.end)?;
    (start <= end).then_some((start, end))
}

pub(crate) fn range_for_byte_span(source: &str, start: usize, end: usize) -> Option<ParserRangeV0> {
    let line_index = OmenaLineIndexV0::new(source);
    range_for_byte_span_with_line_index(source, &line_index, start, end)
}

pub(crate) fn range_for_byte_span_with_line_index(
    source: &str,
    line_index: &OmenaLineIndexV0,
    start: usize,
    end: usize,
) -> Option<ParserRangeV0> {
    if start > end
        || end > source.len()
        || !source.is_char_boundary(start)
        || !source.is_char_boundary(end)
    {
        return None;
    }
    Some(ParserRangeV0 {
        start: position_for_byte_offset_with_line_index(source, line_index, start),
        end: position_for_byte_offset_with_line_index(source, line_index, end),
    })
}

#[cfg(test)]
pub(crate) fn byte_offset_for_position(source: &str, position: ParserPositionV0) -> Option<usize> {
    let line_index = OmenaLineIndexV0::new(source);
    byte_offset_for_position_with_line_index(source, &line_index, position)
}

#[cfg(test)]
fn position_for_byte_offset(source: &str, offset: usize) -> ParserPositionV0 {
    let line_index = OmenaLineIndexV0::new(source);
    position_for_byte_offset_with_line_index(source, &line_index, offset)
}

fn byte_offset_for_position_with_line_index(
    source: &str,
    line_index: &OmenaLineIndexV0,
    position: ParserPositionV0,
) -> Option<usize> {
    line_index.byte_offset_for_position(source, position.line, position.character)
}

fn position_for_byte_offset_with_line_index(
    source: &str,
    line_index: &OmenaLineIndexV0,
    offset: usize,
) -> ParserPositionV0 {
    let (line, character) = line_index.position_for_byte_offset(source, offset);
    ParserPositionV0 { line, character }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_ranges_edit_non_ascii_sources() -> Result<(), String> {
        let source = ".표시 { color: red; }\n";
        let range = ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 1,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 3,
            },
        };
        assert_eq!(
            apply_text_edit(source, range, "card")?,
            ".card { color: red; }\n"
        );
        Ok(())
    }

    #[test]
    fn byte_spans_round_trip_through_utf16_ranges() -> Result<(), String> {
        let source = "한글 .button { color: red; }\n";
        let start = source
            .find("button")
            .ok_or_else(|| "fixture token is missing".to_string())?;
        let end = start + "button".len();
        let range = range_for_byte_span(source, start, end)
            .ok_or_else(|| "fixture byte span is invalid".to_string())?;
        assert_eq!(position_for_byte_offset(source, start), range.start);
        assert_eq!(byte_offset_for_position(source, range.start), Some(start));
        assert_eq!(byte_span_for_range(source, range), Some((start, end)));
        Ok(())
    }
}

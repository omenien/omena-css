use omena_query::{ParserPositionV0, ParserRangeV0};

pub(crate) fn apply_text_edit(
    source: &str,
    range: ParserRangeV0,
    new_text: &str,
) -> Result<String, String> {
    let start = byte_offset_for_position(source, range.start)
        .ok_or_else(|| "edit start position is outside the target source".to_string())?;
    let end = byte_offset_for_position(source, range.end)
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
    let start = byte_offset_for_position(source, range.start)?;
    let end = byte_offset_for_position(source, range.end)?;
    (start <= end).then_some((start, end))
}

pub(crate) fn range_for_byte_span(source: &str, start: usize, end: usize) -> Option<ParserRangeV0> {
    if start > end
        || end > source.len()
        || !source.is_char_boundary(start)
        || !source.is_char_boundary(end)
    {
        return None;
    }
    Some(ParserRangeV0 {
        start: position_for_byte_offset(source, start),
        end: position_for_byte_offset(source, end),
    })
}

pub(crate) fn byte_offset_for_position(source: &str, position: ParserPositionV0) -> Option<usize> {
    let mut line = 0;
    let mut line_start = 0;
    for (offset, character) in source.char_indices() {
        if line == position.line {
            line_start = offset;
            break;
        }
        if character == '\n' {
            line += 1;
            line_start = offset + character.len_utf8();
        }
    }
    if line != position.line {
        if position.line == line && line_start == source.len() {
            return (position.character == 0).then_some(source.len());
        }
        return None;
    }
    let line_source = source[line_start..]
        .split_once('\n')
        .map_or(&source[line_start..], |(line_source, _)| line_source);
    let mut utf16_offset = 0;
    for (byte_offset, character) in line_source.char_indices() {
        if utf16_offset == position.character {
            return Some(line_start + byte_offset);
        }
        utf16_offset += character.len_utf16();
        if utf16_offset > position.character {
            return None;
        }
    }
    (utf16_offset == position.character).then_some(line_start + line_source.len())
}

fn position_for_byte_offset(source: &str, offset: usize) -> ParserPositionV0 {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let character = source[line_start..offset].encode_utf16().count();
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
        assert_eq!(byte_span_for_range(source, range), Some((start, end)));
        Ok(())
    }
}

/// A reusable UTF-8 byte-offset to UTF-16 line/column index.
///
/// The index stores only line starts. Conversions use a binary search followed
/// by a walk within the selected line, so repeated conversions never rescan the
/// source prefix from byte zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaLineIndexV0 {
    line_starts: Vec<u32>,
}

impl OmenaLineIndexV0 {
    pub fn new(source: &str) -> Self {
        assert!(
            u32::try_from(source.len()).is_ok(),
            "OmenaLineIndexV0 sources must fit in the parser's u32 offset space"
        );
        let mut line_starts = vec![0];
        for (index, byte) in source.as_bytes().iter().enumerate() {
            if *byte == b'\n' {
                let Ok(line_start) = u32::try_from(index + 1) else {
                    unreachable!("source length was validated against the u32 offset space");
                };
                line_starts.push(line_start);
            }
        }
        Self { line_starts }
    }

    pub fn position_for_byte_offset(&self, source: &str, byte_offset: usize) -> (usize, usize) {
        let offset = byte_offset.min(source.len());
        let line = self
            .line_starts
            .partition_point(|start| (*start as usize) <= offset);
        let line_index = line.saturating_sub(1);
        let line_start = self
            .line_starts
            .get(line_index)
            .copied()
            .map(|start| start as usize)
            .unwrap_or(0);
        let character = source
            .get(line_start..offset)
            .map(|text| text.encode_utf16().count())
            .unwrap_or_else(|| offset.saturating_sub(line_start));
        (line_index, character)
    }

    pub fn byte_offset_for_position(
        &self,
        source: &str,
        line: usize,
        character: usize,
    ) -> Option<usize> {
        let line_start = self.line_starts.get(line).copied()? as usize;
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .map(|next_start| (next_start as usize).saturating_sub(1))
            .unwrap_or(source.len());
        let line_source = source.get(line_start..line_end)?;
        let mut utf16_offset = 0usize;
        for (byte_offset, value) in line_source.char_indices() {
            if utf16_offset == character {
                return Some(line_start + byte_offset);
            }
            utf16_offset += value.len_utf16();
            if utf16_offset > character {
                return None;
            }
        }
        (utf16_offset == character).then_some(line_start + line_source.len())
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::OmenaLineIndexV0;

    fn legacy_position_for_byte_offset(source: &str, byte_offset: usize) -> (usize, usize) {
        let clamped_offset = byte_offset.min(source.len());
        let mut line = 0usize;
        let mut character = 0usize;
        for (index, value) in source.char_indices() {
            if index >= clamped_offset {
                break;
            }
            if value == '\n' {
                line += 1;
                character = 0;
            } else {
                character += value.len_utf16();
            }
        }
        (line, character)
    }

    fn legacy_byte_offset_for_position(
        source: &str,
        target_line: usize,
        target_character: usize,
    ) -> Option<usize> {
        let mut line = 0usize;
        let mut character = 0usize;
        for (byte_offset, value) in source.char_indices() {
            if line == target_line && character == target_character {
                return Some(byte_offset);
            }
            if value == '\n' {
                if line == target_line {
                    return None;
                }
                line += 1;
                character = 0;
            } else {
                character += value.len_utf16();
                if line == target_line && character > target_character {
                    return None;
                }
            }
        }
        (line == target_line && character == target_character).then_some(source.len())
    }

    #[test]
    fn maps_utf8_bytes_and_utf16_positions_in_both_directions() {
        let source = "ascii\n한😀z\n";
        let index = OmenaLineIndexV0::new(source);
        assert_eq!(index.line_count(), 3);

        for byte_offset in source
            .char_indices()
            .map(|(offset, _)| offset)
            .chain(std::iter::once(source.len()))
        {
            let (line, character) = index.position_for_byte_offset(source, byte_offset);
            assert_eq!(
                index.byte_offset_for_position(source, line, character),
                Some(byte_offset)
            );
        }

        assert_eq!(index.byte_offset_for_position(source, 1, 2), None);
        assert_eq!(index.byte_offset_for_position(source, 99, 0), None);
    }

    #[test]
    fn remains_byte_identical_to_the_legacy_scanners_on_the_corpus() {
        let corpus = [
            "",
            "plain ascii",
            "first\nsecond\n",
            "한글 .표시 { 값: 😀; }\n다음 줄",
            "crlf\r\nline\r\n",
        ];
        for source in corpus {
            let index = OmenaLineIndexV0::new(source);
            let offsets = source
                .char_indices()
                .map(|(offset, _)| offset)
                .chain(std::iter::once(source.len()));
            for byte_offset in offsets {
                let expected = legacy_position_for_byte_offset(source, byte_offset);
                assert_eq!(
                    index.position_for_byte_offset(source, byte_offset),
                    expected,
                    "forward mismatch for {source:?} at byte {byte_offset}"
                );
                assert_eq!(
                    index.byte_offset_for_position(source, expected.0, expected.1),
                    legacy_byte_offset_for_position(source, expected.0, expected.1),
                    "reverse mismatch for {source:?} at position {expected:?}"
                );
            }
        }
    }
}

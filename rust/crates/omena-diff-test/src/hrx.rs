use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HrxArchiveV0 {
    members: Vec<HrxMemberV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HrxMemberV0 {
    path: String,
    delimiter: Vec<u8>,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HrxParseErrorV0 {
    EmptyArchive,
    DuplicatePath(String),
    EmptyMemberPath { offset: usize },
    InvalidMemberPathUtf8 { offset: usize },
    MissingLeadingDelimiter,
}

impl HrxArchiveV0 {
    pub fn parse(input: &[u8]) -> Result<Self, HrxParseErrorV0> {
        parse_hrx_archive_v0(input)
    }

    pub fn members(&self) -> &[HrxMemberV0] {
        self.members.as_slice()
    }

    pub fn member_bytes(&self, path: &str) -> Option<&[u8]> {
        self.members
            .iter()
            .find(|member| member.path == path)
            .map(|member| member.bytes.as_slice())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        for member in &self.members {
            output.extend_from_slice(member.delimiter.as_slice());
            output.extend_from_slice(member.bytes.as_slice());
        }
        output
    }
}

impl HrxMemberV0 {
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

pub fn parse_hrx_archive_v0(input: &[u8]) -> Result<HrxArchiveV0, HrxParseErrorV0> {
    if input.is_empty() {
        return Err(HrxParseErrorV0::EmptyArchive);
    }

    let mut cursor = 0;
    let mut current: Option<OpenMemberV0> = None;
    let mut members = Vec::new();
    let mut seen_paths = BTreeSet::new();

    while cursor < input.len() {
        let line_start = cursor;
        let line_end = next_line_end(input, line_start);
        let line = &input[line_start..line_end];
        let line_body = trim_line_ending(line);

        if line_body.starts_with(b"<===>") {
            if let Some(open) = current.take() {
                members.push(open.finish(input, line_start));
            } else if line_start != 0 {
                return Err(HrxParseErrorV0::MissingLeadingDelimiter);
            }

            let path = parse_member_path(&line_body[5..], line_start)?;
            if !seen_paths.insert(path.clone()) {
                return Err(HrxParseErrorV0::DuplicatePath(path));
            }

            current = Some(OpenMemberV0 {
                path,
                delimiter: line.to_vec(),
                content_start: line_end,
            });
        } else if current.is_none() {
            return Err(HrxParseErrorV0::MissingLeadingDelimiter);
        }

        cursor = line_end;
    }

    let Some(open) = current else {
        return Err(HrxParseErrorV0::EmptyArchive);
    };
    members.push(open.finish(input, input.len()));

    Ok(HrxArchiveV0 { members })
}

#[derive(Debug)]
struct OpenMemberV0 {
    path: String,
    delimiter: Vec<u8>,
    content_start: usize,
}

impl OpenMemberV0 {
    fn finish(self, input: &[u8], content_end: usize) -> HrxMemberV0 {
        HrxMemberV0 {
            path: self.path,
            delimiter: self.delimiter,
            bytes: input[self.content_start..content_end].to_vec(),
        }
    }
}

fn next_line_end(input: &[u8], start: usize) -> usize {
    input[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(input.len(), |newline_offset| start + newline_offset + 1)
}

fn trim_line_ending(line: &[u8]) -> &[u8] {
    let without_lf = line.strip_suffix(b"\n").unwrap_or(line);
    without_lf.strip_suffix(b"\r").unwrap_or(without_lf)
}

fn parse_member_path(raw: &[u8], offset: usize) -> Result<String, HrxParseErrorV0> {
    let trimmed = trim_ascii_horizontal(raw);
    if trimmed.is_empty() {
        return Err(HrxParseErrorV0::EmptyMemberPath { offset });
    }
    std::str::from_utf8(trimmed)
        .map(str::to_owned)
        .map_err(|_| HrxParseErrorV0::InvalidMemberPathUtf8 { offset })
}

fn trim_ascii_horizontal(mut bytes: &[u8]) -> &[u8] {
    while bytes
        .first()
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        bytes = &bytes[1..];
    }
    while bytes
        .last()
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::{HrxArchiveV0, HrxParseErrorV0};

    const CLOSED_ISSUE_ARCHIVE: &[u8] =
        include_bytes!("../fixtures/sass-spec-import/spec/libsass-closed-issues/issue_992.hrx");

    #[test]
    fn hrx_archive_round_trip_preserves_member_bytes() -> Result<(), HrxParseErrorV0> {
        let archive = HrxArchiveV0::parse(CLOSED_ISSUE_ARCHIVE)?;
        assert_eq!(archive.to_bytes(), CLOSED_ISSUE_ARCHIVE);
        assert_eq!(
            archive
                .members()
                .iter()
                .map(|member| member.path())
                .collect::<Vec<_>>(),
            ["input.scss", "output.css"],
        );
        assert_eq!(
            archive.member_bytes("input.scss"),
            Some(b"$color: 'red';\n\n.-text-#{$color}- {\n  color: $color;\n}\n".as_slice()),
        );
        assert_eq!(
            archive.member_bytes("output.css"),
            Some(b".-text-red- {\n  color: \"red\";\n}\n".as_slice()),
        );
        Ok(())
    }

    #[test]
    fn hrx_archive_rejects_duplicate_member_paths() {
        assert_eq!(
            HrxArchiveV0::parse(b"<===> input.scss\n.a{}\n<===> input.scss\n.b{}\n"),
            Err(HrxParseErrorV0::DuplicatePath("input.scss".to_string()))
        );
    }

    #[test]
    fn hrx_archive_requires_a_leading_delimiter() {
        assert_eq!(
            HrxArchiveV0::parse(b"preamble\n<===> input.scss\n.a{}\n"),
            Err(HrxParseErrorV0::MissingLeadingDelimiter)
        );
    }
}

use cstree::text::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssFact {
    pub kind: ParsedIcssFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedIcssFactKind {
    ExportName,
    ImportLocalName,
    ImportRemoteName,
    ImportSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssImportEdgeFact {
    pub local_name: String,
    pub remote_name: String,
    pub import_source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIcssExportEdgeFact {
    pub export_name: String,
    pub reference_names: Vec<String>,
    pub range: TextRange,
}

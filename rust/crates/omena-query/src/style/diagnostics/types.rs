pub(super) const LSP_DIAGNOSTIC_TAG_UNNECESSARY: u8 = 1;
pub(super) const LSP_DIAGNOSTIC_TAG_DEPRECATED: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaQueryExternalModuleModeV0 {
    Ignored,
    Sif,
    Auto,
}

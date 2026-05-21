use omena_meta_macros::{pass, spec};

#[spec(webref = "css-color/properties/color", priority = "P0")]
const CSS_COLOR_SPEC_MARKER: &str = "color";

#[spec(na = "print-margin-descriptor")]
const PRINT_MARGIN_DESCRIPTOR_MARKER: &str = "manual";

#[pass(id = "color-compression", ordinal = 5, layer = "value-normalization")]
fn color_compression_pass_marker() -> &'static str {
    "color-compression"
}

#[test]
fn valid_metadata_attributes_compile_on_supported_items() {
    assert_eq!(CSS_COLOR_SPEC_MARKER, "color");
    assert_eq!(PRINT_MARGIN_DESCRIPTOR_MARKER, "manual");
    assert_eq!(color_compression_pass_marker(), "color-compression");
}

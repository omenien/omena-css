fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!(
        "{}",
        omena_diff_test::render_sass_spec_bail_site_product_view_json()?
    );
    Ok(())
}

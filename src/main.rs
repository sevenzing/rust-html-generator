use rs_html::{run_report_generator, Settings};

fn main() -> Result<(), anyhow::Error> {
    let settings = Settings::new();
    run_report_generator(&settings)?;
    Ok(())
}

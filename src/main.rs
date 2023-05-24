use clap::Parser;
use rs_html::{run_report_generator, Settings};

fn main() -> Result<(), anyhow::Error> {
    let settings = Settings::parse();
    run_report_generator(&settings)?;
    Ok(())
}

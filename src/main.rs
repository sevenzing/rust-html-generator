use clap::Parser;
use rs_html::{app::run_report_generator, args::Settings};

fn main() -> Result<(), anyhow::Error> {
    let settings = Settings::parse();
    run_report_generator(&settings)?;
    Ok(())
}

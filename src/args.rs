use std::path::PathBuf;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Settings {
    #[clap(short, long, value_parser)]
    pub dir: PathBuf,

    #[clap(short, long, value_parser)]
    pub project_name: String,

    #[clap(short, long, value_parser, default_value = "output.html")]
    pub output: String,

    #[clap(short, long, value_parser, default_value_t = false)]
    pub scan_whole: bool,

    #[clap(short, long, value_parser, default_value_t = false)]
    pub no_compress: bool,
}

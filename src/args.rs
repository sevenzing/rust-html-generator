use clap::Parser;
use std::{fs, path::PathBuf};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Settings {
    #[clap(short, long, value_parser)]
    pub dir: PathBuf,

    #[clap(short, long, value_parser, default_value = "")]
    pub project_name: String,

    #[clap(short, long, value_parser, default_value = "output.html")]
    pub output: String,

    #[clap(short, long, value_parser, default_value_t = false)]
    pub scan_whole: bool,

    #[clap(short, long, value_parser, default_value_t = false)]
    pub no_compress: bool,
}

impl Settings {
    pub fn new() -> Self {
        let mut settings = Self::parse();
        settings.dir = fs::canonicalize(&settings.dir).expect("cannot convert to absolute path");
        let project_name = settings
            .dir
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("not a dir"))
            .unwrap()
            .to_string_lossy()
            .to_string();
        settings.project_name = project_name;
        settings
    }
}

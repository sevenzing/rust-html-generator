mod compressor;
mod html_generator;
mod html_processor;
mod report_generator;
pub mod static_files;

pub use compressor::compress_html;
pub use html_generator::{generate_other_file_html, generate_rust_file_html};
pub use html_processor::{HtmlProcessor, HtmlToken};
pub use report_generator::{generate_report, MyPath};

mod compressor;
mod html_processor;
mod generators;
pub mod static_files;

pub use compressor::compress_html;
pub use generators::*;
pub use html_processor::{HtmlProcessor, HtmlToken};

mod compressor;
mod generators;
pub mod static_files;
mod syntax_processor;

pub use compressor::compress_html;
pub use generators::*;
pub use syntax_processor::{HtmlToken, SyntaxProcessor};

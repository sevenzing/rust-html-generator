mod html_processor;
mod html_generator;

pub use html_processor::{HtmlProcessor, HtmlToken};
pub use html_generator::{generate_other_file_html, generate_rust_file_html};

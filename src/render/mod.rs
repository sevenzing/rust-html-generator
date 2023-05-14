mod render;
mod traverse;
use crate::jumps::Jumps;
use ide::{HoverResult, TextRange};
pub use render::{highlight_other_as_html, highlight_rust_file_as_html};

#[derive(Debug, Default)]
pub struct HtmlToken {
    pub is_new_line: bool,
    pub range: TextRange,
    pub highlight: Option<String>,
    pub hover_info: Option<HoverResult>,
    pub type_info: Option<String>,
    pub jumps: Option<Jumps>,
}

impl HtmlToken {
    pub fn from_empty_info(range: TextRange, is_new_line: bool) -> Self {
        Self {
            range,
            is_new_line,
            ..Default::default()
        }
    }
}

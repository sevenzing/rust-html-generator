mod render;
mod traverse;
use crate::jumps::Jumps;
use ide::{HoverResult, TextRange};
pub use render::{highlight_other_as_html, highlight_rust_file_as_html};
use syntax::SyntaxToken;

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
    pub fn from_range(range: TextRange) -> Self {
        Self {
            range,
            ..Default::default()
        }
    }
}

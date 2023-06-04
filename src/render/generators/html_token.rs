use ide::{HoverResult, LineIndex};
use serde::{self, Serialize};
use serde_with::serde_as;
use std::{fmt::Display, sync::Arc};
use syntax::TextRange;

#[derive(Debug, Default)]
pub struct HtmlToken {
    pub is_new_line: bool,
    pub range: TextRange,
    pub highlight: Option<String>,
    pub hover_info: Option<HoverResult>,
    pub type_info: Option<String>,
    pub navigation: Option<Navigation>,
}

#[derive(Debug, Serialize)]
pub struct LineCol {
    pub line: u32,
    pub col: u32,
}

impl From<ide::LineCol> for LineCol {
    fn from(value: ide::LineCol) -> Self {
        Self {
            line: value.line + 1,
            col: value.col,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Navigation {
    #[serde(rename = "def")]
    pub definition: JumpDestination,
    #[serde(rename = "refs")]
    pub references: Vec<JumpDestination>,
    pub from: JumpDestination,
}

#[serde_as]
#[derive(Debug, Serialize, Clone)]
pub struct JumpDestination {
    pub file: String,
    #[serde(rename = "loc")]
    pub location: JumpLocation,
}

#[derive(Debug, Serialize, Clone)]
pub struct JumpLocation {
    pub line: u32,
}

impl JumpDestination {
    pub fn from_focus(file: String, focus: &TextRange, finder: Arc<LineIndex>) -> Self {
        Self {
            file,
            location: JumpLocation::from_focus(focus, finder),
        }
    }
    pub fn new(file: String, location: JumpLocation) -> Self {
        Self { file, location }
    }
}

impl JumpLocation {
    pub fn from_focus(focus: &TextRange, finder: Arc<LineIndex>) -> Self {
        let start: LineCol = finder.line_col(focus.start()).into();
        let line = start.line;
        Self { line }
    }
}

impl HtmlToken {
    pub fn from_empty_info(range: TextRange, is_new_line: bool) -> Self {
        Self {
            range,
            is_new_line,
            ..Default::default()
        }
    }

    pub fn with_highlight(mut self, highlight: Option<String>) -> Self {
        self.highlight = highlight;
        self
    }

    pub fn render(&self, file_content: &str) -> String {
        let raw_chunk = &file_content[self.range];
        let chunk = html_escape::encode_text(raw_chunk).to_string();
        self.render_with_highlight(chunk)
    }

    fn render_with_highlight(&self, content: impl Display) -> String {
        if let Some(mut class) = self.highlight.clone() {
            let hover_info = self
                .hover_info
                .as_ref()
                .map(|h| h.markup.to_string())
                .unwrap_or_default();
            let mut hover_info = match hover_info.as_str() {
                "()" => "",
                "{unknown}" => "",
                _ => &hover_info,
            }
            .to_string();
            if hover_info.is_empty() && self.type_info.is_some() {
                hover_info = self.type_info.as_ref().unwrap().clone();
            }
            if !hover_info.is_empty() {
                hover_info = format!("<span>{}</span>", html_escape::encode_text(&hover_info))
            }

            let jump_attributes = self
                .navigation
                .as_ref()
                .map(|jump| {
                    if let Ok(jump_data) = serde_json::to_string(jump) {
                        class.push_str(" jump");
                        format!("jump-data=\"{}\"", jump_data.replace('"', "'"))
                    } else {
                        Default::default()
                    }
                })
                .unwrap_or_default();

            return format!(
                "<span class=\"hovertext {class}\" {jump_attributes}>{content}{hover_info}</span>",
            );
        };
        content.to_string()
    }
}

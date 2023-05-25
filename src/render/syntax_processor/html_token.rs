use crate::args::Settings;
use ide::{HoverResult, LineIndex};
use serde::{self, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use std::{fmt::Display, path::Path, sync::Arc};
use syntax::TextRange;
use vfs::VfsPath;

#[derive(Debug, Default)]
pub struct HtmlToken {
    pub is_new_line: bool,
    pub range: TextRange,
    pub highlight: Option<String>,
    pub hover_info: Option<HoverResult>,
    pub type_info: Option<String>,
    pub jumps: Option<Jumps>,
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

#[derive(Debug)]
pub struct Jumps {
    pub to: JumpInfo,
    pub from: JumpInfo,
}

#[serde_as]
#[derive(Debug)]
pub struct JumpInfo {
    pub file: VfsPath,
    pub location: JumpLocation,
}

#[derive(Debug, Serialize)]
pub struct JumpLocation {
    pub start: LineCol,
    pub end: LineCol,
}

impl Jumps {
    pub fn serialize(&self, root: &Path, project_name: &str) -> Result<String, anyhow::Error> {
        let content = serde_json::to_string(&serde_json::json!({
            "to": self.to.serialize(root, project_name)?,
            "from": self.from.serialize(root, project_name)?,
        }))?;
        Ok(content.replace('\"', "'"))
    }
}

impl JumpInfo {
    pub fn from_focus(file: VfsPath, focus: &TextRange, finder: Arc<LineIndex>) -> Self {
        Self {
            file,
            location: JumpLocation::from_focus(focus, finder),
        }
    }
    pub fn new(file: VfsPath, location: JumpLocation) -> Self {
        Self { file, location }
    }

    pub fn serialize(&self, root: &Path, project_name: &str) -> Result<Value, anyhow::Error> {
        let file = self.serialize_file(root, project_name)?;
        Ok(serde_json::json!({
            "file": file,
            "location": self.location,
        }))
    }

    fn serialize_file(&self, root: &Path, project_name: &str) -> Result<String, anyhow::Error> {
        relative_path(&self.file, root, project_name)
    }
}

impl JumpLocation {
    pub fn from_focus(focus: &TextRange, finder: Arc<LineIndex>) -> Self {
        let start = finder.line_col(focus.start()).into();
        let end = finder.line_col(focus.end()).into();
        Self { start, end }
    }
}

fn relative_path(vfs: &VfsPath, root: &Path, project_name: &str) -> Result<String, anyhow::Error> {
    let file_relative_path = vfs
        .as_path()
        .ok_or_else(|| anyhow::anyhow!("invalid vfs"))?
        .as_ref()
        .strip_prefix(root)?;
    let s = format!("{project_name}/{}", file_relative_path.to_string_lossy());
    Ok(s)
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

    pub fn render(&self, file_content: &str, settings: &Settings) -> String {
        let raw_chunk = &file_content[self.range];
        let chunk = html_escape::encode_text(raw_chunk).to_string();
        self.render_with_highlight(chunk, settings)
    }

    fn render_with_highlight(&self, content: impl Display, settings: &Settings) -> String {
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
                .jumps
                .as_ref()
                .map(|jump| {
                    if let Ok(jump_data) = jump.serialize(&settings.dir, &settings.project_name) {
                        class.push_str(" jump");
                        format!("jump-data=\"{}\"", jump_data)
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

use ide::{LineIndex, TextRange};
use serde::{self, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use std::{path::Path, sync::Arc};
use vfs::VfsPath;

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

#[serde_as]
#[derive(Debug)]
pub struct JumpInfo {
    pub file: VfsPath,
    pub location: JumpLocation,
}

#[derive(Debug)]
pub struct Jumps {
    pub to: JumpInfo,
    pub from: JumpInfo,
}

impl Jumps {
    pub fn serialize(&self, root: &Path, project_name: &str) -> Result<String, anyhow::Error> {
        let content = serde_json::to_string(&serde_json::json!({
            "to": self.to.serialize(root, project_name)?,
            "from": self.from.serialize(root, project_name)?,
        }))?;
        Ok(content.replace("\"", "'"))
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

#[derive(Debug, Serialize)]
pub struct JumpLocation {
    pub start: LineCol,
    pub end: LineCol,
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
    let s = format!(
        "{project_name}/{}",
        file_relative_path.to_string_lossy().to_string()
    );
    Ok(s)
}

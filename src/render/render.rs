use super::traverse::traverse_syntax;
use crate::{args::Settings, jumps::Jumps, templates::TEMPLATES};
use hir::Semantics;
use ide::{AnalysisHost, FileId, HoverResult};
use serde::Serialize;
use std::{fmt::Display, collections::HashMap};
use syntax::{ast::AstNode, TextRange};
use tera::Context;
use vfs::Vfs;

#[derive(Serialize, Clone)]
struct Line {
    number: usize,
    html_content: String,
    //fold: Vec<Fold>
    fold: Option<FoldingRange>
}

#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FoldingRange {
    start_line: u32,
    end_line: u32,
}
#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum Fold {
    Open,
    Close
}

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

    pub fn with_highlight(mut self, highlight: Option<String>) -> Self {
        self.highlight = highlight;
        self
    }

    pub fn render(&self, file_content: &str, settings: &Settings) -> String {
        let raw_chunk = &file_content[self.range];
        let chunk = html_escape::encode_text(raw_chunk).to_string();
        let chunk = self.render_with_highlight(chunk, settings);
        chunk.to_string()
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

pub fn highlight_rust_file_as_html(
    host: &AnalysisHost,
    vfs: &Vfs,
    file_id: FileId,
    file_content: &str,
    settings: &Settings,
) -> Result<String, anyhow::Error> {
    println!("get highlight ranges");
    let hightlight = get_highlight_ranges(host, vfs, file_id);
    println!("building html");
    let finder = host.analysis().file_line_index(file_id).unwrap();
    let folding_ranges = host
        .analysis()
        .folding_ranges(file_id)
        .unwrap()
        .into_iter()
        .map(|range| {
            let start = range.range.start();
            let start_line = finder.line_col(start).line;
            let end = range.range.end();
            let end_line = finder.line_col(end).line;
            FoldingRange {
                start_line: start_line + 1,
                end_line: end_line + 1,
            }
        })
        .map(|r| (r.start_line, r))
        .collect::<HashMap<_, _>>();
    // let mut folds: HashMap<u32, Vec<Fold>> = Default::default();
    // for fold_range in folding_ranges {
    //     folds.entry(fold_range.start_line).or_default().push(Fold::Open);
    //     folds.entry(fold_range.end_line).or_default().push(Fold::Close);
    // }

    let lines: Vec<Line> = hightlight
        .split_inclusive(|t| t.is_new_line)
        .map(|tokens| {
            tokens
                .into_iter()
                .map(|token| token.render(file_content, &settings))
                .collect::<String>()
        })
        .enumerate()
        .map(|(number, html_content)| {
            let number = number + 1;
            Line {
                number,
                html_content: html_content.to_string(),
                //fold: folds.entry(number as u32).or_default().to_vec(),
                fold: folding_ranges.get(&(number as u32)).cloned()
        }})
        .collect();
    render_lines(&lines)
}

pub fn highlight_other_as_html(content: String) -> Result<String, anyhow::Error> {
    let content = html_escape::encode_text(&content).to_string();
    let lines = content
        .split('\n')
        .enumerate()
        .map(|(number, html_content)| Line {
            number: number + 1,
            html_content: html_content.to_string(),
            fold: Default::default(),
        })
        .collect::<Vec<_>>();
    render_lines(&lines)
}

fn render_lines(lines: &[Line]) -> Result<String, anyhow::Error> {
    let mut context = Context::new();
    context.insert("lines", &lines);
    let result = TEMPLATES.render("code.html", &context)?;
    Ok(result)
}

fn get_highlight_ranges(host: &AnalysisHost, vfs: &Vfs, file_id: FileId) -> Vec<HtmlToken> {
    let sema = Semantics::new(host.raw_database());

    let (root, range_to_highlight) = {
        let source_file = sema.parse(file_id);
        let source_file = source_file.syntax();
        (source_file.clone(), source_file.text_range())
    };
    let krate = sema.scope(&root).expect("cannot load crate").krate();

    traverse_syntax(host, &sema, &vfs, file_id, &root, krate, range_to_highlight)
}

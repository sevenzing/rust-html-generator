use super::{traverse::traverse_syntax, HtmlToken};
use crate::{args::Settings, templates::TEMPLATES};
use hir::Semantics;
use ide::{AnalysisHost, FileId};
use serde::Serialize;
use std::fmt::Display;
use syntax::ast::AstNode;
use tera::Context;
use vfs::Vfs;

#[derive(Serialize)]
struct Line {
    number: usize,
    html_content: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct FoldingRange {
    start_line: u32,
    end_line: u32,
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
    let folding_ranges: Vec<FoldingRange> = host
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
        .collect();

    let lines: Vec<Line> = hightlight
        .split_inclusive(|t| t.is_new_line)
        .map(|tokens| {
            tokens
                .into_iter()
                .map(|token| unwrap_token(file_content, &token, &settings))
                .collect::<String>()
        })
        .enumerate()
        .map(|(number, html_content)| Line {
            number: number + 1,
            html_content: html_content.to_string(),
        })
        .collect();
    render_lines(&lines, &folding_ranges)
}

pub fn highlight_other_as_html(content: String) -> Result<String, anyhow::Error> {
    let content = html_escape::encode_text(&content).to_string();
    let lines = content
        .split('\n')
        .enumerate()
        .map(|(number, html_content)| Line {
            number: number + 1,
            html_content: html_content.to_string(),
        })
        .collect::<Vec<_>>();
    render_lines(&lines, &[])
}

fn render_lines(lines: &[Line], folding_ranges: &[FoldingRange]) -> Result<String, anyhow::Error> {
    let mut context = Context::new();
    context.insert("lines", &lines);
    let result = TEMPLATES.render("code.html", &context)?;
    Ok(result)
}

fn unwrap_token(file_content: &str, token: &HtmlToken, settings: &Settings) -> String {
    // if token.is_new_line {
    //     let raw_chunk = &file_content[token.range];
    //     raw_chunk.replace("\n", NEW_LINE_HELPER)
    // } else {
    let raw_chunk = &file_content[token.range];
    let chunk = html_escape::encode_text(raw_chunk).to_string();
    let chunk = html_token_to_string(chunk, token, settings);
    chunk.to_string()
    //}
}

fn html_token_to_string(content: impl Display, token: &HtmlToken, settings: &Settings) -> String {
    if let Some(mut class) = token.highlight.clone() {
        let hover_info = token
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
        if hover_info.is_empty() && token.type_info.is_some() {
            hover_info = token.type_info.as_ref().unwrap().clone();
        }
        if !hover_info.is_empty() {
            hover_info = format!("<span>{}</span>", html_escape::encode_text(&hover_info))
        }

        let jump_attributes = token
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

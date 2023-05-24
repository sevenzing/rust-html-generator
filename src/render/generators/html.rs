use crate::render::{html_processor::FoldingRange, static_files, HtmlToken};
use crate::args::Settings;
use serde::Serialize;
use std::collections::HashMap;
use tera::Context;

#[derive(Serialize, Clone)]
struct Line {
    number: usize,
    html_content: String,
    fold: Option<FoldingRange>,
}

pub fn generate_rust_file_html(
    hightlight: Vec<HtmlToken>,
    folding_ranges: HashMap<u32, FoldingRange>,
    file_content: &str,
    settings: &Settings,
) -> Result<String, anyhow::Error> {
    let lines: Vec<Line> = hightlight
        .split_inclusive(|t| t.is_new_line)
        .map(|tokens| {
            tokens
                .iter()
                .map(|token| token.render(file_content, settings))
                .collect::<String>()
        })
        .enumerate()
        .map(|(number, html_content)| {
            let number = number + 1;
            Line {
                number,
                html_content,
                //fold: folds.entry(number as u32).or_default().to_vec(),
                fold: folding_ranges.get(&(number as u32)).cloned(),
            }
        })
        .collect();
    render_lines(&lines)
}

pub fn generate_other_file_html(content: String) -> Result<String, anyhow::Error> {
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
    let result = static_files::templates::TEMPLATES.render("code.html", &context)?;
    Ok(result)
}

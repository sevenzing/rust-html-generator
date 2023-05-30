use crate::{
    args::Settings,
    parser::FileInfo,
    render::{static_files, syntax_processor::FoldingRange, SyntaxProcessor},
};
use serde::Serialize;
use tera::Context;
use vfs::FileId;

#[derive(Serialize, Clone)]
struct Line {
    number: usize,
    html_content: String,
    fold: Option<FoldingRange>,
}

#[derive(Debug, Default, Clone)]
pub struct HtmlGenerator {}

impl HtmlGenerator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn generate(
        &self,
        processor: &SyntaxProcessor,
        file_info: FileInfo,
        settings: &Settings,
    ) -> Result<String, anyhow::Error> {
        match file_info.ra_file_id {
            Some(file_id) => {
                self.generate_rust_file_html(processor, file_id, &file_info.content, settings)
            }
            None => self.generate_other_file_html(&file_info.content),
        }
    }

    fn generate_rust_file_html(
        &self,
        processor: &SyntaxProcessor,
        file_id: FileId,
        file_content: &str,
        settings: &Settings,
    ) -> Result<String, anyhow::Error> {
        let tokens = processor.process_file(file_id);
        let folding_ranges = processor.get_folding_ranges(file_id);
        let lines: Vec<Line> = tokens
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

    fn generate_other_file_html(&self, content: &str) -> Result<String, anyhow::Error> {
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
}

fn render_lines(lines: &[Line]) -> Result<String, anyhow::Error> {
    let mut context = Context::new();
    context.insert("lines", &lines);
    let result = static_files::templates::TEMPLATES.render("code.html", &context)?;
    Ok(result)
}

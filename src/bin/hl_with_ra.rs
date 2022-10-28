use ide::{
    Analysis, FileId, FileRange, Highlight, HighlightConfig, HoverConfig, HoverResult, TextRange,
};
use ide_db::base_db::VfsPath;
use rs_html::get_analysis;
use std::{
    fmt::{Display, Write},
    path::PathBuf,
};
use syntax::ast::AstNode;

pub(crate) fn highlight_as_html(
    analysis: &Analysis,
    file_id: FileId,
    file_content: String,
) -> Result<String, anyhow::Error> {
    let highlight_config = HighlightConfig {
        strings: false,
        punctuation: false,
        specialize_punctuation: false,
        specialize_operator: false,
        operator: false,
        inject_doc_comment: false,
        macro_bang: false,
        syntactic_name_ref_highlighting: false,
    };

    println!("start actual highlight");
    let hightlight = analysis.highlight(highlight_config, file_id)?;

    println!("start building html");
    let mut buf = String::new();
    buf.push_str(&rs_html::css::KW_STYLE);
    buf.push_str("<pre><code>");
    for range in &hightlight {
        let trange = TextRange::from(range.range);
        let frange = FileRange {
            file_id,
            range: trange,
        };

        let hover_config = HoverConfig {
            links_in_hover: true,
            documentation: None,
            keywords: true,
        };
        let hover = analysis
            .hover(&hover_config, frange)
            .unwrap()
            .map(|r| r.info);
        let chunk = html_escape::encode_text(&file_content[range.range]);
        let chunk = html_token(chunk, range.highlight, hover);
        write!(buf, "{}", chunk)?;
    }
    buf.push_str("</code></pre>");
    Ok(buf)
}

fn html_token(content: impl Display, highlight: Highlight, hover: Option<HoverResult>) -> String {
    if highlight.is_empty() {
        content.to_string()
    } else {
        let class = highlight.to_string().replace('.', " ");
        let hover_info = hover.map(|h| h.markup.to_string()).unwrap_or_default();
        let mut hover_info = match hover_info.as_str() {
            "()" => "",
            "{unknown}" => "",
            _ => &hover_info,
        }
        .to_string();

        if !hover_info.is_empty() {
            hover_info = format!("<span>{}</span>", html_escape::encode_text(&hover_info))
        }

        format!(
            "<span class=\"hovertext {}\">{}{}</span>",
            class, content, hover_info
        )
    }
}

fn main() {
    let root = PathBuf::from("/Users/levlymarenko/innopolis/thesis/test-rust-crate/");
    let root = PathBuf::from("/Users/levlymarenko/innopolis/thesis/rust-ast/");

    let (host, vfs) = get_analysis(&root).unwrap();

    let path = VfsPath::new_real_path(
        "/Users/levlymarenko/innopolis/thesis/test-rust-crate/src/main.rs".into(),
    );
    let path =
        VfsPath::new_real_path("/Users/levlymarenko/innopolis/thesis/rust-ast/src/lib.rs".into());

    let file_id = vfs.file_id(&path).expect("no file found");
    let sema = hir::Semantics::new(host.raw_database());

    let source_file = sema.parse(file_id);

    let html = highlight_as_html(&host.analysis(), file_id, source_file.syntax().to_string())
        .expect("failed to highlight");
    std::fs::write("./out.html", html).expect("unable to write file");
}

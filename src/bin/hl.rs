use hir::{HirDisplay, Semantics};
use ide::{
    AnalysisHost, ClosureReturnTypeHints, FileId, FileRange, Highlight, HighlightConfig,
    HoverConfig, HoverResult, InlayHintsConfig, RootDatabase, TextRange,
};
use ide_db::base_db::VfsPath;
use rs_html::{get_analysis, html::{Path, Dir, self}};
use std::{
    collections::HashMap,
    fmt::{Display, Write},
    path::PathBuf,
};
use syntax::{ast::AstNode, match_ast, NodeOrToken, SyntaxNode, SyntaxToken};

#[derive(Debug)]
struct HtmlToken {
    syntax_token: SyntaxToken,
    range: TextRange,
    highlight: Option<String>,
    hover_info: Option<HoverResult>,
    type_info: Option<String>,
}

pub fn highlight_file_as_html(
    host: &AnalysisHost,
    file_id: FileId,
    file_content: &str,
) -> Result<String, anyhow::Error> {
    println!("start actual highlight");

    let hightlight = get_highlight_ranges(host, file_id);

    println!("start building html");
    let mut buf = String::new();
    buf.push_str("<pre><code>");
    for token in &hightlight {
        let chunk = if token.syntax_token.kind() == SK::WHITESPACE
            && token.syntax_token.text().contains("\n")
        {
            let raw_chunk = &file_content[token.range];
            raw_chunk.replace("\n", "</code>\n<code>")
        } else {
            let raw_chunk = &file_content[token.range];
            let chunk = html_escape::encode_text(raw_chunk).to_string();
            let chunk = html_token(chunk, token);
            chunk.to_string()
        };
        write!(buf, "{}", chunk)?;
    }
    buf.push_str("</code></pre>");
    Ok(buf)
}

fn html_token(content: impl Display, token: &HtmlToken) -> String {
    if let Some(class) = token.highlight.clone() {
        //let hover_info = token.hover_info.as_ref().map(|h| h.markup.to_string()).unwrap_or_default();
        let mut hover_info = "".to_string();
        // let mut hover_info = match hover_info.as_str() {
        //     "()" => "",
        //     "{unknown}" => "",
        //     _ => &hover_info,
        // }
        // .to_string();
        if token.type_info.is_some() {
            hover_info = token.type_info.as_ref().unwrap().clone();
        }
        if !hover_info.is_empty() {
            hover_info = format!("<span>{}</span>", html_escape::encode_text(&hover_info))
        }
        return format!(
            "<span class=\"hovertext {}\">{}{}</span>",
            class, content, hover_info
        );
    };
    content.to_string()
}

fn get_highlight_ranges(host: &AnalysisHost, file_id: FileId) -> Vec<HtmlToken> {
    let sema = Semantics::new(host.raw_database());

    let (root, range_to_highlight) = {
        let source_file = sema.parse(file_id);
        let source_file = source_file.syntax();
        (source_file.clone(), source_file.text_range())
    };
    let krate = sema.scope(&root).expect("cannot load crate").krate();

    traverse_syntax(host, &sema, file_id, &root, krate, range_to_highlight)
}

use syntax::{
    ast, AstToken, SyntaxKind as SK,
    WalkEvent::{Enter, Leave},
};

fn traverse_syntax(
    host: &AnalysisHost,
    sema: &Semantics<'_, RootDatabase>,
    //config: HighlightConfig,
    file_id: FileId,
    root: &SyntaxNode,
    krate: hir::Crate,
    range_to_highlight: TextRange,
) -> Vec<HtmlToken> {
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
    let hl_map: HashMap<_, _> = host
        .analysis()
        .highlight(highlight_config, file_id)
        .expect("failed to highlight")
        .into_iter()
        .map(|r| (r.range, r.highlight))
        .collect();

    let inline_config = InlayHintsConfig {
        render_colons: false,
        type_hints: true,
        parameter_hints: false,
        chaining_hints: false,
        reborrow_hints: ide::ReborrowHints::Never,
        closure_return_type_hints: ClosureReturnTypeHints::Never,
        binding_mode_hints: false,
        lifetime_elision_hints: ide::LifetimeElisionHints::Never,
        param_names_for_lifetime_elision_hints: false,
        hide_named_constructor_hints: false,
        hide_closure_initialization_hints: false,
        max_length: None,
        closing_brace_hints_min_lines: None,
    };
    let type_map: HashMap<_, _> = host
        .analysis()
        .inlay_hints(&inline_config, file_id, None)
        .unwrap()
        .into_iter()
        .map(|hint| (hint.range, hint))
        .collect();
    println!("type_map={:?}", type_map);

    let mut a = vec![];
    for event in root.preorder_with_tokens() {
        let range = match &event {
            Enter(it) | Leave(it) => it.text_range(),
        };

        let element = match event {
            Enter(it) => it,
            Leave(NodeOrToken::Token(_)) => continue,
            Leave(NodeOrToken::Node(_)) => continue,
        };

        let token = match element {
            NodeOrToken::Node(node) => {
                continue;
            }
            NodeOrToken::Token(token) => token,
        };
        let highlight = highlight_class(&token, hl_map.get(&range).cloned());
        let trange = TextRange::empty(range.start());
        let frange = FileRange {
            file_id,
            range: trange,
        };

        let hover_config = HoverConfig {
            links_in_hover: false,
            documentation: None,
            keywords: true,
        };
        let hover_info = host
            .analysis()
            .hover(&hover_config, frange)
            .unwrap()
            .map(|r| r.info);
        let ty = infer_type(&token, sema);
        let html_token = HtmlToken {
            syntax_token: token,
            range,
            highlight,
            hover_info,
            type_info: type_map.get(&range).map(|h| h.label.to_string()),
        };

        a.push(html_token);
    }
    a
}

pub fn highlight_class(token: &SyntaxToken, ra_highlight: Option<Highlight>) -> Option<String> {
    if let Some(hl) = ra_highlight {
        Some(hl.to_string().replace('.', " "))
    } else {
        if syntax::ast::String::can_cast(token.kind()) {
            return Some("string_literal".into());
        } else {
            None
        }
    }
}

pub fn infer_type(token: &SyntaxToken, sema: &Semantics<'_, RootDatabase>) -> Option<hir::Type> {
    let node = token.parent()?;

    match_ast! {
    match node {
        ast::Pat(it) => {
                if let syntax::ast::Pat::IdentPat(pat) = it {
                    // let descended = sema.descend_node_into_attributes(pat.clone()).pop();
                    // let desc_pat = descended.as_ref().unwrap_or(&pat);
                    let ty = sema.type_of_pat(&pat.into())?.original;
                    Some(ty)
                } else { None }
            },
        ast::Expr(it) => {

            None
        },
        _ => None
        }
    }
}

// fn main() {
//     let root = PathBuf::from("/Users/levlymarenko/innopolis/thesis/test-rust-crate/");
//     //let root = PathBuf::from("/Users/levlymarenko/innopolis/thesis/rust-ast/");

//     let (host, vfs) = get_analysis(&root).unwrap();

//     let path = VfsPath::new_real_path(
//         "/Users/levlymarenko/innopolis/thesis/test-rust-crate/src/main.rs".into(),
//     );
//     //let path = VfsPath::new_real_path("/Users/levlymarenko/innopolis/thesis/rust-ast/src/lib.rs".into());

//     let file_id = vfs.file_id(&path).expect("no file found");
//     let sema = hir::Semantics::new(host.raw_database());

//     let source_file = sema.parse(file_id);

//     let html = highlight_file_as_html(&host, file_id, source_file.syntax().to_string())
//         .expect("failed to highlight");
//     std::fs::write("./out.html", html).expect("unable to write file");
// }

fn main() -> Result<(), anyhow::Error> {
    let root = PathBuf::from("/Users/levlymarenko/innopolis/thesis/test-rust-crate/");
    assert!(root.is_dir());
    let (host, vfs) = get_analysis(&root).unwrap();
    let mut files = vec![];
    for entry in walkdir::WalkDir::new(&root)
        .sort_by_file_name()
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.path().is_file())
        .filter(|f| f.path().extension().map(|e| e == "rs").unwrap_or(false))
    {
        let path = entry.path();

        let vfs_path = VfsPath::new_real_path(path.to_string_lossy().to_string());

        let file_relative_path = path.strip_prefix(root.clone()).expect("failed to extract relative path");
        files.push(Path::new(&file_relative_path.to_string_lossy().to_string()));
        
        let id = vfs.file_id(&vfs_path).expect("failed to read file");
        let content = std::str::from_utf8(vfs.file_contents(id))?;

        //let html_content = highlight_file_as_html(&host, id, content)?;

        //std::fs::write(format!("./out/{}.html"), html_content).expect("unable to write file");
    }
    let tree = Dir::from_paths(files);
    let s = html::generate_file_tree(tree);
    
    let s = format!("{}\n\n{}", rs_html::css::STYLE.to_string(), s);

    std::fs::write("tree.html", s).expect("unable to write file");
    Ok(())
}

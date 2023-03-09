use hir::Semantics;
use ide::{
    AnalysisHost, ClosureReturnTypeHints, FileId, FilePosition, FileRange, Highlight,
    HighlightConfig, HoverConfig, HoverResult, InlayHintsConfig,
    RootDatabase, TextRange,
};
use ide_db::base_db::VfsPath;
use rs_html::{
    get_analysis,
    html::{self, MyPath},
    jumps::{JumpInfo},
    templates::TEMPLATES,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};
use syntax::{ast::AstNode, match_ast, NodeOrToken, SyntaxNode, SyntaxToken};
use tera::Context;
use vfs::Vfs;

const NEW_LINE_HELPER: &str = "<<RUST_HL_NEW_LINE_HELPER>>";

#[derive(Debug)]
struct HtmlToken {
    syntax_token: SyntaxToken,
    range: TextRange,
    highlight: Option<String>,
    hover_info: Option<HoverResult>,
    type_info: Option<String>,
    jumps: Option<JumpInfo>,
}

#[derive(Serialize)]
struct Line {
    number: usize,
    html_content: String,
}

fn render_lines(lines: &[Line]) -> Result<String, anyhow::Error> {
    let mut context = Context::new();
    context.insert("lines", &lines);
    let result = TEMPLATES.render("code.html", &context)?;
    Ok(result)
}

fn is_new_line(syntax_token: &SyntaxToken) -> bool {
    syntax_token.kind() == SK::WHITESPACE && syntax_token.text().contains("\n")
}

fn unwrap_token(file_content: &str, token: &HtmlToken) -> String {
    if is_new_line(&token.syntax_token) {
        let raw_chunk = &file_content[token.range];
        raw_chunk.replace("\n", NEW_LINE_HELPER)
    } else {
        let raw_chunk = &file_content[token.range];
        let chunk = html_escape::encode_text(raw_chunk).to_string();
        let chunk = html_token_to_string(chunk, token);
        chunk.to_string()
    }
}

pub fn highlight_rust_file_as_html(
    host: &AnalysisHost,
    vfs: &Vfs,
    file_id: FileId,
    file_content: &str,
) -> Result<String, anyhow::Error> {
    println!("get highlight ranges");
    let hightlight = get_highlight_ranges(host, vfs, file_id);
    println!("building html");

    let lines = hightlight
        .into_iter()
        .map(|token| unwrap_token(file_content, &token))
        .collect::<String>()
        .split(NEW_LINE_HELPER)
        .enumerate()
        .map(|(number, html_content)| Line {
            number: number + 1,
            html_content: html_content.to_string(),
        })
        .collect::<Vec<_>>();
    render_lines(&lines)
}

fn highlight_other_as_html(content: String) -> Result<String, anyhow::Error> {
    let content = html_escape::encode_text(&content).to_string();
    let lines = content
        .split('\n')
        .enumerate()
        .map(|(number, html_content)| Line {
            number: number + 1,
            html_content: html_content.to_string(),
        })
        .collect::<Vec<_>>();
    render_lines(&lines)
}

fn html_token_to_string(content: impl Display, token: &HtmlToken) -> String {
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
                if let Ok(jump_data) = jump.serialize(&DIR, PROJECT_NAME) {
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

use syntax::{
    ast, AstToken, SyntaxKind as SK,
    WalkEvent::{Enter, Leave},
};

fn traverse_syntax(
    host: &AnalysisHost,
    _sema: &Semantics<'_, RootDatabase>,
    vfs: &Vfs,
    file_id: FileId,
    root: &SyntaxNode,
    _krate: hir::Crate,
    _range_to_highlight: TextRange,
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
            NodeOrToken::Node(_) => {
                continue;
            }
            NodeOrToken::Token(token) => token,
        };
        let highlight = highlight_class(&token, hl_map.get(&range).cloned());
        let frange = FileRange { file_id, range };
        let fposition = FilePosition {
            file_id,
            offset: range.start(),
        };

        let hover_config = HoverConfig {
            links_in_hover: false,
            documentation: None,
            keywords: true,
        };

        let kind = token.kind();
        let useless = kind.is_literal() || kind.is_keyword() || kind.is_punct() || kind.is_trivia();
        let def = if !useless {
            host.analysis().goto_definition(fposition).unwrap()
        } else {
            None
        };

        let jumps = def
            .map(|mut d| {
                d.info = d
                    .info
                    .into_iter()
                    .filter(|t| {
                        t.focus_range
                            .map(|focus_range| focus_range != range)
                            .unwrap_or(false)
                    })
                    .collect();
                d
            })
            .and_then(|r| (!r.info.is_empty()).then_some(r))
            .map(|info| {
                let target = &info.info[0];
                let file = vfs.file_path(target.file_id);
                let line_finder = host.analysis().file_line_index(target.file_id).unwrap();
                let focus = target.focus_range.unwrap();
                JumpInfo::new(file, &focus, line_finder)
            });

        let hover_info = {
            if token.kind() == SK::COMMENT {
                None
            } else {
                host.analysis()
                    .hover(&hover_config, frange)
                    .unwrap()
                    .map(|r| r.info)
            }
        };
        // if def.is_some() {
        //     println!("SOME! {:?}: {:?}", token, def);
        // }

        // let ty = infer_type(&token, sema);
        let html_token = HtmlToken {
            syntax_token: token,
            range,
            highlight,
            hover_info,
            type_info: type_map.get(&range).map(|h| h.label.to_string()),
            jumps,
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
                    let ty = sema.type_of_pat(&pat.into())?.original;
                    Some(ty)
                } else { None }
            },
        ast::Expr(_) => {
            None
        },
        _ => None
        }
    }
}

lazy_static::lazy_static! {
    static ref DIR: PathBuf = PathBuf::from("/Users/levlymarenko/innopolis/thesis/test-rust-crate/");
}
const PROJECT_NAME: &str = "test-rust-crate";

fn main() -> Result<(), anyhow::Error> {
    let root = DIR.clone();
    assert!(root.is_dir());
    let (host, vfs) = get_analysis(&root, true).unwrap();
    let mut files = vec![];
    let mut files_content = HashMap::new();

    let ignore: Vec<&Path> = vec![
        ".git",
        "target",
        "out",
        "out_nice.html",
        "README.md",
        "output_test_rust_create.html",
        "tree_script.js",
        "tree_style.css",
        "tree.html",
    ]
    .into_iter()
    .map(Path::new)
    .collect();

    for entry in walkdir::WalkDir::new(&root)
        .sort_by_file_name()
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.path().is_file())
        .filter(|f| {
            !f.path()
                .ancestors()
                .any(|f| ignore.iter().any(|end| f.ends_with(end)))
        })
    //.filter(|f| f.path().extension().map(|e| e == "rs").unwrap_or(false))
    {
        let path = entry.path();
        let is_rust_file: bool = path.extension().map(|e| e == "rs").unwrap_or(false);

        let vfs_path = VfsPath::new_real_path(path.to_string_lossy().to_string());

        let file_relative_path = path
            .strip_prefix(root.clone())
            .expect("failed to extract relative path");
        files.push(MyPath::new(
            &file_relative_path.to_string_lossy().to_string(),
        ));

        let highlighted_content = if is_rust_file {
            let id = vfs.file_id(&vfs_path).expect("failed to read file");
            let content = std::str::from_utf8(vfs.file_contents(id))?;
            println!("highlight file {:?}", file_relative_path);
            highlight_rust_file_as_html(&host, &vfs, id, content)?
        } else {
            let content = std::fs::read_to_string(path)?;
            highlight_other_as_html(content)?
        };

        let fname = format!(
            "{PROJECT_NAME}/{}",
            file_relative_path.to_string_lossy().to_string()
        );
        println!("{fname}");
        files_content.insert(fname, highlighted_content);
    }
    let s = html::generate(
        files,
        files_content,
        root.file_name().unwrap().to_str().unwrap(),
    );
    std::fs::write("output.html", s).expect("unable to write file");
    Ok(())
}

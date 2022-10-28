use hir::Semantics;
use ide::{
    Analysis, AnalysisHost, FileId, FileRange, Highlight, HighlightConfig, HlRange, HoverConfig,
    HoverResult, RootDatabase, TextRange,
};
use ide_db::base_db::VfsPath;
use phf::phf_map;
use rs_html::{
    get_analysis,
    highlights::{highlight_escape_string, Highlights},
};
use std::{
    fmt::{Display, Write},
    path::PathBuf,
};
use syntax::{ast::AstNode, NodeOrToken, SyntaxNode, SyntaxToken};

pub fn highlight_as_html(
    host: &AnalysisHost,
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

    let hightlight = get_highlight_ranges(host, file_id);

    println!("start building html");
    let mut buf = String::new();
    buf.push_str(&rs_html::css::KW_STYLE);
    buf.push_str("<pre><code>");
    for range in &hightlight {
        println!("{:?}", range);
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
        let hover = host
            .analysis()
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

fn get_highlight_ranges(host: &AnalysisHost, file_id: FileId) -> Vec<HlRange> {
    let sema = Semantics::new(host.raw_database());

    let (root, range_to_highlight) = {
        let source_file = sema.parse(file_id);
        let source_file = source_file.syntax();
        (source_file.clone(), source_file.text_range())
    };
    let mut hl = Highlights::new(root.text_range());
    let krate = match sema.scope(&root) {
        Some(it) => it.krate(),
        None => return hl.to_vec(),
    };
    traverse_syntax(&mut hl, &sema, file_id, &root, krate, range_to_highlight);
    hl.to_vec()
}

use syntax::{
    ast, AstToken, SyntaxKind as SK,
    WalkEvent::{Enter, Leave},
};

fn traverse_syntax(
    hl: &mut Highlights,
    sema: &Semantics<'_, RootDatabase>,
    //config: HighlightConfig,
    file_id: FileId,
    root: &SyntaxNode,
    krate: hir::Crate,
    range_to_highlight: TextRange,
) {
    for event in root.preorder_with_tokens() {
        println!("\n\nevent={:?}", event);
        let range = match &event {
            Enter(it) | Leave(it) => it.text_range(),
        };

        let element = match event {
            Enter(NodeOrToken::Token(tok)) if tok.kind() == SK::WHITESPACE => continue,
            Enter(it) => it,
            Leave(NodeOrToken::Token(_)) => continue,
            Leave(NodeOrToken::Node(_)) => continue,
        };

        let element = match element.clone() {
            NodeOrToken::Node(n) => match ast::NameLike::cast(n) {
                Some(n) => NodeOrToken::Node(n),
                None => continue,
            },
            NodeOrToken::Token(t) => NodeOrToken::Token(t),
        };

        let token = element.as_token().cloned();
        println!("token={:?}", token);
        if let Some(token) = token {
            if ast::String::can_cast(token.kind()) {
                println!("can cast");
                let string = ast::String::cast(token);
                if let Some(string) = string {
                    // highlight_format_string(hl, &string, &string, range);
                    println!("???????");
                    highlight_escape_string(hl, &string, range.start());
                }
            } else if ast::ByteString::can_cast(token.kind()) {
                if let Some(byte_string) = ast::ByteString::cast(token) {
                    highlight_escape_string(hl, &byte_string, range.start());
                }
            }
        }
        // let highlight_result = match element {
        //     NodeOrToken::Node(name_like) => highlight_name_like(
        //         sema,
        //         krate,
        //         //&mut bindings_shadow_count,
        //         name_like,
        //     ),
        //     NodeOrToken::Token(token) => highlight_token(sema, token).zip(Some(None)),
        // };
    }
}

pub fn highlight_name_like(
    sema: &Semantics<'_, RootDatabase>,
    krate: hir::Crate,
    //bindings_shadow_count: &mut FxHashMap<hir::Name, u32>,
    name_like: ast::NameLike,
) -> Option<(Highlight, Option<u64>)> {
    // match name_like {
    //     ast::NameLike::NameRef(name_ref) => highlight_name_ref(
    //         sema,
    //         krate,
    //         bindings_shadow_count,
    //         &mut binding_hash,
    //         syntactic_name_ref_highlighting,
    //         name_ref,
    //     ),
    //     ast::NameLike::Name(name) => {
    //         highlight_name(sema, bindings_shadow_count, &mut binding_hash, krate, name)
    //     }
    //     ast::NameLike::Lifetime(lifetime) => match IdentClass::classify_lifetime(sema, &lifetime) {
    //         Some(IdentClass::NameClass(NameClass::Definition(def))) => {
    //             highlight_def(sema, krate, def) | HlMod::Definition
    //         }
    //         Some(IdentClass::NameRefClass(NameRefClass::Definition(def))) => {
    //             highlight_def(sema, krate, def)
    //         }
    //         // FIXME: Fallback for 'static and '_, as we do not resolve these yet
    //         _ => SymbolKind::LifetimeParam.into(),
    //     },
    // };

    todo!()
}

pub fn highlight_token(
    sema: &Semantics<'_, RootDatabase>,
    token: SyntaxToken,
) -> Option<Highlight> {
    todo!()
}

fn main() {
    let root = PathBuf::from("/Users/levlymarenko/innopolis/thesis/test-rust-crate/");
    //let root = PathBuf::from("/Users/levlymarenko/innopolis/thesis/rust-ast/");

    let (host, vfs) = get_analysis(&root).unwrap();

    let path = VfsPath::new_real_path(
        "/Users/levlymarenko/innopolis/thesis/test-rust-crate/src/main.rs".into(),
    );
    //let path = VfsPath::new_real_path("/Users/levlymarenko/innopolis/thesis/rust-ast/src/bin/highlight.rs".into());

    let file_id = vfs.file_id(&path).expect("no file found");
    let sema = hir::Semantics::new(host.raw_database());

    let source_file = sema.parse(file_id);

    let html = highlight_as_html(&host, file_id, source_file.syntax().to_string())
        .expect("failed to highlight");
    std::fs::write("./out.html", html).expect("unable to write file");
}

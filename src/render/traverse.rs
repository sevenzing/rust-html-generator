use super::HtmlToken;
use crate::jumps::{JumpInfo, JumpLocation, Jumps};
use hir::Semantics;
use ide::{
    AnalysisHost, ClosureReturnTypeHints, FileId, FilePosition, FileRange, Highlight,
    HighlightConfig, HoverConfig, InlayHintsConfig, RootDatabase, TextRange,
};
use std::collections::HashMap;
use syntax::{
    AstToken, NodeOrToken, SyntaxKind as SK, SyntaxNode, SyntaxToken,
    WalkEvent::{Enter, Leave},
};
use vfs::Vfs;

pub fn traverse_syntax(
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

    let mut result_tokens = vec![];
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
        if is_new_line(&token) {
            let tokens = parse_whitespaces(token.text(), token.text_range().start().into());
            result_tokens.extend(tokens);
            continue;
        }

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
                let to = JumpInfo::from_focus(file, &focus, line_finder);

                let origin_file = vfs.file_path(file_id);
                let origin_line_finder = host.analysis().file_line_index(file_id).unwrap();
                let origin_location = JumpLocation::from_focus(&range, origin_line_finder);
                let from = JumpInfo::new(origin_file, origin_location);
                Jumps { to, from }
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
            is_new_line: is_new_line(&token),
            range,
            highlight,
            hover_info,
            type_info: type_map.get(&range).map(|h| h.label.to_string()),
            jumps,
        };

        result_tokens.push(html_token);
    }
    result_tokens
}

fn highlight_class(token: &SyntaxToken, ra_highlight: Option<Highlight>) -> Option<String> {
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

fn is_new_line(syntax_token: &SyntaxToken) -> bool {
    syntax_token.kind() == SK::WHITESPACE && syntax_token.text().contains("\n")
}

fn parse_whitespaces(text: &str, from: u32) -> Vec<HtmlToken> {
    let len = text.split('\n').count();
    assert!(text.contains('\n'));

    let mut shift = 0;
    let tokens = text
        .split('\n')
        .flat_map(|c| [c, "\n"])
        .take(2 * len - 1)
        .filter(|c| !c.is_empty())
        .map(|c| {
            let is_new_line = c == "\n";
            let delta = c.len() as u32;
            let start = from + shift;
            let end = start + delta;
            let range = TextRange::new(start.into(), end.into());
            shift += delta;
            HtmlToken::from_empty_info(range, is_new_line)
        })
        .collect();
    assert!(
        shift == text.len() as u32,
        "invalid invariant. shift: {shift}, len: {len}. text: {text:?}"
    );
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: u32, end: u32) -> TextRange {
        TextRange::new(start.into(), end.into())
    }

    #[test]
    fn test_parse_whitespaces() {
        for (text, expected) in [
            (
                "  \n\n  \n\n",
                [
                    (range(0, 2), false),
                    (range(2, 3), true),
                    (range(3, 4), true),
                    (range(4, 6), false),
                    (range(6, 7), true),
                    (range(7, 8), true),
                ]
                .as_slice(),
            ),
            (
                "\n    ",
                [(range(0, 1), true), (range(1, 5), false)].as_slice(),
            ),
        ] {
            let actual: Vec<_> = parse_whitespaces(text, 0)
                .into_iter()
                .map(|t| (t.range, t.is_new_line))
                .collect();
            assert_eq!(actual, expected)
        }
    }
}

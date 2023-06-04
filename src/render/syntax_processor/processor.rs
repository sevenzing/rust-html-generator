use hir::Semantics;
use ide::{
    Analysis, AnalysisHost, ClosureReturnTypeHints, FileId, FilePosition, FileRange, Highlight,
    HighlightConfig, HoverConfig, InlayHintsConfig, LineIndex, NavigationTarget,
    ReferenceSearchResult, TextRange, SearchScope,
};
use std::{collections::HashMap, path::Path, sync::Arc};
use syntax::{
    AstNode, AstToken, NodeOrToken, SyntaxKind as SK, SyntaxNode, SyntaxToken,
    WalkEvent::{Enter, Leave},
};
use vfs::{Vfs, VfsPath};

use crate::{
    render::{HtmlToken, JumpDestination, JumpLocation, Navigation},
    Settings,
};

use super::{folding::FoldingRanges, FoldingRange};

pub struct SyntaxProcessor {
    host: AnalysisHost,
    vfs: Vfs,
    all_files: Vec<FileId>
}

impl SyntaxProcessor {
    pub fn new(host: AnalysisHost, vfs: Vfs, all_files: Vec<FileId>) -> Self {
        Self { host, vfs, all_files}
    }

    pub fn get_folding_ranges(&self, file_id: FileId) -> FoldingRanges {
        let finder = self.line_finder(file_id);
        self.host
            .analysis()
            .folding_ranges(file_id)
            .expect("RA task cannot be cancelled")
            .into_iter()
            .map(|range| FoldingRange::new(&range.range, finder.as_ref()))
            .map(|r| (r.start_line, r))
            .collect()
    }

    pub fn process_file(&self, file_id: FileId, settings: &Settings) -> Vec<HtmlToken> {
        let sema = Semantics::new(self.host.raw_database());
        let root = {
            let source_file = sema.parse(file_id);
            let source_file = source_file.syntax();
            source_file.clone()
        };
        self.traverse_syntax(file_id, &root, settings)
    }

    fn line_finder(&self, file_id: FileId) -> Arc<LineIndex> {
        self.host.analysis().file_line_index(file_id).unwrap()
    }

    fn traverse_syntax(
        &self,
        file_id: FileId,
        root: &SyntaxNode,
        settings: &Settings,
    ) -> Vec<HtmlToken> {
        let analysis = self.host.analysis();
        let search_scope = SearchScope::files(&self.all_files);
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
        let hl_map: HashMap<_, _> = analysis
            .highlight(highlight_config, file_id)
            .expect("RA task cannot be cancelled")
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
        let type_map: HashMap<_, _> = analysis
            .inlay_hints(&inline_config, file_id, None)
            .expect("RA task cannot be cancelled")
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
                let tokens = parse_new_lines(token.text(), token.text_range().start().into(), None);
                result_tokens.extend(tokens);
                continue;
            }
            let highlight = highlight_class(&token, hl_map.get(&range).cloned());
            if is_string(&token) {
                let tokens = parse_new_lines(
                    token.text(),
                    token.text_range().start().into(),
                    highlight.clone(),
                );
                result_tokens.extend(tokens);
                continue;
            }

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
            let useless =
                kind.is_literal() || kind.is_keyword() || kind.is_punct() || kind.is_trivia();
            let def = if !useless {
                analysis
                    .goto_definition(fposition)
                    .expect("RA task cannot be cancelled")
            } else {
                None
            };
            
            let ref_search = if !useless {
                analysis
                    .find_all_refs(fposition, Some(search_scope.clone()))
                    .expect("RA task cannot be cancelled")
            } else {
                None
            };


            let definition = def
                .map(|mut d| {
                    d.info.retain(|t| {
                        t.focus_range
                            .map(|focus_range| focus_range != range)
                            .unwrap_or(false)
                    });
                    d
                })
                .and_then(|r| (!r.info.is_empty()).then_some(r))
                .and_then(|info| {
                    let target = &info.info[0];
                    jump_from_target(target, &self.vfs, &analysis, settings)
                });
            let from = jump_to_origin(frange, &self.vfs, &analysis, settings);
            let references = jumps_from_ref_search(ref_search, &self.vfs, &analysis, settings);
            let navigation = definition.map(|definition| Navigation {
                definition,
                references,
                from,
            });

            let hover_info = {
                if token.kind() == SK::COMMENT {
                    None
                } else {
                    analysis
                        .hover(&hover_config, frange)
                        .expect("RA task cannot be cancelled")
                        .map(|r| r.info)
                }
            };
            let html_token = HtmlToken {
                is_new_line: is_new_line(&token),
                range,
                highlight,
                hover_info,
                type_info: type_map.get(&range).map(|h| h.label.to_string()),
                navigation,
            };

            result_tokens.push(html_token);
        }
        result_tokens
    }
}

fn jump_from_target(
    target: &NavigationTarget,
    vfs: &Vfs,
    analysis: &Analysis,
    settings: &Settings,
) -> Option<JumpDestination> {
    let frange = FileRange {
        file_id: target.file_id,
        range: target.focus_or_full_range(),
    };
    jump_from_frange(frange, vfs, analysis, settings)
}

fn jump_from_frange(
    frange: FileRange,
    vfs: &Vfs,
    analysis: &Analysis,
    settings: &Settings,
) -> Option<JumpDestination> {
    let file = vfs.file_path(frange.file_id);
    let file_path =
        if let Ok(file_path) = relative_path(&file, &settings.dir, &settings.project_name) {
            file_path
        } else {
            return None;
        };
    let line_finder = analysis.file_line_index(frange.file_id).unwrap();
    
    Some(JumpDestination::from_focus(file_path, &frange.range, line_finder))
}




fn jump_to_origin(
    frange: FileRange,
    vfs: &Vfs,
    analysis: &Analysis,
    settings: &Settings,
) -> JumpDestination {
    let origin_file = vfs.file_path(frange.file_id);
    let origin_file_path = relative_path(&origin_file, &settings.dir, &settings.project_name)
        .expect("origin file should be relative path");
    let origin_line_finder = analysis.file_line_index(frange.file_id).unwrap();
    let origin_location = JumpLocation::from_focus(&frange.range, origin_line_finder);
    JumpDestination::new(origin_file_path, origin_location)
}

fn jumps_from_ref_search(
    maybe_results: Option<Vec<ReferenceSearchResult>>,
    vfs: &Vfs,
    analysis: &Analysis,
    settings: &Settings,
) -> Vec<JumpDestination> {
    maybe_results
        .unwrap_or_default()
        .into_iter()
        .flat_map(|search_result| {
            search_result
                .references
                .into_iter()
                .flat_map(|(file_id, refs)| {
                    refs.into_iter().filter_map(move |(range, _maybe_category)| {
                        let frange = FileRange {
                            file_id: file_id.clone(),
                            range
                        };
                        jump_from_frange(frange, vfs, analysis, settings)
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

fn highlight_class(token: &SyntaxToken, ra_highlight: Option<Highlight>) -> Option<String> {
    if let Some(hl) = ra_highlight {
        Some(hl.to_string().replace('.', " "))
    } else if syntax::ast::String::can_cast(token.kind()) {
        Some("string_literal".into())
    } else {
        None
    }
}

fn is_new_line(syntax_token: &SyntaxToken) -> bool {
    syntax_token.kind() == SK::WHITESPACE && syntax_token.text().contains('\n')
}

fn is_string(syntax_token: &SyntaxToken) -> bool {
    syntax_token.kind() == SK::STRING
}

fn parse_new_lines(text: &str, from: u32, highlight: Option<String>) -> Vec<HtmlToken> {
    let len = text.split('\n').count();

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
            HtmlToken::from_empty_info(range, is_new_line).with_highlight(highlight.clone())
        })
        .collect();
    assert!(
        shift == text.len() as u32,
        "invalid invariant. shift: {shift}, len: {len}. text: {text:?}"
    );
    tokens
}

fn relative_path(vfs: &VfsPath, root: &Path, project_name: &str) -> Result<String, anyhow::Error> {
    let file_relative_path = vfs
        .as_path()
        .ok_or_else(|| anyhow::anyhow!("invalid vfs"))?
        .as_ref()
        .strip_prefix(root)?;
    let s = format!("{project_name}/{}", file_relative_path.to_string_lossy());
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: u32, end: u32) -> TextRange {
        TextRange::new(start.into(), end.into())
    }

    #[test]
    fn test_parse_new_lines() {
        for (text, expected) in [
            ("1234", [(range(0, 4), false)].as_slice()),
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
            (
                "hello\n\nworld\n\n\\n\n\nI am here\\n\\n\\n\n",
                [
                    (range(0, 5), false),
                    (range(5, 6), true),
                    (range(6, 7), true),
                    (range(7, 12), false),
                    (range(12, 13), true),
                    (range(13, 14), true),
                    (range(14, 16), false),
                    (range(16, 17), true),
                    (range(17, 18), true),
                    (range(18, 33), false),
                    (range(33, 34), true),
                ]
                .as_slice(),
            ),
        ] {
            let actual: Vec<_> = parse_new_lines(text, 0, None)
                .into_iter()
                .map(|t| (t.range, t.is_new_line))
                .collect();
            assert_eq!(actual, expected)
        }
    }
}

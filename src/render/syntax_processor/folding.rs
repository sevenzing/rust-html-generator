use std::collections::HashMap;

use ide::LineIndex;
use serde::Serialize;
use syntax::TextRange;

pub type FoldingRanges = HashMap<u32, FoldingRange>;

#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FoldingRange {
    pub start_line: u32,
    pub end_line: u32,
}

impl FoldingRange {
    pub fn new(range: &TextRange, finder: &LineIndex) -> Self {
        let start = range.start();
        let start_line = finder.line_col(start).line;
        let end = range.end();
        let end_line = finder.line_col(end).line;
        Self {
            start_line: start_line + 1,
            end_line: end_line + 1,
        }
    }
}

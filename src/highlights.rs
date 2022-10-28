//! Collects a tree of highlighted ranges and flattens it.
use std::{cmp::Ordering, iter};

use ide::{HlRange, HlTag};
use std::ops;
use syntax::TextRange;

pub struct Highlights {
    root: Node,
}

struct Node {
    hl_range: HlRange,
    nested: Vec<Node>,
}

impl Highlights {
    pub fn new(range: TextRange) -> Highlights {
        Highlights {
            root: Node::new(HlRange {
                range,
                highlight: HlTag::None.into(),
                binding_hash: None,
            }),
        }
    }

    pub fn add(&mut self, hl_range: HlRange) {
        self.root.add(hl_range);
    }

    pub fn to_vec(&self) -> Vec<HlRange> {
        let mut res = Vec::new();
        self.root.flatten(&mut res);
        res
    }
}

impl Node {
    fn new(hl_range: HlRange) -> Node {
        Node {
            hl_range,
            nested: Vec::new(),
        }
    }

    fn add(&mut self, hl_range: HlRange) {
        assert!(self.hl_range.range.contains_range(hl_range.range));

        // Fast path
        if let Some(last) = self.nested.last_mut() {
            if last.hl_range.range.contains_range(hl_range.range) {
                return last.add(hl_range);
            }
            if last.hl_range.range.end() <= hl_range.range.start() {
                return self.nested.push(Node::new(hl_range));
            }
        }

        let overlapping = equal_range_by(&self.nested, |n| {
            TextRange::ordering(n.hl_range.range, hl_range.range)
        });

        if overlapping.len() == 1
            && self.nested[overlapping.start]
                .hl_range
                .range
                .contains_range(hl_range.range)
        {
            return self.nested[overlapping.start].add(hl_range);
        }

        let nested = self
            .nested
            .splice(overlapping.clone(), iter::once(Node::new(hl_range)))
            .collect::<Vec<_>>();
        self.nested[overlapping.start].nested = nested;
    }

    fn flatten(&self, acc: &mut Vec<HlRange>) {
        let mut start = self.hl_range.range.start();
        let mut nested = self.nested.iter();
        loop {
            let next = nested.next();
            let end = next.map_or(self.hl_range.range.end(), |it| it.hl_range.range.start());
            if start < end {
                acc.push(HlRange {
                    range: TextRange::new(start, end),
                    highlight: self.hl_range.highlight,
                    binding_hash: self.hl_range.binding_hash,
                });
            }
            start = match next {
                Some(child) => {
                    child.flatten(acc);
                    child.hl_range.range.end()
                }
                None => break,
            }
        }
    }
}

pub fn equal_range_by<T, F>(slice: &[T], mut key: F) -> ops::Range<usize>
where
    F: FnMut(&T) -> Ordering,
{
    let start = slice.partition_point(|it| key(it) == Ordering::Less);
    let len = slice[start..].partition_point(|it| key(it) == Ordering::Equal);
    start..start + len
}

use syntax::{ast::IsString, TextSize};

pub fn highlight_escape_string<T: IsString>(stack: &mut Highlights, string: &T, start: TextSize) {
    string.escaped_char_ranges(&mut |piece_range, char| {
        if char.is_err() {
            return;
        }

        if string.text()[piece_range.start().into()..].starts_with('\\') {
            stack.add(HlRange {
                range: piece_range + start,
                highlight: HlTag::EscapeSequence.into(),
                binding_hash: None,
            });
        }
    });
}

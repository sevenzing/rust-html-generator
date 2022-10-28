use std::{collections::HashMap, fmt::format};

use serde_json::Value;

#[derive(Debug)]
pub struct Path {
    pub parts: Vec<String>,
}
impl Path {
    pub fn new(path: &str) -> Path {
        Path {
            parts: path.to_string().split("/").map(|s| s.to_string()).collect(),
        }
    }
}


#[derive(Debug)]
pub struct Dir {
    name: String,
    children: Vec<Box<Dir>>,
}

impl Dir {
    pub fn from_paths(paths: Vec<Path>) -> Dir {
        let mut top = Self::new("root");
        for path in paths.iter() {
            Self::build_tree(&mut top, &path.parts, 0);
        };
        top
    }

    pub fn new(name: &str) -> Dir {
        Dir {
            name: name.to_string(),
            children: Default::default(),
        }
    }

    pub fn is_file(&self) -> bool {
        return self.children.is_empty()
    }


    fn build_tree(node: &mut Dir, parts: &Vec<String>, depth: usize) {
        if depth < parts.len() {
            let item = &parts[depth];
    
            let mut dir = match node.find_child(&item) {
                Some(d) => d,
                None => {
                    let d = Dir::new(&item);
                    node.add_child(d);
                    match node.find_child(&item) {
                        Some(d2) => d2,
                        None => panic!("Got here!"),
                    }
                }
            };
            Self::build_tree(&mut dir, parts, depth + 1);
        }
    }

    fn find_child(&mut self, name: &str) -> Option<&mut Dir> {
        for c in self.children.iter_mut() {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<Dir>,
    {
        self.children.push(Box::new(leaf.into()));
        self
    }
}


pub fn generate_file_tree(tree: Dir) -> String {
    let tree = traverse(tree, "/");

    format!(r#"
<div class="tnz-file-tree">
{tree}
</div>
"#)
}

fn traverse(tree: Dir, prefix_path: &str) -> String {
    let dirname = &tree.name;

    if tree.is_file() {
            let full_path = format!("{prefix_path}{dirname}");
            format!(r#"
<label class="tnz-file-tree-item file">
    <input class="tnz-file-tree-cb" type="radio" name="file" value="{full_path}">
    <span class="tnz-file-tree-label">{dirname}</span>
</label>
"#)
    } else {
        let prefix_path = format!("{prefix_path}{dirname}/");
        let result = tree.children.into_iter().map(|d| traverse(*d, &prefix_path)).collect::<Vec<String>>().join("\n\n");
        format!(r#"
<label class="tnz-file-tree-item dir">
<input class="tnz-file-tree-cb" type="checkbox">

<span class="tnz-file-tree-label">{dirname}</span>
<div class="tnz-file-tree-branches">
{result}
</div>
</label>
"#)
    }
}


use std::{collections::HashMap, fmt::format};

use serde_json::Value;

use crate::{js::TREE_SCRIPT, css};

#[derive(Debug)]
pub struct MyPath {
    pub parts: Vec<String>,
}
impl MyPath {
    pub fn new(path: &str) -> MyPath {
        MyPath {
            parts: path.to_string().split("/").map(|s| s.to_string()).collect(),
        }
    }
}


#[derive(Debug)]
pub struct MyDir {
    name: String,
    children: Vec<Box<MyDir>>,
}

impl MyDir {
    pub fn from_paths(paths: Vec<MyPath>, top_dir_name: &str) -> MyDir {
        let mut top = Self::new(top_dir_name);
        for path in paths.iter() {
            Self::build_tree(&mut top, &path.parts, 0);
        };
        top
    }

    pub fn new(name: &str) -> MyDir {
        MyDir {
            name: name.to_string(),
            children: Default::default(),
        }
    }

    pub fn is_file(&self) -> bool {
        return self.children.is_empty()
    }


    fn build_tree(node: &mut MyDir, parts: &Vec<String>, depth: usize) {
        if depth < parts.len() {
            let item = &parts[depth];
    
            let mut dir = match node.find_child(&item) {
                Some(d) => d,
                None => {
                    let d = MyDir::new(&item);
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

    fn find_child(&mut self, name: &str) -> Option<&mut MyDir> {
        for c in self.children.iter_mut() {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<MyDir>,
    {
        self.children.push(Box::new(leaf.into()));
        self
    }
}


fn traverse(tree: MyDir, prefix_path: &str) -> String {
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

fn get_tree_script() -> String {
    TREE_SCRIPT.to_string()
}


fn save_files_in_html(files: HashMap<String, String>) -> String {
    files.into_iter().map(|(fname, content)| format!("
<div id={fname} class='invisible'>{content}</div>")).collect::<Vec<String>>().join("\n\n")
}

pub fn generate(filenames: Vec<MyPath>, files: HashMap<String, String>, dir: &str) -> String {
    let tree = MyDir::from_paths(filenames, dir);
    let tree = traverse(tree, "");
    let script = get_tree_script();
    let styles = css::STYLE.to_string();
    let files = save_files_in_html(files);
    format!(r#"
{styles}

<div class="content">
    <div class="left">
        <div class="tnz-file-tree">
{tree}
        </div>
    </div>
    <div class="right">
        <div id="code" class="code-text">Select file from the left</div>
    </div> 
</div>

<script>
{script}
</script>        

<div id="all-files">
{files}
</div>
"#)
}
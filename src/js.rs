lazy_static::lazy_static! {
    pub static ref TREE_SCRIPT: String = {
        std::fs::read_to_string("tree_script.js").expect("cannot file tree_script.js")
    };
}



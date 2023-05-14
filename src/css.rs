lazy_static::lazy_static! {
    pub static ref STYLE: String = {
        let flatten_css = [
            "css/keywords.css",
            "css/style.css",
            "css/tree_style.css"
        ].map(|name| {
            let content = std::fs::read_to_string(name).expect("cannot file tree_style.css");
            content
        }).join("\n");
        format!("<style>\n{flatten_css}\n</style>")
    };
}

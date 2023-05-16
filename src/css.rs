lazy_static::lazy_static! {
    pub static ref STYLE: String = {
        [
            "css/keywords.css",
            "css/style.css",
            "css/tree_style.css",
            "css/svgs.css",
            "css/fold.css",
        ].map(|name| {
            std::fs::read_to_string(name).expect(&format!("cannot read file {name}"))
        }).join("\n")
    };
}

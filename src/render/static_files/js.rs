lazy_static::lazy_static! {
    pub static ref JAVA_SCRIPT: String = {
        [
            "js/logic.js",
        ].map(|name| {
            std::fs::read_to_string(name).unwrap_or_else(|_| panic!("cannot read file {name}"))
        }).join("\n")
    };
}

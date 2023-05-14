lazy_static::lazy_static! {
    pub static ref JAVA_SCRIPT: String = {
        std::fs::read_to_string("logic.js").expect("cannot file logic.js")
    };
}

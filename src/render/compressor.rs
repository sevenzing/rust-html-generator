use minify_html::{minify, Cfg};

pub fn compress_html(content: &str) -> String {
    let content = content.as_bytes();
    let cfg = Cfg {
        keep_comments: false,
        minify_css_level_2: true,
        minify_js: true,
        remove_bangs: true,
        ..Cfg::default()
    };
    let compressed = minify(content, &cfg);
    std::str::from_utf8(&compressed)
        .expect("should be valid utf-8")
        .to_string()
}

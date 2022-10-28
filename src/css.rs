use phf::phf_map;

pub const KW_STYLES: phf::Map<&'static str, &'static str> = phf_map! {
    "lifetime" => "{ color: #DFAF8F; font-style: italic; }",
    "label" => "{ color: #DFAF8F; font-style: italic; }",
    "comment" => "{ color: #7F9F7F; }",
    "documentation" => "{ color: #629755; }",
    "intra_doc_link" => "{ font-style: italic; }",
    "injected" => "{ opacity: 0.65 ; }",
    "struct" => "{ color: #7CB8BB; }",
    "enum" => "{ color: #7CB8BB; }",
    "enum_variant" => "{ color: #BDE0F3; }",
    "string_literal" => "{ color: #CC9393; }",
    "field" => "{ color: #94BFF3; }",
    "function" => "{ color: #93E0E3; }",
    "function.unsafe" => "{ color: #BC8383; }",
    "trait.unsafe" => "{ color: #BC8383; }",
    "operator.unsafe" => "{ color: #BC8383; }",
    "mutable.unsafe" => "{ color: #BC8383; text-decoration: underline; }",
    "keyword.unsafe" => "{ color: #BC8383; font-weight: bold; }",
    "macro.unsafe" => "{ color: #BC8383; }",
    "parameter" => "{ color: #94BFF3; }",
    "text" => "{ color: #DCDCCC; }",
    "type" => "{ color: #7CB8BB; }",
    "builtin_type" => "{ color: #8CD0D3; }",
    "type_param" => "{ color: #DFAF8F; }",
    "attribute" => "{ color: #94BFF3; }",
    "numeric_literal" => "{ color: #BFEBBF; }",
    "bool_literal" => "{ color: #BFE6EB; }",
    "macro" => "{ color: #94BFF3; }",
    "derive" => "{ color: #94BFF3; font-style: italic; }",
    "module" => "{ color: #AFD8AF; }",
    "value_param" => "{ color: #DCDCCC; }",
    "variable" => "{ color: #DCDCCC; }",
    "format_specifier" => "{ color: #CC696B; }",
    "mutable" => "{ text-decoration: underline; }",
    "escape_sequence" => "{ color: #94BFF3; }",
    "keyword" => "{ color: #0f7bd9; font-weight: bold; }",
    "control" => "{ font-style: italic; }",
    "reference" => "{ font-style: italic; font-weight: bold; }",
    "function.declaration" => "{ color: #c8df39 }",
    "unresolved_reference" => "{ color: #FC5555; text-decoration: wavy underline; }",
};

lazy_static::lazy_static! {
    pub static ref KW_STYLE: String = {
        let kw_styles = KW_STYLES.into_iter().map(|(k, v)| format!(".{k:<30}{v}")).collect::<Vec<_>>().join("\n");
        r#"
body                { margin: 0; }
div                 { display: inline-block }
.hovertext span {
	background-color: rgba(0,0,0, 0.8);
    border-radius: 15px 15px 15px 0px;
    box-shadow: 1px 1px 10px rgb(0 0 0 / 50%);
    color: #fff;
    margin-left: 2px;
    margin-top: -30px;
    opacity: 0;
    padding: 10px 10px 10px 10px;
    position: absolute;
    text-decoration: none;
    visibility: hidden;
    z-index: 0;
}
		
.hovertext:hover span {
	opacity: 1;
	visibility: visible;
}

{{kw_styles}}
"#.replace("{{kw_styles}}", &kw_styles)
    };

    pub static ref TREE_STYLE: String = {
        std::fs::read_to_string("tree_style.css").expect("cannot file tree_style.css")
    };

    pub static ref STYLE: String = {
        format!(r#"
<style>
{}
{}
</style>"#, KW_STYLE.to_string(), TREE_STYLE.to_string())
    };
}



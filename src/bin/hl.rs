use clap::Parser;
use ide_db::base_db::VfsPath;
use rs_html::{
    args::Settings,
    get_analysis,
    html::{self, MyPath},
    render::{highlight_other_as_html, highlight_rust_file_as_html},
};
use std::{collections::HashMap, path::Path};

fn main() -> Result<(), anyhow::Error> {
    let settings = Settings::parse();
    let root = settings.dir.clone();
    assert!(root.is_dir());
    let (host, vfs) = get_analysis(&root, settings.scan_whole).unwrap();
    let mut files = vec![];
    let mut files_content = HashMap::new();

    let ignore: Vec<&Path> = vec![
        ".DS_Store",
        ".git",
        "target",
        "README.md",
        "output_rust_ast.html",
        "output.html",
        "tree_script.js",
        "tree_style.css",
        "tree.html",
        "Cargo.lock",
    ]
    .into_iter()
    .map(Path::new)
    .collect();

    for entry in walkdir::WalkDir::new(&root)
        .sort_by_file_name()
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.path().is_file())
        .filter(|f| {
            !f.path()
                .ancestors()
                .any(|f| ignore.iter().any(|end| f.ends_with(end)))
        })
    //.filter(|f| f.path().extension().map(|e| e == "rs").unwrap_or(false))
    {
        let path = entry.path();
        println!("INFO: walk to {path:?}");
        let is_rust_file: bool = path.extension().map(|e| e == "rs").unwrap_or(false);

        let vfs_path = VfsPath::new_real_path(path.to_string_lossy().to_string());

        let file_relative_path = path
            .strip_prefix(root.clone())
            .expect("failed to extract relative path");
        files.push(MyPath::new(&file_relative_path.to_string_lossy()));

        let highlighted_content = if is_rust_file {
            let id = vfs.file_id(&vfs_path).expect("failed to read file");
            let content = match std::str::from_utf8(vfs.file_contents(id)) {
                Ok(content) => content,
                Err(_) => {
                    println!("WARNING! cannot read file {vfs:?}");
                    continue;
                }
            };
            println!("highlight file {:?}", file_relative_path);
            highlight_rust_file_as_html(&host, &vfs, id, content, &settings)?
        } else {
            let content = std::fs::read_to_string(path)?;
            highlight_other_as_html(content)?
        };

        let fname = format!(
            "{}/{}",
            settings.project_name,
            file_relative_path.to_string_lossy()
        );
        files_content.insert(fname, highlighted_content);
    }
    let s = html::generate(
        files,
        files_content,
        root.file_name().unwrap().to_str().unwrap(),
    );
    std::fs::write(settings.output, s).expect("unable to write file");
    Ok(())
}

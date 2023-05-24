use crate::{
    args::Settings,
    parser,
    render::{
        generate_other_file_html, generate_report, generate_rust_file_html, HtmlProcessor, MyPath,
    },
};
use ide_db::base_db::VfsPath;
use std::{collections::HashMap, path::Path};

pub fn run_report_generator(settings: &Settings) -> Result<(), anyhow::Error> {
    let root = settings.dir.clone();
    if !root.is_dir() {
        return Err(anyhow::anyhow!("dir argument is not actual directory"));
    };
    let (host, vfs) = parser::get_analysis(&root, settings.scan_whole).unwrap();
    let processor = HtmlProcessor::new(host, vfs);
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
            let file_id = processor
                .vfs()
                .file_id(&vfs_path)
                .expect("failed to read file");
            let content = match std::str::from_utf8(processor.vfs().file_contents(file_id)) {
                Ok(content) => content,
                Err(_) => {
                    println!("WARNING! cannot read file {vfs_path:?}");
                    continue;
                }
            };
            println!("highlight file {:?}", file_relative_path);
            let hightlight = processor.get_highlight_ranges(file_id);
            let folding_ranges = processor.get_folding_ranges(file_id);
            generate_rust_file_html(hightlight, folding_ranges, content, settings)?
        } else {
            let content = std::fs::read_to_string(path)?;
            generate_other_file_html(content)?
        };

        let fname = format!(
            "{}/{}",
            settings.project_name,
            file_relative_path.to_string_lossy()
        );
        files_content.insert(fname, highlighted_content);
    }
    let s = generate_report(
        files,
        files_content,
        root.file_name().unwrap().to_str().unwrap(),
    );
    std::fs::write(&settings.output, s).expect("unable to write file");
    Ok(())
}

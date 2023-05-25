use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
};

use ide::AnalysisHost;
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustcSource};
use rust_analyzer::cli::load_cargo::{load_workspace, LoadCargoConfig};
use std::time::Instant;
use vfs::{AbsPathBuf, FileId, Vfs, VfsPath};

pub fn get_analysis(
    path: &PathBuf,
    scan_whole: bool,
) -> Result<(AnalysisHost, Vfs), anyhow::Error> {
    let cargo_config = CargoConfig {
        sysroot: if scan_whole {
            Some(RustcSource::Discover)
        } else {
            None
        },
        ..Default::default()
    };

    let load_cargo_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro: false,
        prefill_caches: true,
    };
    let no_progress = &|_| ();

    let project_start = Instant::now();
    let path = AbsPathBuf::assert(env::current_dir()?.join(path));
    let manifest = ProjectManifest::discover_single(&path)?;

    let mut workspace = ProjectWorkspace::load(manifest, &cargo_config, no_progress)?;
    println!("metadata_load: {}", project_start.elapsed().as_secs_f32());

    let now = Instant::now();
    let bs = workspace.run_build_scripts(&cargo_config, no_progress)?;
    workspace.set_build_scripts(bs);
    println!("build: {}", now.elapsed().as_secs_f32());

    let (host, vfs, _proc_macro) =
        load_workspace(workspace, &cargo_config.extra_env, &load_cargo_config)?;

    println!("db_load: {}", project_start.elapsed().as_secs_f32());

    Ok((host, vfs))
}

pub struct FileInfo {
    pub content: String,
    pub ra_file_id: Option<FileId>,
    pub path: PathBuf,
    pub file_relative_path: String,
}

pub fn scan(root: &PathBuf, vfs: &Vfs) -> Result<BTreeMap<String, FileInfo>, anyhow::Error> {
    let project_name = root
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("not a dir"))?
        .to_string_lossy()
        .to_string();
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
    let mut files = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root)
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
        let content = std::fs::read_to_string(path)?;
        let file_relative_path = path
            .strip_prefix(root.clone())
            .expect("failed to extract relative path");
        let fname = format!("{}/{}", project_name, file_relative_path.to_string_lossy());
        println!("INFO: walk to {path:?}");
        let is_rust_file: bool = path.extension().map(|e| e == "rs").unwrap_or(false);
        let ra_file_id = if is_rust_file {
            let vfs_path = VfsPath::new_real_path(path.to_string_lossy().to_string());
            let file_id = vfs
                .file_id(&vfs_path)
                .ok_or_else(|| anyhow::anyhow!("RA doesnt have rust file"))?;
            Some(file_id)
        } else {
            None
        };

        files.insert(
            fname,
            FileInfo {
                content,
                ra_file_id,
                path: path.to_path_buf(),
                file_relative_path: file_relative_path.to_string_lossy().to_string(),
            },
        );
    }

    Ok(files)
}

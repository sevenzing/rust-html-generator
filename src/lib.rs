pub mod args;
pub mod css;
pub mod highlights;
pub mod html;
pub mod js;
pub mod jumps;
pub mod render;
pub mod templates;

use std::{env, fs, path::PathBuf};

use ide::AnalysisHost;
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustcSource};
use rust_analyzer::cli::load_cargo::{load_workspace, LoadCargoConfig};
use vfs::{AbsPathBuf, Vfs};

use std::time::Instant;

pub fn get_file() -> (String, PathBuf) {
    let filepath = get_filepath();
    let content = fs::read_to_string(&filepath).expect("failed to read file");
    (content, filepath)
}

pub fn get_filepath() -> PathBuf {
    let mut args = env::args_os();
    _ = args.next(); // executable name

    match (args.next(), args.next()) {
        (Some(arg), None) => PathBuf::from(arg),
        _ => {
            panic!("incorrent usage")
        }
    }
}

pub fn get_analysis(
    path: &PathBuf,
    scan_whole: bool,
) -> Result<(AnalysisHost, Vfs), anyhow::Error> {
    let mut cargo_config = CargoConfig::default();

    cargo_config.sysroot = if scan_whole {
        Some(RustcSource::Discover)
    } else {
        None
    };
    //cargo_config.sysroot = None;

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

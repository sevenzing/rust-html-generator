use std::{env, path::PathBuf};

use ide::AnalysisHost;
use project_model::{CargoConfig, ProjectManifest, ProjectWorkspace, RustcSource};
use rust_analyzer::cli::load_cargo::{load_workspace, LoadCargoConfig};
use std::time::Instant;
use vfs::{AbsPathBuf, Vfs};

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

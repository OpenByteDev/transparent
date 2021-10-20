use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use build_target::Profile;
use path_absolutize::Absolutize;
use walkdir::WalkDir;

pub fn main() {
    let profile = Profile::current().unwrap();
    let target_triple = build_target::target_triple().unwrap();

    let runner_crate_name = "virtual-desktop-runner";
    let current_dir = env::current_dir()
        .unwrap()
        .absolutize()
        .unwrap()
        .to_path_buf();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let source_runner_crate_path = current_dir.join(runner_crate_name);
    let source_runner_crate_real_config = source_runner_crate_path.join("Cargo.toml");

    cargo_emit::rerun_if_changed!("{}", source_runner_crate_path.display());

    let target_runner_crate_dir: Box<dyn AsRef<Path>> = if source_runner_crate_real_config.exists()
    {
        Box::new(source_runner_crate_path)
    } else {
        let target_runner_crate_dir = tempfile::Builder::new()
            .prefix("virtual-desktop-runner-")
            .tempdir()
            .unwrap();
        let target_runner_crate_path = target_runner_crate_dir.path();

        let gitignore_path = source_runner_crate_path.join(".gitignore");
        let gitignore_file = gitignore::File::new(&gitignore_path).ok();
        let gitignore_filter = move |path: &Path| {
            !gitignore_file
                .as_ref()
                .map(|f| f.is_excluded(path).unwrap())
                .unwrap_or(false)
        };

        for entry in WalkDir::new(&source_runner_crate_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| gitignore_filter(e.path()))
        {
            if entry.path_is_symlink() {
                continue;
            }

            let source_path = entry.path();
            let relative_path = source_path.strip_prefix(&source_runner_crate_path).unwrap();
            let target_path = target_runner_crate_path.join(relative_path);
            if entry.file_type().is_dir() {
                fs::create_dir(&target_path).unwrap();
            } else {
                assert!(entry.file_type().is_file());
                fs::copy(&source_path, &target_path).unwrap();
            }
        }
        drop(gitignore_filter);

        fs::copy(
            target_runner_crate_path.join("Cargo.toml.why-no-bin-deps"),
            target_runner_crate_path.join("Cargo.toml"),
        )
        .unwrap();

        Box::new(target_runner_crate_dir)
    };

    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--target")
        .arg(&target_triple)
        .current_dir(target_runner_crate_dir.as_ref());
    if profile == Profile::Release {
        command.arg("--release");
    }

    command.spawn().unwrap().wait().unwrap().exit_ok().unwrap();

    let runner_executable_filename = format!("{}.exe", runner_crate_name);
    let runner_executable_path = target_runner_crate_dir
        .as_ref()
        .as_ref()
        .join("target")
        .join(&target_triple)
        .join(profile.as_str())
        .join(&runner_executable_filename);

    fs::copy(
        &runner_executable_path,
        &out_dir.join(runner_executable_filename),
    )
    .unwrap();
}

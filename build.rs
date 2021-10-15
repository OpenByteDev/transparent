#![feature(exit_status_error, path_try_exists)]

use core::fmt;
use std::{
    env::{self, VarError},
    fs,
    path::PathBuf,
    process::Command,
};

fn main() {
    if !cfg!(windows) {
        return;
    }

    let profile = Profile::target().unwrap();
    let target_triple = build_target::target_triple().unwrap();

    let subcrate_name = "virtual-desktop-runner";
    let subcrate_dir = env::current_dir().unwrap().join(subcrate_name);

    fs::copy(subcrate_dir.join("Cargo.toml.why-no-bin-deps"), subcrate_dir.join("Cargo.toml")).unwrap();

    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--target")
        .arg(&target_triple)
        .current_dir(&subcrate_dir);
    if profile == Profile::Release {
        command.arg("--release");
    }

    command.spawn().unwrap().wait().unwrap().exit_ok().unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let runner_executable_filename = format!("{}.exe", subcrate_name);
    let runner_executable_path = subcrate_dir
        .join("target")
        .join(&target_triple)
        .join(profile.as_str())
        .join(&runner_executable_filename);

    cargo_emit::rerun_if_changed!("{}", subcrate_dir.display());

    fs::copy(
        &runner_executable_path,
        &out_dir.join(runner_executable_filename),
    )
    .unwrap();
    
    fs::remove_file(subcrate_dir.join("Cargo.toml")).unwrap();
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Profile {
    Release,
    Debug,
}

impl Profile {
    fn target() -> Result<Self, VarError> {
        match env::var("PROFILE")?.as_str() {
            "release" => Ok(Self::Release),
            "debug" => Ok(Self::Debug),
            _ => unreachable!(),
        }
    }

    const fn as_str(&self) -> &'static str {
        match *self {
            Self::Release => "release",
            Self::Debug => "debug",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

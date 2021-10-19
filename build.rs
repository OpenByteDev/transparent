#![feature(exit_status_error, path_try_exists)]

use core::fmt;
use std::{env::{self, VarError}, fs::{self, File}, io::{Read, Write}, path::{Path, PathBuf}, process::Command};

use zip::{ZipArchive, ZipWriter, result::ZipResult, write::FileOptions};

fn main() {
    if !cfg!(windows) {
        return;
    }

    let profile = Profile::target().unwrap();
    let target_triple = build_target::target_triple().unwrap();

    let runner_crate_name = "virtual-desktop-runner";
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let current_dir = env::current_dir().unwrap();
    let mut runner_crate_dir = current_dir.join(runner_crate_name);
    let runner_crate_archive = current_dir.join(Path::new(&format!("{}.zip", runner_crate_name)));

    if !runner_crate_dir.exists() {
        runner_crate_dir = out_dir.join(runner_crate_name);
        extract_archive_to_directory(&runner_crate_archive, &runner_crate_dir).unwrap();
    }

    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--target")
        .arg(&target_triple)
        .current_dir(&runner_crate_dir);
    if profile == Profile::Release {
        command.arg("--release");
    }
    command.spawn().unwrap().wait().unwrap().exit_ok().unwrap();

    if option_env!("TRANSPARENT_CREATE_RUNNER_ARCHIVE").is_some() {
        cargo_emit::rerun_if_changed!("{}", runner_crate_dir.display());

        let gitignore_path = runner_crate_dir.join(".gitignore");
        let gitignore_file = gitignore::File::new(&gitignore_path).ok();
        let filter = move |path: &Path| !gitignore_file.as_ref().map(|f| f.is_excluded(path).unwrap()).unwrap_or(false);
        create_archive_from_directory(&runner_crate_archive, &runner_crate_dir, FileOptions::default(), filter).unwrap();
    }

    let runner_executable_filename = format!("{}.exe", runner_crate_name);
    let runner_executable_path = runner_crate_dir
        .join("target")
        .join(&target_triple)
        .join(profile.as_str())
        .join(&runner_executable_filename);

    fs::rename(
        &runner_executable_path,
        &out_dir.join(runner_executable_filename),
    )
    .unwrap();
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

fn create_archive_from_directory(archive_path: &Path, target_directory: &Path, options: FileOptions, filter: impl Fn(&Path) -> bool) -> ZipResult<()> {
    let archive_file = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(archive_file);

    let mut path_stack: Vec<PathBuf> = vec![];
    path_stack.push(target_directory.to_path_buf());

    let mut buffer = Vec::new();

    while let Some(next) = path_stack.pop() {
        for entry in fs::read_dir(next)? {
            let entry = entry?;
            let entry_path = entry.path();
            if !filter(&entry_path) {
                continue;
            }

            let entry_metadata = fs::metadata(&entry_path)?;
            if entry_metadata.is_file() {
                let mut f = File::open(&entry_path)?;
                f.read_to_end(&mut buffer)?;
                let relative_path = entry_path.strip_prefix(target_directory).unwrap();
                zip_writer.start_file(relative_path.to_string_lossy(), options)?;
                zip_writer.write_all(&buffer)?;
                buffer.clear();
            } else if entry_metadata.is_dir() {
                let relative_path = entry_path.strip_prefix(target_directory).unwrap();
                zip_writer.add_directory(relative_path.to_string_lossy(), options)?;
                path_stack.push(entry_path);
            }
        }
    }

    zip_writer.finish()?;
    Ok(())
}

fn extract_archive_to_directory(archive_path: &Path, target_directory: &Path) -> ZipResult<()> {
    let archive_file = File::create(archive_path)?;
    let mut zip_archive = ZipArchive::new(archive_file)?;

    let mut buf = Vec::new();
    for file_number in 0..zip_archive.len() {
        let mut next = zip_archive.by_index(file_number)?;
        if next.is_dir() {
            let extracted_folder_path = target_directory.join(next.enclosed_name().unwrap());
            std::fs::create_dir_all(extracted_folder_path)?;
        } else if next.is_file() {
            let _bytes_read = next.read_to_end(&mut buf)?;
            let extracted_file_path = target_directory.join(next.enclosed_name().unwrap());
            fs::write(extracted_file_path, &buf)?;
            buf.clear();
        }
    }

    Ok(())
}

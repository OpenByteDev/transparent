use std::{
    fs,
    io::{self, Write},
    lazy::SyncOnceCell,
    path::Path,
    process::{Child, Command},
    sync::Arc,
};

#[derive(Clone, Debug, Default)]
pub struct TransparentRunnerInner(Arc<SyncOnceCell<tempfile::TempPath>>);

impl TransparentRunnerInner {
    fn write_runner_executable_to_disk() -> io::Result<tempfile::TempPath> {
        #[cfg(feature = "__docs_rs")]
        unreachable!();
        #[cfg(not(feature = "__docs_rs"))]
        let bytes = include_bytes!(concat!(env!("OUT_DIR"), "\\virtual-desktop-runner.exe"));
        let mut file = tempfile::Builder::new()
            .prefix("transparent-runner-")
            .suffix(".exe")
            .tempfile()?;
        file.write_all(bytes)?;
        Ok(file.into_temp_path())
    }

    fn get_runner_executable_path(&self) -> io::Result<&Path> {
        self.0
            .get_or_try_init(Self::write_runner_executable_to_disk)
            .map(tempfile::TempPath::as_ref)
    }

    pub fn spawn_transparent(&self, command: &Command) -> io::Result<Child> {
        let runner_path = dbg!(self.get_runner_executable_path()?);

        let mut runner_command = Command::new(runner_path);
        runner_command
            .arg(fs::canonicalize(command.get_program())?)
            .arg("--")
            .args(command.get_args());

        for env in command.get_envs() {
            match env {
                (k, Some(v)) => runner_command.env(k, v),
                (k, None) => runner_command.env_remove(k),
            };
        }

        if let Some(current_dir) = command.get_current_dir() {
            runner_command.current_dir(current_dir);
        }

        runner_command.spawn()
    }
}

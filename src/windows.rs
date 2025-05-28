use std::{
    io::{self, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, OnceLock}
};

/// Windows-specific state required to run processes transparently.
#[derive(Clone, Debug, Default)]
pub struct TransparentRunnerImpl(Arc<OnceLock<tempfile::TempPath>>);

impl TransparentRunnerImpl {
    fn write_runner_executable_to_disk() -> io::Result<tempfile::TempPath> {
        #[cfg(docsrs)]
        let bytes = &[];
        #[cfg(not(docsrs))]
        let bytes = include_bytes!(concat!(
            env!("OUT_DIR"),
            "\\virtual-desktop-runner\\out\\virtual-desktop-runner.exe"
        ));
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
        let runner_path = self.get_runner_executable_path()?;

        let mut runner_command = Command::new(runner_path);
        runner_command
            .arg(command.get_program())
            .arg("--")
            .args(command.get_args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for env in command.get_envs() {
            match env {
                (k, Some(v)) => runner_command.env(k, v),
                (k, None) => runner_command.env_remove(k),
            };
        }

        if let Some(cd) = command.get_current_dir() {
            runner_command.current_dir(cd);
        } else {
            runner_command.current_dir(std::env::current_dir()?);
        }

        runner_command.spawn()
    }
}

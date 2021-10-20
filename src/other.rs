use std::{
    io,
    process::{Child, Command},
};

#[derive(Clone, Debug, Default)]
pub struct TransparentRunnerImpl;

impl TransparentRunnerImpl {
    pub fn spawn_transparent(&self, command: &Command) -> io::Result<Child> {
        let mut runner_command = Command::new("xvfb-run");
        runner_command
            .arg(command.get_program())
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

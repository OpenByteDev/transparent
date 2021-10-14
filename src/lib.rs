#![feature(once_cell)]

#[cfg(windows)]
mod windows;
use std::{
    io,
    ops::{Deref, DerefMut},
    process::{Child, Command},
};

#[cfg(windows)]
use windows as platform;

#[cfg(not(windows))]
mod other;
#[cfg(not(windows))]
use other as platform;

#[derive(Clone, Debug, Default)]
pub struct TransparentRunner(platform::TransparentRunnerInner);

#[derive(Debug)]
pub struct TransparentChild(Child, TransparentRunner);

impl Deref for TransparentChild {
    type Target = Child;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TransparentChild {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TransparentRunner {
    pub fn new() -> Self {
        Self(platform::TransparentRunnerInner::default())
    }

    pub fn spawn_transparent(&self, command: &Command) -> io::Result<TransparentChild> {
        self.0
            .spawn_transparent(command)
            .map(|child| TransparentChild(child, self.clone()))
    }
}

pub trait CommandExt {
    fn spawn_transparent(&self, runner: &TransparentRunner) -> io::Result<TransparentChild>;
}

impl CommandExt for Command {
    fn spawn_transparent(&self, runner: &TransparentRunner) -> io::Result<TransparentChild> {
        runner.spawn_transparent(self)
    }
}

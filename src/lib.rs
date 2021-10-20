#![feature(once_cell)]

#[cfg(windows)]
mod windows;
use std::{
    borrow::{Borrow, BorrowMut},
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
pub struct TransparentRunner(platform::TransparentRunnerImpl);

impl TransparentRunner {
    pub fn new() -> Self {
        Self(platform::TransparentRunnerImpl::default())
    }

    pub fn spawn_transparent(&self, command: &Command) -> io::Result<TransparentChild> {
        self.0
            .spawn_transparent(command)
            .map(|child| TransparentChild(child, self.clone()))
    }
}

#[derive(Debug)]
pub struct TransparentChild(Child, TransparentRunner);

impl TransparentChild {
    pub fn runner(&self) -> &TransparentRunner {
        &self.1
    }

    delegate::delegate! {
        to self.0 {
            pub fn wait_with_output(self) -> io::Result<std::process::Output>;
        }
    }
}

impl AsRef<Child> for TransparentChild {
    fn as_ref(&self) -> &Child {
        &self.0
    }
}
impl AsMut<Child> for TransparentChild {
    fn as_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}
impl Borrow<Child> for TransparentChild {
    fn borrow(&self) -> &Child {
        &self.0
    }
}
impl BorrowMut<Child> for TransparentChild {
    fn borrow_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}
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

pub trait CommandExt {
    fn spawn_transparent(&self, runner: &TransparentRunner) -> io::Result<TransparentChild>;
}

impl CommandExt for Command {
    fn spawn_transparent(&self, runner: &TransparentRunner) -> io::Result<TransparentChild> {
        runner.spawn_transparent(self)
    }
}

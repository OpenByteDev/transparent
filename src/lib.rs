/*!
A crate for running processes on a virtual desktop / virtual X server environment.

## Usage
This will spawn `some program` on a virtual desktop / virtual X server environment.
```rust
# use std::process::*;
# use transparent::*;
Command::new("some program")
    .spawn_transparent(&TransparentRunner::new())
    .unwrap()
    .wait()
    .unwrap();
```

## How it works
### Windows
On windows `transparent` uses [`CreateDesktopW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createdesktopw) to create a new desktop and then spawns a child process using [`CreateProcessW`](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw) with [`lpStartupInfo.lpDesktop`](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfow#syntax) set to the new desktop. (Actually a helper process is spawned which then in turn spawns the target process; see [`virtual-desktop-runner`](https://github.com/OpenByteDev/transparent/tree/master/virtual-desktop-runner)).

### Unix
On unix `transparent` uses [`xvfb-run`](http://manpages.ubuntu.com/manpages/trusty/man1/xvfb-run.1.html) which runs the target application in a virtual X server environment.

## Known issues
It is currently impossible to determine the specified [`Stdio`](https://doc.rust-lang.org/std/process/struct.Stdio.html) of a [`Command`](https://doc.rust-lang.org/std/process/struct.Command.html) without using [`mem::transmute`](https://doc.rust-lang.org/std/mem/fn.transmute.html) or similar, which is why `transparent` always uses [`Stdio::piped()`](https://doc.rust-lang.org/std/process/struct.Stdio.html#method.piped).

## License
Licensed under the MIT license ([LICENSE](https://github.com/OpenByteDev/transparent/blob/master/LICENSE) or <http://opensource.org/licenses/MIT>)
!*/

use std::{
    borrow::{Borrow, BorrowMut},
    io,
    ops::{Deref, DerefMut},
    process::{Child, Command},
};

#[cfg(all(windows, not(feature = "expose-impl")))]
mod windows;
#[cfg(all(windows, feature = "expose-impl"))]
pub mod windows;
#[cfg(windows)]
use windows as platform;

#[cfg(all(unix, not(feature = "expose-impl")))]
mod unix;
#[cfg(all(unix, feature = "expose-impl"))]
pub mod unix;
#[cfg(unix)]
use unix as platform;

/// Platform-dependent state required to run processes transparently.
#[derive(Clone, Debug, Default)]
pub struct TransparentRunner(platform::TransparentRunnerImpl);

impl TransparentRunner {
    /// Creates a new [`TransparentRunner`].
    pub fn new() -> Self {
        #[allow(clippy::default_constructed_unit_structs)]
        Self(platform::TransparentRunnerImpl::default())
    }

    /// Spawns the given [`Command`] transparently:
    ///  - on windows it is spawned on a new virtual desktop
    ///  - on unix it is spawned in a virtual X server environment
    pub fn spawn_transparent(&self, command: &Command) -> io::Result<TransparentChild> {
        self.0
            .spawn_transparent(command)
            .map(|child| TransparentChild(child, self.clone()))
    }
}

/// Representation of a running or exited child process that was spawned using [`TransparentRunner::spawn_transparent`].
#[derive(Debug)]
pub struct TransparentChild(Child, TransparentRunner);

impl TransparentChild {
    /// Gets the [`TransparentRunner`] used to run the source [`Command`].
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

/// Extension trait for [`Command`] providing an alternative form of [`TransparentRunner::spawn_transparent`].
pub trait CommandExt {
    /// Spawns the given [`Command`] transparently:
    ///  - on windows it is spawned on a new virtual desktop
    ///  - on unix it is spawned in a virtual X server environment
    fn spawn_transparent(&self, runner: &TransparentRunner) -> io::Result<TransparentChild>;
}

impl CommandExt for Command {
    fn spawn_transparent(&self, runner: &TransparentRunner) -> io::Result<TransparentChild> {
        runner.spawn_transparent(self)
    }
}

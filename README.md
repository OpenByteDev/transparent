# transparent

[![CI](https://github.com/OpenByteDev/transparent/actions/workflows/ci.yml/badge.svg)](https://github.com/OpenByteDev/transparent/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/transparent.svg)](https://crates.io/crates/transparent)
[![Documentation](https://docs.rs/transparent/badge.svg)](https://docs.rs/transparent)
[![dependency status](https://deps.rs/repo/github/openbytedev/transparent/status.svg)](https://deps.rs/repo/github/openbytedev/transparent)
[![MIT](https://img.shields.io/crates/l/transparent.svg)](https://github.com/OpenByteDev/transparent/blob/master/LICENSE)

A crate for running processes on a virtual desktop / X server environment.

## Usage
This will spawn `some program` on a new virtual desktop / X server environment.
```rust
Command::new("some program")
    .spawn_transparent(&TransparentRunner::new())
    .unwrap()
    .wait()
    .unwrap();
```

## How it works
### Windows
On windows `transparent` uses [`CreateProcessW`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createdesktopw) to create a new desktop and then spawns a child process using [`CreateProcessW`](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw) with [`lpStartupInfo.lpDesktop`](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfow#syntax) set to the new desktop. (Actually a helper process is spawned which then in turn spawns the target process; see [`virtual-desktop-runner`](https://github.com/OpenByteDev/transparent/tree/master/virtual-desktop-runner)).

### Unix
On unix `transparent` uses [`xvfb-run`](http://manpages.ubuntu.com/manpages/trusty/man1/xvfb-run.1.html) which runs the target application in a virtual X server environment.

## Known issues
It is currently impossible to determine the specified [`Stdio`](https://doc.rust-lang.org/std/process/struct.Stdio.html) of a [`Command`](https://doc.rust-lang.org/std/process/struct.Command.html) without using [`mem::transmute`](https://doc.rust-lang.org/std/mem/fn.transmute.html) or similar, which is why `transparent` always uses [`Stdio::piped()`](https://doc.rust-lang.org/std/process/struct.Stdio.html#method.piped).

## License
Licensed under the MIT license ([LICENSE](https://github.com/OpenByteDev/transparent/blob/master/LICENSE) or http://opensource.org/licenses/MIT)

#![feature(exit_status_error, path_try_exists)]

#[cfg(windows)]
#[path = "build-windows.rs"]
mod windows;

fn main() {
    #[cfg(windows)]
    windows::main();
}

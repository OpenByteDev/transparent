#![feature(exit_status_error, path_try_exists)]

#[path = "build-windows.rs"]
mod windows;

fn main() {
    #[cfg(windows)]
    windows::main();
}

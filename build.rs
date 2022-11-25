#![feature(exit_status_error)]

#[cfg(windows)]
#[path = "build-windows.rs"]
mod windows;

fn main() {
    #[cfg(windows)]
    windows::main();
}

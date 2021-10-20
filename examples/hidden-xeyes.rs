#![feature(exit_status_error)]

use std::process::Command;

use transparent::{CommandExt, TransparentRunner};

fn main() {
    Command::new("xeyes").spawn_transparent(&TransparentRunner::new()).unwrap().wait().unwrap().exit_ok().unwrap();
}

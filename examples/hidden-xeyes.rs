use std::process::Command;

use transparent::{CommandExt, TransparentRunner};

fn main() {
    let status = Command::new("xeyes")
        .spawn_transparent(&TransparentRunner::new())
        .unwrap()
        .wait()
        .unwrap();
    assert!(status.success());
}

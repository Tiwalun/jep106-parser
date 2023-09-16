use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::process::Command;

#[test]
fn help_output() {
    assert_cmd_snapshot!(Command::new(get_cargo_bin("jep106-build")).arg("-h"));
}

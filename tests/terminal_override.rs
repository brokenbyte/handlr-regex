use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use std::process::Command;
use test_case::test_matrix;

#[test_matrix([true, false])]
fn terminal_output_tests_direct(terminal_output: bool) {
    assert_cmd_snapshot!(Command::new(get_cargo_bin("handlr"))
        .arg(format!("--force-terminal-output={terminal_output}"))
        .arg("mime")
        .arg("./assets"))
}

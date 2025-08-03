// tests/cli_tests.rs

use assert_cmd::Command;

#[test]
fn test_commit_command_shows_help() {
    let mut cmd = Command::cargo_bin("cw").unwrap();
    cmd.arg("commit").arg("--help");
    cmd.assert().success().stdout(predicates::str::contains("Guide the user"));
}

#[test]
fn test_invalid_command_exits() {
    let mut cmd = Command::cargo_bin("cw").unwrap();
    cmd.arg("nonexistent");
    cmd.assert().failure().stderr(predicates::str::contains("error"));
}

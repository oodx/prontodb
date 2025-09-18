use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn e2e_dispatch_set_with_verbose() {
    let mut cmd = Command::cargo_bin("prontodb").unwrap();
    cmd.env("DEBUG", "1");
    let assert = cmd.args(["set", "--verbose"]).assert();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("Executing: set"));
    assert!(err.contains("Verbose mode enabled: 1"));
}

#[test]
fn e2e_dispatch_set_with_config() {
    let mut cmd = Command::cargo_bin("prontodb").unwrap();
    cmd.env("DEBUG", "1");
    let assert = cmd.args(["set", "--config=foo.conf"]).assert();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("Executing: set"));
    assert!(err.contains("Config path: foo.conf"));
}

#[test]
fn e2e_unknown_command_fails() {
    let mut cmd = Command::cargo_bin("prontodb").unwrap();
    cmd.env("DEBUG", "1");
    cmd.arg("bogus");
    let assert = cmd.assert().failure();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("Unknown command"));
}

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn test_entanglement_majority() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.args(["entanglement", "majority", "-n", "3"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Entanglement"));
}

#[test]
fn test_entanglement_parity() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.args(["entanglement", "parity", "-n", "3"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Entanglement"));
}

#[test]
fn test_entropy_majority() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.args(["entropy", "majority", "-n", "3"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Entropy"));
}

#[test]
fn test_entropy_with_sets() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.args(["entropy", "--sets", "parity", "-n", "2"]);
    cmd.assert().success();
}

#[test]
fn test_subinfo_majority() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.args(["sub-info", "majority", "-n", "3"]);
    cmd.assert().success();
}

#[test]
fn test_invalid_function_name() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.args(["entanglement", "nonexistent", "-n", "3"]);
    cmd.assert().failure();
}

#[test]
fn test_encode_command() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    let temp_file = std::env::temp_dir().join("test_encode.bin");
    cmd.args([
        "encode",
        "-o",
        temp_file.to_str().unwrap(),
        "majority",
        "-n",
        "3",
    ]);
    cmd.assert().success();

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
}

#[test]
fn test_entanglement_with_sorted_sets() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.args(["entanglement", "--sets", "--sorted", "parity", "-n", "2"]);
    cmd.assert().success();
}

#[test]
fn test_equanimity_importance() {
    let mut cmd = Command::cargo_bin("fbool-cli").unwrap();
    cmd.args(["equanimity-importance", "majority", "-n", "3"]);
    cmd.assert().success();
}

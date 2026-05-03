use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn smoke() {
    let mut cmd = Command::cargo_bin("main").unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn bogus() {
    let dir = tempdir().unwrap();
    let dir = dir.path().to_str().unwrap();

    Command::cargo_bin("main").unwrap().args(["--db", dir, "bogus"]).assert().failure();
}

#[test]
fn init() {
    let dir = tempdir().unwrap();
    let dir = dir.path().to_str().unwrap();

    Command::cargo_bin("main").unwrap().args(["--db", dir, "init"]).assert().success();
}

#[test]
fn info() {
    let dir = tempdir().unwrap();
    let dir = dir.path().to_str().unwrap();

    Command::cargo_bin("main").unwrap().args(["--db", dir, "init"]).unwrap();

    Command::cargo_bin("main").unwrap().args(["--db", dir, "info"]).assert().success();
}

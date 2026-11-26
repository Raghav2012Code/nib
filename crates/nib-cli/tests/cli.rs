use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

fn db(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("nib-cli-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.db"));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
    }
    path
}

fn nib(path: &Path) -> Command {
    let mut cmd = Command::cargo_bin("nib").unwrap();
    cmd.env("NIB_DB", path).env_remove("NIB_PASSPHRASE");
    cmd
}

fn add(path: &Path, body: &str) -> String {
    let output = nib(path).arg("add").write_stdin(body).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn add_prints_a_four_character_id() {
    let path = db("add");
    let id = add(&path, "hello");
    assert_eq!(id.len(), 4);
    assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
}

#[test]
fn get_is_byte_exact_when_piped() {
    let path = db("exact");
    let body = "fn main() {\n\tprintln!(\"hi\");\n}\n\n";
    let id = add(&path, body);
    nib(&path)
        .arg("get")
        .arg(&id)
        .assert()
        .success()
        .stdout(predicate::eq(body.as_bytes()));
}

#[test]
fn ls_shows_the_first_line() {
    let path = db("ls");
    add(&path, "\n\n   let  x = 1;   \nrest");
    nib(&path)
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("let x = 1;"));
}

#[test]
fn search_matches_inside_an_identifier() {
    let path = db("search");
    let id = add(&path, "fn parse_args() -> Args {");
    nib(&path)
        .args(["search", "arse_arg"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id));
}

#[test]
fn bare_nib_with_a_pipe_stores_and_prints_only_the_id() {
    let path = db("bare");
    let output = nib(&path).write_stdin("piped body").output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(text.trim().len(), 4);
    assert!(!text.contains('█'), "no art on a pipe");
}

#[test]
fn sealed_paste_needs_the_passphrase() {
    let path = db("sealed");

    let output = nib(&path)
        .args(["add", "--secret", "--label", "deploy key"])
        .env("NIB_PASSPHRASE", "pw")
        .write_stdin("token-xyz")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let id = String::from_utf8(output.stdout).unwrap().trim().to_string();

    nib(&path)
        .arg("get")
        .arg(&id)
        .assert()
        .failure()
        .stderr(predicate::str::contains("passphrase required"));

    nib(&path)
        .arg("get")
        .arg(&id)
        .env("NIB_PASSPHRASE", "pw")
        .assert()
        .success()
        .stdout(predicate::eq("token-xyz"));

    nib(&path)
        .args(["search", "token-xyz"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no matches"));

    nib(&path)
        .args(["search", "deploy key"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&id));
}

#[test]
fn rm_needs_force_when_there_is_no_terminal() {
    let path = db("rm");
    let id = add(&path, "delete me");

    nib(&path).arg("rm").arg(&id).assert().failure();
    nib(&path).arg("get").arg(&id).assert().success();

    nib(&path).args(["rm", &id, "--force"]).assert().success();
    nib(&path).arg("get").arg(&id).assert().failure();
}

#[test]
fn bare_id_is_shorthand_for_get() {
    let path = db("short");
    let body = "shorthand body";
    let id = add(&path, body);
    nib(&path)
        .arg(&id)
        .assert()
        .success()
        .stdout(predicate::eq(body.as_bytes()));
    nib(&path)
        .arg(&id[..2])
        .assert()
        .success()
        .stdout(predicate::eq(body.as_bytes()));
}

#[test]
fn oversized_paste_is_refused() {
    let path = db("large");
    let body = "x".repeat(64 * 1024 + 1);
    nib(&path)
        .arg("add")
        .write_stdin(body)
        .assert()
        .failure()
        .stderr(predicate::str::contains("limit is 65536"));
}

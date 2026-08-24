use std::fs;

use assert_cmd::Command;
use predicates::str::contains;

fn cmd() -> Command {
    Command::cargo_bin("ar7json").unwrap()
}

#[test]
fn completions_supported_shells_succeed() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        cmd()
            .args(["completions", shell])
            .assert()
            .success()
            .stdout(contains("ar7json"))
            .stderr("");
    }
}

#[test]
fn completions_bash_registers_completeness_function() {
    cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("_ar7json()"));
}

#[test]
fn completions_zsh_registers_compdef() {
    cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(contains("#compdef ar7json"));
}

#[test]
fn completions_lists_all_subcommands_in_bash() {
    let stdout = cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(stdout).unwrap();
    for subcommand in ["to-json", "to-ar7", "check", "format", "completions", "man"] {
        assert!(
            stdout.contains(subcommand),
            "missing {subcommand} in bash completions"
        );
    }
}

#[test]
fn completions_unknown_shell_fails_with_error() {
    cmd()
        .args(["completions", "tcsh"])
        .assert()
        .failure()
        .code(2)
        .stderr(contains("invalid value 'tcsh'"))
        .stderr(contains("possible values"));
}

#[test]
fn completions_output_file_writes_script() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ar7json.bash");
    cmd()
        .args(["completions", "bash", "-o"])
        .arg(&path)
        .assert()
        .success();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("_ar7json()"));
}

#[test]
fn man_page_contains_name_and_synopsis() {
    let stdout = cmd()
        .args(["man"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let man = String::from_utf8(stdout).unwrap();
    assert!(man.contains(".TH ar7json 1"));
    assert!(man.contains(".SH NAME"));
    assert!(man.contains(".SH SYNOPSIS"));
    let plain = man.replace('\\', "");
    for subcommand in ["to-json", "to-ar7", "check", "format", "completions", "man"] {
        assert!(
            plain.contains(subcommand),
            "missing {subcommand} in man page"
        );
    }
}

#[test]
fn man_output_file_writes_page() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ar7json.1");
    cmd().args(["man", "-o"]).arg(&path).assert().success();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains(".TH ar7json 1"));
}

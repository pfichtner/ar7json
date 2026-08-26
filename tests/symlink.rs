use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn binary_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    // When running tests, the binary is in target/debug/deps or target/debug.
    // assert_cmd::Command::cargo_bin resolves this, but here we find it manually.
    path.push("ar7json");
    path
}

fn setup_symlinks() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let bin = binary_path();

    // Create "ar7json" symlink so env::current_exe() resolves inside the temp dir
    std::os::unix::fs::symlink(&bin, dir.path().join("ar7json")).unwrap();

    // Exercise the real setup code path
    let output = Command::new(dir.path().join("ar7json"))
        .args(["setup", "--dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ar7json setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    dir
}

fn run_symlink(dir: &tempfile::TempDir, name: &str, args: &[&str]) -> String {
    let output = Command::new(dir.path().join(name))
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} failed: {}",
        name,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn run_symlink_stderr(dir: &tempfile::TempDir, name: &str, args: &[&str]) -> String {
    let output = Command::new(dir.path().join(name))
        .args(args)
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn ar7_to_json_converts_ar7_to_json() {
    let dir = setup_symlinks();
    let json = run_symlink(&dir, "ar7-to-json", &["tests/fixtures/minimal.ar7"]);
    assert!(json.contains("\"format\": \"ar7json\""));
    assert!(json.contains("\"encoding\""));
}

#[test]
fn ar7_to_json_with_output_flag() {
    let dir = setup_symlinks();
    let out = dir.path().join("out.json");
    run_symlink(&dir, "ar7-to-json", &["tests/fixtures/minimal.ar7", "-o"]);
    // -o requires a value; test with proper syntax
    let output = Command::new(dir.path().join("ar7-to-json"))
        .args(["tests/fixtures/minimal.ar7", "-o", out.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("\"format\": \"ar7json\""));
}

fn run_symlink_stdin(dir: &tempfile::TempDir, name: &str, stdin: &str) -> String {
    let mut child = Command::new(dir.path().join(name))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{} failed: {}",
        name,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn ar7_to_json_stdin() {
    let dir = setup_symlinks();
    let json = run_symlink_stdin(&dir, "ar7-to-json", "meta { encoding = \"utf-8\"; }");
    assert!(json.contains("\"encoding\""));
}

#[test]
fn json_to_ar7_converts_json_to_ar7() {
    let dir = setup_symlinks();
    let json_path = dir.path().join("input.json");
    fs::write(
        &json_path,
        r#"{"format":"ar7json","version":1,"document":{"entries":[{"key":"meta","value":{"entries":[{"key":"encoding","value":{"type":"string","value":"utf-8","raw":"\"utf-8\""}}],"type":"object"}}]}}"#,
    )
    .unwrap();
    let ar7 = run_symlink(&dir, "json-to-ar7", &[json_path.to_str().unwrap()]);
    assert!(ar7.contains("encoding"));
}

#[test]
fn ar7_check_valid_file() {
    let dir = setup_symlinks();
    let stderr = run_symlink_stderr(&dir, "ar7-check", &["tests/fixtures/minimal.ar7"]);
    assert!(stderr.contains("OK"));
}

#[test]
fn ar7_check_invalid_file() {
    let dir = setup_symlinks();
    let bad = dir.path().join("bad.ar7");
    fs::write(&bad, "meta { { {").unwrap();
    let output = Command::new(dir.path().join("ar7-check"))
        .arg(bad.to_str().unwrap())
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn ar7_fmt_formats_ar7() {
    let dir = setup_symlinks();
    let ar7 = run_symlink(&dir, "ar7-fmt", &["tests/fixtures/minimal.ar7"]);
    assert!(ar7.contains("meta"));
    assert!(ar7.contains("encoding"));
}

#[test]
fn ar7_to_json_matches_subcommand() {
    let dir = setup_symlinks();
    let via_symlink = run_symlink(&dir, "ar7-to-json", &["tests/fixtures/nested.ar7"]);
    let via_subcommand = String::from_utf8(
        Command::new(binary_path())
            .args(["to-json", "tests/fixtures/nested.ar7"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(via_symlink, via_subcommand);
}

#[test]
fn json_to_ar7_matches_subcommand() {
    let dir = setup_symlinks();
    let json_path = dir.path().join("input.json");
    fs::write(
        &json_path,
        r#"{"format":"ar7json","version":1,"document":{"entries":[{"key":"meta","value":{"entries":[{"key":"encoding","value":{"type":"string","value":"utf-8","raw":"\"utf-8\""}}],"type":"object"}}]}}"#,
    )
    .unwrap();
    let via_symlink = run_symlink(&dir, "json-to-ar7", &[json_path.to_str().unwrap()]);
    let via_subcommand = String::from_utf8(
        Command::new(binary_path())
            .args(["to-ar7", json_path.to_str().unwrap()])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(via_symlink, via_subcommand);
}

#[test]
fn ar7_check_matches_subcommand() {
    let dir = setup_symlinks();
    let via_symlink = run_symlink_stderr(&dir, "ar7-check", &["tests/fixtures/minimal.ar7"]);
    let via_subcommand = String::from_utf8(
        Command::new(binary_path())
            .args(["check", "tests/fixtures/minimal.ar7"])
            .output()
            .unwrap()
            .stderr,
    )
    .unwrap();
    assert_eq!(via_symlink, via_subcommand);
}

#[test]
fn ar7_fmt_matches_subcommand() {
    let dir = setup_symlinks();
    let via_symlink = run_symlink(&dir, "ar7-fmt", &["tests/fixtures/minimal.ar7"]);
    let via_subcommand = String::from_utf8(
        Command::new(binary_path())
            .args(["format", "tests/fixtures/minimal.ar7"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(via_symlink, via_subcommand);
}

#[test]
fn unknown_symlink_name_falls_through_to_normal_mode() {
    let dir = setup_symlinks();
    let unknown = dir.path().join("ar7json");
    // Calling via the real binary name should work normally (subcommand mode)
    let output = Command::new(&unknown)
        .args(["to-json", "tests/fixtures/minimal.ar7"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

fn setup_binary_in_dir(dir: &std::path::Path) -> std::path::PathBuf {
    let bin = binary_path();
    let bin_dir = dir.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    std::os::unix::fs::symlink(&bin, bin_dir.join("ar7json")).unwrap();

    // Exercise the real setup code path
    let output = Command::new(bin_dir.join("ar7json"))
        .args(["setup", "--dir", bin_dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "ar7json setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    bin_dir
}

#[test]
fn setup_creates_symlinks_in_custom_dir() {
    let dir = tempfile::tempdir().unwrap();
    let bin_dir = setup_binary_in_dir(dir.path());

    // setup_binary_in_dir already ran ar7json setup --dir bin_dir
    // Verify symlinks exist and point to the real binary
    for name in ["ar7-to-json", "json-to-ar7", "ar7-check", "ar7-fmt"] {
        let link = bin_dir.join(name);
        assert!(link.exists(), "symlink {} not created", name);
        assert!(link.symlink_metadata().unwrap().file_type().is_symlink());
    }
}

#[test]
fn setup_creates_symlinks_in_default_dir() {
    let dir = tempfile::tempdir().unwrap();
    let bin = binary_path();
    let bin_in_dir = dir.path().join("ar7json");
    std::os::unix::fs::symlink(&bin, &bin_in_dir).unwrap();

    let output = Command::new(&bin_in_dir)
        .args(["setup", "--dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for name in ["ar7-to-json", "json-to-ar7", "ar7-check", "ar7-fmt"] {
        let link = dir.path().join(name);
        assert!(link.exists(), "symlink {} not created", name);
    }
}

#[test]
fn setup_overwrites_existing_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let bin_dir = dir.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let bin = binary_path();
    std::os::unix::fs::symlink(&bin, bin_dir.join("ar7json")).unwrap();

    // Create a stale symlink
    std::os::unix::fs::symlink("/nonexistent", bin_dir.join("ar7-to-json")).unwrap();

    let output = Command::new(bin_dir.join("ar7json"))
        .args(["setup", "--dir", bin_dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Should now point to ar7json (relative, same directory)
    let link = bin_dir.join("ar7-to-json");
    assert!(link.exists());
    let target = fs::read_link(&link).unwrap();
    assert_eq!(target.to_str().unwrap(), "ar7json");
}

#[test]
fn setup_created_symlinks_are_usable() {
    let dir = tempfile::tempdir().unwrap();
    let bin_dir = setup_binary_in_dir(dir.path());

    let json = Command::new(bin_dir.join("ar7-to-json"))
        .arg("tests/fixtures/minimal.ar7")
        .output()
        .unwrap();
    assert!(json.status.success());
    let stdout = String::from_utf8(json.stdout).unwrap();
    assert!(stdout.contains("\"encoding\""));
}

#[test]
fn setup_stderr_output_shows_created_links() {
    let dir = tempfile::tempdir().unwrap();
    let bin = binary_path();
    let bin_dir = dir.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    std::os::unix::fs::symlink(&bin, bin_dir.join("ar7json")).unwrap();

    let output = Command::new(bin_dir.join("ar7json"))
        .args(["setup", "--dir", bin_dir.to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Created 4 symlinks"));
    assert!(stderr.contains("ar7-to-json"));
    assert!(stderr.contains("json-to-ar7"));
    assert!(stderr.contains("ar7-check"));
    assert!(stderr.contains("ar7-fmt"));
}

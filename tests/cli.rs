use assert_cmd::Command;

#[test]
fn cli() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.trycmd")
        .case("tests/cmd/*.toml");
}

#[test]
fn version_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_ar7json"))
        .arg("version")
        .assert()
        .success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let expected = format!("ar7json {}", env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout.trim(), expected);
}

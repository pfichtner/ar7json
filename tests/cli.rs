#[test]
fn cli() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.trycmd")
        .case("tests/cmd/*.toml");
}

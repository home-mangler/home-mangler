use expect_test::expect;
use harness::OutputExt;

mod harness;

#[test]
fn packages1() {
    let mut session = harness::session();
    let cmd = session
        .cmd
        .args([
            "--flake",
            "path:.",
            // TODO: don't hardcode this lol
            "--hostname",
            "aarch64-darwin.packages1",
            "--config",
            "config.toml",
        ])
        .assert();
    expect![""].assert_eq(&cmd.get_output().stdout_utf8());
}

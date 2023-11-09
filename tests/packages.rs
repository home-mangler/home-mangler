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
    println!(
        "{}",
        String::from_utf8(cmd.get_output().stdout.clone()).unwrap()
    );
}

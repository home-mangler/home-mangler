mod harness;

#[test]
fn packages1() {
    let mut session = harness::session();
    session.cmd.assert().success();
}

use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;

// --------------------------------------------------
#[test]
fn dies_no_args() -> Result<()> {
    Command::cargo_bin("wodgen")?
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
    Ok(())
}

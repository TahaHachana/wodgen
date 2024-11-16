use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;

const PRG: &str = "wodgen";

// --------------------------------------------------
#[test]
fn dies_no_args() -> Result<()> {
    Command::cargo_bin(PRG)?
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
    Ok(())
}

// --------------------------------------------------
#[test]
fn usage() -> Result<()> {
    for flag in &["-h", "--help"] {
        Command::cargo_bin(PRG)?
            .arg(flag)
            .assert()
            .stdout(predicate::str::contains("Usage"));
    }
    Ok(())
}

// --------------------------------------------------
#[test]
fn valid_type() -> Result<()> {
    for bad_type_arg in &["puush", "pul", "lgs", "sore"] {
        Command::cargo_bin(PRG)?
            .args(&["-t", bad_type_arg])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "[possible values: cooldown, core, legs, pull, push]",
            ));
    }
    Ok(())
}

// --------------------------------------------------
#[test]
fn valid_level() -> Result<()> {
    for bad_level_arg in &["beginer", "intermdiate", "advand"] {
        Command::cargo_bin(PRG)?
            .args(&["-l", bad_level_arg])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "[possible values: beginner, intermediate, advanced]",
            ));
    }
    Ok(())
}

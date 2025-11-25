use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::MyOwnError;

use crate::sources::SourceManager;

pub(crate) struct Brew;

impl SourceManager for Brew {
    fn install(&self, application: &str) -> Result<(), MyOwnError> {
        let status = Command::new("brew")
            .args(&vec!["install", application])
            .stdout(stdout())
            .stderr(stderr())
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(MyOwnError::ActualError("# Couldn't install".into()))
        }
    }

    fn uninstall(&self, application: &str) -> Result<(), MyOwnError> {
        let status = Command::new("brew")
            .args(&vec!["uninstall", application])
            .stdout(stdout())
            .stderr(stderr())
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(MyOwnError::ActualError("# Couldn't uninstall".into()))
        }
    }

    fn has_application(&self, application: &str) -> Result<bool, MyOwnError> {
        println!("# Brew: Search application...");
        let output = Command::new("brew")
            .args(&vec!["search", application])
            .output()?;

        let out = String::from_utf8(output.stdout)?;

        if output.status.success() && out.contains(application) {
            println!("# Brew: Found");
            Ok(true)
        } else {
            println!("# Brew: Nothing found");
            Ok(false)
        }
    }
}

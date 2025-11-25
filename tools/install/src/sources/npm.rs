use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::MyOwnError;

use crate::sources::SourceManager;

pub(crate) struct Npm;

impl SourceManager for Npm {
    fn install(&self, application: &str) -> Result<(), MyOwnError> {
        let status = Command::new("npm")
            .args(&vec!["install", application, "-g"])
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
        let status = Command::new("npm")
            .args(&vec!["uninstall", application, "-g"])
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
        println!("# Npm: Search application...");
        let output = Command::new("npm")
            .args(&vec!["view", application])
            .output()?;

        if output.status.success() {
            println!("# Npm: Found");
            Ok(true)
        } else {
            println!("# Npm: Nothing found");
            Ok(false)
        }
    }
}

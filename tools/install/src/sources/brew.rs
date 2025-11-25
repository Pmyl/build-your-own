use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::MyOwnError;

use crate::sources::SourceT;

pub(crate) struct Brew;

impl SourceT for Brew {
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
}

use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::MyOwnError;

use crate::sources::SourceManager;

pub(crate) struct Apt;

impl SourceManager for Apt {
    fn install(&self, application: &str) -> Result<(), MyOwnError> {
        let status = Command::new("apt")
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

    fn uninstall(&self, _: &str) -> Result<(), MyOwnError> {
        todo!("Uninstall with apt is hard, I'll do it later")
    }

    fn has_application(&self, application: &str) -> Result<bool, MyOwnError> {
        println!("# Apt: Search application...");
        let output = Command::new("apt")
            .args(&vec!["show", application])
            .output()?;

        if output.status.success() {
            println!("# Apt: Found");
            Ok(true)
        } else {
            println!("# Apt: Nothing found");
            Ok(false)
        }
    }
}

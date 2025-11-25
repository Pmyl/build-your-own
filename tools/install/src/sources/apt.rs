use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::MyOwnError;

use crate::sources::SourceT;

pub(crate) struct Apt;

impl SourceT for Apt {
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
}

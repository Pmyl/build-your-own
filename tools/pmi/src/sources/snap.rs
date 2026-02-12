use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::MyOwnError;

use crate::{ApplicationInstructions, sources::SourceManager};

pub(crate) struct Snap;

impl SourceManager for Snap {
    fn install(&self, application: &ApplicationInstructions) -> Result<(), MyOwnError> {
        let status = Command::new("snap")
            .args(
                &vec!["install", &application.application]
                    .into_iter()
                    .chain(application.args.iter().map(|arg| arg.as_ref()))
                    .collect::<Vec<_>>(),
            )
            .stdout(stdout())
            .stderr(stderr())
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(MyOwnError::ActualError("# Couldn't install".into()))
        }
    }

    fn uninstall(&self, application: &ApplicationInstructions) -> Result<(), MyOwnError> {
        let status = Command::new("snap")
            .args(&vec!["remove", &application.application])
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
        println!("# Snap: Search application...");
        let output = Command::new("snap")
            .args(&vec!["find", application])
            .output()?;

        let out = String::from_utf8(output.stdout)?;
        if !out.contains("No matching snaps") {
            println!("# Apt: Found");
            Ok(true)
        } else {
            println!("# Apt: Nothing found");
            Ok(false)
        }
    }
}

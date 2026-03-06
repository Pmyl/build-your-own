use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::{applications::ApplicationName, sources::SourceManager};

pub(crate) struct Snap;

impl SourceManager for Snap {
    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        application: &'a ApplicationName,
        args: Args,
    ) -> MyOwnResult<()> {
        let status = Command::new("snap")
            .args(
                &vec!["install", &application.0]
                    .into_iter()
                    .chain(args)
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

    fn uninstall(&self, application: &'_ ApplicationName) -> MyOwnResult<()> {
        let status = Command::new("snap")
            .args(&vec!["remove", &application.0])
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
            println!("# Snap: Found");
            Ok(true)
        } else {
            println!("# Snap: Nothing found");
            Ok(false)
        }
    }
}

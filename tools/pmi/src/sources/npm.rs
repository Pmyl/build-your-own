use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::{applications::ApplicationName, sources::SourceManager};

pub(crate) struct Npm;

impl SourceManager for Npm {
    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        application: &'a ApplicationName,
        args: Args,
    ) -> MyOwnResult<()> {
        let status = Command::new("npm")
            .args(
                &vec!["install", &application.0, "-g"]
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

    fn uninstall<'a>(&self, application: &'a ApplicationName) -> MyOwnResult<()> {
        let status = Command::new("npm")
            .args(&vec!["uninstall", &application.0, "-g"])
            .stdout(stdout())
            .stderr(stderr())
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(MyOwnError::ActualError("# Couldn't uninstall".into()))
        }
    }

    fn has_application(&self, application: &str) -> MyOwnResult<bool> {
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

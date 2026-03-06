use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::{applications::ApplicationName, sources::SourceManager};

pub(crate) struct Brew;

impl SourceManager for Brew {
    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        application: &'a ApplicationName,
        args: Args,
    ) -> MyOwnResult<()> {
        let status = Command::new("brew")
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

    fn uninstall<'a>(&self, application: &'a ApplicationName) -> MyOwnResult<()> {
        let status = Command::new("brew")
            .args(&vec!["uninstall", &application.0])
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

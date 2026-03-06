use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::{applications::ApplicationName, sources::SourceManager};

pub(crate) struct Apt;

impl SourceManager for Apt {
    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        application: &'a ApplicationName,
        args: Args,
    ) -> MyOwnResult<()> {
        let status = Command::new("apt")
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

    fn uninstall<'a>(&self, _: &'a ApplicationName) -> MyOwnResult<()> {
        todo!("Uninstall with apt is hard, I'll do it later")
    }

    fn has_application(&self, application: &str) -> MyOwnResult<bool> {
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

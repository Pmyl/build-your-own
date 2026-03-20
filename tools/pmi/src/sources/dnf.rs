use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::{applications::ApplicationName, sources::SourceManager};

pub(crate) struct Dnf;

impl SourceManager for Dnf {
    fn requires_root_permissions() -> bool {
        true
    }

    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        application: &'a ApplicationName,
        args: Args,
    ) -> MyOwnResult<()> {
        let status = Command::new("dnf")
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

    fn uninstall(&self, application: &ApplicationName) -> MyOwnResult<()> {
        let status = Command::new("dnf")
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

    fn has_application(&self, application: &str) -> MyOwnResult<bool> {
        println!("# Dnf: Search application...");

        let output = Command::new("dnf")
            .args(&vec!["search", application])
            .output()?;

        let out = String::from_utf8(output.stdout)?;

        if output.status.success()
            && out.lines().any(|line| {
                line.contains("Matched fields: ")
                    && line.contains("name")
                    && line.contains("(exact)")
            })
        {
            println!("# Dnf: Found");
            Ok(true)
        } else {
            println!("# Dnf: Nothing found");
            Ok(false)
        }
    }
}

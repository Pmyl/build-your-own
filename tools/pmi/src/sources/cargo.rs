use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::{applications::ApplicationName, sources::SourceManager};

pub(crate) struct Cargo;

impl SourceManager for Cargo {
    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        application: &'a ApplicationName,
        args: Args,
    ) -> MyOwnResult<()> {
        let status = Command::new("cargo")
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
        let status = Command::new("cargo")
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
        println!("# Cargo: Search application...");

        //curl https://crates.io/api/v1/crates/{crate} -s -o /dev/null -w "%{http_code}"
        let output = Command::new("curl")
            .args(&vec![
                &format!("https://crates.io/api/v1/crates/{}", application),
                "-s",
                "-o",
                "/dev/null",
                "-w",
                "\"%{http_code}\"",
            ])
            .output()?;

        let out = String::from_utf8(output.stdout)?;

        if output.status.success() && out == "200" {
            println!("# Cargo: Found");
            Ok(true)
        } else {
            println!("# Cargo: Nothing found");
            Ok(false)
        }
    }
}

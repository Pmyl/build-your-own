use std::{
    io::{stderr, stdout},
    process::Command,
};

use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::{ApplicationInstructions, sources::SourceManager};

pub(crate) struct Brew;

impl SourceManager for Brew {
    fn install(&self, application: &ApplicationInstructions) -> MyOwnResult<()> {
        let status = Command::new("brew")
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

    fn uninstall(&self, application: &ApplicationInstructions) -> MyOwnResult<()> {
        let status = Command::new("brew")
            .args(&vec!["uninstall", application.application.as_ref()])
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

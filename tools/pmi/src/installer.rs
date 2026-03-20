use std::borrow::Cow;

use build_your_own_utils::my_own_error::{DescribableError, MyOwnError, MyOwnResult};

use crate::{
    applications::{
        Application, ApplicationName, PersistedApplication, Persistence, RequestApplication,
    },
    ask_input, ask_permission,
    sources::{InstallError, Source, SourceInstructions, search_source_with_application},
};

pub(crate) struct Installer<'a> {
    pub persistence: Persistence<'a>,
    pub has_root_permissions: bool,
}

impl<'a> Installer<'a> {
    pub(crate) fn install_all(mut self) -> MyOwnResult<()> {
        println!("## Ready to install all applications");

        let mut new_apps = vec![];
        let mut apps_permissions_mismatch = vec![];
        for persisted_app in &self.persistence.list {
            let header = format!("## INSTALLING {} ##", persisted_app.name.0);
            let border = "#".repeat(header.len());
            println!("{}", border);
            println!("{}", header);
            println!("{}", border);
            match persisted_app.install(self.has_root_permissions) {
                Ok(Some(new_app)) => new_apps.push(new_app),
                Ok(None) => {}
                Err(InstallError::Error(e)) => return Err(e),
                Err(InstallError::PermissionsMismatch) => {
                    apps_permissions_mismatch.push(persisted_app.name.clone())
                }
            }
        }

        if apps_permissions_mismatch.is_empty() {
            self.persistence.add_many(new_apps)?;
            println!("## All applications installed");
        } else {
            println!(
                "## Some ({}) applications installed and some ({}) not",
                new_apps.len(),
                apps_permissions_mismatch.len()
            );
            self.persistence.add_many(new_apps)?;

            println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@");
            println!(
                "@ Root permissions mismatch: Needed {} received {}",
                !self.has_root_permissions, self.has_root_permissions
            );
            println!(
                "@ Rerun install all with opposite root
                permissions to install the following applications"
            );
            for (i, app) in apps_permissions_mismatch.iter().enumerate() {
                println!("@@ {}: {}", i + 1, app.0);
            }
        }

        Ok(())
    }

    pub(crate) fn uninstall_all(mut self) -> MyOwnResult<()> {
        println!("## Ready to uninstall all applications");

        let mut apps_permissions_mismatch = vec![];
        let mut uninstalled = vec![];
        for persisted_app in &self.persistence.list {
            println!("## Uninstalling {}", persisted_app.name.0);

            match persisted_app.uninstall(self.has_root_permissions) {
                Ok(_) => uninstalled.push(persisted_app.name.clone()),
                Err(InstallError::Error(e)) => return Err(e),
                Err(InstallError::PermissionsMismatch) => {
                    apps_permissions_mismatch.push(persisted_app.name.clone())
                }
            }
        }

        if apps_permissions_mismatch.is_empty() {
            println!("## All applications uninstalled");
            self.persistence.remove_all()?;
            println!("## Applications removed from list");
        } else {
            println!(
                "## Some ({}) applications uninstalled and some ({}) retained",
                uninstalled.len(),
                apps_permissions_mismatch.len()
            );
            self.persistence.remove_many(uninstalled)?;

            println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@");
            println!(
                "@ Root permissions mismatch: Needed {} received {}",
                !self.has_root_permissions, self.has_root_permissions
            );
            println!(
                "@ Rerun uninstall all with opposite root
                permissions to uninstall the following applications"
            );
            for (i, app) in apps_permissions_mismatch.iter().enumerate() {
                println!("@@ {}: {}", i + 1, app.0);
            }
        }

        Ok(())
    }

    pub(crate) fn install(mut self, request: RequestApplication<'a>) -> Result<(), InstallError> {
        let already_installed = self.persistence.is_already_installed(&request);

        if let Some(_) = already_installed.perfect_match {
            return Err(
                MyOwnError::ActualError(format!("Application already installed").into()).into(),
            );
        }

        if already_installed.similar_matches.len() > 0 {
            println!("# Found installed applications with a similar name");
            for similar in already_installed.similar_matches {
                println!("## {}", similar.name.0);
            }

            ask_permission(&format!(
                "# Do you want to still install `{}`? (Y/n)",
                request.name.0
            ))?;
        }

        let application = match triage_application(request)? {
            ApplicationTriage::NotFound => {
                println!("Found no matches, aborting");
                Err(MyOwnError::EarlyExit)
            }
            ApplicationTriage::Skipped => {
                println!("Skipped, aborting");
                Err(MyOwnError::EarlyExit)
            }
            ApplicationTriage::Found(application) => Ok(application),
        }?;

        println!("## Ready to install application");

        application.install(self.has_root_permissions)?;

        println!("## Application installed");

        self.persistence.add(application)?;

        println!("## Application added to list");

        Ok(())
    }

    pub(crate) fn uninstall(mut self, request: RequestApplication<'a>) -> Result<(), InstallError> {
        println!("## Ready to uninstall application");

        match request.as_application() {
            None => {
                // Uninstall all matching
                let application_to_remove =
                    self.persistence.get_application_by_name(&request.name.0);

                let Some(persisted_app) = application_to_remove else {
                    return Err(
                        MyOwnError::ActualError("Application is not installed".into()).into(),
                    );
                };

                println!("## Found application to uninstall",);
                println!("## Uninstalling {} from all sources", persisted_app.name.0);

                persisted_app.uninstall(self.has_root_permissions)?;
                println!("## Application uninstalled");

                self.persistence.remove(&persisted_app.name.clone())?;

                println!("## Application removed from list");
            }
            Some(application) => {
                // Uninstall perfect match
                if let None = self
                    .persistence
                    .is_already_installed(&request)
                    .perfect_match
                {
                    return Err(MyOwnError::ActualError(
                        "Application is not installed through requested source".into(),
                    )
                    .into());
                }

                println!("## Uninstalling {}", application);

                application.uninstall(self.has_root_permissions)?;
                println!("## Application uninstalled");

                self.persistence.remove_specific(&application)?;

                println!("## Application removed from list");
            }
        }

        Ok(())
    }
}

pub(crate) enum ApplicationConfirmation {
    Confirmed(usize),
    Skipped,
}

pub(crate) fn confirm_application<'a>(
    applications: &[Application<'a>],
) -> MyOwnResult<ApplicationConfirmation> {
    if let [app] = applications {
        println!("# Found only in {}", app.source_instruction.source);
        println!();
        match ask_permission(&format!(
            "# Do you want to install it using {}? ([Y]es/[s]kip)",
            app.source_instruction.source
        )) {
            Ok(_) => Ok(ApplicationConfirmation::Confirmed(0)),
            Err(MyOwnError::EarlyExit) => Ok(ApplicationConfirmation::Skipped),
            Err(err) => Err(err),
        }
    } else {
        println!("# Found {} alternatives", applications.len());
        for (i, app) in applications.iter().enumerate() {
            println!("{}. {}", i + 1, app.source_instruction.source);
        }
        println!();
        let answer = ask_input(&format!("# Select one with a number or [s]kip",))?;
        let answer = answer.trim();

        let answer = if answer.is_empty() || answer.to_lowercase() == "s" {
            return Ok(ApplicationConfirmation::Skipped);
        } else {
            answer
                .parse::<usize>()
                .error_description("Answer should be a number")?
        };

        if answer == 0 || answer > applications.len() {
            Err(MyOwnError::ActualError("Answer outside range".into()))
        } else {
            Ok(ApplicationConfirmation::Confirmed(answer - 1))
        }
    }
}

pub(crate) fn identify_application<'a, 'b>(
    application: &'b str,
) -> MyOwnResult<Vec<Application<'a>>> {
    Ok(search_source_with_application(application)?
        .into_iter()
        .map(|s| Application {
            source_instruction: SourceInstructions {
                source: s,
                args: vec![],
            },
            name: ApplicationName(Cow::Owned(application.to_string())),
        })
        .collect::<Vec<_>>())
}

fn triage_application<'a>(request: RequestApplication<'a>) -> MyOwnResult<ApplicationTriage<'a>> {
    match request.as_application() {
        None => {
            let mut possible_matches = identify_application(&request.name.0)?;
            match possible_matches.len() {
                0 => Ok(ApplicationTriage::NotFound),
                _ => match confirm_application(&possible_matches)? {
                    ApplicationConfirmation::Confirmed(index) => {
                        Ok(ApplicationTriage::Found(possible_matches.remove(index)))
                    }
                    ApplicationConfirmation::Skipped => Ok(ApplicationTriage::Skipped),
                },
            }
        }
        Some(application) => Ok(ApplicationTriage::Found(application)),
    }
}

pub(crate) enum ApplicationTriage<'a> {
    NotFound,
    Skipped,
    Found(Application<'a>),
}

trait InstallableApplication {
    fn install(&self, has_root_permissions: bool) -> Result<(), InstallError>;
    fn uninstall(&self, has_root_permissions: bool) -> Result<(), InstallError>;
}

impl<'a> InstallableApplication for Application<'a> {
    fn install(&self, has_root_permissions: bool) -> Result<(), InstallError> {
        self.source_instruction.source.install(
            &self.name,
            self.source_instruction.args.iter().map(|arg| arg.as_ref()),
            has_root_permissions,
        )
    }

    fn uninstall(&self, has_root_permissions: bool) -> Result<(), InstallError> {
        self.source_instruction
            .source
            .uninstall(&self.name, has_root_permissions)
    }
}

trait InstallablePersistedApplication<'a> {
    // Return Some(Application) if new source is used
    fn install(&self, has_root_permissions: bool) -> Result<Option<Application<'a>>, InstallError>;
    fn uninstall(&self, has_root_permissions: bool) -> Result<(), InstallError>;
}

impl<'a> InstallablePersistedApplication<'a> for PersistedApplication<'a> {
    fn install(&self, has_root_permissions: bool) -> Result<Option<Application<'a>>, InstallError> {
        for source_instruction in &self.source_instructions {
            if let Source::Unknown(name) = &source_instruction.source {
                println!("# Skipping source {} because not in use", name);
                continue;
            }

            println!("# Installing with source {}", source_instruction.source);
            source_instruction.source.install(
                &self.name,
                source_instruction.args.iter().map(|arg| arg.as_ref()),
                has_root_permissions,
            )?;
            break;
        }
        println!(
            "# No sources used to install this application are in use, searching for new source"
        );

        let application = match triage_application(RequestApplication {
            name: self.name.clone(),
            source_instruction: None,
        })? {
            ApplicationTriage::NotFound => {
                println!("Found no matches, skipping");
                return Ok(None);
            }
            ApplicationTriage::Skipped => {
                println!("Skipped");
                return Ok(None);
            }
            ApplicationTriage::Found(application) => application,
        };

        println!("## Ready to install application");

        application.install(has_root_permissions)?;

        println!("## Application installed");

        if self
            .source_instructions
            .contains(&application.source_instruction)
        {
            Ok(None)
        } else {
            Ok(Some(application))
        }
    }

    fn uninstall(&self, has_root_permissions: bool) -> Result<(), InstallError> {
        for source_instruction in &self.source_instructions {
            if let Source::Unknown(name) = &source_instruction.source {
                println!("# Skipping source {} because not in use", name);
                continue;
            }
            source_instruction
                .source
                .uninstall(&self.name, has_root_permissions)?;
        }
        Ok(())
    }
}

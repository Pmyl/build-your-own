use std::borrow::Cow;

use build_your_own_utils::my_own_error::{DescribableError, MyOwnError, MyOwnResult};

use crate::{
    applications::{
        Application, ApplicationName, PersistedApplication, Persistence, RequestApplication,
    },
    ask_input, ask_permission,
    sources::{Source, SourceInstructions, search_source_with_application},
};

pub(crate) struct Installer<'a>(pub Persistence<'a>);

impl<'a> Installer<'a> {
    pub(crate) fn install_all(mut self) -> MyOwnResult<()> {
        println!("## Ready to install all applications");

        let mut new_apps = vec![];
        for persisted_app in &self.0.list {
            println!("## Installing {}", persisted_app.name.0);
            if let Some(new_app) = persisted_app.install()? {
                new_apps.push(new_app);
            }
        }

        self.0.add_many(new_apps)?;

        println!("## All applications installed");

        Ok(())
    }

    pub(crate) fn uninstall_all(mut self) -> MyOwnResult<()> {
        println!("## Ready to uninstall all applications");

        for persisted_app in &self.0.list {
            println!("## Uninstalling {}", persisted_app.name.0);
            persisted_app.uninstall()?
        }

        println!("## All applications uninstalled");

        self.0.remove_all()?;

        println!("## Application removed from list");

        Ok(())
    }

    pub(crate) fn install(mut self, request: RequestApplication<'a>) -> MyOwnResult<()> {
        let already_installed = self.0.is_already_installed(&request);

        if let Some(_) = already_installed.perfect_match {
            return Err(MyOwnError::ActualError(
                format!("Application already installed").into(),
            ));
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
                return Err(MyOwnError::EarlyExit);
            }
            ApplicationTriage::Skipped => {
                println!("Skipped, aborting");
                return Err(MyOwnError::EarlyExit);
            }
            ApplicationTriage::Found(application) => application,
        };

        println!("## Ready to install application");

        application.install()?;

        println!("## Application installed");

        self.0.add(application)?;

        println!("## Application added to list");

        Ok(())
    }

    pub(crate) fn uninstall(mut self, request: RequestApplication<'a>) -> MyOwnResult<()> {
        println!("## Ready to uninstall application");

        match request.as_application() {
            None => {
                // Uninstall all matching
                let application_to_remove = self.0.get_application_by_name(&request.name.0);

                let Some(persisted_app) = application_to_remove else {
                    return Err(MyOwnError::ActualError(
                        "Application is not installed".into(),
                    ));
                };

                println!("## Found application to uninstall",);
                println!("## Uninstalling {} from all sources", persisted_app.name.0);

                persisted_app.uninstall()?;
                println!("## Application uninstalled");

                self.0.remove(&persisted_app.name.clone())?;

                println!("## Application removed from list");
            }
            Some(application) => {
                // Uninstall perfect match
                if let None = self.0.is_already_installed(&request).perfect_match {
                    return Err(MyOwnError::ActualError(
                        "Application is not installed through requested source".into(),
                    ));
                }

                println!("## Uninstalling {}", application);

                application.uninstall()?;
                println!("## Application uninstalled");

                self.0.remove_specific(&application)?;

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
    fn install(&self) -> MyOwnResult<()>;
    fn uninstall(&self) -> MyOwnResult<()>;
}

impl<'a> InstallableApplication for Application<'a> {
    fn install(&self) -> MyOwnResult<()> {
        self.source_instruction.source.install(
            &self.name,
            self.source_instruction.args.iter().map(|arg| arg.as_ref()),
        )
    }

    fn uninstall(&self) -> MyOwnResult<()> {
        self.source_instruction.source.uninstall(&self.name)
    }
}

trait InstallablePersistedApplication<'a> {
    // Return Some(Application) if new source is used
    fn install(&self) -> MyOwnResult<Option<Application<'a>>>;
    fn uninstall(&self) -> MyOwnResult<()>;
}

impl<'a> InstallablePersistedApplication<'a> for PersistedApplication<'a> {
    fn install(&self) -> MyOwnResult<Option<Application<'a>>> {
        for source_instruction in &self.source_instructions {
            if let Source::Unknown(name) = &source_instruction.source {
                println!("# Skipping source {} because not in use", name);
                continue;
            }

            println!("# Installing with source {}", source_instruction.source);
            source_instruction.source.install(
                &self.name,
                source_instruction.args.iter().map(|arg| arg.as_ref()),
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

        application.install()?;

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

    fn uninstall(&self) -> MyOwnResult<()> {
        for source_instruction in &self.source_instructions {
            if let Source::Unknown(name) = &source_instruction.source {
                println!("# Skipping source {} because not in use", name);
                continue;
            }
            source_instruction.source.uninstall(&self.name)?;
        }
        Ok(())
    }
}

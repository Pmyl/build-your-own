mod sources;

use std::{
    borrow::Cow,
    fs::OpenOptions,
    io::{Read, Write, stdin},
};

use build_your_own_macros::cli_options;
use build_your_own_utils::{
    fuzzy_search::fuzzy_search,
    my_own_error::{DescribableError, MyOwnError},
};

use crate::sources::{Source, search_source_with_application};

// My idea!
pub fn install_cli(args: &[&str]) -> Result<(), MyOwnError> {
    let options = InstallOptions::from_args(args)?;

    if options.show_path {
        println!("{}", applications_file());
        return Ok(());
    }

    let application = identify_application(options.source, options.application)?;

    let applications = if options.applications_file_from_stdin {
        Applications::from_reader(stdin())
    } else {
        Applications::from_file()
    }?;
    match (application, options.all, options.uninstall) {
        (Some(application), false, false) => Installer(applications).install(application),
        (Some(application), false, true) => Installer(applications).uninstall(application),
        (None, true, false) => Installer(applications).install_all(),
        (None, true, true) => Installer(applications).uninstall_all(),
        _ => {
            println!("Usage: myown install [-p] [-u] [-a] [application] [source]");
            println!();
            println!("Arguments:");
            println!("  application    Name of app/package to install, mandatory without -a");
            println!("  source         What to use to install the app/package, optional");
            println!("                   If not provided it will find it and ask for confirmation");
            println!("Flags:");
            println!("  -u             Uninstall");
            println!("  -i             Read list of applications from stdin");
            println!("                   instead of {}", applications_file());
            println!("Alternate modes:");
            println!("  -p             Print path of file with list of applications");
            println!("  -a             Install/Uninstall all applications presents in the list");
            println!();
            println!("Example install: myown install tailwindcss npm");
            println!("Example uninstall: myown install -u tailwindcss npm");
            println!();
            if applications.0.len() > 0 {
                println!("# Apps installed [{}]:", applications_file());
                for app in applications.0 {
                    println!("{} | {}", app.source, app.application);
                }
            } else {
                println!("# No apps installed [{}]", applications_file());
            }
            return Ok(());
        }
    }
}

fn identify_application<'a>(
    source: Option<Source>,
    application: Option<&'a str>,
) -> Result<Option<Application<'a>>, MyOwnError> {
    match (source, application) {
        (Some(source), Some(application)) => Ok(Some(Application::new(source, application))),
        (None, Some(application)) => {
            let mut sources_with_app = search_source_with_application(application)?;
            if sources_with_app.is_empty() {
                println!("# Found no sources with requested application, aborting");
                return Err(MyOwnError::EarlyExit);
            }

            println!();
            println!(
                "# Found {} sources with application {}",
                sources_with_app.len(),
                application
            );
            for (i, source) in sources_with_app.iter().enumerate() {
                println!("{}. {}", i + 1, source);
            }
            println!();
            let answer = ask_input(&format!(
                "# Which one do you want to use to install {}?",
                application
            ))?
            .trim()
            .parse::<usize>()
            .error_description("Answer should be a number")?;

            if answer == 0 || answer > sources_with_app.len() {
                Err(MyOwnError::ActualError("Answer outside range".into()))
            } else {
                Ok(Some(Application::new(
                    sources_with_app.remove(answer - 1),
                    application,
                )))
            }
        }
        _ => Ok(None),
    }
}

struct Installer<'a>(Applications<'a>);

impl<'a> Installer<'a> {
    fn install_all(self) -> Result<(), MyOwnError> {
        println!("# Applications list from [{}]", applications_file());
        ask_permission(&format!(
            "# This operation will not modify the list of installed applications. Do you want to install {} applications? Y/n",
            self.0.0.len()
        ))?;

        println!("## Ready to install all applications");

        for application in self.0.0 {
            println!("## Installing {}", application.application);
            application.source.install(&application.application)?;
        }

        println!("## All applications installed");

        Ok(())
    }

    fn uninstall_all(mut self) -> Result<(), MyOwnError> {
        println!("# Applications list from [{}]", applications_file());
        ask_permission(&format!(
            "# This will also remove all the applications from the list of installed applications. Do you want to uninstall {} applications? Y/n",
            self.0.0.len()
        ))?;

        println!("## Ready to uninstall all applications");

        for application in &self.0.0 {
            application.source.uninstall(&application.application)?;
        }

        println!("## All applications uninstalled");

        self.0.remove_all()?;

        println!("## Application removed from list");

        Ok(())
    }

    fn install(mut self, application: Application<'a>) -> Result<(), MyOwnError> {
        let already_installed = self
            .0
            .is_already_installed(&application.source, &application.application);

        if let Some(_) = already_installed.perfect_match {
            return Err(MyOwnError::ActualError(
                format!("Application already installed").into(),
            ));
        }

        if already_installed.similar_matches.len() > 0 {
            println!("# Found installed applications with a similar name");
            for similar in already_installed.similar_matches {
                println!("## {} | {}", similar.source, similar.application);
            }

            ask_permission(&format!(
                "# Do you want to still install {} | {}? Y/n",
                application.source, application.application
            ))?;
        }

        println!("## Ready to install application");

        application.source.install(&application.application)?;

        println!("## Application installed");

        self.0.add(application)?;

        println!("## Application added to list");

        Ok(())
    }

    fn uninstall(mut self, application: Application<'a>) -> Result<(), MyOwnError> {
        let Some(existing_application) = self.0.get_application_by_name(&application.application)
        else {
            return Err(MyOwnError::ActualError(
                "Application is not installed".into(),
            ));
        };

        if existing_application.source != application.source {
            return Err(MyOwnError::ActualError(
                format!(
                    "Application is not installed through {} but through {}",
                    application.source, existing_application.source
                )
                .into(),
            ));
        }

        println!("## Ready to uninstall application");

        application.source.uninstall(&application.application)?;

        println!("## Application uninstalled");

        self.0.remove(application)?;

        println!("## Application removed from list");

        Ok(())
    }
}

fn ask_permission(question: &str) -> Result<(), MyOwnError> {
    println!("{}", question);
    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    Ok(if answer.trim().to_lowercase() == "n" {
        return Err(MyOwnError::EarlyExit);
    })
}

fn ask_input(question: &str) -> Result<String, MyOwnError> {
    println!("{}", question);
    let mut answer = String::new();
    stdin().read_line(&mut answer)?;
    Ok(answer)
}

struct Applications<'a>(Vec<Application<'a>>);

struct Application<'a> {
    source: Source,
    application: Cow<'a, str>,
}

impl<'a> Application<'a> {
    fn new(source: Source, application: &'a str) -> Self {
        Self {
            source,
            application: Cow::Borrowed(application),
        }
    }
}

fn applications_folder() -> String {
    format!("{}/.my-own-installer", std::env::var("HOME").unwrap())
}

fn applications_file() -> String {
    format!(
        "{}/.my-own-installer/applications",
        std::env::var("HOME").unwrap()
    )
}

impl<'a> Applications<'a> {
    fn from_file() -> Result<Self, MyOwnError> {
        std::fs::create_dir_all(applications_folder())?;
        let file_reader = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(applications_file())
            .with_error_description(|| format!("Error while loading {}", applications_file()))?;

        Applications::from_reader(file_reader)
    }

    fn from_reader(mut reader: impl Read) -> Result<Self, MyOwnError> {
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .with_error_description(|| format!("Error while reading applications"))?;

        let mut applications = vec![];
        for line in content.lines() {
            let mut parts = line.split("|");
            let source = parts
                .next()
                .ok_or_else(|| {
                    MyOwnError::ActualError(
                        "Content of applications does not contain source".into(),
                    )
                })?
                .parse()?;
            let application = parts
                .next()
                .ok_or_else(|| {
                    MyOwnError::ActualError(
                        "Content of applications does not contain application".into(),
                    )
                })?
                .to_string();

            applications.push(Application {
                source,
                application: Cow::Owned(application),
            });
        }

        Ok(Applications(applications))
    }

    fn add(&mut self, application: Application<'a>) -> Result<(), MyOwnError> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(applications_file())
            .with_error_description(|| format!("Error while loading {}", applications_file()))?;

        writeln!(file, "{}|{}", application.source, application.application)?;
        file.flush()?;

        self.0.push(application);

        Ok(())
    }

    fn remove(&mut self, application: Application<'a>) -> Result<(), MyOwnError> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .open(applications_file())
            .with_error_description(|| format!("Error while loading {}", applications_file()))?;

        let index = self
            .0
            .iter()
            .position(|app| app.application == application.application)
            .ok_or_else(|| MyOwnError::ActualError("Couldn't find application to remove".into()))?;
        self.0.remove(index);

        for app in &self.0 {
            writeln!(file, "{}|{}", app.source, app.application)?;
        }
        file.flush()?;

        Ok(())
    }

    fn remove_all(&mut self) -> Result<(), MyOwnError> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .open(applications_file())
            .with_error_description(|| format!("Error while loading {}", applications_file()))?;
        file.flush()?;

        Ok(())
    }

    fn get_application_by_name(&self, application: &str) -> Option<&Application<'a>> {
        self.0.iter().find(|app| app.application == application)
    }

    fn get_application(&self, source: &Source, application: &str) -> Option<&Application<'a>> {
        self.0
            .iter()
            .find(|app| &app.source == source && app.application == application)
    }

    fn is_already_installed(
        &'a self,
        source: &'a Source,
        application: &'a str,
    ) -> AlreadyInstalled<'a> {
        let indices = fuzzy_search(
            application,
            &self
                .0
                .iter()
                .map(|a| a.application.as_ref())
                .collect::<Vec<_>>(),
            3,
        );

        let similar_matches = indices
            .into_iter()
            .map(|i| self.0.get(i).unwrap())
            .collect::<Vec<_>>();

        let perfect_match = self.get_application(source, application);

        AlreadyInstalled {
            perfect_match,
            similar_matches,
        }
    }
}

struct AlreadyInstalled<'a> {
    perfect_match: Option<&'a Application<'a>>,
    similar_matches: Vec<&'a Application<'a>>,
}

cli_options! {
    struct InstallOptions<'a> {
        #[option(name = "-p")]
        show_path: bool,

        #[option(name = "-u")]
        uninstall: bool,

        #[option(name = "-i")]
        applications_file_from_stdin: bool,

        #[option(name = "-a")]
        all: bool,

        #[option()]
        application: Option<&'a str>,

        #[option()]
        source: Option<Source>,
    }
}

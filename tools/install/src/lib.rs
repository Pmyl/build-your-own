use std::{
    borrow::Cow,
    fmt::Display,
    fs::OpenOptions,
    io::{Read, Write, stderr, stdin, stdout},
    process::Command,
    str::FromStr,
};

use build_your_own_macros::cli_options;
use build_your_own_utils::{
    fuzzy_search::fuzzy_search,
    my_own_error::{DescribableError, MyOwnError},
};

// My idea!
pub fn install_cli(args: &[&str]) -> Result<(), MyOwnError> {
    let options = InstallOptions::from_args(args)?;

    let application = match (options.source, options.application) {
        (Some(source), Some(application)) => Some(Application {
            source,
            application: Cow::Borrowed(application),
        }),
        _ => None,
    };

    let installer = Installer;
    match (application, options.all, options.uninstall) {
        (Some(application), false, false) => installer.install(application),
        (Some(application), false, true) => installer.uninstall(application),
        (None, true, false) => installer.install_all(),
        (None, true, true) => installer.uninstall_all(),
        _ => {
            println!("Usage: myown install [-u] [-a] [source] [application]");
            println!("Example install: myown install npm tailwindcss");
            println!("Example uninstall: myown install -u npm tailwindcss");
            println!();
            let applications = Applications::read()?;
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

fn install_with_apt(application: &str) -> Result<(), MyOwnError> {
    let status = Command::new("apt")
        .args(&vec!["install", application])
        .stdout(stdout())
        .stderr(stderr())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(MyOwnError::ActualError("# Couldn't install".into()))
    }
}

fn install_with_npm(application: &str) -> Result<(), MyOwnError> {
    let status = Command::new("npm")
        .args(&vec!["install", application, "-g"])
        .stdout(stdout())
        .stderr(stderr())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(MyOwnError::ActualError("# Couldn't install".into()))
    }
}

fn install_with_cargo(application: &str) -> Result<(), MyOwnError> {
    let status = Command::new("cargo")
        .args(&vec!["install", application])
        .stdout(stdout())
        .stderr(stderr())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(MyOwnError::ActualError("# Couldn't install".into()))
    }
}

fn install_with_brew(application: &str) -> Result<(), MyOwnError> {
    let status = Command::new("brew")
        .args(&vec!["install", application])
        .stdout(stdout())
        .stderr(stderr())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(MyOwnError::ActualError("# Couldn't install".into()))
    }
}

fn uninstall_with_npm(application: &str) -> Result<(), MyOwnError> {
    let status = Command::new("npm")
        .args(&vec!["uninstall", application, "-g"])
        .stdout(stdout())
        .stderr(stderr())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(MyOwnError::ActualError("# Couldn't uninstall".into()))
    }
}

fn uninstall_with_apt(_application: &str) -> Result<(), MyOwnError> {
    todo!("Uninstall with apt is hard, I'll do it later")
}

fn uninstall_with_cargo(application: &str) -> Result<(), MyOwnError> {
    let status = Command::new("cargo")
        .args(&vec!["uninstall", application])
        .stdout(stdout())
        .stderr(stderr())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(MyOwnError::ActualError("# Couldn't uninstall".into()))
    }
}

fn uninstall_with_brew(application: &str) -> Result<(), MyOwnError> {
    let status = Command::new("brew")
        .args(&vec!["uninstall", application])
        .stdout(stdout())
        .stderr(stderr())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(MyOwnError::ActualError("# Couldn't uninstall".into()))
    }
}

struct Installer;

impl Installer {
    fn install_all(self) -> Result<(), MyOwnError> {
        let all_applications = Applications::read()?;
        println!("# Applications list from [{}]", applications_file());
        ask_permission(&format!(
            "# This operation will not modify the list of installed applications. Do you want to install {} applications? Y/n",
            all_applications.0.len()
        ))?;

        println!("## Ready to install all applications");

        for application in all_applications.0 {
            println!("## Installing {}", application.application);
            install_through_source(&application.source, application.application.as_ref())?;
        }

        println!("## All applications installed");

        Ok(())
    }

    fn uninstall_all(self) -> Result<(), MyOwnError> {
        let mut all_applications = Applications::read()?;
        println!("# Applications list from [{}]", applications_file());
        ask_permission(&format!(
            "# This will also remove all the applications from the list of installed applications. Do you want to uninstall {} applications? Y/n",
            all_applications.0.len()
        ))?;

        println!("## Ready to uninstall all applications");

        for application in &all_applications.0 {
            uninstall_through_source(&application.source, application.application.as_ref())?;
        }

        println!("## All applications uninstalled");

        all_applications.remove_all()?;

        println!("## Application removed from list");

        Ok(())
    }

    fn install<'a>(self, application: Application<'a>) -> Result<(), MyOwnError> {
        let mut all_applications = Applications::read()?;

        let already_installed =
            all_applications.is_already_installed(&application.source, &application.application);

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

        install_through_source(&application.source, application.application.as_ref())?;

        println!("## Application installed");

        all_applications.add(application)?;

        println!("## Application added to list");

        Ok(())
    }

    fn uninstall<'a>(self, application: Application<'a>) -> Result<(), MyOwnError> {
        let mut all_applications = Applications::read()?;
        let Some(existing_application) =
            all_applications.get_application_by_name(&application.application)
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

        uninstall_through_source(&application.source, application.application.as_ref())?;

        println!("## Application uninstalled");

        all_applications.remove(application)?;

        println!("## Application removed from list");

        Ok(())
    }
}

fn install_through_source(source: &Source, application: &str) -> Result<(), MyOwnError> {
    match &source {
        Source::Apt => install_with_apt(&application),
        Source::Brew => install_with_brew(&application),
        Source::Cargo => install_with_cargo(&application),
        Source::Npm => install_with_npm(&application),
    }
}

fn uninstall_through_source(source: &Source, application: &str) -> Result<(), MyOwnError> {
    match &source {
        Source::Apt => uninstall_with_apt(&application),
        Source::Brew => uninstall_with_brew(&application),
        Source::Cargo => uninstall_with_cargo(&application),
        Source::Npm => uninstall_with_npm(&application),
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

struct Applications<'a>(Vec<Application<'a>>);

struct Application<'a> {
    source: Source,
    application: Cow<'a, str>,
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
    fn read() -> Result<Self, MyOwnError> {
        std::fs::create_dir_all(applications_folder())?;
        let mut content = String::new();
        OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(applications_file())
            .with_error_description(|| format!("Error while loading {}", applications_file()))?
            .read_to_string(&mut content)
            .with_error_description(|| format!("Error while reading {}", applications_file()))?;

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

#[derive(PartialEq)]
enum Source {
    Apt,
    Brew,
    Cargo,
    Npm,
}

impl FromStr for Source {
    type Err = MyOwnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "apt" => Ok(Source::Apt),
            "brew" => Ok(Source::Brew),
            "cargo" => Ok(Source::Cargo),
            "npm" => Ok(Source::Npm),
            _ => Err(format!("{} is not a source", s).into()),
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Apt => write!(f, "apt"),
            Source::Brew => write!(f, "brew"),
            Source::Cargo => write!(f, "cargo"),
            Source::Npm => write!(f, "npm"),
        }
    }
}

cli_options! {
    struct InstallOptions<'a> {
        #[option(name = "-u")]
        uninstall: bool,

        #[option()]
        source: Option<Source>,

        #[option(name = "-a")]
        all: bool,

        #[option()]
        application: Option<&'a str>,
    }
}

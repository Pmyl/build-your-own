mod sources;

use std::{
    borrow::Cow,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write, stdin},
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

    let application = identify_application(options.source, options.application, options.args)?;

    let applications = if options.applications_file_from_stdin {
        Applications::from_stdin()
    } else {
        Applications::from_file()
    }?;
    match (application, options.all, options.uninstall) {
        (Some(application), false, false) => Installer(applications).install(application),
        (Some(application), false, true) => Installer(applications).uninstall(application),
        (None, true, false) => Installer(applications).install_all(),
        (None, true, true) => Installer(applications).uninstall_all(),
        _ => {
            println!("Usage: myown install [-p] [-a] [-u] [-i] [application] [source] [--args]");
            println!();
            println!("Arguments:");
            println!("  application    Name of app/package to install, mandatory without -a");
            println!("  source         What to use to install the app/package, optional");
            println!("                   If not provided it will find it and ask for confirmation");
            println!("  --args         Arguments to pass to the installer, pipe delimited");
            println!("Flags:");
            println!("  -u             Uninstall");
            println!("  -i             Read list of applications from stdin");
            println!("                   instead of {}", applications_file());
            println!("Alternate modes:");
            println!("  -p             Print path of file with list of applications");
            println!("  -a             Install/Uninstall all applications presents in the list");
            println!();
            println!("Example install: myown install tailwindcss npm");
            println!(
                "Example install with args: myown install dx-cli cargo --args \"--no-default-features|--features|web,server\""
            );
            println!("Example uninstall: myown install -u tailwindcss npm");
            println!();
            if applications.list.len() > 0 {
                println!("# Apps installed [{}]:", applications_file());
                for app in applications.list {
                    println!("{} | {}", app.source, app.instructions.application);
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
    args: Vec<&'a str>,
) -> Result<Option<Application<'a>>, MyOwnError> {
    match (source, application) {
        (Some(source), Some(application)) => Ok(Some(Application::new(source, application, args))),
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
                    args,
                )))
            }
        }
        _ => Ok(None),
    }
}

struct Installer<'a>(Applications<'a>);

impl<'a> Installer<'a> {
    fn install_all(self) -> Result<(), MyOwnError> {
        println!("# Applications list from [{}]", self.0.list_source());
        ask_permission(&format!(
            "# This operation will not modify the list of installed applications. Do you want to install {} applications? Y/n",
            self.0.list.len()
        ))?;

        println!("## Ready to install all applications");

        for application in self.0.list {
            println!("## Installing {}", application.instructions.application);
            application.source.install(&application.instructions)?;
        }

        println!("## All applications installed");

        Ok(())
    }

    fn uninstall_all(mut self) -> Result<(), MyOwnError> {
        println!("# Applications list from [{}]", self.0.list_source());
        ask_permission(&format!(
            "# This will also remove all the applications from the list of installed applications. Do you want to uninstall {} applications? Y/n",
            self.0.list.len()
        ))?;

        println!("## Ready to uninstall all applications");

        for application in &self.0.list {
            application.source.uninstall(&application.instructions)?;
        }

        println!("## All applications uninstalled");

        self.0.remove_all()?;

        println!("## Application removed from list");

        Ok(())
    }

    fn install(mut self, application: Application<'a>) -> Result<(), MyOwnError> {
        let already_installed = self
            .0
            .is_already_installed(&application.source, &application.instructions.application);

        if let Some(_) = already_installed.perfect_match {
            return Err(MyOwnError::ActualError(
                format!("Application already installed").into(),
            ));
        }

        if already_installed.similar_matches.len() > 0 {
            println!("# Found installed applications with a similar name");
            for similar in already_installed.similar_matches {
                println!(
                    "## {} | {}",
                    similar.source, similar.instructions.application
                );
            }

            ask_permission(&format!(
                "# Do you want to still install {} | {}? Y/n",
                application.source, application.instructions.application
            ))?;
        }

        println!("## Ready to install application");

        application.source.install(&application.instructions)?;

        println!("## Application installed");

        self.0.add(application)?;

        println!("## Application added to list");

        Ok(())
    }

    fn uninstall(mut self, application: Application<'a>) -> Result<(), MyOwnError> {
        let Some(existing_application) = self
            .0
            .get_application_by_name(&application.instructions.application)
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

        application.source.uninstall(&application.instructions)?;

        println!("## Application uninstalled");

        self.0.remove(application)?;

        println!("## Application removed from list");

        Ok(())
    }
}

fn ask_permission(question: &str) -> Result<(), MyOwnError> {
    ask_input(question).and_then(|answer| {
        Ok(if answer.trim().to_lowercase() == "n" {
            return Err(MyOwnError::EarlyExit);
        })
    })
}

fn ask_input(question: &str) -> Result<String, MyOwnError> {
    let mut tty = BufReader::new(File::open("/dev/tty")?);
    let mut tty_out = File::create("/dev/tty")?;

    writeln!(tty_out, "{}", question)?;
    tty_out.flush()?;

    let mut answer = String::new();
    tty.read_line(&mut answer)?;
    Ok(answer)
}

struct Applications<'a> {
    list: Vec<Application<'a>>,
    list_source: String,
}

struct Application<'a> {
    source: Source,
    instructions: ApplicationInstructions<'a>,
}

struct ApplicationInstructions<'a> {
    application: Cow<'a, str>,
    args: Vec<Cow<'a, str>>,
}

impl<'a> Application<'a> {
    fn new(source: Source, application: &'a str, args: Vec<&'a str>) -> Self {
        Self {
            source,
            instructions: ApplicationInstructions {
                application: Cow::Borrowed(application),
                args: args
                    .into_iter()
                    .map(|arg| Cow::Borrowed(arg))
                    .collect::<Vec<_>>(),
            },
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

        let list = Applications::<'a>::list_from_reader(file_reader)?;

        Ok(Applications {
            list,
            list_source: applications_file(),
        })
    }

    fn from_stdin() -> Result<Self, MyOwnError> {
        let list = Applications::<'a>::list_from_reader(stdin())?;

        Ok(Applications {
            list,
            list_source: "stdin".to_string(),
        })
    }

    fn list_from_reader(mut reader: impl Read) -> Result<Vec<Application<'a>>, MyOwnError> {
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
            let args = parts
                .next()
                .unwrap_or("")
                .split('&')
                .map(|arg| Cow::Owned(arg.to_string()))
                .collect::<Vec<_>>();

            applications.push(Application {
                source,
                instructions: ApplicationInstructions {
                    application: Cow::Owned(application),
                    args,
                },
            });
        }

        Ok(applications)
    }

    fn list_source(&'a self) -> &'a str {
        &self.list_source
    }

    fn add(&mut self, application: Application<'a>) -> Result<(), MyOwnError> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(applications_file())
            .with_error_description(|| format!("Error while loading {}", applications_file()))?;

        writeln!(
            file,
            "{}|{}|{}",
            application.source,
            application.instructions.application,
            application.instructions.args.join("&")
        )?;
        file.flush()?;

        self.list.push(application);

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
            .list
            .iter()
            .position(|app| app.instructions.application == application.instructions.application)
            .ok_or_else(|| MyOwnError::ActualError("Couldn't find application to remove".into()))?;
        self.list.remove(index);

        for app in &self.list {
            writeln!(file, "{}|{}", app.source, app.instructions.application)?;
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
        self.list
            .iter()
            .find(|app| app.instructions.application == application)
    }

    fn get_application(&self, source: &Source, application: &str) -> Option<&Application<'a>> {
        self.list
            .iter()
            .find(|app| &app.source == source && app.instructions.application == application)
    }

    fn is_already_installed(
        &'a self,
        source: &'a Source,
        application: &'a str,
    ) -> AlreadyInstalled<'a> {
        let indices = fuzzy_search(
            application,
            &self
                .list
                .iter()
                .map(|a| a.instructions.application.as_ref())
                .collect::<Vec<_>>(),
            3,
        );

        let similar_matches = indices
            .into_iter()
            .map(|i| self.list.get(i).unwrap())
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

        #[option(name = "--args", delimiters = &['|'])]
        args: Vec<&'a str>,
    }
}

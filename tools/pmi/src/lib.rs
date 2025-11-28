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
pub fn pmi_cli(args: &[&str]) -> Result<(), MyOwnError> {
    let options = InstallOptions::from_args(args)?;

    if options.show_path {
        println!("{}", applications_file());
        return Ok(());
    }

    let applications = if options.applications_file_from_stdin {
        Applications::from_stdin()
    } else {
        Applications::from_file()
    }?;

    let installer = Installer(applications);
    let application = match (options.application, options.source) {
        (Some(application), Some(source)) => {
            Some(Application::new(source, application, options.args))
        }
        (Some(application), None) => {
            let mut possible_matches = if options.uninstall {
                installer.0.get_applications_by_name(application)
            } else {
                installer.identify_application(application)?
            };
            match possible_matches.len() {
                0 => {
                    println!("Found no matches, aborting");
                    return Ok(());
                }
                1 => Some(possible_matches.remove(0)),
                _ => {
                    let index = confirm_application(&possible_matches)?;
                    Some(possible_matches.remove(index))
                }
            }
        }
        _ => None,
    };

    match (application, options.all, options.uninstall) {
        (Some(application), false, false) => installer.install(application),
        (Some(application), false, true) => installer.uninstall(application),
        (None, true, false) => {
            ask_permission(&format!(
                "# This operation will not modify the list of installed applications. Do you want to install {} applications? Y/n",
                installer.0.list.len()
            ))?;
            installer.install_all()
        }
        (None, true, true) => {
            ask_permission(&format!(
                "# This will also remove all the applications from the list of installed applications. Do you want to uninstall {} applications? Y/n",
                installer.0.list.len()
            ))?;
            installer.uninstall_all()
        }
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
                "Example install with args: myown install dx-cli cargo --args \"--no-default-features&--features&web,server\""
            );
            println!("Example uninstall: myown install -u tailwindcss npm");
            println!();
            if installer.0.list.len() > 0 {
                println!("# Apps installed [{}]:", applications_file());
                for app in installer.0.list {
                    println!(
                        "{} | {} | {}",
                        app.source,
                        app.instructions.application,
                        app.instructions.args.join("&")
                    );
                }
            } else {
                println!("# No apps installed [{}]", applications_file());
            }
            return Ok(());
        }
    }
}

fn confirm_application<'a>(applications: &[Application<'a>]) -> Result<usize, MyOwnError> {
    println!("# Found {} alternatives", applications.len());
    for (i, app) in applications.iter().enumerate() {
        println!("{}. {}", i + 1, app.source);
    }
    println!();
    let answer = ask_input(&format!("# Which one?",))?
        .trim()
        .parse::<usize>()
        .error_description("Answer should be a number")?;

    if answer == 0 || answer > applications.len() {
        Err(MyOwnError::ActualError("Answer outside range".into()))
    } else {
        Ok(answer)
    }
}

struct Installer<'a>(Applications<'a>);

impl<'a> Installer<'a> {
    fn install_all(self) -> Result<(), MyOwnError> {
        println!("## Ready to install all applications");

        for application in self.0.list {
            println!("## Installing {}", application.instructions.application);
            application.source.install(&application.instructions)?;
        }

        println!("## All applications installed");

        Ok(())
    }

    fn uninstall_all(mut self) -> Result<(), MyOwnError> {
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
        let already_installed = self.0.is_already_installed(&application);

        if let Some(_) = already_installed.perfect_match {
            return Err(MyOwnError::ActualError(
                format!("Application already installed").into(),
            ));
        }

        if already_installed.similar_matches.len() > 0 {
            println!("# Found installed applications with a similar name");
            for similar in already_installed.similar_matches {
                println!(
                    "## {} | {} | {}",
                    similar.source,
                    similar.instructions.application,
                    similar.instructions.args.join("&")
                );
            }

            ask_permission(&format!(
                "# Do you want to still install {} | {} | {}? Y/n",
                application.source,
                application.instructions.application,
                application.instructions.args.join("&")
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
        if let None = self.0.get_matching_application(&application) {
            return Err(MyOwnError::ActualError(
                "Application is not installed or not installed through same source".into(),
            ));
        };

        println!("## Ready to uninstall application");

        application.source.uninstall(&application.instructions)?;

        println!("## Application uninstalled");

        self.0.remove(application)?;

        println!("## Application removed from list");

        Ok(())
    }

    fn identify_application(
        &self,
        application: &'a str,
    ) -> Result<Vec<Application<'a>>, MyOwnError> {
        Ok(search_source_with_application(application)?
            .into_iter()
            .map(|s| Application::<'a>::new(s, application, vec![]))
            .collect::<Vec<_>>())
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
}

#[derive(Clone)]
struct Application<'a> {
    source: Source,
    instructions: ApplicationInstructions<'a>,
}

#[derive(Clone)]
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

    fn identify_same_application(&self, application: &Application<'a>) -> bool {
        self.source == application.source
            && self.instructions.application == application.instructions.application
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
        let list_source = applications_file();

        println!("# Applications list from [{}]", list_source);
        Ok(Applications { list })
    }

    fn from_stdin() -> Result<Self, MyOwnError> {
        let list = Applications::<'a>::list_from_reader(stdin())?;
        let list_source = "stdin".to_string();

        println!("# Applications list from [{}]", list_source);
        Ok(Applications { list })
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
            writeln!(
                file,
                "{}|{}|{}",
                app.source,
                app.instructions.application,
                app.instructions.args.join("&")
            )?;
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

    fn get_applications_by_name(&self, application: &'a str) -> Vec<Application<'a>> {
        self.list
            .iter()
            .filter(|app| app.instructions.application == application)
            .cloned()
            .collect::<Vec<_>>()
    }

    fn get_matching_application(&self, application: &Application<'a>) -> Option<&Application<'a>> {
        self.list
            .iter()
            .find(|app| app.identify_same_application(application))
    }

    // TODO: rename this and the above
    fn get_application(&self, application: &Application<'a>) -> Option<&Application<'a>> {
        self.list.iter().find(|app| {
            app.source == application.source
                && app.instructions.application == application.instructions.application
                && app.instructions.args == application.instructions.args
        })
    }

    fn is_already_installed(&'a self, application: &Application<'a>) -> AlreadyInstalled<'a> {
        let indices = fuzzy_search(
            &application.instructions.application,
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

        let perfect_match = self.get_application(application);

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

        #[option(name = "--args", delimiters = &['&'])]
        args: Vec<&'a str>,
    }
}

mod applications;
mod installer;
mod sources;

use std::{
    borrow::Cow,
    fs::File,
    io::{BufRead, BufReader, Write, stdout},
};

use build_your_own_macros::cli_options;
use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::{
    applications::{ApplicationName, Persistence, RequestApplication, applications_file},
    installer::Installer,
    sources::{Source, SourceInstructions},
};

// My idea!
pub fn pmi_cli(args: &[&str]) -> MyOwnResult<()> {
    let options = InstallOptions::from_args(args)?;

    if options.modes.show_path {
        println!("{}", applications_file());
        return Ok(());
    }

    let applications = if options.flags.applications_file_from_stdin {
        Persistence::from_stdin(!options.flags.no_save)
    } else {
        Persistence::from_file(!options.flags.no_save)
    }?;

    let installer = Installer(applications);
    let application = match (options.application, &options.source) {
        (Some(application), Some(source)) => Some(RequestApplication {
            name: ApplicationName(Cow::Borrowed(application)),
            source_instruction: Some(SourceInstructions {
                source: source.clone(),
                args: options.args.iter().cloned().map(Cow::Borrowed).collect(),
            }),
        }),
        (Some(application), None) => Some(RequestApplication {
            name: ApplicationName(Cow::Borrowed(application)),
            source_instruction: None,
        }),
        _ => None,
    };

    match (application, options.modes.all, options.flags.uninstall) {
        (Some(application), false, false) => installer.install(application),
        (Some(application), false, true) => installer.uninstall(application),
        (None, true, false) => {
            ask_permission(&format!(
                "# This operation will not modify the list of installed applications. Do you want to install {} applications? (Y/n)",
                installer.0.list.len()
            ))?;
            installer.install_all()
        }
        (None, true, true) => {
            ask_permission(&format!(
                "# This will also remove all the applications from the list of installed applications. Do you want to uninstall {} applications? (Y/n)",
                installer.0.list.len()
            ))?;
            installer.uninstall_all()
        }
        _ => {
            options.print_help(&mut stdout())?;
            println!();
            if installer.0.list.len() > 0 {
                println!("# Apps installed [{}]:", applications_file());
                for app in installer.0.list {
                    println!("{}", app);
                }
            } else {
                println!("# No apps installed [{}]", applications_file());
            }
            return Ok(());
        }
    }
}

fn ask_permission(question: &str) -> MyOwnResult<()> {
    ask_input(question).and_then(|answer| {
        let answer = answer.trim();
        Ok(if !answer.is_empty() && answer.to_lowercase() != "y" {
            return Err(MyOwnError::EarlyExit);
        })
    })
}

fn ask_input(question: &str) -> MyOwnResult<String> {
    let mut tty = BufReader::new(File::open("/dev/tty")?);
    let mut tty_out = File::create("/dev/tty")?;

    writeln!(tty_out, "{}", question)?;
    tty_out.flush()?;

    let mut answer = String::new();
    tty.read_line(&mut answer)?;
    Ok(answer)
}

cli_options! {
    #[options(
        usage = "pmi [-p] [-a] [-u] [-i] [application] [source] [--args]",
        examples = &[
            "Example install: pmi tailwindcss npm",
            "Example install with args: pmi dx-cli cargo --args \"--no-default-features&--features&web,server\"",
            "Example uninstall: pmi -u tailwindcss npm"
        ]
    )]
    struct InstallOptions<'a> {
        #[option(descr = "Name of app/package to install, mandatory without -a")]
        application: Option<&'a str>,

        #[option(descr = "What to use to install the app/package, optional\nIf not provided it will find it and ask for confirmation")]
        source: Option<Source>,

        #[option(name = "--args", delimiters = &['&'], descr = "Arguments to pass to the installer, '&' delimited")]
        args: Vec<&'a str>,

        #[suboptions(name = "flags")]
        struct Flags {
            #[option(name = "-u", alt_names = &["--uninstall"], descr = "Uninstall")]
            uninstall: bool,

            #[option(name = "-i", alt_names = &["--stdin"], descr = "Read list of applications from stdin\ninstead of the default location")]
            applications_file_from_stdin: bool,

            #[option(name = "-n", alt_names = &["--no-save"], descr = "Do not persist installed/uninstalled application in the list of applications")]
            no_save: bool,
        },

        #[suboptions(name = "modes")]
        struct AlternateModes {
            #[option(name = "-p", alt_names = &["--print-path"], descr = "Print path of file with list of applications")]
            show_path: bool,

            #[option(name = "-a", alt_names = &["--all"], descr = "Install/Uninstall all applications present in the list")]
            all: bool,
        },
    }
}

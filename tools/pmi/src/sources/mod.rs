pub(crate) mod apt;
pub(crate) mod brew;
pub(crate) mod cargo;
pub(crate) mod executable_file;
pub(crate) mod install_sh;
pub(crate) mod npm;
pub(crate) mod snap;

use std::{fmt::Display, str::FromStr};

use crate::ApplicationInstructions;
use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::sources::{
    apt::Apt, brew::Brew, cargo::Cargo, executable_file::ExecutableFile, install_sh::InstallSh,
    npm::Npm, snap::Snap,
};

pub(crate) trait SourceManager {
    fn install(&self, application: &ApplicationInstructions) -> MyOwnResult<()>;
    fn uninstall(&self, application: &ApplicationInstructions) -> MyOwnResult<()>;
    fn has_application(&self, application: &str) -> MyOwnResult<bool>;
}

macro_rules! build_sources {
    (
        $name:ident {
            $( $(#[$meta:meta])* $variant:ident ($as_str:expr) ),* $(,)?
        }
    ) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub(crate) enum $name {
            $(
                $(#[$meta])*
                $variant
            ),*
        }

        impl Source {
            pub(crate) fn install(&self, application: &ApplicationInstructions) -> MyOwnResult<()> {
                match self {
                    $(
                        $(#[$meta])*
                        $name::$variant => $variant.install(application)
                    ),*
                }
            }

            pub(crate) fn uninstall(&self, application: &ApplicationInstructions) -> MyOwnResult<()> {
                match self {
                    $(
                        $(#[$meta])*
                        $name::$variant => $variant.uninstall(application)
                    ),*
                }
            }
        }

        impl FromStr for Source {
            type Err = MyOwnError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $(#[$meta])*
                        $as_str => Ok($name::$variant),
                    )*
                    _ => Err(format!("{} is not a source", s).into()),
                }
            }
        }

        impl Display for Source {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $(#[$meta])*
                        $name::$variant => write!(f, $as_str)
                    ),*
                }
            }
        }

        pub(crate) fn search_source_with_application(application: &str) -> MyOwnResult<Vec<Source>> {
            let mut sources = vec![];
            $(
                $(#[$meta])*
                if $variant.has_application(application)? {
                    sources.push($name::$variant);
                }
            )*

            Ok(sources)
        }
    };
}

build_sources!(Source {
    #[cfg(feature = "apt")]
    Apt ("apt"),

    #[cfg(feature = "brew")]
    Brew ("brew"),

    #[cfg(feature = "cargo")]
    Cargo ("cargo"),

    #[cfg(feature = "npm")]
    Npm ("npm"),

    #[cfg(feature = "snap")]
    Snap ("snap"),

    #[cfg(feature = "executable_file")]
    ExecutableFile ("executable_file"),

    #[cfg(feature = "install_sh")]
    InstallSh ("install_sh"),
});

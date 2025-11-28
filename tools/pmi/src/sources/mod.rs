pub(crate) mod apt;
pub(crate) mod brew;
pub(crate) mod cargo;
pub(crate) mod npm;

use std::{fmt::Display, str::FromStr};

use crate::ApplicationInstructions;
use build_your_own_utils::my_own_error::MyOwnError;

use crate::sources::{apt::Apt, brew::Brew, cargo::Cargo, npm::Npm};

pub(crate) trait SourceManager {
    fn install(&self, application: &ApplicationInstructions) -> Result<(), MyOwnError>;
    fn uninstall(&self, application: &ApplicationInstructions) -> Result<(), MyOwnError>;
    fn has_application(&self, application: &str) -> Result<bool, MyOwnError>;
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
            pub(crate) fn install(&self, application: &ApplicationInstructions) -> Result<(), MyOwnError> {
                match self {
                    $(
                        $(#[$meta])*
                        $name::$variant => $variant.install(application)
                    ),*
                }
            }

            pub(crate) fn uninstall(&self, application: &ApplicationInstructions) -> Result<(), MyOwnError> {
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

        pub(crate) fn search_source_with_application(application: &str) -> Result<Vec<Source>, MyOwnError> {
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
});

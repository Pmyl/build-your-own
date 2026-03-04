use std::{fmt::Display, str::FromStr};

use crate::ApplicationInstructions;
use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

pub(crate) trait SourceManager {
    fn install(&self, application: &ApplicationInstructions) -> MyOwnResult<()>;
    fn uninstall(&self, application: &ApplicationInstructions) -> MyOwnResult<()>;
    fn has_application(&self, application: &str) -> MyOwnResult<bool>;
}

mcr::build_sources!(Source {
    #[cfg(feature = "apt")]                 apt::Apt ("apt"),
    #[cfg(feature = "brew")]                brew::Brew ("brew"),
    #[cfg(feature = "cargo")]               cargo::Cargo ("cargo"),
    #[cfg(feature = "npm")]                 npm::Npm ("npm"),
    #[cfg(feature = "snap")]                snap::Snap ("snap"),
    #[cfg(feature = "executable_file")]     executable_file::ExecutableFile ("executable_file"),
    #[cfg(feature = "install_sh")]          install_sh::InstallSh ("install_sh"),
});

mod mcr {
    macro_rules! build_sources {
        (
            $name:ident {
                $( $(#[$meta:meta])* $module:ident::$variant:ident ($as_str:expr) ),* $(,)?
            }
        ) => {
            $(
                $(#[$meta])*
                pub(crate) mod $module;
            )*

            $(
                $(#[$meta])*
                use crate::sources::$module::$variant;
            )*

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

    pub(crate) use build_sources;
}

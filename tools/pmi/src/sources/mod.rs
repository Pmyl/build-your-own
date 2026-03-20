use std::{borrow::Cow, fmt::Display, str::FromStr};

use build_your_own_utils::my_own_error::{MyOwnError, MyOwnResult};

use crate::applications::ApplicationName;

pub(crate) trait SourceManager {
    fn requires_root_permissions() -> bool {
        return false;
    }
    fn install<'a, Args: IntoIterator<Item = &'a str>>(
        &self,
        application: &'a ApplicationName<'a>,
        args: Args,
    ) -> MyOwnResult<()>;
    fn uninstall<'a>(&self, application: &ApplicationName<'a>) -> MyOwnResult<()>;
    fn has_application(&self, application: &str) -> MyOwnResult<bool>;
}

mcr::build_sources!(Source {
    #[cfg(feature = "apt")]             apt::Apt ("apt"),
    #[cfg(feature = "brew")]            brew::Brew ("brew"),
    #[cfg(feature = "cargo")]           cargo::Cargo ("cargo"),
    #[cfg(feature = "npm")]             npm::Npm ("npm"),
    #[cfg(feature = "snap")]            snap::Snap ("snap"),
    #[cfg(feature = "executable_file")] executable_file::ExecutableFile ("executable_file"),
    #[cfg(feature = "install_sh")]      install_sh::InstallSh ("install_sh"),
    #[cfg(feature = "dnf")]             dnf::Dnf ("dnf"),
});

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct SourceInstructions<'a> {
    pub source: Source,
    pub args: Vec<Cow<'a, str>>,
}

pub(crate) enum InstallError {
    PermissionsMismatch,
    Error(MyOwnError),
}

impl From<MyOwnError> for InstallError {
    fn from(value: MyOwnError) -> Self {
        InstallError::Error(value)
    }
}

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
                ),*,
                Unknown(String)
            }

            impl Source {
                pub(crate) fn install<'a, Args: IntoIterator<Item = &'a str>>(&self, application: &'a ApplicationName<'a>, args: Args, has_root_permissions: bool) -> Result<(), InstallError> {
                    match self {
                        $(
                            $(#[$meta])*
                            $name::$variant => if has_root_permissions == $variant::requires_root_permissions() {
                                Ok($variant.install(application, args)?)
                            } else {
                                Err(InstallError::PermissionsMismatch)
                            }
                        ),*,
                        $name::Unknown(_) => Err(InstallError::Error(MyOwnError::ActualError("Attempted to install with Unknown source, bad developer".into())))
                    }
                }

                pub(crate) fn uninstall<'a>(&self, application: &ApplicationName<'a>, has_root_permissions: bool) -> Result<(), InstallError> {
                    match self {
                        $(
                            $(#[$meta])*
                            $name::$variant => if has_root_permissions == $variant::requires_root_permissions() {
                                Ok($variant.uninstall(application)?)
                            } else {
                                Err(InstallError::PermissionsMismatch)
                            }
                        ),*,
                        $name::Unknown(_) => Err(InstallError::Error(MyOwnError::ActualError("Attempted to uninstall with Unknown source, bad developer".into())))
                    }
                }

                pub fn from_persisted(s: String) -> Self {
                    match s.as_str() {
                        $(
                            $(#[$meta])*
                            $as_str => $name::$variant,
                        )*
                        _ => $name::Unknown(s),
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
                        ),*,
                        $name::Unknown(name) => write!(f, "{}", name)
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

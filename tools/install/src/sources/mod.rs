pub(crate) mod apt;
pub(crate) mod brew;
pub(crate) mod cargo;
pub(crate) mod npm;

use build_your_own_utils::my_own_error::MyOwnError;

// TODO: Rename this to Source and the existing Source into something else, or remove
pub(crate) trait SourceT {
    fn install(&self, application: &str) -> Result<(), MyOwnError>;
    fn uninstall(&self, application: &str) -> Result<(), MyOwnError>;
}

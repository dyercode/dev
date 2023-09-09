use thiserror::Error;

use crate::command::SubCommand;

#[derive(Error, Debug, PartialEq)]
pub enum DevError {
    #[error("command to run {0} was not defined")]
    CommandUndefined(SubCommand),
    #[error("dev.yml was not found")] // todo - include cwd?
    FileNotFound,
    #[error("dev.yml could not be parsed at: {0}")]
    YmlProblem(String),
    #[error("dev.yml could not be read")]
    FileUnreadable,
    #[error("process failed")]
    ProcessFailed,
    #[error("Current directory inaccessible")]
    DirectoryManipulationFailed,
    #[error("SubProject faled: `{0}`")]
    SubProjectFailed(String),
    #[error("SubProject not found: `{0}`")]
    SubProjectNotFound(String),
}

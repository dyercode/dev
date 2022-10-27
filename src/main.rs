use crate::DevError::CommandUndefined;
use clap::builder::Str;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::process::Command;
use thiserror::Error;

const BUILD: &str = "build";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CommandsWrapper {
    commands: Commands,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Commands {
    build: Option<String>,
}

#[derive(Debug)]
pub enum SubCommand {
    Build,
}

impl TryFrom<Str> for SubCommand {
    type Error = ();

    fn try_from(value: Str) -> Result<Self, Self::Error> {
        match value {
            s if s == BUILD => Ok(SubCommand::Build),
            _ => Err(()),
        }
    }
}

impl From<SubCommand> for Str {
    fn from(sc: SubCommand) -> Self {
        match sc {
            SubCommand::Build => BUILD.into(),
        }
    }
}

impl TryFrom<String> for SubCommand {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value {
            s if s == BUILD => Ok(SubCommand::Build),
            _ => Err(()),
        }
    }
}

impl Display for SubCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubCommand::Build => write!(f, "{}", BUILD),
        }
    }
}

#[derive(Error, Debug)]
pub enum DevError {
    #[error("command to run {0} was not defined")]
    CommandUndefined(SubCommand),
    #[error("dev.yml was not found")] // todo - include pwd?
    FileNotFound,
    #[error("dev.yml could not be parsed")]
    YmlProblem,
    #[error("dev.yml could not be read")]
    FileUnreadable,
}

fn read_commands() -> Result<Commands, DevError> {
    let file_path = "./dev.yml";
    if std::path::Path::new(file_path).exists() {
        let raw = fs::read_to_string(file_path).map_err(|_| DevError::FileUnreadable)?;
        let cw: CommandsWrapper = serde_yaml::from_str(&raw).map_err(|_| DevError::YmlProblem)?;
        Ok(cw.commands)
    } else {
        Err(DevError::FileNotFound)
    }
}

fn run_command(command: &str) {
    let mut child = Command::new("sh").arg("-c").arg(&command).spawn().unwrap();
    child.wait().unwrap();
}

fn read_command(cmd: SubCommand) -> Result<String, DevError> {
    let commands = read_commands()?;
    match cmd {
        SubCommand::Build => commands.build.ok_or(CommandUndefined(SubCommand::Build)),
    }
}

fn process_command(command: SubCommand) {
    match read_command(command) {
        Ok(cmd) => run_command(&cmd),
        Err(e) => eprintln!("{:?}", e),
    }
}

fn main() {
    let my_command = clap::Command::new("dev")
        .subcommand_required(true)
        .subcommand(clap::Command::new(SubCommand::Build));

    match my_command.get_matches().subcommand() {
        Some((BUILD, _)) => process_command(SubCommand::Build),
        _ => eprint!("Invalid command"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_build() {
        let result = "real command";
        let res: CommandsWrapper =
            serde_yaml::from_str(&format!("commands:\n  build: {}\n", result)).unwrap();
        assert_eq!(
            res.commands,
            Commands {
                build: Some(result.to_owned())
            }
        );
    }
}

use crate::DevError::CommandUndefined;
use clap::builder::Str;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::process::Command;
use thiserror::Error;

const BUILD: &str = "build";
const RELEASE: &str = "release";
const TEST: &str = "test";
const LINT: &str = "lint";
const CLEAN: &str = "clean";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CommandsWrapper {
    commands: Commands,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Commands {
    build: Option<String>,
    release: Option<String>,
    test: Option<String>,
    lint: Option<String>,
    clean: Option<String>,
}

use proptest_derive::Arbitrary;

#[derive(Debug, PartialEq, Eq, Arbitrary)]
pub enum SubCommand {
    Build,
    Release,
    Test,
    Lint,
    Clean,
}

// todo - consider just allowing &str's with clap feature flag
impl From<SubCommand> for Str {
    fn from(sc: SubCommand) -> Self {
        match sc {
            SubCommand::Build => BUILD,
            SubCommand::Release => RELEASE,
            SubCommand::Test => TEST,
            SubCommand::Lint => LINT,
            SubCommand::Clean => CLEAN,
        }
            .into()
    }
}

// todo - this is super fragile to change, no exhaustiveness check
impl TryFrom<String> for SubCommand {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            BUILD => Ok(SubCommand::Build),
            RELEASE => Ok(SubCommand::Release),
            TEST => Ok(SubCommand::Test),
            LINT => Ok(SubCommand::Lint),
            CLEAN => Ok(SubCommand::Clean),
            _ => Err(()),
        }
    }
}

impl Display for SubCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubCommand::Build => write!(f, "{}", BUILD),
            SubCommand::Release => write!(f, "{}", RELEASE),
            SubCommand::Test => write!(f, "{}", TEST),
            SubCommand::Lint => write!(f, "{}", LINT),
            SubCommand::Clean => write!(f, "{}", CLEAN),
        }
    }
}

#[derive(Error, Debug)]
pub enum DevError {
    #[error("command to run {0} was not defined")]
    CommandUndefined(SubCommand),
    #[error("dev.yml was not found")] // todo - include cwd?
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
        SubCommand::Build => commands.build,
        SubCommand::Release => commands.release,
        SubCommand::Test => commands.test,
        SubCommand::Lint => commands.lint,
        SubCommand::Clean => commands.clean,
    }
        .ok_or(CommandUndefined(cmd))
}

fn process_command(command: SubCommand) {
    match read_command(command) {
        Ok(cmd) => run_command(&cmd),
        Err(e) => eprintln!("{}", e),
    }
}

fn main() {
    let my_command = clap::Command::new("dev")
        .subcommand_required(true)
        .subcommand(clap::Command::new(SubCommand::Build))
        .subcommand(clap::Command::new(SubCommand::Release))
        .subcommand(clap::Command::new(SubCommand::Test))
        .subcommand(clap::Command::new(SubCommand::Lint));

    if let Some((value, _)) = my_command.get_matches().subcommand() {
        match SubCommand::try_from(value.to_owned()) {
            Ok(cmd) => process_command(cmd),
            Err(_) => eprint!("Invalid command '{}'", value), // todo - should be unreachable because clap won't find it
        }
    };
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use super::*;

    #[test]
    fn read_build() {
        let result = "real command";
        let res: CommandsWrapper =
            serde_yaml::from_str(&format!("commands:\n  build: {}\n", result)).unwrap();
        assert_eq!(res.commands.build, Some(result.to_owned()));
    }

    #[test]
    fn read_release() {
        let result = "real command";
        let res: CommandsWrapper =
            serde_yaml::from_str(&format!("commands:\n  release: {}\n", result)).unwrap();
        assert_eq!(res.commands.release, Some(result.to_owned()));
    }

    #[test]
    fn read_test() {
        let result = "real command";
        let res: CommandsWrapper =
            serde_yaml::from_str(&format!("commands:\n  test: {}\n", result)).unwrap();
        assert_eq!(res.commands.test, Some(result.to_owned()));
    }

    proptest! {
        #[test]
        fn sub_command_to_string_and_back(subject in any::<SubCommand>()) {
            assert_eq!(
                subject.to_string().try_into(),
                Ok(subject)
            )
        }
    }
}

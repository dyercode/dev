use core::fmt::{Display, Formatter};
use core::str::FromStr;
use std::fs;
use std::process::{Command, ExitCode, ExitStatus};

use clap::builder::Str;
#[cfg(test)]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Root {
    commands: Commands,
    sub_projects: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Commands {
    build: Option<String>,
    release: Option<String>,
    test: Option<String>,
    lint: Option<String>,
    clean: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Arbitrary))]
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
        sc.into()
    }
}

const BUILD: &str = "build";
const RELEASE: &str = "release";
const TEST: &str = "test";
const LINT: &str = "lint";
const CLEAN: &str = "clean";

impl FromStr for SubCommand {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
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
        let cw: Root = serde_yaml::from_str(&raw).map_err(|_| DevError::YmlProblem)?;
        Ok(cw.commands)
    } else {
        Err(DevError::FileNotFound)
    }
}

fn run_command(command: &str) -> std::io::Result<ExitStatus> {
    Command::new("sh").arg("-c").arg(command).status()
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
    .ok_or(DevError::CommandUndefined(cmd))
}

fn process_command(command: SubCommand) -> ExitCode {
    match read_command(command) {
        Ok(cmd) => match run_command(&cmd) {
            Ok(es) => {
                if es.success() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(_) => ExitCode::FAILURE,
        },
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let my_command = clap::Command::new("dev")
        .subcommand_required(true)
        .subcommand(clap::Command::new(SubCommand::Build))
        .subcommand(clap::Command::new(SubCommand::Release))
        .subcommand(clap::Command::new(SubCommand::Test))
        .subcommand(clap::Command::new(SubCommand::Lint));

    if let Some((value, _)) = my_command.get_matches().subcommand() {
        match SubCommand::from_str(value) {
            Ok(cmd) => process_command(cmd),
            Err(_) => {
                // todo - should be unreachable because clap won't find it
                eprint!("Invalid command '{}'", value);
                ExitCode::FAILURE
            }
        }
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn read_build() {
        let result = "real command";
        let res: Root =
            serde_yaml::from_str(&format!("commands:\n  build: {}\n", result)).unwrap();
        assert_eq!(res.commands.build, Some(result.to_owned()));
    }

    #[test]
    fn read_release() {
        let result = "real command";
        let res: Root =
            serde_yaml::from_str(&format!("commands:\n  release: {}\n", result)).unwrap();
        assert_eq!(res.commands.release, Some(result.to_owned()));
    }

    #[test]
    fn read_test() {
        let result = "real command";
        let res: Root =
            serde_yaml::from_str(&format!("commands:\n  test: {}\n", result)).unwrap();
        assert_eq!(res.commands.test, Some(result.to_owned()));
    }

    proptest! {
        #[test]
        fn sub_command_to_string_and_back(subject in any::<SubCommand>()) {
            assert_eq!(
                SubCommand::from_str(&subject.to_string()),
                Ok(subject)
            )
        }
    }
}

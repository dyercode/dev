use crate::err::DevError;
use crate::filesystem::cwd;
use clap::{Subcommand, ValueEnum};
use core::fmt::{Display, Formatter};
#[cfg(test)]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Root {
    pub commands: Option<Commands>,
    pub subprojects: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Commands {
    #[serde(default)]
    build: UserCommand,
    #[serde(default)]
    package: UserCommand,
    #[serde(default)]
    check: UserCommand,
    #[serde(default)]
    clean: UserCommand,
    #[serde(default)]
    install: UserCommand,
    #[serde(default)]
    run: UserCommand,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum UserCommand {
    None,
    Command(String),
    Commands(Vec<String>),
}

impl Default for UserCommand {
    fn default() -> Self {
        Self::None
    }
}

impl Commands {
    pub fn by_sub_command(self, sub_command: &SubCommand) -> UserCommand {
        match *sub_command {
            SubCommand::Build => self.build,
            SubCommand::Package => self.package,
            SubCommand::Check => self.check,
            SubCommand::Clean => self.clean,
            SubCommand::Install => self.install,
            SubCommand::Run => self.run,
        }
    }
}

#[derive(Subcommand, ValueEnum, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum SubCommand {
    Build,
    Package,
    Check,
    Clean,
    Install,
    Run,
}

const BUILD: &str = "build";
const PACKAGE: &str = "package";
const CHECK: &str = "check";
const CLEAN: &str = "clean";
const INSTALL: &str = "install";
const RUN: &str = "run";

impl FromStr for SubCommand {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            BUILD => Ok(SubCommand::Build),
            PACKAGE => Ok(SubCommand::Package),
            CHECK => Ok(SubCommand::Check),
            CLEAN => Ok(SubCommand::Clean),
            INSTALL => Ok(SubCommand::Install),
            RUN => Ok(SubCommand::Run),
            _ => Err(()),
        }
    }
}

impl Display for SubCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            SubCommand::Build => write!(f, "{BUILD}"),
            SubCommand::Package => write!(f, "{PACKAGE}"),
            SubCommand::Check => write!(f, "{CHECK}"),
            SubCommand::Clean => write!(f, "{CLEAN}"),
            SubCommand::Install => write!(f, "{INSTALL}"),
            SubCommand::Run => write!(f, "{RUN}"),
        }
    }
}

pub fn read_yaml(file_path: &Path) -> Result<Root, DevError> {
    if file_path.exists() {
        let raw = fs::read_to_string(file_path).map_err(|_| DevError::FileUnreadable)?;
        let parsed: Result<Root, serde_yaml::Error> = serde_yaml::from_str(&raw);
        match parsed {
            Ok(root) => Ok(root),
            Err(_) => {
                let cwd = cwd();
                Err(DevError::YmlProblem(
                    cwd?.to_str().unwrap_or_default().to_owned(),
                ))
            }
        }
    } else {
        Err(DevError::FileNotFound)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn sub_command_to_string_and_back(subject in any::<SubCommand>()) {
            assert_eq!(
                subject.to_string().parse::<SubCommand>(),
                Ok(subject)
            )
        }

        #[test]
        fn read_sub_command_from_yaml(subject in any::<SubCommand>()) {
            let result = "real command";
            let res: Root = serde_yaml::from_str(&format!("commands:\n  {}: {}\n", subject, result)).unwrap();
            assert_eq!(
                res.commands.unwrap().by_sub_command(&subject),
                UserCommand::Command(result.to_owned()),
            );
        }
    }

    #[test]
    fn sub_command_errors_when_unknown_string() {
        assert_eq!("fish".parse::<SubCommand>(), Err(()));
    }

    #[test]
    fn read_yaml_returns_dev_error_when_file_not_found() {
        assert_eq!(
            Err(DevError::FileNotFound),
            read_yaml(Path::new("./scroobledoobledoo"))
        )
    }

    #[test]
    fn read_yaml_returns_err_when_bad_yaml() {
        let result = read_yaml(Path::new("./tests/bad_yaml/dev.yml"));
        match result {
            Ok(_) => panic!("was read successfully?"),
            Err(e) => assert!(matches!(e, DevError::YmlProblem { .. })),
        }
    }
}

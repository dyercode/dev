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
    build: Option<String>,
    package: Option<String>,
    test: Option<String>,
    lint: Option<String>,
    clean: Option<String>,
    install: Option<String>,
}

impl Commands {
    pub fn by_sub_command(self, sub_command: &SubCommand) -> Option<String> {
        match *sub_command {
            SubCommand::Build => self.build,
            SubCommand::Package => self.package,
            SubCommand::Test => self.test,
            SubCommand::Lint => self.lint,
            SubCommand::Clean => self.clean,
            SubCommand::Install => self.install,
        }
    }
}

#[derive(Subcommand, ValueEnum, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum SubCommand {
    Build,
    Package,
    Test,
    Lint,
    Clean,
    Install,
}

const BUILD: &str = "build";
const PACKAGE: &str = "package";
const TEST: &str = "test";
const LINT: &str = "lint";
const CLEAN: &str = "clean";
const INSTALL: &str = "install";

impl FromStr for SubCommand {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            BUILD => Ok(SubCommand::Build),
            PACKAGE => Ok(SubCommand::Package),
            TEST => Ok(SubCommand::Test),
            LINT => Ok(SubCommand::Lint),
            CLEAN => Ok(SubCommand::Clean),
            INSTALL => Ok(SubCommand::Install),
            _ => Err(()),
        }
    }
}

impl Display for SubCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            SubCommand::Build => write!(f, "{}", BUILD),
            SubCommand::Package => write!(f, "{}", PACKAGE),
            SubCommand::Test => write!(f, "{}", TEST),
            SubCommand::Lint => write!(f, "{}", LINT),
            SubCommand::Clean => write!(f, "{}", CLEAN),
            SubCommand::Install => write!(f, "{}", INSTALL),
        }
    }
}

pub fn read_yaml() -> Result<Root, DevError> {
    let file_path = Path::new("./dev.yml");
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
            assert_eq!(res.commands.unwrap().by_sub_command(&subject), Some(result.to_owned()));
        }

    }

    #[test]
    fn sub_command_errors_when_unknown_string() {
        assert_eq!("fish".parse::<SubCommand>(), Err(()));
    }
}

use clap::{Parser, Subcommand, ValueEnum};
use core::fmt::{Display, Formatter};
use core::str::FromStr;
#[cfg(test)]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::env::set_current_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Root {
    commands: Option<Commands>,
    subprojects: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Commands {
    build: Option<String>,
    package: Option<String>,
    test: Option<String>,
    lint: Option<String>,
    clean: Option<String>,
    install: Option<String>,
}

impl Commands {
    fn by_sub_command(self, sub_command: &SubCommand) -> Option<String> {
        match sub_command {
            SubCommand::Build => self.build,
            SubCommand::Package => self.package,
            SubCommand::Test => self.test,
            SubCommand::Lint => self.lint,
            SubCommand::Clean => self.clean,
            SubCommand::Install => self.install,
        }
    }
}

#[derive(Parser)]
struct Cli {
    command: SubCommand,
}

#[derive(Subcommand, ValueEnum, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
enum SubCommand {
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

#[derive(Error, Debug)]
enum DevError {
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

fn read_yaml() -> Result<Root, DevError> {
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

fn run_command(command: &str) -> Result<ExitStatus, DevError> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .map_err(|_| DevError::ProcessFailed)
}

fn run_dev_command(command: &SubCommand) -> Result<ExitStatus, std::io::Error> {
    Command::new("dev").arg(command.to_string()).status()
}

fn read_command(cmd: SubCommand, commands: Commands) -> Result<String, DevError> {
    commands
        .by_sub_command(&cmd)
        .ok_or(DevError::CommandUndefined(cmd))
}

fn run_project_command(sub_command: SubCommand, commands: Commands) -> Result<(), DevError> {
    log::info!("run_project_command {:?}:{:?}", &sub_command, &commands);
    read_command(sub_command, commands).and_then(|cmd| {
        run_command(&cmd).and_then(|es| {
            if es.success() {
                Ok(())
            } else {
                Err(DevError::ProcessFailed)
            }
        })
    })
}

fn cd(p: &Path) -> Result<(), DevError> {
    set_current_dir(p).map_err(|_| DevError::DirectoryManipulationFailed)
}

fn cwd() -> Result<PathBuf, DevError> {
    std::env::current_dir().map_err(|_| DevError::DirectoryManipulationFailed)
}

fn run_subproject_command(
    command: &SubCommand,
    cwd: &Path,
    sub_project: &str,
) -> Result<(), DevError> {
    log::info!("run_subproject_command {:?}:{}", &command, sub_project);
    cd(cwd)?;
    let sub_path = Path::new(sub_project);
    let sub_dev_yml = sub_path.join("dev.yml"); // todo - slashes probably wrong
    if sub_path.exists() && sub_dev_yml.exists() {
        cd(sub_path)?;
        run_dev_command(command)
            .map(|_| ())
            .map_err(|_| DevError::SubProjectFailed(sub_project.to_owned()))
    } else {
        Err(DevError::SubProjectNotFound(sub_project.to_owned()))
    }
}

fn run_subprojects(command: SubCommand, sub_projects: Vec<String>) -> Result<(), DevError> {
    let cwd: PathBuf = cwd()?;
    for sp in sub_projects {
        run_subproject_command(&command, &cwd, &sp)?;
    }
    Ok(())
}

fn process_command(command: SubCommand) -> Result<(), DevError> {
    log::info!("processing command");
    read_yaml().and_then(|root| {
        log::info!("config: {:?}", &root);
        match (root.subprojects, root.commands) {
            (None, None) => Err(DevError::YmlProblem(format!(
                "{:?}, no tasks or subprojects present",
                cwd()?.to_str()
            ))),
            (None, Some(commands)) => run_project_command(command, commands),
            (Some(sp), Some(commands)) if sp.is_empty() => run_project_command(command, commands),
            (Some(sp), _) => run_subprojects(command, sp),
        }
    })
}

fn main() -> ExitCode {
    env_logger::init();
    log::info!("start prog");
    let my_command = Cli::parse();

    log::info!("god command: {:?}", &my_command.command);
    match process_command(my_command.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
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

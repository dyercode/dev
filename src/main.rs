use anyhow::{anyhow, Result};
use clap::builder::Str;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Command;
use std::{fs, io};

const BUILD: &str = "build";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CommandsWrapper {
    commands: Commands,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Commands {
    build: Option<String>,
}

enum SubCommand {
    Build,
}

impl TryFrom<Str> for SubCommand {
    type Error = ();

    fn try_from(value: Str) -> std::result::Result<Self, Self::Error> {
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

fn read_commands() -> Result<Commands> {
    let file_path = "./dev.yml";
    if std::path::Path::new(file_path).exists() {
        let raw = fs::read_to_string(file_path)?;
        let cw: CommandsWrapper = serde_yaml::from_str(&raw)?;
        Ok(cw.commands)
    } else {
        Err(anyhow!("dev.yml not found on path"))
    }
}

fn process_command(command: SubCommand) {
    match command {
        SubCommand::Build => match read_commands() {
            Ok(Commands {
                build: Some(build_cmd),
            }) => {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&build_cmd)
                    .output()
                    .unwrap_or_else(|_| panic!("Build cmd {} failed", build_cmd));
                io::stdout().write_all(&output.stdout).unwrap();
            }
            _ => panic!("build command not found"),
        },
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

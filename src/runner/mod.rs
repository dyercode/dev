use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use crate::command::{read_yaml, Commands, UserCommand};
use crate::filesystem::cwd;
use crate::{command::SubCommand, err::DevError, filesystem::cd};

fn run_command(command: &str) -> Result<ExitStatus, DevError> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .map_err(|_| DevError::ProcessFailed)
}

pub fn run_project_commands(sub_command: &SubCommand, commands: Commands) -> Result<(), DevError> {
    log::info!("run_project_command {:?}:{:?}", &sub_command, &commands);
    match commands.by_sub_command(sub_command) {
        UserCommand::None => Err(DevError::CommandUndefined(sub_command.clone())),
        UserCommand::Command(cmd) => run_command(&cmd).map(|_| ()),
        UserCommand::Commands(cmds) => cmds.iter().try_for_each(|cmd| run_command(cmd).map(|_| ())),
    }
}

fn run_subprojects(command: &SubCommand, sub_projects: Vec<String>) -> Result<(), DevError> {
    let cwd: PathBuf = cwd()?;
    for sp in sub_projects {
        run_subproject_command(command, &cwd, &sp)?;
    }
    Ok(())
}

pub fn process_command(command: &SubCommand) -> Result<(), DevError> {
    log::info!("processing command");
    read_yaml().and_then(|root| {
        log::info!("config: {:?}", &root);
        match (root.subprojects, root.commands) {
            (None, None) => Err(DevError::YmlProblem(format!(
                "{}, no tasks or subprojects present",
                cwd()
                    .ok()
                    .and_then(|path| path.to_str().map(str::to_owned))
                    .unwrap_or("no directory found".to_string())
            ))),
            (None, Some(commands)) => run_project_commands(command, commands),
            (Some(sp), Some(commands)) if sp.is_empty() => run_project_commands(command, commands),
            (Some(sp), _) => run_subprojects(command, sp),
        }
    })
}

pub fn run_subproject_command(
    command: &SubCommand,
    cwd: &Path,
    sub_project: &str,
) -> Result<(), DevError> {
    log::info!("run_subproject_command {:?}:{}", &command, sub_project);
    cd(cwd)?;
    let sub_path = Path::new(sub_project);
    let sub_dev_yml = sub_path.join("dev.yml");
    if sub_path.exists() && sub_dev_yml.exists() {
        cd(sub_path)?;
        process_command(command).map_err(|_err| DevError::SubProjectFailed(sub_project.to_owned()))
    } else {
        Err(DevError::SubProjectNotFound(sub_project.to_owned()))
    }
}

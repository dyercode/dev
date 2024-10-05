use std::path::{Path, PathBuf};
use std::process::Command;

use crate::command::{read_yaml, Commands, UserCommand};
use crate::filesystem::cwd;
use crate::{command::SubCommand, err::DevError, filesystem::cd};

fn run_command(command: &str) -> Result<(), DevError> {
    match Command::new("sh").arg("-c").arg(command).status() {
        Ok(status) => match status.code() {
            Some(0) => Ok(()),
            Some(_) => Err(DevError::ProcessFailed),
            None => unreachable!("this is how you make no status code"),
        },
        Err(err) => unreachable!(
            "the method to make sh actually fail turns out to be: {:?}",
            err
        ),
    }
}

pub fn run_project_commands(sub_command: &SubCommand, commands: Commands) -> Result<(), DevError> {
    log::info!("run_project_command {:?}:{:?}", &sub_command, &commands);
    match commands.by_sub_command(sub_command) {
        UserCommand::None => Err(DevError::CommandUndefined(sub_command.clone())),
        UserCommand::Command(cmd) => run_command(&cmd),
        UserCommand::Commands(cmds) => cmds.iter().try_for_each(|cmd| run_command(cmd)),
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
    // todo - put this somewhere better
    let dev_path = Path::new("./dev.yml");
    read_yaml(dev_path).and_then(|root| {
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

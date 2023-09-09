use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use crate::filesystem::cwd;
use crate::{command::SubCommand, err::DevError, filesystem::cd};

fn run_dev_command(command: &SubCommand) -> Result<ExitStatus, std::io::Error> {
    Command::new("dev").arg(command.to_string()).status()
}

pub fn run_subprojects(command: SubCommand, sub_projects: Vec<String>) -> Result<(), DevError> {
    let cwd: PathBuf = cwd()?;
    for sp in sub_projects {
        run_subproject_command(&command, &cwd, &sp)?;
    }
    Ok(())
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
        match run_dev_command(command) {
            Ok(status) if status.success() => Ok(()),
            Ok(_) => Err(DevError::SubProjectFailed(sub_project.to_owned())),
            Err(_) => Err(DevError::SubProjectFailed(sub_project.to_owned()))
        }
    } else {
        Err(DevError::SubProjectNotFound(sub_project.to_owned()))
    }
}

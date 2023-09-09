use clap::Parser;
use dev::command::{read_yaml, Commands, SubCommand};
use dev::err::DevError;
use dev::filesystem::cwd;
use dev::runner::run_subprojects;
use std::process::{Command, ExitCode, ExitStatus};

#[derive(Parser)]
struct Cli {
    command: SubCommand,
}

fn run_command(command: &str) -> Result<ExitStatus, DevError> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .map_err(|_| DevError::ProcessFailed)
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

    log::info!("got command: {:?}", &my_command.command);
    match process_command(my_command.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}

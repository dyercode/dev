use clap::Parser;
use dev::{command::SubCommand, runner::process_command};
use std::process::ExitCode;

#[derive(Parser)]
struct Cli {
    command: SubCommand,
}

fn main() -> ExitCode {
    env_logger::init();
    log::info!("start prog");
    let my_command = Cli::parse();

    log::info!("got command: {:?}", &my_command.command);
    match process_command(&my_command.command) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

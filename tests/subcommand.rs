use dev::{command::SubCommand, err::DevError, filesystem::cwd, runner::run_subproject_command};

#[test]
fn run_subproject_command_nonexistent_directory_returns_subproject_not_found() {
    let res = run_subproject_command(&SubCommand::Build, &cwd().unwrap(), "fake_dir");
    assert_eq!(
        res.unwrap_err(),
        DevError::SubProjectNotFound("fake_dir".to_owned())
    );
}

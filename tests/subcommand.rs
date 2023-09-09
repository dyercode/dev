use dev::{command::SubCommand, err::DevError, filesystem::cwd, runner::run_subproject_command};

#[test]
fn run_subproject_command_nonexistent_directory_returns_subproject_not_found() {
    let res = run_subproject_command(&SubCommand::Build, &cwd().unwrap(), "fake_dir");
    assert_eq!(
        res.unwrap_err(),
        DevError::SubProjectNotFound("fake_dir".to_owned())
    );
}

#[test]
fn run_subproject_command_failing_test_returns_subproject_failed() {
    let res = run_subproject_command(
        &SubCommand::Test,
        &cwd().unwrap(),
        "tests/failing_subproject",
    );
    assert_eq!(
        res.unwrap_err(),
        DevError::SubProjectFailed("tests/failing_subproject".to_owned())
    );
}

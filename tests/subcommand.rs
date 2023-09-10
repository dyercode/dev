use std::{path::PathBuf, sync::Mutex};

use dev::{
    command::SubCommand,
    err::DevError,
    filesystem::{cd, cwd},
    runner::run_subproject_command,
};

static WD_LOCKER: Mutex<&str> = Mutex::new("");

fn dir_locker<F>(closure: F) -> Result<(), DevError>
where
    F: FnOnce(&PathBuf) -> Result<(), DevError>,
{
    let _locker = WD_LOCKER.lock();
    let cwd = cwd().unwrap();
    log::info!("{}", cwd.clone().into_os_string().into_string().unwrap());
    let res = closure(&cwd);
    cd(&cwd).unwrap();
    res
}

#[test]
fn run_subproject_command_nonexistent_directory_returns_subproject_not_found() {
    let res = dir_locker(|cwd| run_subproject_command(&SubCommand::Build, cwd, "fake_dir"));
    assert_eq!(
        res.unwrap_err(),
        DevError::SubProjectNotFound("fake_dir".to_owned())
    );
}

#[test]
fn run_subproject_command_failing_test_returns_subproject_failed() {
    let dev_location = "tests/failing_subproject";
    let res = dir_locker(|cwd| run_subproject_command(&SubCommand::Test, cwd, dev_location));
    assert_eq!(
        res.unwrap_err(),
        DevError::SubProjectFailed(dev_location.to_owned())
    );
}

#[test]
fn run_subproject_command_success_is_ok() {
    let res =
        dir_locker(|cwd| run_subproject_command(&SubCommand::Test, cwd, "tests/pass_subproject"));
    assert_eq!(res, Ok(()),);
}

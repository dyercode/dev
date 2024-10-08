use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use dev::{
    command::SubCommand,
    err::DevError,
    filesystem::{cd, cwd},
    runner::{process_command, run_subproject_command},
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
    let res = dir_locker(|cwd| run_subproject_command(&SubCommand::Check, cwd, dev_location));
    assert_eq!(
        res.unwrap_err(),
        DevError::SubProjectFailed(dev_location.to_owned())
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn run_subproject_command_success_is_ok() {
    let res =
        dir_locker(|cwd| run_subproject_command(&SubCommand::Check, cwd, "tests/pass_subproject"));
    assert_eq!(res, Ok(()),);
}

#[test]
#[cfg_attr(miri, ignore)]
fn run_multitask_command_successes_is_ok() {
    let res = dir_locker(|cwd| {
        run_subproject_command(&SubCommand::Check, cwd, "tests/multitask_subproject")
    });
    assert_eq!(res, Ok(()),);
}

#[test]
#[cfg_attr(miri, ignore)]
fn process_command_can_run_subproject_commmands_success_is_ok() {
    let res = dir_locker(|_| {
        cd(Path::new("tests/with_subprojects")).unwrap();
        process_command(&SubCommand::Check)
    });
    assert_eq!(res, Ok(()));
}

#[test]
#[cfg_attr(miri, ignore)]
fn process_command_end_on_first_failure() {
    let res = dir_locker(|_| {
        cd(Path::new("tests/multitask_early_failure")).unwrap();
        process_command(&SubCommand::Check)
    });
    assert_eq!(res, Err(DevError::ProcessFailed));
}

#[test]
#[cfg_attr(miri, ignore)]
fn process_command_rus_root_project_command_when_subprojects_empty() {
    let res = dir_locker(|_| {
        cd(Path::new("tests/empty_subprojects")).unwrap();
        process_command(&SubCommand::Check)
    });
    assert_eq!(res, Err(DevError::ProcessFailed));
}

#[test]
fn process_command_yml_error_when_no_commands_or_subprojects() {
    let mut dir: String = "".to_string();
    let res = dir_locker(|_| {
        cd(Path::new("tests/empty_dev")).unwrap();
        cwd().unwrap().to_str().unwrap().clone_into(&mut dir);
        process_command(&SubCommand::Check)
    });
    assert_eq!(
        res,
        Err(DevError::YmlProblem(format!(
            "{}, no tasks or subprojects present",
            dir
        )))
    );
}

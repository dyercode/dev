use std::env::set_current_dir;
use std::path::{Path, PathBuf};

use crate::err::DevError;

pub fn cd(p: &Path) -> Result<(), DevError> {
    set_current_dir(p).map_err(|_| DevError::DirectoryManipulationFailed)
}

pub fn cwd() -> Result<PathBuf, DevError> {
    std::env::current_dir().map_err(|_| DevError::DirectoryManipulationFailed)
}

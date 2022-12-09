use std::ffi::OsString;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    String(String),
    
    #[error("Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("OsString Error: {0}")]
    OsString(String),
    
    #[error("Error: {0}")]
    FsExtra(#[from] fs_extra::error::Error),
    
    #[cfg(target_os = "windows")]
    #[error("Error: {0}")]
    #[cfg(target_os = "windows")]
    WinRes(#[from] winres_edit::Error)
    
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::String(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::String(s)
    }
}

impl From<OsString> for Error {
    fn from(os_str: OsString) -> Error {
        Error::OsString(format!("{:?}", os_str))
    }
}

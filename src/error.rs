use std::{ffi::OsString, array::TryFromSliceError};
use globset::Error as GlobError;
use thiserror::Error;

cfg_if::cfg_if!{
    if #[cfg(not(target_os = "windows"))] {
        mod winred_edit { pub type Error = String; }
    }
}

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
    
    // #[cfg(target_os = "windows")]
    #[error("Error: {0}")]
    // #[cfg(target_os = "windows")]
    WinRes(#[from] winres_edit::Error),
    
    #[error("Error: {0}")]
    GlobError(#[from] GlobError),
    
    #[error("Error: {0}")]
    TryFromSliceError(#[from] TryFromSliceError),
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

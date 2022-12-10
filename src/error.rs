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
    
    #[error("Unknown architecture: '{0}'")]
    InvalidArchitecture(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("OsString error: {0}")]
    OsString(String),
    
    #[error("FileSystem error: {0}")]
    FsExtra(#[from] fs_extra::error::Error),
    
    #[cfg(target_os = "windows")]
    #[error("Windows resource error: {0}")]
    #[cfg(target_os = "windows")]
    WinRes(#[from] winres_edit::Error),
    
    #[error("Glob error: {0}")]
    GlobError(#[from] GlobError),
    
    #[error("YAML error: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),
    
    // #[error("Error: {0}")]
    // TryFromSliceError(#[from] TryFromSliceError),
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

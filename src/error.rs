use std::{
    ffi::OsString,
    // array::TryFromSliceError
};
use globset::Error as GlobError;
use thiserror::Error;

cfg_if::cfg_if!{
    if #[cfg(not(any(target_os = "windows", feature = "multiplatform")))] {
        #[allow(dead_code)]
        mod winred_edit { pub type Error = String; }
    }
}

#[derive(Error, Debug)]
pub enum Error {

    #[error("abc ")]
    Exit,

    #[error("Error: {0}")]
    String(String),

    #[error("Warning: {0}")]
    Warning(String),
    
    #[error("Unknown architecture: '{0}'")]
    InvalidArchitecture(String),
    
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("OsString error: {0}")]
    OsString(String),
    
    #[error("FileSystem error: {0}")]
    FsExtra(#[from] fs_extra::error::Error),
    
    #[cfg(any(target_os = "windows", feature = "multiplatform"))]
    #[error("Windows resource error: {0}")]
    #[cfg(any(target_os = "windows", feature = "multiplatform"))]
    WinRes(#[from] winres_edit::Error),
    
    #[error("Glob error: {0}")]
    Glob(#[from] GlobError),
    
    #[error("YAML error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
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

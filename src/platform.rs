use cfg_if::cfg_if;
use std::{fmt, str::FromStr};
use clap::Subcommand;
use crate::error::Error;
use serde::{Serialize,Deserialize};

#[derive(Debug, Clone, Subcommand, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

impl Default for Platform {
    fn default() -> Platform {
        cfg_if! {
            // if #[cfg(any(target_os = "linux", feature = "unix"))] {
            if #[cfg(target_os = "linux")] {
                Platform::Linux
            } else if #[cfg(target_os = "macos")] {
                Platform::MacOS
            } else if #[cfg(target_os = "windows")] {
                Platform::Windows
            } else {
                panic!("unsupported platform")
            }
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Windows => write!(f, "windows"),
            Platform::Linux => write!(f, "linux"),
            Platform::MacOS => write!(f, "macos"),
        }
    }
}

impl FromStr for Platform {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        match s {
            "windows" => Ok(Platform::Windows),
            "macos" => Ok(Platform::MacOS),
            "linux" => Ok(Platform::Linux),
            _ => Err(Error::UnknownPlatform(s.to_string())),
        }
    }
}



#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Architecture {
    x64,
    ia32,
    aarch64,
}

impl Default for Architecture {
    fn default() -> Self {
        Architecture::x64
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::x64 => write!(f, "x64"),
            Architecture::ia32 => write!(f, "ia32"),
            Architecture::aarch64 => write!(f, "aarch64"),
        }
    }
}

impl FromStr for Architecture {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x64" => Ok(Architecture::x64),
            "ia32" => Ok(Architecture::ia32),
            "aarch64" => Ok(Architecture::aarch64),
            _ => Err(Error::InvalidArchitecture(s.to_string())),
        }
    }
}

// pub enum Target {
//     Archive,
//     DMG,
//     InnoSetup,
// }

// #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
// pub struct Family {
//     family : String
// }

pub fn family(platform: &Platform, arch: &Architecture) -> String {
    format!("{}-{}", platform, arch)
}

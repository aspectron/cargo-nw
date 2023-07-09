use crate::error::Error;
use crate::result::Result;
use cfg_if::cfg_if;
use clap::Subcommand;
use duct::cmd;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

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
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "windows" => Ok(Platform::Windows),
            "macos" => Ok(Platform::MacOS),
            "linux" => Ok(Platform::Linux),
            _ => Err(Error::UnknownPlatform(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodePlatform {
    Windows,
    Linux,
    MacOS,
}

impl From<Platform> for NodePlatform {
    fn from(platform: Platform) -> Self {
        match platform {
            Platform::Windows => NodePlatform::Windows,
            Platform::Linux => NodePlatform::Linux,
            Platform::MacOS => NodePlatform::MacOS,
        }
    }
}

impl fmt::Display for NodePlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodePlatform::Windows => write!(f, "win32"),
            NodePlatform::Linux => write!(f, "linux"),
            NodePlatform::MacOS => write!(f, "darwin"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NwPlatform {
    Windows,
    Linux,
    MacOS,
}

impl From<Platform> for NwPlatform {
    fn from(platform: Platform) -> Self {
        match platform {
            Platform::Windows => NwPlatform::Windows,
            Platform::Linux => NwPlatform::Linux,
            Platform::MacOS => NwPlatform::MacOS,
        }
    }
}

impl fmt::Display for NwPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NwPlatform::Windows => write!(f, "win"),
            NwPlatform::Linux => write!(f, "linux"),
            NwPlatform::MacOS => write!(f, "osx"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Architecture {
    x64,
    ia32,
    arm64,
}

// impl Default for Architecture {
//     fn default() -> Self {
//         Architecture::x64
//     }
// }

impl Architecture {
    pub fn detect() -> Result<Self> {
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                let uname = cmd!("uname", "-m").read()?;
                match uname.trim() {
                    "x86_64" => Ok(Architecture::x64),
                    "arm64" => Ok(Architecture::arm64),
                    "i386" => Ok(Architecture::ia32),
                    _ => { Err("Unable to determine target platform architecture, please supply via the `--arch=<arch>` argument".into()) }
                }
            }
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::x64 => write!(f, "x64"),
            Architecture::ia32 => write!(f, "ia32"),
            Architecture::arm64 => write!(f, "arm64"),
        }
    }
}

impl FromStr for Architecture {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "x64" => Ok(Architecture::x64),
            "ia32" => Ok(Architecture::ia32),
            "arm64" => Ok(Architecture::arm64),
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

// pub fn family(platform: &Platform, arch: &Architecture) -> String {
//     format!("{}-{}", platform, arch)
// }

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformFamily {
    Windows,
    Unix,
}

impl Default for PlatformFamily {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_family = "windows")] {
                PlatformFamily::Windows
            } else {
                PlatformFamily::Unix
            }
        }
    }
}

impl FromStr for PlatformFamily {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "windows" => Ok(PlatformFamily::Windows),
            "unix" => Ok(PlatformFamily::Unix),
            _ => Err(Error::InvalidFamily(s.to_string())),
        }
    }
}

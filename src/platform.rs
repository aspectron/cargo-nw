use cfg_if::cfg_if;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

impl Default for Platform {
    fn default() -> Platform {
        cfg_if! {
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

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Architecture {
    x64,
    ia32,
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
        }
    }
}


// pub enum Target {
//     Archive,
//     DMG,
//     InnoSetup,
// }

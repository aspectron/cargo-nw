use cfg_if::cfg_if;

#[derive(Debug, Clone)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
}

impl Platform {
    pub fn new() -> Platform {
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

// pub enum Target {
//     Archive,
//     DMG,
//     InnoSetup,
// }

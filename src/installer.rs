use crate::prelude::*;
use async_std::path::PathBuf;
use cfg_if::cfg_if;
use clap::Subcommand;
use std::{collections::HashSet, str::FromStr};

// #[derive(Debug, Clone)]
// pub enum Target {
//     Archive,
//     DMG,
//     InnoSetup,
// }

#[derive(Debug, Clone, Subcommand, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    All,
    Archive,
    #[cfg(any(target_os = "macos", feature = "unix", feature = "multiplatform"))]
    DMG,
    #[cfg(any(target_os = "windows", feature = "multiplatform"))]
    #[clap(name = "innosetup")]
    InnoSetup,
    #[cfg(any(target_os = "linux", feature = "unix", feature = "multiplatform"))]
    Snap,
}

impl ToString for Target {
    fn to_string(&self) -> String {
        match self {
            Target::All => "all",
            Target::Archive => "Archive",
            #[cfg(any(target_os = "macos", feature = "unix", feature = "multiplatform"))]
            Target::DMG => "DMG",
            #[cfg(any(target_os = "windows", feature = "multiplatform"))]
            Target::InnoSetup => "InnoSetup",
            #[cfg(any(target_os = "linux", feature = "unix", feature = "multiplatform"))]
            Target::Snap => "Snap",
        }
        .to_string()
    }
}

impl FromStr for Target {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "all" => Ok(Target::All),
            "archive" => Ok(Target::Archive),
            #[cfg(target_os = "macos")]
            "dmg" => Ok(Target::DMG),
            #[cfg(target_os = "windows")]
            "innosetup" => Ok(Target::InnoSetup),
            #[cfg(any(target_os = "linux", feature = "unix"))]
            "snap" => Ok(Target::Snap),
            _ => Err(format!("Unsupported target: {s}").into()),
        }
    }
}

impl Target {
    pub fn get_all_targets() -> HashSet<Target> {
        // let list =
        cfg_if! {
            if #[cfg(target_os = "macos")] {
                vec![
                    Target::Archive,
                    Target::DMG
                ].into_iter().collect()
            } else if #[cfg(target_os = "windows")] {
                vec![
                    Target::Archive,
                    Target::InnoSetup
                ].into_iter().collect()
            } else if #[cfg(target_os = "linux")] {
                vec![
                    Target::Archive,
                    Target::Snap
                ].into_iter().collect()
            }
        }
    }
}

pub type TargetSet = HashSet<Target>;

#[derive(Default, Debug, Clone, Subcommand, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Channel {
    #[serde(rename = "stable")]
    #[default]
    Stable,
    #[serde(rename = "devel")]
    Devel,
}

impl ToString for Channel {
    fn to_string(&self) -> String {
        match self {
            Channel::Stable => "stable",
            Channel::Devel => "devel",
        }
        .to_string()
    }
}

impl FromStr for Channel {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "stable" => Ok(Channel::Stable),
            "devel" => Ok(Channel::Devel),
            _ => Err(format!("unsupported channel: {s} (must be 'stable' or 'devel')").into()),
        }
    }
}

#[derive(Default, Debug, Clone, Subcommand, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Confinement {
    #[serde(rename = "strict")]
    Strict,
    #[serde(rename = "classic")]
    #[default]
    Classic,
    #[serde(rename = "devmode")]
    Devmode,
}

impl ToString for Confinement {
    fn to_string(&self) -> String {
        match self {
            Confinement::Strict => "strict",
            Confinement::Classic => "classic",
            Confinement::Devmode => "devmode",
        }
        .to_string()
    }
}

impl FromStr for Confinement {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "strict" => Ok(Confinement::Strict),
            "classic" => Ok(Confinement::Classic),
            "devmode" => Ok(Confinement::Devmode),
            _ => Err(format!(
                "unsupported confinement: {s} (must be one of: 'strict','classic','devmode')"
            )
            .into()),
        }
    }
}

#[async_trait]
pub trait Installer {
    async fn init(&self, targets: &TargetSet) -> Result<()>;
    async fn create(&self, targets: &TargetSet) -> Result<Vec<PathBuf>>;
    async fn check(&self, targets: &TargetSet) -> Result<()>;
    fn tpl(&self) -> Tpl;
    fn target_folder(&self) -> PathBuf;
}

pub fn create_installer_tpl(ctx: &Context, target_folder: &PathBuf) -> Tpl {
    let mut tpl = ctx.tpl();
    tpl.set(&[
        // ("$SOURCE",source.to_str().unwrap()),
        // ("$OUTPUT",target_folder.to_str().unwrap()),
        ("TARGET", target_folder.to_str().unwrap()),
    ]);
    // let application = &ctx.manifest.application;
    // let tpl: Tpl = [
    //     ("$SOURCE",source.to_str().unwrap().to_string()),
    //     ("$OUTPUT",output.to_str().unwrap().to_string()),
    //     ("$NAME",application.name.clone()),
    //     ("$TITLE",application.title.clone()),
    //     ("$VERSION",application.version.clone()),
    // ].as_slice().try_into()?;

    tpl
}

pub fn create_installer(ctx: &Arc<Context>) -> Box<dyn Installer> {
    cfg_if! {
        if #[cfg(feature = "unix")] {
            let installer: Box<dyn Installer> = match &ctx.platform {
                Platform::Linux => {
                    Box::new(crate::linux::Linux::new(ctx.clone()))
                },
                Platform::MacOS => {
                    Box::new(crate::macos::MacOS::new(ctx.clone()))
                },
                Platform::Windows => {
                    panic!("Windows platform can not be used in *nix environment");
                }
            };
        } else
        if #[cfg(target_os = "windows")] {
            let installer: Box<dyn Installer> = Box::new(crate::windows::Windows::new(ctx.clone()));
        } else if #[cfg(target_os = "macos")] {
            let installer: Box<dyn Installer> = Box::new(crate::macos::MacOS::new(ctx.clone()));
            // Box::new(windows::Windows::new(self.ctx.clone()))
        } else if #[cfg(target_os = "linux")] {
            let installer: Box<dyn Installer> = Box::new(crate::linux::Linux::new(ctx.clone()));
        }
    }

    installer
}

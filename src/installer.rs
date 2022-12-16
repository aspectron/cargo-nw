use crate::prelude::*;
use cfg_if::cfg_if;
use async_std::path::PathBuf;
use clap::Subcommand;
use std::{collections::HashSet, str::FromStr};

// #[derive(Debug, Clone)]
// pub enum Target {
//     Archive,
//     DMG,
//     InnoSetup,
// }

#[derive(Debug, Clone, Subcommand, Hash, PartialEq, Eq)]
pub enum Target {
    All,
    Archive,
    #[cfg(any(target_os = "macos", feature = "unix"))]
    DMG,
    #[cfg(target_os = "windows")]
    #[clap(name = "innosetup")]
    InnoSetup,
    #[cfg(any(target_os = "linux", feature = "unix"))]
    Snap,
}

impl ToString for Target {
    fn to_string(&self) -> String {
        match self {
            Target::All => "all",
            Target::Archive => "Archive",
            #[cfg(any(target_os = "macos", feature = "unix"))]
            Target::DMG => "DMG",
            #[cfg(target_os = "windows")]
            Target::InnoSetup => "InnoSetup",
            #[cfg(any(target_os = "linux", feature = "unix"))]
            Target::Snap => "Snap",
        }.to_string()
    }
}

impl FromStr for Target {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        match s {
            "all" => Ok(Target::All),
            "archive" => Ok(Target::Archive),
            #[cfg(target_os = "macos")]
            "dmg" => Ok(Target::DMG),
            #[cfg(target_os = "windows")]
            "innosetup" => Ok(Target::InnoSetup),
            #[cfg(any(target_os = "linux", feature = "unix"))]
            "snap" => Ok(Target::Snap),
            _ => Err(format!("Unsupported target: {}", s).into()),
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


#[derive(Debug, Clone, Subcommand, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Channel {
    #[serde(rename = "stable")]
    Stable,
    #[serde(rename = "devel")]
    Devel
}

impl Default for Channel {
    fn default() -> Self {
        Channel::Stable
    }
}

impl ToString for Channel {
    fn to_string(&self) -> String {
        match self {
            Channel::Stable => "stable",
            Channel::Devel => "devel",
        }.to_string()
    }
}

impl FromStr for Channel {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        match s {
            "stable" => Ok(Channel::Stable),
            "devel" => Ok(Channel::Devel),
            _ => Err(format!("unsupported channel: {} (must be 'stable' or 'devel')", s).into()),
        }
    }
}

#[derive(Debug, Clone, Subcommand, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Confinement {
    #[serde(rename = "strict")]
    Strict,
    #[serde(rename = "classic")]
    Classic,
    #[serde(rename = "devmode")]
    Devmode,
}

impl Default for Confinement {
    fn default() -> Self {
        Confinement::Strict
        // Confinement::Classic
    }
}

impl ToString for Confinement {
    fn to_string(&self) -> String {
        match self {
            Confinement::Strict => "strict",
            Confinement::Classic => "classic",
            Confinement::Devmode => "devmode",
        }.to_string()
    }
}

impl FromStr for Confinement {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        match s {
            "strict" => Ok(Confinement::Strict),
            "classic" => Ok(Confinement::Classic),
            "devmode" => Ok(Confinement::Devmode),
            _ => Err(format!("unsupported confinement: {} (must be one of: 'strict','classic','devmode')", s).into()),
        }
    }
}

#[async_trait]
pub trait Installer {
    async fn create(&self, targets: TargetSet) -> Result<Vec<PathBuf>>;
    async fn check(&self, targets: TargetSet) -> Result<()>;
    fn target_folder(&self) -> PathBuf;
}

pub fn create_installer_tpl(ctx: &Context, source: &PathBuf, output: &PathBuf) -> Result<Tpl> {
    let application = &ctx.manifest.application;
    let tpl: Tpl = [
        ("$SOURCE",source.to_str().unwrap().to_string()),
        ("$OUTPUT",output.to_str().unwrap().to_string()),
        ("$NAME",application.name.clone()),
        ("$TITLE",application.title.clone()),
        ("$VERSION",application.version.clone()),
    ].as_slice().try_into()?;

    Ok(tpl)
}
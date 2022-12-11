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
    #[cfg(target_os = "macos")]
    DMG,
    #[cfg(target_os = "windows")]
    #[clap(name = "innosetup")]
    InnoSetup,
    #[cfg(target_os = "linux")]
    Snap,
}

impl ToString for Target {
    fn to_string(&self) -> String {
        match self {
            Target::All => "all",
            Target::Archive => "Archive",
            #[cfg(target_os = "macos")]
            Target::DMG => "DMG",
            #[cfg(target_os = "windows")]
            Target::InnoSetup => "InnoSetup",
            #[cfg(target_os = "linux")]
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
            #[cfg(target_os = "linux")]
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

#[async_trait]
pub trait Installer {
    async fn create(&self, targets: TargetSet) -> Result<Vec<PathBuf>>;
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
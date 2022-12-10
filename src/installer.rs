use crate::prelude::*;
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
    Archive,
    // #[cfg(target_os = "macos")]
    DMG,
    // #[cfg(target_os = "windows")]
    InnoSetup,
    // #[cfg(target_os = "linux")]
    Snap,
}

impl FromStr for Target {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        match s {
            "dmg" => Ok(Target::DMG),
            #[cfg(target_os = "macos")]
            "archive" => Ok(Target::Archive),
            #[cfg(target_os = "windows")]
            "innosetup" => Ok(Target::InnoSetup),
            #[cfg(target_os = "linux")]
            "snap" => Ok(Target::Snap),
            _ => Err(format!("Unsupported target: {}", s).into()),
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
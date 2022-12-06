use crate::prelude::*;
use async_std::path::PathBuf;
use clap::Subcommand;
use std::collections::HashSet;

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
}

pub type TargetSet = HashSet<Target>;

#[async_trait]
pub trait Installer {
    async fn create(&self, targets: TargetSet) -> Result<Vec<PathBuf>>;
}
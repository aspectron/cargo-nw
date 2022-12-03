use crate::prelude::*;
use async_std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum InstallerType {
    Archive,
    DMG,
    InnoSetup,
}

#[async_trait]
pub trait Installer {
    async fn create(&self, package_type: InstallerType) -> Result<Vec<PathBuf>>;
}
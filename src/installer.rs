use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum InstallerType {
    Archive,
    DMG,
    InnoSetup,
}

#[async_trait]
pub trait Installer {
    async fn create(&self, ctx: &Context, package_type: InstallerType) -> Result<()>;
}
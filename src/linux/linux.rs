use async_std::path::PathBuf;
use crate::prelude::*;

pub struct Linux {
    ctx : Arc<Context>,
}

impl Linux {
    pub fn new(ctx: Arc<Context>) -> Linux {
        Linux {
            ctx
        }
    }
}

#[async_trait]
impl Installer for Linux {
    async fn create(&self, installer_type: InstallerType) -> Result<Vec<PathBuf>> {

        println!("[linux] creating {:?} installer",installer_type);


        Ok(vec![])
    }
}

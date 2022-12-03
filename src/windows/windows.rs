use async_std::path::PathBuf;
use crate::prelude::*;

pub struct Windows {
    ctx : Arc<Context>,
}

impl Windows {
    pub fn new(ctx: Arc<Context>) -> Windows {
        Windows {
            ctx
        }
    }
}

#[async_trait]
impl Installer for Windows {
    async fn create(&self, installer_type: InstallerType) -> Result<Vec<PathBuf>> {
        log!("Windows","creating {:?} installer",installer_type);

        Ok(vec![])
    }
}
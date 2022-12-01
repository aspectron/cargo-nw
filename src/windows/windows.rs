
use crate::prelude::*;

pub struct Windows {

}

impl Windows {
    pub fn new(_ctx: &Context) -> Windows {
        Windows {
        }
    }
}

#[async_trait]
impl Installer for Windows {
    async fn create(&self, _ctx : &Context, installer_type: InstallerType) -> Result<()> {
        println!("[windows] creating {:?} installer",installer_type);

        Ok(())
    }
}
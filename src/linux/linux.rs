
use crate::prelude::*;

pub struct Linux {

}

impl Linux {
    pub fn new(_ctx: &Context) -> Linux {
        Linux {
        }
    }
}

#[async_trait]
impl Installer for Linux {
    async fn create(&self, _ctx : &Context, installer_type: InstallerType) -> Result<()> {

        println!("[linux] creating {:?} installer",installer_type);


        Ok(())
    }
}

use async_std::path::PathBuf;
use crate::prelude::*;
use super::iss::ISS;

pub struct Windows {
    ctx : Arc<Context>,
    // nwjs_root_folder: PathBuf,
}

impl Windows {
    pub fn new(ctx: Arc<Context>) -> Windows {

        // let nwjs_root_folder = ctx.build_folder.join(&ctx.manifest.application.title);

        Windows {
            ctx,
            // nwjs_root_folder
        }
    }
}

#[async_trait]
impl Installer for Windows {
    async fn create(&self, installer_type: InstallerType) -> Result<Vec<PathBuf>> {
        log!("Windows","creating {:?} installer",installer_type);

        match installer_type {
            InstallerType::Archive => {
                Ok(vec![])
            },
            InstallerType::InnoSetup => {


                let setup_script = ISS::new(
                    self.ctx.clone()
                    // &self.ctx.manifest.application.name,
                    // &self.ctx.manifest.application.title,
                    // &self.ctx.manifest.application.version,
                    // &self.ctx.platform.to_string(),
                    // &self.ctx.arch.to_string(),
                    // self.nwjs_root_folder.clone(),
                    // self.ctx.build_folder.clone(),
                    // self.ctx.output_folder.clone(),
                );

                setup_script.create().await?;
                Ok(vec![])
            },
            _ => {
                Err(format!("Unsupported installer type: {:?}", installer_type).into())
            }
        }
    }
}
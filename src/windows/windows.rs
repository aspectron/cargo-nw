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
    async fn create(&self, targets: TargetSet) -> Result<Vec<PathBuf>> {
        
        if targets.contains(&Target::Archive) {
            log!("Windows","creating archive");
            
        }

        if targets.contains(&Target::InnoSetup) {

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
        }

        // ^ TODO - list all files...
        Ok(vec![])
    }
}
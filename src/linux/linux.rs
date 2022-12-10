use async_std::path::{PathBuf, Path};
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
    async fn create(&self, targets: TargetSet) -> Result<Vec<PathBuf>> {

        self.copy_nwjs_folder().await?;
        self.copy_app_data().await?;

        let mut files = Vec::new();

        if targets.contains(&Target::Archive)  || self.ctx.manifest.package.archive.unwrap_or(false) {
            log!("Linux","creating archive");
            
            let filename = Path::new(&format!("{}.tgz",self.ctx.app_snake_name)).to_path_buf();
            files.push(filename);
        }

        if targets.contains(&Target::Snap) {
            log!("Linux","creating SNAP package");
            
            // let filename = Path::new(format!("{}.zip",self.ctx.app_snake_name)).to_path_buf();
            // files.push(filename);
            let snap = crate::linux::snap::Snap::new(&self.ctx);
            snap.create().await?;
        }

        Ok(vec![])
    }

    async fn copy_nwjs_folder(&self) -> Result<()>{
        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.skip_exist = true;
        
        log!("Integrating","NWJS binaries");
        dir::copy(
            Path::new(&self.ctx.deps.nwjs.target),
            &self.nwjs_root_folder, 
            &options
        )?;

        Ok(())
    }

    async fn copy_app_data(&self) -> Result<()> {
        log!("Integrating","application data");
        copy_folder_with_glob_filters(
            &self.ctx.app_root_folder,
            &self.nwjs_root_folder,
            self.ctx.manifest.package.include.clone(),
            self.ctx.manifest.package.exclude.clone(),
            self.ctx.manifest.package.hidden.unwrap_or(false),
        ).await?;
        Ok(())
    }


}

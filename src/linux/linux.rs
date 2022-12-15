use async_std::path::{PathBuf, Path};
use fs_extra::dir;
use async_std::fs;
use crate::prelude::*;

pub struct Linux {
    ctx : Arc<Context>,
    pub nwjs_root_folder : PathBuf,
}

impl Linux {
    pub fn new(ctx: Arc<Context>) -> Linux {

        let nwjs_root_folder = ctx.build_folder.join(&ctx.app_snake_name);

        Linux {
            ctx,
            nwjs_root_folder
        }
    }
}

#[async_trait]
impl Installer for Linux {
    async fn create(&self, targets: TargetSet) -> Result<Vec<PathBuf>> {

        self.copy_nwjs_folder().await?;
        self.copy_app_data().await?;

        let tpl = create_installer_tpl(
            &self.ctx,
            &self.ctx.app_root_folder,
            &self.nwjs_root_folder
        )?;

        if let Some(actions) = &self.ctx.manifest.package.execute {
            for action in actions {
                log_info!("Build","executing pack action");
                if let Execute::Pack(ec) = action {
                    log_info!("Linux","executing `{}`",ec.display(Some(&tpl)));
                    self.ctx.execute_with_context(ec, Some(&self.nwjs_root_folder), None).await?;
                }
            }
        }

        let mut files = Vec::new();

        if targets.contains(&Target::Archive) {
            log_info!("Linux","creating archive");
            
            let level = self.ctx.manifest.package.archive.clone().unwrap_or_default();
            let filename = Path::new(&format!("{}.zip",self.ctx.app_snake_name)).to_path_buf();
            let target_file = self.ctx.output_folder.join(&filename);
            compress_folder(
                &self.nwjs_root_folder,
                &target_file,
                level
            )?;

            files.push(target_file);
        }

        #[cfg(target_os = "linux")]
        if targets.contains(&Target::Snap) {
            log_info!("Linux","creating SNAP package");
            
            // let filename = Path::new(format!("{}.zip",self.ctx.app_snake_name)).to_path_buf();
            // files.push(filename);
            let snap = crate::linux::snap::Snap::new(&self.ctx);
            snap.create().await?;
        }

        Ok(files)
    }

    fn target_folder(&self) -> PathBuf {
        self.nwjs_root_folder.clone()
    }

}

impl Linux {

    async fn copy_nwjs_folder(&self) -> Result<()>{
        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.skip_exist = true;
        
        log_info!("Integrating","NW binaries");
        dir::copy(
            Path::new(&self.ctx.deps.nwjs.target()),
            &self.nwjs_root_folder, 
            &options
        )?;

        if self.ctx.manifest.node_webkit.ffmpeg.unwrap_or(false) {
            log_info!("Integrating","FFMPEG binaries");
            fs::create_dir_all(self.nwjs_root_folder.join("lib")).await?;
            fs::copy(
                Path::new(&self.ctx.deps.ffmpeg.as_ref().unwrap().target()).join("ffmpeg.dll"),
                self.nwjs_root_folder.join("lib").join("ffmpeg.dll")
            ).await?;
        }

        Ok(())
    }

    async fn copy_app_data(&self) -> Result<()> {
        log_info!("Integrating","application data");

        let tpl = self.ctx.tpl_clone();
        copy_folder_with_filters(
            &self.ctx.app_root_folder,
            &self.nwjs_root_folder,
            (&tpl,&self.ctx.include,&self.ctx.exclude).try_into()?,
            CopyOptions::new(self.ctx.manifest.package.hidden.unwrap_or(false)),
        ).await?;
        Ok(())
    }


}

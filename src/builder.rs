use async_std::path::Path;
use console::style;
use std::time::Instant;
use crate::prelude::*;

pub struct Builder {
    pub ctx : Arc<Context>
}

impl Builder {
    pub fn new(ctx: Arc<Context>) -> Self {
        Builder {
            ctx
        }
    }


    pub async fn execute(&self, targets: TargetSet) -> Result<()> {


        println!("{:#?}", self.ctx.manifest);

        let snap = crate::linux::snap::Snap::new(&self.ctx);
        snap.store(Path::new("snap.yaml"))?;

        return Ok(());

        let ts_start = Instant::now();
        log!("Build","Building {} Version {}",style(&self.ctx.manifest.application.title).cyan(),style(&self.ctx.manifest.application.version).cyan());
        log!("Build","Installer type: {}",style(format!("{:?}", targets)).cyan());

        self.ctx.clean().await?;
        self.ctx.deps.ensure().await?;
        self.ctx.ensure_folders().await?;

        if let Some(actions) = &self.ctx.manifest.application.execute {
            log!("Build","Executing build actions");
            for action in actions {
                if let Execute::Build { cmd, folder, platform, arch } = action {
                    execute(&self.ctx,cmd,folder,platform,arch).await?;
                }
            }
        }

        cfg_if! {
            if #[cfg(target_os = "windows")] {
                let installer = Box::new(crate::windows::Windows::new(self.ctx.clone()));
            } else if #[cfg(target_os = "macos")] {
                let installer = Box::new(crate::macos::MacOS::new(self.ctx.clone()));
                // Box::new(windows::Windows::new(self.ctx.clone()))
            } else if #[cfg(target_os = "linux")] {
                let installer = Box::new(crate::linux::Linux::new(self.ctx.clone()));
            }
        }

        let files = installer.create(targets).await?;

        if files.is_empty() {
            panic!("Build produced no output");
        }

        let duration = ts_start.elapsed();

        for file in files {
            generate_sha256sum(&file).await?;
        }

        // let package_name = files[0].to_str().unwrap();
        // log!("Finished","{} package{} in {:.2}s", style(package_name).cyan(), duration.as_millis() as f64/1000.0);
        // let suffix = files.len()
        let packages = if files.len() > 1 { "packages" } else { "package" };
        log!("Finished","build of ({} {}) completed in{:.2}s", files.len(), packages, duration.as_millis() as f64/1000.0);

        if let Some(actions) = &self.ctx.manifest.application.execute {
            log!("Build","Executing deploy actions");
            for action in actions {
                if let Execute::Deploy { cmd, folder, platform, arch } = action {
                    execute(&self.ctx,cmd,folder,platform,arch).await?;
                }
            }
        }

        println!("");

        Ok(())
    }

}
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

        // if let Some(builds) = self.ctx.manifest.build {
        //     for Build { cmd, folder } in builds {
        //         let folder = if let Some(folder) = folder {
        //             self.ctx.app_root_folder.join(folder)
        //         } else {
        //             self.ctx.app_root_folder
        //         };
        //         if let Err(e) = spawn(&cmd,&folder).await {
        //             println!("Error executing build setup: {}", cmd);
        //             println!("{}", e);
        //         }
        //     }
        // }
        if let Some(actions) = self.ctx.manifest.application.execute {
            for action in actions {
                if let Execute::Build { cmd, folder } = action {
                    self.run(&cmd,&folder).await?;
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
        log!("Finished","build completed in{:.2}s", duration.as_millis() as f64/1000.0);
        println!("");

        if let Some(actions) = self.ctx.manifest.application.execute {
            for action in actions {
                if let Execute::Deploy { cmd, folder } = action {
                    self.run(&cmd,&folder).await?;
                }
            }
        }
        // if let Some(deploys) = self.ctx.manifest.deploy {
        //     for Deploy { cmd, folder } in deploys {
        //         let folder = if let Some(folder) = folder {
        //             self.ctx.app_root_folder.join(folder)
        //         } else {
        //             self.ctx.app_root_folder
        //         };
        //         if let Err(e) = spawn(&cmd,&folder).await {
        //             println!("Error executing build setup: {}", cmd);
        //             println!("{}", e);
        //         }
        //     }
        // }



        Ok(())
    }

    pub async fn run(&self, cmd : &str, folder : &Option<String>) -> Result<()> {

        let folder = if let Some(folder) = folder {
            self.ctx.app_root_folder.join(folder)
        } else {
            self.ctx.app_root_folder.clone()
        };
        if let Err(e) = spawn(&cmd,&folder).await {
            println!("Error executing run action: {}", cmd);
            println!("{}", e);
        }

        Ok(())
    }
}
// use std::path::Path;
use console::style;
// use std::sync::Arc;
use std::time::Instant;
// use async_std::path::PathBuf;
// use trauma::{download::Download, downloader::DownloaderBuilder, Error};

// use crate::platform::*;
// use crate::manifest::*;
// use crate::utils::*;
// use crate::context::*;

use crate::prelude::*;
use crate::{windows,linux,macos};
// pub struct NwjsFiles {
//     ffmpeg : String,

// }

// use fs_extra::dir;
// use fs_extra::dir::CopyOptions;

// this.NWJS_SUFFIX = { windows : 'win', darwin : 'osx', linux : 'linux' }[PLATFORM];
// 		this.NWJS_ARCHIVE_EXTENSION = { windows : 'zip', darwin : 'zip', 'linux' : 'tar.gz' }[PLATFORM];

pub struct Builder {
    pub ctx : Arc<Context>
}

impl Builder {
    pub fn new(ctx: Arc<Context>) -> Self {
        Builder {
            ctx
        }
    }

    // pub async fn execute(&self) -> Result<&Self> {
    //     let cwd = std::env::current_dir()?;

    //     let argv : Vec<String> = self.cmd.split(" ").map(|s|s.to_string()).collect();
    //     let program = argv.first().expect("missing program in build config");
    //     let args = argv[1..].to_vec();
    //     cmd(program,args).dir(cwd.join(&self.folder)).run()?;

    //     Ok(self)
    // }

    pub async fn execute(&self, installer_type: InstallerType) -> Result<()> {


        let ts_start = Instant::now();
        log!("Build","Building {} Version {}",style(&self.ctx.manifest.application.title).cyan(),style(&self.ctx.manifest.application.version).cyan());
        log!("Build","Installer type: {}",style(format!("{:?}", installer_type)).cyan());

        self.ctx.clean().await?;
        
        self.ctx.deps.ensure().await?;
        
        self.ctx.ensure_folders().await?;

        // let installer = match installer_type

        let installer: Box<dyn Installer> = match &self.ctx.platform {
            Platform::Windows => {
                Box::new(windows::Windows::new(self.ctx.clone()))
                
                
            },
            Platform::Linux => {
                Box::new(linux::Linux::new(self.ctx.clone()))
                
            },
            Platform::MacOS => {
                Box::new(windows::Windows::new(self.ctx.clone()))
                // Box::new(macos::MacOS::new(self.ctx.clone()))
            }
        };

        let files = installer.create(installer_type).await?;

        if files.is_empty() {
            panic!("Build produced no output");
        }

        let duration = ts_start.elapsed();
        let package_name = files[0].to_str().unwrap();
        log!("Finished","package '{}' in {:.2}s", style(package_name).cyan(), duration.as_millis() as f64/1000.0);
        println!("");
        Ok(())
    }

    /*
    ^ IF MISSING
        ^ DOWNLOAD ALL DEPENDENCIES
        ^ EXTRACT ALL DEPENDENCIES
    ^ COPY NWJS TO TARGET
    ^ COPY PROJECT TO TARGET
    ^ COPY ICONS/IMAGES + RENAME + PLIST TO TARGET (MACOS)
    ^ PACKAGE DMG
    
    */
}
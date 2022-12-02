// use std::path::Path;
use console::style;
use std::time::Instant;
// use std::path::PathBuf;
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

pub struct Build {
    pub ctx : Context
}

impl Build {
    pub fn new(ctx: Context) -> Self {
        Build {
            ctx
        }
    }

    pub async fn execute(&self, installer_type: InstallerType) -> Result<()> {


        let ts_start = Instant::now();
        log!("Build","Building {} Version {}",style(&self.ctx.manifest.application.title).cyan(),style(&self.ctx.manifest.application.version).cyan());
        log!("Build","Installer type: {}",style(format!("{:?}", installer_type)).cyan());

        self.ctx.clean().await?;
        
        self.ctx.deps.ensure().await?;
        
        self.ctx.ensure_folders().await?;


        let installer: Box<dyn Installer> = match &self.ctx.platform {
            Platform::Windows => {
                Box::new(windows::Windows::new(&self.ctx))
                

            },
            Platform::Linux => {
                Box::new(linux::Linux::new(&self.ctx))

            },
            Platform::MacOS => {
                Box::new(macos::MacOS::new(&self.ctx))
            }
        };

        installer.create(&self.ctx, installer_type).await?;

        let duration = ts_start.elapsed();
        let package_name = "<filename>";
        log!("Finished","package '{}' in {:.2}s", style(package_name).cyan(), duration.as_millis() as f64/1000.0);

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
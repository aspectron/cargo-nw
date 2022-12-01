use std::path::Path;
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

use fs_extra::dir;
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

    pub async fn execute(&self) -> Result<()> {

        self.ctx.deps.ensure().await?;

        self.ctx.ensure_folders().await?;

        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.skip_exist = true;
        
        // copy source/dir1 to target/dir1
        // let src = self.ctx.deps.nwjs.get_extract_path()
        println!("[build] copying NWJS binaries");
        dir::copy(
            Path::new(&self.ctx.deps.nwjs.target).join("nwjs.app"), 
            &self.ctx.nwjs_root_folder, 
            &options
        )?;

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

        installer.create(&self.ctx, InstallerType::Archive).await?;

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
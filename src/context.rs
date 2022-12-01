use async_std::path::Path;
use async_std::path::PathBuf;
use std::env;
use crate::prelude::*;

#[derive(Debug)]
pub struct Options {
    pub sdk : bool
}

impl Default for Options {
    fn default() -> Self {
        Options {
            sdk: false
        }
    }
}

#[derive(Debug)]
pub struct Context {

    pub manifest : Manifest,
    pub platform : Platform,

    
    pub home_folder : PathBuf,
    // pub deps_dir : PathBuf,
    pub nwjs_root_folder : PathBuf,
    pub app_root_folder : PathBuf,
    pub setup_resources_folder : PathBuf,
    // pub app_root_folder : PathBuf,
    // pub target_dir : PathBuf,
    
    pub sdk : bool,
    pub deps : Dependencies,
}


impl Context {
    pub fn new(
        platform: Platform, 
        manifest: Manifest,
        options: Options,
    ) -> Context {
        let home_folder: PathBuf = home::home_dir().unwrap().into();
        // let deps_dir: PathBuf = Path::new(&home_dir).join(".nwjs");
        let cargo_target_folder : PathBuf = env::current_dir().unwrap().join("target").into();
        let nwjs_root_folder : PathBuf = Path::new(&cargo_target_folder).join(&manifest.application.title);
        
        let app_root_folder : PathBuf = env::current_dir().unwrap().join("root").into();
        let setup_resources_folder : PathBuf = env::current_dir().unwrap().join(&manifest.application.resources.as_ref().unwrap_or(&"resources".to_string())).into();
        // let app_root_folder : PathBuf = Path::new(&cargo_target_folder).join(&manifest.package.title).join("nw.app");

        let sdk = manifest.nwjs.sdk.unwrap_or(options.sdk);

        let deps = Dependencies::new(&platform,&manifest,sdk);

        Context {
            manifest,
            platform,
            home_folder,
            nwjs_root_folder,
            app_root_folder,
            setup_resources_folder,
            // app_root_folder,
            sdk,
            deps,
        }
    }

    pub async fn ensure_folders(&self) -> Result<()> {
        if !std::path::Path::new(&self.nwjs_root_folder).exists() {
            std::fs::create_dir_all(&self.nwjs_root_folder)?;
        }
        Ok(())
    }

    pub async fn clean(&self) -> Result<()> {
        if self.nwjs_root_folder.exists().await {
            async_std::fs::remove_dir_all(&self.nwjs_root_folder).await?;
        }
        Ok(())
    }

}
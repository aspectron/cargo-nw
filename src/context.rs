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
    pub cargo_target_folder : PathBuf,
    pub app_root_folder : PathBuf,
    pub setup_resources_folder : PathBuf,
    // pub app_root_folder : PathBuf,
    // pub build_cache_folder : PathBuf,
    pub build_folder : PathBuf,
    pub output_folder : PathBuf,
    
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
        let cargo_nwjs_target_folder : PathBuf = cargo_target_folder.join("nwjs");

        // let application_folder_name = if platform == Platform::MacOS {
        //     format!("{}.app", &manifest.application.title)
        // } else {
        //     manifest.application.title.clone()
        // };

        let build_folder : PathBuf = Path::new(&cargo_nwjs_target_folder).join("build");//.join(application_folder_name);
        // let build_cache_folder : PathBuf = Path::new(&cargo_nwjs_target_folder).join("cache").join(&manifest.application.title);
        
        let app_root_folder : PathBuf = env::current_dir().unwrap().join("root").into();
        let setup_resources_folder : PathBuf = env::current_dir().unwrap().join(&manifest.application.resources.as_ref().unwrap_or(&"resources".to_string())).into();
        // let app_root_folder : PathBuf = Path::new(&cargo_target_folder).join(&manifest.package.title).join("nw.app");
        let output_folder : PathBuf = Path::new(&cargo_nwjs_target_folder).join("output");//.join(&manifest.application.title);

        let sdk = manifest.nwjs.sdk.unwrap_or(options.sdk);

        let deps = Dependencies::new(&platform,&manifest,sdk);

        Context {
            manifest,
            platform,
            home_folder,
            cargo_target_folder,
            build_folder,
            app_root_folder,
            setup_resources_folder,

            // build_cache_folder,
            output_folder,

            // app_root_folder,
            sdk,
            deps,
        }
    }

    pub async fn ensure_folders(&self) -> Result<()> {
        if !std::path::Path::new(&self.build_folder).exists() {
            std::fs::create_dir_all(&self.build_folder)?;
        }
        Ok(())
    }

    pub async fn clean(&self) -> Result<()> {
        if self.build_folder.exists().await {
            async_std::fs::remove_dir_all(&self.build_folder).await?;
        }
        Ok(())
    }

}
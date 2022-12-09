use async_std::path::Path;
use async_std::path::PathBuf;
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
    pub arch : Architecture,
    
    pub home_folder : PathBuf,
    /// Cargo `target` folder
    pub cargo_target_folder : PathBuf,
    /// Source application folder
    pub app_root_folder : PathBuf,
    /// Folder that contains setup resources
    pub setup_resources_folder : PathBuf,
    pub build_cache_folder : PathBuf,
    pub build_folder : PathBuf,
    pub output_folder : PathBuf,
    
    pub sdk : bool,
    pub deps : Dependencies,
}


impl Context {
    pub async fn create(
        platform: Platform, 
        arch : Architecture,
        manifest: Manifest,
        project_root: &Path,
        options: Options,
    ) -> Result<Context> {
        let home_folder: PathBuf = home::home_dir().unwrap().into();
        let cwd = current_dir().await;
        let cargo_toml_folder = search_upwards(&cwd,"Cargo.toml").await
            .map(|location|location.parent().unwrap().to_path_buf())
            .unwrap_or(cwd.clone());
        let cargo_target_folder = cargo_toml_folder.join("target");
        let cargo_nw_target_folder = cargo_target_folder.join("nw");
        let build_folder = Path::new(&cargo_nw_target_folder).join("build");//.join(application_folder_name);
        let build_cache_folder = Path::new(&cargo_nw_target_folder).join("cache").join(&manifest.application.title);

        let project_root = project_root.to_path_buf();
        let app_root_folder = manifest.application.root.as_ref()
            .map(|root|project_root.to_path_buf().join(root))
            .unwrap_or(project_root);

        // let app_root_folder = if let Some(root) = manifest.application.root {
        //     project_root.to_path_buf();
        // } else {
        //     project_root.to_path_buf();
        // };

        let setup_resources_folder = cwd.join(&manifest.application.resources.as_ref().unwrap_or(&"resources".to_string())).into();
        let output_folder = Path::new(&cargo_nw_target_folder).join("setup");//.join(&manifest.application.title);
        let sdk = manifest.nwjs.sdk.unwrap_or(options.sdk);
        let deps = Dependencies::new(&platform,&manifest,sdk);

        let ctx = Context {
            manifest,
            platform,
            arch,
            home_folder,
            cargo_target_folder,
            build_folder,
            app_root_folder,
            setup_resources_folder,

            build_cache_folder,
            output_folder,

            // app_root_folder,
            sdk,
            deps,
        };

        Ok(ctx)
    }

    pub async fn ensure_folders(&self) -> Result<()> {
        let folders = [&self.build_folder, &self.output_folder];
        for folder in folders {
            if !std::path::Path::new(folder).exists() {
                std::fs::create_dir_all(folder)?;
            }
        }
        // if !std::path::Path::new(&self.build_folder).exists() {
        // }
        // if !std::path::Path::new(&self.build_folder).exists() {
        //     std::fs::create_dir_all(&self.build_folder)?;
        // }
        Ok(())
    }

    pub async fn clean(&self) -> Result<()> {
        if self.build_folder.exists().await {
            async_std::fs::remove_dir_all(&self.build_folder).await?;
        }
        Ok(())
    }

}
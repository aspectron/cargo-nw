use std::sync::Mutex;

use async_std::path::Path;
use async_std::path::PathBuf;
use crate::prelude::*;
use path_dedot::*;

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
    /// Project folder (nw.toml location). Can be the same as [`app_root_folder`]
    pub project_root_folder : PathBuf,
    /// Folder that contains setup resources
    pub setup_resources_folder : PathBuf,
    // pub app_snake_name : PathBuf,
    pub cache_folder : PathBuf,
    pub build_folder : PathBuf,
    pub output_folder : PathBuf,
    
    pub app_snake_name : String,

    pub include : Vec<String>,
    pub exclude : Vec<String>,

    pub sdk : bool,
    pub deps : Dependencies,
    pub tpl : Arc<Mutex<Tpl>>,
}


impl Context {
    pub async fn create(
        location : Option<String>,
        output : Option<String>,
        platform: Platform, 
        arch : Architecture,
        options: Options,
    ) -> Result<Context> {
        println!("");

        let home_folder: PathBuf = home::home_dir().unwrap().into();
        let manifest_toml = Manifest::locate(location).await?;
        log!("Manifest","`{}`",manifest_toml.to_str().unwrap());
        let manifest_folder = manifest_toml.parent().unwrap().to_path_buf();
        let manifest = Manifest::load(&manifest_toml).await?;
        let project_root = manifest_toml.parent().unwrap();
        let app_snake_name = format!("{}-{}-{}-{}",
            manifest.application.name,
            manifest.application.version,
            platform,
            arch
        );

        let cargo_toml_folder = search_upwards(&manifest_folder,"Cargo.toml").await
            .map(|location|location.parent().unwrap().to_path_buf())
            .unwrap_or(manifest_folder.clone());
        let cargo_target_folder = cargo_toml_folder.join("target");
        let cargo_nw_target_folder = cargo_target_folder.join("nw");
        let build_folder = Path::new(&cargo_nw_target_folder).join("build").join(&app_snake_name);
        let cache_folder = Path::new(&cargo_nw_target_folder).join("cache").join(&app_snake_name);
        
        let output_folder = if let Some(output) = output.or(manifest.package.output.clone()) {
            let output = Path::new(&output);
            if output.is_absolute() {
                output.to_owned()
            } else {
                project_root.join(output)
            }
        } else {
            Path::new(&cargo_nw_target_folder).join("setup")
        };

        let project_root_folder = project_root.to_path_buf();
        let app_root_folder = manifest.package.root.as_ref()
            .map(|root|project_root_folder.to_path_buf().join(root))
            .unwrap_or(project_root_folder.clone());
        let app_root_folder: PathBuf = std::path::PathBuf::from(&app_root_folder).parse_dot()?.to_path_buf().into();

        let setup_resources_folder = manifest_folder.join(&manifest.package.resources.as_ref().unwrap_or(&"resources".to_string())).into();
        let sdk = manifest.node_webkit.sdk.unwrap_or(options.sdk);
        let deps = Dependencies::new(&platform,&manifest,sdk);

        let include = manifest.package.include.clone().unwrap_or(vec![]);
        let mut exclude = manifest.package.exclude.clone().unwrap_or(vec![]);

        log!("Target","`{}`",output_folder.to_str().unwrap());

        if manifest.package.gitignore.unwrap_or(true) {
            let gitignore = app_root_folder.join(".gitignore");
            if gitignore.exists().await {
                let gitignore = match std::fs::read_to_string(&gitignore) {
                    Ok(gitignore) => gitignore,
                    Err(e) => {
                        return Err(format!("Unable to open '{}' - {}",gitignore.display(),e).into());
                    }
                };
                let list = gitignore
                    .split("\n")
                    .filter(|s|!s.is_empty())
                    .map(|s|s.to_string())
                    .collect::<Vec<_>>();
                exclude.extend(list);
            }
        }

        let tpl : Tpl = [
            ("$ROOT",app_root_folder.to_str().unwrap().to_string()),
            ("$NAME",manifest.application.name.clone()),
            ("$VERSION",manifest.application.version.clone()),
            ("$OUTPUT",output_folder.to_str().unwrap().to_string()),
            ("$PLATFORM",platform.to_string()),
            ("$ARCH",arch.to_string()),
        ].as_slice().try_into()?;

        let ctx = Context {
            manifest,
            platform,
            arch,
            home_folder,
            cargo_target_folder,
            build_folder,
            app_snake_name,
            app_root_folder,
            project_root_folder,
            setup_resources_folder,

            cache_folder,
            output_folder,

            include,
            exclude,
            // app_root_folder,
            sdk,
            deps,
            tpl : Arc::new(Mutex::new(tpl)),
        };

        Ok(ctx)
    }

    pub async fn ensure_folders(&self) -> Result<()> {
        let folders = [&self.build_folder, &self.output_folder, &self.cache_folder];
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

    pub fn tpl(&self, text : &str) -> String {
        let tpl = self.tpl.lock().unwrap();
        tpl.transform(text)
    }

}
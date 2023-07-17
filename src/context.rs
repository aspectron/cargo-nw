use crate::prelude::*;
use async_std::path::Path;
use async_std::path::PathBuf;

#[derive(Default, Debug)]
pub struct Options {
    pub sdk: bool,
    pub nwjs_version_override: Option<String>,
    pub dry_run: bool,
    pub channel: Option<Channel>,
    pub confinement: Option<Confinement>,
}

// impl Default for Options {
//     fn default() -> Self {
//         Options {
//             sdk: false,
//             dry_run: false,
//             channel: None,
//             confinement: None,
//         }
//     }
// }

#[derive(Debug)]
pub struct Context {
    pub manifest: Manifest,
    // pub package_json : Option<PackageJson>,
    pub platform: Platform,
    pub arch: Architecture,

    pub home_folder: PathBuf,
    /// Cargo `target` folder
    pub cargo_target_folder: PathBuf,
    /// Source application folder
    pub app_root_folder: PathBuf,
    /// Project folder (nw.toml location). Can be the same as [`app_root_folder`]
    pub project_root_folder: PathBuf,
    /// Folder that contains setup resources
    pub setup_resources_folder: PathBuf,
    // pub app_snake_name : PathBuf,
    pub cache_folder: PathBuf,
    pub root_folder: PathBuf,
    pub build_folder: PathBuf,
    pub output_folder: PathBuf,
    pub temp_folder: PathBuf,
    pub dependencies_folder: PathBuf,

    pub app_snake_name: String,

    pub include: Option<Vec<CopyFilter>>,
    pub exclude: Option<Vec<CopyFilter>>,

    pub images: Images,

    pub sdk: bool,
    pub dry_run: bool,
    pub channel: Channel,
    pub confinement: Confinement,
    pub deps: Deps,
    pub tpl: Tpl,
}

impl Context {
    pub async fn create(
        location: Option<String>,
        output: Option<String>,
        platform: Platform,
        arch: Architecture,
        options: Options,
    ) -> Result<Context> {
        // println!("");

        let node_platform: NodePlatform = platform.clone().into();
        let nw_platform: NwPlatform = platform.clone().into();
        let mut tpl: Tpl = [
            ("PLATFORM", platform.to_string()),
            ("NODE-PLATFORM", node_platform.to_string()),
            ("NW-PLATFORM", nw_platform.to_string()),
            ("ARCH", arch.to_string()),
        ]
        .as_slice()
        .try_into()?;
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                tpl.set(&[("EXE",".exe")]);
                tpl.set(&[("BAT",".bat")]);
                tpl.set(&[("CMD",".cmd")]);
                tpl.set(&[("PS1",".ps1")]);
                tpl.set(&[("SH","")]);
            } else {
                tpl.set(&[("EXE","")]);
                tpl.set(&[("BAT","")]);
                tpl.set(&[("CMD","")]);
                tpl.set(&[("PS1","")]);
                tpl.set(&[("SH",".sh")]);
            }
        }

        let home_folder: PathBuf = home::home_dir().unwrap().into();
        let manifest_toml = Manifest::locate(location).await?;
        log_info!("Manifest", "`{}`", manifest_toml.to_str().unwrap());
        let manifest_folder = manifest_toml.parent().unwrap().to_path_buf();
        let manifest = Manifest::load(&manifest_toml).await?;
        let project_root = manifest_toml.parent().unwrap();

        tpl.set(&[
            ("NAME", manifest.application.name.as_str()),
            ("TITLE", manifest.application.title.as_str()),
            ("ORGANIZATION", manifest.application.organization.as_str()),
            ("SHORT", manifest.description.short.as_str()),
            ("LONG", manifest.description.long.as_str()),
            ("VERSION", manifest.application.version.as_str()),
        ]);

        [
            ("AUTHORS", &manifest.application.authors),
            ("COPYRIGHT", &manifest.application.copyright),
            ("TRADEMARKS", &manifest.application.trademarks),
            ("EULA", &manifest.application.eula),
            ("URL", &manifest.application.url),
        ]
        .iter()
        .for_each(|(k, v)| {
            if let Some(v) = v {
                tpl.set(&[(k, v.as_str())]);
            }
        });

        let root_folder  = if let Some(root) = &manifest.package.root {
            normalize(manifest_folder.join(root))?
        } else {
            search_upwards(&manifest_folder, "Cargo.toml")
                .await
                .map(|location| location.parent().unwrap().to_path_buf())
                .unwrap_or_else(|| manifest_folder.clone())
        };

        let app_snake_name = format!(
            "{}-{}-{}-{}",
            manifest.application.name, manifest.application.version, platform, arch
        );

        let cargo_target_folder = root_folder.join("target");
        let cargo_nw_target_folder = cargo_target_folder.join("nw");
        let build_folder = Path::new(&cargo_nw_target_folder)
            .join("build")
            .join(&app_snake_name);
        let cache_folder = Path::new(&cargo_nw_target_folder)
            .join("cache")
            .join(&app_snake_name);
        let dependencies_folder = Path::new(&cargo_nw_target_folder)
            .join("deps")
            .join(&app_snake_name);

        let output_folder = if let Some(output) = output.or_else(|| manifest.package.output.clone())
        {
            let output = Path::new(&output);
            if output.is_absolute() {
                output.to_owned()
            } else {
                project_root.join(output)
            }
        } else {
            Path::new(&cargo_nw_target_folder).join("setup")
        };
        let output_folder = normalize(&tpl.transform(output_folder.to_str().unwrap()))?;
        tpl.set(&[
            ("OUTPUT", output_folder.to_str().unwrap()),
            // ("$SETUP",output_folder.to_str().unwrap()),
        ]);

        let temp_folder = Path::new(&home_folder)
            .join(".cargo-nw")
            .join("temp")
            .join(&app_snake_name);
        tpl.set(&[("TEMP", temp_folder.to_str().unwrap())]);

        // let dependencies_folder = temp_folder.join("deps");

        let project_root_folder = project_root.to_path_buf();
        let app_root_folder = manifest
            .package
            .source
            .as_ref()
            .map(|root| project_root_folder.to_path_buf().join(root))
            .unwrap_or_else(|| project_root_folder.clone());

        let app_root_folder: PathBuf =
            match std::path::PathBuf::from(&app_root_folder).canonicalize() {
                Ok(path) => sanitize(path),
                Err(err) => {
                    return Err(format!(
                        "unable to locate application root folder `{}`: {}",
                        app_root_folder.display(),
                        err
                    )
                    .into());
                }
            };

        tpl.set(&[("SOURCE", app_root_folder.to_str().unwrap())]);

        let setup_resources_folder = manifest_folder.join(
            manifest
                .package
                .resources
                .as_ref()
                .unwrap_or(&"resources/setup".to_string()),
        );
        let sdk = manifest.nwjs.sdk.unwrap_or(options.sdk);
        let dry_run = options.dry_run;
        let snap = manifest.snap.clone().unwrap_or_default();
        let channel = options.channel.or(snap.channel).unwrap_or_default();
        let confinement = options.confinement.or(snap.confinement).unwrap_or_default();
        let deps = Deps::new(&platform, &arch, &manifest, sdk, options.nwjs_version_override);

        let include = manifest.package.include.clone(); //.unwrap_or(vec![]);
        let exclude = manifest.package.exclude.clone(); //.unwrap_or(vec![]);

        let images = manifest.images.clone().unwrap_or_default();

        log_info!("Target", "`{}`", output_folder.to_str().unwrap());

        let ctx = Context {
            manifest,
            // package_json,
            platform,
            arch,
            home_folder,
            app_snake_name,
            app_root_folder,
            project_root_folder,
            setup_resources_folder,

            root_folder,
            cargo_target_folder,
            build_folder,
            cache_folder,
            temp_folder,
            dependencies_folder,
            output_folder,

            include,
            exclude,

            images,
            // app_root_folder,
            sdk,
            dry_run,
            channel,
            confinement,
            deps,
            tpl, // : Arc::new(Mutex::new(tpl)),
        };

        Ok(ctx)
    }

    pub async fn ensure_folders(&self) -> Result<()> {
        let folders = [
            &self.build_folder,
            &self.output_folder,
            &self.cache_folder,
            &self.dependencies_folder,
        ];
        for folder in folders {
            if !std::path::Path::new(folder).exists() {
                std::fs::create_dir_all(folder)?;
            }
        }

        Ok(())
    }

    pub async fn clean_dependencies(&self) -> Result<()> {
        if self.dependencies_folder.exists().await {
            log_info!("Cleaning", "`{}`", self.dependencies_folder.display());
            async_std::fs::remove_dir_all(&self.dependencies_folder).await?;
        }
        Ok(())
    }
    pub async fn clean(&self) -> Result<()> {
        if self.build_folder.exists().await {
            log_info!("Cleaning", "`{}`", self.build_folder.display());
            async_std::fs::remove_dir_all(&self.build_folder).await?;
        }
        if self.cache_folder.exists().await {
            log_info!("Cleaning", "`{}`", self.cache_folder.display());
            async_std::fs::remove_dir_all(&self.cache_folder).await?;
        }
        Ok(())
    }

    pub fn tpl(&self) -> Tpl {
        self.tpl.clone()
    }

    pub async fn update_package_json(&self, target_folder : &Path) -> Result<()> {
        if self.manifest.package.update_package_json.unwrap_or(false) {
            log_info!("Manifest","Updating package.json manifest");
            let path = target_folder.join("package.json");
            let mut package_json = PackageJson::try_load(&path)?;
            package_json.version = Some(self.manifest.application.version.clone());
            package_json.name = self.manifest.application.name.clone();
            package_json.description = Some(self.manifest.description.short.clone());
            package_json.try_store(&path).await?;
        }
        Ok(())
    }
}
